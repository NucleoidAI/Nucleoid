use super::statement::heads_suppose;
use super::{LoopCondition, Parser};
use crate::error::Result;
use crate::lang::ast::lexer::{Keyword, Token};
use crate::lang::ast::{Expr, Stmt};

const DEFAULT_PARAMETER: &str = "error";

const REPEAT: &str = "repeat";

const UNTIL: &str = "until";

const MISSING_UNTIL: &str = "Expected 'until' after 'repeat'";

fn is_identifier(token: &Token, name: &str) -> bool {
    matches!(token, Token::Identifier(identifier) if identifier == name)
}

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
        self.loop_header("while")
    }

    pub(super) fn repeat_count(&mut self) -> Option<Expr> {
        self.loop_header(REPEAT)
    }

    pub(super) fn starts_repeat_until(&self) -> bool {
        is_identifier(self.peek(), REPEAT)
            && self.peek_at(1) == &Token::Colon
            && !self.is_type_declaration_at(0)
    }

    pub(super) fn repeat_until_statement(&mut self) -> Result<Stmt> {
        self.advance();
        let body = self.suite()?;
        self.until_line()?;
        let condition = self.clause_expression()?;
        match self.peek() {
            Token::Newline | Token::Semicolon | Token::Dedent | Token::RightBrace | Token::Eof => {
                Ok(Stmt::RepeatUntil { body, condition })
            }
            other => Err(self.error(format!("Unexpected {}", other.describe()))),
        }
    }

    fn until_line(&mut self) -> Result<()> {
        if let Some(offset) = self.line_start() {
            self.position += offset;
            if is_identifier(self.peek(), UNTIL) {
                self.advance();
                return Ok(());
            }
        }
        Err(self.error(MISSING_UNTIL))
    }

    fn loop_header(&mut self, word: &str) -> Option<Expr> {
        if !matches!(self.peek(), Token::Identifier(name) if name == word)
            || self.starts_colon_statement_at(1)
        {
            return None;
        }

        let attempt = (self.position, self.depth);
        let outcome = if let Some(outcome) = self.while_conditions.get(&attempt) {
            outcome.clone()
        } else {
            let outcome = self.attempt_loop_header();
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

    fn attempt_loop_header(&mut self) -> Option<LoopCondition> {
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
            && self.accepted(Self::attempt_loop_header).is_none()
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
        let body = self.clause_suite()?;
        self.skip_newlines();

        let parameter = self.clause_parameter()?;
        let catch = self.clause_suite()?;

        Ok(match self.finally_suite()? {
            Some(finally) => Stmt::TryFinally {
                body,
                parameter,
                catch,
                finally,
            },
            None => Stmt::Try {
                body,
                parameter,
                catch,
            },
        })
    }

    fn clause_suite(&mut self) -> Result<Vec<Stmt>> {
        if self.check(&Token::Colon) || !self.has_indented_body(0) {
            self.suite()
        } else {
            self.suite_body()
        }
    }

    fn clause_parameter(&mut self) -> Result<String> {
        if let Some(parameter) = self.except_header() {
            return Ok(parameter);
        }

        if !self.eat_keyword(Keyword::Catch) {
            return Err(self.error("Expected 'catch' after 'try'"));
        }

        Ok(match self.peek().clone() {
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
            _ => DEFAULT_PARAMETER.to_string(),
        })
    }

    fn except_header(&mut self) -> Option<String> {
        if !is_identifier(self.peek(), "except") {
            return None;
        }
        let (binder, colon) = if !is_identifier(self.peek_at(1), "Exception") {
            (DEFAULT_PARAMETER, 1)
        } else if !is_identifier(self.peek_at(2), "as") {
            (DEFAULT_PARAMETER, 2)
        } else if let Token::Identifier(name) = self.peek_at(3) {
            (name.as_str(), 4)
        } else {
            return None;
        };
        if self.peek_at(colon) != &Token::Colon {
            return None;
        }
        let parameter = binder.to_string();
        self.position += colon;
        Some(parameter)
    }

    fn finally_suite(&mut self) -> Result<Option<Vec<Stmt>>> {
        let Some(offset) = self.finally_clause() else {
            return Ok(None);
        };
        self.position += offset + 1;
        self.suite().map(Some)
    }

    fn finally_clause(&self) -> Option<usize> {
        let offset = self.line_start()?;
        let finally = is_identifier(self.peek_at(offset), "finally")
            && self.peek_at(offset + 1) == &Token::Colon;
        (finally && !self.is_type_declaration_at(offset)).then_some(offset)
    }

    fn line_start(&self) -> Option<usize> {
        let offset = self
            .tokens
            .iter()
            .skip(self.position)
            .take_while(|spanned| spanned.token == Token::Newline)
            .count();
        let starts_line = (self.position + offset)
            .checked_sub(1)
            .and_then(|previous| self.tokens.get(previous))
            .is_some_and(|previous| matches!(previous.token, Token::Newline | Token::Dedent));
        starts_line.then_some(offset)
    }
}
