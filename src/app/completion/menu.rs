use crossterm::event::{KeyCode, KeyEvent};

use crate::app::{App, state::CompletionState};

const COMPLETION_WINDOW_ROWS: usize = 8;

impl App {
    pub(crate) fn handle_completion_key(&mut self, key: KeyEvent) -> bool {
        let Some(line) = self.completion.as_ref().map(|state| state.line) else {
            return false;
        };
        if self.cursor_line != line {
            self.completion = None;
            return false;
        }

        match key.code {
            KeyCode::Up => {
                if let Some(state) = self.completion.as_mut() {
                    state.selected = state.selected.saturating_sub(1);
                    ensure_selection_visible(state);
                }
                true
            }
            KeyCode::Down => {
                if let Some(state) = self.completion.as_mut() {
                    let max = state.items.len().saturating_sub(1);
                    state.selected = (state.selected + 1).min(max);
                    ensure_selection_visible(state);
                }
                true
            }
            KeyCode::Enter | KeyCode::Tab => {
                self.apply_selected_completion();
                true
            }
            KeyCode::Esc => {
                self.completion = None;
                true
            }
            _ => {
                self.completion = None;
                false
            }
        }
    }
}

fn ensure_selection_visible(state: &mut CompletionState) {
    if state.selected < state.scroll {
        state.scroll = state.selected;
    }
    let end = state.scroll + COMPLETION_WINDOW_ROWS;
    if state.selected >= end {
        state.scroll = state.selected + 1 - COMPLETION_WINDOW_ROWS;
    }
}
