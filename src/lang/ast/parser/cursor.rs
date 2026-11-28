use super::Parser;
use crate::error::{Error, Position, Result};
use crate::lang::ast::lexer::{Keyword, Token};

impl Parser {
    pub(super) fn peek(&self) -> &Token {
        self.tokens
            .get(self.position)
            .map_or(&Token::Eof, |spanned| &spanned.token)
    }

    pub(super) fn peek_at(&self, offset: usize) -> &Token {
        self.tokens
            .get(self.position + offset)
            .map_or(&Token::Eof, |spanned| &spanned.token)
    }

    pub(super) fn advance(&mut self) -> Token {
        let token = self
            .tokens
            .get(self.position)
            .map_or(Token::Eof, |spanned| spanned.token.clone());

        if self.position < self.tokens.len() {
            self.position += 1;
        }

        token
    }

    /// Where the parser is looking.
    pub(super) fn here(&self) -> Position {
        self.tokens
            .get(self.position)
            .or_else(|| self.tokens.last())
            .map_or_else(|| Position::new(1, 1), |spanned| spanned.position)
    }

    /// A syntax error placed where the parser is looking.
    pub(super) fn error(&self, message: impl Into<String>) -> Error {
        Error::syntax(message).at(self.here())
    }

    pub(super) fn check(&self, token: &Token) -> bool {
        self.peek() == token
    }

    pub(super) fn eat(&mut self, token: &Token) -> bool {
        if self.check(token) {
            self.position += 1;
            true
        } else {
            false
        }
    }

    pub(super) fn expect(&mut self, token: &Token) -> Result<()> {
        if self.eat(token) {
            Ok(())
        } else {
            Err(self.error(format!(
                "Expected {} but found {}",
                token.describe(),
                self.peek().describe()
            )))
        }
    }

    /// Consumes either arrow spelling, `=>` or `->`.
    pub(super) fn eat_arrow(&mut self) -> bool {
        if self.peek().is_arrow() {
            self.position += 1;
            true
        } else {
            false
        }
    }

    /// Requires either arrow spelling, naming the canonical one when absent.
    pub(super) fn expect_arrow(&mut self) -> Result<()> {
        if self.eat_arrow() {
            Ok(())
        } else {
            self.expect(&Token::Arrow)
        }
    }

    pub(super) fn eat_keyword(&mut self, keyword: Keyword) -> bool {
        if self.peek() == &Token::Keyword(keyword) {
            self.position += 1;
            true
        } else {
            false
        }
    }

    pub(super) fn skip_separators(&mut self) {
        while matches!(
            self.peek(),
            Token::Newline | Token::Semicolon | Token::Indent | Token::Dedent
        ) {
            self.position += 1;
        }
    }

    pub(super) fn skip_newlines(&mut self) {
        while matches!(self.peek(), Token::Newline | Token::Semicolon) {
            self.position += 1;
        }
    }
}
