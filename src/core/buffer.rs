use ropey::Rope;

use crate::core::Cursor;

#[derive(Debug, Default)]
pub struct Buffer {
    rope: Rope,
}

impl Buffer {
    pub fn new(text: &str) -> Self {
        Self {
            rope: Rope::from_str(text),
        }
    }

    pub fn line_count(&self) -> usize {
        self.rope.len_lines().max(1)
    }

    pub fn text(&self) -> String {
        self.rope.to_string()
    }

    pub fn line_text(&self, line: usize) -> String {
        if line >= self.line_count() {
            return String::new();
        }

        self.rope.line(line).to_string()
    }

    pub fn line_len_chars(&self, line: usize) -> usize {
        let text = self.line_text(line);
        text.chars()
            .take_while(|character| *character != '\n')
            .count()
    }

    pub fn line_len_chars_including_newline(&self, line: usize) -> usize {
        self.line_text(line).chars().count()
    }

    pub fn line_to_char(&self, line: usize) -> usize {
        let clamped = line.min(self.line_count().saturating_sub(1));
        self.rope.line_to_char(clamped)
    }

    pub fn cursor_to_char(&self, cursor: Cursor) -> usize {
        let line = cursor.line.min(self.line_count().saturating_sub(1));
        let line_start = self.line_to_char(line);
        let column = cursor.column.min(self.line_len_chars_including_newline(line));
        line_start + column
    }

    pub fn char_to_cursor(&self, char_index: usize) -> Cursor {
        let clamped = char_index.min(self.rope.len_chars());
        let line = self.rope.char_to_line(clamped);
        let line_start = self.rope.line_to_char(line);
        Cursor::new(line, clamped.saturating_sub(line_start))
    }

    pub fn insert(&mut self, cursor: Cursor, text: &str) -> Cursor {
        let char_index = self.cursor_to_char(cursor);
        self.rope.insert(char_index, text);
        self.char_to_cursor(char_index + text.chars().count())
    }

    pub fn remove(&mut self, start: Cursor, end: Cursor) {
        let start_char = self.cursor_to_char(start);
        let end_char = self.cursor_to_char(end);
        if start_char < end_char {
            self.rope.remove(start_char..end_char);
        }
    }

    pub fn slice_string(&self, start: Cursor, end: Cursor) -> String {
        let start_char = self.cursor_to_char(start);
        let end_char = self.cursor_to_char(end);
        if start_char >= end_char {
            String::new()
        } else {
            self.rope.slice(start_char..end_char).to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::{Buffer, Cursor};

    #[test]
    fn inserts_at_beginning_middle_and_end() {
        let mut buffer = Buffer::new("bc");
        let cursor = buffer.insert(Cursor::new(0, 0), "a");
        assert_eq!(cursor, Cursor::new(0, 1));
        let cursor = buffer.insert(Cursor::new(0, 2), "d");
        assert_eq!(cursor, Cursor::new(0, 3));
        let cursor = buffer.insert(Cursor::new(0, 1), "X");
        assert_eq!(cursor, Cursor::new(0, 2));
        assert_eq!(buffer.text(), "aXbcd");
    }

    #[test]
    fn removes_range() {
        let mut buffer = Buffer::new("hello");
        buffer.remove(Cursor::new(0, 1), Cursor::new(0, 4));
        assert_eq!(buffer.text(), "ho");
    }

    #[test]
    fn converts_between_line_and_char_coordinates() {
        let buffer = Buffer::new("ab\ncd");
        assert_eq!(buffer.cursor_to_char(Cursor::new(1, 1)), 4);
        assert_eq!(buffer.char_to_cursor(4), Cursor::new(1, 1));
    }
}
