use ratatui::Frame;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Clear, List, ListItem, Paragraph, Row, Table};
use unicode_width::UnicodeWidthStr;

use super::{ACCENT, BACKGROUND, ERROR, FOREGROUND, MUTED, PANEL, Regions, as_u16, base, panel};
use crate::terminal::App;
use crate::terminal::output::{Output, TableData, View, visible};

pub(super) fn draw_text(
    frame: &mut Frame<'_>,
    area: Rect,
    output: &Output,
    view: View,
    scroll: &mut u16,
    horizontal: &mut u16,
) {
    let text = output.text(view);
    clamp_scroll(text, area, scroll, horizontal);
    frame.render_widget(
        Paragraph::new(text)
            .style(
                Style::default()
                    .fg(if output.is_error(view) {
                        ERROR
                    } else {
                        FOREGROUND
                    })
                    .bg(PANEL),
            )
            .scroll((*scroll, *horizontal)),
        area,
    );
}

pub(super) fn draw_diagnostics(
    frame: &mut Frame<'_>,
    area: Rect,
    diagnostics: &str,
    focused: bool,
    scroll: &mut u16,
    horizontal: &mut u16,
) -> Rect {
    let block = panel(" Diagnostics / graph ", focused);
    let inner = block.inner(area);
    clamp_scroll(diagnostics, inner, scroll, horizontal);
    frame.render_widget(
        Paragraph::new(diagnostics)
            .block(block)
            .style(Style::default().fg(MUTED).bg(PANEL))
            .scroll((*scroll, *horizontal)),
        area,
    );
    inner
}

pub(super) fn draw_table(
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
            .map(|row| row.get(index).map_or(0, |cell| cell.width()))
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

pub(super) fn completions(
    frame: &mut Frame<'_>,
    editor_area: Rect,
    app: &App,
    regions: &mut Regions,
) {
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
