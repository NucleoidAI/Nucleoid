//! Template literals. Mirrors `ref/src/lang/ast/Template.js`.

use crate::error::Result;
use crate::lang::ast::TemplatePart;
use crate::runtime::Runtime;
use crate::scope::Scope;
use crate::value::Value;

impl Runtime {
    pub(crate) fn evaluate_template(
        &mut self,
        parts: &[TemplatePart],
        scope: &mut Scope,
    ) -> Result<Value> {
        let mut output = String::new();

        for part in parts {
            match part {
                TemplatePart::Literal(literal) => output.push_str(literal),
                TemplatePart::Expression(expression) => {
                    let value = self.evaluate(expression, scope)?;
                    output.push_str(&value.to_string());
                }
            }
        }

        Ok(Value::String(output))
    }
}
