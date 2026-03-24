use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver};

use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};

pub struct FileWatcher {
    receiver: Receiver<notify::Result<Event>>,
    watcher: RecommendedWatcher,
}

impl FileWatcher {
    pub fn new(root: &Path) -> Result<Self, notify::Error> {
        let (sender, receiver) = mpsc::channel();
        let mut watcher = RecommendedWatcher::new(sender, Config::default())?;
        watcher.watch(root, RecursiveMode::Recursive)?;
        Ok(Self { receiver, watcher })
    }

    pub fn poll_paths(&mut self) -> Vec<PathBuf> {
        let _ = &self.watcher;
        let mut paths = Vec::new();
        while let Ok(event) = self.receiver.try_recv() {
            if let Ok(event) = event {
                paths.extend(event.paths);
            }
        }
        paths
    }
}
