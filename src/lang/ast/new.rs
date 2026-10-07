//! Recognizing class instantiation.
//!
//! `Class(...)` is parsed as an ordinary [`call`](crate::lang::ast::call).
//! [`New`] identifies calls to class names; [`crate::nuc::object`] creates the
//! instances.

use crate::lang::ast::Expr;
use crate::runtime::Runtime;

pub struct New<'a> {
    pub node: &'a Expr,
}

impl<'a> New<'a> {
    pub fn of(node: &'a Expr) -> Self {
        New { node }
    }

    /// The class and arguments, when the call really is an instantiation.
    pub fn resolve(&self, runtime: &Runtime) -> Option<(String, Vec<Expr>)> {
        let Expr::Call { callee, arguments } = self.node else {
            return None;
        };

        let Expr::Identifier(name) = callee.as_ref() else {
            return None;
        };

        if runtime.state.has_class(name) {
            Some((name.clone(), arguments.clone()))
        } else {
            None
        }
    }
}

impl Runtime {
    /// Recognises `Class(...)` on the right of an assignment, which names the
    /// new instance after what it is assigned to.
    pub(crate) fn instantiation(&self, value: &Expr) -> Option<(String, Vec<Expr>)> {
        New::of(value).resolve(self)
    }
}
