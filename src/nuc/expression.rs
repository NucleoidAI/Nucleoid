//! `EXPRESSION` — a statement that is only an expression, and the `.value`
//! freezing every stored declaration goes through first. Mirrors
//! `ref/src/nuc/EXPRESSION.js`, whose `before()` does the same rewrite; the
//! kinds that file a declaration — [`variable`](crate::nuc::variable),
//! [`property`](crate::nuc::property) and `if` — reach it through
//! `Runtime::freeze`.

use crate::error::{Error, Result};
use crate::lang::ast::{Expr, Stmt, literal};
use crate::nuc::Outcome;
use crate::runtime::{MAX_DEPTH, Runtime};
use crate::scope::Scope;

#[derive(Debug, Clone)]
pub struct Expression {
    pub tokens: Expr,
}

impl Expression {
    pub fn new(tokens: Expr) -> Self {
        Expression { tokens }
    }

    pub fn run(&mut self, runtime: &mut Runtime, scope: &mut Scope) -> Result<Outcome> {
        let value = runtime.evaluate(&self.tokens, scope)?;
        Ok(Outcome::value(value))
    }
}

impl Runtime {
    /// Replaces every `x.value` read with the value it has right now, so the
    /// stored declaration keeps that value instead of following `x` later.
    pub(crate) fn freeze(&mut self, expression: &Expr, scope: &mut Scope) -> Result<Expr> {
        self.depth += 1;

        if self.depth > MAX_DEPTH {
            self.depth -= 1;
            return Err(Error::type_error("Maximum expression depth exceeded"));
        }

        let result = self.freeze_inner(expression, scope);
        self.depth -= 1;
        result
    }

    fn freeze_inner(&mut self, expression: &Expr, scope: &mut Scope) -> Result<Expr> {
        Ok(match expression {
            Expr::Member { object, property } if property == "value" => {
                let value = self.evaluate(expression, scope)?;
                literal::of(&value).unwrap_or_else(|| Expr::Member {
                    object: object.clone(),
                    property: property.clone(),
                })
            }
            Expr::Member { object, property } => Expr::Member {
                object: Box::new(self.freeze(object, scope)?),
                property: property.clone(),
            },
            Expr::Binary {
                operator,
                left,
                right,
            } => Expr::Binary {
                operator: *operator,
                left: Box::new(self.freeze(left, scope)?),
                right: Box::new(self.freeze(right, scope)?),
            },
            Expr::Logical {
                operator,
                left,
                right,
            } => Expr::Logical {
                operator: *operator,
                left: Box::new(self.freeze(left, scope)?),
                right: Box::new(self.freeze(right, scope)?),
            },
            Expr::Unary { operator, operand } => Expr::Unary {
                operator: *operator,
                operand: Box::new(self.freeze(operand, scope)?),
            },
            Expr::Call { callee, arguments } => {
                let mut frozen = Vec::with_capacity(arguments.len());
                for argument in arguments {
                    frozen.push(self.freeze(argument, scope)?);
                }
                Expr::Call {
                    callee: Box::new(self.freeze(callee, scope)?),
                    arguments: frozen,
                }
            }
            Expr::Index { object, index } => Expr::Index {
                object: Box::new(self.freeze(object, scope)?),
                index: Box::new(self.freeze(index, scope)?),
            },
            other => other.clone(),
        })
    }
}

/// `assert` is a test helper, so it does not become the program's result.
pub(crate) fn is_assertion(statement: &Stmt) -> bool {
    let Stmt::Expression(Expr::Call { callee, .. }) = statement else {
        return false;
    };

    matches!(callee.as_ref(), Expr::Identifier(name) if name == "assert")
}
