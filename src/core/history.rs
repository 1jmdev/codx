use crate::core::{Cursor, Document};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditKind {
    Insert,
    Delete,
    Replace,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditRecord {
    pub kind: EditKind,
    pub start: Cursor,
    pub inserted_text: String,
    pub deleted_text: String,
    pub cursor_before: Cursor,
    pub cursor_after: Cursor,
}

impl EditRecord {
    pub fn new(
        start: Cursor,
        inserted_text: String,
        deleted_text: String,
        cursor_before: Cursor,
        cursor_after: Cursor,
    ) -> Self {
        let kind = match (inserted_text.is_empty(), deleted_text.is_empty()) {
            (false, true) => EditKind::Insert,
            (true, false) => EditKind::Delete,
            (false, false) => EditKind::Replace,
            (true, true) => EditKind::Delete,
        };

        Self {
            kind,
            start,
            inserted_text,
            deleted_text,
            cursor_before,
            cursor_after,
        }
    }

    fn inserted_end(&self, document: &Document) -> Cursor {
        document.advance_cursor(self.start, &self.inserted_text)
    }

    fn deleted_end(&self, document: &Document) -> Cursor {
        document.advance_cursor(self.start, &self.deleted_text)
    }

    pub fn undo(&self, document: &mut Document) -> Cursor {
        if !self.inserted_text.is_empty() {
            document.delete_range(self.start, self.inserted_end(document));
        }

        if !self.deleted_text.is_empty() {
            let _ = document.insert_text(self.start, &self.deleted_text);
        }

        self.cursor_before
    }

    pub fn redo(&self, document: &mut Document) -> Cursor {
        if !self.deleted_text.is_empty() {
            document.delete_range(self.start, self.deleted_end(document));
        }

        if !self.inserted_text.is_empty() {
            let _ = document.insert_text(self.start, &self.inserted_text);
        }

        self.cursor_after
    }
}

#[derive(Debug, Default)]
pub struct History {
    undo_stack: Vec<EditRecord>,
    redo_stack: Vec<EditRecord>,
    coalescing_active: bool,
}

impl History {
    pub fn push_edit(&mut self, record: EditRecord, coalesce: bool) {
        if coalesce && self.coalescing_active && self.try_coalesce_insert(&record) {
            self.redo_stack.clear();
            return;
        }

        self.undo_stack.push(record);
        self.redo_stack.clear();
        self.coalescing_active = coalesce;
    }

    pub fn undo(&mut self, document: &mut Document) -> Option<Cursor> {
        let record = self.undo_stack.pop()?;
        let cursor = record.undo(document);
        self.redo_stack.push(record);
        self.coalescing_active = false;
        Some(cursor)
    }

    pub fn redo(&mut self, document: &mut Document) -> Option<Cursor> {
        let record = self.redo_stack.pop()?;
        let cursor = record.redo(document);
        self.undo_stack.push(record);
        self.coalescing_active = false;
        Some(cursor)
    }

    fn try_coalesce_insert(&mut self, record: &EditRecord) -> bool {
        if record.kind != EditKind::Insert {
            return false;
        }

        let Some(previous) = self.undo_stack.last_mut() else {
            return false;
        };

        if previous.kind != EditKind::Insert || previous.cursor_after != record.cursor_before {
            return false;
        }

        previous.inserted_text.push_str(&record.inserted_text);
        previous.cursor_after = record.cursor_after;
        true
    }
}

#[cfg(test)]
mod tests {
    use crate::core::{Cursor, Document, EditRecord, History};

    #[test]
    fn coalesces_adjacent_insertions() {
        let mut history = History::default();
        history.push_edit(
            EditRecord::new(
                Cursor::new(0, 0),
                String::from("a"),
                String::new(),
                Cursor::new(0, 0),
                Cursor::new(0, 1),
            ),
            true,
        );
        history.push_edit(
            EditRecord::new(
                Cursor::new(0, 1),
                String::from("b"),
                String::new(),
                Cursor::new(0, 1),
                Cursor::new(0, 2),
            ),
            true,
        );

        let mut document = Document::from_text(None, "ab");
        let cursor = history.undo(&mut document).unwrap_or_default();
        assert_eq!(document.text(), "");
        assert_eq!(cursor, Cursor::new(0, 0));
    }

    #[test]
    fn redo_is_cleared_by_new_edit() {
        let mut history = History::default();
        let record = EditRecord::new(
            Cursor::new(0, 0),
            String::from("a"),
            String::new(),
            Cursor::new(0, 0),
            Cursor::new(0, 1),
        );
        history.push_edit(record, false);
        let mut document = Document::from_text(None, "a");
        let _ = history.undo(&mut document);
        history.push_edit(
            EditRecord::new(
                Cursor::new(0, 0),
                String::from("b"),
                String::new(),
                Cursor::new(0, 0),
                Cursor::new(0, 1),
            ),
            false,
        );
        assert!(history.redo(&mut document).is_none());
    }
}
