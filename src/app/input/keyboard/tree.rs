use crossterm::event::{KeyCode, KeyEvent};

use crate::app::App;

impl App {
    pub(super) fn handle_tree_key(&mut self, key: KeyEvent) {
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

    pub(super) fn activate_tree_item(&mut self) {
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
}
