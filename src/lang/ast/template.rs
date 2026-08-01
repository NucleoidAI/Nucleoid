//! Template literals. Mirrors `ref/src/lang/ast/Template.js`.

use crate::error::Result;
use crate::lang::ast::{Expr, TemplatePart};
use crate::runtime::Runtime;
use crate::scope::Scope;
use crate::value::Value;

pub struct Template<'a> {
    pub node: &'a Expr,
}

impl<'a> Template<'a> {
    pub fn new(node: &'a Expr) -> Self {
        Template { node }
    }

    pub fn resolve(&self, runtime: &mut Runtime, scope: &mut Scope) -> Result<Value> {
        let Expr::Template(parts) = self.node else {
            unreachable!("Template only wraps Expr::Template")
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
