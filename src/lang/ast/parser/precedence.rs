use super::{ExpressionContext, MAX_CHAIN, Parser};
use crate::error::Result;
use crate::lang::ast::lexer::{Keyword, Token};
use crate::lang::ast::{BinaryOp, Expr, LogicalOp, Noun, UnaryOp};
use crate::reasoning::Stage;

impl Parser {
    pub(super) fn logical_or(&mut self) -> Result<Expr> {
        let mut left = self.logical_and()?;

        let mut chain = 0;

        while self.check(&Token::OrOr) || self.check(&Token::Keyword(Keyword::Or)) {
            self.advance();
            chain += 1;

            if chain > MAX_CHAIN {
                return Err(self.error("Expression has too many operands"));
            }

            let right = self.logical_and()?;
            left = Expr::Logical {
                operator: LogicalOp::Or,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn logical_and(&mut self) -> Result<Expr> {
        let mut left = self.equality()?;

        let mut chain = 0;

        while self.check(&Token::AndAnd) || self.check(&Token::Keyword(Keyword::And)) {
            self.advance();
            chain += 1;

            if chain > MAX_CHAIN {
                return Err(self.error("Expression has too many operands"));
            }

            let right = self.equality()?;
            left = Expr::Logical {
                operator: LogicalOp::And,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn equality(&mut self) -> Result<Expr> {
        let mut left = self.relational()?;

        let mut chain = 0;

        loop {
            let operator = match self.peek() {
                Token::Equal => BinaryOp::Equal,
                Token::NotEqual => BinaryOp::NotEqual,
                Token::StrictEqual => BinaryOp::StrictEqual,
                Token::StrictNotEqual => BinaryOp::StrictNotEqual,
                _ => match self.word_comparison() {
                    Some(operator) => operator,
                    None => break,
                },
            };

            self.advance();
            if operator == BinaryOp::IsNot {
                self.advance();
            }
            chain += 1;

            if chain > MAX_CHAIN {
                return Err(self.error("Expression has too many operands"));
            }

            let start = self.position;
            let right = self.relational()?;
            let word = self.position == start + 1 && matches!(right, Expr::Identifier(_));
            left = Expr::Binary {
                operator: match operator {
                    BinaryOp::Is if !word => BinaryOp::Equal,
                    BinaryOp::IsNot if !word => BinaryOp::NotEqual,
                    operator => operator,
                },
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn relational(&mut self) -> Result<Expr> {
        let mut left = self.additive()?;

        let mut chain = 0;

        loop {
            let operator = match self.peek() {
                Token::Less => BinaryOp::Less,
                Token::LessEqual => BinaryOp::LessEqual,
                Token::Greater => BinaryOp::Greater,
                Token::GreaterEqual => BinaryOp::GreaterEqual,
                _ => break,
            };

            self.advance();
            chain += 1;

            if chain > MAX_CHAIN {
                return Err(self.error("Expression has too many operands"));
            }

            let right = self.additive()?;
            left = Expr::Binary {
                operator,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn additive(&mut self) -> Result<Expr> {
        let mut left = self.multiplicative()?;

        let mut chain = 0;

        loop {
            let operator = match self.peek() {
                Token::Plus => BinaryOp::Add,
                Token::Minus => BinaryOp::Subtract,
                _ => break,
            };

            self.advance();
            chain += 1;

            if chain > MAX_CHAIN {
                return Err(self.error("Expression has too many operands"));
            }

            let right = self.multiplicative()?;
            left = Expr::Binary {
                operator,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn multiplicative(&mut self) -> Result<Expr> {
        let mut left = self.unary()?;

        let mut chain = 0;

        loop {
            let operator = match self.peek() {
                Token::Star => BinaryOp::Multiply,
                Token::Slash => BinaryOp::Divide,
                Token::Percent => BinaryOp::Modulo,
                _ => break,
            };

            self.advance();
            chain += 1;

            if chain > MAX_CHAIN {
                return Err(self.error("Expression has too many operands"));
            }

            let right = self.unary()?;
            left = Expr::Binary {
                operator,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    pub(super) fn unary(&mut self) -> Result<Expr> {
        if let Token::Identifier(name) = self.peek() {
            if let Some(stage) = Stage::from_name(name) {
                // An ordinary object value wins before the next explicit
                // field, even on the right of an operator or an arrow.
                let field_boundary = self.context == ExpressionContext::ObjectValue
                    && matches!(stage, Stage::Stated | Stage::Derived)
                    && matches!(self.peek_at(1), Token::Identifier(_))
                    && self.peek_at(2) == &Token::Colon;
                if self.starts_operand(1) && !field_boundary {
                    self.advance();
                    let operand = self.nested_expression(Self::unary)?;

                    return Ok(Expr::Reason {
                        stage,
                        source: Box::new(operand),
                    });
                }
            }
        }

        if let Some(noun) = self.noun_query() {
            self.position += 2;
            let operand = self.nested_expression(Self::unary)?;

            return Ok(Expr::Of {
                noun,
                operand: Box::new(operand),
            });
        }

        let operator = match self.peek() {
            Token::Bang | Token::Keyword(Keyword::Not) => UnaryOp::Not,
            Token::Minus => UnaryOp::Negate,
            Token::Plus => UnaryOp::Plus,
            Token::Keyword(Keyword::TypeOf) => UnaryOp::TypeOf,
            _ => return self.exponent(),
        };

        self.advance();
        let operand = self.nested_expression(Self::unary)?;

        Ok(Expr::Unary {
            operator,
            operand: Box::new(operand),
        })
    }

    fn exponent(&mut self) -> Result<Expr> {
        let base = self.postfix()?;
        if !self.eat(&Token::StarStar) {
            return Ok(base);
        }
        let exponent = self.nested_expression(Self::unary)?;
        Ok(Expr::Binary {
            operator: BinaryOp::Power,
            left: Box::new(base),
            right: Box::new(exponent),
        })
    }

    /// The noun of a `NOUN of OPERAND` query starting here (NUC 75): a
    /// noun-named list query, then `of`. Accepted syntax gives `of` no other
    /// place in an expression; a `for` header takes its loop name before any
    /// expression starts. A template interpolation keeps ignoring trailing
    /// tokens, so `${sum of xs}` still reads `sum`, as quantifiers and
    /// `lambda` stay names there.
    pub(super) fn noun_query(&self) -> Option<Noun> {
        if self.context == ExpressionContext::Interpolation
            || self.peek_at(1) != &Token::Keyword(Keyword::Of)
        {
            return None;
        }

        match self.peek() {
            Token::Identifier(name) => Noun::from_name(name),
            _ => None,
        }
    }

    fn starts_operand(&self, offset: usize) -> bool {
        matches!(
            self.peek_at(offset),
            Token::Identifier(_) | Token::ClassRef(_)
        )
    }
}
