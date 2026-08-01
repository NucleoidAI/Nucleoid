//! Instantiation. Mirrors `ref/src/lang/ast/New.js`.
//!
//! Nucleoid has no `new` keyword, so there is no node kind for this and
//! [`New`] is not something [`crate::lang::ast::Ast::convert`] ever produces:
//! `Class(...)` is an ordinary [`call`](crate::lang::ast::call) until the name
//! turns out to be a class. Deciding that is all this does. Creating the
//! instance is [`crate::nuc::object`], matching `ref/src/nuc/OBJECT.js`.

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
