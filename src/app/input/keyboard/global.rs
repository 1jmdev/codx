use crate::app::{palette::PaletteKind, App, Focus};

use super::keymap::{resolve_global_command, GlobalCommand};
use crossterm::event::KeyEvent;

impl App {
    pub(super) fn handle_global_shortcuts(&mut self, key: KeyEvent) -> bool {
        let Some(command) = resolve_global_command(key, self.focus) else {
            return false;
        };

        match command {
            GlobalCommand::ShowCommandPalette => {
                self.open_palette(PaletteKind::Commands);
                true
            }
            GlobalCommand::ShowFilePalette => {
                self.open_palette(PaletteKind::Files);
                true
            }
            GlobalCommand::ShowSearch => {
                self.open_search_replace(false);
                true
            }
            GlobalCommand::ShowSearchReplace => {
                self.open_search_replace(true);
                true
            }
            GlobalCommand::ShowGrepSearch => {
                self.open_palette(PaletteKind::GrepSearch);
                true
            }
            GlobalCommand::ShowGrepReplace => {
                self.open_palette(PaletteKind::GrepReplace);
                true
            }
            GlobalCommand::TriggerSuggest => {
                self.trigger_completion();
                true
            }
            GlobalCommand::ToggleSidebar => {
                self.sidebar_open = !self.sidebar_open;
                if !self.sidebar_open {
                    self.focus = Focus::Editor;
                }
                true
            }
            GlobalCommand::Save => {
                if let Err(error) = self.save_file() {
                    self.status = format!("Save failed: {error}");
                }
                true
            }
            GlobalCommand::Quit => {
                self.should_quit = true;
                true
            }
            GlobalCommand::Undo => {
                self.undo();
                true
            }
            GlobalCommand::Redo => {
                self.redo();
                true
            }
            GlobalCommand::DuplicateLine => {
                self.duplicate_line_or_selection();
                true
            }
            GlobalCommand::DeleteLine => {
                self.delete_line_or_selection();
                true
            }
            GlobalCommand::ActivateTreeItem => {
                self.activate_tree_item();
                true
            }
        }
    }
}
