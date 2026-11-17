use std::sync::Arc;

use super::is_assignment::Branches;
use super::{ExpressionContext, Lists, Parser};
use crate::error::Result;
use crate::lang::ast::lexer::{Keyword, Token};
use crate::lang::ast::{Expr, Function, FunctionBody, Parameter};

fn method_call(receiver: Expr, method: &str, function: Expr) -> Expr {
    Expr::Call {
        callee: Box::new(Expr::Member {
            object: Box::new(receiver),
            property: method.to_string(),
        }),
        arguments: vec![function],
    }
}

fn lambda(parameter: Parameter, body: Expr) -> Expr {
    Expr::Function(Arc::new(Function {
        name: None,
        parameters: vec![parameter],
        body: FunctionBody::Expression(Box::new(body)),
    }))
}

impl Parser {
    pub(super) fn starts_comprehension(&self) -> bool {
        self.lists == Lists::Comprehensions && self.check(&Token::Keyword(Keyword::For))
    }

    pub(super) fn list_head(&mut self) -> Result<Expr> {
        if self.holds_instance() && self.list_holds_for() {
            return self.without_instance(|parser| parser.within(true, Self::item));
        }
        self.within(true, Self::item)
    }

    fn list_holds_for(&self) -> bool {
        let mut depth = 0usize;
        for spanned in self.tokens.iter().skip(self.position) {
            match spanned.token {
                Token::LeftParen | Token::LeftBracket | Token::LeftBrace => depth += 1,
                Token::RightParen | Token::RightBracket | Token::RightBrace => {
                    let Some(outer) = depth.checked_sub(1) else {
                        return false;
                    };
                    depth = outer;
                }
                Token::Keyword(Keyword::For) if depth == 0 => return true,
                _ => {}
            }
        }
        false
    }

    pub(super) fn comprehension(&mut self, start: usize, element: Expr) -> Result<Expr> {
        if let Some(variable) = self.at_variable_since(start) {
            return self.reject_at_variable(variable);
        }
        let element = self.unmark(Branches::Plain, element);
        self.advance();
        let parameter = self.comprehension_name()?;
        if !self.eat_keyword(Keyword::In) && !self.eat_keyword(Keyword::Of) {
            return Err(self.error(format!(
                "Expected 'in' or 'of' but found {}",
                self.peek().describe()
            )));
        }
        let source = self.within(true, |parser| {
            parser.expression_in(ExpressionContext::ComprehensionSource)
        })?;
        let source = if self.eat_keyword(Keyword::If) {
            let items = self.leave_items();
            let condition = self.without_instance(|parser| parser.within(true, Self::expression));
            self.items = items;
            method_call(source, "filter", lambda(parameter.clone(), condition?))
        } else {
            source
        };
        self.expect(&Token::RightBracket)?;
        Ok(method_call(source, "map", lambda(parameter, element)))
    }

    fn comprehension_name(&mut self) -> Result<Parameter> {
        if matches!(self.peek(), Token::Identifier(_)) {
            return self.untyped_parameter();
        }
        Err(self.error(format!(
            "Expected a name but found {}",
            self.peek().describe()
        )))
    }
}
