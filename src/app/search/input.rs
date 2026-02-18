use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::types::SearchField;
use crate::app::App;

impl App {
    /// Returns `true` when the key event was consumed by the search/replace panel.
    pub(crate) fn handle_search_replace_key(&mut self, key: KeyEvent) -> bool {
        if self.search_replace.is_none() {
            return false;
        }

        match key.code {
            KeyCode::Esc => self.close_search_replace(),

            // Tab / Shift+Tab — switch between Search and Replace fields
            KeyCode::Tab | KeyCode::BackTab => self.toggle_search_field(),

            // Enter / F3 — navigate matches (Shift reverses direction)
            KeyCode::Enter if key.modifiers == KeyModifiers::SHIFT => self.goto_prev_match(),
            KeyCode::Enter => self.goto_next_match(),
            KeyCode::F(3) if key.modifiers == KeyModifiers::SHIFT => self.goto_prev_match(),
            KeyCode::F(3) => self.goto_next_match(),

            // Alt+R — replace current match
            KeyCode::Char('r') if key.modifiers == KeyModifiers::ALT => self.replace_current(),

            // Alt+A — replace all matches
            KeyCode::Char('a') if key.modifiers == KeyModifiers::ALT => self.replace_all(),

            // Regular typing — route to focused field
            KeyCode::Char(ch)
                if key.modifiers.is_empty() || key.modifiers == KeyModifiers::SHIFT =>
            {
                self.push_search_char(ch);
            }
            KeyCode::Backspace => self.pop_search_char(),

            _ => return false,
        }

        true
    }

    fn toggle_search_field(&mut self) {
        if let Some(sr) = self.search_replace.as_mut() {
            sr.focused_field = match sr.focused_field {
                SearchField::Search => SearchField::Replace,
                SearchField::Replace => SearchField::Search,
            };
        }
    }

    fn push_search_char(&mut self, ch: char) {
        if let Some(sr) = self.search_replace.as_mut() {
            match sr.focused_field {
                SearchField::Search => {
                    sr.query.push(ch);
                    sr.current_match = 0;
                }
                SearchField::Replace => sr.replacement.push(ch),
            }
        }
        self.refresh_matches();
    }

    fn pop_search_char(&mut self) {
        if let Some(sr) = self.search_replace.as_mut() {
            match sr.focused_field {
                SearchField::Search => {
                    sr.query.pop();
                    sr.current_match = 0;
                }
                SearchField::Replace => {
                    sr.replacement.pop();
                }
            }
        }
        self.refresh_matches();
    }
}
