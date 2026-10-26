//! Retained values belong to storage slots, never to a second kind of value.

use crate::lang::ast::VersionSelector;
use crate::value::{ObjectId, Value};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum HistoryKey {
    Variable(String),
    Property(ObjectId, String),
}

#[derive(Debug, Clone, Default)]
pub(crate) struct History {
    values: Vec<Value>,
}

impl History {
    pub(crate) fn len(&self) -> usize {
        self.values.len()
    }

    pub(crate) fn push(&mut self, value: Value) {
        self.values.push(value);
    }

    pub(crate) fn truncate(&mut self, length: usize) {
        self.values.truncate(length);
    }

    pub(crate) fn get(&self, selector: VersionSelector) -> Option<&Value> {
        let index = match selector {
            VersionSelector::Index(index) => index,
            VersionSelector::First => 0,
            VersionSelector::Last => self.values.len().checked_sub(1)?,
            VersionSelector::Previous => self.values.len().checked_sub(2)?,
        };
        self.values.get(index)
    }
}
