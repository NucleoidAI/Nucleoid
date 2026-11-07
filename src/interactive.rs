use std::process::ExitCode;

use nucleoid::Runtime;
use rustyline::error::ReadlineError;
use rustyline::{Cmd, DefaultEditor, EventHandler, KeyCode, KeyEvent, Modifiers};

use super::continuation::Pending;
use super::{INDENT, REPL_TAGLINE, REPL_TITLE, print_graph, render};

pub(super) const CONTINUATION_PROMPT: &str = "..";

pub(super) fn repl_editor() -> Result<DefaultEditor, ReadlineError> {
    let mut editor = DefaultEditor::new()?;
    let _: Option<EventHandler> = editor.bind_sequence(
        KeyEvent(KeyCode::Tab, Modifiers::NONE),
        Cmd::Insert(1, INDENT.into()),
    );
    Ok(editor)
}

pub(super) fn repl(runtime: &mut Runtime, mut json: bool, mut graph: bool) -> ExitCode {
    let options = runtime.options();
    let mut editor = match repl_editor() {
        Ok(editor) => editor,
        Err(error) => {
            eprintln!("nucleoid: cannot start interactive editor: {error}");
            return ExitCode::FAILURE;
        }
    };
    let mut pending = Pending::default();

    println!("{REPL_TITLE}");
    println!("{REPL_TAGLINE}");
    println!("Type .help for available commands.");

    loop {
        let prompt = if pending.source.is_empty() {
            "> "
        } else {
            CONTINUATION_PROMPT
        };
        let line = match editor.readline_with_initial(prompt, (&pending.indentation, "")) {
            Ok(line) => line,
            Err(ReadlineError::Interrupted) => {
                pending = Pending::default();
                println!("^C");
                continue;
            }
            Err(ReadlineError::Eof) => {
                if !pending.source.trim().is_empty() {
                    run_interactive(runtime, &pending.source, json, graph);
                }
                return ExitCode::SUCCESS;
            }
            Err(error) => {
                eprintln!("nucleoid: interactive input failed: {error}");
                return ExitCode::FAILURE;
            }
        };

        let trimmed = line.trim();

        if pending.source.is_empty() && trimmed.starts_with('.') {
            record_history(&mut editor, trimmed);
            if repl_command(trimmed, runtime, &mut json, &mut graph) {
                return ExitCode::SUCCESS;
            }
            continue;
        }

        if let Some(source) = pending.accept(&line, options) {
            record_history(&mut editor, source.trim_end());
            run_interactive(runtime, &source, json, graph);
        }
    }
}

fn record_history(editor: &mut DefaultEditor, entry: &str) {
    if let Err(error) = editor.add_history_entry(entry) {
        eprintln!("nucleoid: cannot record history: {error}");
    }
}

fn repl_command(command: &str, runtime: &mut Runtime, json: &mut bool, graph: &mut bool) -> bool {
    match command {
        ".exit" | ".quit" => return true,
        ".help" => print_repl_help(),
        ".clear" => {
            runtime.clear();
            println!("State cleared.");
        }
        _ if command.starts_with(".json") => update_repl_setting(command, ".json", json),
        _ if command.starts_with(".graph") => update_repl_setting(command, ".graph", graph),
        _ => eprintln!("Unknown command '{command}'. Type .help for commands."),
    }
    false
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

pub(super) fn setting_value(argument: &str, current: bool) -> Option<bool> {
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
Body indentation is filled in automatically. Tab inserts four spaces.
Use Backspace to dedent.
Enter a blank line to run a multiline indented block."
    );
}
