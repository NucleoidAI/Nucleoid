use crossterm::event::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Position;

use super::output::View;
use super::{App, Focus, Mode, editor};

fn scrolled(offset: u16, forward: bool) -> u16 {
    if forward {
        offset.saturating_add(3)
    } else {
        offset.saturating_sub(3)
    }
}

impl App {
    pub(super) fn page_scroll(&mut self, forward: bool) {
        let height = if self.focus == Focus::Diagnostics {
            self.regions.diagnostics.height
        } else {
            let table_header = self.mode == Mode::Editor
                && self.view == View::Table
                && self
                    .entries
                    .get(self.selected)
                    .is_some_and(|entry| entry.output.table().is_some());
            self.regions
                .output
                .height
                .saturating_sub(u16::from(table_header))
        };
        self.scroll_vertical(forward, height.max(1));
    }

    pub(super) fn scroll_vertical(&mut self, forward: bool, amount: u16) {
        if self.mode == Mode::Chat {
            self.transcript.scroll(forward, amount);
            return;
        }
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

    pub(super) fn scroll_horizontal(&mut self, forward: bool, diagnostics: bool) {
        let scroll = if diagnostics {
            &mut self.diagnostic_horizontal
        } else if self.mode == Mode::Chat {
            &mut self.transcript.horizontal
        } else {
            &mut self.horizontal
        };
        *scroll = if forward {
            scroll.saturating_add(1)
        } else {
            scroll.saturating_sub(1)
        };
    }

    pub(super) fn mouse(&mut self, mouse: MouseEvent) {
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
        if self.completion_at(position) {
            self.click_completion(position);
        } else if let Some(mode) = [Mode::Editor, Mode::Chat]
            .into_iter()
            .zip(self.regions.tabs)
            .find_map(|(mode, area)| area.contains(position).then_some(mode))
        {
            self.set_mode(mode);
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

    fn completion_at(&self, position: Position) -> bool {
        self.completion.is_some() && self.regions.completion_popup.contains(position)
    }

    fn click_completion(&mut self, position: Position) {
        if !self.regions.completions.contains(position) {
            return;
        }
        if let Some(mut completion) = self.completion.take() {
            let index = self.regions.completion_start
                + usize::from(position.y - self.regions.completions.y);
            if index < completion.candidates.len() {
                completion.selected = index;
                completion.apply(&mut self.editor);
                self.history_index = None;
            }
        }
    }

    fn wheel(&mut self, mouse: MouseEvent, position: Position, forward: bool) {
        let sideways = mouse.modifiers.contains(KeyModifiers::SHIFT);
        if self.completion_at(position) {
            if let Some(completion) = &mut self.completion {
                completion.select(forward);
            }
        } else if self.regions.editor.contains(position) {
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
            } else if self.mode == Mode::Chat {
                self.transcript.scroll(forward, 3);
            } else {
                self.scroll = scrolled(self.scroll, forward);
            }
        }
    }

    fn wheel_sideways(&mut self, position: Position, forward: bool) {
        if self.completion_at(position) {
            return;
        }
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
