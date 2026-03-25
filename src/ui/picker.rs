use crate::file::FinderItem;
use crate::util::compute_scroll_offset;

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
    scroll_offset: usize,
}

impl PickerState {
    pub fn new(kind: PickerKind) -> Self {
        Self {
            kind,
            query: String::new(),
            items: Vec::new(),
            selected: 0,
            scroll_offset: 0,
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
        self.scroll_offset = 0;
    }

    pub fn items(&self) -> &[PickerItem] {
        &self.items
    }

    pub fn selected(&self) -> usize {
        self.selected
    }

    pub fn set_selected(&mut self, selected: usize) {
        self.selected = selected.min(self.items.len().saturating_sub(1));
    }

    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    pub fn sync_scroll(&mut self, viewport_height: usize) {
        self.scroll_offset = compute_scroll_offset(
            self.scroll_offset,
            self.selected,
            self.items.len(),
            viewport_height,
        );
    }

    pub fn scroll_by(&mut self, delta: isize, viewport_height: usize) {
        if self.items.is_empty() || viewport_height == 0 {
            self.scroll_offset = 0;
            return;
        }

        let max_offset = self.items.len().saturating_sub(viewport_height);
        self.scroll_offset = if delta.is_negative() {
            self.scroll_offset.saturating_sub(delta.unsigned_abs())
        } else {
            (self.scroll_offset + delta as usize).min(max_offset)
        };
    }

    pub fn move_selection(&mut self, delta: isize) {
        if self.items.is_empty() {
            self.selected = 0;
            self.scroll_offset = 0;
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
        self.scroll_offset = self.scroll_offset.min(self.selected);
    }

    pub fn set_buffer_items(&mut self, items: Vec<PickerItem>) {
        self.items = items;
        self.selected = self.selected.min(self.items.len().saturating_sub(1));
        self.scroll_offset = self.scroll_offset.min(self.selected);
    }
}
