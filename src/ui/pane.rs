use crate::core::{Cursor, Selection};
use crate::editor::SearchState;
use crate::view::Viewport;

#[derive(Debug)]
pub struct Pane {
    id: u64,
    buffer_id: u64,
    cursor: Cursor,
    selection: Selection,
    viewport: Viewport,
    search: SearchState,
}

impl Pane {
    pub fn new(id: u64, buffer_id: u64) -> Self {
        Self {
            id,
            buffer_id,
            cursor: Cursor::default(),
            selection: Selection::caret(Cursor::default()),
            viewport: Viewport::default(),
            search: SearchState::default(),
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn buffer_id(&self) -> u64 {
        self.buffer_id
    }

    pub fn set_buffer_id(&mut self, buffer_id: u64) {
        self.buffer_id = buffer_id;
    }

    pub fn cursor(&self) -> Cursor {
        self.cursor
    }

    pub fn set_cursor(&mut self, cursor: Cursor) {
        self.cursor = cursor;
    }

    pub fn selection(&self) -> Selection {
        self.selection
    }

    pub fn set_selection(&mut self, selection: Selection) {
        self.selection = selection;
    }

    pub fn viewport(&self) -> &Viewport {
        &self.viewport
    }

    pub fn viewport_mut(&mut self) -> &mut Viewport {
        &mut self.viewport
    }

    pub fn search(&self) -> &SearchState {
        &self.search
    }

    pub fn search_mut(&mut self) -> &mut SearchState {
        &mut self.search
    }
}
