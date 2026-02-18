use crossterm::event::{KeyCode, KeyEvent};
use lsp_types::{CompletionItem as LspCompletionItem, CompletionTextEdit, InsertTextFormat};
use std::path::Path;

use crate::app::editor::{byte_index_for_char, line_len_chars};
use crate::app::{App, state::CompletionState};

const MAX_COMPLETIONS: usize = 12;
const COMPLETION_WINDOW_ROWS: usize = 8;

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
        if !same_path(current_path, &update.path) {
            return;
        }
        if self.cursor_line != update.line {
            return;
        }

        let default_start = identifier_start_col(&self.lines[self.cursor_line], update.col);
        let mut items = Vec::new();
        for item in update.items {
            if let Some((replace_start_col, replace_end_col)) =
                completion_replace_range(&item, update.line, default_start, update.col)
            {
                let label = item.label.clone();
                let sort_text = item.sort_text.clone().unwrap_or_else(|| label.clone());
                let insert_text = completion_insert_text(&item);
                if insert_text.is_empty() {
                    continue;
                }
                items.push(crate::app::state::CompletionItem {
                    label,
                    insert_text,
                    replace_start_col,
                    replace_end_col,
                    sort_text,
                });
            }
        }

        let typed_prefix = self.current_typed_prefix(update.line, update.col, &items);
        items.sort_by(|left, right| {
            completion_rank(left, &typed_prefix)
                .cmp(&completion_rank(right, &typed_prefix))
                .then(left.sort_text.cmp(&right.sort_text))
                .then(left.label.cmp(&right.label))
        });
        items.truncate(MAX_COMPLETIONS);

        if items.is_empty() {
            self.completion = None;
            return;
        }

        self.completion = Some(CompletionState {
            line: update.line,
            anchor_col: update.col,
            selected: 0,
            scroll: 0,
            items,
        });
    }

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
                    ensure_completion_selection_visible(state);
                }
                true
            }
            KeyCode::Down => {
                if let Some(state) = self.completion.as_mut() {
                    let max = state.items.len().saturating_sub(1);
                    state.selected = (state.selected + 1).min(max);
                    ensure_completion_selection_visible(state);
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

    fn apply_selected_completion(&mut self) {
        let Some(state) = self.completion.as_ref() else {
            return;
        };
        let Some(item) = state.items.get(state.selected).cloned() else {
            self.completion = None;
            return;
        };

        self.begin_edit();
        self.delete_selection_inner();

        let line = &mut self.lines[self.cursor_line];
        let start_col = item.replace_start_col.min(line_len_chars(line));
        let end_col = item
            .replace_end_col
            .min(line_len_chars(line))
            .max(start_col);
        let start_byte = byte_index_for_char(line, start_col);
        let end_byte = byte_index_for_char(line, end_col);
        line.replace_range(start_byte..end_byte, &item.insert_text);

        self.cursor_col = start_col + item.insert_text.chars().count();
        self.preferred_col = self.cursor_col;
        self.selection_anchor = None;
        self.completion = None;
        self.mark_changed();
    }

    fn current_typed_prefix(
        &self,
        line: usize,
        col: usize,
        items: &[crate::app::state::CompletionItem],
    ) -> String {
        let text = self.lines.get(line).map(String::as_str).unwrap_or_default();
        let mut start = identifier_start_col(text, col);
        if let Some(min_start) = items.iter().map(|item| item.replace_start_col).min() {
            start = start.max(min_start);
        }

        let end = col.min(line_len_chars(text));
        if start >= end {
            return String::new();
        }

        let start_byte = byte_index_for_char(text, start);
        let end_byte = byte_index_for_char(text, end);
        text[start_byte..end_byte].to_string()
    }
}

fn ensure_completion_selection_visible(state: &mut CompletionState) {
    if state.selected < state.scroll {
        state.scroll = state.selected;
    }
    let end = state.scroll + COMPLETION_WINDOW_ROWS;
    if state.selected >= end {
        state.scroll = state.selected + 1 - COMPLETION_WINDOW_ROWS;
    }
}

fn completion_rank(item: &crate::app::state::CompletionItem, typed_prefix: &str) -> (u8, String) {
    if typed_prefix.is_empty() {
        return (3, item.label.to_ascii_lowercase());
    }

    let prefix = typed_prefix.to_ascii_lowercase();
    let label = item.label.to_ascii_lowercase();
    let insert = item.insert_text.to_ascii_lowercase();

    if label == prefix || insert == prefix {
        return (0, label);
    }
    if label.starts_with(&prefix) || insert.starts_with(&prefix) {
        return (1, label);
    }
    if label.contains(&prefix) || insert.contains(&prefix) {
        return (2, label);
    }
    (3, label)
}

fn completion_insert_text(item: &LspCompletionItem) -> String {
    let from_text_edit = match item.text_edit.as_ref() {
        Some(CompletionTextEdit::Edit(edit)) => Some(edit.new_text.clone()),
        Some(CompletionTextEdit::InsertAndReplace(edit)) => Some(edit.new_text.clone()),
        None => None,
    };

    let base = from_text_edit
        .or_else(|| item.insert_text.clone())
        .unwrap_or_else(|| item.label.clone());

    if item.insert_text_format == Some(InsertTextFormat::SNIPPET) {
        strip_snippet_markers(&base)
    } else {
        base
    }
}

fn completion_replace_range(
    item: &LspCompletionItem,
    line: usize,
    default_start: usize,
    default_end: usize,
) -> Option<(usize, usize)> {
    match item.text_edit.as_ref() {
        Some(CompletionTextEdit::Edit(edit)) => {
            if edit.range.start.line as usize != line || edit.range.end.line as usize != line {
                return None;
            }
            Some((
                edit.range.start.character as usize,
                edit.range.end.character as usize,
            ))
        }
        Some(CompletionTextEdit::InsertAndReplace(edit)) => {
            if edit.insert.start.line as usize != line || edit.insert.end.line as usize != line {
                return None;
            }
            Some((
                edit.insert.start.character as usize,
                edit.insert.end.character as usize,
            ))
        }
        None => Some((default_start, default_end)),
    }
}

fn identifier_start_col(line: &str, col: usize) -> usize {
    let chars: Vec<char> = line.chars().collect();
    let mut idx = col.min(chars.len());
    while idx > 0 {
        let ch = chars[idx - 1];
        if ch.is_alphanumeric() || ch == '_' {
            idx -= 1;
        } else {
            break;
        }
    }
    idx
}

fn strip_snippet_markers(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '$' {
            match chars.peek() {
                Some('{') => {
                    let _ = chars.next();
                    let mut placeholder = String::new();
                    for part in chars.by_ref() {
                        if part == '}' {
                            break;
                        }
                        placeholder.push(part);
                    }
                    if let Some((_, value)) = placeholder.split_once(':') {
                        out.push_str(value);
                    }
                }
                Some(next) if next.is_ascii_digit() => {
                    while let Some(next) = chars.peek() {
                        if next.is_ascii_digit() {
                            let _ = chars.next();
                        } else {
                            break;
                        }
                    }
                }
                _ => out.push(ch),
            }
        } else {
            out.push(ch);
        }
    }

    out
}

fn same_path(left: &Path, right: &Path) -> bool {
    if left == right {
        return true;
    }

    match (std::fs::canonicalize(left), std::fs::canonicalize(right)) {
        (Ok(a), Ok(b)) => a == b,
        _ => false,
    }
}
