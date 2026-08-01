use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use nucleoid::Runtime;

#[derive(Parser)]
#[command(
    name = "nucleoid",
    version,
    about = "Nucleoid declarative logic runtime"
)]
struct Cli {
    /// A source file to run. Without one, statements are read from the terminal.
    file: Option<PathBuf>,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let mut runtime = Runtime::new();

    match cli.file {
        Some(path) => {
            let source = match std::fs::read_to_string(&path) {
                Ok(source) => source,
                Err(error) => {
                    eprintln!("nucleoid: cannot read {}: {error}", path.display());
                    return ExitCode::FAILURE;
                }
            };

            match runtime.run(&source) {
                Ok(value) => {
                    println!("{value}");
                    report(&mut runtime)
                }
                Err(error) => {
                    eprintln!("{error}");
                    ExitCode::FAILURE
                }
            }
        }
        None => repl(&mut runtime),
    }
}

fn report(runtime: &mut Runtime) -> ExitCode {
    let failures = runtime.take_assertions();

    if failures.is_empty() {
        return ExitCode::SUCCESS;
    }

    for failure in &failures {
        eprintln!(
            "assertion failed: expected {}, got {}",
            failure.expected, failure.actual
        );
    }

    ExitCode::FAILURE
}

fn repl(runtime: &mut Runtime) -> ExitCode {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        print!("> ");
        if stdout.flush().is_err() {
            return ExitCode::FAILURE;
        }

        let mut line = String::new();

        match stdin.lock().read_line(&mut line) {
            Ok(0) => return ExitCode::SUCCESS,
            Ok(_) => {}
            Err(error) => {
                eprintln!("nucleoid: {error}");
                return ExitCode::FAILURE;
            }
        }

        if line.trim().is_empty() {
            continue;
        }

        match runtime.run(&line) {
            Ok(value) => println!("{value}"),
            Err(error) => eprintln!("{error}"),
        }

        for failure in runtime.take_assertions() {
            eprintln!(
                "assertion failed: expected {}, got {}",
                failure.expected, failure.actual
            );
        }
    }
}
