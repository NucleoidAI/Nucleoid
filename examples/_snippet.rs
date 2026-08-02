use nucleoid::Runtime;

fn main() {
    let mut runtime = Runtime::new();

    let source = "a = 1\nb = a + 2\na = 3\nassert(b, 5)\n";

    runtime.run(source).unwrap();

    println!("b = {}", runtime.run("b").unwrap());
    println!("assertions run: {}", runtime.assertions_run());

    let failures = runtime.take_assertions();

    for failure in &failures {
        println!(
            "assertion failed: expected {}, got {}",
            failure.expected, failure.actual
        );
    }

    println!("{}", if failures.is_empty() { "ok" } else { "FAILED" });
}
