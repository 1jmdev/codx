use crate::app::App;
use crate::core::{Cursor, EditRecord, Selection};

impl App {
    pub(crate) fn insert_text(&mut self, text: &str, allow_coalesce: bool) {
        let selection = self.active_pane().selection();
        let coalesce = allow_coalesce
            && selection.normalized().is_none()
            && text.chars().count() == 1
            && text != "\n"
            && text != "\t";
        let (start, end) = self.current_edit_range();
        self.apply_edit(start, end, text, coalesce);
    }

    pub(crate) fn backspace(&mut self) {
        if let Some((start, end)) = self.active_pane().selection().normalized() {
            self.apply_edit(start, end, "", false);
            return;
        }

        let cursor = self.active_pane().cursor();
        let previous = self.active_document().previous_position(cursor);
        if previous != cursor {
            self.apply_edit(previous, cursor, "", false);
        }
    }

    pub(crate) fn delete_forward(&mut self) {
        if let Some((start, end)) = self.active_pane().selection().normalized() {
            self.apply_edit(start, end, "", false);
            return;
        }

        let cursor = self.active_pane().cursor();
        let next = self.active_document().next_position(cursor);
        if next != cursor {
            self.apply_edit(cursor, next, "", false);
        }
    }

    pub(crate) fn delete_word_backward(&mut self) {
        if let Some((start, end)) = self.active_pane().selection().normalized() {
            self.apply_edit(start, end, "", false);
            return;
        }

        let cursor = self.active_pane().cursor();
        let word_start = self.active_document().previous_word_start(cursor);
        if word_start != cursor {
            self.apply_edit(word_start, cursor, "", false);
        }
    }

    pub(crate) fn delete_to_end_of_line(&mut self) {
        if let Some((start, end)) = self.active_pane().selection().normalized() {
            self.apply_edit(start, end, "", false);
            return;
        }

        let cursor = self.active_pane().cursor();
        let line_end = self.active_document().line_end_including_newline(cursor.line);
        if line_end != cursor {
            self.apply_edit(cursor, line_end, "", false);
        }
    }

    pub(crate) fn selection_text(&self) -> Option<String> {
        self.active_pane()
            .selection()
            .normalized()
            .map(|(start, end)| self.active_document().slice_string(start, end))
    }

    pub(crate) fn apply_edit(
        &mut self,
        start: Cursor,
        end: Cursor,
        inserted_text: &str,
        coalesce: bool,
    ) {
        let pane_id = self.active_pane_id();
        let buffer_id = self.active_buffer_id;
        let cursor_before = self.active_pane().cursor();

        let Some(buffer_index) = self.buffers.iter().position(|buffer| buffer.id == buffer_id) else {
            return;
        };
        let (cursor_after, document_ref) = {
            let buffer = &mut self.buffers[buffer_index];
            let deleted_text = buffer.document.slice_string(start, end);
            if deleted_text.is_empty() && inserted_text.is_empty() {
                return;
            }

            buffer.document.delete_range(start, end);
            let new_cursor = if inserted_text.is_empty() {
                start
            } else {
                buffer.document.insert_text(start, inserted_text)
            };
            let cursor_after =
                new_cursor.with_preferred_column(buffer.document.display_column(new_cursor));

            buffer.history.push_edit(
                EditRecord::new(
                    start,
                    inserted_text.to_owned(),
                    deleted_text,
                    cursor_before,
                    cursor_after,
                ),
                coalesce,
            );
            buffer.document.set_dirty(buffer.document.text() != buffer.saved_snapshot);
            (cursor_after, &buffer.document as *const _)
        };

        if let Some(pane) = self.layout.pane_mut(pane_id) {
            pane.set_cursor(cursor_after);
            pane.set_selection(Selection::caret(cursor_after));
            let document = unsafe { &*document_ref };
            pane.search_mut().refresh_for_document(document, cursor_after);
        }
        self.ensure_cursor_visible();
    }

    fn current_edit_range(&self) -> (Cursor, Cursor) {
        self.active_pane()
            .selection()
            .normalized()
            .unwrap_or((self.active_pane().cursor(), self.active_pane().cursor()))
    }
}
