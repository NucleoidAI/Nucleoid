use std::collections::HashMap;

use crate::engine::Engine;

/// Mirrors `src/transaction.js`: snapshot every known variable's value
/// before processing a statement, and restore all of them verbatim if
/// anything in that statement fails.
///
/// This is coarser than the original, which patches individual
/// `object[property]` writes as they happen. Snapshotting whole variables up
/// front is simpler and correct for the cases this prototype tracks
/// (top-level `let`/`const`/`var` variables), but — like the original — it
/// won't see through mutations of nested object/array contents.
pub struct Snapshot(HashMap<String, Option<String>>);

pub fn snapshot(engine: &Engine, known_vars: &std::collections::HashSet<String>) -> Result<Snapshot, String> {
    let mut values = HashMap::new();
    for name in known_vars {
        values.insert(name.clone(), engine.read(name)?);
    }
    Ok(Snapshot(values))
}

pub fn restore(engine: &Engine, snapshot: &Snapshot) -> Result<(), String> {
    for (name, value) in &snapshot.0 {
        engine.restore(name, value)?;
    }
    Ok(())
}
