use super::{ArgumentContext, MAX_CHAIN, Parser};
use crate::error::{Error, Position, Result};
use crate::lang::ast::lexer::{EXPECTED_VERSION_SELECTOR, Token};
use crate::lang::ast::{Expr, Function, UnaryOp, VersionSelector};

impl Parser {
    pub(super) fn postfix(&mut self) -> Result<Expr> {
        let mut expression = self.primary()?;
        let mut chain = 0;

        loop {
            chain += 1;

            if chain > MAX_CHAIN {
                return Err(self.error("Expression has too many parts"));
            }

            match self.peek() {
                Token::At => {
                    self.advance();
                    expression = Expr::Version {
                        reference: Box::new(expression),
                        selector: self.version_selector()?,
                    };
                }
                Token::Dot => {
                    self.advance();
                    expression = Expr::Member {
                        object: Box::new(expression),
                        property: self.member_property()?,
                    };
                }
                Token::LeftBracket => {
                    self.advance();
                    expression = self.subscript(expression)?;
                }
                Token::LeftParen => {
                    self.advance();
                    // Only direct names can construct classes. Even a
                    // reserved helper name may name a class on the existing
                    // assignment-instantiation route; actual call routing
                    // decides whether a bare assignment is a label.
                    let context = if matches!(&expression, Expr::Identifier(_)) {
                        ArgumentContext::PotentialConstructor
                    } else {
                        ArgumentContext::Ordinary
                    };
                    let arguments = self.arguments(context)?;
                    expression = Expr::Call {
                        callee: Box::new(expression),
                        arguments,
                    };
                }
                _ => break,
            }
        }

        Ok(expression)
    }

    fn version_selector(&mut self) -> Result<VersionSelector> {
        let selector = match self.peek() {
            Token::VersionIndex(index) => Some(VersionSelector::Index(*index)),
            Token::Identifier(name) => VersionSelector::from_name(name),
            _ => None,
        };
        let Some(selector) = selector else {
            return Err(self.error(EXPECTED_VERSION_SELECTOR));
        };
        self.advance();
        Ok(selector)
    }

    fn member_property(&mut self) -> Result<String> {
        match self.advance() {
            Token::Identifier(name) => Ok(name),
            Token::Keyword(keyword) => Ok(keyword.as_str().to_string()),
            other => Err(self.error(format!(
                "Expected a property but found {}",
                other.describe()
            ))),
        }
    }

    fn subscript(&mut self, object: Expr) -> Result<Expr> {
        if self.eat(&Token::Colon) {
            let end = if self.check(&Token::RightBracket) {
                None
            } else {
                Some(Box::new(self.within(true, Self::expression)?))
            };
            self.expect(&Token::RightBracket)?;
            return Ok(Expr::Slice {
                object: Box::new(object),
                start: None,
                end,
            });
        }

        let index = self.clause_expression()?;

        if self.eat(&Token::Colon) {
            let end = if self.check(&Token::RightBracket) {
                None
            } else {
                Some(Box::new(self.within(true, Self::expression)?))
            };
            self.expect(&Token::RightBracket)?;
            return Ok(Expr::Slice {
                object: Box::new(object),
                start: Some(Box::new(index)),
                end,
            });
        }

        self.expect(&Token::RightBracket)?;

        Ok(Expr::Index {
            object: Box::new(object),
            index: Box::new(index),
        })
    }

    pub(super) fn arguments(&mut self, context: ArgumentContext) -> Result<Vec<Expr>> {
        let mut arguments = Vec::new();

        while !self.check(&Token::RightParen) {
            let length = self.selector_length();
            let argument = if self.starts_selector(length) {
                self.nested_expression(|parser| parser.selector(length))?
            } else {
                self.within(true, |parser| parser.argument(context))?
            };
            arguments.push(argument);

            if !self.eat(&Token::Comma) {
                break;
            }
        }

        self.expect(&Token::RightParen)?;
        Ok(arguments)
    }

    fn selector_length(&self) -> usize {
        let mut length = 0;
        while length < MAX_CHAIN
            && self.peek_at(2 * length) == &Token::Dot
            && matches!(
                self.peek_at(2 * length + 1),
                Token::Identifier(_) | Token::Keyword(_)
            )
        {
            length += 1;
        }
        length
    }

    fn starts_selector(&self, length: usize) -> bool {
        length == MAX_CHAIN
            || (length > 0 && matches!(self.peek_at(2 * length), Token::Comma | Token::RightParen))
    }

    fn selector(&mut self, length: usize) -> Result<Expr> {
        if length == MAX_CHAIN {
            return Err(Error::syntax("Expression has too many parts")
                .at(self.position_at(2 * (MAX_CHAIN - 1))));
        }
        let mut path = Vec::with_capacity(length);
        for _ in 0..length {
            self.advance();
            path.push(self.member_property()?);
        }
        debug_assert!(matches!(self.peek(), Token::Comma | Token::RightParen));
        Ok(Expr::Function(Function::selector(path)))
    }

    fn position_at(&self, offset: usize) -> Position {
        self.tokens
            .get(self.position + offset)
            .map_or_else(|| self.here(), |spanned| spanned.position)
    }

    pub(super) fn argument(&mut self, context: ArgumentContext) -> Result<Expr> {
        if self.starts_item_label(context) {
            return self.item_label();
        }
        let bare_label =
            matches!(self.peek(), Token::Identifier(_)) && self.peek_at(1) == &Token::Assign;
        let argument = self.expression()?;
        // Grouping is normally erased by primary(). Preserve it only when
        // erasing it could turn an explicit assignment into a constructor
        // label. Compound assignments need the same escape. All other
        // arguments, including member calls, keep their existing AST shape.
        if matches!(context, ArgumentContext::PotentialConstructor)
            && !bare_label
            && matches!(&argument, Expr::Assign { target, .. }
                if matches!(target.as_ref(), Expr::Identifier(_)))
        {
            Ok(Expr::Unary {
                operator: UnaryOp::Positional,
                operand: Box::new(argument),
            })
        } else {
            Ok(argument)
        }
    }
}
