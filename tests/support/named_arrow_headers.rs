//! Test-only spelling normalization of complete untyped bare named headers.
//! Arrow exports stay arrow-only; block operations require genuine indentation.
//! Never touch bodies, typed lists or other comma sites.
#![allow(dead_code)]

use nucleoid::Position;
use nucleoid::lang::ast::lexer::{Keyword, Spanned, Token, tokenize};

#[derive(Clone, Copy)]
enum Body {
    Arrow,
    IndentedBlock,
}

impl Body {
    fn qualifies(self, tokens: &[Spanned], end: usize) -> bool {
        let token = |offset: usize| tokens.get(offset).map(|spanned| &spanned.token);
        match self {
            Self::Arrow => token(end) == Some(&Token::Arrow),
            Self::IndentedBlock => {
                token(end) == Some(&Token::Colon)
                    && matches!(token(end + 1), Some(Token::Newline | Token::Indent))
                    && tokens[end + 1..]
                        .iter()
                        .find(|spanned| !matches!(spanned.token, Token::Newline | Token::Semicolon))
                        .is_some_and(|spanned| spanned.token == Token::Indent)
            }
        }
    }
}

fn byte_at(source: &str, position: Position) -> usize {
    let start = source
        .split_inclusive('\n')
        .take(position.line - 1)
        .map(str::len)
        .sum::<usize>();
    start
        + source[start..]
            .char_indices()
            .nth(position.column - 1)
            .unwrap()
            .0
}

fn header_edits(
    source: &str,
    remove_commas: bool,
    body: Body,
) -> Vec<(usize, usize, &'static str)> {
    let tokens = tokenize(source).unwrap();
    let mut edits = Vec::new();
    for (start, token) in tokens.iter().enumerate() {
        if token.token != Token::Keyword(Keyword::Def)
            || !matches!(
                tokens.get(start + 1).map(|next| &next.token),
                Some(Token::Identifier(_))
            )
        {
            continue;
        }
        let mut cursor = start + 2;
        let mut parameters = Vec::new();
        let mut commas = Vec::new();
        loop {
            if !matches!(
                tokens.get(cursor).map(|next| &next.token),
                Some(Token::Identifier(_))
            ) {
                break;
            }
            parameters.push(cursor);
            cursor += 1;
            if remove_commas
                && matches!(
                    tokens.get(cursor).map(|next| &next.token),
                    Some(Token::Comma)
                )
            {
                commas.push(cursor);
                cursor += 1;
                continue;
            }
            if !remove_commas
                && matches!(
                    tokens.get(cursor).map(|next| &next.token),
                    Some(Token::Identifier(_))
                )
            {
                continue;
            }
            if parameters.len() >= 2 && body.qualifies(&tokens, cursor) {
                if remove_commas {
                    for comma in commas {
                        let byte = byte_at(source, tokens[comma].position);
                        let identifier_byte =
                            |byte: u8| byte.is_ascii_alphanumeric() || byte == b'_';
                        let boundary = identifier_byte(source.as_bytes()[byte - 1])
                            && identifier_byte(source.as_bytes()[byte + 1]);
                        edits.push((byte, byte + 1, if boundary { " " } else { "" }));
                    }
                } else {
                    for &parameter in &parameters[..parameters.len() - 1] {
                        let Token::Identifier(name) = &tokens[parameter].token else {
                            unreachable!()
                        };
                        let byte = byte_at(source, tokens[parameter].position) + name.len();
                        edits.push((byte, byte, ","));
                    }
                }
            }
            break;
        }
    }
    edits
}

fn rewrite(source: &str, remove_commas: bool, body: Body) -> String {
    let mut result = source.to_string();
    let edits = header_edits(source, remove_commas, body);
    for (start, end, replacement) in edits.into_iter().rev() {
        result.replace_range(start..end, replacement);
    }
    result
}

pub fn comma_free_headers(source: &str) -> String {
    rewrite(source, true, Body::Arrow)
}

pub fn legacy_headers(source: &str) -> String {
    rewrite(source, false, Body::Arrow)
}

pub fn comma_free_block_headers(source: &str) -> String {
    rewrite(source, true, Body::IndentedBlock)
}

pub fn legacy_block_headers(source: &str) -> String {
    rewrite(source, false, Body::IndentedBlock)
}

/// Exact eligible byte edits, also used by the source-only migration audit.
pub fn block_comma_edits(source: &str) -> Vec<(usize, usize, &'static str)> {
    header_edits(source, true, Body::IndentedBlock)
}

/// Restore only the top-level JSONL code string, preserving every non-code byte.
/// Frozen-history callers still compare their original raw byte fingerprints.
pub fn legacy_block_training_bytes(bytes: &[u8]) -> Vec<u8> {
    fn whitespace(line: &str, mut cursor: usize) -> usize {
        while line
            .as_bytes()
            .get(cursor)
            .is_some_and(u8::is_ascii_whitespace)
        {
            cursor += 1;
        }
        cursor
    }

    fn value(line: &str, start: usize) -> (serde_json::Value, usize) {
        let mut stream =
            serde_json::Deserializer::from_str(&line[start..]).into_iter::<serde_json::Value>();
        let value = stream.next().unwrap().unwrap();
        (value, start + stream.byte_offset())
    }

    let mut result = String::new();
    for line in std::str::from_utf8(bytes).unwrap().split_inclusive('\n') {
        let original: serde_json::Value = serde_json::from_str(line).unwrap();
        let code = original["code"].as_str().expect("top-level code string");
        let restored = legacy_block_headers(code);
        if restored == code {
            result.push_str(line);
            continue;
        }

        let actual = nucleoid::lang::ast::parse(code).unwrap();
        let expected = nucleoid::lang::ast::parse(&restored).unwrap();
        assert_eq!(
            actual, expected,
            "restoration must preserve the complete AST"
        );
        assert_eq!(
            nucleoid::lang::ast::generator::generate_all(&actual),
            nucleoid::lang::ast::generator::generate_all(&expected)
        );

        let mut cursor = whitespace(line, 0);
        assert_eq!(line.as_bytes()[cursor], b'{');
        cursor += 1;
        let mut code_range = None;
        loop {
            cursor = whitespace(line, cursor);
            if line.as_bytes()[cursor] == b'}' {
                break;
            }
            let (key, end) = value(line, cursor);
            let key = key.as_str().expect("JSON field name");
            cursor = whitespace(line, end);
            assert_eq!(line.as_bytes()[cursor], b':');
            let start = whitespace(line, cursor + 1);
            let (field, end) = value(line, start);
            if key == "code" {
                assert!(
                    code_range.replace(start..end).is_none(),
                    "duplicate code field"
                );
                assert_eq!(field.as_str(), Some(code));
            }
            cursor = whitespace(line, end);
            match line.as_bytes()[cursor] {
                b',' => cursor += 1,
                b'}' => break,
                other => panic!("invalid JSON field separator {other}"),
            }
        }
        let range = code_range.expect("exactly one top-level code field");
        // Changed canonical records have canonical JSON string escapes. Never
        // silently reserialize an arbitrary record or normalize its whitespace.
        assert_eq!(&line[range.clone()], serde_json::to_string(code).unwrap());
        let mut rewritten = line.to_string();
        rewritten.replace_range(range, &serde_json::to_string(&restored).unwrap());
        let mut before_fields = original;
        let mut after_fields: serde_json::Value = serde_json::from_str(&rewritten).unwrap();
        before_fields
            .as_object_mut()
            .unwrap()
            .remove("code")
            .unwrap();
        after_fields
            .as_object_mut()
            .unwrap()
            .remove("code")
            .unwrap();
        assert_eq!(
            before_fields, after_fields,
            "non-code fields must remain exact"
        );
        result.push_str(&rewritten);
    }
    result.into_bytes()
}
