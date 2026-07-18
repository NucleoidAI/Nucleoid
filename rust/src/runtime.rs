use std::collections::HashSet;

use crate::datastore::{now_ms, Datastore, Record};
use crate::engine::Engine;
use crate::event::EventRecord;
use crate::graph::DependencyGraph;
use crate::statement::{self, Parsed};

pub struct ProcessOptions {
    pub declarative: bool,
}

impl Default for ProcessOptions {
    fn default() -> Self {
        // Unlike upstream (which defaults to imperative), this prototype
        // defaults to declarative so the headline feature — reactive
        // recompute — is on by default when exercised over HTTP.
        ProcessOptions { declarative: true }
    }
}

#[derive(serde::Serialize)]
pub struct ProcessDetails {
    pub result: Option<serde_json::Value>,
    pub declarative: bool,
    pub date_ms: u128,
    pub time_ms: u128,
    pub error: Option<String>,
    pub events: Vec<EventRecord>,
    pub tracked: Vec<String>,
}

/// Combines the engine, dependency graph and datastore into the same
/// orchestration `src/runtime.js` (timing/events/datastore) and
/// `src/stack.js` (statement loop, dependency recompute) provide together.
pub struct NucleoidRuntime {
    engine: Engine,
    graph: DependencyGraph,
    datastore: Datastore,
    known_vars: HashSet<String>,
}

impl NucleoidRuntime {
    pub fn new() -> Result<Self, String> {
        Ok(NucleoidRuntime {
            engine: Engine::new()?,
            graph: DependencyGraph::new(),
            datastore: Datastore::new(),
            known_vars: HashSet::new(),
        })
    }

    pub fn process(&mut self, source: &str, options: ProcessOptions) -> ProcessDetails {
        let statements = statement::split(source);

        if statements.is_empty() {
            return ProcessDetails {
                result: None,
                declarative: options.declarative,
                date_ms: now_ms(),
                time_ms: 0,
                error: None,
                events: Vec::new(),
                tracked: Vec::new(),
            };
        }

        let before = now_ms();

        let snapshot = match crate::transaction::snapshot(&self.engine, &self.known_vars) {
            Ok(s) => s,
            Err(err) => {
                return ProcessDetails {
                    result: None,
                    declarative: options.declarative,
                    date_ms: now_ms(),
                    time_ms: 0,
                    error: Some(err),
                    events: Vec::new(),
                    tracked: Vec::new(),
                }
            }
        };

        let mut declarative = options.declarative;
        let mut last_value: Option<String> = None;
        let mut tracked: Vec<String> = Vec::new();
        let mut error: Option<String> = None;

        'statements: for stmt in &statements {
            let parsed = statement::parse(stmt);

            match self.engine.eval(stmt) {
                Ok(value) => {
                    if value.as_deref() == Some("\"use declarative\"") {
                        declarative = true;
                    } else if value.as_deref() == Some("\"use imperative\"") {
                        declarative = false;
                    }

                    last_value = value;
                }
                Err(err) => {
                    error = Some(err);
                    break 'statements;
                }
            }

            if let Parsed::Assignment { name, rhs, .. } = parsed {
                self.known_vars.insert(name.clone());

                if declarative {
                    self.graph.track(&name, rhs, &self.known_vars);
                    tracked.push(name.clone());

                    match self.graph.affected_topo_order(&name) {
                        Ok(order) => {
                            for dependent in order {
                                let Some(expr) = self.graph.expression_of(&dependent).cloned()
                                else {
                                    continue;
                                };

                                match self.engine.eval(&format!("{dependent} = ({expr});")) {
                                    Ok(_) => tracked.push(dependent),
                                    Err(err) => {
                                        error = Some(err);
                                        break 'statements;
                                    }
                                }
                            }
                        }
                        Err(err) => {
                            error = Some(err);
                            break 'statements;
                        }
                    }
                }
            }
        }

        if error.is_some() {
            // Best-effort: restores JS-visible values; dependency-graph
            // bookkeeping accumulated during the failed pass is left as-is.
            let _ = crate::transaction::restore(&self.engine, &snapshot);
        }

        let time_ms = now_ms().saturating_sub(before);
        let date_ms = now_ms();

        let events = self.engine.drain_events().unwrap_or_default();

        let result = last_value.and_then(|text| serde_json::from_str(&text).ok());

        let record = Record {
            source: source.to_string(),
            result: result.clone(),
            declarative,
            time_ms,
            date_ms,
            error: error.clone(),
            events: events.clone(),
            tracked: tracked.clone(),
        };
        self.datastore.write(record);

        ProcessDetails {
            result,
            declarative,
            date_ms,
            time_ms,
            error,
            events,
            tracked,
        }
    }

    pub fn tail(&self, n: usize) -> Vec<Record> {
        self.datastore.tail(n).into_iter().cloned().collect()
    }

    pub fn clear(&mut self) {
        self.datastore.clear();
    }

    pub fn graph_snapshot(&self) -> crate::graph::GraphSnapshot {
        self.graph.snapshot()
    }
}
