//! Runs the Nucleoid snippets in `README.md`.
//!
//! They are the first thing anyone reads, so they are executed from the file
//! itself rather than copied here — there is nothing to drift.

mod common;

const README: &str = include_str!("../README.md");

/// Every ```nucleoid fenced block in the README, in order.
fn snippets() -> Vec<common::Case> {
    let mut cases = Vec::new();
    let mut rest = README;
    let mut index = 0;

    while let Some(start) = rest.find("```nucleoid") {
        let after = &rest[start + "```nucleoid".len()..];

        let Some(end) = after.find("```") else {
            panic!("unterminated ```nucleoid block in README.md");
        };

        index += 1;
        cases.push(common::Case {
            title: format!("README snippet {index}"),
            source: after[..end].to_string(),
            expected: None,
        });

        rest = &after[end + 3..];
    }

    cases
}

#[test]
fn readme_snippets_run() {
    let cases = snippets();

    assert!(
        !cases.is_empty(),
        "README.md should show what the language looks like"
    );

    let mut failures = Vec::new();

    for case in &cases {
        if let Err(reason) = common::check(case) {
            failures.push((case.title.clone(), reason));
        }
    }

    println!(
        "\nreadme: {}/{} snippets run",
        cases.len() - failures.len(),
        cases.len()
    );

    for (title, reason) in &failures {
        println!("  FAIL {title}\n       {reason}");
    }

    assert!(
        failures.is_empty(),
        "{} README snippets fail",
        failures.len()
    );
}
