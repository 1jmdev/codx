use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

const MAX_RECENT_FILES: usize = 50;

#[derive(Debug, Default)]
pub struct RecentFiles {
    entries: Vec<PathBuf>,
    storage_path: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
struct RecentFilesStore {
    entries: Vec<PathBuf>,
}

impl RecentFiles {
    pub fn load() -> Self {
        let storage_path = default_storage_path();
        let entries = fs::read_to_string(&storage_path)
            .ok()
            .and_then(|raw| serde_json::from_str::<RecentFilesStore>(&raw).ok())
            .map(|store| store.entries)
            .unwrap_or_default();

        Self {
            entries,
            storage_path,
        }
    }

    pub fn record(&mut self, path: &Path) {
        self.entries.retain(|item| item != path);
        self.entries.insert(0, path.to_path_buf());
        self.entries.truncate(MAX_RECENT_FILES);
        let _ = self.persist();
    }

    fn persist(&self) -> Result<(), std::io::Error> {
        if let Some(parent) = self.storage_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let raw = serde_json::to_string_pretty(&RecentFilesStore {
            entries: self.entries.clone(),
        })?;
        fs::write(&self.storage_path, raw)
    }
}

fn default_storage_path() -> PathBuf {
    let base = std::env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/state"))
        })
        .unwrap_or_else(std::env::temp_dir);
    base.join("codx").join("recent_files.json")
}
