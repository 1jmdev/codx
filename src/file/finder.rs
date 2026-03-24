use std::path::{Path, PathBuf};

use globset::{Glob, GlobSet, GlobSetBuilder};
use ignore::WalkBuilder;
use nucleo::pattern::{CaseMatching, Normalization, Pattern};
use nucleo::{Config, Matcher, Utf32String};

#[derive(Debug, Clone)]
pub struct FinderItem {
    pub path: PathBuf,
    pub display: String,
}

#[derive(Debug)]
pub struct FileFinder {
    root: PathBuf,
    files: Vec<FinderItem>,
    matcher: Matcher,
}

impl FileFinder {
    pub fn new(root: PathBuf) -> Self {
        let matcher = Matcher::new(Config::DEFAULT.match_paths());
        let files = collect_files(&root);
        Self {
            root,
            files,
            matcher,
        }
    }

    pub fn refresh(&mut self) {
        self.files = collect_files(&self.root);
    }

    pub fn search(&mut self, query: &str, limit: usize) -> Vec<FinderItem> {
        if query.trim().is_empty() {
            return self.files.iter().take(limit).cloned().collect();
        }

        let pattern = Pattern::parse(query, CaseMatching::Smart, Normalization::Smart);
        let mut scored = self
            .files
            .iter()
            .filter_map(|item| {
                let haystack = Utf32String::from(item.display.as_str());
                pattern
                    .score(haystack.slice(..), &mut self.matcher)
                    .map(|score| (score, item.clone()))
            })
            .collect::<Vec<_>>();

        scored.sort_by(|left, right| right.0.cmp(&left.0).then_with(|| left.1.display.cmp(&right.1.display)));
        scored.into_iter().take(limit).map(|(_, item)| item).collect()
    }
}

fn collect_files(root: &Path) -> Vec<FinderItem> {
    let ignore_set = ignored_paths();
    WalkBuilder::new(root)
        .hidden(false)
        .git_ignore(true)
        .git_exclude(true)
        .parents(true)
        .build()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_some_and(|kind| kind.is_file()))
        .filter_map(|entry| {
            let relative = entry.path().strip_prefix(root).ok()?.to_path_buf();
            if ignore_set.is_match(&relative) {
                return None;
            }

            Some(FinderItem {
                display: relative.display().to_string(),
                path: entry.into_path(),
            })
        })
        .collect()
}

fn ignored_paths() -> GlobSet {
    let mut builder = GlobSetBuilder::new();
    for pattern in ["target/**", ".git/**"] {
        if let Ok(glob) = Glob::new(pattern) {
            builder.add(glob);
        }
    }
    builder
        .build()
        .unwrap_or_else(|_| GlobSetBuilder::new().build().unwrap_or_else(|_| unreachable!()))
}
