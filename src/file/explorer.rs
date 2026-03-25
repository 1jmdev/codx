use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct ExplorerEntry {
    pub path: PathBuf,
    pub depth: usize,
    pub is_dir: bool,
}

#[derive(Debug)]
pub struct ExplorerState {
    root: PathBuf,
    visible: bool,
    entries: Vec<ExplorerEntry>,
    selected: usize,
    scroll_offset: usize,
    expanded: BTreeSet<PathBuf>,
}

impl ExplorerState {
    pub fn new(root: PathBuf) -> Self {
        let mut expanded = BTreeSet::new();
        expanded.insert(root.clone());
        let mut explorer = Self {
            root,
            visible: false,
            entries: Vec::new(),
            selected: 0,
            scroll_offset: 0,
            expanded,
        };
        explorer.refresh();
        explorer
    }

    pub fn visible(&self) -> bool {
        self.visible
    }

    pub fn toggle(&mut self) {
        self.visible = !self.visible;
        if self.visible {
            self.refresh();
        }
    }

    pub fn entries(&self) -> &[ExplorerEntry] {
        &self.entries
    }

    pub fn selected(&self) -> usize {
        self.selected
    }

    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    pub fn sync_scroll(&mut self, viewport_height: usize) {
        self.scroll_offset = compute_scroll_offset(
            self.scroll_offset,
            self.selected,
            self.entries.len(),
            viewport_height,
        );
    }

    pub fn move_selection(&mut self, delta: isize) {
        if self.entries.is_empty() {
            self.selected = 0;
            return;
        }

        let max_index = self.entries.len().saturating_sub(1);
        self.selected = if delta.is_negative() {
            self.selected.saturating_sub(delta.unsigned_abs())
        } else {
            (self.selected + delta as usize).min(max_index)
        };
    }

    pub fn selected_entry(&self) -> Option<&ExplorerEntry> {
        self.entries.get(self.selected)
    }

    pub fn is_expanded(&self, path: &std::path::Path) -> bool {
        self.expanded.contains(path)
    }

    pub fn toggle_selected_expansion(&mut self) {
        let Some(path) = self.selected_entry().map(|entry| entry.path.clone()) else {
            return;
        };
        let is_dir = self
            .selected_entry()
            .map(|entry| entry.is_dir)
            .unwrap_or(false);
        if !is_dir {
            return;
        }

        if self.expanded.contains(&path) {
            self.expanded.remove(&path);
        } else {
            self.expanded.insert(path);
        }
        self.refresh();
    }

    pub fn refresh(&mut self) {
        let selected_path = self.selected_entry().map(|entry| entry.path.clone());
        self.entries.clear();
        let root = self.root.clone();
        self.walk_directory(&root, 0);
        if let Some(selected_path) = selected_path {
            if let Some(index) = self
                .entries
                .iter()
                .position(|entry| entry.path == selected_path)
            {
                self.selected = index;
                self.scroll_offset = self.scroll_offset.min(self.selected);
                return;
            }
        }
        self.selected = self.selected.min(self.entries.len().saturating_sub(1));
        self.scroll_offset = self.scroll_offset.min(self.selected);
    }

    pub fn create(&mut self, relative: &str, directory: bool) -> Result<PathBuf, std::io::Error> {
        let path = self.root.join(relative);
        if directory {
            fs::create_dir_all(&path)?;
            self.expanded.insert(path.clone());
        } else {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            let _ = fs::File::create(&path)?;
        }
        self.refresh();
        Ok(path)
    }

    pub fn rename_selected(&mut self, relative: &str) -> Result<PathBuf, std::io::Error> {
        let Some(entry) = self.selected_entry() else {
            return Ok(self.root.clone());
        };
        let destination = self.root.join(relative);
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::rename(&entry.path, &destination)?;
        self.refresh();
        Ok(destination)
    }

    pub fn delete_selected(&mut self) -> Result<Option<PathBuf>, std::io::Error> {
        let Some(entry) = self.selected_entry() else {
            return Ok(None);
        };
        if entry.is_dir {
            fs::remove_dir_all(&entry.path)?;
        } else {
            fs::remove_file(&entry.path)?;
        }
        let removed = entry.path.clone();
        self.refresh();
        Ok(Some(removed))
    }

    fn walk_directory(&mut self, path: &Path, depth: usize) {
        let is_root = path == self.root;
        if !is_root {
            self.entries.push(ExplorerEntry {
                path: path.to_path_buf(),
                depth,
                is_dir: true,
            });
        }

        if !self.expanded.contains(path) {
            return;
        }

        let mut children = fs::read_dir(path)
            .ok()
            .into_iter()
            .flat_map(|entries| entries.filter_map(Result::ok))
            .collect::<Vec<_>>();
        children.sort_by_key(|entry| entry.path());

        for child in children {
            let child_path = child.path();
            let Ok(file_type) = child.file_type() else {
                continue;
            };

            if file_type.is_dir() {
                self.walk_directory(&child_path, depth + usize::from(!is_root));
            } else {
                self.entries.push(ExplorerEntry {
                    path: child_path,
                    depth: depth + usize::from(!is_root),
                    is_dir: false,
                });
            }
        }
    }
}

fn compute_scroll_offset(
    current_offset: usize,
    selected: usize,
    total_items: usize,
    viewport_height: usize,
) -> usize {
    if viewport_height == 0 || total_items == 0 {
        return 0;
    }

    let max_offset = total_items.saturating_sub(viewport_height);
    let mut offset = current_offset.min(max_offset);
    if selected < offset {
        offset = selected;
    } else if selected >= offset + viewport_height {
        offset = selected + 1 - viewport_height;
    }
    offset.min(max_offset)
}
