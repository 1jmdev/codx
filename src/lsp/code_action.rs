use crate::app::{App, MessageKind};

impl App {
    pub(crate) fn show_code_actions(&mut self) {
        let Some(path) = self.active_document().path().map(|path| path.to_path_buf()) else {
            return;
        };
        let cursor = self.active_pane().cursor();
        let actions =
            self.lsp
                .request_code_actions(&path, &self.workspace_root, cursor.line, cursor.column);
        if actions.is_empty() {
            self.set_message("No code actions", MessageKind::Info);
            return;
        }
        let mut picker = crate::ui::PickerState::new(crate::ui::PickerKind::Files);
        picker.set_buffer_items(
            actions
                .into_iter()
                .map(|title| crate::ui::PickerItem {
                    title,
                    subtitle: String::from("code action"),
                    path: None,
                    buffer_id: None,
                    line: None,
                    column: None,
                })
                .collect(),
        );
        self.picker = Some(picker);
    }
}
