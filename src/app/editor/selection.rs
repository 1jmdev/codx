use crate::app::{App, state::CursorPos};

use super::{byte_index_for_char, line_len_chars};

impl App {
    pub(crate) fn clear_selection(&mut self) {
        self.selection_anchor = None;
    }

    pub(crate) fn has_selection(&self) -> bool {
        self.selection_bounds().is_some()
    }

    pub(crate) fn set_selection_anchor_if_missing(&mut self) {
        if self.selection_anchor.is_none() {
            self.selection_anchor = Some(self.cursor_pos());
        }
    }

    pub(crate) fn selection_bounds(&self) -> Option<(CursorPos, CursorPos)> {
        let anchor = self.selection_anchor?;
        let cursor = self.cursor_pos();
        if anchor == cursor {
            return None;
        }

        if (anchor.line, anchor.col) <= (cursor.line, cursor.col) {
            Some((anchor, cursor))
        } else {
            Some((cursor, anchor))
        }
    }

    pub(crate) fn selection_cols_for_line(&self, line_idx: usize) -> Option<(usize, usize)> {
        let (start, end) = self.selection_bounds()?;
        if line_idx < start.line || line_idx > end.line {
            return None;
        }

        if start.line == end.line {
            return Some((start.col, end.col));
        }

        if line_idx == start.line {
            return Some((start.col, line_len_chars(&self.lines[line_idx])));
        }

        if line_idx == end.line {
            return Some((0, end.col));
        }

        Some((0, line_len_chars(&self.lines[line_idx])))
    }

    pub(crate) fn selection_line_bounds(&self) -> Option<(usize, usize)> {
        let (start, end) = self.selection_bounds()?;
        Some((start.line, end.line))
    }

    pub(crate) fn cursor_pos(&self) -> CursorPos {
        CursorPos {
            line: self.cursor_line,
            col: self.cursor_col,
        }
    }

    pub(crate) fn delete_selection_inner(&mut self) -> bool {
        let Some((start, end)) = self.selection_bounds() else {
            return false;
        };

        if start.line == end.line {
            let line = &mut self.lines[start.line];
            let start_byte = byte_index_for_char(line, start.col);
            let end_byte = byte_index_for_char(line, end.col);
            line.replace_range(start_byte..end_byte, "");
        } else {
            let first = self.lines[start.line].clone();
            let last = self.lines[end.line].clone();
            let first_prefix = &first[..byte_index_for_char(&first, start.col)];
            let last_suffix = &last[byte_index_for_char(&last, end.col)..];

            self.lines[start.line] = format!("{}{}", first_prefix, last_suffix);
            self.lines.drain(start.line + 1..=end.line);
        }

        self.cursor_line = start.line;
        self.cursor_col = start.col;
        self.preferred_col = self.cursor_col;
        self.selection_anchor = None;
        true
    }
}
