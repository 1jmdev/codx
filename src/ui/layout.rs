use ratatui::layout::Rect;

use crate::ui::{Pane, SplitDirection, WindowNode};

#[derive(Debug)]
pub struct LayoutState {
    root: WindowNode,
    focused_pane_id: u64,
    next_pane_id: u64,
}

impl LayoutState {
    pub fn new(initial_buffer_id: u64) -> Self {
        let pane = Pane::new(1, initial_buffer_id);
        Self {
            root: WindowNode::Leaf(pane),
            focused_pane_id: 1,
            next_pane_id: 2,
        }
    }

    pub fn focused_pane_id(&self) -> u64 {
        self.focused_pane_id
    }

    pub fn focused_pane(&self) -> Option<&Pane> {
        self.root.pane(self.focused_pane_id)
    }

    pub fn focused_pane_mut(&mut self) -> Option<&mut Pane> {
        self.root.pane_mut(self.focused_pane_id)
    }

    pub fn pane_mut(&mut self, pane_id: u64) -> Option<&mut Pane> {
        self.root.pane_mut(pane_id)
    }

    pub fn pane(&self, pane_id: u64) -> Option<&Pane> {
        self.root.pane(pane_id)
    }

    pub fn split_focused(&mut self, direction: SplitDirection, buffer_id: u64) -> Option<u64> {
        let pane_id = self.focused_pane_id;
        let new_pane_id = self.next_pane_id;
        self.next_pane_id += 1;
        let new_pane = Pane::new(new_pane_id, buffer_id);
        if self.root.split_leaf(pane_id, new_pane, direction) {
            self.focused_pane_id = new_pane_id;
            Some(new_pane_id)
        } else {
            None
        }
    }

    pub fn focus_next(&mut self) {
        let mut ids = Vec::new();
        self.root.collect_leaf_ids(&mut ids);
        if ids.is_empty() {
            return;
        }
        let index = ids
            .iter()
            .position(|pane_id| *pane_id == self.focused_pane_id)
            .unwrap_or(0);
        self.focused_pane_id = ids[(index + 1) % ids.len()];
    }

    pub fn resize_focused(&mut self, delta: i16) {
        let _ = self.root.resize_split(self.focused_pane_id, delta);
    }

    pub fn leaves_in_area(&self, area: Rect) -> Vec<(u64, Rect, &Pane)> {
        let mut output = Vec::new();
        self.root.collect_layout(area, &mut output);
        output
    }

    pub fn pane_ids(&self) -> Vec<u64> {
        let mut ids = Vec::new();
        self.root.collect_leaf_ids(&mut ids);
        ids
    }
}
