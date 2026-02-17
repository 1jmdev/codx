use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::editor::line_len_chars;
use crate::app::{App, Focus};

impl App {
    pub(crate) fn on_key(&mut self, key: KeyEvent) {
        if self.handle_global_shortcuts(key) {
            return;
        }

        match self.focus {
            Focus::FileTree => self.handle_tree_key(key),
            Focus::Editor => self.handle_editor_key(key),
        }
    }

    fn handle_global_shortcuts(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('q') => {
                    self.should_quit = true;
                    return true;
                }
                KeyCode::Char('s') => {
                    if let Err(error) = self.save_file() {
                        self.status = format!("Save failed: {error}");
                    }
                    return true;
                }
                _ => {}
            }
        }

        if key.code == KeyCode::Tab {
            self.focus = match self.focus {
                Focus::Editor => Focus::FileTree,
                Focus::FileTree => Focus::Editor,
            };
            return true;
        }

        false
    }

    fn handle_tree_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up => {
                if self.tree_selected > 0 {
                    self.tree_selected -= 1;
                }
            }
            KeyCode::Down => {
                if self.tree_selected + 1 < self.tree_items.len() {
                    self.tree_selected += 1;
                }
            }
            KeyCode::Enter | KeyCode::Right => self.activate_tree_item(),
            KeyCode::Left => self.collapse_tree_item(),
            _ => {}
        }
    }

    fn activate_tree_item(&mut self) {
        if self.tree_items.is_empty() {
            return;
        }

        if self.tree_items[self.tree_selected].is_dir {
            self.toggle_selected_dir();
            return;
        }

        let path = self.tree_items[self.tree_selected].path.clone();
        if let Err(error) = self.open_file(path) {
            self.status = format!("Open failed: {error}");
        }
    }

    fn collapse_tree_item(&mut self) {
        if self.tree_items.is_empty() {
            return;
        }

        if self.tree_items[self.tree_selected].is_dir
            && self
                .expanded_dirs
                .contains(&self.tree_items[self.tree_selected].path)
        {
            self.toggle_selected_dir();
        }
    }

    fn handle_editor_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up => self.move_cursor_up(),
            KeyCode::Down => self.move_cursor_down(),
            KeyCode::Left => self.move_cursor_left(),
            KeyCode::Right => self.move_cursor_right(),
            KeyCode::Home => {
                self.cursor_col = 0;
                self.preferred_col = self.cursor_col;
            }
            KeyCode::End => {
                self.cursor_col = line_len_chars(&self.lines[self.cursor_line]);
                self.preferred_col = self.cursor_col;
            }
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

    fn move_cursor_up(&mut self) {
        if self.cursor_line > 0 {
            self.cursor_line -= 1;
            let max = line_len_chars(&self.lines[self.cursor_line]);
            self.cursor_col = self.preferred_col.min(max);
        }
    }

    fn move_cursor_down(&mut self) {
        if self.cursor_line + 1 < self.lines.len() {
            self.cursor_line += 1;
            let max = line_len_chars(&self.lines[self.cursor_line]);
            self.cursor_col = self.preferred_col.min(max);
        }
    }

    fn move_cursor_left(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        } else if self.cursor_line > 0 {
            self.cursor_line -= 1;
            self.cursor_col = line_len_chars(&self.lines[self.cursor_line]);
        }

        self.preferred_col = self.cursor_col;
    }

    fn move_cursor_right(&mut self) {
        let len = line_len_chars(&self.lines[self.cursor_line]);
        if self.cursor_col < len {
            self.cursor_col += 1;
        } else if self.cursor_line + 1 < self.lines.len() {
            self.cursor_line += 1;
            self.cursor_col = 0;
        }

        self.preferred_col = self.cursor_col;
    }
}
