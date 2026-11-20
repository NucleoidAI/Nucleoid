use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Rect;
use ratatui::{Terminal, backend::TestBackend};

use super::*;

fn key(app: &mut App, runtime: &mut Runtime, code: KeyCode, modifiers: KeyModifiers) -> bool {
    app.event(Event::Key(KeyEvent::new(code, modifiers)), runtime)
}

fn run(app: &mut App, runtime: &mut Runtime, source: &str) {
    app.editor = editor::new(source);
    assert!(!key(app, runtime, KeyCode::F(5), KeyModifiers::NONE));
}

#[test]
fn comments_reach_terminal_submission_indentation_and_diagnostics() {
    let mut app = App::new(false, false);
    let mut runtime = Runtime::new();
    app.editor = editor::new("def half value => // header:");
    key(
        &mut app,
        &mut runtime,
        KeyCode::Char('j'),
        KeyModifiers::CONTROL,
    );
    assert_eq!(app.editor.lines()[1], "    ");
    app.editor.insert_str("value /* units */ / 2");
    key(&mut app, &mut runtime, KeyCode::F(5), KeyModifiers::NONE);
    run(&mut app, &mut runtime, "half(10) // result");
    assert_eq!(app.entries.last().unwrap().output.text(View::Text), "5");

    app.editor = editor::new("if true:\n    /* doc\n                    */    if true:");
    key(
        &mut app,
        &mut runtime,
        KeyCode::Char('j'),
        KeyModifiers::CONTROL,
    );
    assert_eq!(app.editor.lines()[3], "        ");
    app.editor.insert_str("5");
    key(&mut app, &mut runtime, KeyCode::F(5), KeyModifiers::NONE);
    assert_eq!(app.entries.last().unwrap().output.text(View::Text), "5");

    app.editor = editor::new("/* open\nif true:");
    key(
        &mut app,
        &mut runtime,
        KeyCode::Char('j'),
        KeyModifiers::CONTROL,
    );
    assert_eq!(app.editor.lines()[2], "");
    key(&mut app, &mut runtime, KeyCode::F(5), KeyModifiers::NONE);
    assert_eq!(
        app.entries.last().unwrap().output.text(View::Text),
        "SyntaxError: Unterminated block comment (line 1, column 1)"
    );
    run(&mut app, &mut runtime, ".clear");
    run(&mut app, &mut runtime, "/* done */ 7");
    assert_eq!(app.entries.last().unwrap().output.text(View::Text), "7");
    assert_eq!(runtime.assertions_run(), 0);
}

#[test]
fn is_assignments_reach_terminal_submission_and_survive_clear() {
    let mut app = App::new(false, false);
    let mut runtime = Runtime::new();
    app.editor = editor::new("if true:");
    key(
        &mut app,
        &mut runtime,
        KeyCode::Char('j'),
        KeyModifiers::CONTROL,
    );
    assert_eq!(app.editor.lines()[1], "    ");
    app.editor.insert_str("status is OPEN");
    key(&mut app, &mut runtime, KeyCode::F(5), KeyModifiers::NONE);
    run(&mut app, &mut runtime, "label is status");
    assert_eq!(app.entries.last().unwrap().output.text(View::Text), "OPEN");
    run(&mut app, &mut runtime, "OPEN = \"open\"");
    run(&mut app, &mut runtime, "label");
    assert_eq!(app.entries.last().unwrap().output.text(View::Text), "open");
    run(&mut app, &mut runtime, ".clear");
    run(&mut app, &mut runtime, "other is 'x'");
    assert_eq!(app.entries.last().unwrap().output.text(View::Text), "x");
    assert_eq!(runtime.assertions_run(), 0);
}

#[test]
fn conditional_words_reach_terminal_submission_and_survive_clear() {
    let mut app = App::new(false, false);
    let mut runtime = Runtime::new();
    run(&mut app, &mut runtime, "ready = false");
    app.editor = editor::new("if true:");
    key(
        &mut app,
        &mut runtime,
        KeyCode::Char('j'),
        KeyModifiers::CONTROL,
    );
    assert_eq!(app.editor.lines()[1], "    ");
    app.editor
        .insert_str("status is OPEN if ready else WAITING");
    key(&mut app, &mut runtime, KeyCode::F(5), KeyModifiers::NONE);
    run(&mut app, &mut runtime, "status");
    assert_eq!(
        app.entries.last().unwrap().output.text(View::Text),
        "WAITING"
    );
    run(&mut app, &mut runtime, "ready = true");
    run(&mut app, &mut runtime, "OPEN = \"open\"");
    run(&mut app, &mut runtime, "status");
    assert_eq!(app.entries.last().unwrap().output.text(View::Text), "open");
    run(&mut app, &mut runtime, ".clear");
    run(&mut app, &mut runtime, "other is YES if true else NO");
    assert_eq!(app.entries.last().unwrap().output.text(View::Text), "YES");
    assert_eq!(runtime.assertions_run(), 0);
}

#[test]
fn item_words_reach_terminal_submission_and_survive_clear() {
    let mut app = App::new(false, false);
    let mut runtime = Runtime::new();
    app.editor = editor::new("if true:");
    key(
        &mut app,
        &mut runtime,
        KeyCode::Char('j'),
        KeyModifiers::CONTROL,
    );
    assert_eq!(app.editor.lines()[1], "    ");
    app.editor.insert_str("colors is [RED, GREEN]");
    key(&mut app, &mut runtime, KeyCode::F(5), KeyModifiers::NONE);
    run(&mut app, &mut runtime, "colors[0]");
    assert_eq!(app.entries.last().unwrap().output.text(View::Text), "RED");
    run(&mut app, &mut runtime, "RED = \"red\"");
    run(&mut app, &mut runtime, "colors[0]");
    assert_eq!(app.entries.last().unwrap().output.text(View::Text), "red");
    run(&mut app, &mut runtime, ".clear");
    run(&mut app, &mut runtime, "pair is {left: YES}");
    run(&mut app, &mut runtime, "pair.left");
    assert_eq!(app.entries.last().unwrap().output.text(View::Text), "YES");
    run(&mut app, &mut runtime, "late = [YES]");
    assert_eq!(
        app.entries.last().unwrap().output.text(View::Text),
        "ReferenceError: YES is not defined (line 1, column 1)"
    );
    assert_eq!(runtime.assertions_run(), 0);
}

#[test]
fn at_variables_reach_terminal_submission_and_indent_colon_free_headers() {
    let mut app = App::new(false, false);
    let mut runtime = Runtime::new();
    run(&mut app, &mut runtime, "x = 2");
    run(&mut app, &mut runtime, "C = 2");
    app.editor = editor::new("if x is @C");
    key(
        &mut app,
        &mut runtime,
        KeyCode::Char('j'),
        KeyModifiers::CONTROL,
    );
    assert_eq!(app.editor.lines()[1], "    ");
    app.editor.insert_str("y is @x + 1");
    key(&mut app, &mut runtime, KeyCode::F(5), KeyModifiers::NONE);
    run(&mut app, &mut runtime, "y");
    assert_eq!(app.entries.last().unwrap().output.text(View::Text), "3");
    run(&mut app, &mut runtime, ".clear");
    run(&mut app, &mut runtime, "b is @c + 3");
    assert_eq!(
        app.entries.last().unwrap().output.text(View::Text),
        "ReferenceError: c is not defined (line 1, column 1)"
    );
    assert_eq!(runtime.assertions_run(), 0);
}

#[test]
fn enter_preserves_pasted_indentation_and_never_executes_blank_lines() {
    let mut app = App::new(false, false);
    let mut runtime = Runtime::new();
    for line in [
        "class Person(name):",
        "    this.name = name",
        "    this.active = true",
        "",
    ] {
        app.editor.insert_str(line);
        key(&mut app, &mut runtime, KeyCode::Enter, KeyModifiers::NONE);
    }
    assert_eq!(
        app.editor.lines(),
        [
            "class Person(name):",
            "    this.name = name",
            "    this.active = true",
            "",
            ""
        ]
    );
    assert!(app.entries.is_empty());
    assert!(!runtime.state.has_class("Person"));
    key(&mut app, &mut runtime, KeyCode::F(5), KeyModifiers::NONE);
    assert!(runtime.state.has_class("Person"));
    run(
        &mut app,
        &mut runtime,
        "person = Person(\"Ada\")\nperson.name",
    );
    assert_eq!(app.entries.last().unwrap().output.text(View::Text), "Ada");
}

#[test]
fn indented_newlines_open_a_while_suite_and_submission_runs_the_loop_once() {
    let mut app = App::new(false, false);
    let mut runtime = Runtime::new();
    for line in ["count = 0", "while count < 3:"] {
        app.editor.insert_str(line);
        key(
            &mut app,
            &mut runtime,
            KeyCode::Char('j'),
            KeyModifiers::CONTROL,
        );
    }
    app.editor.insert_str("count = count + 1");
    assert_eq!(
        app.editor.lines(),
        ["count = 0", "while count < 3:", "    count = count + 1"]
    );
    assert!(app.entries.is_empty());
    key(&mut app, &mut runtime, KeyCode::F(5), KeyModifiers::NONE);
    assert_eq!(
        runtime.state.variable("count"),
        Some(&nucleoid::Value::Number(3.0))
    );
    run(&mut app, &mut runtime, "count = 0\ncount");
    assert_eq!(app.entries.last().unwrap().output.text(View::Text), "0");
}

#[test]
fn paste_waits_for_submission_and_history_restores_multiline_drafts() {
    let mut app = App::new(false, false);
    let mut runtime = Runtime::new();
    app.event(Event::Paste("a = 1\r\nb = a + 1".into()), &mut runtime);
    assert!(!runtime.state.has_variable("a"));
    key(&mut app, &mut runtime, KeyCode::F(5), KeyModifiers::NONE);
    assert!(runtime.state.has_variable("b"));
    app.editor = editor::new("unfinished\n    draft");
    key(
        &mut app,
        &mut runtime,
        KeyCode::Char('p'),
        KeyModifiers::CONTROL,
    );
    assert_eq!(app.editor.lines(), ["a = 1", "b = a + 1"]);
    key(
        &mut app,
        &mut runtime,
        KeyCode::Char('n'),
        KeyModifiers::CONTROL,
    );
    assert_eq!(app.editor.lines(), ["unfinished", "    draft"]);
}

#[test]
fn selector_programs_show_written_paths_in_results_and_graph_diagnostics() {
    let mut app = App::new(false, true);
    let mut runtime = Runtime::new();
    run(
        &mut app,
        &mut runtime,
        "class Bid(amount): this.amount = amount\nBid(5)\nif Bid.some(.amount):\n    open = true\ntop = Bid.max(.amount)\n(why top)[0].rule",
    );
    let entry = app.entries.last().unwrap();
    for view in [View::Text, View::Json, View::Table] {
        assert!(
            entry.output.text(view).contains("top = Bid.max(.amount)"),
            "{}",
            entry.output.text(view)
        );
    }
    assert!(
        entry.diagnostics.contains("if(Bid.some(.amount))"),
        "{}",
        entry.diagnostics
    );
    assert!(!entry.diagnostics.contains("function()"));
}

#[test]
fn settings_errors_and_clear_preserve_runtime_and_result_history() {
    let mut app = App::new(true, false);
    let mut runtime = Runtime::new();
    run(&mut app, &mut runtime, "value = 7");
    run(&mut app, &mut runtime, ".table on");
    assert!(app.view == View::Table);
    run(&mut app, &mut runtime, ".graph on");
    run(&mut app, &mut runtime, "value");
    assert!(
        app.entries
            .last()
            .unwrap()
            .diagnostics
            .contains("logic graph")
    );
    run(&mut app, &mut runtime, ".json invalid");
    assert!(app.entries.last().unwrap().output.is_error(app.view));
    run(&mut app, &mut runtime, "value = (");
    assert_eq!(
        runtime.state.variable("value"),
        Some(&nucleoid::Value::Number(7.0))
    );
    run(&mut app, &mut runtime, ".clear");
    assert!(!runtime.state.has_variable("value"));
    assert_eq!(app.entries[0].output.text(View::Text), "7");
    app.editor = editor::new(".exit");
    assert!(app.submit(&mut runtime));
}

#[test]
fn keyboard_completion_cancellation_and_release_events_do_not_submit() {
    let mut app = App::new(false, false);
    let mut runtime = Runtime::new();
    app.editor = editor::new("Math.sq");
    app.edited(&runtime);
    key(&mut app, &mut runtime, KeyCode::Tab, KeyModifiers::NONE);
    assert_eq!(app.editor.lines(), ["Math.sqrt"]);
    key(
        &mut app,
        &mut runtime,
        KeyCode::Char('c'),
        KeyModifiers::CONTROL,
    );
    assert!(app.editor.is_empty());
    let release =
        KeyEvent::new_with_kind(KeyCode::Enter, KeyModifiers::NONE, KeyEventKind::Release);
    assert!(!app.event(Event::Key(release), &mut runtime));
    assert!(app.entries.is_empty());
    assert!(key(
        &mut app,
        &mut runtime,
        KeyCode::Char('d'),
        KeyModifiers::CONTROL
    ));
}

#[test]
fn familiar_undo_redo_and_cut_paste_shortcuts_work_in_the_editor() {
    let mut app = App::new(false, false);
    let mut runtime = Runtime::new();
    key(
        &mut app,
        &mut runtime,
        KeyCode::Char('x'),
        KeyModifiers::NONE,
    );
    key(
        &mut app,
        &mut runtime,
        KeyCode::Char('z'),
        KeyModifiers::CONTROL,
    );
    assert!(app.editor.is_empty());
    key(
        &mut app,
        &mut runtime,
        KeyCode::Char('y'),
        KeyModifiers::CONTROL,
    );
    assert_eq!(app.editor.lines(), ["x"]);
    app.editor.select_all();
    key(
        &mut app,
        &mut runtime,
        KeyCode::Char('x'),
        KeyModifiers::CONTROL,
    );
    assert!(app.editor.is_empty());
    key(
        &mut app,
        &mut runtime,
        KeyCode::Char('v'),
        KeyModifiers::CONTROL,
    );
    assert_eq!(app.editor.lines(), ["x"]);
}

#[test]
fn windows_control_enter_is_a_pasted_lf_not_an_execution_shortcut() {
    let mut app = App::new(false, false);
    let mut runtime = Runtime::new();
    app.editor = editor::new("value = 7");
    key(
        &mut app,
        &mut runtime,
        KeyCode::Enter,
        KeyModifiers::CONTROL,
    );
    if cfg!(windows) {
        assert_eq!(app.editor.lines(), ["value = 7", ""]);
        assert!(app.entries.is_empty());
        assert!(!runtime.state.has_variable("value"));
    } else {
        assert_eq!(app.entries.len(), 1);
        assert!(runtime.state.has_variable("value"));
    }
}

#[test]
fn result_focus_scrolls_output_without_editing_or_recalling() {
    let mut app = App::new(false, false);
    let mut runtime = Runtime::new();
    run(&mut app, &mut runtime, "value = 7");
    app.editor = editor::new("    ");
    key(&mut app, &mut runtime, KeyCode::F(4), KeyModifiers::NONE);
    assert!(app.focus == Focus::Result);
    for code in [
        KeyCode::Backspace,
        KeyCode::Char('x'),
        KeyCode::Enter,
        KeyCode::Tab,
        KeyCode::Down,
        KeyCode::Down,
        KeyCode::Up,
        KeyCode::Right,
    ] {
        assert!(!key(&mut app, &mut runtime, code, KeyModifiers::NONE));
    }
    assert_eq!(app.editor.lines(), ["    "]);
    assert!(app.history_index.is_none());
    assert_eq!((app.scroll, app.horizontal), (1, 1));
    key(&mut app, &mut runtime, KeyCode::End, KeyModifiers::NONE);
    assert_eq!(app.scroll, u16::MAX);
    key(&mut app, &mut runtime, KeyCode::Home, KeyModifiers::NONE);
    assert_eq!((app.scroll, app.horizontal), (0, 0));

    key(&mut app, &mut runtime, KeyCode::F(4), KeyModifiers::NONE);
    assert!(app.focus == Focus::Editor);
    key(
        &mut app,
        &mut runtime,
        KeyCode::Backspace,
        KeyModifiers::NONE,
    );
    assert_eq!(app.editor.lines(), [""]);
    app.editor = editor::new("x");
    assert!(!key(
        &mut app,
        &mut runtime,
        KeyCode::Char('d'),
        KeyModifiers::CONTROL
    ));
    assert_eq!(app.editor.lines(), ["x"]);

    key(&mut app, &mut runtime, KeyCode::F(2), KeyModifiers::NONE);
    key(&mut app, &mut runtime, KeyCode::F(4), KeyModifiers::NONE);
    assert!(app.focus == Focus::Result);
    key(&mut app, &mut runtime, KeyCode::Esc, KeyModifiers::NONE);
    assert!(app.focus == Focus::Editor);
    assert_eq!(app.tab, 0);

    app.editor = editor::new(".exit now");
    assert!(!app.submit(&mut runtime));
    assert_eq!(
        app.entries.last().unwrap().output.text(View::Text),
        "Unknown command '.exit now'. Type .help for commands."
    );
}

#[test]
fn diagnostics_focus_needs_diagnostics_and_scrolls_its_own_offsets() {
    let mut app = App::new(false, true);
    let mut runtime = Runtime::new();
    run(&mut app, &mut runtime, "first = 1");
    run(&mut app, &mut runtime, "second = 2");
    assert_eq!(app.selected, 1);
    for expected in [
        Focus::Result,
        Focus::Diagnostics,
        Focus::Editor,
        Focus::Result,
        Focus::Diagnostics,
    ] {
        key(&mut app, &mut runtime, KeyCode::F(4), KeyModifiers::NONE);
        assert!(app.focus == expected);
    }
    key(&mut app, &mut runtime, KeyCode::Down, KeyModifiers::NONE);
    key(&mut app, &mut runtime, KeyCode::Right, KeyModifiers::NONE);
    assert_eq!(
        (
            app.scroll,
            app.horizontal,
            app.diagnostic_scroll,
            app.diagnostic_horizontal
        ),
        (0, 0, 1, 1)
    );
    key(&mut app, &mut runtime, KeyCode::Up, KeyModifiers::CONTROL);
    assert_eq!(app.selected, 0);
    assert!(app.focus == Focus::Diagnostics);
    assert_eq!((app.diagnostic_scroll, app.diagnostic_horizontal), (0, 0));

    app.graph = false;
    run(&mut app, &mut runtime, "third = 3");
    key(&mut app, &mut runtime, KeyCode::F(4), KeyModifiers::NONE);
    key(&mut app, &mut runtime, KeyCode::F(4), KeyModifiers::NONE);
    assert!(app.focus == Focus::Editor);
}

#[test]
fn mouse_hits_real_tab_view_history_and_editor_regions() {
    let mut app = App::new(false, false);
    let mut runtime = Runtime::new();
    run(&mut app, &mut runtime, "[{name: \"Ada\"}, {name: \"Lin\"}]");
    let mut terminal = Terminal::new(TestBackend::new(100, 30)).unwrap();
    terminal.draw(|frame| app.draw(frame)).unwrap();
    let click = |area: ratatui::layout::Rect| MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: area.x,
        row: area.y,
        modifiers: KeyModifiers::NONE,
    };
    app.mouse(click(app.regions.views[2]));
    assert!(app.view == View::Table);
    app.mouse(click(app.regions.tabs[1]));
    assert_eq!(app.tab, 1);
    terminal.draw(|frame| app.draw(frame)).unwrap();
    assert_eq!(app.regions.output.width, 0);
    app.mouse(click(app.regions.tabs[0]));
    app.editor = editor::new("value = 1");
    terminal.draw(|frame| app.draw(frame)).unwrap();
    let mut position = click(app.regions.editor);
    position.column += 2;
    app.mouse(position);
    assert_eq!(app.editor.cursor(), (0, 2));
    position.kind = MouseEventKind::Drag(MouseButton::Left);
    position.column += 3;
    app.mouse(position);
    assert_eq!(app.editor.selection_range(), Some(((0, 2), (0, 5))));
}

#[test]
fn original_branding_renders_top_left_above_the_tabs() {
    let mut app = App::new(false, false);
    for (width, height) in [(80, 12), (80, 24), (100, 30)] {
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        terminal.draw(|frame| app.draw(frame)).unwrap();
        let screen = terminal.backend().buffer();
        let line = |row| {
            (0..width)
                .map(|column| screen[(column, row)].symbol())
                .collect::<String>()
        };
        assert_eq!(line(0).trim(), "");
        assert_eq!(screen[(15, 1)].symbol(), "🌿");
        assert!(line(1).contains("Nucleoid - Logic Language for World Models"));
        assert_eq!(screen[(15, 2)].symbol(), "🌎");
        assert!(line(2).contains("Inspired by Nature"));
        for label in ["F1 REPL", "F2 Tab 2", "F3 Tab 3"] {
            assert!(line(3).contains(label));
        }
        assert!(
            app.regions
                .tabs
                .iter()
                .all(|tab| tab.y == 3 && tab.height == 1)
        );
    }
}

#[test]
fn original_wordmark_stays_top_left_after_results_and_tab_changes() {
    for (width, height) in [(40, 18), (80, 24), (100, 30)] {
        let mut app = App::new(false, false);
        let mut runtime = Runtime::new();
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        let assert_wordmark = |screen: &ratatui::buffer::Buffer| {
            for (row, logo) in [(1, "⣿⠋⢻⡄⣿ ⢸⡇⢰⡟⠙⠷"), (2, "⣿ ⢸⡇⢻⣄⣼⡇⠸⣧⣠⡶")]
            {
                let line: String = (0..width)
                    .map(|column| screen[(column, row)].symbol())
                    .collect();
                assert!(line.starts_with(&format!(" {logo}")), "{line}");
                for (column, _) in (1_u16..).zip(logo.chars()) {
                    assert_eq!(
                        screen[(column, row)].fg,
                        ratatui::style::Color::LightGreen,
                        "logo cell ({column}, {row})"
                    );
                }
            }
            let text: String = screen
                .content
                .iter()
                .map(ratatui::buffer::Cell::symbol)
                .collect();
            assert_eq!(text.matches("⣿⠋⢻⡄").count(), 1, "{text}");
        };
        terminal.draw(|frame| app.draw(frame)).unwrap();
        assert_wordmark(terminal.backend().buffer());
        let editor_area = app.regions.editor;

        run(&mut app, &mut runtime, "\"first result\"");
        terminal.draw(|frame| app.draw(frame)).unwrap();
        assert_wordmark(terminal.backend().buffer());
        let text: String = terminal
            .backend()
            .buffer()
            .content
            .iter()
            .map(ratatui::buffer::Cell::symbol)
            .collect();
        assert!(text.contains("first result"), "{text}");
        assert_eq!(app.regions.editor, editor_area);

        key(&mut app, &mut runtime, KeyCode::F(2), KeyModifiers::NONE);
        terminal.draw(|frame| app.draw(frame)).unwrap();
        assert_wordmark(terminal.backend().buffer());
        assert_eq!(app.regions.editor, editor_area);
    }
}

#[test]
fn idle_logo_sparkles_briefly_without_changing_text_or_layout() {
    let pause_bounds = Duration::from_secs(4)..=Duration::from_secs(12);
    for (width, height) in [(40, 12), (80, 24), (100, 30)] {
        let mut app = App::new(false, false);
        let now = Instant::now();
        app.sparkle = LogoSparkle::new(now);
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        terminal.draw(|frame| app.draw(frame)).unwrap();
        let original = terminal.backend().buffer().clone();
        let editor_area = app.regions.editor;
        let start = app.sparkle.next;
        assert!(pause_bounds.contains(&start.duration_since(now)));

        app.sparkle
            .advance(start.checked_sub(Duration::from_millis(1)).unwrap());
        assert!(app.sparkle.cell.is_none());

        app.sparkle.advance(start);
        let active_cell = app.sparkle.cell.unwrap();
        let row = u16::try_from(active_cell.0).unwrap() + 1;
        let column = u16::try_from(active_cell.1).unwrap();
        let end = app.sparkle.next;
        assert_eq!(end, start + Duration::from_millis(220));
        terminal.draw(|frame| app.draw(frame)).unwrap();
        let mut flashed = terminal.backend().buffer().clone();
        assert_eq!(
            flashed[(column, row)].symbol(),
            original[(column, row)].symbol()
        );
        assert_ne!(flashed[(column, row)].symbol().trim(), "");
        assert_eq!(flashed[(column, row)].fg, ratatui::style::Color::White);
        assert!(
            flashed[(column, row)]
                .modifier
                .contains(ratatui::style::Modifier::BOLD)
        );
        flashed[(column, row)] = original[(column, row)].clone();
        assert_eq!(flashed, original);
        assert_eq!(app.regions.editor, editor_area);

        app.sparkle
            .advance(end.checked_sub(Duration::from_millis(1)).unwrap());
        assert_eq!(app.sparkle.cell, Some(active_cell));

        app.sparkle.advance(end);
        assert!(app.sparkle.cell.is_none());
        assert!(app.sparkle.next > end);
        assert!(pause_bounds.contains(&app.sparkle.next.duration_since(end)));
        terminal.draw(|frame| app.draw(frame)).unwrap();
        assert_eq!(terminal.backend().buffer(), &original);
        assert_eq!(app.regions.editor, editor_area);
    }
}

#[test]
fn rendering_handles_small_resized_and_scrolled_table_workspaces() {
    let mut app = App::new(false, false);
    let mut runtime = Runtime::new();
    run(
        &mut app,
        &mut runtime,
        "[{name: \"Ada\", score: 9}, {name: \"Lin\", score: 8}]",
    );
    app.set_view(View::Table);
    for (width, height) in [(100, 30), (40, 12), (1, 1), (80, 24)] {
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        app.scroll = u16::MAX;
        app.horizontal = u16::MAX;
        terminal.draw(|frame| app.draw(frame)).unwrap();
        if width >= 40 && height >= 12 {
            let screen = terminal.backend().buffer();
            let text = screen
                .content
                .iter()
                .map(ratatui::buffer::Cell::symbol)
                .collect::<String>();
            assert!(text.contains("Nucleoid"));
            assert!(text.contains("Editor"));
        }
    }
}

#[test]
fn compact_workspaces_keep_results_and_editor_visible_with_optional_diagnostics() {
    for (width, height) in [(40, 12), (60, 14), (80, 18)] {
        let mut app = App::new(false, true);
        let mut runtime = Runtime::new();
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        terminal.draw(|frame| app.draw(frame)).unwrap();
        assert!(app.regions.output.height > 0);
        assert!(screen_line(&terminal, app.regions.output.y).contains("F5 runs it."));
        assert!(app.regions.editor.height > 0);

        app.mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: app.regions.output.x,
            row: app.regions.output.y,
            modifiers: KeyModifiers::NONE,
        });
        assert!(app.focus == Focus::Result);

        run(&mut app, &mut runtime, "assert(7, 7)\n7");
        assert_eq!(runtime.assertions_run(), 1);
        terminal.draw(|frame| app.draw(frame)).unwrap();
        let result = app.regions.output;
        assert!(result.height > 0);
        assert_eq!(
            terminal.backend().buffer()[(result.x, result.y)].symbol(),
            "7"
        );
        if app.regions.diagnostics.height == 0 {
            assert!(screen_line(&terminal, result.y - 1).contains("F4 diagnostics"));
        }

        key(&mut app, &mut runtime, KeyCode::F(4), KeyModifiers::NONE);
        key(&mut app, &mut runtime, KeyCode::F(4), KeyModifiers::NONE);
        terminal.draw(|frame| app.draw(frame)).unwrap();
        assert!(app.focus == Focus::Diagnostics);
        assert!(app.regions.diagnostics.height > 0);
        assert!(screen_line(&terminal, app.regions.diagnostics.y).contains("logic graph"));
        assert!(app.regions.editor.height > 0);

        key(&mut app, &mut runtime, KeyCode::F(4), KeyModifiers::NONE);
        terminal.draw(|frame| app.draw(frame)).unwrap();
        assert!(app.regions.output.height > 0);
        assert_eq!(runtime.assertions_run(), 1);
        assert_eq!(app.entries.len(), 1);
    }
}

#[test]
fn status_bar_tracks_focus_and_completion_without_hiding_run_or_quit() {
    for width in [40, 80, 100] {
        let mut app = App::new(false, true);
        let mut runtime = Runtime::new();
        run(&mut app, &mut runtime, "1");
        let mut terminal = Terminal::new(TestBackend::new(width, 24)).unwrap();
        for label in [
            "EDITOR",
            "RESULT",
            if width >= 80 { "DIAGNOSTICS" } else { "DIAG" },
        ] {
            terminal.draw(|frame| app.draw(frame)).unwrap();
            let status = screen_line(&terminal, 23);
            for hint in [label, "F5 run", "F4 focus", "Ctrl-Q quit"] {
                assert!(status.contains(hint), "{status:?} missing {hint:?}");
            }
            if label == "RESULT" && width >= 80 {
                assert!(status.contains("PgUp/Dn scroll"), "{status:?}");
            }
            key(&mut app, &mut runtime, KeyCode::F(4), KeyModifiers::NONE);
        }
        app.editor = editor::new("Mat");
        app.edited(&runtime);
        assert!(app.completion.is_some());
        terminal.draw(|frame| app.draw(frame)).unwrap();
        if width >= 80 {
            let status = screen_line(&terminal, 23);
            assert!(status.contains("Tab accept"), "{status:?}");
            assert!(status.contains("Esc dismiss"), "{status:?}");
        }
    }
}

#[test]
fn black_surfaces_preserve_readable_selection_and_explicit_error_markers() {
    use ratatui::style::Color;

    let mut app = App::new(false, false);
    let mut runtime = Runtime::new();
    run(&mut app, &mut runtime, "\n\nvalue = 7");
    run(&mut app, &mut runtime, "unknown_name");
    let mut terminal = Terminal::new(TestBackend::new(100, 30)).unwrap();
    terminal.draw(|frame| app.draw(frame)).unwrap();
    let screen = terminal.backend().buffer();
    assert_eq!(screen[(0, 0)].bg, Color::Black);
    let history = app.regions.history;
    let result = app.regions.output;
    let result_cell = &screen[(result.x, result.y)];
    let selected_cell = &screen[(history.x, history.y + 1)];
    let unselected_cell = &screen[(history.x, history.y)];
    for background in [result_cell.bg, selected_cell.bg, unselected_cell.bg] {
        assert!(
            matches!(background, Color::Rgb(red, green, blue)
                if red == green && green == blue && red <= 34),
            "{background:?}"
        );
    }
    assert_ne!(selected_cell.bg, unselected_cell.bg);
    assert_ne!(selected_cell.fg, selected_cell.bg);
    assert_ne!(result_cell.fg, unselected_cell.fg);
    assert_eq!(screen[(history.x + 4, history.y + 1)].symbol(), "!");
    assert!(screen_line(&terminal, history.y).contains("value = 7"));
    assert!(screen_line(&terminal, result.y - 1).contains("Error"));
    assert!(screen_line(&terminal, result.y).contains("ReferenceError"));
}

#[test]
fn paging_uses_visible_diagnostic_and_table_rows() {
    for (width, height) in [(100, 30), (40, 14)] {
        let mut app = App::new(false, false);
        let mut runtime = Runtime::new();
        app.entries.push(multiline_entry());
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        key(&mut app, &mut runtime, KeyCode::F(4), KeyModifiers::NONE);
        terminal.draw(|frame| app.draw(frame)).unwrap();
        let result_page = app.regions.output.height;
        key(
            &mut app,
            &mut runtime,
            KeyCode::PageDown,
            KeyModifiers::NONE,
        );
        assert_eq!(app.scroll, result_page);
        terminal.draw(|frame| app.draw(frame)).unwrap();
        assert!(
            screen_line(&terminal, app.regions.output.y)
                .contains(&format!("line {result_page:02}"))
        );
        key(&mut app, &mut runtime, KeyCode::PageUp, KeyModifiers::NONE);
        assert_eq!(app.scroll, 0);

        key(&mut app, &mut runtime, KeyCode::F(4), KeyModifiers::NONE);
        terminal.draw(|frame| app.draw(frame)).unwrap();
        let diagnostic_page = app.regions.diagnostics.height;
        key(
            &mut app,
            &mut runtime,
            KeyCode::PageDown,
            KeyModifiers::NONE,
        );
        assert_eq!(app.diagnostic_scroll, diagnostic_page);
        assert_eq!(app.scroll, 0);
        terminal.draw(|frame| app.draw(frame)).unwrap();
        assert!(
            screen_line(&terminal, app.regions.diagnostics.y)
                .contains(&format!("line {diagnostic_page:02}"))
        );
        key(&mut app, &mut runtime, KeyCode::PageUp, KeyModifiers::NONE);
        assert_eq!(app.diagnostic_scroll, 0);

        run(&mut app, &mut runtime, &array_source());
        app.set_view(View::Table);
        key(&mut app, &mut runtime, KeyCode::F(4), KeyModifiers::NONE);
        terminal.draw(|frame| app.draw(frame)).unwrap();
        let table_page = app.regions.output.height - 1;
        assert!(table_page > 0);
        key(
            &mut app,
            &mut runtime,
            KeyCode::PageDown,
            KeyModifiers::NONE,
        );
        assert_eq!(app.scroll, table_page);
        terminal.draw(|frame| app.draw(frame)).unwrap();
        assert!(screen_line(&terminal, app.regions.output.y).contains("Index"));
        key(&mut app, &mut runtime, KeyCode::PageUp, KeyModifiers::NONE);
        assert_eq!(app.scroll, 0);
    }
}

#[test]
fn history_without_diagnostics_returns_focus_to_the_result() {
    for mouse in [false, true] {
        let mut app = App::new(false, false);
        let mut runtime = Runtime::new();
        app.entries.push(multiline_entry());
        app.entries.push(Entry {
            source: ".clear".into(),
            output: output::Output::Message("State cleared.".into()),
            diagnostics: String::new(),
        });
        app.editor = editor::new("unfinished draft");
        let mut terminal = Terminal::new(TestBackend::new(100, 30)).unwrap();
        key(&mut app, &mut runtime, KeyCode::F(4), KeyModifiers::NONE);
        key(&mut app, &mut runtime, KeyCode::F(4), KeyModifiers::NONE);
        terminal.draw(|frame| app.draw(frame)).unwrap();
        assert!(app.focus == Focus::Diagnostics);
        if mouse {
            app.mouse(MouseEvent {
                kind: MouseEventKind::Down(MouseButton::Left),
                column: app.regions.history.x,
                row: app.regions.history.y + 1,
                modifiers: KeyModifiers::NONE,
            });
        } else {
            key(&mut app, &mut runtime, KeyCode::Down, KeyModifiers::CONTROL);
        }
        assert_eq!(app.selected, 1);
        assert!(app.focus == Focus::Result);
        terminal.draw(|frame| app.draw(frame)).unwrap();
        assert!(screen_line(&terminal, 29).contains("RESULT"));
        key(&mut app, &mut runtime, KeyCode::Down, KeyModifiers::NONE);
        assert_eq!((app.scroll, app.diagnostic_scroll), (1, 0));
        assert_eq!(app.editor.lines(), ["unfinished draft"]);
        assert_eq!(app.entries.len(), 2);
    }
}

#[test]
fn completion_popup_captures_wheels_including_its_border() {
    let mut app = App::new(false, false);
    let mut runtime = Runtime::new();
    app.entries.push(multiline_entry());
    app.entries.push(multiline_entry());
    app.editor = editor::new("Math.");
    app.edited(&runtime);
    let mut terminal = Terminal::new(TestBackend::new(100, 30)).unwrap();
    terminal.draw(|frame| app.draw(frame)).unwrap();
    let inner = app.regions.completions;
    let popup = app.regions.completion_popup;
    for (kind, column, row, expected) in [
        (MouseEventKind::ScrollDown, inner.x, inner.y, 1),
        (MouseEventKind::ScrollUp, popup.x + 2, popup.y, 0),
        (MouseEventKind::ScrollRight, inner.x, inner.y, 0),
        (MouseEventKind::ScrollLeft, popup.x + 2, popup.y, 0),
        (MouseEventKind::ScrollDown, inner.x, inner.y, 1),
    ] {
        app.mouse(MouseEvent {
            kind,
            column,
            row,
            modifiers: KeyModifiers::NONE,
        });
        assert_eq!(app.completion.as_ref().unwrap().selected, expected);
        assert_eq!((app.selected, app.scroll, app.horizontal), (0, 0, 0));
        assert_eq!((app.diagnostic_scroll, app.diagnostic_horizontal), (0, 0));
        assert_eq!(app.editor.lines(), ["Math."]);
        terminal.draw(|frame| app.draw(frame)).unwrap();
    }
    let selected = app.completion.as_ref().unwrap().candidates[1].clone();
    key(&mut app, &mut runtime, KeyCode::Tab, KeyModifiers::NONE);
    assert_eq!(app.editor.lines(), [selected]);
    assert!(app.completion.is_none());
    assert_eq!(app.entries.len(), 2);
}

#[test]
fn gutter_scrollbars_follow_overflow_and_disappear_when_content_fits() {
    let mut app = App::new(false, false);
    let mut runtime = Runtime::new();
    app.entries.push(multiline_entry());
    let mut terminal = Terminal::new(TestBackend::new(100, 30)).unwrap();
    key(&mut app, &mut runtime, KeyCode::F(4), KeyModifiers::NONE);
    terminal.draw(|frame| app.draw(frame)).unwrap();
    let top = gutter_symbols(&terminal, app.regions.output);
    assert!(!top.trim().is_empty());
    key(
        &mut app,
        &mut runtime,
        KeyCode::PageDown,
        KeyModifiers::NONE,
    );
    terminal.draw(|frame| app.draw(frame)).unwrap();
    assert_ne!(gutter_symbols(&terminal, app.regions.output), top);

    key(&mut app, &mut runtime, KeyCode::F(4), KeyModifiers::NONE);
    terminal.draw(|frame| app.draw(frame)).unwrap();
    let top = gutter_symbols(&terminal, app.regions.diagnostics);
    assert!(!top.trim().is_empty());
    key(&mut app, &mut runtime, KeyCode::End, KeyModifiers::NONE);
    terminal.draw(|frame| app.draw(frame)).unwrap();
    assert_ne!(gutter_symbols(&terminal, app.regions.diagnostics), top);

    run(&mut app, &mut runtime, &array_source());
    app.set_view(View::Table);
    key(&mut app, &mut runtime, KeyCode::F(4), KeyModifiers::NONE);
    terminal.draw(|frame| app.draw(frame)).unwrap();
    let output = app.regions.output;
    let body = Rect::new(output.x, output.y + 1, output.width, output.height - 1);
    assert_eq!(
        terminal.backend().buffer()[(output.right(), output.y)].symbol(),
        " "
    );
    let top = gutter_symbols(&terminal, body);
    assert!(!top.trim().is_empty());
    key(&mut app, &mut runtime, KeyCode::End, KeyModifiers::NONE);
    terminal.draw(|frame| app.draw(frame)).unwrap();
    assert_ne!(gutter_symbols(&terminal, body), top);

    let mut expanded = Terminal::new(TestBackend::new(100, 120)).unwrap();
    expanded.draw(|frame| app.draw(frame)).unwrap();
    assert_eq!(app.scroll, 0);
    assert!(
        gutter_symbols(&expanded, app.regions.output)
            .trim()
            .is_empty()
    );
}

fn multiline_entry() -> Entry {
    let lines = (0..60)
        .map(|row| format!("line {row:02}"))
        .collect::<Vec<_>>()
        .join("\n");
    Entry {
        source: ".help".into(),
        output: output::Output::Message(lines.clone()),
        diagnostics: lines,
    }
}

fn array_source() -> String {
    let items = (0..60)
        .map(|value| value.to_string())
        .collect::<Vec<_>>()
        .join(",");
    format!("[{items}]")
}

fn gutter_symbols(terminal: &Terminal<TestBackend>, area: Rect) -> String {
    let screen = terminal.backend().buffer();
    (area.y..area.bottom())
        .map(|row| screen[(area.right(), row)].symbol())
        .collect()
}

fn screen_line(terminal: &Terminal<TestBackend>, row: u16) -> String {
    let screen = terminal.backend().buffer();
    (0..screen.area.width)
        .map(|column| screen[(column, row)].symbol())
        .collect()
}

#[test]
fn universal_claims_reach_terminal_submission_and_indent_every_quantifier_headers() {
    let mut app = App::new(false, false);
    let mut runtime = Runtime::new();
    run(&mut app, &mut runtime, "xs = [1, 2]");
    app.editor = editor::new("if every v in xs: v > 0");
    key(
        &mut app,
        &mut runtime,
        KeyCode::Char('j'),
        KeyModifiers::CONTROL,
    );
    assert_eq!(app.editor.lines()[1], "    ");
    app.editor.insert_str("r = 1");
    key(&mut app, &mut runtime, KeyCode::F(5), KeyModifiers::NONE);
    run(&mut app, &mut runtime, "r");
    assert_eq!(app.entries.last().unwrap().output.text(View::Text), "1");
    run(
        &mut app,
        &mut runtime,
        "class Human\nevery Human is mortal\nh = Human()\nh.mortal",
    );
    assert_eq!(app.entries.last().unwrap().output.text(View::Text), "true");
    run(&mut app, &mut runtime, ".clear");
    run(&mut app, &mut runtime, "every Human is mortal");
    assert_eq!(
        app.entries.last().unwrap().output.text(View::Text),
        "ReferenceError: Human is not defined (line 1, column 1)"
    );
    assert_eq!(runtime.assertions_run(), 0);
}

#[test]
fn any_and_all_class_references_reach_terminal_submission_and_indent_their_headers() {
    let mut app = App::new(false, false);
    let mut runtime = Runtime::new();
    run(
        &mut app,
        &mut runtime,
        "class Object(number):\n    this.number = number\nevery Object.label is square",
    );
    app.editor = editor::new("if any Object.number > 4");
    key(
        &mut app,
        &mut runtime,
        KeyCode::Char('j'),
        KeyModifiers::CONTROL,
    );
    assert_eq!(app.editor.lines()[1], "    ");
    app.editor.insert_str("all Object.color is red");
    key(&mut app, &mut runtime, KeyCode::F(5), KeyModifiers::NONE);
    run(
        &mut app,
        &mut runtime,
        "a = Object(3)\nb = Object(7)\nb.color",
    );
    assert_eq!(app.entries.last().unwrap().output.text(View::Text), "red");
    run(&mut app, &mut runtime, "a.color");
    assert_eq!(
        app.entries.last().unwrap().output.text(View::Text),
        "undefined"
    );
    run(&mut app, &mut runtime, "any Object is shaped\na.shaped");
    assert_eq!(app.entries.last().unwrap().output.text(View::Text), "true");
    run(&mut app, &mut runtime, ".clear");
    run(&mut app, &mut runtime, "all Object.color is red");
    assert_eq!(
        app.entries.last().unwrap().output.text(View::Text),
        "ReferenceError: Object is not defined (line 1, column 1)"
    );
    assert_eq!(runtime.assertions_run(), 0);
}

#[test]
fn every_inside_if_reaches_terminal_submission_as_a_syntax_error() {
    let mut app = App::new(false, false);
    let mut runtime = Runtime::new();
    run(
        &mut app,
        &mut runtime,
        "class Order(total):\n    this.total = total\nlarge = Order(200)",
    );
    app.editor = editor::new("if every Order.total > 100");
    key(
        &mut app,
        &mut runtime,
        KeyCode::Char('j'),
        KeyModifiers::CONTROL,
    );
    assert_eq!(app.editor.lines()[1], "    ");
    app.editor.insert_str("every Order.vip = true");
    key(&mut app, &mut runtime, KeyCode::F(5), KeyModifiers::NONE);
    assert_eq!(
        app.entries.last().unwrap().output.text(View::Text),
        "SyntaxError: Use all instead of every inside if (line 1, column 4)"
    );
    run(&mut app, &mut runtime, "large.vip");
    assert_eq!(
        app.entries.last().unwrap().output.text(View::Text),
        "undefined"
    );
    run(
        &mut app,
        &mut runtime,
        "if any Order.total > 100:\n    all Order.vip = true\nlarge.vip",
    );
    assert_eq!(app.entries.last().unwrap().output.text(View::Text), "true");
    assert_eq!(runtime.assertions_run(), 0);
}

#[test]
fn indented_newlines_open_a_repeat_suite_and_submission_runs_the_loop_once() {
    let mut app = App::new(false, false);
    let mut runtime = Runtime::new();
    for line in ["count = 0", "repeat 3:"] {
        app.editor.insert_str(line);
        key(
            &mut app,
            &mut runtime,
            KeyCode::Char('j'),
            KeyModifiers::CONTROL,
        );
    }
    app.editor.insert_str("count = count + 2");
    assert_eq!(
        app.editor.lines(),
        ["count = 0", "repeat 3:", "    count = count + 2"]
    );
    assert!(app.entries.is_empty());
    key(&mut app, &mut runtime, KeyCode::F(5), KeyModifiers::NONE);
    assert_eq!(
        runtime.state.variable("count"),
        Some(&nucleoid::Value::Number(6.0))
    );
    run(&mut app, &mut runtime, "count = 0\ncount");
    assert_eq!(app.entries.last().unwrap().output.text(View::Text), "0");
}
