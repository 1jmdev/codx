use crate::app::App;
use crate::app::editor::{byte_index_for_char, line_len_chars};

impl App {
    pub(super) fn apply_selected_completion(&mut self) {
        let Some(state) = self.completion.as_ref() else {
            return;
        };
        let Some(item) = state.items.get(state.selected).cloned() else {
            self.completion = None;
            return;
        };

        self.begin_edit();
        self.delete_selection_inner();

        let line = &mut self.lines[self.cursor_line];
        let start_col = item.replace_start_col.min(line_len_chars(line));
        let end_col = item
            .replace_end_col
            .min(line_len_chars(line))
            .max(start_col);
        let start_byte = byte_index_for_char(line, start_col);
        let end_byte = byte_index_for_char(line, end_col);
        line.replace_range(start_byte..end_byte, &item.insert_text);

        self.cursor_col = start_col + item.insert_text.chars().count();
        self.preferred_col = self.cursor_col;
        self.selection_anchor = None;
        self.completion = None;
        self.mark_changed();
    }
}
