#[derive(Debug, Clone, Default)]
pub struct RenameRequest {
    pub new_name: String,
}

use crate::app::{App, MessageKind};

impl App {
    pub(crate) fn rename_symbol(&mut self) {
        self.command_bar.input.clear();
        self.mode = crate::app::AppMode::CommandBar(crate::app::CommandBarMode::LspRename);
    }

    pub(crate) fn apply_lsp_rename(&mut self, new_name: &str) {
        let Some(path) = self.active_document().path().map(|path| path.to_path_buf()) else {
            return;
        };
        let cursor = self.active_pane().cursor();
        let edits = self.lsp.rename(
            &path,
            &self.workspace_root,
            cursor.line,
            cursor.column,
            new_name,
        );
        if edits.is_empty() {
            self.set_message("Rename returned no edits", MessageKind::Info);
            return;
        }
        for (edit_path, text) in edits {
            let document = crate::core::Document::from_text(Some(edit_path.clone()), &text);
            if let Some(index) = self
                .buffers
                .iter()
                .position(|buffer| buffer.document.path().is_some_and(|p| p == edit_path))
            {
                self.buffers[index].document = document;
                self.buffers[index].saved_snapshot = self.buffers[index].document.text();
                self.buffers[index].document.set_dirty(true);
                continue;
            }
            let _ = self.push_buffer(
                document,
                crate::core::History::default(),
                String::new(),
                crate::util::DetectedEncoding::default(),
            );
        }
        self.set_message("Rename applied", MessageKind::Info);
    }
}
