use lsp_types::CompletionItem as LspCompletionItem;

use crate::app::state::CompletionItem;

use super::fuzzy::{best_edit_distance, subsequence_score};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct MatchRank {
    tier: u8,
    distance: u8,
    gaps: usize,
    len_delta: usize,
}

pub(super) fn completion_match_text(item: &LspCompletionItem, insert_text: &str) -> String {
    item.filter_text
        .clone()
        .or_else(|| item.insert_text.clone())
        .unwrap_or_else(|| insert_text.to_string())
}

pub(super) fn completion_rank(item: &CompletionItem, typed_prefix: &str) -> Option<MatchRank> {
    let query = typed_prefix.trim().to_ascii_lowercase();
    if query.is_empty() {
        return Some(MatchRank {
            tier: 4,
            distance: 0,
            gaps: 0,
            len_delta: item.match_text.chars().count(),
        });
    }

    let match_text = item.match_text.to_ascii_lowercase();
    let label = item.label.to_ascii_lowercase();
    if match_text == query || label == query {
        return Some(MatchRank {
            tier: 0,
            distance: 0,
            gaps: 0,
            len_delta: 0,
        });
    }
    if match_text.starts_with(&query) || label.starts_with(&query) {
        return Some(MatchRank {
            tier: 1,
            distance: 0,
            gaps: 0,
            len_delta: match_text
                .chars()
                .count()
                .saturating_sub(query.chars().count()),
        });
    }
    if let Some((gaps, len_delta)) = subsequence_score(&query, &match_text) {
        return Some(MatchRank {
            tier: 2,
            distance: 0,
            gaps,
            len_delta,
        });
    }

    let distance = best_edit_distance(&query, &match_text)?;
    let max_distance = if query.chars().count() >= 7 { 2 } else { 1 };
    if distance > max_distance {
        return None;
    }

    Some(MatchRank {
        tier: 3,
        distance,
        gaps: 0,
        len_delta: match_text.chars().count().abs_diff(query.chars().count()),
    })
}

pub(super) fn identifier_start_col(line: &str, col: usize) -> usize {
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
