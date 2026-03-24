use std::path::{Path, PathBuf};

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::core::{Buffer, Cursor};

#[derive(Debug)]
pub struct Document {
    buffer: Buffer,
    path: Option<PathBuf>,
    dirty: bool,
}

impl Document {
    pub fn new_empty(path: Option<PathBuf>) -> Self {
        Self {
            buffer: Buffer::default(),
            path,
            dirty: false,
        }
    }

    pub fn from_text(path: Option<PathBuf>, text: &str) -> Self {
        Self {
            buffer: Buffer::new(text),
            path,
            dirty: false,
        }
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn set_dirty(&mut self, dirty: bool) {
        self.dirty = dirty;
    }

    pub fn mark_saved(&mut self, path: PathBuf) {
        self.path = Some(path);
        self.dirty = false;
    }

    pub fn text(&self) -> String {
        self.buffer.text()
    }

    pub fn line_count(&self) -> usize {
        self.buffer.line_count()
    }

    pub fn last_line_index(&self) -> usize {
        self.line_count().saturating_sub(1)
    }

    pub fn line_text(&self, line: usize) -> String {
        self.buffer
            .line_text(line)
            .trim_end_matches('\n')
            .to_string()
    }

    pub fn raw_line_text(&self, line: usize) -> String {
        self.buffer.line_text(line)
    }

    pub fn insert_text(&mut self, cursor: Cursor, text: &str) -> Cursor {
        self.dirty = true;
        self.buffer.insert(cursor, text)
    }

    pub fn delete_range(&mut self, start: Cursor, end: Cursor) {
        self.dirty = true;
        self.buffer.remove(start, end);
    }

    pub fn slice_string(&self, start: Cursor, end: Cursor) -> String {
        self.buffer.slice_string(start, end)
    }

    pub fn line_start(&self, line: usize) -> Cursor {
        Cursor::new(line.min(self.last_line_index()), 0)
    }

    pub fn line_end(&self, line: usize) -> Cursor {
        let clamped = line.min(self.last_line_index());
        Cursor::new(clamped, self.buffer.line_len_chars(clamped))
    }

    pub fn line_end_including_newline(&self, line: usize) -> Cursor {
        let clamped = line.min(self.last_line_index());
        Cursor::new(clamped, self.buffer.line_len_chars_including_newline(clamped))
    }

    pub fn previous_position(&self, cursor: Cursor) -> Cursor {
        if cursor.column > 0 {
            Cursor::new(cursor.line, cursor.column - 1)
        } else if cursor.line > 0 {
            self.line_end(cursor.line - 1)
        } else {
            cursor
        }
    }

    pub fn next_position(&self, cursor: Cursor) -> Cursor {
        let line_length = self.buffer.line_len_chars_including_newline(cursor.line);
        if cursor.column < line_length {
            Cursor::new(cursor.line, cursor.column + 1)
        } else if cursor.line < self.last_line_index() {
            Cursor::new(cursor.line + 1, 0)
        } else {
            cursor
        }
    }

    pub fn move_vertically(&self, cursor: Cursor, delta: isize) -> Cursor {
        let target_line = if delta < 0 {
            cursor.line.saturating_sub(delta.unsigned_abs())
        } else {
            (cursor.line + delta as usize).min(self.last_line_index())
        };

        let target_column = self.column_for_display(target_line, cursor.preferred_column);
        Cursor::new(target_line, target_column).with_preferred_column(cursor.preferred_column)
    }

    pub fn display_column(&self, cursor: Cursor) -> usize {
        let line = self.raw_line_text(cursor.line);
        let mut display_width = 0usize;

        for (index, grapheme) in line.graphemes(true).enumerate() {
            if index >= cursor.column {
                break;
            }

            if grapheme == "\n" {
                break;
            }

            if grapheme == "\t" {
                display_width += 4;
            } else {
                display_width += grapheme.width().max(1);
            }
        }

        display_width
    }

    pub fn column_for_display(&self, line: usize, target_display: usize) -> usize {
        let raw = self.raw_line_text(line);
        let mut display = 0usize;
        let mut column = 0usize;

        for grapheme in raw.graphemes(true) {
            if grapheme == "\n" {
                break;
            }

            let width = if grapheme == "\t" {
                4
            } else {
                grapheme.width().max(1)
            };

            if display + width > target_display {
                break;
            }

            display += width;
            column += 1;
        }

        column
    }

    pub fn advance_cursor(&self, start: Cursor, text: &str) -> Cursor {
        let mut line = start.line;
        let mut column = start.column;

        for character in text.chars() {
            if character == '\n' {
                line += 1;
                column = 0;
            } else {
                column += 1;
            }
        }

        Cursor::new(line, column)
    }

    pub fn next_word_start(&self, cursor: Cursor) -> Cursor {
        let line = self.raw_line_text(cursor.line);
        let chars: Vec<char> = line.chars().collect();
        let effective_len = chars.iter().take_while(|&&c| c != '\n').count();
        let mut col = cursor.column.min(effective_len);

        if col >= effective_len {
            if cursor.line + 1 < self.line_count() {
                return Cursor::new(cursor.line + 1, 0);
            }
            return cursor;
        }

        let is_word_char = |c: char| c.is_alphanumeric() || c == '_';

        if matches!(chars[col], ' ' | '\t') {
            while col < effective_len && matches!(chars[col], ' ' | '\t') {
                col += 1;
            }
        } else {
            let word_class = is_word_char(chars[col]);
            while col < effective_len && !matches!(chars[col], ' ' | '\t') && is_word_char(chars[col]) == word_class {
                col += 1;
            }
            while col < effective_len && matches!(chars[col], ' ' | '\t') {
                col += 1;
            }
        }

        Cursor::new(cursor.line, col)
    }

    pub fn previous_word_start(&self, cursor: Cursor) -> Cursor {
        if cursor.column == 0 {
            if cursor.line == 0 {
                return cursor;
            }
            return self.line_end(cursor.line - 1);
        }

        let line = self.raw_line_text(cursor.line);
        let chars: Vec<char> = line.chars().collect();
        let mut col = cursor.column.min(chars.len());

        while col > 0 && matches!(chars[col - 1], ' ' | '\t') {
            col -= 1;
        }

        if col == 0 {
            return Cursor::new(cursor.line, 0);
        }

        let is_word_char = |c: char| c.is_alphanumeric() || c == '_';
        let word_class = is_word_char(chars[col - 1]);

        while col > 0 && is_word_char(chars[col - 1]) == word_class {
            col -= 1;
        }

        Cursor::new(cursor.line, col)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::core::{Cursor, Document};

    #[test]
    fn dirty_state_changes_after_edits_and_save() {
        let mut document = Document::new_empty(None);
        assert!(!document.is_dirty());
        let cursor = document.insert_text(Cursor::new(0, 0), "hello");
        assert_eq!(cursor, Cursor::new(0, 5));
        assert!(document.is_dirty());
        document.mark_saved(PathBuf::from("file.txt"));
        assert!(!document.is_dirty());
    }
}
