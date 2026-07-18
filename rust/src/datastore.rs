use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::event::EventRecord;

/// One processed statement, written after every `NucleoidRuntime::process`
/// call — mirrors the object shape `runtime.js` hands to `datastore.write`.
#[derive(Debug, Clone, Serialize)]
pub struct Record {
    pub source: String,
    pub result: Option<serde_json::Value>,
    pub declarative: bool,
    pub time_ms: u128,
    pub date_ms: u128,
    pub error: Option<String>,
    pub events: Vec<EventRecord>,
    pub tracked: Vec<String>,
}

/// In-memory store, equivalent to `src/cache.js` (the default when
/// `@nucleoidjs/datastore` — persistent storage — isn't configured).
#[derive(Default)]
pub struct Datastore {
    records: Vec<Record>,
}

impl Datastore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn write(&mut self, record: Record) {
        self.records.push(record);
    }

    pub fn read(&self) -> &[Record] {
        &self.records
    }

    pub fn tail(&self, n: usize) -> Vec<&Record> {
        self.records.iter().rev().take(n).collect()
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }
}

pub fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}
