use super::SourceTokens;
use super::indentation::{Lines, layout};
use super::scanner::scan_until;
use super::token::Token;
use crate::error::Result;

pub(in crate::lang::ast) fn tokenize_source(source: &str) -> Result<SourceTokens> {
    let lines = Lines::new(source);
    let (flat, _) = scan_until(source, &lines, |_| false)?;
    let mut line_breaks = Vec::new();
    let mut end_line = 1;
    for item in &flat {
        if item.token == Token::Newline {
            continue;
        }
        if item.position.line > end_line {
            line_breaks.push(item.position);
        }
        end_line = item.end_line;
    }
    Ok(SourceTokens {
        tokens: layout(flat, lines.end()),
        line_breaks,
    })
}
