use std::cmp::Reverse;

use crate::app::App;

use super::{
    fuzzy::fuzzy_score,
    types::{PaletteAction, PaletteCommand, PaletteKind, PaletteMatch, PaletteState},
};

const MAX_RESULTS: usize = 8;

impl App {
    pub(super) fn palette_matches(&self, state: &PaletteState) -> Vec<PaletteMatch> {
        let mut matches = if state.kind == PaletteKind::Files {
            self.file_matches(&state.query)
        } else {
            self.command_matches(&state.query)
        };
        matches.sort_by_key(|item| (Reverse(item.score), item.label.clone()));
        matches.truncate(MAX_RESULTS);
        matches
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
}
