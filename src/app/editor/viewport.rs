use crate::app::App;

impl App {
    pub(crate) fn line_number_width(&self) -> usize {
        self.lines.len().max(1).to_string().len()
    }

    pub(crate) fn ensure_cursor_visible(&mut self, view_height: usize) {
        if view_height == 0 {
            return;
        }

        let top_trigger_row = view_height / 4;
        let bottom_trigger_row = (view_height * 3 / 4).min(view_height.saturating_sub(1));

        let top_trigger_line = self.editor_scroll + top_trigger_row;
        if self.cursor_line < top_trigger_line {
            self.editor_scroll = self.cursor_line.saturating_sub(top_trigger_row);
        }

        let bottom_trigger_line = self.editor_scroll + bottom_trigger_row;
        if self.cursor_line > bottom_trigger_line {
            self.editor_scroll = self.cursor_line.saturating_sub(bottom_trigger_row);
        }

        let max_scroll = self.lines.len().saturating_sub(view_height);
        self.editor_scroll = self.editor_scroll.min(max_scroll);
    }

    pub(crate) fn ensure_cursor_in_view(&mut self, view_height: usize) {
        if view_height == 0 {
            return;
        }

        if self.cursor_line < self.editor_scroll {
            self.editor_scroll = self.cursor_line;
        }

        let bottom = self.editor_scroll + view_height - 1;
        if self.cursor_line > bottom {
            self.editor_scroll = self.cursor_line + 1 - view_height;
        }

        let max_scroll = self.lines.len().saturating_sub(view_height);
        self.editor_scroll = self.editor_scroll.min(max_scroll);
    }

    pub(crate) fn scroll_editor_lines(&mut self, delta_lines: isize, view_height: usize) {
        if view_height == 0 {
            return;
        }

        let max_scroll = self.lines.len().saturating_sub(view_height) as isize;
        let next = self.editor_scroll as isize + delta_lines;
        self.editor_scroll = next.clamp(0, max_scroll.max(0)) as usize;
    }
}
