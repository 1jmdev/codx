use std::collections::HashMap;
use std::path::{Path, PathBuf};

use ignore::WalkBuilder;

use crate::syntax::{language_for_path, LanguageId};

#[derive(Debug, Clone, Default)]
pub struct WorkspaceDiscovery {
    pub languages: Vec<LanguageId>,
}

impl WorkspaceDiscovery {
    pub fn discover(root: &Path) -> Self {
        let mut by_language: HashMap<LanguageId, Vec<PathBuf>> = HashMap::new();

        for entry in WalkBuilder::new(root)
            .hidden(true)
            .git_ignore(true)
            .git_exclude(true)
            .parents(true)
            .build()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_some_and(|kind| kind.is_file()))
        {
            let path = entry.into_path();
            if let Some(language_id) = language_for_path(&path) {
                by_language.entry(language_id).or_default().push(path);
            }
        }

        let mut languages = by_language.keys().copied().collect::<Vec<_>>();
        languages.sort_unstable_by_key(|language| *language as u8);

        Self { languages }
    }
}
