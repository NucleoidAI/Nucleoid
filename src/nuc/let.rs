//! Local assignments to parameters, loop variables or names bound by an
//! enclosing block. These are written directly and never filed in the graph.

use crate::error::Result;
use crate::lang::ast::Expr;
use crate::nuc::Outcome;
use crate::runtime::Runtime;
use crate::scope::Scope;

#[derive(Debug, Clone)]
pub struct Let {
    pub name: String,
    pub value: Expr,
}

impl Let {
    pub fn new(name: String, value: Expr) -> Self {
        Let { name, value }
    }

    pub fn run(&mut self, runtime: &mut Runtime, scope: &mut Scope) -> Result<Outcome> {
        let (evaluated, _) = runtime.evaluate_tracked(&self.value, scope, None)?;

        if !scope.assign(&self.name, evaluated.clone()) {
            scope.declare(self.name.clone(), evaluated.clone());
        }

        Ok(Outcome::value(evaluated))
    }
}
