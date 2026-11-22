use super::query::optional_selector;
use crate::error::{Error, Result};
use crate::runtime::Runtime;
use crate::value::Value;

impl Runtime {
    pub(super) fn zip_items(&mut self, items: Vec<Value>, arguments: &[Value]) -> Result<Value> {
        let partners = self.zip_partners(arguments.first(), items.len())?;
        let combine = optional_selector(arguments.get(1).cloned(), "zip")?;
        let mut zipped = Vec::with_capacity(items.len().min(partners.len()));

        for (item, partner) in items.into_iter().zip(partners) {
            zipped.push(match &combine {
                Some(function) => self.invoke(function, &[item, partner])?,
                None => Value::List(vec![item, partner]),
            });
        }

        Ok(Value::List(zipped))
    }

    fn zip_partners(&mut self, argument: Option<&Value>, limit: usize) -> Result<Vec<Value>> {
        let partners = match argument {
            Some(Value::List(other)) => Some(other.iter().take(limit).cloned().collect()),
            Some(Value::Class(name)) => self.first_instances(name, limit),
            None
            | Some(
                Value::Undefined
                | Value::Null
                | Value::Bool(_)
                | Value::Number(_)
                | Value::String(_)
                | Value::Object(_)
                | Value::Date(_)
                | Value::Regex(_)
                | Value::Function(_),
            ) => None,
        };
        partners.ok_or_else(|| Error::type_error("zip expects a list"))
    }
}

#[cfg(test)]
mod tests;
