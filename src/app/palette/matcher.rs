use std::cmp::Reverse;

use crate::app::App;

use super::{
    files::grep_project,
    fuzzy::fuzzy_score,
    types::{PaletteAction, PaletteCommand, PaletteKind, PaletteMatch, PaletteState},
};

const MAX_FUZZY_RESULTS: usize = 8;

impl App {
    pub(super) fn palette_matches(&self, state: &PaletteState) -> Vec<PaletteMatch> {
        match state.kind {
            PaletteKind::Files => {
                let mut matches = self.file_matches(&state.query);
                matches.sort_by_key(|item| (Reverse(item.score), item.label.clone()));
                matches.truncate(MAX_FUZZY_RESULTS);
                matches
            }
            PaletteKind::Commands => {
                let mut matches = self.command_matches(&state.query);
                matches.sort_by_key(|item| (Reverse(item.score), item.label.clone()));
                matches.truncate(MAX_FUZZY_RESULTS);
                matches
            }
            PaletteKind::GrepSearch | PaletteKind::GrepReplace => self.grep_matches(&state.query),
        }
    }

    fn file_matches(&self, query: &str) -> Vec<PaletteMatch> {
        self.file_picker_cache
            .iter()
            .filter_map(|path| {
                let label = path
                    .strip_prefix(&self.cwd)
                    .unwrap_or(path)
                    .to_string_lossy()
                    .replace('\\', "/");
                Some(PaletteMatch {
                    score: fuzzy_score(&label, query)?,
                    label,
                    action: PaletteAction::OpenFile(path.clone()),
                })
            })
            .collect()
    }

    fn command_matches(&self, query: &str) -> Vec<PaletteMatch> {
        let command = "Reload LSP Server";
        let Some(score) = fuzzy_score(command, query) else {
            return Vec::new();
        };
        vec![PaletteMatch {
            label: String::from(command),
            score,
            action: PaletteAction::Command(PaletteCommand::ReloadLsp),
        }]
    }

    fn grep_matches(&self, query: &str) -> Vec<PaletteMatch> {
        if query.is_empty() {
            return Vec::new();
        }
        let mut hits = grep_project(&self.cwd, query);
        hits.sort_by(|a, b| a.path.cmp(&b.path).then(a.line_number.cmp(&b.line_number)));

        hits.into_iter()
            .map(|m| {
                let rel = m
                    .path
                    .strip_prefix(&self.cwd)
                    .unwrap_or(&m.path)
                    .to_string_lossy()
                    .replace('\\', "/");
                let label = format!("{}:{}: {}", rel, m.line_number + 1, m.line_text.trim());
                PaletteMatch {
                    score: 0,
                    label,
                    action: PaletteAction::OpenFileAt(m.path, m.line_number, m.col_start),
                }
            })
            .collect()
    }
}
