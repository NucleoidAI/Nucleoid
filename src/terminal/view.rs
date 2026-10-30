use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, BorderType, Borders, Clear, List, ListItem, Paragraph, Row, Table, Wrap,
};
use unicode_width::UnicodeWidthStr;

use crate::{REPL_TAGLINE, REPL_TITLE};

use super::output::{TableData, View, visible};
use super::{App, Focus};

const BACKGROUND: Color = Color::Rgb(16, 20, 29);
const PANEL: Color = Color::Rgb(23, 29, 40);
const BORDER: Color = Color::Rgb(61, 73, 91);
const FOREGROUND: Color = Color::Rgb(218, 227, 238);
const MUTED: Color = Color::Rgb(132, 149, 170);
const ACCENT: Color = Color::Rgb(91, 220, 187);
const ERROR: Color = Color::Rgb(255, 133, 133);

#[derive(Default)]
pub(super) struct Regions {
    pub tabs: [Rect; 3],
    pub views: [Rect; 3],
    pub history: Rect,
    pub output: Rect,
    pub diagnostics: Rect,
    pub editor: Rect,
    pub completions: Rect,
    pub completion_start: usize,
}

pub(super) fn draw(frame: &mut Frame<'_>, app: &mut App) -> Regions {
    let area = frame.area();
    frame.render_widget(Block::default().style(base()), area);
    if area.width < 40 || area.height < 12 {
        frame.render_widget(
            Paragraph::new("Resize to at least 40 x 12.\nCtrl-Q exits.")
                .style(base())
                .wrap(Wrap { trim: false }),
            area,
        );
        return Regions::default();
    }
    let editor_height = (area.height / 3).clamp(5, 12);
    let layout = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(3),
        Constraint::Length(editor_height),
        Constraint::Length(1),
    ])
    .split(area);
    let mut regions = Regions::default();
    header(frame, layout[0], app, &mut regions);

    if app.tab == 0 {
        results(frame, layout[1], app, &mut regions);
    } else {
        frame.render_widget(
            Paragraph::new(format!(
                "\nTab {} is reserved for a future workspace.\n\nF1 or click REPL to return.",
                app.tab + 1
            ))
            .alignment(Alignment::Center)
            .style(Style::default().fg(MUTED))
            .block(panel(" Reserved ", false)),
            layout[1],
        );
    }

    let (row, column) = app.editor.cursor();
    let editor_block = panel(
        &format!(" Editor  {}:{} ", row + 1, column + 1),
        app.focus == Focus::Editor,
    )
    .title_bottom(Line::from(
        " Enter newline  F5 run  Tab complete  Ctrl-J autoindent ",
    ));
    regions.editor = editor_block.inner(layout[2]);
    app.editor.set_block(editor_block);
    app.editor.set_style(base().bg(PANEL));
    app.editor.set_cursor_line_style(Style::default());
    app.editor.set_cursor_style(if app.focus == Focus::Editor {
        Style::default().fg(BACKGROUND).bg(ACCENT)
    } else {
        Style::default().fg(FOREGROUND).bg(PANEL)
    });
    app.editor.set_selection_style(Style::default().bg(BORDER));
    app.editor.set_placeholder_style(Style::default().fg(MUTED));
    frame.render_widget(&app.editor, layout[2]);
    frame.render_widget(
        Paragraph::new(" F1-F3 tabs  F4 focus  F6 view  Ctrl-P/N history  .help  Ctrl-Q quit")
            .style(Style::default().fg(MUTED).bg(BACKGROUND)),
        layout[3],
    );

    if app.tab == 0 && app.focus == Focus::Editor {
        completions(frame, layout[2], app, &mut regions);
    }
    regions
}

fn header(frame: &mut Frame<'_>, area: Rect, app: &App, regions: &mut Regions) {
    let rows = Layout::vertical([Constraint::Length(2), Constraint::Length(1)]).split(area);
    frame.render_widget(
        Paragraph::new(vec![
            Line::styled(
                format!(" {REPL_TITLE}"),
                Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
            ),
            Line::styled(format!(" {REPL_TAGLINE}"), Style::default().fg(MUTED)),
        ])
        .style(base()),
        rows[0],
    );
    let parts = Layout::horizontal([
        Constraint::Length(12),
        Constraint::Length(12),
        Constraint::Length(12),
        Constraint::Min(0),
    ])
    .split(rows[1]);
    for (index, label) in ["F1 REPL", "F2 Tab 2", "F3 Tab 3"].into_iter().enumerate() {
        regions.tabs[index] = parts[index];
        button(frame, label, parts[index], app.tab == index);
    }
    frame.render_widget(
        Paragraph::new(format!("v{} ", env!("CARGO_PKG_VERSION")))
            .alignment(Alignment::Right)
            .style(Style::default().fg(MUTED)),
        parts[3],
    );
}

fn results(frame: &mut Frame<'_>, area: Rect, app: &mut App, regions: &mut Regions) {
    let layout = Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).split(area);
    let toolbar = Layout::horizontal([
        Constraint::Length(9),
        Constraint::Length(9),
        Constraint::Length(9),
        Constraint::Min(0),
    ])
    .split(layout[0]);
    for (index, view) in View::ALL.into_iter().enumerate() {
        regions.views[index] = toolbar[index];
        button(frame, view.label(), toolbar[index], app.view == view);
    }
    frame.render_widget(
        Paragraph::new(format!(
            "{} results  |  graph {} ",
            app.entries.len(),
            if app.graph { "on" } else { "off" }
        ))
        .alignment(Alignment::Right)
        .style(Style::default().fg(MUTED)),
        toolbar[3],
    );
    let history_width = if area.width >= 80 { 24 } else { 0 };
    let panes = Layout::horizontal([Constraint::Length(history_width), Constraint::Min(0)])
        .split(layout[1]);
    if history_width > 0 {
        let block = panel(" History ", false);
        regions.history = block.inner(panes[0]);
        let items: Vec<_> = app
            .entries
            .iter()
            .enumerate()
            .map(|(index, entry)| {
                ListItem::new(format!(
                    "{}  {}",
                    index + 1,
                    visible(entry.source.lines().next().unwrap_or_default(), false)
                ))
            })
            .collect();
        app.list_state
            .select((!app.entries.is_empty()).then_some(app.selected));
        frame.render_stateful_widget(
            List::new(items)
                .block(block)
                .style(base().bg(PANEL))
                .highlight_style(Style::default().fg(ACCENT).bg(BORDER)),
            panes[0],
            &mut app.list_state,
        );
    }

    let Some(entry) = app.entries.get(app.selected) else {
        frame.render_widget(
            Paragraph::new(vec![
                Line::from(""),
                Line::from(Span::styled(
                    "A workspace for your world model.",
                    Style::default().fg(ACCENT),
                )),
                Line::from(""),
                Line::from("Write code below. Results appear here."),
                Line::from("Switch between Text, JSON and Table without rerunning."),
                Line::from(""),
                Line::from("Tab completes names, properties and built-ins."),
                Line::from("Use .help for all keyboard and mouse controls."),
            ])
            .alignment(Alignment::Center)
            .style(base())
            .block(panel(" Results ", app.focus == Focus::Result))
            .wrap(Wrap { trim: false }),
            panes[1],
        );
        return;
    };
    let diagnostic_height = if entry.diagnostics.is_empty() {
        0
    } else {
        (panes[1].height / 3).clamp(3, 7)
    };
    let output_layout =
        Layout::vertical([Constraint::Min(0), Constraint::Length(diagnostic_height)])
            .split(panes[1]);
    let result_block = panel(
        &format!(" Result {} / {} ", app.selected + 1, app.entries.len()),
        app.focus == Focus::Result,
    )
    .title_bottom(Line::from(format!(
        " {} ",
        visible(entry.source.lines().next().unwrap_or_default(), false)
    )));
    regions.output = result_block.inner(output_layout[0]);
    frame.render_widget(result_block, output_layout[0]);
    if regions.output.height > 0 && regions.output.width > 0 {
        if let Some(table) = entry.output.table().filter(|_| app.view == View::Table) {
            draw_table(
                frame,
                regions.output,
                table,
                &mut app.scroll,
                &mut app.horizontal,
            );
        } else {
            let text = entry.output.text(app.view);
            clamp_scroll(text, regions.output, &mut app.scroll, &mut app.horizontal);
            frame.render_widget(
                Paragraph::new(text)
                    .style(
                        Style::default()
                            .fg(if entry.output.is_error(app.view) {
                                ERROR
                            } else {
                                FOREGROUND
                            })
                            .bg(PANEL),
                    )
                    .scroll((app.scroll, app.horizontal)),
                regions.output,
            );
        }
    }
    if diagnostic_height > 0 {
        let block = panel(" Diagnostics / graph ", app.focus == Focus::Diagnostics);
        regions.diagnostics = block.inner(output_layout[1]);
        clamp_scroll(
            &entry.diagnostics,
            regions.diagnostics,
            &mut app.diagnostic_scroll,
            &mut app.diagnostic_horizontal,
        );
        frame.render_widget(
            Paragraph::new(entry.diagnostics.as_str())
                .block(block)
                .style(Style::default().fg(MUTED).bg(PANEL))
                .scroll((app.diagnostic_scroll, app.diagnostic_horizontal)),
            output_layout[1],
        );
    }
}

fn draw_table(
    frame: &mut Frame<'_>,
    area: Rect,
    data: &TableData,
    scroll: &mut u16,
    column: &mut u16,
) {
    *column = (*column).min(as_u16(data.headers.len().saturating_sub(1)));
    *scroll = (*scroll).min(as_u16(
        data.rows
            .len()
            .saturating_sub(usize::from(area.height.saturating_sub(1))),
    ));
    let start = usize::from(*column);
    let mut remaining = area.width;
    let mut widths = Vec::new();
    for (index, header) in data.headers.iter().enumerate().skip(start) {
        if remaining == 0 {
            break;
        }
        let width = data
            .rows
            .iter()
            .map(|row| row[index].width())
            .max()
            .unwrap_or_default()
            .max(header.width())
            .clamp(8, 36);
        let width = as_u16(width).min(remaining);
        widths.push(Constraint::Length(width));
        remaining = remaining.saturating_sub(width.saturating_add(1));
    }
    let count = widths.len();
    let headers = data
        .headers
        .iter()
        .skip(start)
        .take(count)
        .map(|name| visible(name, false));
    let rows = data
        .rows
        .iter()
        .skip(usize::from(*scroll))
        .take(usize::from(area.height.saturating_sub(1)))
        .enumerate()
        .map(|(index, row)| {
            Row::new(row.iter().skip(start).take(count).map(String::as_str))
                .style(Style::default().bg(if index % 2 == 0 { PANEL } else { BACKGROUND }))
        });
    frame.render_widget(
        Table::new(rows, widths)
            .header(
                Row::new(headers).style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
            )
            .column_spacing(1)
            .style(base()),
        area,
    );
}

fn completions(frame: &mut Frame<'_>, editor_area: Rect, app: &App, regions: &mut Regions) {
    let Some(completion) = &app.completion else {
        return;
    };
    let height = as_u16(completion.candidates.len().min(6))
        .saturating_add(2)
        .min(editor_area.y.saturating_sub(3));
    if height < 3 {
        return;
    }
    let width = as_u16(
        completion
            .candidates
            .iter()
            .map(|name| name.width())
            .max()
            .unwrap_or(20),
    )
    .saturating_add(4)
    .clamp(24, 60)
    .min(editor_area.width);
    let area = Rect::new(editor_area.x, editor_area.y - height, width, height);
    let block = panel(" Complete / Tab ", true);
    regions.completions = block.inner(area);
    let visible_count = usize::from(regions.completions.height);
    let start = completion
        .selected
        .saturating_sub(visible_count.saturating_sub(1));
    regions.completion_start = start;
    let items = completion
        .candidates
        .iter()
        .enumerate()
        .skip(start)
        .take(visible_count)
        .map(|(index, candidate)| {
            ListItem::new(candidate.as_str()).style(if index == completion.selected {
                Style::default().fg(BACKGROUND).bg(ACCENT)
            } else {
                base().bg(PANEL)
            })
        });
    frame.render_widget(Clear, area);
    frame.render_widget(List::new(items).block(block).style(base().bg(PANEL)), area);
}

fn clamp_scroll(text: &str, area: Rect, vertical: &mut u16, horizontal: &mut u16) {
    *vertical = (*vertical).min(as_u16(
        text.lines()
            .count()
            .saturating_sub(usize::from(area.height)),
    ));
    let width = text
        .lines()
        .map(UnicodeWidthStr::width)
        .max()
        .unwrap_or_default();
    *horizontal = (*horizontal).min(as_u16(width.saturating_sub(usize::from(area.width))));
}

fn as_u16(value: usize) -> u16 {
    u16::try_from(value).unwrap_or(u16::MAX)
}

fn base() -> Style {
    Style::default().fg(FOREGROUND).bg(BACKGROUND)
}

fn panel(title: &str, focused: bool) -> Block<'static> {
    Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(if focused { ACCENT } else { BORDER }))
        .style(base().bg(PANEL))
        .title(title.to_owned())
}

fn button(frame: &mut Frame<'_>, label: &str, area: Rect, selected: bool) {
    frame.render_widget(
        Paragraph::new(label)
            .alignment(Alignment::Center)
            .style(
                Style::default()
                    .fg(if selected { ACCENT } else { MUTED })
                    .bg(if selected { PANEL } else { BACKGROUND }),
            )
            .block(
                Block::default()
                    .borders(if area.height > 1 {
                        Borders::BOTTOM
                    } else {
                        Borders::NONE
                    })
                    .border_style(Style::default().fg(if selected { ACCENT } else { BORDER })),
            ),
        area,
    );
}
