use super::types::{SearchField, SearchReplaceState};
use crate::app::{App, Focus};

impl App {
    pub(crate) fn open_search_replace(&mut self, show_replace: bool) {
        self.completion = None;
        self.palette = None;
        self.focus = Focus::Editor;

        if let Some(sr) = self.search_replace.as_mut() {
            sr.show_replace = show_replace;
            sr.focused_field = if show_replace {
                SearchField::Replace
            } else {
                SearchField::Search
            };
            return;
        }

        let query = self.selected_text().unwrap_or_default();
        let focused_field = if show_replace {
            SearchField::Replace
        } else {
            SearchField::Search
        };
        let mut state = SearchReplaceState {
            query,
            replacement: String::new(),
            focused_field,
            matches: Vec::new(),
            current_match: 0,
            show_replace,
        };
        self.recompute_matches(&mut state);
        self.search_replace = Some(state);
        self.scroll_to_current_match();
    }

    pub(crate) fn close_search_replace(&mut self) {
        self.search_replace = None;
    }

    /// Return the currently selected text (single-line only), if any.
    pub(super) fn selected_text(&self) -> Option<String> {
        let (start, end) = self.selection_bounds()?;
        if start.line != end.line {
            return None;
        }
        let chars: Vec<char> = self.lines[start.line].chars().collect();
        Some(chars[start.col..end.col].iter().collect())
    }
}
