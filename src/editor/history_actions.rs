use crate::app::{App, MessageKind};
use crate::core::Selection;

impl App {
    pub(crate) fn undo(&mut self) {
        self.apply_history_step(true);
    }

    pub(crate) fn redo(&mut self) {
        self.apply_history_step(false);
    }

    fn apply_history_step(&mut self, undo: bool) {
        let pane_id = self.active_pane_id();
        let buffer_id = self.active_buffer_id;
        let Some(buffer_index) = self
            .buffers
            .iter()
            .position(|buffer| buffer.id == buffer_id)
        else {
            self.set_message(
                if undo {
                    "Nothing to undo"
                } else {
                    "Nothing to redo"
                },
                MessageKind::Info,
            );
            return;
        };

        let cursor = {
            let buffer = &mut self.buffers[buffer_index];
            let cursor = if undo {
                buffer.history.undo(&mut buffer.document)
            } else {
                buffer.history.redo(&mut buffer.document)
            };
            cursor.map(|cursor| {
                let preferred = buffer.document.display_column(cursor);
                buffer
                    .document
                    .set_dirty(buffer.document.text() != buffer.saved_snapshot);
                cursor.with_preferred_column(preferred)
            })
        };

        let Some(cursor) = cursor else {
            self.set_message(
                if undo {
                    "Nothing to undo"
                } else {
                    "Nothing to redo"
                },
                MessageKind::Info,
            );
            return;
        };

        if let Some(pane) = self.layout.pane_mut(pane_id) {
            pane.set_cursor(cursor);
            pane.set_selection(Selection::caret(pane.cursor()));
        }

        let document = &self.buffers[buffer_index].document;
        if let Some(pane) = self.layout.pane_mut(pane_id) {
            let pane_cursor = pane.cursor();
            pane.search_mut()
                .refresh_for_document(document, pane_cursor);
        }
    }
}
