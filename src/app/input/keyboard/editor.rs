use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::editor::line_len_chars;
use crate::app::App;

impl App {
    pub(super) fn handle_editor_key(&mut self, key: KeyEvent) {
        let selecting = key.modifiers.contains(KeyModifiers::SHIFT);
        match key.code {
            KeyCode::Up => self.move_cursor_up(selecting),
            KeyCode::Down => self.move_cursor_down(selecting),
            KeyCode::Left => {
                if selecting {
                    self.move_cursor_word_left(true);
                } else {
                    self.move_cursor_left(false);
                }
            }
            KeyCode::Right => {
                if selecting {
                    self.move_cursor_word_right(true);
                } else {
                    self.move_cursor_right(false);
                }
            }
            KeyCode::Home => {
                if selecting {
                    self.set_selection_anchor_if_missing();
                } else {
                    self.clear_selection();
                }
                self.cursor_col = 0;
                self.preferred_col = self.cursor_col;
            }
            KeyCode::End => {
                if selecting {
                    self.set_selection_anchor_if_missing();
                } else {
                    self.clear_selection();
                }
                self.cursor_col = line_len_chars(&self.lines[self.cursor_line]);
                self.preferred_col = self.cursor_col;
            }
            KeyCode::Tab => self.indent_selection_or_insert_tab(),
            KeyCode::BackTab => self.outdent_selection(),
            KeyCode::Backspace => self.backspace(),
            KeyCode::Delete => self.delete(),
            KeyCode::Enter => self.insert_newline(),
            KeyCode::Char(ch) => {
                if key.modifiers.is_empty() || key.modifiers == KeyModifiers::SHIFT {
                    self.insert_char(ch);
                }
            }
            _ => {}
        }
    }

    fn move_cursor_up(&mut self, selecting: bool) {
        if selecting {
            self.set_selection_anchor_if_missing();
        } else {
            self.clear_selection();
        }

        if self.cursor_line > 0 {
            self.cursor_line -= 1;
            let max = line_len_chars(&self.lines[self.cursor_line]);
            self.cursor_col = self.preferred_col.min(max);
        }
    }

    fn move_cursor_down(&mut self, selecting: bool) {
        if selecting {
            self.set_selection_anchor_if_missing();
        } else {
            self.clear_selection();
        }

        if self.cursor_line + 1 < self.lines.len() {
            self.cursor_line += 1;
            let max = line_len_chars(&self.lines[self.cursor_line]);
            self.cursor_col = self.preferred_col.min(max);
        }
    }

    fn move_cursor_left(&mut self, selecting: bool) {
        if selecting {
            self.set_selection_anchor_if_missing();
        } else {
            self.clear_selection();
        }

        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        } else if self.cursor_line > 0 {
            self.cursor_line -= 1;
            self.cursor_col = line_len_chars(&self.lines[self.cursor_line]);
        }

        self.preferred_col = self.cursor_col;
    }

    fn move_cursor_right(&mut self, selecting: bool) {
        if selecting {
            self.set_selection_anchor_if_missing();
        } else {
            self.clear_selection();
        }

        let len = line_len_chars(&self.lines[self.cursor_line]);
        if self.cursor_col < len {
            self.cursor_col += 1;
        } else if self.cursor_line + 1 < self.lines.len() {
            self.cursor_line += 1;
            self.cursor_col = 0;
        }

        self.preferred_col = self.cursor_col;
    }

    fn move_cursor_word_left(&mut self, selecting: bool) {
        if selecting {
            self.set_selection_anchor_if_missing();
        } else {
            self.clear_selection();
        }

        if self.cursor_col == 0 {
            if self.cursor_line > 0 {
                self.cursor_line -= 1;
                self.cursor_col = line_len_chars(&self.lines[self.cursor_line]);
            }
        } else {
            let line = &self.lines[self.cursor_line];
            self.cursor_col = previous_word_boundary(line, self.cursor_col);
        }

        self.preferred_col = self.cursor_col;
    }

    fn move_cursor_word_right(&mut self, selecting: bool) {
        if selecting {
            self.set_selection_anchor_if_missing();
        } else {
            self.clear_selection();
        }

        let len = line_len_chars(&self.lines[self.cursor_line]);
        if self.cursor_col >= len {
            if self.cursor_line + 1 < self.lines.len() {
                self.cursor_line += 1;
                self.cursor_col = 0;
            }
        } else {
            let line = &self.lines[self.cursor_line];
            self.cursor_col = next_word_boundary(line, self.cursor_col);
        }

        self.preferred_col = self.cursor_col;
    }
}

fn is_word_char(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_'
}

fn previous_word_boundary(line: &str, col: usize) -> usize {
    let chars: Vec<char> = line.chars().collect();
    let mut idx = col.min(chars.len());
    while idx > 0 && chars[idx - 1].is_whitespace() {
        idx -= 1;
    }
    if idx == 0 {
        return 0;
    }

    let mode = is_word_char(chars[idx - 1]);
    while idx > 0 {
        let ch = chars[idx - 1];
        if ch.is_whitespace() || is_word_char(ch) != mode {
            break;
        }
        idx -= 1;
    }
    idx
}

fn next_word_boundary(line: &str, col: usize) -> usize {
    let chars: Vec<char> = line.chars().collect();
    let mut idx = col.min(chars.len());
    while idx < chars.len() && chars[idx].is_whitespace() {
        idx += 1;
    }
    if idx >= chars.len() {
        return chars.len();
    }

    let mode = is_word_char(chars[idx]);
    while idx < chars.len() {
        let ch = chars[idx];
        if ch.is_whitespace() || is_word_char(ch) != mode {
            break;
        }
        idx += 1;
    }
    idx
}
