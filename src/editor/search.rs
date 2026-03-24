use crate::core::{Cursor, Document};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SearchMatch {
    pub line: usize,
    pub start_column: usize,
    pub end_column: usize,
}

#[derive(Debug, Default, Clone)]
pub struct SearchState {
    confirmed_query: String,
    preview_query: Option<String>,
    matches: Vec<SearchMatch>,
    active_index: Option<usize>,
}

impl SearchState {
    pub fn confirmed_query(&self) -> &str {
        &self.confirmed_query
    }

    pub fn active_match(&self) -> Option<SearchMatch> {
        self.active_index
            .and_then(|index| self.matches.get(index).copied())
    }

    #[cfg(test)]
    pub fn matches(&self) -> &[SearchMatch] {
        &self.matches
    }

    pub fn begin_preview(&mut self, query: String, document: &Document, cursor: Cursor) {
        self.preview_query = Some(query.clone());
        self.recompute(&query, document, cursor);
    }

    pub fn update_preview(&mut self, query: String, document: &Document, cursor: Cursor) {
        self.preview_query = Some(query.clone());
        self.recompute(&query, document, cursor);
    }

    pub fn confirm_preview(&mut self) {
        self.confirmed_query = self.preview_query.clone().unwrap_or_default();
    }

    pub fn restore_confirmed(&mut self, document: &Document, cursor: Cursor) {
        self.preview_query = None;
        let query = self.confirmed_query.clone();
        self.recompute(&query, document, cursor);
    }

    pub fn clear(&mut self) {
        self.confirmed_query.clear();
        self.preview_query = None;
        self.matches.clear();
        self.active_index = None;
    }

    pub fn refresh_for_document(&mut self, document: &Document, cursor: Cursor) {
        let query = self
            .preview_query
            .clone()
            .unwrap_or_else(|| self.confirmed_query.clone());
        self.recompute(&query, document, cursor);
    }

    pub fn select_next(&mut self) -> Option<SearchMatch> {
        let len = self.matches.len();
        if len == 0 {
            self.active_index = None;
            return None;
        }

        let next = self
            .active_index
            .map(|index| (index + 1) % len)
            .unwrap_or(0);
        self.active_index = Some(next);
        self.active_match()
    }

    pub fn select_previous(&mut self) -> Option<SearchMatch> {
        let len = self.matches.len();
        if len == 0 {
            self.active_index = None;
            return None;
        }

        let previous = self
            .active_index
            .map(|index| if index == 0 { len - 1 } else { index - 1 })
            .unwrap_or(len - 1);
        self.active_index = Some(previous);
        self.active_match()
    }

    pub fn is_match_at(&self, line: usize, column: usize) -> bool {
        self.matches.iter().any(|item| {
            item.line == line && column >= item.start_column && column < item.end_column
        })
    }

    pub fn is_active_match_at(&self, line: usize, column: usize) -> bool {
        self.active_match().is_some_and(|item| {
            item.line == line && column >= item.start_column && column < item.end_column
        })
    }

    fn recompute(&mut self, query: &str, document: &Document, cursor: Cursor) {
        self.matches = if query.is_empty() {
            Vec::new()
        } else {
            collect_matches(document, query)
        };
        self.active_index = find_active_match_index(&self.matches, cursor);
    }
}

fn collect_matches(document: &Document, query: &str) -> Vec<SearchMatch> {
    let case_sensitive = query.chars().any(char::is_uppercase);
    let query_lower = query.to_lowercase();
    let needle_chars: Vec<char> = query.chars().collect();
    let needle_len = needle_chars.len();
    if needle_len == 0 {
        return Vec::new();
    }

    let mut matches = Vec::new();
    for line in 0..document.line_count() {
        let line_text = document.line_text(line);
        let chars: Vec<char> = line_text.chars().collect();
        let mut index = 0usize;
        while index + needle_len <= chars.len() {
            let hay = chars[index..index + needle_len].iter().collect::<String>();
            let matched = if case_sensitive {
                hay == query
            } else {
                hay.to_lowercase() == query_lower
            };

            if matched {
                matches.push(SearchMatch {
                    line,
                    start_column: index,
                    end_column: index + needle_len,
                });
                index += needle_len;
            } else {
                index += 1;
            }
        }
    }

    matches
}

fn find_active_match_index(matches: &[SearchMatch], cursor: Cursor) -> Option<usize> {
    if matches.is_empty() {
        return None;
    }

    matches
        .iter()
        .position(|item| {
            item.line > cursor.line
                || (item.line == cursor.line && item.start_column >= cursor.column)
        })
        .or(Some(0))
}

#[cfg(test)]
mod tests {
    use crate::core::{Cursor, Document};
    use crate::editor::SearchState;

    #[test]
    fn lowercase_query_is_case_insensitive() {
        let document = Document::from_text(None, "Hello\nhello");
        let mut state = SearchState::default();
        state.begin_preview(String::from("hello"), &document, Cursor::new(0, 0));
        assert_eq!(state.matches().len(), 2);
    }

    #[test]
    fn uppercase_query_is_case_sensitive() {
        let document = Document::from_text(None, "Hello\nhello");
        let mut state = SearchState::default();
        state.begin_preview(String::from("Hello"), &document, Cursor::new(0, 0));
        assert_eq!(state.matches().len(), 1);
    }
}
