#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Cursor {
    pub line: usize,
    pub column: usize,
    pub preferred_column: usize,
}

impl Cursor {
    pub fn new(line: usize, column: usize) -> Self {
        Self {
            line,
            column,
            preferred_column: column,
        }
    }

    pub fn with_preferred_column(self, preferred_column: usize) -> Self {
        Self {
            preferred_column,
            ..self
        }
    }
}
