use super::*;

#[test]
fn selected_lines_keep_unicode_boundaries_direction_and_cut_text() {
    let source = ["a\u{754c}", "", " \tsecond", "last"];
    let expected = ["    a\u{754c}", "    ", "     \tsecond", "last"];
    for (end, shifted_end) in [((3, 0), (3, 0)), ((2, 3), (2, 7))] {
        for reversed in [false, true] {
            let mut editor = TextArea::from(source);
            let (anchor, cursor) = if reversed {
                (end, (0, 1))
            } else {
                ((0, 1), end)
            };
            move_to(&mut editor, anchor);
            editor.start_selection();
            move_to(&mut editor, cursor);
            editor.set_yank_text("previous\ncut\n");

            assert!(indent_selection(&mut editor));
            assert_eq!(editor.lines(), expected);
            assert_eq!(editor.selection_range(), Some(((0, 5), shifted_end)));
            assert_eq!(editor.cursor(), if reversed { (0, 5) } else { shifted_end });
            assert_eq!(editor.yank_text(), "previous\ncut\n");

            assert!(unindent(&mut editor));
            assert_eq!(editor.lines(), source);
            assert_eq!(editor.selection_range(), Some(((0, 1), end)));
            assert_eq!(editor.cursor(), cursor);
            assert_eq!(editor.yank_text(), "previous\ncut\n");
        }
    }
}

#[test]
fn unindent_removes_one_leading_level_without_deleting_code() {
    for (source, column, expected, shifted_column) in [
        ("    value", 7, "value", 3),
        ("\tvalue", 3, "value", 2),
        (" \tvalue", 4, "value", 2),
        ("  value", 0, "value", 0),
        (" \t   value", 7, "   value", 5),
        ("value", 3, "value", 3),
    ] {
        let mut editor = TextArea::from([source]);
        move_to(&mut editor, (0, column));
        editor.set_yank_text("previous cut");

        assert_eq!(unindent(&mut editor), source != expected);
        assert_eq!(editor.lines(), [expected]);
        assert_eq!(editor.cursor(), (0, shifted_column));
        assert!(!editor.is_selecting());
        assert_eq!(editor.yank_text(), "previous cut");
    }
}

#[test]
fn unindent_preserves_positions_beyond_cursor_jump_limits() {
    let long_line = format!("    {}", "x".repeat(usize::from(u16::MAX) + 1));
    let mut many_lines = vec![String::new(); usize::from(u16::MAX) + 1];
    many_lines.push("    value".into());
    for mut editor in [TextArea::from([long_line]), TextArea::new(many_lines)] {
        editor.move_cursor(CursorMove::Bottom);
        editor.move_cursor(CursorMove::End);
        let (row, column) = editor.cursor();
        let original = editor.lines().to_vec();

        assert!(unindent(&mut editor));
        assert_eq!(editor.cursor(), (row, column - INDENT.len()));
        assert_eq!(editor.lines().len(), original.len());
        assert_eq!(editor.lines()[row], original[row].trim_start());
        assert!(editor.undo());
        assert_eq!(editor.lines(), original);
    }
}
