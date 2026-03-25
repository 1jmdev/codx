use crate::app::{App, MessageKind};

impl App {
    pub(crate) fn goto_definition(&mut self) {
        let Some(path) = self.active_document().path().map(|path| path.to_path_buf()) else {
            return;
        };
        let cursor = self.active_pane().cursor();
        if let Some((target_path, line, column)) =
            self.lsp
                .goto_definition(&path, &self.workspace_root, cursor.line, cursor.column)
        {
            if self.open_path_in_active_pane(&target_path).is_ok() {
                let c = crate::core::Cursor::new(line, column);
                if let Some(pane) = self.layout.focused_pane_mut() {
                    pane.set_cursor(c);
                    pane.set_selection(crate::core::Selection::caret(c));
                }
                self.ensure_cursor_visible();
            }
        } else {
            self.set_message("No definition found", MessageKind::Info);
        }
    }

    pub(crate) fn goto_references(&mut self) {
        let Some(path) = self.active_document().path().map(|path| path.to_path_buf()) else {
            return;
        };
        let cursor = self.active_pane().cursor();
        let refs =
            self.lsp
                .goto_references(&path, &self.workspace_root, cursor.line, cursor.column);
        if refs.is_empty() {
            self.set_message("No references found", MessageKind::Info);
            return;
        }
        let mut picker = crate::ui::PickerState::new(crate::ui::PickerKind::Files);
        picker.set_buffer_items(
            refs.into_iter()
                .map(|(p, l, c)| crate::ui::PickerItem {
                    title: format!("{}:{}", p.display(), l + 1),
                    subtitle: String::from("reference"),
                    path: Some(p),
                    buffer_id: None,
                    line: Some(l),
                    column: Some(c),
                })
                .collect(),
        );
        self.picker = Some(picker);
    }
}
