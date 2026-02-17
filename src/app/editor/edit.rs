use crate::app::{editor::TAB_WIDTH, App};

use super::{byte_index_for_char, leading_indent_width, line_len_chars};

impl App {
    pub(crate) fn insert_char(&mut self, ch: char) {
        self.begin_edit();
        self.delete_selection_inner();

        let line = &mut self.lines[self.cursor_line];
        let byte_index = byte_index_for_char(line, self.cursor_col);
        line.insert(byte_index, ch);
        self.cursor_col += 1;
        self.preferred_col = self.cursor_col;
        self.selection_anchor = None;
        self.mark_changed();
    }

    pub(crate) fn insert_newline(&mut self) {
        self.begin_edit();
        self.delete_selection_inner();

        let current = self.lines[self.cursor_line].clone();
        let split = byte_index_for_char(&current, self.cursor_col);
        let (left, right) = current.split_at(split);
        self.lines[self.cursor_line] = left.to_string();
        self.lines.insert(self.cursor_line + 1, right.to_string());
        self.cursor_line += 1;
        self.cursor_col = 0;
        self.preferred_col = self.cursor_col;
        self.selection_anchor = None;
        self.mark_changed();
    }

    pub(crate) fn backspace(&mut self) {
        if self.has_selection() {
            self.begin_edit();
            self.delete_selection_inner();
            self.mark_changed();
            return;
        }

        if self.cursor_col > 0 {
            self.begin_edit();
            let line = &mut self.lines[self.cursor_line];
            let remove_at = byte_index_for_char(line, self.cursor_col - 1);
            line.remove(remove_at);
            self.cursor_col -= 1;
            self.preferred_col = self.cursor_col;
            self.selection_anchor = None;
            self.mark_changed();
            return;
        }

        if self.cursor_line > 0 {
            self.begin_edit();
            let current = self.lines.remove(self.cursor_line);
            self.cursor_line -= 1;
            let prev_len = line_len_chars(&self.lines[self.cursor_line]);
            self.lines[self.cursor_line].push_str(&current);
            self.cursor_col = prev_len;
            self.preferred_col = self.cursor_col;
            self.selection_anchor = None;
            self.mark_changed();
        }
    }

    pub(crate) fn delete(&mut self) {
        if self.has_selection() {
            self.begin_edit();
            self.delete_selection_inner();
            self.mark_changed();
            return;
        }

        let line_len = line_len_chars(&self.lines[self.cursor_line]);
        if self.cursor_col < line_len {
            self.begin_edit();
            let line = &mut self.lines[self.cursor_line];
            let remove_at = byte_index_for_char(line, self.cursor_col);
            line.remove(remove_at);
            self.preferred_col = self.cursor_col;
            self.selection_anchor = None;
            self.mark_changed();
            return;
        }

        if self.cursor_line + 1 < self.lines.len() {
            self.begin_edit();
            let next = self.lines.remove(self.cursor_line + 1);
            self.lines[self.cursor_line].push_str(&next);
            self.preferred_col = self.cursor_col;
            self.selection_anchor = None;
            self.mark_changed();
        }
    }

    pub(crate) fn indent_selection_or_insert_tab(&mut self) {
        if let Some((start, end)) = self.selection_line_bounds() {
            self.begin_edit();
            let pad = " ".repeat(TAB_WIDTH);
            for line in start..=end {
                self.lines[line].insert_str(0, &pad);
            }

            self.cursor_col += TAB_WIDTH;
            if let Some(anchor) = self.selection_anchor.as_mut() {
                anchor.col += TAB_WIDTH;
            }
            self.preferred_col = self.cursor_col;
            self.mark_changed();
            return;
        }

        self.begin_edit();
        let pad = " ".repeat(TAB_WIDTH);
        let line = &mut self.lines[self.cursor_line];
        let byte_index = byte_index_for_char(line, self.cursor_col);
        line.insert_str(byte_index, &pad);
        self.cursor_col += TAB_WIDTH;
        self.preferred_col = self.cursor_col;
        self.selection_anchor = None;
        self.mark_changed();
    }

    pub(crate) fn outdent_selection(&mut self) {
        let Some((start, end)) = self.selection_line_bounds() else {
            return;
        };

        self.begin_edit();
        let mut removed = Vec::with_capacity(end - start + 1);
        for line in start..=end {
            let count = leading_indent_width(&self.lines[line]);
            if count > 0 {
                let byte_end = byte_index_for_char(&self.lines[line], count);
                self.lines[line].replace_range(0..byte_end, "");
            }
            removed.push(count);
        }

        if self.cursor_line >= start && self.cursor_line <= end {
            let amount = removed[self.cursor_line - start];
            self.cursor_col = self.cursor_col.saturating_sub(amount);
        }
        if let Some(anchor) = self.selection_anchor.as_mut() {
            if anchor.line >= start && anchor.line <= end {
                let amount = removed[anchor.line - start];
                anchor.col = anchor.col.saturating_sub(amount);
            }
        }

        self.preferred_col = self.cursor_col;
        self.mark_changed();
    }

    pub(crate) fn duplicate_line_or_selection(&mut self) {
        self.begin_edit();
        if let Some((start, end)) = self.selection_line_bounds() {
            let copied: Vec<String> = self.lines[start..=end].to_vec();
            let insert_at = end + 1;
            for (idx, line) in copied.iter().enumerate() {
                self.lines.insert(insert_at + idx, line.clone());
            }

            let shift = copied.len();
            self.cursor_line += shift;
            if let Some(anchor) = self.selection_anchor.as_mut() {
                anchor.line += shift;
            }
            self.cursor_col = self
                .cursor_col
                .min(line_len_chars(&self.lines[self.cursor_line]));
            self.preferred_col = self.cursor_col;
            self.mark_changed();
            return;
        }

        let cloned = self.lines[self.cursor_line].clone();
        self.lines.insert(self.cursor_line + 1, cloned);
        self.cursor_line += 1;
        self.cursor_col = self
            .cursor_col
            .min(line_len_chars(&self.lines[self.cursor_line]));
        self.preferred_col = self.cursor_col;
        self.selection_anchor = None;
        self.mark_changed();
    }

    pub(crate) fn delete_line_or_selection(&mut self) {
        self.begin_edit();
        let (start, end) = self
            .selection_line_bounds()
            .unwrap_or((self.cursor_line, self.cursor_line));

        self.lines.drain(start..=end);
        if self.lines.is_empty() {
            self.lines.push(String::new());
        }

        self.cursor_line = start.min(self.lines.len() - 1);
        self.cursor_col = self
            .cursor_col
            .min(line_len_chars(&self.lines[self.cursor_line]));
        self.preferred_col = self.cursor_col;
        self.selection_anchor = None;
        self.mark_changed();
    }
}
