use std::ops::Range;

use super::scanner::Scanned;
use super::token::Token;
use super::{Spanned, excerpt};
use crate::error::Position;

/// Turns byte offsets into line and column numbers.
pub(super) struct Lines<'a> {
    source: &'a str,
    starts: Vec<usize>,
}

impl<'a> Lines<'a> {
    pub(super) fn new(source: &'a str) -> Self {
        let mut starts = vec![0];

        for (index, byte) in source.bytes().enumerate() {
            if byte == b'\n' {
                starts.push(index + 1);
            }
        }

        Lines { source, starts }
    }

    pub(super) fn position(&self, offset: usize) -> Position {
        let offset = offset.min(self.source.len());

        let line = match self.starts.binary_search(&offset) {
            Ok(index) => index,
            Err(index) => index.saturating_sub(1),
        };

        debug_assert!(line < self.starts.len());
        let start = self.starts.get(line).copied().unwrap_or_default();
        let column = excerpt(self.source, start..offset).chars().count() + 1;

        Position::new(line + 1, column)
    }

    pub(super) fn end(&self) -> Position {
        self.position(self.source.len())
    }
}

/// Comments have zero indentation width. Only actual whitespace outside them
/// may indent the first code token; whitespace inside a block is not layout.
pub(super) fn line_position_without_comments(
    source: &str,
    start: usize,
    comments: &[Range<usize>],
) -> (bool, usize) {
    let line_start = excerpt(source, 0..start)
        .rfind('\n')
        .map_or(0, |index| index + 1);
    let first = comments.partition_point(|comment| comment.end <= line_start);
    let mut cursor = line_start;
    let mut indent = 0;
    let mut leading = true;
    for comment in comments.get(first..).unwrap_or_default() {
        if comment.start >= start {
            break;
        }
        let end = comment.start.max(cursor);
        for character in excerpt(source, cursor..end).chars() {
            leading &= character.is_whitespace();
            indent += if character == '\t' { 4 } else { 1 };
        }
        cursor = comment.end.min(start);
    }
    for character in excerpt(source, cursor..start).chars() {
        leading &= character.is_whitespace();
        indent += if character == '\t' { 4 } else { 1 };
    }
    (leading, indent)
}

pub(super) fn line_position(source: &str, start: usize) -> (bool, usize) {
    let before = excerpt(source, 0..start);
    let line = before.rsplit_once('\n').map_or(before, |(_, line)| line);

    let line_start = line.chars().all(char::is_whitespace);
    let indent = line
        .chars()
        .map(|character| if character == '\t' { 4 } else { 1 })
        .sum();

    (line_start, indent)
}

/// Turns the flat token stream into a block-structured one by inserting
/// `Indent`, `Dedent` and `Newline` tokens. Layout is suppressed inside `(` and
/// `[`, where line breaks carry no meaning.
pub(super) fn layout(scanned: Vec<Scanned>, end: Position) -> Vec<Spanned> {
    let mut layout = Layout::new();
    let mut depth = 0usize;

    for item in scanned {
        if item.token == Token::Newline {
            if depth == 0 {
                layout.pending_newline = true;
            }
            continue;
        }

        let depth_before = depth;

        match item.token {
            Token::LeftParen | Token::LeftBracket => depth += 1,
            Token::RightParen | Token::RightBracket => depth = depth.saturating_sub(1),
            _ => {}
        }

        if depth_before == 0 {
            layout.separate(&item);
        }

        layout.previous = item.position;
        layout.push(item.token, item.position);
    }

    layout.finish(end)
}

struct Layout {
    output: Vec<Spanned>,
    indents: Vec<usize>,
    pending_newline: bool,
    // An end-of-line belongs to the line it ends, not to the one that follows.
    previous: Position,
}

impl Layout {
    fn new() -> Self {
        Layout {
            output: Vec::new(),
            indents: vec![0],
            pending_newline: false,
            previous: Position::new(1, 1),
        }
    }

    fn push(&mut self, token: Token, position: Position) {
        self.output.push(Spanned { token, position });
    }

    fn separate(&mut self, item: &Scanned) {
        if item.line_start {
            if self.pending_newline && !self.output.is_empty() {
                self.push(Token::Newline, self.previous);
            }
            self.pending_newline = false;
            self.indent(item.indent, item.position);
        } else if self.pending_newline {
            self.push(Token::Newline, self.previous);
            self.pending_newline = false;
        }
    }

    fn indent(&mut self, indent: usize, position: Position) {
        let current = self.indents.last().copied().unwrap_or_default();

        if indent > current {
            self.indents.push(indent);
            self.push(Token::Indent, position);
        } else if indent < current {
            while self.indents.len() > 1 && self.indents.last().is_some_and(|&top| top > indent) {
                self.indents.pop();
                self.push(Token::Dedent, position);
            }
        }

        debug_assert_eq!(self.indents.first(), Some(&0));
        debug_assert!(self.indents.is_sorted_by(|lower, upper| lower < upper));
    }

    fn finish(mut self, end: Position) -> Vec<Spanned> {
        if self.pending_newline {
            self.push(Token::Newline, self.previous);
        }

        while self.indents.len() > 1 {
            self.indents.pop();
            self.push(Token::Dedent, end);
        }

        self.push(Token::Eof, end);
        self.output
    }
}
