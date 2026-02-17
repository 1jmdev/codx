use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

use super::{rect_contains, App, Focus};
use crate::app::editor::line_len_chars;

impl App {
    pub(crate) fn on_key(&mut self, key: KeyEvent) {
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('q') => {
                    self.should_quit = true;
                    return;
                }
                KeyCode::Char('s') => {
                    if let Err(error) = self.save_file() {
                        self.status = format!("Save failed: {error}");
                    }
                    return;
                }
                _ => {}
            }
        }

        if key.code == KeyCode::Tab {
            self.focus = match self.focus {
                Focus::Editor => Focus::FileTree,
                Focus::FileTree => Focus::Editor,
            };
            return;
        }

        match self.focus {
            Focus::FileTree => self.handle_tree_key(key),
            Focus::Editor => self.handle_editor_key(key),
        }
    }

    pub(crate) fn on_mouse(&mut self, mouse: MouseEvent) {
        if !matches!(mouse.kind, MouseEventKind::Down(MouseButton::Left)) {
            return;
        }

        let x = mouse.column;
        let y = mouse.row;

        if rect_contains(self.ui.tree_inner, x, y) {
            self.focus = Focus::FileTree;
            let clicked_row = (y - self.ui.tree_inner.y) as usize;
            let idx = self.tree_scroll + clicked_row;
            if idx < self.tree_items.len() {
                self.tree_selected = idx;
                if self.tree_items[idx].is_dir {
                    self.toggle_selected_dir();
                } else {
                    let path = self.tree_items[idx].path.clone();
                    if let Err(error) = self.open_file(path) {
                        self.status = format!("Open failed: {error}");
                    }
                }
            }
            return;
        }

        if rect_contains(self.ui.editor_inner, x, y) {
            self.focus = Focus::Editor;
            let clicked_row = (y - self.ui.editor_inner.y) as usize;
            let line = self.editor_scroll + clicked_row;
            if !self.lines.is_empty() {
                self.cursor_line = line.min(self.lines.len() - 1);
            }

            let number_width = self.line_number_width() + 1;
            let col =
                (x.saturating_sub(self.ui.editor_inner.x) as usize).saturating_sub(number_width);
            let max_col = line_len_chars(&self.lines[self.cursor_line]);
            self.cursor_col = col.min(max_col);
        }
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
            KeyCode::Enter | KeyCode::Right => {
                if self.tree_items.is_empty() {
                    return;
                }

                if self.tree_items[self.tree_selected].is_dir {
                    self.toggle_selected_dir();
                } else {
                    let path = self.tree_items[self.tree_selected].path.clone();
                    if let Err(error) = self.open_file(path) {
                        self.status = format!("Open failed: {error}");
                    }
                }
            }
            KeyCode::Left => {
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
            _ => {}
        }
    }

    fn handle_editor_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up => {
                if self.cursor_line > 0 {
                    self.cursor_line -= 1;
                    self.clamp_cursor_col();
                }
            }
            KeyCode::Down => {
                if self.cursor_line + 1 < self.lines.len() {
                    self.cursor_line += 1;
                    self.clamp_cursor_col();
                }
            }
            KeyCode::Left => {
                if self.cursor_col > 0 {
                    self.cursor_col -= 1;
                } else if self.cursor_line > 0 {
                    self.cursor_line -= 1;
                    self.cursor_col = line_len_chars(&self.lines[self.cursor_line]);
                }
            }
            KeyCode::Right => {
                let len = line_len_chars(&self.lines[self.cursor_line]);
                if self.cursor_col < len {
                    self.cursor_col += 1;
                } else if self.cursor_line + 1 < self.lines.len() {
                    self.cursor_line += 1;
                    self.cursor_col = 0;
                }
            }
            KeyCode::Home => self.cursor_col = 0,
            KeyCode::End => self.cursor_col = line_len_chars(&self.lines[self.cursor_line]),
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
}
