#[derive(Debug, Clone, Default)]
pub struct CompletionItemView {
    pub label: String,
    pub detail: String,
    pub insert_text: String,
    pub is_snippet: bool,
}

#[derive(Debug, Clone, Default)]
pub struct CompletionContext {
    pub active: bool,
    pub trigger_column: usize,
    pub items: Vec<CompletionItemView>,
    pub selected: usize,
    pub filter: String,
}

impl CompletionContext {
    pub fn close(&mut self) {
        self.active = false;
        self.items.clear();
        self.selected = 0;
        self.filter.clear();
    }

    pub fn set_items(&mut self, trigger_column: usize, mut items: Vec<CompletionItemView>) {
        items.sort_by(|left, right| left.label.cmp(&right.label));
        self.active = !items.is_empty();
        self.trigger_column = trigger_column;
        self.items = items;
        self.selected = 0;
    }

    pub fn selected_item(&self) -> Option<&CompletionItemView> {
        self.items.get(self.selected)
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
}
