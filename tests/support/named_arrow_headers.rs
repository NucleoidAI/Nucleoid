//! Test-only spelling normalization, strictly limited to complete untyped bare
//! named-arrow headers. Never touch bodies, typed lists or other comma sites.
#![allow(dead_code)]

use nucleoid::Position;
use nucleoid::lang::ast::lexer::{Keyword, Token, tokenize};

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

fn rewrite(source: &str, remove_commas: bool) -> String {
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
            if parameters.len() >= 2
                && matches!(
                    tokens.get(cursor).map(|next| &next.token),
                    Some(Token::Arrow)
                )
            {
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
    let mut result = source.to_string();
    for (start, end, replacement) in edits.into_iter().rev() {
        result.replace_range(start..end, replacement);
    }
    result
}

pub fn comma_free_headers(source: &str) -> String {
    rewrite(source, true)
}

pub fn legacy_headers(source: &str) -> String {
    rewrite(source, false)
}
