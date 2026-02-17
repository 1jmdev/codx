use std::{cmp::Ordering, fs, path::Path};

use super::{App, TreeItem};

impl App {
    pub(crate) fn ensure_tree_selection_visible(&mut self, view_height: usize) {
        if view_height == 0 {
            return;
        }

        if self.tree_selected < self.tree_scroll {
            self.tree_scroll = self.tree_selected;
        }

        let bottom = self.tree_scroll + view_height - 1;
        if self.tree_selected > bottom {
            self.tree_scroll = self.tree_selected + 1 - view_height;
        }
    }

    pub(crate) fn toggle_selected_dir(&mut self) {
        if self.tree_items.is_empty() {
            return;
        }

        if !self.tree_items[self.tree_selected].is_dir {
            return;
        }

        let path = self.tree_items[self.tree_selected].path.clone();
        if self.expanded_dirs.contains(&path) {
            self.expanded_dirs.remove(&path);
        } else {
            self.expanded_dirs.insert(path);
        }

        self.rebuild_tree();
    }

    pub(crate) fn rebuild_tree(&mut self) {
        let mut items = Vec::new();
        self.walk_dir(&self.cwd, 0, &mut items);
        self.tree_items = items;

        if self.tree_selected >= self.tree_items.len() {
            self.tree_selected = self.tree_items.len().saturating_sub(1);
        }
    }

    fn walk_dir(&self, dir: &Path, depth: usize, out: &mut Vec<TreeItem>) {
        let entries = match fs::read_dir(dir) {
            Ok(read_dir) => {
                let mut values = Vec::new();
                for entry in read_dir.flatten() {
                    values.push(entry);
                }
                values
            }
            Err(_) => return,
        };

        let mut entries = entries;
        entries.sort_by(|a, b| {
            let a_path = a.path();
            let b_path = b.path();
            let a_is_dir = a_path.is_dir();
            let b_is_dir = b_path.is_dir();

            match (a_is_dir, b_is_dir) {
                (true, false) => Ordering::Less,
                (false, true) => Ordering::Greater,
                _ => a
                    .file_name()
                    .to_string_lossy()
                    .cmp(&b.file_name().to_string_lossy()),
            }
        });

        for entry in entries {
            let path = entry.path();
            let is_dir = path.is_dir();
            out.push(TreeItem {
                path: path.clone(),
                name: entry.file_name().to_string_lossy().to_string(),
                depth,
                is_dir,
            });

            if is_dir && self.expanded_dirs.contains(&path) {
                self.walk_dir(&path, depth + 1, out);
            }
        }
    }
}
