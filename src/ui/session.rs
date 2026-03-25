use ratatui::layout::Size;

use crate::app::{App, Message, MessageKind};

impl App {
    pub(crate) fn ensure_explorer_selection_visible(&mut self) {
        let terminal_size = self
            .layout
            .focused_pane()
            .map(|pane| pane.viewport().terminal_size())
            .unwrap_or_else(|| Size::new(120, 30));
        let workspace_height = terminal_size.height.saturating_sub(2);

        if self.explorer.visible() {
            let explorer_inner_height = workspace_height.saturating_sub(2) as usize;
            self.explorer.sync_scroll(explorer_inner_height);
        }
    }

    pub(crate) fn ensure_picker_selection_visible(&mut self) {
        let terminal_size = self
            .layout
            .focused_pane()
            .map(|pane| pane.viewport().terminal_size())
            .unwrap_or_else(|| Size::new(120, 30));
        let workspace_height = terminal_size.height.saturating_sub(2);

        if let Some(picker) = self.picker.as_mut() {
            let popup_height = workspace_height.saturating_mul(60) / 100;
            let picker_list_height = popup_height.saturating_sub(3) as usize;
            picker.sync_scroll(picker_list_height);
        }
    }

    pub(crate) fn ensure_cursor_visible(&mut self) {
        let pane_id = self.active_pane_id();
        let (line_count, cursor, display_column, text_height, text_width) =
            if let Some(pane) = self.layout.pane(pane_id) {
                let line_count = self.active_document().line_count();
                (
                    line_count,
                    pane.cursor(),
                    self.active_document().display_column(pane.cursor()),
                    pane.viewport().text_height(),
                    pane.viewport().text_width(line_count),
                )
            } else {
                return;
            };

        if let Some(pane) = self.layout.pane_mut(pane_id) {
            pane.viewport_mut().ensure_cursor_visible(
                cursor,
                display_column,
                line_count,
                text_height,
                text_width,
            );
        }
    }

    pub(crate) fn ensure_cursor_visible_minimal(&mut self) {
        let pane_id = self.active_pane_id();
        let (line_count, cursor, display_column, text_height, text_width) =
            if let Some(pane) = self.layout.pane(pane_id) {
                let line_count = self.active_document().line_count();
                (
                    line_count,
                    pane.cursor(),
                    self.active_document().display_column(pane.cursor()),
                    pane.viewport().text_height(),
                    pane.viewport().text_width(line_count),
                )
            } else {
                return;
            };

        if let Some(pane) = self.layout.pane_mut(pane_id) {
            pane.viewport_mut().ensure_cursor_visible_minimal(
                cursor,
                display_column,
                line_count,
                text_height,
                text_width,
            );
        }
    }

    pub(crate) fn set_message(&mut self, text: &str, kind: MessageKind) {
        self.message = Some(Message {
            text: text.to_owned(),
            kind,
        });
    }

    pub(crate) fn clear_message(&mut self) {
        self.message = None;
    }

    pub(crate) fn set_terminal_size(&mut self, size: Size) {
        for pane_id in self.layout.pane_ids() {
            if let Some(pane) = self.layout.pane_mut(pane_id) {
                pane.viewport_mut().set_terminal_size(size);
            }
        }
    }

    pub fn display_column(&self) -> usize {
        self.active_document()
            .display_column(self.active_pane().cursor())
    }

    pub fn is_read_only(&self) -> bool {
        self.active_document()
            .path()
            .and_then(|path| std::fs::metadata(path).ok())
            .map(|metadata| metadata.permissions().readonly())
            .unwrap_or(false)
    }
}
