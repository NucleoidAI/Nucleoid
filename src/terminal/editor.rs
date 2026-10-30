use ratatui::layout::Rect;
use tui_textarea::{CursorMove, TextArea};
use unicode_width::UnicodeWidthChar;

use crate::{INDENT, indented_body_header, next_indentation};

pub(super) fn new(source: &str) -> TextArea<'static> {
    let mut editor = TextArea::from(source.split('\n'));
    editor.set_tab_length(INDENT.len() as u8);
    editor.set_placeholder_text("Write Nucleoid code or .help");
    editor.move_cursor(CursorMove::Bottom);
    editor.move_cursor(CursorMove::End);
    editor
}

pub(super) fn newline(editor: &mut TextArea<'_>) {
    let (row, column) = editor.cursor();
    let prefix: String = editor.lines()[row].chars().take(column).collect();
    let indentation = next_indentation(&prefix, indented_body_header(&prefix));
    editor.insert_newline();
    editor.insert_str(indentation);
}

pub(super) fn dedent(editor: &mut TextArea<'_>) -> bool {
    let (row, column) = editor.cursor();
    if editor.is_selecting()
        || column == 0
        || !editor.lines()[row]
            .chars()
            .take(column)
            .all(|character| character == ' ')
    {
        return false;
    }
    let count = (column - 1) % INDENT.len() + 1;
    for _ in 0..count {
        editor.delete_char();
    }
    true
}

pub(super) fn click(editor: &mut TextArea<'_>, area: Rect, x: u16, y: u16, drag: bool) {
    if !drag {
        editor.cancel_selection();
    }
    // InViewport clamps columns to the probed line's length. Read the horizontal
    // origin on the cursor's rendered line before probing a potentially short
    // top line.
    let (row, _) = editor.cursor();
    editor.move_cursor(CursorMove::Jump(u16::try_from(row).unwrap_or(u16::MAX), 0));
    editor.move_cursor(CursorMove::InViewport);
    let (_, left) = editor.cursor();
    editor.move_cursor(CursorMove::Jump(0, 0));
    editor.move_cursor(CursorMove::InViewport);
    let (top, _) = editor.cursor();
    let row = (top + usize::from(y.saturating_sub(area.y))).min(editor.lines().len() - 1);
    let target = left + usize::from(x.saturating_sub(area.x));
    let tab_width = usize::from(editor.tab_length());
    let mut width = 0;
    let mut column = 0;
    for character in editor.lines()[row].chars() {
        let character_width = if character == '\t' && tab_width > 0 {
            tab_width - width % tab_width
        } else {
            character.width().unwrap_or_default()
        };
        let next = width + character_width;
        if next > target {
            break;
        }
        width = next;
        column += 1;
    }
    editor.move_cursor(CursorMove::Jump(
        u16::try_from(row).unwrap_or(u16::MAX),
        u16::try_from(column).unwrap_or(u16::MAX),
    ));
    if !drag {
        editor.start_selection();
    }
}

pub(super) fn paste(editor: &mut TextArea<'_>, source: &str) {
    let source = source.replace("\r\n", "\n").replace('\r', "\n");
    let source: String = source
        .chars()
        .filter(|character| !character.is_control() || matches!(character, '\n' | '\t'))
        .collect();
    editor.insert_str(source);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{Terminal, backend::TestBackend};

    #[test]
    fn newlines_reuse_lexical_indentation_and_backspace_dedents() {
        let mut editor = new("try # body");
        newline(&mut editor);
        assert_eq!(editor.lines(), ["try # body", "    "]);
        editor.insert_str("if ready");
        newline(&mut editor);
        assert_eq!(editor.lines()[2], "        ");
        assert!(dedent(&mut editor));
        assert_eq!(editor.lines()[2], "    ");
        assert!(dedent(&mut editor));
        assert_eq!(editor.lines()[2], "");
        assert!(!dedent(&mut editor));
    }

    #[test]
    fn mouse_positioning_tracks_scrolled_lines_and_wide_characters() {
        let mut editor = new("first\nsecond\nthird\nlast");
        let mut terminal = Terminal::new(TestBackend::new(10, 2)).unwrap();
        terminal
            .draw(|frame| frame.render_widget(&editor, frame.area()))
            .unwrap();
        click(&mut editor, Rect::new(0, 0, 10, 2), 2, 0, false);
        assert_eq!(editor.cursor(), (2, 2));
        let mut editor = new("a界b");
        terminal
            .draw(|frame| frame.render_widget(&editor, frame.area()))
            .unwrap();
        click(&mut editor, Rect::new(0, 0, 10, 2), 3, 0, false);
        assert_eq!(editor.cursor(), (0, 2));
    }

    #[test]
    fn mouse_positioning_uses_display_cells_with_horizontal_scroll_and_tabs() {
        for (source, x, expected_column) in [
            ("short\n0123456789abcdefghij", 2, 13),
            ("abcdefghij界klmnopqrst", 0, 11),
            ("\tvalue", 4, 1),
            ("a\tvalue", 4, 2),
        ] {
            let mut editor = new(source);
            let row = editor.cursor().0;
            let mut terminal = Terminal::new(TestBackend::new(10, 2)).unwrap();
            terminal
                .draw(|frame| frame.render_widget(&editor, frame.area()))
                .unwrap();
            click(
                &mut editor,
                Rect::new(0, 0, 10, 2),
                x,
                u16::try_from(row).unwrap(),
                false,
            );
            assert_eq!(editor.cursor(), (row, expected_column), "{source:?}");
            click(
                &mut editor,
                Rect::new(0, 0, 10, 2),
                x + 1,
                u16::try_from(row).unwrap(),
                true,
            );
            assert_eq!(
                editor.selection_range(),
                Some(((row, expected_column), (row, expected_column + 1))),
                "{source:?}"
            );
        }
    }

    #[test]
    fn bracketed_paste_is_multiline_without_executing_or_injecting_controls() {
        let mut editor = new("");
        paste(&mut editor, "a = 1\r\nb = a + 1\r\u{1b}[2J");
        assert_eq!(editor.lines(), ["a = 1", "b = a + 1", "[2J"]);
    }
}
