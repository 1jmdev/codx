mod editor;
mod global;
mod keymap;
mod tree;

use crossterm::event::KeyEvent;
use crossterm::event::{KeyCode, KeyModifiers};

use crate::app::{App, Focus};

impl App {
    pub(crate) fn on_key(&mut self, key: KeyEvent) {
        let had_completion = self.completion.is_some();

        if self.handle_completion_key(key) {
            return;
        }

        if self.handle_palette_key(key) {
            return;
        }

        if self.handle_search_replace_key(key) {
            return;
        }

        if self.handle_global_shortcuts(key) {
            return;
        }

        match self.focus {
            Focus::FileTree => self.handle_tree_key(key),
            Focus::Editor => self.handle_editor_key(key),
        }

        if had_completion && self.focus == Focus::Editor && is_completion_refresh_key(key) {
            self.trigger_completion();
        }
    }
}

fn is_completion_refresh_key(key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Backspace | KeyCode::Delete => true,
        KeyCode::Char(_) => key.modifiers.is_empty() || key.modifiers == KeyModifiers::SHIFT,
        _ => false,
    }
}
