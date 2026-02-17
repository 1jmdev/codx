use crate::app::App;

use super::line_len_chars;

impl App {
    pub(crate) fn undo(&mut self) {
        let Some(snapshot) = self.undo_stack.pop() else {
            return;
        };

        let current = self.capture_snapshot();
        self.redo_stack.push(current);
        self.restore_snapshot(snapshot);
        self.dirty = true;
        self.notify_lsp_change();
    }

    pub(crate) fn redo(&mut self) {
        let Some(snapshot) = self.redo_stack.pop() else {
            return;
        };

        let current = self.capture_snapshot();
        self.undo_stack.push(current);
        self.restore_snapshot(snapshot);
        self.dirty = true;
        self.notify_lsp_change();
    }

    pub(crate) fn begin_edit(&mut self) {
        self.undo_stack.push(self.capture_snapshot());
        self.redo_stack.clear();
    }

    pub(crate) fn mark_changed(&mut self) {
        self.dirty = true;
        self.notify_lsp_change();
    }

    fn capture_snapshot(&self) -> crate::app::state::EditorSnapshot {
        crate::app::state::EditorSnapshot {
            lines: self.lines.clone(),
            cursor_line: self.cursor_line,
            cursor_col: self.cursor_col,
            preferred_col: self.preferred_col,
            selection_anchor: self.selection_anchor,
        }
    }

    fn restore_snapshot(&mut self, snapshot: crate::app::state::EditorSnapshot) {
        self.lines = snapshot.lines;
        self.cursor_line = snapshot.cursor_line.min(self.lines.len().saturating_sub(1));
        self.cursor_col = snapshot
            .cursor_col
            .min(line_len_chars(&self.lines[self.cursor_line]));
        self.preferred_col = snapshot.preferred_col;
        self.selection_anchor = snapshot.selection_anchor;
    }
}
