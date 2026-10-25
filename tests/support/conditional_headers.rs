//! Fixture-only lexical edits, never a parser or an execution fallback.

use nucleoid::lang::ast::lexer::{Keyword, Spanned, Token, tokenize};

pub const REWRITES: &[(&str, &str)] = &[
    ("spec", "Nucleoid selects direct conditional alternatives"),
    (
        "spec",
        "Nucleoid keeps direct conditional class alternatives reactive",
    ),
    (
        "spec",
        "Nucleoid preserves direct alternative evaluation order",
    ),
    (
        "spec",
        "Nucleoid preserves direct alternative scope and return flow",
    ),
    ("spec", "Nucleoid rolls back failed direct alternatives"),
    (
        "reference",
        "Nucleoid routes records through direct alternatives",
    ),
    ("readme", "Conditional alternatives omit the second keyword"),
    (
        "examples",
        "Conditional alternatives keep frozen classifiers and live inputs separate",
    ),
];

fn suite_boundary(tokens: &[Spanned], start: usize) -> Option<usize> {
    let mut delimiters = Vec::new();
    for (index, spanned) in tokens.iter().enumerate().skip(start) {
        match &spanned.token {
            token @ (Token::LeftParen | Token::LeftBracket | Token::LeftBrace) => {
                delimiters.push(token);
            }
            token @ (Token::RightParen | Token::RightBracket | Token::RightBrace) => {
                if !matches!(
                    (delimiters.pop(), token),
                    (Some(Token::LeftParen), Token::RightParen)
                        | (Some(Token::LeftBracket), Token::RightBracket)
                        | (Some(Token::LeftBrace), Token::RightBrace)
                ) {
                    return None;
                }
            }
            Token::Colon | Token::Newline | Token::Indent if delimiters.is_empty() => {
                let body = &tokens[index + usize::from(spanned.token == Token::Colon)..];
                return (matches!(body[0].token, Token::Newline | Token::Indent)
                    && body
                        .iter()
                        .find(|token| !matches!(token.token, Token::Newline | Token::Semicolon))
                        .is_some_and(|token| token.token == Token::Indent))
                .then_some(index);
            }
            Token::Semicolon | Token::Dedent | Token::Eof if delimiters.is_empty() => {
                return None;
            }
            _ => {}
        }
    }
    None
}

fn headers(tokens: &[Spanned]) -> impl Iterator<Item = usize> + '_ {
    tokens.iter().enumerate().filter_map(|(index, spanned)| {
        match spanned.token {
            Token::Keyword(Keyword::If) => {}
            Token::Keyword(Keyword::Else)
                if tokens[index + 1].token != Token::Keyword(Keyword::If) => {}
            _ => return None,
        }
        let boundary = suite_boundary(tokens, index + 1)?;
        // Only condition-bearing headers belong to this revision. A final
        // Else immediately meets its colon/layout and must remain byte-exact.
        (boundary > index + 1).then_some(boundary)
    })
}

/// Remove only condition-bearing header colons with genuine indentation.
/// Keep unconditional final Else headers, inline/braced headers, literal/comment
/// text, annotations, object fields and slices byte-exact.
pub fn omit(source: &str) -> (String, usize) {
    let tokens = tokenize(source).unwrap();
    let mut edits = Vec::new();
    for index in headers(&tokens) {
        if tokens[index].token != Token::Colon {
            continue;
        }
        let position = tokens[index].position;
        let mut offset = 0;
        for (line, text) in source.split_inclusive('\n').enumerate() {
            if line + 1 == position.line {
                offset += text.char_indices().nth(position.column - 1).unwrap().0;
                break;
            }
            offset += text.len();
        }
        edits.push(offset);
    }
    let count = edits.len();
    let mut result = source.to_string();
    for offset in edits.into_iter().rev() {
        assert_eq!(result.remove(offset), ':');
    }
    (result, count)
}

/// Restore only condition-bearing headers in the eight selected, sealed,
/// uncommented fixtures for a later proposal's ORIGINAL historical byte seal.
/// Keep final Else headers unchanged; never refresh the seal. This inverse is
/// deliberately not a general source rewriter.
pub fn historical_source(document: &str, title: &str, source: &str) -> String {
    let Some(fixture) = REWRITES.iter().position(|id| *id == (document, title)) else {
        return source.to_string();
    };
    let tokens = tokenize(source).unwrap();
    let mut edits = Vec::new();
    for index in headers(&tokens) {
        assert_eq!(
            tokens[index].token,
            Token::Newline,
            "{title}: sealed bare header"
        );
        let line = tokens[index].position.line;
        let mut offset = 0;
        for (number, text) in source.split_inclusive('\n').enumerate() {
            if number + 1 == line {
                // These exact eight accepted fixtures have no header comments
                // or trailing whitespace. The unchanged byte seal enforces it.
                offset += text.trim_end_matches(['\r', '\n']).len();
                break;
            }
            offset += text.len();
        }
        edits.push(offset);
    }
    let count = edits.len();
    assert_eq!(count, [3, 3, 5, 3, 4, 3, 2, 6][fixture], "{title}");
    let mut original = source.to_string();
    for offset in edits.into_iter().rev() {
        original.insert(offset, ':');
    }
    assert_eq!(
        omit(&original),
        (source.to_string(), count),
        "{title}: exact colon-only inverse"
    );
    let actual = nucleoid::lang::ast::parse_program(source).unwrap();
    let accepted = nucleoid::lang::ast::parse_program(&original).unwrap();
    assert_eq!(actual.statements, accepted.statements, "{title}: full AST");
    assert_eq!(actual.positions, accepted.positions, "{title}: positions");
    original
}
