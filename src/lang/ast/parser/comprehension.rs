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
            let condition = self.within(true, Self::expression);
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
