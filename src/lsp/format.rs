use crate::app::{App, MessageKind};

impl App {
    pub(crate) fn format_document(&mut self) {
        let Some(path) = self.active_document().path().map(|path| path.to_path_buf()) else {
            return;
        };
        let current = self.active_document().text();
        if let Some(formatted) = self
            .lsp
            .format_document(&path, &self.workspace_root, &current)
        {
            let cursor = self.active_pane().cursor();
            if let Some(buffer) = self.buffer_by_id_mut(self.active_buffer_id) {
                buffer.document = crate::core::Document::from_text(Some(path), &formatted);
                buffer.document.set_dirty(true);
                buffer.syntax.mark_dirty();
            }
            if let Some(pane) = self.layout.focused_pane_mut() {
                pane.set_cursor(cursor);
                pane.set_selection(crate::core::Selection::caret(cursor));
            }
            self.ensure_cursor_visible();
            self.set_message("Document formatted", MessageKind::Info);
        } else {
            self.set_message("No formatting edits", MessageKind::Info);
        }
    }
}
