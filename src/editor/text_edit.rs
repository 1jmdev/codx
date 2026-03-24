use crate::app::App;
use crate::core::{Cursor, EditRecord, Selection};
use crate::syntax::compute_indent;
use tree_sitter::Point;

fn cursor_to_point(document: &crate::core::Document, cursor: Cursor) -> Point {
    let line = document.raw_line_text(cursor.line);
    let byte_col = line
        .chars()
        .take(cursor.column)
        .map(char::len_utf8)
        .sum::<usize>();
    Point {
        row: cursor.line,
        column: byte_col,
    }
}

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
        let line_end = self
            .active_document()
            .line_end_including_newline(cursor.line);
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

        let Some(buffer_index) = self
            .buffers
            .iter()
            .position(|buffer| buffer.id == buffer_id)
        else {
            return;
        };
        let (cursor_after, document_ref) = {
            let buffer = &mut self.buffers[buffer_index];
            let start_byte = buffer.document.cursor_to_byte(start);
            let old_end_byte = buffer.document.cursor_to_byte(end);
            let start_position = cursor_to_point(&buffer.document, start);
            let old_end_position = cursor_to_point(&buffer.document, end);
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
            buffer
                .document
                .set_dirty(buffer.document.text() != buffer.saved_snapshot);
            let new_end_position = cursor_to_point(&buffer.document, cursor_after);
            let new_end_byte = start_byte + inserted_text.len();
            buffer.syntax.apply_edit(
                start_byte,
                old_end_byte,
                new_end_byte,
                start_position,
                old_end_position,
                new_end_position,
            );
            (cursor_after, &buffer.document as *const _)
        };

        if let Some(pane) = self.layout.pane_mut(pane_id) {
            pane.set_cursor(cursor_after);
            pane.set_selection(Selection::caret(cursor_after));
            let document = unsafe { &*document_ref };
            pane.search_mut()
                .refresh_for_document(document, cursor_after);
        }
        if let Some(path) = self.active_document().path().map(|path| path.to_path_buf()) {
            let version = self.active_document().text().len() as i32;
            self.lsp.did_change(
                &path,
                &self.active_document().text(),
                version,
                &self.workspace_root,
            );
        }
        self.ensure_cursor_visible();
    }

    fn current_edit_range(&self) -> (Cursor, Cursor) {
        self.active_pane()
            .selection()
            .normalized()
            .unwrap_or((self.active_pane().cursor(), self.active_pane().cursor()))
    }

    pub(crate) fn insert_newline_with_indent(&mut self) {
        let cursor = self.active_pane().cursor();
        let buffer_id = self.active_buffer_id;

        let (current_indent, cursor_byte, tree_ref, next_char) = {
            let Some(buffer) = self.buffer_by_id(buffer_id) else {
                self.insert_text("\n", false);
                return;
            };

            let raw_line = buffer.document.raw_line_text(cursor.line);
            let leading: String = raw_line
                .chars()
                .take_while(|c| *c == ' ' || *c == '\t')
                .collect();

            let line_start_byte = buffer.document.line_to_byte(cursor.line);
            let line_chars: String = raw_line.chars().take(cursor.column).collect();
            let col_byte: usize = line_chars.len();
            let cb = line_start_byte + col_byte;

            let tree_ptr = buffer.syntax.tree().map(|t| t as *const _);
            let next = if cursor.column < buffer.document.line_text(cursor.line).chars().count() {
                buffer
                    .document
                    .raw_line_text(cursor.line)
                    .chars()
                    .nth(cursor.column)
            } else {
                None
            };

            (leading, cb, tree_ptr, next)
        };

        let new_indent = {
            let buffer = self.buffer_by_id(buffer_id);
            let source = buffer.map(|b| b.document.text()).unwrap_or_default();
            let tree = tree_ref.map(|ptr| unsafe { &*ptr });
            compute_indent(tree, source.as_bytes(), cursor_byte, &current_indent)
        };

        let is_closing = next_char
            .map(|c| matches!(c, '}' | ')' | ']'))
            .unwrap_or(false);

        if is_closing {
            let insert = format!("\n{}\n{}", new_indent, current_indent);
            self.insert_text(&insert, false);
            let new_cursor = self.active_pane().cursor();
            let target_line = new_cursor.line - 1;
            let col = new_indent.chars().count();
            let c = Cursor::new(target_line, col);
            if let Some(pane) = self.layout.pane_mut(self.active_pane_id()) {
                pane.set_cursor(c);
                pane.set_selection(crate::core::Selection::caret(c));
            }
            self.ensure_cursor_visible();
        } else {
            let insert = format!("\n{}", new_indent);
            self.insert_text(&insert, false);
        }
    }
}
