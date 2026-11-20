mod content;
mod status;

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, List, ListItem, Padding, Paragraph, Wrap};
use unicode_width::UnicodeWidthStr;

use crate::{REPL_TAGLINE, REPL_TITLE};

use super::output::{View, visible};
use super::{App, Focus, REPL_LOGO};
use content::{completions, draw_diagnostics, draw_table, draw_text};

const BACKGROUND: Color = Color::Black;
const PANEL: Color = Color::Rgb(12, 12, 12);
const ACTIVE_LINE: Color = Color::Rgb(20, 20, 20);
const HIGHLIGHT: Color = Color::Rgb(34, 34, 34);
const BORDER: Color = Color::Rgb(64, 64, 64);
const FOREGROUND: Color = Color::Rgb(230, 230, 230);
const MUTED: Color = Color::Rgb(155, 155, 155);
const ACCENT: Color = Color::Rgb(154, 218, 183);
const ERROR: Color = Color::Rgb(245, 139, 139);

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
    let editor_height = (area.height / 3)
        .clamp(3, 12)
        .min(area.height.saturating_sub(9));
    let [top, body, editor, status] = Layout::vertical([
        Constraint::Length(4),
        Constraint::Min(4),
        Constraint::Length(editor_height),
        Constraint::Length(1),
    ])
    .areas(area);
    let mut regions = Regions::default();
    header(frame, top, app, &mut regions);

    if app.tab == 0 {
        results(frame, body, app, &mut regions);
    } else {
        reserved(frame, body, app);
    }

    editor_panel(frame, editor, app, &mut regions);
    status::draw(frame, status, app);

    if app.tab == 0 && app.focus == Focus::Editor {
        completions(frame, editor, app, &mut regions);
    }
    regions
}

fn reserved(frame: &mut Frame<'_>, area: Rect, app: &App) {
    frame.render_widget(
        Paragraph::new(format!(
            "\nTab {} is reserved for a future workspace.\n\nF1 or click REPL to return.",
            app.tab + 1
        ))
        .alignment(Alignment::Center)
        .style(Style::default().fg(MUTED))
        .block(panel(" Reserved ", false)),
        area,
    );
}

fn editor_panel(frame: &mut Frame<'_>, area: Rect, app: &mut App, regions: &mut Regions) {
    let (row, column) = app.editor.cursor();
    let editor_block = panel(
        &format!(" Editor  {}:{} ", row + 1, column + 1),
        app.focus == Focus::Editor,
    )
    .title(
        Line::styled(
            " F5 run ",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Right),
    )
    .title_bottom(Line::from(if area.width >= 70 {
        " Enter newline  Tab complete  Ctrl-J autoindent "
    } else {
        " Enter newline  Tab complete "
    }));
    regions.editor = editor_block.inner(area);
    app.editor.set_block(editor_block);
    app.editor.set_style(base().bg(PANEL));
    app.editor
        .set_cursor_line_style(Style::default().bg(if app.focus == Focus::Editor {
            ACTIVE_LINE
        } else {
            PANEL
        }));
    app.editor.set_cursor_style(if app.focus == Focus::Editor {
        Style::default().fg(BACKGROUND).bg(ACCENT)
    } else {
        Style::default().fg(FOREGROUND).bg(PANEL)
    });
    app.editor
        .set_selection_style(Style::default().fg(FOREGROUND).bg(HIGHLIGHT));
    app.editor.set_placeholder_style(Style::default().fg(MUTED));
    frame.render_widget(&app.editor, area);
}

fn header(frame: &mut Frame<'_>, area: Rect, app: &App, regions: &mut Regions) {
    let [_, banner, tabs] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(2),
        Constraint::Length(1),
    ])
    .areas(area);
    let logo_width = as_u16(
        REPL_LOGO
            .lines()
            .map(UnicodeWidthStr::width)
            .max()
            .unwrap_or_default(),
    )
    .saturating_add(1);
    let [logo, title] =
        Layout::horizontal([Constraint::Length(logo_width), Constraint::Min(0)]).areas(banner);
    frame.render_widget(
        Paragraph::new(logo_lines(app)).style(base().fg(Color::LightGreen)),
        logo,
    );
    frame.render_widget(
        Paragraph::new(vec![
            Line::styled(
                format!(" {REPL_TITLE}"),
                Style::default().fg(FOREGROUND).add_modifier(Modifier::BOLD),
            ),
            Line::styled(format!(" {REPL_TAGLINE}"), Style::default().fg(MUTED)),
        ])
        .style(base()),
        title,
    );
    let [first, second, third, version] = Layout::horizontal([
        Constraint::Length(12),
        Constraint::Length(12),
        Constraint::Length(12),
        Constraint::Min(0),
    ])
    .areas(tabs);
    regions.tabs = [first, second, third];
    let labels = ["F1 REPL", "F2 Tab 2", "F3 Tab 3"];
    for (index, (label, tab)) in labels.into_iter().zip(regions.tabs).enumerate() {
        button(frame, label, tab, app.tab == index);
    }
    frame.render_widget(
        Paragraph::new(format!("v{} ", env!("CARGO_PKG_VERSION")))
            .alignment(Alignment::Right)
            .style(Style::default().fg(MUTED)),
        version,
    );
}

fn logo_lines(app: &App) -> Vec<Line<'static>> {
    REPL_LOGO
        .lines()
        .enumerate()
        .map(|(row, line)| {
            Line::from(
                line.chars()
                    .enumerate()
                    .map(|(column, cell)| {
                        let style = if app.sparkle.cell == Some((row, column)) {
                            Style::default()
                                .fg(Color::White)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default()
                        };
                        Span::styled(cell.to_string(), style)
                    })
                    .collect::<Vec<_>>(),
            )
        })
        .collect()
}

fn results(frame: &mut Frame<'_>, area: Rect, app: &mut App, regions: &mut Regions) {
    let [toolbar, body] = Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).areas(area);
    let [text, json, table, summary] = Layout::horizontal([
        Constraint::Length(9),
        Constraint::Length(9),
        Constraint::Length(9),
        Constraint::Min(0),
    ])
    .areas(toolbar);
    regions.views = [text, json, table];
    for (view, view_area) in View::ALL.into_iter().zip(regions.views) {
        button(frame, view.label(), view_area, app.view == view);
    }
    let summary_text = if summary.width >= 24 {
        format!(
            "{} results  |  graph {} ",
            app.entries.len(),
            if app.graph { "on" } else { "off" }
        )
    } else {
        format!("{} results ", app.entries.len())
    };
    frame.render_widget(
        Paragraph::new(summary_text)
            .alignment(Alignment::Right)
            .style(Style::default().fg(MUTED)),
        summary,
    );
    let history_width = if area.width >= 80 { 24 } else { 0 };
    let [history, output] =
        Layout::horizontal([Constraint::Length(history_width), Constraint::Min(0)]).areas(body);
    if history_width > 0 {
        history_panel(frame, history, app, regions);
    }
    entry_panels(frame, output, app, regions);
}

fn history_panel(frame: &mut Frame<'_>, area: Rect, app: &mut App, regions: &mut Regions) {
    let block = panel(" History ", false);
    regions.history = block.inner(area);
    let items: Vec<_> = if app.entries.is_empty() {
        vec![ListItem::new("No results yet").style(Style::default().fg(MUTED))]
    } else {
        app.entries
            .iter()
            .enumerate()
            .map(|(index, entry)| {
                let failed = entry.output.is_error(app.view);
                ListItem::new(Line::from(vec![
                    Span::styled(format!("{:>3} ", index + 1), Style::default().fg(MUTED)),
                    Span::styled(if failed { "! " } else { "  " }, Style::default().fg(ERROR)),
                    Span::raw(source_preview(&entry.source)),
                ]))
            })
            .collect()
    };
    app.list_state
        .select((!app.entries.is_empty()).then_some(app.selected));
    frame.render_stateful_widget(
        List::new(items)
            .block(block)
            .style(base().bg(PANEL))
            .highlight_style(
                Style::default()
                    .fg(FOREGROUND)
                    .bg(HIGHLIGHT)
                    .add_modifier(Modifier::BOLD),
            ),
        area,
        &mut app.list_state,
    );
}

fn welcome(frame: &mut Frame<'_>, area: Rect, focused: bool) -> Rect {
    let block = panel(" Results ", focused);
    let inner = block.inner(area);
    frame.render_widget(
        Paragraph::new(vec![
            Line::from(Span::styled(
                "Write code below. F5 runs it.",
                Style::default().fg(FOREGROUND).add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from("Switch between Text, JSON and Table without rerunning."),
            Line::from(""),
            Line::from("Tab completes names, properties and built-ins."),
            Line::from("Use .help for all keyboard and mouse controls."),
        ])
        .alignment(Alignment::Center)
        .style(base().fg(MUTED))
        .block(block)
        .wrap(Wrap { trim: false }),
        area,
    );
    inner
}

fn entry_panels(frame: &mut Frame<'_>, area: Rect, app: &mut App, regions: &mut Regions) {
    let Some(entry) = app.entries.get(app.selected) else {
        regions.output = welcome(frame, area, app.focus == Focus::Result);
        return;
    };
    let diagnostic_height = if entry.diagnostics.is_empty() {
        0
    } else if area.height < 6 {
        if app.focus == Focus::Diagnostics {
            area.height
        } else {
            0
        }
    } else {
        (area.height / 3).clamp(3, 7)
    };
    let [result, diagnostics] =
        Layout::vertical([Constraint::Min(0), Constraint::Length(diagnostic_height)]).areas(area);
    let result_block = result_block(app, diagnostic_height == 0);
    regions.output = result_block.inner(result);
    frame.render_widget(result_block, result);
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
            draw_text(
                frame,
                regions.output,
                &entry.output,
                app.view,
                &mut app.scroll,
                &mut app.horizontal,
            );
        }
    }
    if diagnostic_height > 0 {
        regions.diagnostics = draw_diagnostics(
            frame,
            diagnostics,
            &entry.diagnostics,
            app.focus == Focus::Diagnostics,
            &mut app.diagnostic_scroll,
            &mut app.diagnostic_horizontal,
        );
    }
}

fn result_block(app: &App, compact: bool) -> Block<'static> {
    let Some(entry) = app.entries.get(app.selected) else {
        return panel(" Results ", app.focus == Focus::Result);
    };
    let failed = entry.output.is_error(app.view);
    let mut block = panel("", app.focus == Focus::Result)
        .title(Line::styled(
            format!(
                " {} {} / {} ",
                if failed { "Error" } else { "Result" },
                app.selected + 1,
                app.entries.len()
            ),
            Style::default().fg(if failed { ERROR } else { FOREGROUND }),
        ))
        .title_bottom(Line::from(format!(" {} ", source_preview(&entry.source))));
    if compact && !entry.diagnostics.is_empty() {
        block = block.title(Line::from(" F4 diagnostics ").alignment(Alignment::Right));
    }
    block
}

fn source_preview(source: &str) -> String {
    visible(
        source
            .lines()
            .find(|line| !line.trim().is_empty())
            .unwrap_or_default(),
        false,
    )
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
        .padding(Padding::horizontal(1))
        .title(title.to_owned())
}

fn button(frame: &mut Frame<'_>, label: &str, area: Rect, selected: bool) {
    frame.render_widget(
        Paragraph::new(label)
            .alignment(Alignment::Center)
            .style(
                Style::default()
                    .fg(if selected { ACCENT } else { MUTED })
                    .bg(if selected { HIGHLIGHT } else { BACKGROUND })
                    .add_modifier(if selected {
                        Modifier::BOLD
                    } else {
                        Modifier::empty()
                    }),
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
