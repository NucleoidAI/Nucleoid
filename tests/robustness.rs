//! The runtime is a library: bad input has to come back as an error rather
//! than a panic, a hang, or a blown stack.

use nucleoid::{ErrorKind, Runtime};

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
fn long_prefix_chains_are_rejected_without_overflowing() {
    for prefix in ["-", "!", "not ", "typeof ", "why ", "new ", "delete "] {
        let source = format!("result = {}input", prefix.repeat(4096));
        let error = Runtime::check(&source).expect_err("prefix chains must respect nesting limits");
        assert_eq!(error.kind(), ErrorKind::Syntax);
        assert_eq!(error.message(), "Expressions are nested too deeply");
    }

    assert!(Runtime::check(&format!("{}1", "-".repeat(62))).is_ok());
    assert!(Runtime::check(&format!("{}1", "-".repeat(63))).is_err());

    let mut runtime = Runtime::new();
    assert_eq!(runtime.run("- - 3").unwrap().to_string(), "3");
    assert_eq!(runtime.run("not not true").unwrap().to_string(), "true");
}

#[test]
fn deep_implicit_inheritance_does_not_recurse_on_the_host_stack() {
    let mut runtime = Runtime::new();
    runtime
        .run("class Base(amount):\n    this.amount = amount")
        .unwrap();
    let mut parent = String::from("Base");

    for index in 0..500 {
        let class = format!("Level{index}");
        runtime
            .run(&format!("class {class}: {parent}\n    pass"))
            .unwrap();
        parent = class;
    }

    runtime.run(&format!("last = {parent}(42)")).unwrap();
    assert_eq!(runtime.run("last.amount").unwrap().to_string(), "42");
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

#[test]
fn oversized_string_repetitions_are_errors_not_panics() {
    let mut runtime = Runtime::new();
    runtime.run("saved = 5").unwrap();

    for receiver in [r#""a""#, r#""ab""#, "String.fromCharCode(233)"] {
        for count in ["Number.POSITIVE_INFINITY", "Number.MAX_VALUE"] {
            let source = format!("saved = 6\nresult = {receiver}.repeat({count})");
            let error = runtime
                .run(&source)
                .expect_err("oversized repetitions must return an error");

            assert_eq!(error.kind(), ErrorKind::Type);
            assert_eq!(error.message(), "Repeated string is too large");
            assert_eq!(
                error
                    .position()
                    .map(|position| (position.line, position.column)),
                Some((2, 1))
            );
            assert_eq!(runtime.run("saved").unwrap().to_string(), "5");
            assert!(runtime.state.variable("result").is_none());
            assert_eq!(
                runtime.run(r#""ab".repeat(3)"#).unwrap().to_string(),
                "ababab"
            );
        }
    }
}
