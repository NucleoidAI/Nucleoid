mod completion;
mod editor;
mod indentation;
mod input;
mod output;
mod pointer;
mod view;

use std::io;
use std::time::{Duration, Instant};

use crossterm::event::{
    self, DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture,
    Event, KeyEventKind,
};
use crossterm::execute;
use nucleoid::Runtime;
use rand::seq::IteratorRandom;
use ratatui::Frame;
use ratatui::widgets::ListState;
use tui_textarea::TextArea;

use completion::Completion;
use output::{Entry, View};
use view::{Regions, Transcript};

const HISTORY_LIMIT: usize = 200;
const REPL_LOGO: &str = " ⣿⠋⢻⡄⣿ ⢸⡇⢰⡟⠙⠷\n ⣿ ⢸⡇⢻⣄⣼⡇⠸⣧⣠⡶";
const SPARKLE_DURATION: Duration = Duration::from_millis(220);
const HELP: &str = "\
.help            Show this help
.json [on|off]   Toggle the JSON result view
.table [on|off]  Toggle the table result view
.graph [on|off]  Include the logic graph with subsequent results
.editor          Switch to editor mode
.chat            Switch to chat mode
.clear           Clear runtime state and graph
.exit, .quit     Exit

F1 selects editor mode; F2 selects chat mode. Ctrl-Tab switches modes.
Both modes share runtime state, input history, results and the unfinished draft.
Editor: Enter inserts a newline. Chat: Enter submits code to the transcript.
Shift-Enter inserts a newline when the terminal reports it distinctly.
F5 runs the entire input in either mode.
Ctrl-Enter also runs on non-Windows terminals that report it distinctly.
Windows delivers pasted LF as Ctrl-Enter, so both insert a newline there.
Ctrl-J inserts an automatically indented newline.

Tab accepts a completion, or inserts four spaces when none is available.
With selected text, Tab indents its lines without replacing the selection.
Shift-Tab unindents the selected lines, or the current line from any position.
Ctrl-Space opens completions; Up/Down selects; Escape dismisses.
Ctrl-P / Ctrl-N recalls history and restores your unfinished draft.
Ctrl-Z / Ctrl-Y undoes / redoes edits, including each shifted line.
Shift-arrows selects text.
Ctrl-X cuts; Ctrl-V pastes the editor's last cut text.
Ctrl-C cancels input. Ctrl-D on an empty editor / Ctrl-Q exits.

F6 changes Text / JSON / Table. Ctrl-Up / Ctrl-Down selects earlier results.
F4 cycles editor, result and diagnostics focus. PgUp/PgDn scrolls by a visible
page; Left/Right scrolls text or moves table columns. Scrollbars mark overflow.
Chat includes diagnostics in the transcript; F4 switches input/transcript focus.
Home/End with transcript focus jumps to the start/end of chat history.

Click modes, result views or history entries. Click/drag in the editor to
position/select text. Use the mouse wheel to scroll each pane; Shift-wheel
scrolls results horizontally. Over completions, the wheel selects suggestions
without scrolling the pane underneath. Pasted code is never executed automatically.";

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
enum Mode {
    Editor,
    Chat,
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
    mode: Mode,
    transcript: Transcript,
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
            mode: Mode::Editor,
            transcript: Transcript::default(),
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
}

#[cfg(test)]
mod tests;
