use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};

use crate::app::editor::line_len_chars;
use crate::app::{rect_contains, App, Focus};

impl App {
    pub(crate) fn on_mouse(&mut self, mouse: MouseEvent) {
        if !matches!(mouse.kind, MouseEventKind::Down(MouseButton::Left)) {
            return;
        }

        let x = mouse.column;
        let y = mouse.row;

        if rect_contains(self.ui.tree_inner, x, y) {
            self.handle_tree_click(y);
            return;
        }

        if rect_contains(self.ui.editor_inner, x, y) {
            self.handle_editor_click(x, y);
        }
    }

    fn handle_tree_click(&mut self, y: u16) {
        self.focus = Focus::FileTree;
        let clicked_row = (y - self.ui.tree_inner.y) as usize;
        let idx = self.tree_scroll + clicked_row;
        if idx >= self.tree_items.len() {
            return;
        }

        self.tree_selected = idx;
        if self.tree_items[idx].is_dir {
            self.toggle_selected_dir();
            return;
        }

        let path = self.tree_items[idx].path.clone();
        if let Err(error) = self.open_file(path) {
            self.status = format!("Open failed: {error}");
        }
    }

    fn handle_editor_click(&mut self, x: u16, y: u16) {
        self.focus = Focus::Editor;
        let clicked_row = (y - self.ui.editor_inner.y) as usize;
        let line = self.editor_scroll + clicked_row;
        if !self.lines.is_empty() {
            self.cursor_line = line.min(self.lines.len() - 1);
        }

        let number_width = self.line_number_width() + 1;
        let col = (x.saturating_sub(self.ui.editor_inner.x) as usize).saturating_sub(number_width);
        let max_col = line_len_chars(&self.lines[self.cursor_line]);
        self.cursor_col = col.min(max_col);
        self.preferred_col = self.cursor_col;
    }
}
