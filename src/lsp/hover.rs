#[derive(Debug, Clone, Default)]
pub struct HoverView {
    pub title: String,
    pub contents: String,
    pub line: usize,
    pub column: usize,
    pub visible: bool,
}

impl HoverView {
    pub fn clear(&mut self) {
        self.visible = false;
        self.title.clear();
        self.contents.clear();
    }
}

use crate::app::{App, MessageKind};

impl App {
    pub(crate) fn show_hover(&mut self) {
        self.lsp.hover.clear();
        let Some(path) = self.active_document().path().map(|path| path.to_path_buf()) else {
            return;
        };
        let cursor = self.active_pane().cursor();
        if let Some(contents) =
            self.lsp
                .request_hover(&path, &self.workspace_root, cursor.line, cursor.column)
        {
            self.lsp.hover.visible = true;
            self.lsp.hover.title = String::from("Hover");
            self.lsp.hover.contents = contents;
            self.lsp.hover.line = cursor.line;
            self.lsp.hover.column = cursor.column;
        } else {
            self.set_message("No hover information", MessageKind::Info);
        }
    }
}
