use std::collections::HashSet;
use std::path::Path;

use ignore::WalkBuilder;

use crate::syntax::{language_for_path, LanguageId};

#[derive(Debug, Clone, Default)]
pub struct WorkspaceDiscovery {
    pub languages: Vec<LanguageId>,
}

impl WorkspaceDiscovery {
    pub fn discover(root: &Path) -> Self {
        let mut found = HashSet::new();
        let mut visitor = WalkBuilder::new(root);
        visitor
            .hidden(true)
            .git_ignore(true)
            .git_exclude(true)
            .parents(true)
            .max_depth(Some(6));

        for entry in visitor
            .build()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_some_and(|kind| kind.is_file()))
        {
            let path = entry.path();
            if let Some(language_id) = language_for_path(path) {
                found.insert(language_id);
            }
        }

        let mut languages = found.into_iter().collect::<Vec<_>>();
        languages.sort_unstable_by_key(|language| *language as u8);
        Self { languages }
    }
}
