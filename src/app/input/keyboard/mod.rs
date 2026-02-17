mod editor;
mod global;
mod tree;

use crossterm::event::KeyEvent;

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
}
