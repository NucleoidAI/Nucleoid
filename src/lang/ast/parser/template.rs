use std::ops::Range;

use super::{ExpressionContext, Parser};
use crate::error::{Error, Position, Result};
use crate::lang::ast::lexer::{excerpt, interpolation_end};
use crate::lang::ast::{Expr, TemplatePart};

fn template_position(relative: Position, origin: Position, prefix: &str) -> Position {
    let newlines = prefix.bytes().filter(|byte| *byte == b'\n').count();
    let column = match prefix.rsplit_once('\n') {
        Some((_, tail)) => tail.chars().count() + 1,
        None => origin.column + 1 + prefix.chars().count(),
    };
    Position::new(
        origin.line + newlines + relative.line - 1,
        if relative.line == 1 {
            column + relative.column - 1
        } else {
            relative.column
        },
    )
}

/// Secondary parsers report relative positions; diagnostics retain the location
/// in the caller's original template, including Unicode and newlines.
fn template_error(error: Error, origin: Position, prefix: &str) -> Error {
    let Some(relative) = error.position() else {
        return error;
    };
    error.at(template_position(relative, origin, prefix))
}

fn interpolation_close(raw: &str, start: usize, origin: Position) -> Result<Option<usize>> {
    let body = raw.get(start..).unwrap_or_default();
    Ok(interpolation_end(body)
        .map_err(|error| template_error(error, origin, excerpt(raw, 0..start)))?
        .map(|offset| start + offset))
}

impl Parser {
    pub(super) fn template(&mut self, raw: &str) -> Result<Expr> {
        let origin = self.previous_position();
        let mut parts = Vec::new();
        let mut literal = String::new();
        let mut characters = raw.char_indices().peekable();

        while let Some((index, character)) = characters.next() {
            if character == '\\' {
                if let Some((_, escaped)) = characters.next() {
                    literal.push(match escaped {
                        'n' => '\n',
                        't' => '\t',
                        other => other,
                    });
                }
                continue;
            }

            if character != '$' || characters.peek().map(|(_, next)| *next) != Some('{') {
                literal.push(character);
                continue;
            }

            let start = index + 2;
            let Some(end) = interpolation_close(raw, start, origin)? else {
                return Err(self.error("Unterminated template expression"));
            };

            if !literal.is_empty() {
                parts.push(TemplatePart::Literal(std::mem::take(&mut literal)));
            }

            parts.push(TemplatePart::Expression(self.interpolation(
                raw,
                start..end,
                origin,
            )?));

            while let Some((offset, _)) = characters.peek() {
                if *offset <= end {
                    characters.next();
                } else {
                    break;
                }
            }
        }

        if !literal.is_empty() {
            parts.push(TemplatePart::Literal(literal));
        }

        Ok(Expr::Template(parts))
    }

    pub(super) fn previous_position(&self) -> Position {
        self.position
            .checked_sub(1)
            .and_then(|previous| self.tokens.get(previous))
            .map_or_else(|| self.here(), |spanned| spanned.position)
    }

    fn interpolation(&mut self, raw: &str, span: Range<usize>, origin: Position) -> Result<Expr> {
        let prefix = excerpt(raw, 0..span.start);
        // Trailing tokens after an interpolated expression are ignored,
        // so a top-level `lambda` keeps its accepted name reading here
        // and a trailing Python-style `if` stays ignored.
        let (expression, misplaced_every) = self.interpolated_expression(excerpt(raw, span));
        self.adopt(misplaced_every.map(|relative| template_position(relative, origin, prefix)));
        expression.map_err(|error| template_error(error, origin, prefix))
    }

    fn interpolated_expression(&self, source: &str) -> (Result<Expr>, Option<Position>) {
        let fresh = match Parser::from_source(source) {
            Ok(fresh) => fresh,
            Err(error) => return (Err(error), None),
        };
        let mut parser = Parser {
            words: self.words,
            lists: self.lists,
            in_class: self.in_class,
            dots: self.dots,
            receiver: self.receiver.clone(),
            if_statements: self.if_statements,
            ..fresh
        };
        let expression = parser.leading_expression(ExpressionContext::Interpolation);
        (expression, parser.misplaced_every)
    }
}
