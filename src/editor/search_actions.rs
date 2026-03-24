use crate::app::App;
use crate::core::{Cursor, Selection};
use crate::editor::SearchMatch;

impl App {
    pub(crate) fn search_next(&mut self) {
        let next_match = self.active_pane_mut().search_mut().select_next();
        if let Some(item) = next_match {
            self.move_cursor_to_match(item);
        }
    }

    pub(crate) fn search_previous(&mut self) {
        let previous = self.active_pane_mut().search_mut().select_previous();
        if let Some(item) = previous {
            self.move_cursor_to_match(item);
        }
    }

    pub(crate) fn jump_to_active_search_match(&mut self) {
        let active = self.active_pane().search().active_match();
        if let Some(item) = active {
            self.move_cursor_to_match(item);
        }
    }

    fn move_cursor_to_match(&mut self, item: SearchMatch) {
        let target = Cursor::new(item.line, item.start_column);
        let preferred = self.active_document().display_column(target);
        {
            let pane = self.active_pane_mut();
            pane.set_cursor(target.with_preferred_column(preferred));
            pane.set_selection(Selection::caret(pane.cursor()));
        }
        self.ensure_cursor_visible();
    }
}
