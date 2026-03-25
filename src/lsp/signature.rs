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

use crate::app::{App, MessageKind};

impl App {
    pub(crate) fn show_signature_help(&mut self) {
        self.lsp.signature.clear();
        let Some(path) = self.active_document().path().map(|path| path.to_path_buf()) else {
            return;
        };
        let cursor = self.active_pane().cursor();
        if let Some((label, active)) =
            self.lsp
                .request_signature(&path, &self.workspace_root, cursor.line, cursor.column)
        {
            self.lsp.signature.visible = true;
            self.lsp.signature.label = label;
            self.lsp.signature.active_parameter = active;
        } else {
            self.set_message("No signature help", MessageKind::Info);
        }
    }
}
