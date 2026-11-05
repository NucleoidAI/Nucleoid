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

use crate::error::{Error, Result};
use crate::graph::{NodeKey, ShapeKey};
use crate::lang::ast::{Expr, VersionSelector};
use crate::runtime::Runtime;
use crate::scope::Scope;
use crate::value::{ObjectId, Value};

/// How a statement finished: normally, or by returning out of its block.
#[derive(Debug)]
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
    target: Option<NodeKey>,
    anchored: bool,
    pending_anchor: bool,
}

impl Tracking {
    fn reachable(&self) -> &[TrackingFrame] {
        self.frames.get(self.barrier()..).unwrap_or_default()
    }

    fn reachable_mut(&mut self) -> &mut [TrackingFrame] {
        let barrier = self.barrier();
        self.frames.get_mut(barrier..).unwrap_or_default()
    }

    fn barrier(&self) -> usize {
        self.frames
            .iter()
            .rposition(|frame| frame.mode == TrackingMode::Isolated)
            .unwrap_or(0)
    }
}

pub(crate) struct PropertyEvaluation {
    pub value: Value,
    pub dependencies: IndexSet<NodeKey>,
    /// A self-history must not manufacture the version it is waiting for.
    pub pending_anchor: bool,
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
        self.tracked_assignment(mode, None, false, operation)
    }

    fn tracked_assignment<T>(
        &mut self,
        mode: TrackingMode,
        target: Option<NodeKey>,
        anchored: bool,
        operation: impl FnOnce(&mut Self) -> Result<T>,
    ) -> Result<(T, TrackingFrame)> {
        self.tracking.frames.push(TrackingFrame {
            keys: IndexSet::new(),
            shapes: IndexSet::new(),
            mode,
            target,
            anchored,
            pending_anchor: false,
        });
        let depth = self.tracking.frames.len();
        let result = operation(self);
        debug_assert_eq!(self.tracking.frames.len(), depth);
        let frame = self
            .tracking
            .frames
            .pop()
            .ok_or_else(|| Error::type_error("Dependency tracking lost its frame"))?;

        result.map(|value| (value, frame))
    }

    /// Records a read. Reads reach every enclosing frame up to a barrier, so an
    /// enclosing block depends on whatever its statements read.
    pub(crate) fn track(&mut self, key: &NodeKey) {
        for frame in self.tracking.reachable_mut() {
            frame.keys.insert(key.clone());
        }
    }

    /// History still observes its real slot. Only an unavailable fixed
    /// self-anchor defers an assignment on the active class receiver.
    pub(crate) fn track_property_version(
        &mut self,
        key: &NodeKey,
        selector: VersionSelector,
        available: bool,
    ) {
        self.track(key);
        if available || !selector.is_fixed() {
            return;
        }

        for frame in self.tracking.reachable_mut() {
            if frame.anchored && frame.target.as_ref() == Some(key) {
                frame.pending_anchor = true;
            }
        }
    }

    pub(crate) fn track_shape(&mut self, key: &ShapeKey) {
        for frame in self.tracking.reachable_mut() {
            frame.shapes.insert(key.clone());
        }
    }

    pub(crate) fn declares(&self, key: &NodeKey) -> bool {
        self.tracking
            .reachable()
            .iter()
            .any(|frame| frame.target.as_ref() == Some(key))
    }

    pub(crate) fn evaluate_tracked(
        &mut self,
        expression: &Expr,
        scope: &mut Scope,
        exclude: Option<&NodeKey>,
    ) -> Result<(Value, IndexSet<NodeKey>)> {
        let evaluated = self.evaluate_assignment(expression, scope, exclude.cloned(), false)?;
        Ok((evaluated.value, evaluated.dependencies))
    }

    pub(crate) fn evaluate_property_tracked(
        &mut self,
        expression: &Expr,
        scope: &mut Scope,
        object: &ObjectId,
        property: &str,
    ) -> Result<PropertyEvaluation> {
        let key = NodeKey::property(object, property);
        let anchored = scope.instance() == Some(object);
        self.evaluate_assignment(expression, scope, Some(key), anchored)
    }

    fn evaluate_assignment(
        &mut self,
        expression: &Expr,
        scope: &mut Scope,
        target: Option<NodeKey>,
        anchored: bool,
    ) -> Result<PropertyEvaluation> {
        let saved = std::mem::take(&mut self.null_read);
        let result =
            self.tracked_assignment(TrackingMode::Inherited, target, anchored, |runtime| {
                let value = runtime.evaluate(expression, scope)?;
                Ok(runtime.settle(value))
            });
        self.null_read = saved;
        let (value, frame) = result?;
        let mut dependencies = frame.keys;

        if let Some(declared) = &frame.target {
            dependencies.shift_remove(declared);
        }

        // Reads made while evaluating still belong to any enclosing declaration.
        for dependency in &dependencies {
            self.track(dependency);
        }

        Ok(PropertyEvaluation {
            value: if frame.pending_anchor {
                Value::Null
            } else {
                value
            },
            dependencies,
            pending_anchor: frame.pending_anchor,
        })
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

    pub(crate) fn evaluate_settled(
        &mut self,
        expression: &Expr,
        scope: &mut Scope,
    ) -> Result<Value> {
        let null_read = std::mem::take(&mut self.null_read);
        let undefined_read = std::mem::take(&mut self.undefined_read);
        let value = self
            .evaluate(expression, scope)
            .map(|value| self.settle(value));
        let read_undefined = std::mem::replace(&mut self.undefined_read, undefined_read);
        self.null_read = null_read;
        let value = value?;

        if !value.is_null() {
            self.undefined_read |= read_undefined;
        }

        Ok(value)
    }

    pub(crate) fn evaluate_condition(
        &mut self,
        condition: &Expr,
        scope: &mut Scope,
    ) -> Result<(Value, bool)> {
        let enclosing = std::mem::take(&mut self.undefined_read);
        let value = self.evaluate(condition, scope);
        let unresolved = std::mem::replace(&mut self.undefined_read, enclosing);
        Ok((value?, unresolved))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame(mode: TrackingMode, target: &str, anchored: bool) -> TrackingFrame {
        TrackingFrame {
            keys: IndexSet::new(),
            shapes: IndexSet::new(),
            mode,
            target: Some(NodeKey::variable(target)),
            anchored,
            pending_anchor: false,
        }
    }

    fn reached(runtime: &mut Runtime, targets: &[&str]) -> Vec<[bool; 4]> {
        let read = NodeKey::variable("read");
        runtime.track(&read);
        runtime.track_shape(&ShapeKey::Model);
        let declared: Vec<bool> = targets
            .iter()
            .map(|target| runtime.declares(&NodeKey::variable(*target)))
            .collect();
        for target in targets {
            runtime.track_property_version(
                &NodeKey::variable(*target),
                VersionSelector::First,
                false,
            );
        }
        assert!(!runtime.declares(&read));
        runtime
            .tracking
            .frames
            .iter()
            .zip(declared)
            .map(|(frame, declared)| {
                [
                    frame.keys.contains(&read),
                    frame.shapes.contains(&ShapeKey::Model),
                    frame.pending_anchor,
                    declared,
                ]
            })
            .collect()
    }

    #[test]
    fn reads_reach_every_frame_up_to_the_innermost_isolated_frame() {
        let mut runtime = Runtime::new();
        assert_eq!(reached(&mut runtime, &[]), Vec::<[bool; 4]>::new());

        runtime.tracking.frames = vec![
            frame(TrackingMode::Inherited, "outer", true),
            frame(TrackingMode::Inherited, "inner", true),
        ];
        assert_eq!(reached(&mut runtime, &["outer", "inner"]), [[true; 4]; 2]);

        runtime.tracking.frames = vec![
            frame(TrackingMode::Inherited, "outer", true),
            frame(TrackingMode::Isolated, "shadowed", true),
            frame(TrackingMode::Inherited, "between", true),
            frame(TrackingMode::Isolated, "barrier", true),
            frame(TrackingMode::Inherited, "unanchored", false),
            frame(TrackingMode::Inherited, "inner", true),
        ];
        assert_eq!(
            reached(
                &mut runtime,
                &[
                    "outer",
                    "shadowed",
                    "between",
                    "barrier",
                    "unanchored",
                    "inner"
                ]
            ),
            [
                [false; 4],
                [false; 4],
                [false; 4],
                [true; 4],
                [true, true, false, true],
                [true; 4],
            ]
        );
    }
}
