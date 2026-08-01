//! Instantiation. Mirrors `ref/src/lang/ast/New.js`, which — like Nucleoid —
//! treats `Class(...)` as an ordinary call until the name turns out to be a
//! class.
//!
//! Creating the instance itself is [`crate::nuc::object`], matching
//! `ref/src/nuc/OBJECT.js`.

use crate::lang::ast::Expr;
use crate::runtime::Runtime;

impl Runtime {
    /// Recognises `Class(...)` on the right of an assignment, which names the
    /// new instance after what it is assigned to.
    pub(crate) fn instantiation(&self, value: &Expr) -> Option<(String, Vec<Expr>)> {
        let Expr::Call { callee, arguments } = value else {
            return None;
        };

        let Expr::Identifier(name) = callee.as_ref() else {
            return None;
        };

        if self.state.classes.contains_key(name) {
            Some((name.clone(), arguments.clone()))
        } else {
            None
        }
    }
}
