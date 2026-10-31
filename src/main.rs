mod terminal;

use std::io::{self, IsTerminal};
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use nucleoid::graph::NodeKey;
use nucleoid::lang::ast::Quantifier;
use nucleoid::lang::ast::lexer::{Keyword, Spanned, Token, tokenize};
use nucleoid::{Runtime, Value};
use rustyline::error::ReadlineError;
use rustyline::{Cmd, DefaultEditor, KeyCode, KeyEvent, Modifiers};

const REPL_TITLE: &str = "🌿 Nucleoid - Logic Language for World Models";
const REPL_TAGLINE: &str = "🌎 Inspired by Nature";
const INDENT: &str = "    ";
const CONTINUATION_PROMPT: &str = "..";

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

    /// Use the line-oriented REPL instead of the full-screen terminal workspace.
    #[arg(long)]
    plain: bool,
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
        None if !cli.plain && io::stdin().is_terminal() && io::stdout().is_terminal() => {
            match terminal::run(&mut runtime, cli.json, cli.graph) {
                Ok(()) => ExitCode::SUCCESS,
                Err(error) => {
                    eprintln!("nucleoid: terminal editor failed: {error}");
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

fn repl_editor() -> Result<DefaultEditor, ReadlineError> {
    let mut editor = DefaultEditor::new()?;
    let _ = editor.bind_sequence(
        KeyEvent(KeyCode::Tab, Modifiers::NONE),
        Cmd::Insert(1, INDENT.into()),
    );
    Ok(editor)
}

fn repl(runtime: &mut Runtime, mut json: bool, mut graph: bool) -> ExitCode {
    let mut editor = match repl_editor() {
        Ok(editor) => editor,
        Err(error) => {
            eprintln!("nucleoid: cannot start interactive editor: {error}");
            return ExitCode::FAILURE;
        }
    };
    let mut source = String::new();
    let mut multiline = false;
    let mut indentation = String::new();

    println!("{REPL_TITLE}");
    println!("{REPL_TAGLINE}");
    println!("Type .help for available commands.");

    loop {
        let prompt = if source.is_empty() {
            "> "
        } else {
            CONTINUATION_PROMPT
        };
        let line = match editor.readline_with_initial(prompt, (&indentation, "")) {
            Ok(line) => line,
            Err(ReadlineError::Interrupted) => {
                source.clear();
                multiline = false;
                indentation.clear();
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
                indentation.clear();
            }
            continue;
        }

        let opens_body = indented_body_header(&line);
        multiline |= line.trim_end().ends_with(':') || opens_body;
        source.push_str(&line);
        source.push('\n');

        if multiline {
            indentation = next_indentation(&line, opens_body);
        } else {
            if let Err(error) = editor.add_history_entry(source.trim_end()) {
                eprintln!("nucleoid: cannot record history: {error}");
            }
            run_interactive(runtime, &source, json, graph);
            source.clear();
            indentation.clear();
        }
    }
}

fn indented_body_header(line: &str) -> bool {
    if indented_named_arrow_header(line)
        || indented_try_header(line)
        || indented_condition_header(line, Keyword::If)
    {
        return true;
    }
    let Ok(tokens) = tokenize(line) else {
        return false;
    };
    let mut tokens = header_tokens(&tokens);
    let first = tokens.next();
    let last = tokens.next_back();
    matches!(last, Some(Token::Colon))
        || (matches!(first, Some(Token::Keyword(Keyword::Class)))
            && matches!(last, Some(Token::Identifier(_)))
            && matches!(tokens.next_back(), Some(Token::Colon)))
}

fn next_indentation(line: &str, opens_body: bool) -> String {
    // A pasted, balanced conditional header may span several physical lines.
    let opens_condition = indented_condition_header(line, Keyword::Else);
    let line = line.lines().next_back().unwrap_or_default();
    let mut indentation: String = line
        .chars()
        .take_while(|character| matches!(character, ' ' | '\t'))
        .collect();
    if opens_body || opens_condition || indented_alternative_header(line) {
        indentation.push_str(INDENT);
    }
    indentation
}

fn indented_alternative_header(line: &str) -> bool {
    if indented_condition_header(line, Keyword::Else) {
        return true;
    }
    let Ok(tokens) = tokenize(line) else {
        return false;
    };
    let mut tokens = header_tokens(&tokens);
    match tokens.next() {
        Some(Token::Keyword(Keyword::Else)) => tokens.next().is_none(),
        Some(Token::Keyword(Keyword::Catch)) => match tokens.next() {
            None => true,
            Some(Token::Identifier(_)) => tokens.next().is_none(),
            Some(Token::LeftParen) => {
                matches!(tokens.next(), Some(Token::Identifier(_)))
                    && matches!(tokens.next(), Some(Token::RightParen))
                    && tokens.next().is_none()
            }
            _ => false,
        },
        _ => false,
    }
}

/// Buffer only a lexical, nonempty balanced condition without a suite delimiter.
/// A top-level conditional expression's colon pairs with its `?`; it is not a
/// suite delimiter. Alternatives only adjust indentation inside an existing
/// submission; they do not start one. This is not expression validation:
/// commit/EOF still runs the unchanged real parser, including its body
/// requirements and resource bounds.
fn indented_condition_header(line: &str, keyword: Keyword) -> bool {
    let Ok(tokens) = tokenize(line) else {
        return false;
    };
    let mut tokens = tokens.iter().map(|spanned| &spanned.token).peekable();
    while matches!(tokens.peek(), Some(Token::Indent)) {
        tokens.next();
    }
    if tokens.next() != Some(&Token::Keyword(keyword)) {
        return false;
    }
    if keyword == Keyword::Else && tokens.peek() == Some(&&Token::Keyword(Keyword::If)) {
        tokens.next();
    }
    let mut nonempty = false;
    let mut delimiters = Vec::new();
    let mut conditionals = 0usize;
    while let Some(token) = tokens.next() {
        match token {
            Token::At => match tokens.next() {
                Some(Token::VersionIndex(_)) => {}
                Some(Token::Identifier(name))
                    if matches!(name.as_str(), "first" | "last" | "previous") => {}
                _ => return false,
            },
            Token::Eof => return nonempty && delimiters.is_empty() && conditionals == 0,
            Token::Newline | Token::Dedent if delimiters.is_empty() => {
                return nonempty
                    && conditionals == 0
                    && tokens
                        .all(|token| matches!(token, Token::Newline | Token::Dedent | Token::Eof));
            }
            Token::Question if delimiters.is_empty() => conditionals += 1,
            Token::Identifier(_)
                if delimiters.is_empty() && starts_quantifier(token, tokens.clone()) =>
            {
                conditionals += 1
            }
            Token::Colon if delimiters.is_empty() && conditionals > 0 => conditionals -= 1,
            Token::Colon | Token::Semicolon | Token::Indent if delimiters.is_empty() => {
                return false;
            }
            Token::LeftParen | Token::LeftBracket | Token::LeftBrace => {
                delimiters.push(token);
            }
            Token::RightParen | Token::RightBracket | Token::RightBrace
                if !matches!(
                    (delimiters.pop(), token),
                    (Some(Token::LeftParen), Token::RightParen)
                        | (Some(Token::LeftBracket), Token::RightBracket)
                        | (Some(Token::LeftBrace), Token::RightBrace)
                ) =>
            {
                return false;
            }
            _ => {}
        }
        nonempty = true;
    }
    false
}

fn starts_quantifier<'a>(word: &Token, mut rest: impl Iterator<Item = &'a Token>) -> bool {
    match (rest.next(), rest.next()) {
        (Some(binder), Some(keyword)) => Quantifier::starting(word, binder, keyword).is_some(),
        _ => false,
    }
}

/// Buffer a lexical named-arrow header, not anonymous arrows or arrow text in
/// literals/comments. Either spelling, `=>` or `->`, ends the header. The
/// parser still owns parameter validation and the real indentation
/// requirement when the input is committed.
fn indented_named_arrow_header(line: &str) -> bool {
    let Ok(tokens) = tokenize(line) else {
        return false;
    };
    let mut tokens = header_tokens(&tokens);
    matches!(tokens.next(), Some(Token::Keyword(Keyword::Def)))
        && matches!(tokens.next(), Some(Token::Identifier(_)))
        && tokens.next_back().is_some_and(Token::is_arrow)
        && !tokens.any(Token::is_arrow)
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
Body indentation is filled in automatically. Tab inserts four spaces.
Use Backspace to dedent.
Enter a blank line to run a multiline indented block."
    );
}

#[cfg(test)]
mod tests {
    use super::{
        CONTINUATION_PROMPT, INDENT, indented_alternative_header, indented_body_header,
        indented_condition_header, indented_named_arrow_header, indented_try_header,
        next_indentation, repl_editor, setting_value,
    };
    use nucleoid::lang::ast::lexer::Keyword;
    use nucleoid::{Runtime, Value};
    use rustyline::{Cmd, EventHandler, KeyCode, KeyEvent, Modifiers};

    #[test]
    fn continuation_prompt_has_no_extra_separator_before_body_indentation() {
        let indentation = next_indentation("class Student(name, age):", true);
        assert_eq!(format!("{CONTINUATION_PROMPT}{indentation}"), "..    ");
    }

    #[test]
    fn tab_inserts_the_same_four_literal_spaces_as_body_indentation() {
        let mut editor = repl_editor().unwrap();
        let Some(EventHandler::Simple(Cmd::Insert(1, spaces))) =
            editor.unbind_sequence(KeyEvent(KeyCode::Tab, Modifiers::NONE))
        else {
            panic!("Tab must insert literal spaces");
        };
        assert_eq!(spaces, INDENT);
        assert_eq!(spaces, "    ");
    }

    #[test]
    fn class_body_keystrokes_use_the_editors_visible_indentation() {
        let mut source = String::new();
        let mut indentation = String::new();
        for input in [
            "class Student(name, age):",
            "this.name = name",
            "this.age = age",
        ] {
            let line = format!("{indentation}{input}");
            source.push_str(&line);
            source.push('\n');
            indentation = next_indentation(&line, indented_body_header(&line));
        }
        assert_eq!(
            source,
            "class Student(name, age):\n    this.name = name\n    this.age = age\n"
        );
        assert_eq!(indentation, "    ");
        let mut runtime = Runtime::new();
        runtime.run(&source).unwrap();
        runtime
            .run("student = Student(age=20, name=\"Can\")")
            .unwrap();
        assert_eq!(
            runtime.run("student.name").unwrap(),
            Value::String("Can".into())
        );
        assert_eq!(runtime.run("student.age").unwrap(), Value::Number(20.0));
    }

    #[test]
    fn indentation_tracks_nested_bodies_tabs_and_explicit_dedents() {
        for (line, expected) in [
            ("class Student(name, age):", "    "),
            ("    this.name = name", "    "),
            ("    if age > 18:", "        "),
            ("    if age > 18 # colon: in comment", "        "),
            ("\telif if ready # comment", "\t    "),
            ("        this.adult = true", "        "),
            ("    this.age = age", "    "),
            ("\tif ready:", "\t    "),
            ("\t\tvalue = 1", "\t\t"),
            ("    def identity value => # body", "        "),
            ("    try # body", "        "),
            ("    # a comment ending in a colon:", "    "),
            ("value = \"class Student:\"", ""),
        ] {
            assert_eq!(
                next_indentation(line, indented_body_header(line)),
                expected,
                "{line:?}"
            );
        }
    }

    #[test]
    fn lexical_block_headers_include_comments_and_parent_classes() {
        for line in [
            "class Student(name, age): # properties",
            "class Student: Human",
            "class Student(name, age): Human # parent",
            "def compute(value): # body",
            "if ready: # body",
            "else: # body",
            "try: # body",
            "catch issue: # body",
        ] {
            assert!(indented_body_header(line), "{line:?}");
        }
        for line in [
            "# class Student:",
            "\"class Student:\"",
            "message = \"if ready:\"",
            "class Student(name, age)",
            "class Student\n    this.name = name\n    this.age = age",
            "catch",
            "else",
        ] {
            assert!(!indented_body_header(line), "{line:?}");
        }
    }

    #[test]
    fn bare_alternatives_indent_only_their_existing_continuation_body() {
        for line in [
            "else",
            "else # body",
            "else ready",
            "elif ready # body",
            "else if ready",
            "elif if ready",
            "catch",
            "catch issue",
            "catch(issue)",
            "catch (issue) # body",
        ] {
            assert!(indented_alternative_header(line), "{line:?}");
            assert_eq!(next_indentation(line, false), "    ", "{line:?}");
        }
        for line in [
            "# catch",
            "\"else\"",
            "else ready;",
            "catch issue extra",
            "catch()",
            "catch(issue, other)",
            "catch(issue) extra",
        ] {
            assert!(!indented_alternative_header(line), "{line:?}");
        }
    }

    #[test]
    fn colon_free_condition_continuation_uses_balanced_lexical_headers_not_suffixes() {
        for condition in [
            "ready",
            "ready@first",
            "items@0[1:3] == [2, 3]",
            "not ready and available",
            "items[1:3] == [2, 3]",
            "check({ready: items[:]}, (x: int) => x)",
            "(x: int) => { return x }",
            "({ready: true}).ready",
            "(# unmatched-looking :[}\n ready)",
            "\"else if :\" == \"else if :\"",
            "/[:{}]/.test(\":\")",
            "`else if ${value}:`",
            "ready # : [} => are only a comment",
        ] {
            for prefix in ["if", "else if", "elif if", "else", "elif"] {
                let line = format!("\t{prefix} {condition}");
                let keyword = if prefix == "if" {
                    Keyword::If
                } else {
                    Keyword::Else
                };
                assert!(indented_condition_header(&line, keyword), "{line:?}");
                let last_indent = line
                    .lines()
                    .next_back()
                    .unwrap()
                    .chars()
                    .take_while(|character| matches!(character, ' ' | '\t'))
                    .collect::<String>();
                assert_eq!(
                    next_indentation(&line, indented_body_header(&line)),
                    format!("{last_indent}{INDENT}")
                );
                assert_eq!(indented_body_header(&line), prefix == "if", "{line:?}");
            }
        }
        for line in [
            "",
            "# if ready",
            "\"if ready\"",
            "/if ready/",
            "`if ready`",
            "iffy = 1",
            "value = \"if ready\"",
            "if",
            "if # condition missing",
            "if ready:",
            "if ready: pass",
            "if ready;\n    pass",
            "if ready\n    pass",
            "if (ready",
            "if ready)",
            "if [ready)",
            "if {ready: true]",
            "if ready @",
            "if ready@unknown",
            "if ready@-1",
            "if \"unterminated",
            "if ready\u{a0}",
        ] {
            assert!(!indented_condition_header(line, Keyword::If), "{line:?}");
        }
        // Qualification never pretends that these are valid expressions.
        // Real parsing must still report them on blank-line commit or EOF.
        assert!(indented_condition_header("if ready +", Keyword::If));
        assert!(!indented_body_header("else ready"));
        assert!(!indented_body_header("elif if ready"));
    }

    #[test]
    fn conditional_expression_colons_pair_with_their_question_marks_in_headers() {
        for condition in [
            "ready ? on : off",
            "ready ? (x: int) => x : off",
            "a ? b : c ? d : e",
            "a ? b ? c : d : e",
            "(ready ? on : off) == on",
            "on if ready else off",
            "ready ? `${a ? b : c}` : off",
        ] {
            for prefix in ["if", "else if", "elif if", "else", "elif"] {
                let line = format!("{prefix} {condition}");
                let keyword = if prefix == "if" {
                    Keyword::If
                } else {
                    Keyword::Else
                };
                assert!(indented_condition_header(&line, keyword), "{line:?}");
                assert_eq!(indented_body_header(&line), prefix == "if", "{line:?}");
            }
        }
        for line in [
            "if ready ?",
            "if ready ? on",
            "if ready : off",
            "if ready ? on : off:",
            "if ready ? on : off: pass",
            "if ready ? on : off; other",
        ] {
            assert!(!indented_condition_header(line, Keyword::If), "{line:?}");
        }
        // A colon-bearing header still buffers through its final colon.
        assert!(indented_body_header("if ready ? on : off:"));
        assert!(!indented_body_header("label = ready ? on : off"));
        assert!(!indented_body_header("label = on if ready else off"));
    }

    #[test]
    fn quantifier_colons_pair_with_their_prefix_in_headers() {
        for condition in [
            "all v in xs: v > 0",
            "any v in xs: v ? on : off",
            "(all v in xs: v) and ready",
            "all t in Task: any w in Worker: w.skill == t.skill",
            "ready ? all v in xs: v : false",
            "all v in ready ? xs : ys: v",
            "all v in xs: [v][0:1]",
            "all v in xs: v ?? false",
        ] {
            for prefix in ["if", "else if", "elif if", "else", "elif"] {
                let line = format!("{prefix} {condition}");
                let keyword = if prefix == "if" {
                    Keyword::If
                } else {
                    Keyword::Else
                };
                assert!(indented_condition_header(&line, keyword), "{line:?}");
                assert_eq!(indented_body_header(&line), prefix == "if", "{line:?}");
            }
        }
        for line in [
            "if all v in xs",
            "if all v in xs: v > 0:",
            "if all v in xs: v > 0: pass",
            "if all v in xs: v; other",
            "if all v of xs: v",
            "if all: v",
            "if all in xs: v",
        ] {
            assert!(!indented_condition_header(line, Keyword::If), "{line:?}");
        }
        assert!(indented_body_header("if all v in xs: v > 0:"));
        assert!(!indented_body_header("ok = all v in xs: v > 0"));
        assert!(indented_alternative_header("else any v in xs: v > 1"));
        assert_eq!(next_indentation("elif all v in xs: v", false), INDENT);
    }

    #[test]
    fn nullish_coalescing_pairs_with_no_colon_in_headers() {
        for condition in [
            "ready ?? false",
            "a ?? b ?? c",
            "(ready ?? false) == true",
            "ready ? on ?? off : off",
            "ready ?? false ? on : off",
            "on if ready ?? false else off",
            "`${ready ?? \"off\"}` == \"on\"",
            "items[index ?? 0]",
        ] {
            for prefix in ["if", "else if", "elif if", "else", "elif"] {
                let line = format!("{prefix} {condition}");
                let keyword = if prefix == "if" {
                    Keyword::If
                } else {
                    Keyword::Else
                };
                assert!(indented_condition_header(&line, keyword), "{line:?}");
                assert_eq!(indented_body_header(&line), prefix == "if", "{line:?}");
            }
        }
        for line in [
            "if ready ? ? off",
            "if ready ?? on : off",
            "if ready ?? false:",
            "if ready ?? false: pass",
            "if ready ?? false; other",
        ] {
            assert!(!indented_condition_header(line, Keyword::If), "{line:?}");
        }
        assert!(indented_body_header("if ready ?? false:"));
        assert!(!indented_body_header("label = ready ?? \"off\""));
    }

    #[test]
    fn multiline_input_keeps_the_final_physical_lines_indentation() {
        let source = "class Student(name, age):\n    this.name = name\n    this.age = age\n";
        assert!(!indented_body_header(source));
        assert_eq!(next_indentation(source, false), "    ");
        assert_eq!(
            next_indentation("    if ready:\n        value = 1", false),
            "        "
        );
    }

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
    fn thin_named_arrow_headers_buffer_exactly_like_fat_ones() {
        for line in [
            "def f() ->",
            "def f(value: int,) -> # body",
            "    def f value ->",
            "\tdef f left: int, right, factor: number -> # ->",
            "def f left right ->",
        ] {
            assert!(indented_named_arrow_header(line), "{line}");
            assert!(indented_body_header(line), "{line}");
            assert!(
                indented_named_arrow_header(&line.replace("->", "=>")),
                "{line}"
            );
        }
        for line in [
            "value ->",
            "(value: int) ->",
            "callback = value ->",
            "def f value -> value",
            "def f value -> next ->",
            "def f value => next ->",
            "def f value -> next =>",
            "def f value -> \"->\"",
            "def f value -> /->/",
            "def f value -> `->`",
            "def f value -> value # ->",
            "\"def f value ->\"",
            "# def f value ->",
            "def f value - >",
            "def f value -> @",
            "f = lambda:",
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
