use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use nucleoid::Runtime;

use super::completion::Completion;
use super::output::{Entry, Output, View};
use super::{App, Focus, HELP, HISTORY_LIMIT, editor, output};

impl App {
    pub(super) fn key(&mut self, key: KeyEvent, runtime: &mut Runtime) -> bool {
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

    pub(super) fn edited(&mut self, runtime: &Runtime) {
        self.history_index = None;
        self.completion = Completion::find(runtime, &self.editor, false);
    }

    pub(super) fn submit(&mut self, runtime: &mut Runtime) -> bool {
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
        let Some(enabled) = crate::interactive::setting_value(argument, current) else {
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

    pub(super) fn select_entry(&mut self, selected: usize) {
        self.selected = selected.min(self.entries.len().saturating_sub(1));
        self.scroll = 0;
        self.horizontal = 0;
        self.diagnostic_scroll = 0;
        self.diagnostic_horizontal = 0;
    }

    pub(super) fn set_view(&mut self, view: View) {
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
}
