use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use nucleoid::graph::NodeKey;
use nucleoid::lang::ast::lexer::{Keyword, Spanned, Token, tokenize};
use nucleoid::{Runtime, Value};
use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;

#[derive(Parser)]
#[command(
    name = "nucleoid",
    version,
    about = "Nucleoid declarative logic runtime"
)]
struct Cli {
    /// A source file to run. Without one, statements are read from the terminal.
    file: Option<PathBuf>,

    /// Print the logic graph after running.
    #[arg(long)]
    graph: bool,

    /// Serialize the program's result as JSON.
    #[arg(long)]
    json: bool,
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
        None => repl(&mut runtime, cli.json, cli.graph),
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

fn repl(runtime: &mut Runtime, mut json: bool, mut graph: bool) -> ExitCode {
    let mut editor = match DefaultEditor::new() {
        Ok(editor) => editor,
        Err(error) => {
            eprintln!("nucleoid: cannot start interactive editor: {error}");
            return ExitCode::FAILURE;
        }
    };
    let mut source = String::new();
    let mut multiline = false;

    println!("🌿 Nucleoid - Logic Language for World Models");
    println!("🌎 Inspired by Nature");
    println!("Type .help for available commands.");

    loop {
        let prompt = if source.is_empty() { "> " } else { "... " };
        let line = match editor.readline(prompt) {
            Ok(line) => line,
            Err(ReadlineError::Interrupted) => {
                source.clear();
                multiline = false;
                println!("^C");
                continue;
            }
            Err(ReadlineError::Eof) => {
                if !source.trim().is_empty() {
                    run_interactive(runtime, &source, json, graph);
                }
                return ExitCode::SUCCESS;
            }
            Err(error) => {
                eprintln!("nucleoid: interactive input failed: {error}");
                return ExitCode::FAILURE;
            }
        };

        let trimmed = line.trim();

        if source.is_empty() && trimmed.starts_with('.') {
            if let Err(error) = editor.add_history_entry(trimmed) {
                eprintln!("nucleoid: cannot record history: {error}");
            }

            match trimmed {
                ".exit" | ".quit" => return ExitCode::SUCCESS,
                ".help" => print_repl_help(),
                ".clear" => {
                    runtime.clear();
                    println!("State cleared.");
                }
                command if command.starts_with(".json") => {
                    update_repl_setting(command, ".json", &mut json);
                }
                command if command.starts_with(".graph") => {
                    update_repl_setting(command, ".graph", &mut graph);
                }
                _ => eprintln!("Unknown command '{trimmed}'. Type .help for commands."),
            }
            continue;
        }

        if trimmed.is_empty() {
            if !source.is_empty() {
                if let Err(error) = editor.add_history_entry(source.trim_end()) {
                    eprintln!("nucleoid: cannot record history: {error}");
                }
                run_interactive(runtime, &source, json, graph);
                source.clear();
                multiline = false;
            }
            continue;
        }

        multiline |= line.trim_end().ends_with(':')
            || indented_named_arrow_header(&line)
            || indented_try_header(&line);
        source.push_str(&line);
        source.push('\n');

        if !multiline {
            if let Err(error) = editor.add_history_entry(source.trim_end()) {
                eprintln!("nucleoid: cannot record history: {error}");
            }
            run_interactive(runtime, &source, json, graph);
            source.clear();
        }
    }
}

/// Buffer a lexical named-arrow header, not anonymous arrows or arrow text in
/// literals/comments. The parser still owns parameter validation and the real
/// indentation requirement when the input is committed.
fn indented_named_arrow_header(line: &str) -> bool {
    let Ok(tokens) = tokenize(line) else {
        return false;
    };
    let mut tokens = header_tokens(&tokens);
    matches!(tokens.next(), Some(Token::Keyword(Keyword::Def)))
        && matches!(tokens.next(), Some(Token::Identifier(_)))
        && matches!(tokens.next_back(), Some(Token::Arrow))
        && !tokens.any(|token| matches!(token, Token::Arrow))
}

/// A bare try starts the existing buffer; catch stays in that same submission.
/// Lexing the whole line rejects malformed suffixes rather than hiding errors.
/// The parser still validates both suites when blank-line commit or EOF runs it.
fn indented_try_header(line: &str) -> bool {
    let Ok(tokens) = tokenize(line) else {
        return false;
    };
    let mut tokens = header_tokens(&tokens);
    matches!(tokens.next(), Some(Token::Keyword(Keyword::Try))) && tokens.next().is_none()
}

fn header_tokens(tokens: &[Spanned]) -> impl DoubleEndedIterator<Item = &Token> {
    tokens.iter().map(|spanned| &spanned.token).filter(|token| {
        !matches!(
            token,
            Token::Newline | Token::Indent | Token::Dedent | Token::Eof
        )
    })
}

fn run_interactive(runtime: &mut Runtime, source: &str, json: bool, graph: bool) {
    match runtime.run(source) {
        Ok(value) => match render(runtime, &value, json) {
            Ok(rendered) => println!("{rendered}"),
            Err(error) => eprintln!("{error}"),
        },
        Err(error) => eprintln!("{error}"),
    }

    if graph {
        print_graph(runtime);
    }

    for failure in runtime.take_assertions() {
        eprintln!(
            "assertion failed: expected {}, got {}",
            failure.expected, failure.actual
        );
    }
}

fn update_repl_setting(command: &str, name: &str, setting: &mut bool) {
    let argument = command.strip_prefix(name).unwrap_or_default().trim();

    match setting_value(argument, *setting) {
        Some(value) => {
            *setting = value;
            println!(
                "{} {}.",
                name.trim_start_matches('.'),
                if *setting { "on" } else { "off" }
            );
        }
        None => eprintln!("Usage: {name} [on|off]"),
    }
}

fn setting_value(argument: &str, current: bool) -> Option<bool> {
    match argument {
        "" => Some(!current),
        "on" => Some(true),
        "off" => Some(false),
        _ => None,
    }
}

fn print_repl_help() {
    println!(
        "\
.help            Show this help
.json [on|off]   Toggle JSON result output
.graph [on|off]  Toggle logic graph output
.clear           Clear runtime state and graph
.exit, .quit     Exit the interactive CLI

Use Up/Down for history and Ctrl-C to cancel the current input.
Enter a blank line to run a multiline indented block."
    );
}

#[cfg(test)]
mod tests {
    use super::{indented_named_arrow_header, indented_try_header, setting_value};

    #[test]
    fn only_a_complete_lexical_bare_try_header_starts_exception_continuation() {
        for line in [
            "try",
            "try ",
            "\ttry\t",
            "    try # catch: and => are only a comment",
            "try\t# try",
            "try \u{c}\r# accepted layout",
        ] {
            assert!(indented_try_header(line), "{line:?}");
        }
    }

    #[test]
    fn exception_continuation_rejects_text_prefixes_extra_tokens_and_lexical_errors() {
        for line in [
            "",
            " \t",
            "# try",
            "\"try\"",
            "'try'",
            "`try`",
            "/try/",
            "tryout",
            "try_",
            "trying",
            "value = \"try\"",
            "value = try",
            "try = 1",
            "try:",
            "try;",
            "try pass",
            "try {}",
            "try()",
            "try error",
            "try =>",
            "catch",
            "catch error",
            "catch(error)",
            "def f() =>",
            "try @",
            "try \"unterminated",
            "try\u{a0}",
            "try\u{2003}",
            "try\u{b}",
        ] {
            assert!(!indented_try_header(line), "{line:?}");
        }
    }

    #[test]
    fn only_named_headers_with_a_bodyless_arrow_start_continuation() {
        for line in [
            "def f() =>",
            "def f(value: int,) => # body",
            "    def f value =>",
            "\tdef f left: int, right, factor: number => # =>",
        ] {
            assert!(indented_named_arrow_header(line), "{line}");
        }
        for line in [
            "value =>",
            "(value: int) =>",
            "callback = value =>",
            "def f value => value",
            "def f value => next =>",
            "def f value => \"=>\"",
            "def f value => /=>/",
            "def f value => `=>`",
            "def f value => value # =>",
            "\"def f value =>\"",
            "# def f value =>",
            "1 # =>",
            "def f value => @",
        ] {
            assert!(!indented_named_arrow_header(line), "{line}");
        }
    }

    #[test]
    fn repl_settings_toggle_or_accept_explicit_values() {
        assert_eq!(setting_value("", false), Some(true));
        assert_eq!(setting_value("", true), Some(false));
        assert_eq!(setting_value("on", false), Some(true));
        assert_eq!(setting_value("off", true), Some(false));
        assert_eq!(setting_value("invalid", false), None);
    }
}
