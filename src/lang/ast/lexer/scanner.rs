use std::ops::Range;

use logos::Logos;

use super::indentation::{Lines, line_position, line_position_without_comments};
use super::raw::Raw;
use super::token::Token;
use super::{EXPECTED_VERSION_SELECTOR, excerpt};
use crate::error::{Error, Position, Result};

pub(super) struct Scanned {
    pub(super) token: Token,
    pub(super) position: Position,
    pub(super) line_start: bool,
    pub(super) indent: usize,
}

pub(super) fn scan_until<'a>(
    source: &'a str,
    lines: &'a Lines<'a>,
    mut stop: impl FnMut(&Token) -> bool,
) -> Result<(Vec<Scanned>, Option<usize>)> {
    let mut lexer = Raw::lexer(source);
    let mut scanner = Scanner::new(source, lines);

    while let Some(next) = lexer.next() {
        let span = lexer.span();

        let raw = next.map_err(|()| {
            Error::syntax(format!(
                "Unexpected character '{}'",
                excerpt(source, span.clone())
            ))
            .at(lines.position(span.start))
        })?;

        let token = match raw {
            Raw::Number(_) if scanner.follows_at() => version_index(source, &span, lines)?,
            slash @ (Raw::Slash | Raw::SlashAssign) => {
                if let Some(end) = scanner.comment(span.start)? {
                    lexer.bump(end - span.end);
                    continue;
                }
                match scanner.regex(span.start) {
                    Some((pattern, end)) => {
                        lexer.bump(end - span.end);
                        Token::Regex(pattern)
                    }
                    None => slash.plain(),
                }
            }
            other => other.plain(),
        };

        if stop(&token) {
            return Ok((scanner.scanned, Some(span.start)));
        }
        scanner.record(token, span.start);
    }

    Ok((scanner.scanned, None))
}

struct Scanner<'a> {
    source: &'a str,
    lines: &'a Lines<'a>,
    scanned: Vec<Scanned>,
    comments: Vec<Range<usize>>,
    layout_depth: usize,
    // Keep suppressed trivia runs from repeatedly searching past old newlines.
    last_non_newline: Option<usize>,
}

impl<'a> Scanner<'a> {
    fn new(source: &'a str, lines: &'a Lines<'a>) -> Self {
        Scanner {
            source,
            lines,
            scanned: Vec::new(),
            comments: Vec::new(),
            layout_depth: 0,
            last_non_newline: None,
        }
    }

    fn previous(&self) -> Option<&Scanned> {
        if self.layout_depth > 0 {
            self.last_non_newline
                .and_then(|index| self.scanned.get(index))
        } else {
            self.scanned.last()
        }
    }

    fn follows_at(&self) -> bool {
        self.previous()
            .is_some_and(|previous| previous.token == Token::At)
    }

    fn comment(&mut self, start: usize) -> Result<Option<usize>> {
        let remainder = self.source.get(start..).unwrap_or_default();
        let end = if remainder.starts_with("//") {
            start + remainder.find('\n').unwrap_or(remainder.len())
        } else if let Some(body) = remainder.strip_prefix("/*") {
            let closing = body.find("*/").ok_or_else(|| {
                Error::syntax("Unterminated block comment").at(self.lines.position(start))
            })?;
            start + 2 + closing + 2
        } else {
            return Ok(None);
        };
        // Keep physical line breaks for the ordinary layout
        // pass, but never count comment text as indentation.
        for (offset, byte) in excerpt(self.source, start..end).bytes().enumerate() {
            if byte == b'\n' {
                self.scanned.push(Scanned {
                    token: Token::Newline,
                    position: self.lines.position(start + offset),
                    line_start: false,
                    indent: 0,
                });
            }
        }
        self.comments.push(start..end);
        Ok(Some(end))
    }

    fn regex(&self, start: usize) -> Option<(String, usize)> {
        let starts_regex = self
            .previous()
            .is_none_or(|previous| previous.token.precedes_regex());
        if !starts_regex {
            return None;
        }
        // Logos may have consumed `/=`. In an operand position,
        // that `=` still belongs to an existing regex such as /=/.
        let (pattern, consumed) = read_regex(self.source.get(start + 1..).unwrap_or_default())?;
        Some((pattern, start + 1 + consumed))
    }

    fn record(&mut self, token: Token, start: usize) {
        match token {
            Token::LeftParen | Token::LeftBracket => self.layout_depth += 1,
            Token::RightParen | Token::RightBracket => {
                self.layout_depth = self.layout_depth.saturating_sub(1);
            }
            _ => {}
        }
        let (line_start, indent) = if self.comments.is_empty() {
            line_position(self.source, start)
        } else {
            line_position_without_comments(self.source, start, &self.comments)
        };
        if token != Token::Newline {
            self.last_non_newline = Some(self.scanned.len());
        }
        self.scanned.push(Scanned {
            token,
            position: self.lines.position(start),
            line_start,
            indent,
        });
    }
}

fn version_index(source: &str, span: &Range<usize>, lines: &Lines<'_>) -> Result<Token> {
    let digits = excerpt(source, span.clone());
    let index = digits
        .bytes()
        .all(|byte| byte.is_ascii_digit())
        .then(|| digits.parse::<usize>().ok())
        .flatten();
    let malformed_suffix = source
        .get(span.end..)
        .and_then(|rest| rest.chars().next())
        .is_some_and(|character| character.is_ascii_alphanumeric() || character == '_');
    match index {
        Some(index) if !malformed_suffix => Ok(Token::VersionIndex(index)),
        _ => Err(Error::syntax(EXPECTED_VERSION_SELECTOR).at(lines.position(span.start))),
    }
}

/// Reads the body of a regular expression literal, given the source that
/// follows its opening slash. Returns the pattern and the bytes consumed.
fn read_regex(remainder: &str) -> Option<(String, usize)> {
    let mut pattern = String::new();
    let mut characters = remainder.char_indices();
    let mut in_class = false;

    while let Some((index, character)) = characters.next() {
        match character {
            '\\' => {
                pattern.push(character);
                let (_, escaped) = characters.next()?;
                pattern.push(escaped);
            }
            '\n' => return None,
            '[' => {
                in_class = true;
                pattern.push(character);
            }
            ']' => {
                in_class = false;
                pattern.push(character);
            }
            '/' if !in_class => {
                let mut consumed = index + character.len_utf8();

                for flag in remainder.get(consumed..).unwrap_or_default().chars() {
                    if flag.is_ascii_alphabetic() {
                        consumed += flag.len_utf8();
                    } else {
                        break;
                    }
                }

                return Some((pattern, consumed));
            }
            _ => pattern.push(character),
        }
    }

    None
}
