//! What an evaluation produced, and what it read to get there.
//!
//! `ref/src/lang/Evaluation.js` is a bare wrapper around a produced value;
//! [`Flow`] is the typed form of the same idea, and carries the one thing the
//! JavaScript version signals out of band — that a `return` ended the block.
//!
//! The dependency tracking sits here too: recording what an expression read is
//! part of evaluating it, and is what `ref` collects through
//! `Expression.graph()`.

use indexmap::IndexSet;

use crate::error::Result;
use crate::graph::NodeKey;
use crate::lang::ast::Expr;
use crate::runtime::Runtime;
use crate::scope::Scope;
use crate::value::Value;

/// How a statement finished: normally, or by returning out of its block.
pub enum Flow {
    Normal(Value),
    Return(Value),
}

#[derive(Debug, Clone, Default)]
pub(crate) struct Tracking {
    keys: IndexSet<NodeKey>,
    barrier: bool,
}

impl Runtime {
    pub(crate) fn push_tracking(&mut self, barrier: bool) {
        self.tracking.push(Tracking {
            keys: IndexSet::new(),
            barrier,
        });
    }

    pub(crate) fn pop_tracking(&mut self) -> IndexSet<NodeKey> {
        self.tracking
            .pop()
            .map(|frame| frame.keys)
            .unwrap_or_default()
    }

    /// Records a read. Reads reach every enclosing frame up to a barrier, so an
    /// enclosing block depends on whatever its statements read.
    pub(crate) fn track(&mut self, key: NodeKey) {
        for frame in self.tracking.iter_mut().rev() {
            frame.keys.insert(key.clone());

            if frame.barrier {
                break;
            }
        }
    }

    pub(crate) fn evaluate_tracked(
        &mut self,
        expression: &Expr,
        scope: &mut Scope,
        exclude: Option<&NodeKey>,
    ) -> Result<(Value, IndexSet<NodeKey>)> {
        let saved = self.null_read;
        self.null_read = false;
        self.push_tracking(false);
        let value = self.evaluate(expression, scope);
        let mut dependencies = self.pop_tracking();
        let value = self.settle(value?);
        self.null_read = saved;

        if let Some(exclude) = exclude {
            dependencies.shift_remove(exclude);
        }

        // Reads made while evaluating still belong to any enclosing declaration.
        for dependency in &dependencies {
            self.track(dependency.clone());
        }

        Ok((value, dependencies))
    }

    pub(crate) fn note_nullish(&mut self, value: &Value) {
        if value.is_null() {
            self.null_read = true;
        }
    }

    /// Applies the null and undefined rules to a value about to be stored.
    pub(crate) fn settle(&self, value: Value) -> Value {
        if value.is_undefined() || self.null_read {
            Value::Null
        } else {
            value
        }
    }
}
