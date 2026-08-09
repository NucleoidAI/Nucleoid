use crate::error::{Error, Result};
use crate::graph::NodeKey;
use crate::lang::ast::Expr;
use crate::reasoning::Selection;
use crate::runtime::Runtime;
use crate::scope::Scope;
use crate::value::{ObjectId, Value};

pub struct Reason<'a> {
    pub node: &'a Expr,
}

impl<'a> Reason<'a> {
    pub fn new(node: &'a Expr) -> Self {
        Reason { node }
    }

    pub fn resolve(&self, runtime: &mut Runtime, scope: &mut Scope) -> Result<Value> {
        let selection = runtime.reason(self.node, scope)?;
        Ok(runtime.materialize(&selection))
    }
}

impl Runtime {
    pub(crate) fn reason(&mut self, expression: &Expr, scope: &mut Scope) -> Result<Selection> {
        match expression {
            Expr::Reason { stage, source } => {
                let selection = self.reason(source, scope)?;
                Ok(self.apply_stage(*stage, &selection))
            }
            Expr::Model => {
                let keys = self.model_keys();
                Ok(self.selection_of(keys))
            }
            other => {
                let key = self.selection_key(other, scope)?;
                Ok(self.selection_of(vec![key]))
            }
        }
    }

    fn selection_key(&mut self, expression: &Expr, scope: &mut Scope) -> Result<NodeKey> {
        match expression {
            Expr::Identifier(name) => {
                if let Some(Value::Object(id)) = scope.retrieve(name) {
                    return Ok(NodeKey::object(&id.clone()));
                }

                self.reference_key(name)
            }

            Expr::Member { object, property } => {
                let base = self.evaluate(object, scope)?;

                match base {
                    Value::Object(id) => Ok(NodeKey::property(&id, property)),
                    Value::Undefined | Value::Null => {
                        Err(Error::not_defined(self.describe(object, scope)))
                    }
                    _ => Err(Error::type_error(format!("Cannot explain {expression}"))),
                }
            }

            Expr::ObjectRef(id) => Ok(NodeKey::object(&ObjectId::from(id.clone()))),

            other => Err(Error::type_error(format!("Cannot explain {other}"))),
        }
    }
}
