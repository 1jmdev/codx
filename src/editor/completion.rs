use crate::app::App;

impl App {
    pub(crate) fn trigger_completion(&mut self) {
        let Some(path) = self.active_document().path().map(|path| path.to_path_buf()) else {
            return;
        };
        let cursor = self.active_pane().cursor();
        let items =
            self.lsp
                .request_completion(&path, &self.workspace_root, cursor.line, cursor.column);
        self.lsp.completion.set_items(cursor.column, items);
    }

    pub(crate) fn accept_completion(&mut self) {
        let Some(item) = self.lsp.completion.selected_item().cloned() else {
            return;
        };
        let text = if item.is_snippet {
            crate::editor::snippet::expand_snippet_body(&item.insert_text)
        } else {
            item.insert_text
        };
        self.insert_text(&text, false);
        self.lsp.completion.close();
    }

    pub(crate) fn close_completion(&mut self) {
        self.lsp.completion.close();
    }

    pub(crate) fn completion_active(&self) -> bool {
        self.lsp.completion.active
    }
}
