mod controller;
mod input;
mod matches;
mod replace;
mod types;

pub(crate) use types::{SearchField, SearchReplaceState};

use crate::app::App;

impl App {
    /// All match positions — consumed by the editor renderer for highlights.
    pub(crate) fn search_match_ranges(&self) -> &[(usize, usize, usize)] {
        self.search_replace
            .as_ref()
            .map(|sr| sr.matches.as_slice())
            .unwrap_or(&[])
    }

    /// Index of the active match, or `None` when there are no matches.
    pub(crate) fn search_current_match_idx(&self) -> Option<usize> {
        let sr = self.search_replace.as_ref()?;
        if sr.matches.is_empty() {
            None
        } else {
            Some(sr.current_match)
        }
    }
}
