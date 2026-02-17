use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::{palette::PaletteKind, App, Focus};

impl App {
    pub(super) fn handle_global_shortcuts(&mut self, key: KeyEvent) -> bool {
        if key.code == KeyCode::F(1) {
            self.open_palette(PaletteKind::Commands);
            return true;
        }

        if !key.modifiers.contains(KeyModifiers::CONTROL) {
            return false;
        }

        let shift = key.modifiers.contains(KeyModifiers::SHIFT);

        if key.code == KeyCode::Char(' ') && !shift {
            self.open_palette(PaletteKind::Commands);
            return true;
        }

        let KeyCode::Char(ch) = key.code else {
            return false;
        };

        match ch.to_ascii_lowercase() {
            'q' if !shift => {
                self.should_quit = true;
                true
            }
            's' if !shift => {
                if let Err(error) = self.save_file() {
                    self.status = format!("Save failed: {error}");
                }
                true
            }
            'z' if !shift && self.focus == Focus::Editor => {
                self.undo();
                true
            }
            'y' if !shift && self.focus == Focus::Editor => {
                self.redo();
                true
            }
            'd' if !shift && self.focus == Focus::Editor => {
                self.duplicate_line_or_selection();
                true
            }
            'k' if shift && self.focus == Focus::Editor => {
                self.delete_line_or_selection();
                true
            }
            'e' if shift && self.focus == Focus::FileTree => {
                self.activate_tree_item();
                true
            }
            'b' if !shift => {
                self.sidebar_open = !self.sidebar_open;
                if !self.sidebar_open {
                    self.focus = Focus::Editor;
                }
                true
            }
            'p' if !shift => {
                self.open_palette(PaletteKind::Files);
                true
            }
            _ => false,
        }
    }
}
