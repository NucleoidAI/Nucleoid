use tui_textarea::{CursorMove, TextArea};

use crate::INDENT;

#[derive(Clone, Copy)]
enum Direction {
    Indent,
    Unindent,
}

struct Block {
    first: usize,
    last: usize,
    cursor: (usize, usize),
    anchor: Option<(usize, usize)>,
}

impl Block {
    fn capture(editor: &TextArea<'_>) -> Self {
        let cursor = editor.cursor();
        let Some((start, end)) = editor.selection_range().filter(|(start, end)| start != end)
        else {
            return Self {
                first: cursor.0,
                last: cursor.0,
                cursor,
                anchor: None,
            };
        };
        let last = if end.0 > start.0 && end.1 == 0 {
            end.0 - 1
        } else {
            end.0
        };
        Self {
            first: start.0,
            last,
            cursor,
            anchor: Some(if cursor == start { end } else { start }),
        }
    }

    fn shift(&mut self, row: usize, count: usize, direction: Direction) {
        for position in std::iter::once(&mut self.cursor).chain(self.anchor.iter_mut()) {
            if position.0 == row {
                position.1 = match direction {
                    Direction::Indent => position.1.saturating_add(count),
                    Direction::Unindent => position.1.saturating_sub(count),
                };
            }
        }
    }

    fn restore(&self, editor: &mut TextArea<'_>) {
        if let Some(anchor) = self.anchor {
            move_to(editor, anchor);
            editor.start_selection();
        }
        move_to(editor, self.cursor);
    }
}

pub(super) fn indent_selection(editor: &mut TextArea<'_>) -> bool {
    let block = Block::capture(editor);
    if block.anchor.is_none() {
        return false;
    }
    change(editor, block, Direction::Indent)
}

pub(super) fn unindent(editor: &mut TextArea<'_>) -> bool {
    let block = Block::capture(editor);
    change(editor, block, Direction::Unindent)
}

fn change(editor: &mut TextArea<'_>, mut block: Block, direction: Direction) -> bool {
    editor.cancel_selection();
    let mut changed = false;
    for row in block.first..=block.last {
        let count = match direction {
            Direction::Indent => INDENT.len(),
            Direction::Unindent => editor
                .lines()
                .get(row)
                .map_or(0, |line| leading_indent(line)),
        };
        if count == 0 {
            continue;
        }
        move_to(editor, (row, 0));
        let edited = match direction {
            Direction::Indent => editor.insert_str(INDENT),
            Direction::Unindent => remove_indent(editor, count),
        };
        if edited {
            block.shift(row, count, direction);
            changed = true;
        }
    }
    block.restore(editor);
    changed
}

fn remove_indent(editor: &mut TextArea<'_>, count: usize) -> bool {
    editor.start_selection();
    for _ in 0..count {
        editor.move_cursor(CursorMove::Forward);
    }
    editor.delete_char()
}

fn leading_indent(line: &str) -> usize {
    let mut width = 0;
    let mut count = 0;
    for character in line.chars() {
        width += match character {
            ' ' => 1,
            '\t' => INDENT.len() - width % INDENT.len(),
            _ => break,
        };
        count += 1;
        if width >= INDENT.len() {
            break;
        }
    }
    count
}

fn move_to(editor: &mut TextArea<'_>, (row, column): (usize, usize)) {
    if let (Ok(row), Ok(column)) = (u16::try_from(row), u16::try_from(column)) {
        editor.move_cursor(CursorMove::Jump(row, column));
        return;
    }
    let start = u16::try_from(row).unwrap_or(u16::MAX);
    editor.move_cursor(CursorMove::Jump(start, 0));
    for _ in usize::from(start)..row {
        editor.move_cursor(CursorMove::Down);
    }
    if column == 0 {
        return;
    }
    editor.move_cursor(CursorMove::End);
    let end = editor.cursor().1;
    for _ in column.min(end)..end {
        editor.move_cursor(CursorMove::Back);
    }
}

#[cfg(test)]
mod tests;
