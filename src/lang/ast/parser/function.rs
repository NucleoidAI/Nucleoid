use std::sync::Arc;

use super::class::initializer;
use super::{FunctionSpelling, ParameterList, Parser, ScalarLiteral};
use crate::error::{Position, Result};
use crate::lang::ast::lexer::{Keyword, Token};
use crate::lang::ast::{Function, FunctionBody, Parameter, ParameterDefault, Stmt};

impl Parser {
    /// Offset of the delimiter after a complete bare list of at least two
    /// ordinary identifiers with optional identifier annotations, ending in an
    /// arrow or suite colon. Inspect without consuming tokens, allocating names
    /// or skipping layout; the caller checks a block's real indentation.
    fn bare_multiple_parameter_end(&self) -> Option<usize> {
        let mut offset = 0;
        let mut multiple = false;
        loop {
            if !matches!(self.peek_at(offset), Token::Identifier(_)) {
                return None;
            }
            offset += 1;
            if matches!(
                (self.peek_at(offset), self.peek_at(offset + 1)),
                (Token::Colon, Token::Identifier(_))
            ) {
                offset += 2;
            }
            match self.peek_at(offset) {
                delimiter if multiple && (delimiter.is_arrow() || delimiter == &Token::Colon) => {
                    return Some(offset);
                }
                Token::Comma => {
                    multiple = true;
                    offset += 1;
                }
                _ => return None,
            }
        }
    }

    /// Scan a complete uninterrupted run without consuming or allocating names.
    /// Each caller must still qualify its own delimiter and body layout.
    fn comma_free_parameter_end(&self) -> Option<usize> {
        let mut end = 0;
        while matches!(self.peek_at(end), Token::Identifier(_)) {
            end += 1;
        }
        (end >= 2).then_some(end)
    }

    /// Keep the arrow guard arrow-only; blocks need a genuine indented suite.
    pub(super) fn comma_free_arrow_end(&self) -> Option<usize> {
        self.comma_free_parameter_end()
            .filter(|&end| self.peek_at(end).is_arrow())
    }

    pub(super) fn comma_free_block_end(&self) -> Option<usize> {
        self.comma_free_parameter_end()
            .filter(|&end| self.has_indented_suite(end))
    }

    /// Only used after complete lookahead has qualified an untyped name.
    /// In particular, leave a final suite colon for `suite()`, not `parameter()`.
    pub(super) fn untyped_parameter(&mut self) -> Result<Parameter> {
        match self.advance() {
            Token::Identifier(name) => Ok(Parameter {
                name,
                type_name: None,
                default: None,
                rest: false,
            }),
            other => Err(self.error(format!(
                "Expected a parameter but found {}",
                other.describe()
            ))),
        }
    }

    pub(super) fn function_statement(&mut self) -> Result<Stmt> {
        self.function_declaration().map(Stmt::Function)
    }

    pub(super) fn starts_function_declaration(&self) -> bool {
        match self.peek() {
            Token::Keyword(Keyword::Def) => true,
            Token::Keyword(Keyword::Function) => matches!(self.peek_at(1), Token::Identifier(_)),
            _ => false,
        }
    }

    pub(super) fn function_declaration(&mut self) -> Result<Arc<Function>> {
        let spelling = self.function_keyword()?;
        let name = self.function_name()?;
        let parameters = match spelling {
            FunctionSpelling::Def => self.named_parameters()?,
            FunctionSpelling::Function => self.parameters()?,
        };
        let body = self.declaration_body(spelling)?;

        Ok(Arc::new(Function {
            name: Some(name),
            parameters,
            body: FunctionBody::Block(body),
        }))
    }

    pub(super) fn function_keyword(&mut self) -> Result<FunctionSpelling> {
        if self.eat_keyword(Keyword::Function) {
            return Ok(FunctionSpelling::Function);
        }
        self.expect(&Token::Keyword(Keyword::Def))?;
        Ok(FunctionSpelling::Def)
    }

    pub(super) fn declaration_body(&mut self, spelling: FunctionSpelling) -> Result<Vec<Stmt>> {
        match spelling {
            FunctionSpelling::Def => self.named_body(),
            FunctionSpelling::Function => self.braced(),
        }
    }

    pub(super) fn function_name(&mut self) -> Result<String> {
        match self.advance() {
            Token::Identifier(name) => Ok(name),
            other => Err(self.error(format!("Expected a name but found {}", other.describe()))),
        }
    }

    fn named_parameters(&mut self) -> Result<Vec<Parameter>> {
        // Either arrow spelling, `=>` or `->`, ends a bare arrow header.
        let bare_arrow = matches!(self.peek(), Token::Identifier(_))
            && (self.peek_at(1).is_arrow()
                || (self.peek_at(1) == &Token::Colon
                    && matches!(self.peek_at(2), Token::Identifier(_))
                    && self.peek_at(3).is_arrow()));
        let bare_multiple_end = self
            .bare_multiple_parameter_end()
            .filter(|&end| self.peek_at(end).is_arrow() || self.has_indented_suite(end));
        let bare_block = matches!(self.peek(), Token::Identifier(_)) && self.has_indented_suite(1);
        let bare_typed_block = matches!(
            (
                self.peek(),
                self.peek_at(1),
                self.peek_at(2),
                self.peek_at(3)
            ),
            (
                Token::Identifier(_),
                Token::Colon,
                Token::Identifier(_),
                Token::Colon
            )
        ) && self.has_indented_suite(3);
        let comma_free_end = self
            .comma_free_arrow_end()
            .or_else(|| self.comma_free_block_end());
        if let Some(end) = comma_free_end {
            (0..end).map(|_| self.untyped_parameter()).collect()
        } else if let Some(end) = bare_multiple_end {
            self.bare_parameters(self.position + end)
        } else if bare_arrow || bare_typed_block {
            Ok(vec![self.parameter()?])
        } else if bare_block {
            Ok(vec![self.untyped_parameter()?])
        } else {
            // Do not skip separators or consume other prefixes: preserve the
            // parenthesized parameter parser's diagnostics and positions.
            self.parameters()
        }
    }

    fn bare_parameters(&mut self, delimiter_position: usize) -> Result<Vec<Parameter>> {
        let mut parameters = Vec::new();
        loop {
            let parameter = if self.position + 1 == delimiter_position
                && matches!(self.peek_at(1), Token::Colon)
            {
                // A final untyped parameter's colon belongs to suite(),
                // not parameter()'s annotation parser.
                self.untyped_parameter()?
            } else {
                self.parameter()?
            };
            parameters.push(parameter);
            if !self.eat(&Token::Comma) {
                break;
            }
        }
        // Leave the complete list's arrow or suite colon for the body.
        Ok(parameters)
    }

    pub(super) fn named_body(&mut self) -> Result<Vec<Stmt>> {
        if !self.eat_arrow() {
            return self.suite();
        }
        let indented = self.has_indented_body(0);
        if indented {
            self.skip_newlines();
            self.expect(&Token::Indent)?;
            self.skip_newlines();
        }
        let expression = self.expression()?;
        if indented {
            self.skip_newlines();
            self.expect(&Token::Dedent)?;
        }
        Ok(vec![Stmt::Return(Some(expression))])
    }

    fn parameter(&mut self) -> Result<Parameter> {
        let name = match self.advance() {
            Token::Identifier(name) => name,
            other => {
                return Err(self.error(format!(
                    "Expected a parameter but found {}",
                    other.describe()
                )));
            }
        };

        let type_name = if self.eat(&Token::Colon) {
            match self.advance() {
                Token::Identifier(type_name) => Some(type_name),
                other => {
                    return Err(
                        self.error(format!("Expected a type but found {}", other.describe()))
                    );
                }
            }
        } else {
            None
        };

        Ok(Parameter {
            name,
            type_name,
            default: None,
            rest: false,
        })
    }

    /// Only `parameters()` calls this: bare signature parsing is unchanged.
    fn parameter_default(&mut self) -> Result<ParameterDefault> {
        let default = match self.scalar_literal() {
            ScalarLiteral::Value(default) => default,
            ScalarLiteral::NotFinite => {
                return Err(self.error("Parameter default must be a finite number"));
            }
            ScalarLiteral::Missing => {
                return Err(self.error("Expected a scalar literal parameter default"));
            }
        };
        if !matches!(self.peek(), Token::Comma | Token::RightParen) {
            return Err(self.error("Expected ',' or ')' after parameter default"));
        }
        Ok(default)
    }

    pub(super) fn scalar_literal(&mut self) -> ScalarLiteral {
        let negative = if self.eat(&Token::Minus) {
            Some(true)
        } else if self.eat(&Token::Plus) {
            Some(false)
        } else {
            None
        };
        ScalarLiteral::Value(match self.peek().clone() {
            Token::Number(number) if !number.is_finite() => return ScalarLiteral::NotFinite,
            Token::Number(number) => {
                self.advance();
                ParameterDefault::Number(if negative == Some(true) {
                    -number
                } else {
                    number
                })
            }
            Token::Str(value) if negative.is_none() => {
                self.advance();
                ParameterDefault::String(value)
            }
            Token::Keyword(Keyword::True | Keyword::False) if negative.is_none() => {
                let value = self.eat_keyword(Keyword::True);
                if !value {
                    self.advance();
                }
                ParameterDefault::Bool(value)
            }
            Token::Keyword(Keyword::Null) if negative.is_none() => {
                self.advance();
                ParameterDefault::Null
            }
            _ => return ScalarLiteral::Missing,
        })
    }

    /// Recognize adjacent dots only here, without changing tokenization of
    /// any existing source or adding rest syntax to bare signatures.
    pub(super) fn starts_rest_parameter(&self) -> bool {
        let start = self.here();
        (0..3).all(|offset| {
            self.tokens
                .get(self.position + offset)
                .is_some_and(|spanned| {
                    spanned.token == Token::Dot
                        && spanned.position == Position::new(start.line, start.column + offset)
                })
        })
    }

    pub(super) fn parameters(&mut self) -> Result<Vec<Parameter>> {
        let list = self.parameter_list(false)?;
        debug_assert_eq!(list.initializers, Vec::<Stmt>::new());
        Ok(list.parameters)
    }

    pub(super) fn constructor_parameters(&mut self) -> Result<ParameterList> {
        self.parameter_list(true)
    }

    fn starts_initializing_parameter(&self) -> bool {
        matches!(
            (
                self.peek(),
                self.peek_at(1),
                self.peek_at(2),
                self.peek_at(3)
            ),
            (
                Token::Keyword(Keyword::This),
                Token::Dot,
                Token::Identifier(_),
                Token::Comma | Token::RightParen | Token::Colon | Token::Assign
            )
        )
    }

    fn parameter_list(&mut self, initializing: bool) -> Result<ParameterList> {
        self.expect(&Token::LeftParen)?;

        let mut parameters: Vec<Parameter> = Vec::new();
        let mut initializers = Vec::new();

        while !self.check(&Token::RightParen) {
            let rest = self.starts_rest_parameter();
            if rest {
                self.position += 3;
            }
            let initialized = initializing && self.starts_initializing_parameter();
            if initialized {
                self.position += 2;
            }
            let mut parameter = self.parameter()?;
            parameter.rest = rest;
            if rest && parameters.iter().any(|fixed| fixed.name == parameter.name) {
                return Err(self.error(format!(
                    "Rest parameter '{}' duplicates a fixed parameter",
                    parameter.name
                )));
            }
            if self.eat(&Token::Assign) {
                if rest {
                    return Err(self.error("Rest parameter cannot have a default"));
                }
                parameter.default = Some(self.parameter_default()?);
            }
            if initialized {
                initializers.push(initializer(&parameter.name));
            }
            parameters.push(parameter);

            if !self.eat(&Token::Comma) {
                break;
            }
            if rest && !self.check(&Token::RightParen) {
                return Err(self.error("Rest parameter must be last"));
            }
        }

        self.expect(&Token::RightParen)?;
        debug_assert!(initializers.len() <= parameters.len());
        Ok(ParameterList {
            parameters,
            initializers,
        })
    }
}
