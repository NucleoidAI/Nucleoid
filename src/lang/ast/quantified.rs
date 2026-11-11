use crate::error::{Error, Result};
use crate::lang::ast::{Expr, Quantifier};
use crate::runtime::Runtime;
use crate::scope::Scope;
use crate::value::Value;

#[derive(Debug)]
pub struct Quantified<'a> {
    pub node: &'a Expr,
}

impl<'a> Quantified<'a> {
    #[must_use]
    pub fn new(node: &'a Expr) -> Self {
        Quantified { node }
    }

    pub fn resolve(&self, runtime: &mut Runtime, scope: &mut Scope) -> Result<Value> {
        let Expr::Quantified {
            quantifier,
            binder,
            collection,
            condition,
        } = self.node
        else {
            return Err(Error::type_error(format!(
                "{} is not a quantifier",
                self.node
            )));
        };

        runtime.evaluate_quantified(*quantifier, binder, collection, condition, scope)
    }
}

impl Runtime {
    pub(crate) fn evaluate_quantified(
        &mut self,
        quantifier: Quantifier,
        binder: &str,
        collection: &Expr,
        condition: &Expr,
        scope: &mut Scope,
    ) -> Result<Value> {
        let members = match self.evaluate(collection, scope)? {
            Value::List(items) => Some(items),
            Value::Class(name) => self.population(&name),
            Value::Null | Value::Undefined => return Ok(Value::Undefined),
            _ => None,
        };
        let Some(members) = members else {
            return Err(Error::not_a_collection(collection.describe()));
        };

        let decisive = quantifier == Quantifier::Any;

        for member in members {
            scope.push();
            scope.declare(binder, member);
            let holds = self.evaluate(condition, scope);
            scope.pop();

            if holds?.truthy() == decisive {
                return Ok(Value::Bool(decisive));
            }
        }

        Ok(Value::Bool(!decisive))
    }
}
