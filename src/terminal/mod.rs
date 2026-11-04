mod completion;
mod editor;
mod output;
mod view;

use std::io;
use std::time::{Duration, Instant};

use crossterm::event::{
    self, DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture,
    Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use crossterm::execute;
use nucleoid::Runtime;
use rand::seq::IteratorRandom;
use ratatui::Frame;
use ratatui::layout::Position;
use ratatui::widgets::ListState;
use tui_textarea::TextArea;

use completion::Completion;
use output::{Entry, Output, View};
use view::Regions;

const HISTORY_LIMIT: usize = 200;
const REPL_LOGO: &str = " ⣿⠋⢻⡄⣿ ⢸⡇⢰⡟⠙⠷\n ⣿ ⢸⡇⢻⣄⣼⡇⠸⣧⣠⡶";
const SPARKLE_DURATION: Duration = Duration::from_millis(220);
const HELP: &str = "\
.help            Show this help
.json [on|off]   Toggle the JSON result view
.table [on|off]  Toggle the table result view
.graph [on|off]  Include the logic graph with subsequent results
.clear           Clear runtime state and graph
.exit, .quit     Exit

Enter inserts a newline, never submits code. F5 runs the entire editor buffer.
Ctrl-Enter also runs on non-Windows terminals that report it distinctly.
Windows delivers pasted LF as Ctrl-Enter, so both insert a newline there.
Ctrl-J inserts an automatically indented newline.

Tab accepts a completion, or inserts four spaces when none is available.
Ctrl-Space opens completions; Up/Down selects; Escape dismisses.
Ctrl-P / Ctrl-N recalls history and restores your unfinished draft.
Ctrl-Z / Ctrl-Y undoes / redoes edits. Shift-arrows selects text.
Ctrl-X cuts; Ctrl-V pastes the editor's last cut text.
Ctrl-C cancels input. Ctrl-D on an empty editor / Ctrl-Q exits.

F1-F3 changes tabs; the extra tabs are reserved for later.
F6 changes Text / JSON / Table. Ctrl-Up / Ctrl-Down selects earlier results.
F4 cycles editor, result and diagnostics focus. PgUp/PgDn scrolls the focused
result; Left/Right scrolls text or moves table columns.

Click tabs, result views or history entries. Click/drag in the editor to
position/select text. Use the mouse wheel to scroll each pane; Shift-wheel
scrolls results horizontally. Pasted code is never executed automatically.";

pub(super) fn run(runtime: &mut Runtime, json: bool, graph: bool) -> io::Result<()> {
    let _session = Session;
    let mut terminal = ratatui::try_init()?;
    execute!(io::stdout(), EnableMouseCapture, EnableBracketedPaste)?;
    let mut app = App::new(json, graph);

    loop {
        app.sparkle.advance(Instant::now());
        terminal.draw(|frame| app.draw(frame))?;
        let timeout = app.sparkle.next.saturating_duration_since(Instant::now());
        if event::poll(timeout)? && app.event(event::read()?, runtime) {
            return Ok(());
        }
    }
}

struct Session;

impl Drop for Session {
    fn drop(&mut self) {
        let disabled = execute!(io::stdout(), DisableBracketedPaste, DisableMouseCapture);
        ratatui::restore();
        if let Err(error) = disabled {
            eprintln!("nucleoid: could not restore terminal input modes: {error}");
        }
    }
}

struct LogoSparkle {
    next: Instant,
    cell: Option<(usize, usize)>,
}

impl LogoSparkle {
    fn new(now: Instant) -> Self {
        Self {
            next: now + Duration::from_millis(rand::random_range(4_000..=12_000)),
            cell: None,
        }
    }

    fn advance(&mut self, now: Instant) {
        if now < self.next {
            return;
        }
        if self.cell.is_some() {
            *self = Self::new(now);
            return;
        }
        self.cell = REPL_LOGO
            .lines()
            .enumerate()
            .flat_map(|(row, line)| {
                line.chars()
                    .enumerate()
                    .filter(|(_, cell)| !cell.is_whitespace())
                    .map(move |(column, _)| (row, column))
            })
            .choose(&mut rand::rng());
        self.next = now + SPARKLE_DURATION;
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Focus {
    Editor,
    Result,
    Diagnostics,
}

struct App {
    editor: TextArea<'static>,
    completion: Option<Completion>,
    entries: Vec<Entry>,
    selected: usize,
    history: Vec<String>,
    history_index: Option<usize>,
    draft: String,
    list_state: ListState,
    tab: usize,
    view: View,
    graph: bool,
    focus: Focus,
    scroll: u16,
    horizontal: u16,
    diagnostic_scroll: u16,
    diagnostic_horizontal: u16,
    dragging: bool,
    sparkle: LogoSparkle,
    regions: Regions,
}

impl App {
    fn new(json: bool, graph: bool) -> Self {
        Self {
            editor: editor::new(""),
            completion: None,
            entries: Vec::new(),
            selected: 0,
            history: Vec::new(),
            history_index: None,
            draft: String::new(),
            list_state: ListState::default(),
            tab: 0,
            view: if json { View::Json } else { View::Text },
            graph,
            focus: Focus::Editor,
            scroll: 0,
            horizontal: 0,
            diagnostic_scroll: 0,
            diagnostic_horizontal: 0,
            dragging: false,
            sparkle: LogoSparkle::new(Instant::now()),
            regions: Regions::default(),
        }
    }

    fn draw(&mut self, frame: &mut Frame<'_>) {
        self.regions = view::draw(frame, self);
    }

    fn event(&mut self, event: Event, runtime: &mut Runtime) -> bool {
        match event {
            Event::Key(key) if key.kind != KeyEventKind::Release => self.key(key, runtime),
            Event::Paste(text) if self.focus == Focus::Editor => {
                editor::paste(&mut self.editor, &text);
                self.edited(runtime);
                false
            }
            Event::Mouse(mouse) => {
                self.mouse(mouse);
                false
            }
            _ => false,
        }
    }

    fn key(&mut self, key: KeyEvent, runtime: &mut Runtime) -> bool {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        if let Some(quit) = self.global_key(key.code, ctrl, runtime) {
            return quit;
        }
        if self.focus == Focus::Editor {
            self.editor_key(key, ctrl, runtime);
        } else {
            self.result_key(key.code);
        }
        false
    }

    fn global_key(&mut self, code: KeyCode, ctrl: bool, runtime: &mut Runtime) -> Option<bool> {
        match code {
            KeyCode::Char('q') if ctrl => return Some(true),
            KeyCode::Char('d') if ctrl && self.editor.is_empty() => return Some(true),
            KeyCode::Char('c') if ctrl => {
                self.editor = editor::new("");
                self.completion = None;
                self.history_index = None;
                self.focus = Focus::Editor;
            }
            KeyCode::F(number @ 1..=3) => {
                self.tab = usize::from(number - 1);
                self.completion = None;
            }
            KeyCode::Tab if ctrl => {
                self.tab = (self.tab + 1) % 3;
                self.completion = None;
            }
            KeyCode::F(4) => {
                self.focus = match self.focus {
                    Focus::Editor => Focus::Result,
                    Focus::Result
                        if self
                            .entries
                            .get(self.selected)
                            .is_some_and(|entry| !entry.diagnostics.is_empty()) =>
                    {
                        Focus::Diagnostics
                    }
                    _ => Focus::Editor,
                };
                self.completion = None;
            }
            KeyCode::F(6) => self.set_view(self.view.next()),
            KeyCode::F(5) => return Some(self.submit(runtime)),
            // Native Windows input maps pasted LF to Ctrl-Enter as well.
            KeyCode::Enter if ctrl && !cfg!(windows) => return Some(self.submit(runtime)),
            KeyCode::Up if ctrl => self.select_entry(self.selected.saturating_sub(1)),
            KeyCode::Down if ctrl => self.select_entry(self.selected.saturating_add(1)),
            KeyCode::PageUp => self.scroll_vertical(false, self.regions.output.height.max(1)),
            KeyCode::PageDown => self.scroll_vertical(true, self.regions.output.height.max(1)),
            KeyCode::Esc => {
                if self.completion.take().is_none() {
                    self.editor.cancel_selection();
                    self.focus = Focus::Editor;
                    self.tab = 0;
                }
            }
            _ => return None,
        }
        Some(false)
    }

    fn result_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::Up => self.scroll_vertical(false, 1),
            KeyCode::Down => self.scroll_vertical(true, 1),
            KeyCode::Left => self.scroll_horizontal(false, self.focus == Focus::Diagnostics),
            KeyCode::Right => self.scroll_horizontal(true, self.focus == Focus::Diagnostics),
            KeyCode::Home => {
                self.scroll = 0;
                self.diagnostic_scroll = 0;
                self.horizontal = 0;
                self.diagnostic_horizontal = 0;
            }
            KeyCode::End => self.scroll_vertical(true, u16::MAX),
            _ => {}
        }
    }

    fn editor_key(&mut self, key: KeyEvent, ctrl: bool, runtime: &Runtime) {
        match key.code {
            KeyCode::Char('p') if ctrl => self.recall(false),
            KeyCode::Char('n') if ctrl => self.recall(true),
            KeyCode::Char(' ') if ctrl => {
                self.completion = Completion::find(runtime, &self.editor, true);
            }
            KeyCode::Char('z') if ctrl => {
                if key.modifiers.contains(KeyModifiers::SHIFT) {
                    self.editor.redo();
                } else {
                    self.editor.undo();
                }
                self.edited(runtime);
            }
            KeyCode::Char('y') if ctrl => {
                self.editor.redo();
                self.edited(runtime);
            }
            KeyCode::Char('v') if ctrl => {
                self.editor.paste();
                self.edited(runtime);
            }
            KeyCode::Up | KeyCode::Down if self.completion.is_some() => {
                if let Some(completion) = &mut self.completion {
                    completion.select(key.code == KeyCode::Down);
                }
            }
            KeyCode::Up if self.editor.lines().len() == 1 => self.recall(false),
            KeyCode::Down if self.editor.lines().len() == 1 => self.recall(true),
            KeyCode::Tab => self.complete_or_indent(runtime),
            KeyCode::BackTab => {
                editor::dedent(&mut self.editor);
                self.completion = None;
            }
            KeyCode::Backspace if editor::dedent(&mut self.editor) => self.edited(runtime),
            KeyCode::Enter => {
                self.completion = None;
                self.editor.insert_newline();
                self.history_index = None;
            }
            KeyCode::Char('j') if ctrl => {
                editor::newline_with_options(&mut self.editor, runtime.options());
                self.completion = None;
                self.history_index = None;
            }
            _ => {
                if self.editor.input(key) {
                    self.edited(runtime);
                } else {
                    self.completion = None;
                }
            }
        }
    }

    fn complete_or_indent(&mut self, runtime: &Runtime) {
        if self.completion.is_none() {
            self.completion = Completion::find(runtime, &self.editor, false);
        }
        if let Some(completion) = self.completion.take() {
            completion.apply(&mut self.editor);
        } else {
            self.editor.insert_str(crate::INDENT);
        }
        self.history_index = None;
    }

    fn edited(&mut self, runtime: &Runtime) {
        self.history_index = None;
        self.completion = Completion::find(runtime, &self.editor, false);
    }

    fn submit(&mut self, runtime: &mut Runtime) -> bool {
        let source = self.editor.lines().join("\n");
        let source = source.trim_end().to_owned();
        self.editor = editor::new("");
        self.completion = None;
        self.history_index = None;
        self.focus = Focus::Editor;
        self.tab = 0;
        if source.trim().is_empty() {
            return false;
        }
        if self.history.last() != Some(&source) {
            if self.history.len() == HISTORY_LIMIT {
                self.history.remove(0);
            }
            self.history.push(source.clone());
        }

        let command = source.trim();
        if command.contains('\n') || !command.starts_with('.') {
            let entry = Entry::execute(runtime, source, self.graph);
            self.push(entry);
            return false;
        }
        let Some(output) = self.command(command, runtime) else {
            return true;
        };
        self.push(Entry {
            source,
            output,
            diagnostics: String::new(),
        });
        false
    }

    fn command(&mut self, command: &str, runtime: &mut Runtime) -> Option<Output> {
        let mut words = command.split_whitespace();
        Some(match words.next().unwrap_or_default() {
            ".exit" | ".quit" if words.next().is_none() => return None,
            ".help" if words.next().is_none() => Output::Message(HELP.into()),
            ".clear" if words.next().is_none() => {
                runtime.clear();
                Output::Message("State cleared.".into())
            }
            name @ (".json" | ".table" | ".graph") => self.setting(name, command),
            _ => Output::Error(format!(
                "Unknown command '{}'. Type .help for commands.",
                output::visible(command, false)
            )),
        })
    }

    fn setting(&mut self, name: &str, command: &str) -> Output {
        let argument = command.strip_prefix(name).unwrap_or_default().trim();
        let current = match name {
            ".json" => self.view == View::Json,
            ".table" => self.view == View::Table,
            _ => self.graph,
        };
        let Some(enabled) = crate::setting_value(argument, current) else {
            return Output::Error(format!("Usage: {name} [on|off]"));
        };
        match name {
            ".json" if enabled => self.set_view(View::Json),
            ".table" if enabled => self.set_view(View::Table),
            ".graph" => self.graph = enabled,
            _ if current => self.set_view(View::Text),
            _ => {}
        }
        Output::Message(format!(
            "{} {}.",
            name.trim_start_matches('.'),
            if enabled { "on" } else { "off" }
        ))
    }

    fn push(&mut self, entry: Entry) {
        if self.entries.len() == HISTORY_LIMIT {
            self.entries.remove(0);
        }
        self.entries.push(entry);
        self.select_entry(self.entries.len() - 1);
    }

    fn select_entry(&mut self, selected: usize) {
        self.selected = selected.min(self.entries.len().saturating_sub(1));
        self.scroll = 0;
        self.horizontal = 0;
        self.diagnostic_scroll = 0;
        self.diagnostic_horizontal = 0;
    }

    fn set_view(&mut self, view: View) {
        self.view = view;
        self.scroll = 0;
        self.horizontal = 0;
        self.completion = None;
    }

    fn recall(&mut self, forward: bool) {
        if self.history.is_empty() || forward && self.history_index.is_none() {
            return;
        }
        let index = match self.history_index {
            None => {
                self.draft = self.editor.lines().join("\n");
                self.history.len() - 1
            }
            Some(index) if forward => index + 1,
            Some(index) => index.saturating_sub(1),
        };
        self.completion = None;
        if index == self.history.len() {
            self.history_index = None;
            self.editor = editor::new(&self.draft);
        } else if let Some(recalled) = self.history.get(index) {
            self.history_index = Some(index);
            self.editor = editor::new(recalled);
        }
    }

    fn scroll_vertical(&mut self, forward: bool, amount: u16) {
        let scroll = if self.focus == Focus::Diagnostics {
            &mut self.diagnostic_scroll
        } else {
            &mut self.scroll
        };
        *scroll = if forward {
            scroll.saturating_add(amount)
        } else {
            scroll.saturating_sub(amount)
        };
    }

    fn scroll_horizontal(&mut self, forward: bool, diagnostics: bool) {
        let scroll = if diagnostics {
            &mut self.diagnostic_horizontal
        } else {
            &mut self.horizontal
        };
        *scroll = if forward {
            scroll.saturating_add(1)
        } else {
            scroll.saturating_sub(1)
        };
    }

    fn mouse(&mut self, mouse: MouseEvent) {
        let position = (mouse.column, mouse.row).into();
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => self.click(mouse, position),
            MouseEventKind::Drag(MouseButton::Left)
                if self.dragging && self.regions.editor.contains(position) =>
            {
                editor::click(
                    &mut self.editor,
                    self.regions.editor,
                    mouse.column,
                    mouse.row,
                    true,
                );
            }
            MouseEventKind::Up(MouseButton::Left) => {
                self.dragging = false;
                if self
                    .editor
                    .selection_range()
                    .is_some_and(|(start, end)| start == end)
                {
                    self.editor.cancel_selection();
                }
            }
            MouseEventKind::ScrollUp | MouseEventKind::ScrollDown => {
                self.wheel(mouse, position, mouse.kind == MouseEventKind::ScrollDown);
            }
            MouseEventKind::ScrollLeft | MouseEventKind::ScrollRight => {
                self.wheel_sideways(position, mouse.kind == MouseEventKind::ScrollRight);
            }
            _ => {}
        }
    }

    fn click(&mut self, mouse: MouseEvent, position: Position) {
        if self.regions.completions.contains(position) {
            if let Some(mut completion) = self.completion.take() {
                let index = self.regions.completion_start
                    + usize::from(mouse.row - self.regions.completions.y);
                if index < completion.candidates.len() {
                    completion.selected = index;
                    completion.apply(&mut self.editor);
                    self.history_index = None;
                }
            }
        } else if let Some(tab) = self
            .regions
            .tabs
            .iter()
            .position(|tab| tab.contains(position))
        {
            self.tab = tab;
            self.completion = None;
        } else if let Some(view) = View::ALL
            .into_iter()
            .zip(self.regions.views)
            .find_map(|(view, area)| area.contains(position).then_some(view))
        {
            self.set_view(view);
        } else if self.regions.editor.contains(position) {
            self.focus = Focus::Editor;
            self.completion = None;
            self.dragging = true;
            editor::click(
                &mut self.editor,
                self.regions.editor,
                mouse.column,
                mouse.row,
                false,
            );
        } else if self.regions.history.contains(position) {
            let index = self.list_state.offset() + usize::from(mouse.row - self.regions.history.y);
            if index < self.entries.len() {
                self.select_entry(index);
            }
        } else if self.regions.diagnostics.contains(position) {
            self.focus = Focus::Diagnostics;
            self.completion = None;
        } else if self.regions.output.contains(position) {
            self.focus = Focus::Result;
            self.completion = None;
        }
    }

    fn wheel(&mut self, mouse: MouseEvent, position: Position, forward: bool) {
        let sideways = mouse.modifiers.contains(KeyModifiers::SHIFT);
        if self.regions.editor.contains(position) {
            self.editor.scroll((if forward { 3 } else { -3 }, 0));
            self.completion = None;
        } else if self.regions.history.contains(position) {
            self.select_entry(if forward {
                self.selected.saturating_add(1)
            } else {
                self.selected.saturating_sub(1)
            });
        } else if self.regions.diagnostics.contains(position) {
            if sideways {
                self.scroll_horizontal(forward, true);
            } else {
                self.diagnostic_scroll = scrolled(self.diagnostic_scroll, forward);
            }
        } else if self.regions.output.contains(position) {
            if sideways {
                self.scroll_horizontal(forward, false);
            } else {
                self.scroll = scrolled(self.scroll, forward);
            }
        }
    }

    fn wheel_sideways(&mut self, position: Position, forward: bool) {
        if self.regions.editor.contains(position) {
            self.editor.scroll((0, if forward { 3 } else { -3 }));
            self.completion = None;
        } else if self.regions.diagnostics.contains(position) {
            self.scroll_horizontal(forward, true);
        } else if self.regions.output.contains(position) {
            self.scroll_horizontal(forward, false);
        }
    }
}

fn scrolled(offset: u16, forward: bool) -> u16 {
    if forward {
        offset.saturating_add(3)
    } else {
        offset.saturating_sub(3)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{Terminal, backend::TestBackend};

    fn key(app: &mut App, runtime: &mut Runtime, code: KeyCode, modifiers: KeyModifiers) -> bool {
        app.event(Event::Key(KeyEvent::new(code, modifiers)), runtime)
    }

    fn run(app: &mut App, runtime: &mut Runtime, source: &str) {
        app.editor = editor::new(source);
        assert!(!key(app, runtime, KeyCode::F(5), KeyModifiers::NONE));
    }

    #[test]
    fn opted_in_comments_reach_terminal_submission_indentation_and_diagnostics() {
        let mut app = App::new(false, false);
        let mut runtime = Runtime::with_options(nucleoid::SyntaxOptions {
            c_style_comments: true,
        });
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
}
