use crate::file::FinderItem;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PickerKind {
    Files,
    Buffers,
}

#[derive(Debug, Clone)]
pub struct PickerItem {
    pub title: String,
    pub subtitle: String,
    pub path: Option<std::path::PathBuf>,
    pub buffer_id: Option<u64>,
    pub line: Option<usize>,
    pub column: Option<usize>,
}

#[derive(Debug)]
pub struct PickerState {
    kind: PickerKind,
    query: String,
    items: Vec<PickerItem>,
    selected: usize,
}

impl PickerState {
    pub fn new(kind: PickerKind) -> Self {
        Self {
            kind,
            query: String::new(),
            items: Vec::new(),
            selected: 0,
        }
    }

    pub fn kind(&self) -> PickerKind {
        self.kind
    }

    pub fn query(&self) -> &str {
        &self.query
    }

    pub fn set_query(&mut self, query: String) {
        self.query = query;
        self.selected = 0;
    }

    pub fn items(&self) -> &[PickerItem] {
        &self.items
    }

    pub fn selected(&self) -> usize {
        self.selected
    }

    pub fn move_selection(&mut self, delta: isize) {
        if self.items.is_empty() {
            self.selected = 0;
            return;
        }
        let max_index = self.items.len().saturating_sub(1);
        self.selected = if delta.is_negative() {
            self.selected.saturating_sub(delta.unsigned_abs())
        } else {
            (self.selected + delta as usize).min(max_index)
        };
    }

    pub fn selected_item(&self) -> Option<&PickerItem> {
        self.items.get(self.selected)
    }

    pub fn set_file_items(&mut self, items: Vec<FinderItem>) {
        self.items = items
            .into_iter()
            .map(|item| PickerItem {
                title: item.display,
                subtitle: String::new(),
                path: Some(item.path),
                buffer_id: None,
                line: None,
                column: None,
            })
            .collect();
        self.selected = self.selected.min(self.items.len().saturating_sub(1));
    }

    pub fn set_buffer_items(&mut self, items: Vec<PickerItem>) {
        self.items = items;
        self.selected = self.selected.min(self.items.len().saturating_sub(1));
    }
}
