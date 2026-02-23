use std::collections::HashSet;
use std::path::Path;

use crate::app::state::CompletionItem;

use super::matching::completion_match_text;
use super::text_edit::{completion_insert_text, completion_replace_range};

pub(super) fn collect_items(
    incoming: Vec<lsp_types::CompletionItem>,
    line: usize,
    col: usize,
    default_start: usize,
) -> Vec<CompletionItem> {
    let mut items = Vec::new();
    let mut seen = HashSet::new();

    for item in incoming {
        let Some((replace_start_col, replace_end_col)) =
            completion_replace_range(&item, line, default_start, col)
        else {
            continue;
        };

        let insert_text = completion_insert_text(&item);
        if insert_text.is_empty() {
            continue;
        }

        let label = item.label.clone();
        let sort_text = item.sort_text.clone().unwrap_or_else(|| label.clone());
        let match_text = completion_match_text(&item, &insert_text);
        let dedupe_key = format!(
            "{}\u{1f}{}\u{1f}{}\u{1f}{}",
            replace_start_col, replace_end_col, match_text, insert_text
        );
        if !seen.insert(dedupe_key) {
            continue;
        }

        items.push(CompletionItem {
            label,
            insert_text,
            match_text,
            replace_start_col,
            replace_end_col,
            sort_text,
        });
    }

    items
}

pub(super) fn same_path(left: &Path, right: &Path) -> bool {
    if left == right {
        return true;
    }
    match (std::fs::canonicalize(left), std::fs::canonicalize(right)) {
        (Ok(a), Ok(b)) => a == b,
        _ => false,
    }
}
