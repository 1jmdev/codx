use std::{fs, io, path::PathBuf};

use super::{App, Focus};

impl App {
    pub(crate) fn open_file(&mut self, path: PathBuf) -> io::Result<()> {
        let content = fs::read_to_string(&path)?;
        let mut lines: Vec<String> = content.split('\n').map(|line| line.to_string()).collect();
        if lines.is_empty() {
            lines.push(String::new());
        }

        self.current_file = Some(path.clone());
        self.lines = lines;
        self.cursor_line = 0;
        self.cursor_col = 0;
        self.preferred_col = 0;
        self.editor_scroll = 0;
        self.dirty = false;
        self.status = format!("Opened {}", path.display());
        self.focus = Focus::Editor;
        self.lsp
            .open_file(&path, self.lines.join("\n"), &mut self.status);
        Ok(())
    }

    pub(crate) fn save_file(&mut self) -> io::Result<()> {
        let Some(path) = self.current_file.clone() else {
            self.status = String::from("No file selected. Open a file from the tree first.");
            return Ok(());
        };

        fs::write(&path, self.lines.join("\n"))?;
        self.dirty = false;
        self.notify_lsp_save();
        self.status = format!("Saved {}", path.display());
        Ok(())
    }

    pub(crate) fn insert_char(&mut self, ch: char) {
        let line = &mut self.lines[self.cursor_line];
        let byte_index = byte_index_for_char(line, self.cursor_col);
        line.insert(byte_index, ch);
        self.cursor_col += 1;
        self.preferred_col = self.cursor_col;
        self.dirty = true;
        self.notify_lsp_change();
    }

    pub(crate) fn insert_newline(&mut self) {
        let current = self.lines[self.cursor_line].clone();
        let split = byte_index_for_char(&current, self.cursor_col);
        let (left, right) = current.split_at(split);
        self.lines[self.cursor_line] = left.to_string();
        self.lines.insert(self.cursor_line + 1, right.to_string());
        self.cursor_line += 1;
        self.cursor_col = 0;
        self.preferred_col = self.cursor_col;
        self.dirty = true;
        self.notify_lsp_change();
    }

    pub(crate) fn backspace(&mut self) {
        if self.cursor_col > 0 {
            let line = &mut self.lines[self.cursor_line];
            let remove_at = byte_index_for_char(line, self.cursor_col - 1);
            line.remove(remove_at);
            self.cursor_col -= 1;
            self.preferred_col = self.cursor_col;
            self.dirty = true;
            self.notify_lsp_change();
            return;
        }

        if self.cursor_line > 0 {
            let current = self.lines.remove(self.cursor_line);
            self.cursor_line -= 1;
            let prev_len = line_len_chars(&self.lines[self.cursor_line]);
            self.lines[self.cursor_line].push_str(&current);
            self.cursor_col = prev_len;
            self.preferred_col = self.cursor_col;
            self.dirty = true;
            self.notify_lsp_change();
        }
    }

    pub(crate) fn delete(&mut self) {
        let line_len = line_len_chars(&self.lines[self.cursor_line]);
        if self.cursor_col < line_len {
            let line = &mut self.lines[self.cursor_line];
            let remove_at = byte_index_for_char(line, self.cursor_col);
            line.remove(remove_at);
            self.preferred_col = self.cursor_col;
            self.dirty = true;
            self.notify_lsp_change();
            return;
        }

        if self.cursor_line + 1 < self.lines.len() {
            let next = self.lines.remove(self.cursor_line + 1);
            self.lines[self.cursor_line].push_str(&next);
            self.preferred_col = self.cursor_col;
            self.dirty = true;
            self.notify_lsp_change();
        }
    }

    pub(crate) fn line_number_width(&self) -> usize {
        self.lines.len().max(1).to_string().len()
    }

    pub(crate) fn ensure_cursor_visible(&mut self, view_height: usize) {
        if view_height == 0 {
            return;
        }

        let top_trigger_row = view_height / 4;
        let bottom_trigger_row = (view_height * 3 / 4).min(view_height.saturating_sub(1));

        let top_trigger_line = self.editor_scroll + top_trigger_row;
        if self.cursor_line < top_trigger_line {
            self.editor_scroll = self.cursor_line.saturating_sub(top_trigger_row);
        }

        let bottom_trigger_line = self.editor_scroll + bottom_trigger_row;
        if self.cursor_line > bottom_trigger_line {
            self.editor_scroll = self.cursor_line.saturating_sub(bottom_trigger_row);
        }

        let max_scroll = self.lines.len().saturating_sub(view_height);
        self.editor_scroll = self.editor_scroll.min(max_scroll);
    }
}

pub(crate) fn line_len_chars(line: &str) -> usize {
    line.chars().count()
}

pub(crate) fn byte_index_for_char(line: &str, char_index: usize) -> usize {
    line.char_indices()
        .nth(char_index)
        .map(|(index, _)| index)
        .unwrap_or(line.len())
}
