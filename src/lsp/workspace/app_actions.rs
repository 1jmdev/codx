use crate::app::{App, MessageKind};

impl App {
    pub(crate) fn open_workspace_symbols(&mut self) {
        self.command_bar.input.clear();
        self.mode = crate::app::AppMode::CommandBar(crate::app::CommandBarMode::WorkspaceSymbols);
    }

    pub(crate) fn open_workspace_symbols_query(&mut self, query: &str) {
        self.lsp
            .open_workspace_symbols(query, &self.workspace_root, &mut self.picker);
        if self.picker.is_none() {
            self.set_message("No workspace symbols found", MessageKind::Info);
        }
    }

    pub(crate) fn toggle_diagnostics_panel(&mut self) {
        self.lsp.toggle_diagnostics_panel();
        let status = if self.lsp.diagnostics_panel_open {
            "opened"
        } else {
            "closed"
        };
        self.set_message(&format!("Diagnostics panel {status}"), MessageKind::Info);
    }
}
