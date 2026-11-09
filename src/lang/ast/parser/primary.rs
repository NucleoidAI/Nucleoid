use std::sync::Arc;

use super::{ArgumentContext, ExpressionContext, Parser};
use crate::error::Result;
use crate::lang::ast::lexer::{Keyword, Token};
use crate::lang::ast::{Expr, Function, FunctionBody, ObjectEntry, Parameter, Quantifier};

fn object_expression(entries: Vec<ObjectEntry>) -> Expr {
    if entries
        .iter()
        .any(|entry| matches!(entry, ObjectEntry::Spread(_)))
    {
        return Expr::SpreadObject(entries);
    }
    Expr::ObjectLiteral(
        entries
            .into_iter()
            .filter_map(|entry| match entry {
                ObjectEntry::Property(key, value) => Some((key, value)),
                ObjectEntry::Spread(_) => None,
            })
            .collect(),
    )
}

impl Parser {
    pub(super) fn primary(&mut self) -> Result<Expr> {
        if self.check(&Token::At) {
            return self.at_variable();
        }
        if self.context != ExpressionContext::Interpolation {
            if let Some(quantifier) = self.quantifier_at(0) {
                return self.quantified(quantifier);
            }
        }
        // `name => ...` or `name -> ...`. Check before taking the name so
        // that ordinary identifier operands are not cloned here.
        if matches!(self.peek(), Token::Identifier(_)) && self.peek_at(1).is_arrow() {
            let parameter = self.untyped_parameter()?;
            self.advance();
            return self.arrow_body(vec![parameter]);
        }

        // `lambda a, b: ...`
        if let Some(count) = self.lambda_parameter_count() {
            return self.lambda(count);
        }

        // `(a, b) => ...` or `(a, b) -> ...`
        if self.check(&Token::LeftParen) && self.is_arrow_parameters() {
            let parameters = self.parameters()?;
            self.expect_arrow()?;
            return self.arrow_body(parameters);
        }

        match self.advance() {
            Token::Number(number) => Ok(Expr::Number(number)),
            Token::Str(string) => Ok(Expr::String(string)),
            Token::Template(template) => self.template(&template),
            Token::Regex(pattern) => Ok(Expr::Regex(pattern)),
            Token::Identifier(name) => Ok(Expr::Identifier(name)),
            Token::ClassRef(name) => Ok(Expr::ClassRef(name)),
            Token::Keyword(Keyword::True) => Ok(Expr::Bool(true)),
            Token::Keyword(Keyword::False) => Ok(Expr::Bool(false)),
            Token::Keyword(Keyword::Null) => Ok(Expr::Null),
            Token::Keyword(Keyword::This) => Ok(Expr::This),
            Token::Keyword(Keyword::New) => self.nested_expression(Self::primary),
            Token::Keyword(Keyword::Delete) => {
                let operand = self.nested_expression(Self::unary)?;
                Ok(Expr::Delete(Box::new(operand)))
            }
            Token::Keyword(Keyword::Super) => {
                // Only an immediate `(` introduces arguments, not separators.
                let arguments = if self.eat(&Token::LeftParen) {
                    self.arguments(ArgumentContext::PotentialConstructor)?
                } else {
                    Vec::new()
                };
                Ok(Expr::Super(arguments))
            }
            Token::Keyword(Keyword::Function) => self.function_literal(),
            Token::LeftParen => {
                let expression = self.within(true, Self::expression)?;
                self.expect(&Token::RightParen)?;
                Ok(expression)
            }
            Token::LeftBracket => self.list_literal(),
            Token::LeftBrace => self.object_literal(),
            other => Err(self.error(format!("Unexpected {}", other.describe()))),
        }
    }

    fn function_literal(&mut self) -> Result<Expr> {
        let parameters = self.parameters()?;
        let items = self.leave_items();
        let body = self.braced();
        self.items = items;
        let body = body?;
        Ok(Expr::Function(Arc::new(Function {
            name: None,
            parameters,
            body: FunctionBody::Block(body),
        })))
    }

    fn list_literal(&mut self) -> Result<Expr> {
        let mut items = Vec::new();

        while !self.check(&Token::RightBracket) {
            let start = self.position;
            let mut item = self.within(true, Self::item)?;
            if items.is_empty() && self.starts_comprehension() {
                return self.comprehension(start, item);
            }
            self.items.mark(start, self.position, &mut item);
            items.push(item);

            if !self.eat(&Token::Comma) {
                break;
            }
        }

        self.expect(&Token::RightBracket)?;
        Ok(Expr::List(items))
    }

    fn object_literal(&mut self) -> Result<Expr> {
        let mut entries = Vec::new();

        loop {
            self.skip_newlines();

            if self.eat(&Token::RightBrace) {
                break;
            }

            entries.push(self.object_entry()?);

            self.skip_newlines();

            if !self.eat(&Token::Comma) {
                self.skip_newlines();
                // A complete ordinary value wins before considering the next
                // explicit field. Keep shorthand and key parsing unchanged.
                if matches!(
                    self.peek(),
                    Token::Str(_) | Token::Identifier(_) | Token::Number(_)
                ) && self.peek_at(1) == &Token::Colon
                {
                    continue;
                }
                self.expect(&Token::RightBrace)?;
                break;
            }
        }

        Ok(object_expression(entries))
    }

    fn object_entry(&mut self) -> Result<ObjectEntry> {
        if self.starts_rest_parameter() {
            self.position += 3;
            let source = self.object_value()?;
            return Ok(ObjectEntry::Spread(source));
        }

        let (key, is_identifier) = match self.advance() {
            Token::Str(key) => (key, false),
            Token::Identifier(key) => (key, true),
            Token::Number(number) => (crate::value::format_number(number), false),
            other => {
                return Err(self.error(format!("Expected a key but found {}", other.describe())));
            }
        };

        let mut offset = 0;
        while matches!(self.peek_at(offset), Token::Newline | Token::Semicolon) {
            offset += 1;
        }
        let value =
            if is_identifier && matches!(self.peek_at(offset), Token::Comma | Token::RightBrace) {
                Expr::Identifier(key.clone())
            } else {
                self.expect(&Token::Colon)?;
                let start = self.position;
                let mut value = self.within(true, Self::object_item)?;
                self.items.mark(start, self.position, &mut value);
                value
            };
        Ok(ObjectEntry::Property(key, value))
    }

    fn object_value(&mut self) -> Result<Expr> {
        self.within(true, |parser| {
            parser.expression_in(ExpressionContext::ObjectValue)
        })
    }

    fn arrow_body(&mut self, parameters: Vec<Parameter>) -> Result<Expr> {
        let items = self.leave_items();
        let body = if self.check(&Token::LeftBrace) {
            self.braced().map(FunctionBody::Block)
        } else {
            self.expression_in(self.context)
                .map(|body| FunctionBody::Expression(Box::new(body)))
        };
        self.items = items;
        let body = body?;

        Ok(Expr::Function(Arc::new(Function {
            name: None,
            parameters,
            body,
        })))
    }

    /// The parameter count of a `lambda` expression starting here, qualified
    /// without consuming tokens or allocating names. Only an ordinary
    /// identifier spelled `lambda`, then nothing or comma-separated ordinary
    /// names, then a `:` qualifies; `lambda` is not reserved.
    ///
    /// The name stays an ordinary identifier wherever accepted syntax owns
    /// that colon: a clause header, subscript start, conditional consequent or
    /// template interpolation; a single name in an object value, where
    /// `lambda next:` is a comma-free property boundary; and a single name
    /// whose colon starts a statement, which statement loops accept directly
    /// after the expression ending at `lambda`: a typed declaration
    /// (`lambda x: int`) or an assertion statement (`lambda assert: x, 1`).
    pub(super) fn lambda_parameter_count(&self) -> Option<usize> {
        if matches!(
            self.context,
            ExpressionContext::Clause | ExpressionContext::Interpolation
        ) || !matches!(self.peek(), Token::Identifier(name) if name == "lambda")
        {
            return None;
        }

        let mut count = 0;
        let mut colon = 1;
        if self.peek_at(colon) != &Token::Colon {
            loop {
                if !matches!(self.peek_at(colon), Token::Identifier(_)) {
                    return None;
                }
                count += 1;
                colon += 1;
                match self.peek_at(colon) {
                    Token::Comma => colon += 1,
                    Token::Colon => break,
                    _ => return None,
                }
            }
        }

        // A single name and its colon keep every accepted reading: the next
        // comma-free property, or a declaration or assertion statement.
        let accepted_colon = count == 1
            && (self.context == ExpressionContext::ObjectValue
                || self.starts_colon_statement_at(1));
        (!accepted_colon).then_some(count)
    }

    /// Parses a lambda that [`Self::lambda_parameter_count`] qualified. It
    /// lowers to exactly the anonymous `(names) => (expression)` function:
    /// untyped parameters and an expression body, where braces are an object.
    fn lambda(&mut self, count: usize) -> Result<Expr> {
        self.advance();
        let mut parameters = Vec::with_capacity(count);
        for index in 0..count {
            if index > 0 {
                self.expect(&Token::Comma)?;
            }
            parameters.push(self.untyped_parameter()?);
        }
        self.expect(&Token::Colon)?;
        let items = self.leave_items();
        let body = self.expression_in(self.context);
        self.items = items;
        let body = body?;

        Ok(Expr::Function(Arc::new(Function {
            name: None,
            parameters,
            body: FunctionBody::Expression(Box::new(body)),
        })))
    }

    pub(super) fn quantifier_at(&self, offset: usize) -> Option<Quantifier> {
        Quantifier::starting(
            self.peek_at(offset),
            self.peek_at(offset + 1),
            self.peek_at(offset + 2),
        )
    }

    fn quantified(&mut self, quantifier: Quantifier) -> Result<Expr> {
        self.advance();
        let binder = match self.advance() {
            Token::Identifier(binder) => binder,
            other => {
                return Err(self.error(format!("Expected a name but found {}", other.describe())));
            }
        };
        self.advance();
        let collection = self.clause_expression()?;
        self.expect(&Token::Colon)?;
        let condition = self.expression_in(self.context)?;

        Ok(Expr::Quantified {
            quantifier,
            binder,
            collection: Box::new(collection),
            condition: Box::new(condition),
        })
    }

    /// Looks ahead past a balanced `( ... )` for an arrow, which distinguishes
    /// an arrow parameter list from a parenthesised expression.
    fn is_arrow_parameters(&self) -> bool {
        let mut depth = 0usize;
        let mut offset = 0usize;

        loop {
            match self.peek_at(offset) {
                Token::LeftParen | Token::LeftBracket => depth += 1,
                Token::RightParen | Token::RightBracket => {
                    depth -= 1;
                    if depth == 0 {
                        return self.peek_at(offset + 1).is_arrow();
                    }
                }
                Token::Eof => return false,
                _ => {}
            }

            offset += 1;
        }
    }
}
