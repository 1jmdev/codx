use super::types::SearchReplaceState;
use crate::app::App;

impl App {
    /// Recompute all match positions for the current query.
    pub(super) fn recompute_matches(&self, state: &mut SearchReplaceState) {
        state.matches.clear();
        if state.query.is_empty() {
            return;
        }

        let q_chars: Vec<char> = state.query.chars().collect();
        let qlen = q_chars.len();

        for (line_idx, line) in self.lines.iter().enumerate() {
            let chars: Vec<char> = line.chars().collect();
            if chars.len() < qlen {
                continue;
            }
            let mut col = 0usize;
            while col + qlen <= chars.len() {
                let hit = chars[col..col + qlen]
                    .iter()
                    .zip(q_chars.iter())
                    .all(|(a, b)| a.eq_ignore_ascii_case(b));
                if hit {
                    state.matches.push((line_idx, col, col + qlen));
                    col += qlen; // no overlapping matches
                } else {
                    col += 1;
                }
            }
        }

        // Keep current_match in bounds after the query changed.
        if state.matches.is_empty() {
            state.current_match = 0;
        } else {
            state.current_match = state.current_match.min(state.matches.len() - 1);
        }
    }

    /// Recompute matches and scroll the editor to the active one.
    pub(super) fn refresh_matches(&mut self) {
        // Take the state out, recompute, put it back — avoids split-borrow issues.
        if let Some(mut sr) = self.search_replace.take() {
            self.recompute_matches(&mut sr);
            self.search_replace = Some(sr);
        }
        self.scroll_to_current_match();
    }

    /// Move the cursor to the active match and scroll it into view.
    pub(super) fn scroll_to_current_match(&mut self) {
        let Some(&(line, col, _)) = self
            .search_replace
            .as_ref()
            .and_then(|sr| sr.matches.get(sr.current_match))
        else {
            return;
        };
        self.cursor_line = line;
        self.cursor_col = col;
        self.preferred_col = col;
        let view_height = self.ui.editor_inner.height as usize;
        self.ensure_cursor_visible(view_height);
    }

    pub(super) fn goto_next_match(&mut self) {
        if let Some(sr) = self.search_replace.as_mut() {
            if !sr.matches.is_empty() {
                sr.current_match = (sr.current_match + 1) % sr.matches.len();
            }
        }
        self.scroll_to_current_match();
    }

    pub(super) fn goto_prev_match(&mut self) {
        if let Some(sr) = self.search_replace.as_mut() {
            if !sr.matches.is_empty() {
                let len = sr.matches.len();
                sr.current_match = sr.current_match.checked_sub(1).unwrap_or(len - 1);
            }
        }
        self.scroll_to_current_match();
    }
}
