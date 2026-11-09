mod indentation;
mod raw;
mod scanner;
mod token;

use std::ops::Range;

use crate::error::{Position, Result};
use indentation::{Lines, layout};
use scanner::scan_until;
pub use token::{Keyword, Token};

fn delimited(slice: &str) -> &str {
    let mut characters = slice.chars();
    characters.next();
    characters.next_back();
    characters.as_str()
}

pub(crate) fn excerpt(source: &str, range: Range<usize>) -> &str {
    debug_assert!(source.get(range.clone()).is_some());
    source.get(range).unwrap_or_default()
}

fn unescape(slice: &str) -> String {
    let inner = delimited(slice);
    let mut output = String::with_capacity(inner.len());
    let mut characters = inner.chars();

    while let Some(character) = characters.next() {
        if character != '\\' {
            output.push(character);
            continue;
        }

        match characters.next() {
            Some('n') => output.push('\n'),
            Some('t') => output.push('\t'),
            Some('r') => output.push('\r'),
            Some('0') => output.push('\0'),
            Some(other) => output.push(other),
            None => output.push('\\'),
        }
    }

    output
}

/// A token and where it came from.
#[derive(Debug, Clone, PartialEq)]
pub struct Spanned {
    pub token: Token,
    pub position: Position,
}

pub fn tokenize(source: &str) -> Result<Vec<Spanned>> {
    let lines = Lines::new(source);
    let (flat, _) = scan_until(source, &lines, |_| false)?;
    Ok(layout(flat, lines.end()))
}

/// Finds an interpolation's closing brace using the same literal/comment rules
/// as its expression parser, without lexing the template text that follows it.
pub(crate) fn interpolation_end(source: &str) -> Result<Option<usize>> {
    let mut depth = 0usize;
    let (_, end) = scan_until(source, &Lines::new(source), |token| {
        match token {
            Token::LeftBrace => depth += 1,
            Token::RightBrace if depth == 0 => return true,
            Token::RightBrace => depth -= 1,
            _ => {}
        }
        false
    })?;
    Ok(end)
}

#[cfg(test)]
mod tests;
