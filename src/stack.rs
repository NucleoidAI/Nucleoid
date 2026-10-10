//! The propagation queue: what is waiting to be re-evaluated, and the loop that
//! works through it.
//!
//! Propagation is a queue rather than recursion, so the length of a dependency
//! chain is not limited by the call stack.

use indexmap::IndexSet;
use std::cmp::Reverse;
use std::collections::BinaryHeap;

use crate::error::{Error, Result};
use crate::graph::{Graph, NodeKey, ShapeKey};
use crate::lang::evaluation::TrackingMode;
use crate::runtime::Runtime;
use crate::scope::Scope;

/// How many statements one change may re-evaluate before the runtime decides it
/// is not settling. The dependency graph is acyclic, but imperative rule bodies
/// can feed changes back into earlier inputs.
const MAX_PROPAGATION: usize = 100_000;

/// Statements waiting to be re-evaluated. Upstream work runs first; otherwise
/// declaration order decides which statement is next.
#[derive(Debug, Clone, Default)]
pub struct Stack {
    pending: BinaryHeap<Reverse<(u64, NodeKey)>>,
    queued: IndexSet<NodeKey>,
    /// What is running right now. A statement does not re-run itself: what it
    /// writes while running is its own doing, not news to it.
    running: IndexSet<NodeKey>,
    draining: bool,
}

impl Stack {
    pub fn new() -> Self {
        Stack::default()
    }

    pub fn is_draining(&self) -> bool {
        self.draining
    }

    pub(crate) fn is_running(&self, key: &NodeKey) -> bool {
        self.running.contains(key)
    }

    pub fn len(&self) -> usize {
        self.pending.len()
    }

    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }

    fn push(&mut self, sequence: u64, key: NodeKey) {
        if self.queued.insert(key.clone()) {
            self.pending.push(Reverse((sequence, key)));
        }
    }

    fn pop(&mut self, graph: &Graph) -> Result<Option<NodeKey>> {
        let mut blocked = Vec::new();
        let mut next = None;

        while let Some(Reverse((sequence, key))) = self.pending.pop() {
            if let Some(node) = graph.retrieve(&key) {
                if node.sequence != sequence {
                    self.pending.push(Reverse((node.sequence, key)));
                    continue;
                }
            }

            if self.queued.len() > 1 && graph.depends_on_any(&key, &self.queued) {
                blocked.push(Reverse((sequence, key)));
                continue;
            }

            self.queued.swap_remove(&key);
            next = Some(key);
            break;
        }

        self.pending.extend(blocked);

        if next.is_none() && !self.pending.is_empty() {
            return Err(Error::type_error("Circular Dependency"));
        }

        Ok(next)
    }

    fn clear(&mut self) {
        self.pending.clear();
        self.queued.clear();
    }
}

impl Runtime {
    /// Explicit assignments still trigger their rules. Inside a cascade, an
    /// unchanged value must not keep feedback rules waking each other.
    pub(crate) fn propagate_change(&mut self, key: &NodeKey, changed: bool) -> Result<()> {
        if changed || !self.stack.is_draining() {
            self.propagate(key)
        } else {
            Ok(())
        }
    }

    /// Notes that everything reading `key` is now out of date. The work is done
    /// by whoever is draining the queue, which keeps propagation flat however
    /// long the chain is.
    pub(crate) fn propagate(&mut self, key: &NodeKey) -> Result<()> {
        for dependent in self.graph.dependents_in_order(key) {
            self.enqueue(dependent);
        }

        self.drain_pending()
    }

    pub(crate) fn queue_shape_change(&mut self, source: &ShapeKey) {
        let observers: Vec<NodeKey> = self.graph.shape_dependents(source).cloned().collect();
        for observer in observers {
            self.enqueue(observer);
        }
    }

    pub(crate) fn drain_pending(&mut self) -> Result<()> {
        if self.stack.is_draining() || self.stack.is_empty() {
            return Ok(());
        }

        let deleted = self.deleted.clone();
        self.stack.draining = true;
        let result = self.drain();
        self.stack.draining = false;
        self.deleted = deleted;

        if result.is_err() {
            self.stack.clear();
        }

        result
    }

    fn enqueue(&mut self, key: NodeKey) {
        if self.stack.is_running(&key) {
            return;
        }

        let sequence = self
            .graph
            .retrieve(&key)
            .map(|node| node.sequence)
            .unwrap_or(u64::MAX);

        self.stack.push(sequence, key);
    }

    /// Re-evaluates ready work in declaration order, waiting for queued upstream
    /// changes. A statement re-run here may queue more work, which this same loop
    /// picks up.
    fn drain(&mut self) -> Result<()> {
        let mut steps = 0usize;

        while let Some(dependent) = self.stack.pop(&self.graph)? {
            steps += 1;

            if steps > MAX_PROPAGATION {
                return Err(Error::type_error("Propagation did not settle"));
            }

            let Some(node) = self.graph.retrieve(&dependent) else {
                continue;
            };

            let Some(mut nuc) = node.node.clone() else {
                continue;
            };
            let instance = node.instance.clone();

            self.stack.running.insert(dependent.clone());

            let mut scope = Scope::new();
            scope.set_instance(instance.clone());

            let nested = self.suspend_imperative();
            if let Some(instance) = &instance {
                self.instances.push(instance.clone());
            }
            let result = self.with_tracking(TrackingMode::Isolated, |runtime| {
                runtime.rerun(&mut nuc, &mut scope)
            });
            if instance.is_some() {
                self.instances.pop();
            }
            self.restore_imperative(nested);

            self.stack.running.shift_remove(&dependent);
            result?;
        }

        Ok(())
    }

    /// Re-runs everything that read what was just removed. Each dependent finds
    /// the name undefined, settles to null, and passes that on to its own
    /// dependents, so a whole chain clears.
    pub(crate) fn cascade_removal(&mut self, key: &NodeKey) -> Result<()> {
        self.deleted.insert(key.clone());
        let outermost = !self.stack.is_draining();
        let result = self.propagate(key);

        // Only the run that actually drained the queue is finished with it; a
        // nested deletion leaves the name undefined until the outer drain ends.
        if outermost {
            self.deleted.shift_remove(key);
        }

        result
    }
}
