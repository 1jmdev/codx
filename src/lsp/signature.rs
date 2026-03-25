#[derive(Debug, Clone, Default)]
pub struct SignatureHelpView {
    pub label: String,
    pub active_parameter: Option<u32>,
    pub visible: bool,
}

impl SignatureHelpView {
    pub fn clear(&mut self) {
        self.label.clear();
        self.active_parameter = None;
        self.visible = false;
    }
}

use crate::app::App;

impl App {
    pub(crate) fn show_signature_help(&mut self) {
        let Some(path) = self.active_document().path().map(|path| path.to_path_buf()) else {
            return;
        };
        let cursor = self.active_pane().cursor();
        self.lsp
            .request_signature(&path, &self.workspace_root, cursor.line, cursor.column);
    }
}
