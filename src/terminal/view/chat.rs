use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::Paragraph;
use unicode_width::UnicodeWidthStr;

use super::content::gutter_scrollbar;
use super::{ACCENT, ERROR, FOREGROUND, MUTED, PANEL, Regions, as_u16, base, panel};
use crate::terminal::output::{Entry, TableData, View, visible};
use crate::terminal::{App, Focus};

pub(in crate::terminal) struct Transcript {
    pub(in crate::terminal) scroll: usize,
    pub(in crate::terminal) horizontal: u16,
    follow_tail: bool,
    selected: Option<usize>,
}

impl Default for Transcript {
    fn default() -> Self {
        Self {
            scroll: 0,
            horizontal: 0,
            follow_tail: true,
            selected: None,
        }
    }
}

impl Transcript {
    pub(in crate::terminal) fn scroll(&mut self, forward: bool, amount: u16) {
        self.follow_tail = false;
        self.selected = None;
        self.scroll = if forward {
            self.scroll.saturating_add(usize::from(amount))
        } else {
            self.scroll.saturating_sub(usize::from(amount))
        };
    }

    pub(in crate::terminal) fn home(&mut self) {
        self.scroll = 0;
        self.horizontal = 0;
        self.follow_tail = false;
        self.selected = None;
    }

    pub(in crate::terminal) fn follow(&mut self) {
        self.end();
        self.horizontal = 0;
    }

    pub(in crate::terminal) fn end(&mut self) {
        self.follow_tail = true;
        self.selected = None;
    }

    pub(in crate::terminal) fn select(&mut self, index: usize) {
        self.follow_tail = false;
        self.horizontal = 0;
        self.selected = Some(index);
    }

    fn layout(&mut self, area: Rect, content: &Content) {
        let last = content.lines.len().saturating_sub(usize::from(area.height));
        if let Some(start) = self
            .selected
            .take()
            .and_then(|index| content.entries.get(index))
        {
            self.scroll = *start;
        } else if self.follow_tail {
            self.scroll = last;
        }
        self.scroll = self.scroll.min(last);
        self.follow_tail = self.scroll == last;
        let width = content
            .lines
            .iter()
            .map(Line::width)
            .max()
            .unwrap_or_default();
        self.horizontal = self
            .horizontal
            .min(as_u16(width.saturating_sub(usize::from(area.width))));
    }
}

#[derive(Default)]
struct Content {
    lines: Vec<Line<'static>>,
    entries: Vec<usize>,
}

pub(super) fn draw(frame: &mut Frame<'_>, area: Rect, app: &mut App, regions: &mut Regions) {
    let block = panel(" Chat ", app.focus == Focus::Result);
    regions.output = block.inner(area);
    frame.render_widget(block, area);
    let content = content(&app.entries, app.view, app.selected);
    app.transcript.layout(regions.output, &content);
    let lines = content.lines.len();
    let visible: Vec<_> = content
        .lines
        .into_iter()
        .skip(app.transcript.scroll)
        .take(usize::from(regions.output.height))
        .collect();
    frame.render_widget(
        Paragraph::new(visible)
            .style(base().bg(PANEL))
            .scroll((0, app.transcript.horizontal)),
        regions.output,
    );
    gutter_scrollbar(frame, regions.output, lines, app.transcript.scroll);
}

fn content(entries: &[Entry], view: View, selected: usize) -> Content {
    let mut content = Content::default();
    if entries.is_empty() {
        content.lines.push(Line::styled(
            "Write code below. Enter runs it.",
            Style::default().fg(MUTED),
        ));
    }
    for (index, entry) in entries.iter().enumerate() {
        if index > 0 {
            content.lines.push(Line::default());
        }
        content.entries.push(content.lines.len());
        content.lines.push(Line::styled(
            format!("You {}", index + 1),
            Style::default()
                .fg(if index == selected { ACCENT } else { MUTED })
                .add_modifier(Modifier::BOLD),
        ));
        append_text(
            &mut content.lines,
            &entry.source,
            Style::default().fg(FOREGROUND),
        );
        let failed = entry.output.is_error(view);
        let result_style = Style::default().fg(if failed { ERROR } else { FOREGROUND });
        content.lines.push(Line::styled(
            if failed {
                "Nucleoid / Error"
            } else {
                "Nucleoid"
            },
            result_style.add_modifier(Modifier::BOLD),
        ));
        if let Some(table) = entry.output.table().filter(|_| view == View::Table) {
            append_table(&mut content.lines, table);
        } else {
            append_text(&mut content.lines, entry.output.text(view), result_style);
        }
        if !entry.diagnostics.is_empty() {
            content.lines.push(Line::styled(
                "Diagnostics / graph",
                Style::default().fg(MUTED).add_modifier(Modifier::BOLD),
            ));
            append_text(
                &mut content.lines,
                &entry.diagnostics,
                Style::default().fg(MUTED),
            );
        }
    }
    content
}

fn append_text(lines: &mut Vec<Line<'static>>, text: &str, style: Style) {
    lines.extend(
        visible(text, true)
            .split('\n')
            .map(|line| Line::styled(line.to_owned(), style)),
    );
}

fn append_table(lines: &mut Vec<Line<'static>>, table: &TableData) {
    let widths: Vec<_> = table
        .headers
        .iter()
        .enumerate()
        .map(|(index, header)| {
            table
                .rows
                .iter()
                .filter_map(|row| row.get(index))
                .map(|cell| cell.width())
                .max()
                .unwrap_or_default()
                .max(visible(header, false).width())
                .clamp(8, 36)
        })
        .collect();
    lines.push(Line::styled(
        table_row(&table.headers, &widths),
        Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
    ));
    lines.extend(
        table
            .rows
            .iter()
            .map(|row| Line::from(table_row(row, &widths))),
    );
}

fn table_row(cells: &[String], widths: &[usize]) -> String {
    cells
        .iter()
        .zip(widths)
        .map(|(text, width)| fit_cell(text, *width))
        .collect::<Vec<_>>()
        .join(" | ")
}

fn fit_cell(text: &str, width: usize) -> String {
    let line = Line::from(visible(text, false));
    let mut cell = String::new();
    let mut columns = 0;
    for grapheme in line.styled_graphemes(Style::default()) {
        let grapheme_width = grapheme.symbol.width();
        if columns + grapheme_width > width {
            break;
        }
        cell.push_str(grapheme.symbol);
        columns += grapheme_width;
    }
    cell.push_str(&" ".repeat(width - columns));
    cell
}
