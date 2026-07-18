use serde::{Deserialize, Serialize};

/// Mirrors `src/event.js`: a user statement can call `event(name, data)`,
/// and every emitted event since the last drain rides along with that
/// statement's result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventRecord {
    pub name: String,
    /// JSON-encoded payload, matching the original's `JSON.stringify(data)`.
    pub data: String,
}
