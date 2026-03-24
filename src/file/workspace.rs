use std::path::Path;

use crate::app::{App, AppError, AppMode, CommandBarMode, FocusTarget, MessageKind};
use crate::core::History;
use crate::syntax::language_for_path;

impl App {
    pub(crate) fn request_quit(&mut self) {
        if self.buffers.iter().any(|buffer| buffer.document.is_dirty()) {
            self.mode = AppMode::ConfirmQuit;
            self.set_message(
                "Unsaved changes: y quit, n cancel, s save",
                MessageKind::Warning,
            );
        } else {
            self.should_quit = true;
        }
    }

    pub(crate) fn save_or_prompt(&mut self) -> Result<(), AppError> {
        let path = self.active_document().path().map(Path::to_path_buf);
        if let Some(path) = path {
            self.save_to_path(&path)
        } else {
            self.begin_save_as_prompt();
            Ok(())
        }
    }

    pub(crate) fn save_to_path(&mut self, path: &Path) -> Result<(), AppError> {
        let saved_text = {
            let buffer = self
                .buffer_by_id_mut(self.active_buffer_id)
                .ok_or_else(|| AppError::Invariant(String::from("active buffer is missing")))?;
            crate::file::save_document(path, &buffer.document, buffer.encoding)?;
            buffer.document.mark_saved(path.to_path_buf());
            buffer.saved_snapshot = buffer.document.text();
            buffer.document.text()
        };
        self.recent_files.record(path);
        self.file_finder.refresh();
        self.explorer.refresh();

        if self.pending_quit_after_save {
            self.should_quit = true;
        } else {
            self.set_message(&format!("Saved {}", path.display()), MessageKind::Info);
        }
        self.lsp.did_save(path, &saved_text, &self.workspace_root);
        self.pending_quit_after_save = false;
        Ok(())
    }

    pub(crate) fn open_path_in_active_pane(&mut self, path: &Path) -> Result<(), AppError> {
        let buffer_id = if let Some(existing) = self
            .buffers
            .iter()
            .find(|buffer| buffer.document.path().is_some_and(|item| item == path))
            .map(|buffer| buffer.id)
        {
            existing
        } else {
            let loaded = crate::file::load_document(path)?;
            let saved_snapshot = loaded.document.text();
            self.push_buffer(
                loaded.document,
                History::default(),
                saved_snapshot,
                loaded.encoding,
            )
        };
        self.switch_to_buffer(buffer_id);
        self.recent_files.record(path);
        if let Some(text) = self
            .buffer_by_id(buffer_id)
            .map(|buffer| buffer.document.text())
        {
            self.lsp.did_open(path, &text, &self.workspace_root);
        }
        Ok(())
    }

    pub(crate) fn poll_background_tasks(&mut self) {
        self.lsp.poll_server_messages();
        let watched = match self.watcher.as_mut() {
            Some(watcher) => watcher.poll_paths(),
            None => return,
        };
        if watched.is_empty() {
            return;
        }

        let mut need_refresh = false;

        for path in watched {
            let buffer_index = self
                .buffers
                .iter()
                .position(|b| b.document.path().is_some_and(|p| p == path));

            match buffer_index {
                None => {
                    // Not open — refresh explorer/finder silently
                    need_refresh = true;
                }
                Some(idx) if !self.buffers[idx].document.is_dirty() => {
                    // Open but clean — silently reload
                    if let Ok(loaded) = crate::file::load_document(&path) {
                        self.buffers[idx].document = loaded.document;
                        self.buffers[idx].encoding = loaded.encoding;
                        self.buffers[idx].saved_snapshot = self.buffers[idx].document.text();
                        let language_id = self.buffers[idx]
                            .document
                            .path()
                            .and_then(language_for_path);
                        let _ = self.buffers[idx].syntax.set_language_id(language_id);
                        self.buffers[idx].syntax.mark_dirty();
                    }
                    need_refresh = true;
                }
                Some(idx) => {
                    // Open AND dirty — only conflict if on-disk content differs from saved snapshot
                    let on_disk_differs = crate::file::load_document(&path)
                        .map(|loaded| loaded.document.text() != self.buffers[idx].saved_snapshot)
                        .unwrap_or(false);
                    if on_disk_differs && !self.pending_conflict_paths.contains(&path) {
                        self.pending_conflict_paths.push(path);
                    }
                }
            }
        }

        if need_refresh {
            self.explorer.refresh();
            self.file_finder.refresh();
        }

        if !self.pending_conflict_paths.is_empty() && matches!(self.mode, AppMode::Editing) {
            self.mode = AppMode::ExternalChangeConflict;
        }
    }

    pub(crate) fn toggle_explorer(&mut self) {
        self.explorer.toggle();
        self.focus = if self.explorer.visible() {
            FocusTarget::Explorer
        } else {
            FocusTarget::Editor
        };
    }

    pub(crate) fn open_selected_explorer_entry(&mut self) -> Result<(), AppError> {
        let Some(entry) = self.explorer.selected_entry().cloned() else {
            return Ok(());
        };

        if entry.is_dir {
            self.explorer.toggle_selected_expansion();
            return Ok(());
        }

        self.open_path_in_active_pane(&entry.path)
    }

    pub(crate) fn begin_explorer_create_file(&mut self) {
        self.command_bar.input.clear();
        self.mode = AppMode::CommandBar(CommandBarMode::ExplorerCreateFile);
    }

    pub(crate) fn begin_explorer_create_directory(&mut self) {
        self.command_bar.input.clear();
        self.mode = AppMode::CommandBar(CommandBarMode::ExplorerCreateDirectory);
    }

    pub(crate) fn begin_explorer_rename(&mut self) {
        self.command_bar.input = self
            .explorer
            .selected_entry()
            .and_then(|entry| entry.path.strip_prefix(self.workspace_root()).ok())
            .map(|path| path.display().to_string())
            .unwrap_or_default();
        self.mode = AppMode::CommandBar(CommandBarMode::ExplorerRename);
    }

    pub(crate) fn delete_selected_explorer_entry(&mut self) -> Result<(), AppError> {
        if let Some(removed) = self.explorer.delete_selected()? {
            self.buffers
                .retain(|buffer| buffer.document.path().is_none_or(|path| path != removed));
            self.ensure_nonempty_buffer_set();
            self.file_finder.refresh();
            self.set_message("Explorer entry deleted", MessageKind::Info);
        }
        Ok(())
    }
}
