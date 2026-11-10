use super::{ExpressionContext, MAX_DESTRUCTURING_TARGETS, Parser, Pattern, Read, Target};
use crate::error::{Error, Result};
use crate::lang::ast::lexer::{Keyword, Token};
use crate::lang::ast::{Expr, Stmt};

impl Target {
    fn declaration(self, value: &Expr) -> Stmt {
        let object = Box::new(value.clone());
        Stmt::Assign {
            target: Expr::Identifier(self.name),
            value: match self.read {
                Read::Key(property) => Expr::Member { object, property },
                Read::Index(index) => Expr::Index {
                    object,
                    index: Box::new(Expr::Number(f64::from(index))),
                },
            },
        }
    }
}

impl Parser {
    pub(super) fn destructuring(&mut self) -> Result<Option<Vec<Stmt>>> {
        if !self.starts_statement() {
            return Ok(None);
        }
        let Some(pattern) = self.object_pattern().or_else(|| self.unpacking_pattern()) else {
            return Ok(None);
        };

        let start = self.position;
        self.position += pattern.length;
        let value = match self.nested_expression(Self::destructured_value) {
            Ok(value) if self.ends_line_statement() => value,
            _ => {
                self.position = start;
                return Ok(None);
            }
        };

        if let Some(target) = pattern.targets.get(MAX_DESTRUCTURING_TARGETS) {
            return Err(Error::syntax("Destructuring has too many targets").at(target.position));
        }

        Ok(Some(
            pattern
                .targets
                .into_iter()
                .map(|target| target.declaration(&value))
                .collect(),
        ))
    }

    fn destructured_value(&mut self) -> Result<Expr> {
        self.without_receiver(|parser| {
            parser.nested_expression_in(ExpressionContext::Ordinary, Self::conditional)
        })
    }

    pub(super) fn ends_line_statement(&self) -> bool {
        self.ends_line_statement_at(0)
    }

    pub(super) fn ends_line_statement_at(&self, offset: usize) -> bool {
        matches!(
            self.peek_at(offset),
            Token::Newline
                | Token::Semicolon
                | Token::Dedent
                | Token::RightBrace
                | Token::Eof
                | Token::Keyword(Keyword::Else | Keyword::Catch)
        )
    }

    fn target_at(&self, offset: usize, name: &str, read: Read) -> Target {
        Target {
            name: name.to_string(),
            read,
            position: self
                .tokens
                .get(self.position + offset)
                .map_or_else(|| self.here(), |spanned| spanned.position),
        }
    }

    fn object_pattern(&self) -> Option<Pattern> {
        if !matches!(
            (self.peek(), self.peek_at(1)),
            (Token::LeftBrace, Token::Identifier(_))
        ) {
            return None;
        }
        let mut targets = Vec::new();
        let mut offset = 1;
        loop {
            match (
                self.peek_at(offset),
                self.peek_at(offset + 1),
                self.peek_at(offset + 2),
            ) {
                (Token::RightBrace, _, _) if !targets.is_empty() => break,
                (Token::Identifier(key), Token::Colon, Token::Identifier(name)) => {
                    targets.push(self.target_at(offset + 2, name, Read::Key(key.clone())));
                    offset += 3;
                    match (self.peek_at(offset), self.peek_at(offset + 1)) {
                        (Token::Comma, _) => offset += 1,
                        (Token::RightBrace, _) | (Token::Identifier(_), Token::Colon) => {}
                        _ => return None,
                    }
                }
                (Token::Identifier(name), Token::Comma | Token::RightBrace, _) => {
                    targets.push(self.target_at(offset, name, Read::Key(name.clone())));
                    offset += 1;
                    if self.peek_at(offset) == &Token::Comma {
                        offset += 1;
                    }
                }
                _ => return None,
            }
        }
        (self.peek_at(offset + 1) == &Token::Assign).then_some(Pattern {
            targets,
            length: offset + 2,
        })
    }

    fn unpacking_pattern(&self) -> Option<Pattern> {
        if !matches!(
            (self.peek(), self.peek_at(1)),
            (Token::Identifier(_), Token::Comma)
        ) {
            return None;
        }
        let mut targets = Vec::new();
        let mut offset = 0;
        loop {
            match self.peek_at(offset) {
                Token::Identifier(name) => {
                    let index = u32::try_from(targets.len()).ok()?;
                    targets.push(self.target_at(offset, name, Read::Index(index)));
                    offset += 1;
                    match self.peek_at(offset) {
                        Token::Comma => offset += 1,
                        Token::Assign => break,
                        _ => return None,
                    }
                }
                Token::Assign => break,
                _ => return None,
            }
        }
        Some(Pattern {
            targets,
            length: offset + 1,
        })
    }
}
