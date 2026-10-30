mod completion;
mod editor;
mod output;
mod view;

use std::io;

use crossterm::event::{
    self, DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture,
    Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use crossterm::execute;
use nucleoid::Runtime;
use ratatui::Frame;
use ratatui::widgets::ListState;
use tui_textarea::TextArea;

use completion::Completion;
use output::{Entry, Output, View};
use view::Regions;

const HISTORY_LIMIT: usize = 200;
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
        terminal.draw(|frame| app.draw(frame))?;
        if app.event(event::read()?, runtime) {
            return Ok(());
        }
    }
}

struct Session;

impl Drop for Session {
    fn drop(&mut self) {
        let _ = execute!(io::stdout(), DisableBracketedPaste, DisableMouseCapture);
        ratatui::restore();
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
        match key.code {
            KeyCode::Char('q') if ctrl => return true,
            KeyCode::Char('d') if ctrl && self.editor.is_empty() => return true,
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
            KeyCode::F(5) => return self.submit(runtime),
            // Native Windows input maps pasted LF to Ctrl-Enter as well.
            KeyCode::Enter if ctrl && !cfg!(windows) => return self.submit(runtime),
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
            _ if self.focus != Focus::Editor => match key.code {
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
            },
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
            KeyCode::Tab => {
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
                editor::newline(&mut self.editor);
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
        false
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
        let output = if !command.contains('\n') && command.starts_with('.') {
            let mut words = command.split_whitespace();
            match words.next().unwrap_or_default() {
                ".exit" | ".quit" if words.next().is_none() => return true,
                ".help" if words.next().is_none() => Output::Message(HELP.into()),
                ".clear" if words.next().is_none() => {
                    runtime.clear();
                    Output::Message("State cleared.".into())
                }
                name @ (".json" | ".table" | ".graph") => {
                    let argument = command.strip_prefix(name).unwrap_or_default().trim();
                    let current = match name {
                        ".json" => self.view == View::Json,
                        ".table" => self.view == View::Table,
                        _ => self.graph,
                    };
                    match crate::setting_value(argument, current) {
                        Some(enabled) => {
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
                        None => Output::Error(format!("Usage: {name} [on|off]")),
                    }
                }
                _ => Output::Error(format!(
                    "Unknown command '{}'. Type .help for commands.",
                    output::visible(command, false)
                )),
            }
        } else {
            let entry = Entry::execute(runtime, source, self.graph);
            self.push(entry);
            return false;
        };
        self.push(Entry {
            source,
            output,
            diagnostics: String::new(),
        });
        false
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
        } else {
            self.history_index = Some(index);
            self.editor = editor::new(&self.history[index]);
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
            MouseEventKind::Down(MouseButton::Left) => {
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
                } else if let Some(view) = self
                    .regions
                    .views
                    .iter()
                    .position(|view| view.contains(position))
                {
                    self.set_view(View::ALL[view]);
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
                    let index =
                        self.list_state.offset() + usize::from(mouse.row - self.regions.history.y);
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
                let forward = mouse.kind == MouseEventKind::ScrollDown;
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
                    if mouse.modifiers.contains(KeyModifiers::SHIFT) {
                        self.scroll_horizontal(forward, true);
                    } else {
                        self.diagnostic_scroll = if forward {
                            self.diagnostic_scroll.saturating_add(3)
                        } else {
                            self.diagnostic_scroll.saturating_sub(3)
                        };
                    }
                } else if self.regions.output.contains(position) {
                    if mouse.modifiers.contains(KeyModifiers::SHIFT) {
                        self.scroll_horizontal(forward, false);
                    } else {
                        self.scroll = if forward {
                            self.scroll.saturating_add(3)
                        } else {
                            self.scroll.saturating_sub(3)
                        };
                    }
                }
            }
            MouseEventKind::ScrollLeft | MouseEventKind::ScrollRight => {
                let forward = mouse.kind == MouseEventKind::ScrollRight;
                if self.regions.editor.contains(position) {
                    self.editor.scroll((0, if forward { 3 } else { -3 }));
                    self.completion = None;
                } else if self.regions.diagnostics.contains(position) {
                    self.scroll_horizontal(forward, true);
                } else if self.regions.output.contains(position) {
                    self.scroll_horizontal(forward, false);
                }
            }
            _ => {}
        }
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
        for (width, height) in [(80, 24), (100, 30)] {
            let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
            terminal.draw(|frame| app.draw(frame)).unwrap();
            let screen = terminal.backend().buffer();
            let line = |row| {
                (0..width)
                    .map(|column| screen[(column, row)].symbol())
                    .collect::<String>()
            };
            assert_eq!(screen[(1, 0)].symbol(), "🌿");
            assert!(line(0).contains("Nucleoid - Logic Language for World Models"));
            assert_eq!(screen[(1, 1)].symbol(), "🌎");
            assert!(line(1).contains("Inspired by Nature"));
            for label in ["F1 REPL", "F2 Tab 2", "F3 Tab 3"] {
                assert!(line(2).contains(label));
            }
            assert!(
                app.regions
                    .tabs
                    .iter()
                    .all(|tab| tab.y == 2 && tab.height == 1)
            );
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
                    .map(|cell| cell.symbol())
                    .collect::<String>();
                assert!(text.contains("Nucleoid"));
                assert!(text.contains("Editor"));
            }
        }
    }
}
