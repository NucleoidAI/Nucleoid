//! The runtime is a library: bad input has to come back as an error rather
//! than a panic, a hang, or a blown stack.

use nucleoid::Runtime;

fn rejects(source: &str) {
    let mut runtime = Runtime::new();

    if let Ok(value) = runtime.run(source) {
        panic!("expected an error from {source:?}, got {value}");
    }
}

fn survives(source: &str) {
    let mut runtime = Runtime::new();
    let _ = runtime.run(source);
}

#[test]
fn malformed_source_is_rejected() {
    rejects("a = ");
    rejects("a = \"unterminated");
    rejects("a = (1 + 2");
    rejects("a = [1, 2");
    rejects("{ a = 1");
    rejects("if a");
    rejects("class");
    rejects("def (");
    rejects("a = 1 +* 2");
    rejects("for x of");
    rejects("a = @");
}

#[test]
fn undefined_names_are_reported_not_panicked() {
    rejects("a = missing + 1");
    rejects("missing()");
    rejects("missing.property = 1");
    rejects("delete missing.deeper.deepest");
    rejects("a = Missing()");
}

#[test]
fn deeply_nested_expressions_do_not_blow_the_stack() {
    let depth = 200;
    let source = format!("a = {}1{}", "(".repeat(depth), ")".repeat(depth));
    survives(&source);

    let chained: String = (0..500).map(|index| format!(" + {index}")).collect();
    survives(&format!("b = 0{chained}"));
}

#[test]
fn cyclic_objects_compare_without_recursing_forever() {
    let mut runtime = Runtime::new();

    runtime.run("class Node:\n    pass\n").unwrap();
    runtime.run("first = Node()").unwrap();
    runtime.run("second = Node()").unwrap();
    runtime.run("first.link = first").unwrap();
    runtime.run("second.link = second").unwrap();

    // Structurally identical but distinct, so the comparison walks both.
    runtime.run("assert(first, second)").unwrap();
    let _ = runtime.take_assertions();
}

#[test]
fn mutual_dependencies_terminate() {
    let mut runtime = Runtime::new();

    runtime.run("a = 1").unwrap();
    runtime.run("b = a + 1").unwrap();

    // Closing the loop is refused rather than looping.
    assert!(runtime.run("a = b + 1").is_err());
    assert_eq!(runtime.run("a").unwrap().to_string(), "1");
}

#[test]
fn a_rule_that_feeds_itself_terminates() {
    let mut runtime = Runtime::new();

    runtime.run("class Counter:\n    pass\n").unwrap();
    runtime.run("counter1 = Counter()").unwrap();
    runtime.run("counter1.count = 0").unwrap();

    // Self-reference reads the current value, so this settles instead of
    // running away.
    runtime.run("counter1.count = counter1.count + 1").unwrap();
    assert_eq!(runtime.run("counter1.count").unwrap().to_string(), "1");
}

/// Propagation used to recurse, so a chain longer than about forty links could
/// not be updated at all — the depth limit stopped it. It is a queue now.
#[test]
fn a_long_dependency_chain_still_updates() {
    let mut runtime = Runtime::new();
    let length = 500;

    let mut source = String::from("v0 = 1\n");
    for index in 1..length {
        source.push_str(&format!("v{index} = v{} + 1\n", index - 1));
    }

    runtime.run(&source).expect("the chain should build");
    assert_eq!(runtime.run("v499").unwrap().to_string(), "500");

    runtime.run("v0 = 2").expect("the head should reassign");
    assert_eq!(runtime.run("v499").unwrap().to_string(), "501");
}

#[test]
fn a_failed_run_leaves_the_state_untouched() {
    let mut runtime = Runtime::new();

    runtime.run("a = 5").unwrap();
    runtime.run("if a > 5:\n    throw 'TOO_BIG'\n").unwrap();

    assert!(runtime.run("a = 6").is_err());
    assert_eq!(runtime.run("a").unwrap().to_string(), "5");
}

#[test]
fn an_empty_program_is_fine() {
    let mut runtime = Runtime::new();

    assert_eq!(runtime.run("").unwrap().to_string(), "null");
    assert_eq!(
        runtime
            .run("   \n\n  # only a comment\n")
            .unwrap()
            .to_string(),
        "null"
    );
}

#[test]
fn unicode_source_round_trips() {
    let mut runtime = Runtime::new();

    runtime.run("greeting = \"héllo wörld\"").unwrap();
    assert_eq!(runtime.run("greeting.length").unwrap().to_string(), "11");
    assert_eq!(runtime.run("greeting[1]").unwrap().to_string(), "é");
}
