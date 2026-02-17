use crate::app::App;

impl App {
    pub(super) fn reload_lsp_server(&mut self) {
        let Some(path) = self.current_file.clone() else {
            self.status = String::from("Open a file first to reload LSP.");
            return;
        };

        self.lsp
            .reload_for_file(&path, self.lines.join("\n"), &mut self.status);
    }
}
