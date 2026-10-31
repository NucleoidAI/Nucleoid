use indexmap::IndexSet;

use crate::error::Result;
use crate::lang::evaluation::{Flow, TrackingMode};
use crate::nuc::{Nuc, Outcome};
use crate::runtime::Runtime;
use crate::scope::Scope;

#[derive(Debug, Clone)]
pub struct ValueAssign {
    pub assignment: Box<Nuc>,
}

impl ValueAssign {
    pub fn new(assignment: Nuc) -> Self {
        ValueAssign {
            assignment: Box::new(assignment),
        }
    }

    pub fn run(&mut self, runtime: &mut Runtime, scope: &mut Scope) -> Result<Outcome> {
        let assignment = self.assignment.as_mut();
        let (outcome, _) = runtime.with_tracking(TrackingMode::Isolated, |runtime| {
            assignment.before(runtime, scope)?;
            assignment.run(runtime, scope)
        })?;

        if let Flow::Normal(value) = &outcome.flow {
            match assignment {
                Nuc::Variable(node) => node.hold(value)?,
                Nuc::Property(node) => node.hold(value)?,
                _ => {}
            }
        }

        Ok(Outcome::flow(outcome.flow))
    }

    pub fn graph(&self, runtime: &mut Runtime) -> Result<()> {
        self.assignment.graph(runtime, IndexSet::new())
    }

    pub fn after(&self, runtime: &mut Runtime) -> Result<()> {
        self.assignment.after(runtime)
    }
}
