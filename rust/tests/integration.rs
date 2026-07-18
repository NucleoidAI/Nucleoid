use nucleoid::runtime::{NucleoidRuntime, ProcessOptions};

fn declarative() -> ProcessOptions {
    ProcessOptions { declarative: true }
}

#[test]
fn persists_state_across_calls() {
    let mut runtime = NucleoidRuntime::new().expect("engine starts");

    runtime.process("let a = 5;", declarative());
    let details = runtime.process("a + 1;", declarative());

    assert_eq!(details.result, Some(serde_json::json!(6)));
    assert!(details.error.is_none());
}

#[test]
fn reactively_recomputes_dependents() {
    let mut runtime = NucleoidRuntime::new().expect("engine starts");

    runtime.process("let a = 1;", declarative());
    runtime.process("let b = a + 1;", declarative());
    runtime.process("a = 10;", declarative());

    let details = runtime.process("b;", declarative());
    assert_eq!(details.result, Some(serde_json::json!(11)));
}

#[test]
fn cascades_through_a_diamond() {
    let mut runtime = NucleoidRuntime::new().expect("engine starts");

    runtime.process("let a = 1;", declarative());
    runtime.process("let b = a + 1;", declarative());
    runtime.process("let c = a + 2;", declarative());
    runtime.process("let d = b + c;", declarative());

    runtime.process("a = 10;", declarative());

    let details = runtime.process("d;", declarative());
    // b=11, c=12 -> d=23
    assert_eq!(details.result, Some(serde_json::json!(23)));
}

#[test]
fn detects_circular_dependency() {
    let mut runtime = NucleoidRuntime::new().expect("engine starts");

    runtime.process("let a = 1;", declarative());
    runtime.process("let b = a + 1;", declarative());

    // Redefining `a` in terms of `b` closes the cycle a -> b -> a; this is
    // the statement that introduces it, so it's the one that fails.
    let details = runtime.process("a = b + 1;", declarative());
    assert_eq!(details.error.as_deref(), Some("Circular Dependency"));

    // JS-visible state rolls back even though the graph edges don't.
    let after = runtime.process("a;", declarative());
    assert_eq!(after.result, Some(serde_json::json!(1)));
}

#[test]
fn rolls_back_on_error() {
    let mut runtime = NucleoidRuntime::new().expect("engine starts");

    runtime.process("let a = 1;", declarative());
    let details = runtime.process("a = 2; throw new Error('boom');", declarative());

    assert!(details.error.is_some());
    let after = runtime.process("a;", declarative());
    assert_eq!(after.result, Some(serde_json::json!(1)));
}

#[test]
fn records_history_in_datastore() {
    let mut runtime = NucleoidRuntime::new().expect("engine starts");

    runtime.process("let a = 1;", declarative());
    runtime.process("a + 1;", declarative());

    let tail = runtime.tail(10);
    assert_eq!(tail.len(), 2);
    assert_eq!(tail[0].source, "a + 1;");
}

#[test]
fn captures_events() {
    let mut runtime = NucleoidRuntime::new().expect("engine starts");

    let details = runtime.process("event('greeted', { who: 'world' });", declarative());

    assert_eq!(details.events.len(), 1);
    assert_eq!(details.events[0].name, "greeted");
}
