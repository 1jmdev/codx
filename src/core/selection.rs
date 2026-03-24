use crate::core::Cursor;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Selection {
    anchor: Cursor,
    active: Cursor,
}

impl Selection {
    pub fn caret(cursor: Cursor) -> Self {
        Self {
            anchor: cursor,
            active: cursor,
        }
    }

    pub fn with_active(self, active: Cursor) -> Self {
        Self { active, ..self }
    }

    pub fn is_empty(&self) -> bool {
        self.anchor == self.active
    }

    pub fn normalized(&self) -> Option<(Cursor, Cursor)> {
        if self.is_empty() {
            None
        } else if self.anchor <= self.active {
            Some((self.anchor, self.active))
        } else {
            Some((self.active, self.anchor))
        }
    }

    pub fn contains(&self, line: usize, column: usize) -> bool {
        let Some((start, end)) = self.normalized() else {
            return false;
        };

        let position = Cursor::new(line, column);
        position >= start && position < end
    }

    pub fn starts_at(&self, line: usize, column: usize) -> bool {
        self.normalized()
            .map(|(start, _)| start.line == line && start.column == column)
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use crate::core::{Cursor, Selection};

    #[test]
    fn empty_selection_has_no_range() {
        let selection = Selection::caret(Cursor::new(1, 2));
        assert!(selection.normalized().is_none());
    }

    #[test]
    fn selection_normalizes_order() {
        let selection = Selection::caret(Cursor::new(3, 4)).with_active(Cursor::new(1, 2));
        let normalized = selection.normalized().unwrap_or_else(|| panic!("selection should not be empty"));
        assert_eq!(normalized.0, Cursor::new(1, 2));
        assert_eq!(normalized.1, Cursor::new(3, 4));
    }
}
