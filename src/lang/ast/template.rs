//! Template literals.

use crate::error::{Error, Result};
use crate::lang::ast::{Expr, TemplatePart};
use crate::runtime::Runtime;
use crate::scope::Scope;
use crate::value::Value;

#[derive(Debug)]
pub struct Template<'a> {
    pub node: &'a Expr,
}

impl<'a> Template<'a> {
    #[must_use]
    pub fn new(node: &'a Expr) -> Self {
        Template { node }
    }

    pub fn resolve(&self, runtime: &mut Runtime, scope: &mut Scope) -> Result<Value> {
        let Expr::Template(parts) = self.node else {
            return Err(Error::type_error(format!(
                "{} is not a template",
                self.node
            )));
        };

        let mut output = String::new();

        for part in parts {
            match part {
                TemplatePart::Literal(literal) => output.push_str(literal),
                TemplatePart::Expression(expression) => {
                    let value = runtime.evaluate(expression, scope)?;
                    output.push_str(&value.to_string());
                }
            }
        }

        Ok(Value::String(output))
    }
}
