//! `LET` — a name bound in the surrounding scope: a parameter, a loop variable,
//! or a name an enclosing block already assigned. Written directly and never
//! filed in the graph. Mirrors `ref/src/nuc/LET.js`.

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
        let saved = runtime.null_read;
        runtime.null_read = false;
        runtime.push_tracking(false);
        let evaluated = runtime.evaluate(&self.value, scope);
        let dependencies = runtime.pop_tracking();
        let evaluated = runtime.settle(evaluated?);
        runtime.null_read = saved;

        // A local is not a graph node, but the declaration around it still
        // depends on whatever the value read.
        for dependency in dependencies {
            runtime.track(dependency);
        }

        if !scope.assign(&self.name, evaluated.clone()) {
            scope.declare(self.name.clone(), evaluated.clone());
        }

        Ok(Outcome::value(evaluated))
    }
}
