use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use super::{ACCENT, FOREGROUND, HIGHLIGHT, MUTED, PANEL};
use crate::terminal::{App, Focus};

pub(super) fn draw(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let label = match app.focus {
        Focus::Editor => "EDITOR",
        Focus::Result => "RESULT",
        Focus::Diagnostics if area.width >= 80 => "DIAGNOSTICS",
        Focus::Diagnostics => "DIAG",
    };
    let mut spans = vec![
        Span::styled(
            format!(" {label} "),
            Style::default()
                .fg(ACCENT)
                .bg(HIGHLIGHT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
    ];
    hint(&mut spans, "F5", "run");
    if area.width >= 80 {
        match app.focus {
            Focus::Editor
                if app
                    .editor
                    .selection_range()
                    .is_some_and(|(start, end)| start != end) =>
            {
                hint(&mut spans, "Tab", "indent");
                hint(&mut spans, "Shift-Tab", "unindent");
            }
            Focus::Editor if app.completion.is_some() => {
                hint(&mut spans, "Tab", "accept");
                hint(&mut spans, "Esc", "dismiss");
            }
            Focus::Editor => {
                hint(&mut spans, "Enter", "newline");
                hint(&mut spans, "Tab", "complete");
            }
            Focus::Result | Focus::Diagnostics => {
                hint(&mut spans, "PgUp/Dn", "scroll");
                hint(&mut spans, "Left/Right", "pan");
            }
        }
    }
    hint(&mut spans, "F4", "focus");
    if area.width >= 100 {
        hint(&mut spans, "F6", "view");
    }
    hint(&mut spans, "Ctrl-Q", "quit");
    frame.render_widget(
        Paragraph::new(Line::from(spans)).style(Style::default().fg(MUTED).bg(PANEL)),
        area,
    );
}

fn hint(spans: &mut Vec<Span<'static>>, key: &'static str, action: &'static str) {
    spans.push(Span::styled(
        key,
        Style::default().fg(FOREGROUND).add_modifier(Modifier::BOLD),
    ));
    spans.push(Span::raw(format!(" {action}  ")));
}
