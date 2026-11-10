use super::statement::heads_suppose;
use super::{LoopCondition, Parser};
use crate::error::Result;
use crate::lang::ast::lexer::{Keyword, Token};
use crate::lang::ast::{Expr, Stmt};

impl Parser {
    pub(super) fn if_statement(&mut self) -> Result<Stmt> {
        self.expect(&Token::Keyword(Keyword::If))?;
        self.condition_clause()
    }

    /// Explicit `else if` and qualified direct alternatives share the entire
    /// condition, suite and recursive alternate parser.
    fn condition_clause(&mut self) -> Result<Stmt> {
        self.if_statements += 1;
        let condition = self.clause_expression()?;
        // Explicit colons retain every accepted suite and diagnostic. Only a
        // real indented body can select the optional conditional spelling.
        let consequent = if self.check(&Token::Colon) || !self.has_indented_body(0) {
            self.suite()?
        } else {
            self.suite_body()?
        };

        let mut alternate = None;
        let checkpoint = self.position;
        self.skip_newlines();

        if self.eat_keyword(Keyword::Else) {
            if self.check(&Token::Keyword(Keyword::If)) {
                alternate = Some(Box::new(self.if_statement()?));
            } else if self.has_direct_condition_suite() {
                alternate = Some(Box::new(self.condition_clause()?));
            } else {
                let body = if self.has_indented_body(0) {
                    self.suite_body()?
                } else {
                    self.suite()?
                };
                alternate = Some(Box::new(Stmt::Block(body)));
            }
        } else {
            self.position = checkpoint;
        }

        debug_assert!(self.if_statements > 0);
        self.if_statements -= 1;
        Ok(Stmt::If {
            condition,
            consequent,
            alternate,
        })
    }

    /// Qualify a nonempty, balanced header without consuming tokens or parsing
    /// an expression. Only a top-level colon or real body layout may end it;
    /// colons in calls, slices and object literals still belong to the header,
    /// as does each top-level conditional's colon after its `?`.
    pub(super) fn has_direct_condition_suite(&self) -> bool {
        let mut delimiters = Vec::new();
        let mut conditionals = 0usize;
        for (offset, spanned) in self.tokens.iter().skip(self.position).enumerate() {
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
                        return false;
                    }
                }
                Token::Question if delimiters.is_empty() => conditionals += 1,
                Token::Identifier(_)
                    if delimiters.is_empty() && self.quantifier_at(offset).is_some() =>
                {
                    conditionals += 1;
                }
                Token::Colon if delimiters.is_empty() && conditionals > 0 => conditionals -= 1,
                Token::Colon if delimiters.is_empty() => {
                    return offset > 0 && self.has_indented_suite(offset);
                }
                Token::Newline | Token::Indent if delimiters.is_empty() => {
                    return offset > 0 && self.has_indented_body(offset);
                }
                Token::Semicolon | Token::ColonAssign | Token::Dedent | Token::Eof
                    if delimiters.is_empty() =>
                {
                    return false;
                }
                _ => {}
            }
        }
        false
    }

    pub(super) fn while_condition(&mut self) -> Option<Expr> {
        if !matches!(self.peek(), Token::Identifier(name) if name == "while")
            || self.starts_colon_statement_at(1)
        {
            return None;
        }

        let attempt = (self.position, self.depth);
        let outcome = if let Some(outcome) = self.while_conditions.get(&attempt) {
            outcome.clone()
        } else {
            let outcome = self.attempt_while_condition();
            self.while_conditions.insert(attempt, outcome.clone());
            outcome
        };
        let LoopCondition {
            condition,
            colon,
            misplaced_every,
        } = outcome?;
        self.position = colon;
        self.adopt(misplaced_every);
        Some(condition)
    }

    fn attempt_while_condition(&mut self) -> Option<LoopCondition> {
        let start = self.position;
        self.advance();
        let (clause, misplaced_every) = self.tentatively(Self::clause_before_colon);
        let outcome = clause
            .filter(|condition| !heads_suppose(condition))
            .map(|condition| LoopCondition {
                condition,
                colon: self.position,
                misplaced_every,
            });
        self.position = start;
        let colon = outcome.as_ref()?.colon;
        if self.words
            && self.accepted(Self::attempt_while_condition).is_none()
            && self.separates(start, colon)
        {
            return None;
        }
        outcome
    }

    pub(super) fn for_statement(&mut self) -> Result<Stmt> {
        self.expect(&Token::Keyword(Keyword::For))?;

        if let Some((key, value)) = self.entry_names() {
            let iterable = self.clause_expression()?;
            let body = self.suite()?;
            return Ok(Stmt::ForEntries {
                key,
                value,
                iterable,
                body,
            });
        }

        let variable = if matches!(self.peek(), Token::Identifier(_)) && self.has_indented_suite(1)
        {
            // Leave the named source and its complete suite to the existing
            // parsers. Nonqualifying prefixes retain the explicit-loop path.
            "it".to_string()
        } else {
            let variable = match self.advance() {
                Token::Identifier(name) => name,
                other => {
                    return Err(
                        self.error(format!("Expected a name but found {}", other.describe()))
                    );
                }
            };

            if !self.eat_keyword(Keyword::Of) && !self.eat_keyword(Keyword::In) {
                return Err(self.error("Expected 'of' in for statement"));
            }

            variable
        };

        let iterable = self.clause_expression()?;
        let body = self.suite()?;

        Ok(Stmt::For {
            variable,
            iterable,
            body,
        })
    }

    fn entry_names(&mut self) -> Option<(String, String)> {
        let (Token::Identifier(key), Token::Comma, Token::Identifier(value)) =
            (self.peek(), self.peek_at(1), self.peek_at(2))
        else {
            return None;
        };
        if !matches!(self.peek_at(3), Token::Keyword(Keyword::Of | Keyword::In)) {
            return None;
        }
        let names = (key.clone(), value.clone());
        self.position += 4;
        Some(names)
    }

    /// Recognizes only the real-indented colon path already accepted by
    /// `suite()`, without consuming tokens or skipping header separators.
    pub(super) fn has_indented_suite(&self, colon_offset: usize) -> bool {
        matches!(self.peek_at(colon_offset), Token::Colon)
            && self.has_indented_body(colon_offset + 1)
    }

    /// Qualify the complete layout prefix before consuming any of it. A
    /// same-line semicolon is not a newline, even if an indent follows later.
    pub(super) fn has_indented_body(&self, offset: usize) -> bool {
        matches!(self.peek_at(offset), Token::Newline | Token::Indent)
            && self
                .tokens
                .iter()
                .skip(self.position + offset)
                .find(|spanned| !matches!(spanned.token, Token::Newline | Token::Semicolon))
                .is_some_and(|spanned| spanned.token == Token::Indent)
    }

    pub(super) fn try_statement(&mut self) -> Result<Stmt> {
        self.expect(&Token::Keyword(Keyword::Try))?;
        // Explicit colons keep the original suite path and diagnostics.
        let body = if self.check(&Token::Colon) || !self.has_indented_body(0) {
            self.suite()?
        } else {
            self.suite_body()?
        };
        self.skip_newlines();

        if !self.eat_keyword(Keyword::Catch) {
            return Err(self.error("Expected 'catch' after 'try'"));
        }

        let parameter = match self.peek().clone() {
            Token::Identifier(name) => {
                self.advance();
                name
            }
            Token::LeftParen => {
                self.advance();
                let name = match self.advance() {
                    Token::Identifier(name) => name,
                    other => {
                        return Err(
                            self.error(format!("Expected a name but found {}", other.describe()))
                        );
                    }
                };
                self.expect(&Token::RightParen)?;
                name
            }
            _ => "error".to_string(),
        };

        let catch = if self.check(&Token::Colon) || !self.has_indented_body(0) {
            self.suite()?
        } else {
            self.suite_body()?
        };

        Ok(Stmt::Try {
            body,
            parameter,
            catch,
        })
    }
}
