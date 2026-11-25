use super::Parser;
use crate::error::Result;
use crate::lang::ast::Expr;
use crate::lang::ast::lexer::Token;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum ListSeparators {
    Commas,
    PhysicalLines,
}

impl Parser {
    pub(super) fn list_literal(&mut self) -> Result<Expr> {
        let mut items = Vec::new();

        while !self.check(&Token::RightBracket) {
            let start = self.position;
            let mut item = if items.is_empty() {
                self.list_head()?
            } else {
                self.within(true, Self::item)?
            };
            debug_assert!(self.position > start);
            if items.is_empty() && self.starts_comprehension() {
                return self.comprehension(start, item);
            }
            self.items.mark(start, self.position, &mut item);
            items.push(item);

            if !self.eat(&Token::Comma) && !self.list_line_break() {
                break;
            }
        }

        self.expect(&Token::RightBracket)?;
        Ok(Expr::List(items))
    }

    fn list_line_break(&self) -> bool {
        let position = self.here();
        self.list_separators == ListSeparators::PhysicalLines
            && !matches!(self.peek(), Token::Identifier(name) if name == "is")
            && self
                .line_breaks
                .binary_search_by_key(&(position.line, position.column), |boundary| {
                    (boundary.line, boundary.column)
                })
                .is_ok()
    }
}
