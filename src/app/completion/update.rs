use crate::app::{
    App,
    state::{CompletionItem, CompletionState},
};

use super::items::{collect_items, same_path};
use super::matching::{completion_rank, identifier_start_col};

const MAX_COMPLETIONS: usize = 12;

impl App {
    pub(crate) fn trigger_completion(&mut self) {
        let Some(path) = self.current_file.clone() else {
            self.status = String::from("Open a file first to request completions.");
            return;
        };

        if self
            .lsp
            .request_completion(&path, self.cursor_line, self.cursor_col, &mut self.status)
        {
            self.completion = None;
        }
    }

    pub(crate) fn refresh_completion_from_lsp(&mut self) {
        let Some(update) = self.lsp.take_completion() else {
            return;
        };
        let Some(current_path) = self.current_file.as_ref() else {
            return;
        };
        if !same_path(current_path, &update.path) || self.cursor_line != update.line {
            return;
        }

        let default_start = identifier_start_col(&self.lines[self.cursor_line], update.col);
        let mut items = collect_items(update.items, update.line, update.col, default_start);
        let typed_prefix = self.current_typed_prefix(update.line, update.col, &items);

        items.retain(|item| completion_rank(item, &typed_prefix).is_some());
        items.sort_by(|left, right| {
            completion_rank(left, &typed_prefix)
                .cmp(&completion_rank(right, &typed_prefix))
                .then(left.sort_text.cmp(&right.sort_text))
                .then(left.label.cmp(&right.label))
        });
        items.truncate(MAX_COMPLETIONS);

        self.completion = (!items.is_empty()).then_some(CompletionState {
            line: update.line,
            anchor_col: update.col,
            selected: 0,
            scroll: 0,
            items,
        });
    }

    fn current_typed_prefix(&self, line: usize, col: usize, items: &[CompletionItem]) -> String {
        let text = self.lines.get(line).map(String::as_str).unwrap_or_default();
        let mut start = identifier_start_col(text, col);
        if let Some(min_start) = items.iter().map(|item| item.replace_start_col).min() {
            start = start.max(min_start);
        }

        let end = col.min(crate::app::editor::line_len_chars(text));
        if start >= end {
            return String::new();
        }

        let start_byte = crate::app::editor::byte_index_for_char(text, start);
        let end_byte = crate::app::editor::byte_index_for_char(text, end);
        text[start_byte..end_byte].to_string()
    }
}
