//! The propagation queue: what is waiting to be re-evaluated, and the loop that
//! works through it. Mirrors `ref/src/stack.js`.
//!
//! Propagation is a queue rather than recursion, so the length of a dependency
//! chain is not limited by the call stack.

use indexmap::IndexSet;
use std::cmp::Reverse;
use std::collections::BinaryHeap;

use crate::error::{Error, Result};
use crate::graph::NodeKey;
use crate::runtime::Runtime;
use crate::scope::Scope;

/// How many statements one change may re-evaluate before the runtime decides it
/// is not settling. Cycles are refused when they are declared, so this is a
/// backstop that should never be reached — it is set low enough to fail quickly
/// rather than appear to hang.
const MAX_PROPAGATION: usize = 100_000;

/// Statements waiting to be re-evaluated, ordered by when they were declared.
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

    fn pop(&mut self) -> Option<NodeKey> {
        let Reverse((_, key)) = self.pending.pop()?;
        self.queued.swap_remove(&key);
        Some(key)
    }

    fn clear(&mut self) {
        self.pending.clear();
        self.queued.clear();
    }
}

impl Runtime {
    /// Notes that everything reading `key` is now out of date. The work is done
    /// by whoever is draining the queue, which keeps propagation flat however
    /// long the chain is.
    pub(crate) fn propagate(&mut self, key: &NodeKey) -> Result<()> {
        for dependent in self.graph.dependents_in_order(key) {
            self.enqueue(dependent);
        }

        if self.stack.is_draining() {
            return Ok(());
        }

        self.stack.draining = true;
        let result = self.drain();
        self.stack.draining = false;

        if result.is_err() {
            self.stack.clear();
        }

        result
    }

    fn enqueue(&mut self, key: NodeKey) {
        if self.stack.running.contains(&key) {
            return;
        }

        let sequence = self
            .graph
            .retrieve(&key)
            .map(|node| node.sequence)
            .unwrap_or(u64::MAX);

        self.stack.push(sequence, key);
    }

    /// Re-evaluates what is waiting, in the order it was declared, until nothing
    /// is left. A statement re-run here may queue more work, which this same
    /// loop picks up.
    fn drain(&mut self) -> Result<()> {
        // Once each. A statement that writes something it also reads would
        // otherwise wake itself for as long as it kept changing.
        let mut settled: IndexSet<NodeKey> = IndexSet::new();
        let mut steps = 0usize;

        while let Some(dependent) = self.stack.pop() {
            steps += 1;

            if steps > MAX_PROPAGATION {
                return Err(Error::type_error("Propagation did not settle"));
            }

            if !settled.insert(dependent.clone()) {
                continue;
            }

            let Some(node) = self.graph.retrieve(&dependent).cloned() else {
                continue;
            };

            let Some(mut nuc) = node.node.clone() else {
                continue;
            };

            self.stack.running.insert(dependent.clone());

            let mut scope = Scope::new();
            scope.set_instance(node.instance.clone());

            let nested = self.suspend_imperative();
            if let Some(instance) = &node.instance {
                self.instances.push(instance.clone());
            }
            self.push_tracking(true);
            let result = self.rerun(&mut nuc, &mut scope);
            self.pop_tracking();
            if node.instance.is_some() {
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
