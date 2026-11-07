use crossterm::event::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Position;

use super::output::View;
use super::{App, Focus, editor};

fn scrolled(offset: u16, forward: bool) -> u16 {
    if forward {
        offset.saturating_add(3)
    } else {
        offset.saturating_sub(3)
    }
}

impl App {
    pub(super) fn scroll_vertical(&mut self, forward: bool, amount: u16) {
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
