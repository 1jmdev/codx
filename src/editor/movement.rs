use crate::app::App;
use crate::core::{Cursor, Selection};

impl App {
    pub(crate) fn move_left(&mut self, extend: bool) {
        let target = self.active_document().previous_position(self.active_pane().cursor());
        self.update_cursor(target, extend);
    }

    pub(crate) fn move_right(&mut self, extend: bool) {
        let target = self.active_document().next_position(self.active_pane().cursor());
        self.update_cursor(target, extend);
    }

    pub(crate) fn move_up(&mut self, extend: bool) {
        let target = self
            .active_document()
            .move_vertically(self.active_pane().cursor(), -1);
        self.update_cursor(target, extend);
    }

    pub(crate) fn move_down(&mut self, extend: bool) {
        let target = self
            .active_document()
            .move_vertically(self.active_pane().cursor(), 1);
        self.update_cursor(target, extend);
    }

    pub(crate) fn move_line_start(&mut self, extend: bool) {
        let target = self.active_document().line_start(self.active_pane().cursor().line);
        self.update_cursor(target, extend);
    }

    pub(crate) fn move_line_end(&mut self, extend: bool) {
        let target = self.active_document().line_end(self.active_pane().cursor().line);
        self.update_cursor(target, extend);
    }

    pub(crate) fn move_word_left(&mut self, extend: bool) {
        let target = self.active_document().previous_word_start(self.active_pane().cursor());
        self.update_cursor(target, extend);
    }

    pub(crate) fn move_word_right(&mut self, extend: bool) {
        let target = self.active_document().next_word_start(self.active_pane().cursor());
        self.update_cursor(target, extend);
    }

    pub(crate) fn move_document_start(&mut self, extend: bool) {
        self.update_cursor(Cursor::default(), extend);
    }

    pub(crate) fn move_document_end(&mut self, extend: bool) {
        let last_line = self.active_document().last_line_index();
        let target = self.active_document().line_end(last_line);
        self.update_cursor(target, extend);
    }

    pub(crate) fn page_up(&mut self, extend: bool) {
        let page_height = self.active_pane().viewport().text_height().max(1);
        let target = self
            .active_document()
            .move_vertically(self.active_pane().cursor(), -(page_height as isize));
        self.update_cursor(target, extend);
    }

    pub(crate) fn page_down(&mut self, extend: bool) {
        let page_height = self.active_pane().viewport().text_height().max(1);
        let target = self
            .active_document()
            .move_vertically(self.active_pane().cursor(), page_height as isize);
        self.update_cursor(target, extend);
    }

    fn update_cursor(&mut self, target: Cursor, extend: bool) {
        let preferred = self.active_document().display_column(target);
        let selection = if extend {
            self.active_pane().selection().with_active(target.with_preferred_column(preferred))
        } else {
            Selection::caret(target.with_preferred_column(preferred))
        };
        {
            let pane = self.active_pane_mut();
            pane.set_cursor(target.with_preferred_column(preferred));
            pane.set_selection(selection);
        }
        self.ensure_cursor_visible();
    }
}
