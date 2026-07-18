use std::collections::{HashMap, HashSet, VecDeque};

/// Reactive dependency graph, the Rust-side stand-in for `src/graph.js` +
/// the recompute behaviour spread across `src/stack.js` / `src/Expression.js`.
///
/// Real Nucleoid derives dependencies from the ESTree AST. We don't have
/// that here, so dependencies are inferred by scanning an expression's
/// source text for identifiers that happen to be known variables
/// (`statement::referenced_vars`). This is a deliberate simplification —
/// it can't see through helper functions or object/array literals containing
/// identifiers wrapped in more complex expressions.
#[derive(Default)]
pub struct DependencyGraph {
    /// variable -> source text of the expression that (re)computes it.
    expressions: HashMap<String, String>,
    /// variable -> variables it reads.
    depends_on: HashMap<String, HashSet<String>>,
    /// variable -> variables that read it.
    dependents: HashMap<String, HashSet<String>>,
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record (or replace) the formula behind `name`, wiring up dependency
    /// edges against the current set of known variables.
    pub fn track(&mut self, name: &str, rhs: &str, known: &HashSet<String>) {
        // Drop this variable's old outgoing edges before recomputing them,
        // in case it was previously defined by a different expression.
        if let Some(old_deps) = self.depends_on.remove(name) {
            for dep in old_deps {
                if let Some(set) = self.dependents.get_mut(&dep) {
                    set.remove(name);
                }
            }
        }

        let refs: HashSet<String> = crate::statement::referenced_vars(rhs, known)
            .into_iter()
            .filter(|v| v != name)
            .collect();

        for dep in &refs {
            self.dependents
                .entry(dep.clone())
                .or_default()
                .insert(name.to_string());
        }

        self.depends_on.insert(name.to_string(), refs);
        self.expressions.insert(name.to_string(), rhs.to_string());
    }

    pub fn expression_of(&self, name: &str) -> Option<&String> {
        self.expressions.get(name)
    }

    /// Variables transitively dependent on `changed`, in an order safe to
    /// recompute in (each variable appears only after everything it itself
    /// depends on within the affected set). `Err` on a circular dependency,
    /// mirroring the `ReferenceError("Circular Dependency")` in `stack.js`.
    pub fn affected_topo_order(&self, changed: &str) -> Result<Vec<String>, String> {
        let mut affected: HashSet<String> = HashSet::new();
        let mut queue: VecDeque<String> = VecDeque::new();
        queue.push_back(changed.to_string());

        while let Some(current) = queue.pop_front() {
            if let Some(deps) = self.dependents.get(&current) {
                for d in deps {
                    if affected.insert(d.clone()) {
                        queue.push_back(d.clone());
                    }
                }
            }
        }

        if affected.is_empty() {
            return Ok(Vec::new());
        }

        let mut in_degree: HashMap<String, usize> =
            affected.iter().map(|v| (v.clone(), 0)).collect();

        for var in &affected {
            let deps = self.depends_on.get(var).cloned().unwrap_or_default();
            for dep in deps {
                // A dependency on `changed` itself doesn't block readiness —
                // the root of the cascade is already resolved. Only
                // dependencies on other not-yet-recomputed affected
                // variables should (that includes `changed` looping back
                // into `affected` in a genuine cycle).
                if affected.contains(&dep) {
                    *in_degree.get_mut(var).expect("in affected set") += 1;
                }
            }
        }

        let mut ready: VecDeque<String> = in_degree
            .iter()
            .filter(|(_, deg)| **deg == 0)
            .map(|(v, _)| v.clone())
            .collect();

        let mut order = Vec::new();

        while let Some(var) = ready.pop_front() {
            order.push(var.clone());

            if let Some(next) = self.dependents.get(&var) {
                for n in next {
                    if let Some(deg) = in_degree.get_mut(n) {
                        *deg -= 1;
                        if *deg == 0 {
                            ready.push_back(n.clone());
                        }
                    }
                }
            }
        }

        if order.len() != affected.len() {
            return Err("Circular Dependency".to_string());
        }

        Ok(order)
    }

    pub fn snapshot(&self) -> GraphSnapshot {
        GraphSnapshot {
            expressions: self.expressions.clone(),
            dependents: self
                .dependents
                .iter()
                .map(|(k, v)| (k.clone(), v.iter().cloned().collect()))
                .collect(),
        }
    }
}

#[derive(serde::Serialize)]
pub struct GraphSnapshot {
    pub expressions: HashMap<String, String>,
    pub dependents: HashMap<String, Vec<String>>,
}
