//! Measures how the runtime scales, so the cost of the dependency graph is a
//! known quantity rather than an assumption.
//!
//! Run with `cargo run --release --example scaling`.
//!
//! Each workload is run at doubling sizes. The ratio column is the time against
//! the previous size: about 2 is linear, about 4 is quadratic.

use std::time::{Duration, Instant};

use nucleoid::Runtime;

fn main() {
    report("chain of dependent variables", &[100, 200, 400, 800], chain);
    report("fan-out from one variable", &[100, 200, 400, 800], fan_out);
    report(
        "instances of a plain type",
        &[100, 200, 400, 800],
        instances,
    );
    report("instances under a class rule", &[100, 200, 400, 800], ruled);
    report(
        "reassignment through a chain",
        &[50, 100, 200, 400],
        resettle,
    );
    report("try blocks", &[25, 50, 100, 200], guarded);
}

fn report(name: &str, sizes: &[usize], workload: fn(usize) -> Duration) {
    println!("\n{name}");
    println!("{:>8}  {:>12}  {:>8}", "n", "time", "ratio");

    let mut previous: Option<Duration> = None;

    for &size in sizes {
        let elapsed = workload(size);

        let ratio = match previous {
            Some(previous) if previous.as_secs_f64() > 0.0 => {
                format!("{:.1}x", elapsed.as_secs_f64() / previous.as_secs_f64())
            }
            _ => "-".to_string(),
        };

        println!("{size:>8}  {:>12}  {ratio:>8}", format!("{elapsed:.2?}"));
        previous = Some(elapsed);
    }
}

/// `v0 = 1`, then `v1 = v0 + 1`, `v2 = v1 + 1`, ... Each assignment adds one
/// link, and every change has to walk the whole chain.
fn chain(size: usize) -> Duration {
    let mut source = String::from("v0 = 1\n");

    for index in 1..size {
        source.push_str(&format!("v{index} = v{} + 1\n", index - 1));
    }

    time(&source)
}

/// One variable that many others read.
fn fan_out(size: usize) -> Duration {
    let mut source = String::from("base = 1\n");

    for index in 0..size {
        source.push_str(&format!("out{index} = base + {index}\n"));
    }

    time(&source)
}

fn instances(size: usize) -> Duration {
    let mut source = String::from("class Item:\n    pass\n\n");

    for index in 0..size {
        source.push_str(&format!("item{index} = Item()\n"));
    }

    time(&source)
}

/// The same, with a rule that every instance has to be given.
fn ruled(size: usize) -> Duration {
    let mut source = String::from("class Item(code: str):\n    this.code = code\n\n");
    source.push_str("$Item.label = \"item:\" + $Item.code\n\n");

    for index in 0..size {
        source.push_str(&format!("item{index} = Item(\"{index}\")\n"));
    }

    time(&source)
}

/// Builds a chain, then changes its head, which re-evaluates everything.
fn resettle(size: usize) -> Duration {
    let mut runtime = Runtime::new();
    let mut source = String::from("v0 = 1\n");

    for index in 1..size {
        source.push_str(&format!("v{index} = v{} + 1\n", index - 1));
    }

    runtime.run(&source).expect("chain should build");

    let start = Instant::now();
    runtime.run("v0 = 2").expect("head should reassign");
    start.elapsed()
}

/// Every `try` has to be able to undo whatever its body did.
fn guarded(size: usize) -> Duration {
    let mut source = String::from("guard = 0\n");

    for index in 0..size {
        source.push_str(&format!(
            "try:\n    step{index} = guard + {index}\ncatch error:\n    pass\n"
        ));
    }

    time(&source)
}

fn time(source: &str) -> Duration {
    let mut runtime = Runtime::new();
    let start = Instant::now();

    runtime
        .run(source)
        .unwrap_or_else(|error| panic!("workload failed: {error}"));

    start.elapsed()
}
