#![warn(
    clippy::pedantic,
    clippy::debug_assert_with_mut_call,
    clippy::exit,
    clippy::expect_used,
    clippy::format_push_string,
    clippy::indexing_slicing,
    clippy::infinite_loop,
    clippy::let_underscore_must_use,
    clippy::let_underscore_untyped,
    clippy::panic,
    clippy::panic_in_result_fn,
    clippy::shadow_unrelated,
    clippy::string_slice,
    clippy::todo,
    clippy::unimplemented,
    clippy::unreachable,
    clippy::unused_result_ok,
    clippy::unwrap_used,
    missing_debug_implementations,
    rust_2018_idioms,
    single_use_lifetimes,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unused_import_braces,
    unused_lifetimes,
    unused_qualifications
)]
#![cfg_attr(
    test,
    allow(
        clippy::expect_used,
        clippy::indexing_slicing,
        clippy::panic,
        clippy::panic_in_result_fn,
        clippy::shadow_unrelated,
        clippy::string_slice,
        clippy::too_many_lines,
        clippy::unreachable,
        clippy::unwrap_used
    )
)]

mod continuation;
mod interactive;
mod terminal;

use std::io::{self, IsTerminal};
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use nucleoid::graph::NodeKey;
use nucleoid::{Runtime, SyntaxOptions, Value};

const REPL_TITLE: &str = "🌿 Nucleoid - Logic Language for World Models";
const REPL_TAGLINE: &str = "🌎 Inspired by Nature";
const INDENT: &str = "    ";

#[derive(Parser)]
#[command(
    name = "nucleoid",
    version,
    about = "Nucleoid declarative logic runtime"
)]
#[expect(clippy::struct_excessive_bools)]
struct Cli {
    /// A source file to run. Without one, statements are read from the terminal.
    file: Option<PathBuf>,

    /// Print the logic graph after running.
    #[arg(long)]
    graph: bool,

    /// Serialize the program's result as JSON.
    #[arg(long)]
    json: bool,

    /// Use the line-oriented REPL instead of the full-screen terminal workspace.
    #[arg(long)]
    plain: bool,

    /// Enable // line comments and non-nesting /* ... */ block comments.
    #[arg(long)]
    c_style_comments: bool,

    #[arg(
        long,
        help = "Read a `target is value` statement as `target = value`; a bare word reads its name or else its own text"
    )]
    is_assignment: bool,

    #[arg(
        long,
        help = "With --is-assignment, a bare word branch of a conditional value also reads its name or else its own text"
    )]
    conditional_words: bool,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let mut runtime = Runtime::with_options(SyntaxOptions {
        c_style_comments: cli.c_style_comments,
        is_assignment: cli.is_assignment,
        conditional_words: cli.conditional_words,
    });

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
                    let rendered = match render(&runtime, &value, cli.json) {
                        Ok(rendered) => rendered,
                        Err(error) => {
                            eprintln!("{error}");
                            return ExitCode::FAILURE;
                        }
                    };

                    println!("{rendered}");

                    if cli.graph {
                        print_graph(&runtime);
                    }

                    report(&mut runtime)
                }
                Err(error) => {
                    eprintln!("{error}");
                    ExitCode::FAILURE
                }
            }
        }
        None if !cli.plain && io::stdin().is_terminal() && io::stdout().is_terminal() => {
            match terminal::run(&mut runtime, cli.json, cli.graph) {
                Ok(()) => ExitCode::SUCCESS,
                Err(error) => {
                    eprintln!("nucleoid: terminal editor failed: {error}");
                    ExitCode::FAILURE
                }
            }
        }
        None => interactive::repl(&mut runtime, cli.json, cli.graph),
    }
}

fn render(runtime: &Runtime, value: &Value, json: bool) -> nucleoid::Result<String> {
    if json {
        runtime.serialize_json(value)
    } else {
        Ok(value.to_string())
    }
}

/// Prints what the runtime is holding: every statement it tracks and what each
/// one waits on.
fn print_graph(runtime: &Runtime) {
    let graph = &runtime.graph;
    println!("\nlogic graph ({} nodes)", graph.len());

    for node in graph.nodes() {
        println!("  {} [{}]", node.key, node.kind);

        if !node.dependencies.is_empty() {
            let names: Vec<String> = node.dependencies.iter().map(NodeKey::to_string).collect();
            println!("    reads    {}", names.join(", "));
        }

        let observations: Vec<String> = graph
            .shape_dependencies(&node.key)
            .map(ToString::to_string)
            .collect();
        if !observations.is_empty() {
            println!("    observes {}", observations.join(", "));
        }

        if !node.dependents.is_empty() {
            let names: Vec<String> = node.dependents.iter().map(NodeKey::to_string).collect();
            println!("    updates  {}", names.join(", "));
        }
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
