//! What an evaluation produced, and what it read to get there.
//!
//! [`Flow`] carries the produced value and distinguishes normal completion
//! from a `return` that ends the block.
//!
//! Dependency tracking records each read as the expression is evaluated.
//! Frames belong to scoped operations and are removed before returning either
//! a value or an error. Local and graph assignments share the same evaluator,
//! which also restores the enclosing null-tracking state on both paths.

use indexmap::IndexSet;

use crate::error::Result;
use crate::graph::{NodeKey, ShapeKey};
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
    frames: Vec<TrackingFrame>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TrackingMode {
    Inherited,
    Isolated,
}

#[derive(Debug, Clone)]
struct TrackingFrame {
    keys: IndexSet<NodeKey>,
    shapes: IndexSet<ShapeKey>,
    mode: TrackingMode,
}

impl Runtime {
    /// Records reads for one operation. Isolated frames keep their reads out
    /// of enclosing declarations; inherited frames let those reads through.
    pub(crate) fn with_tracking<T>(
        &mut self,
        mode: TrackingMode,
        operation: impl FnOnce(&mut Self) -> Result<T>,
    ) -> Result<(T, IndexSet<NodeKey>)> {
        self.tracked(mode, operation)
            .map(|(value, frame)| (value, frame.keys))
    }

    pub(crate) fn with_shape_tracking<T>(
        &mut self,
        mode: TrackingMode,
        operation: impl FnOnce(&mut Self) -> Result<T>,
    ) -> Result<(T, IndexSet<ShapeKey>)> {
        self.tracked(mode, operation)
            .map(|(value, frame)| (value, frame.shapes))
    }

    fn tracked<T>(
        &mut self,
        mode: TrackingMode,
        operation: impl FnOnce(&mut Self) -> Result<T>,
    ) -> Result<(T, TrackingFrame)> {
        self.tracking.frames.push(TrackingFrame {
            keys: IndexSet::new(),
            shapes: IndexSet::new(),
            mode,
        });
        let result = operation(self);
        let frame = self
            .tracking
            .frames
            .pop()
            .expect("a scoped tracking operation must retain its frame");

        result.map(|value| (value, frame))
    }

    /// Records a read. Reads reach every enclosing frame up to a barrier, so an
    /// enclosing block depends on whatever its statements read.
    pub(crate) fn track(&mut self, key: NodeKey) {
        for frame in self.tracking.frames.iter_mut().rev() {
            frame.keys.insert(key.clone());

            if frame.mode == TrackingMode::Isolated {
                break;
            }
        }
    }

    pub(crate) fn track_shape(&mut self, key: ShapeKey) {
        for frame in self.tracking.frames.iter_mut().rev() {
            frame.shapes.insert(key.clone());

            if frame.mode == TrackingMode::Isolated {
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
        let saved = std::mem::take(&mut self.null_read);
        let result = self.with_tracking(TrackingMode::Inherited, |runtime| {
            let value = runtime.evaluate(expression, scope)?;
            Ok(runtime.settle(value))
        });
        self.null_read = saved;
        let (value, mut dependencies) = result?;

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
