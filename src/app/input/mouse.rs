use std::time::{Duration, Instant};

use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};

use crate::app::editor::line_len_chars;
use crate::app::state::{CursorPos, MouseSelectMode};
use crate::app::{App, Focus, rect_contains};

const DOUBLE_CLICK_WINDOW: Duration = Duration::from_millis(350);

impl App {
    pub(crate) fn on_mouse(&mut self, mouse: MouseEvent) {
        let x = mouse.column;
        let y = mouse.row;

        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                self.mouse_selecting = false;
                self.word_select_origin = None;
                self.mouse_select_mode = MouseSelectMode::Char;

                if rect_contains(self.ui.tree_inner, x, y) {
                    self.handle_tree_click(y);
                    self.last_left_click = Some((Instant::now(), x, y));
                    return;
                }

                if rect_contains(self.ui.editor_inner, x, y) {
                    if self.is_double_click(x, y) {
                        self.handle_editor_double_click(x, y);
                    } else {
                        self.handle_editor_press(x, y);
                    }
                }

                self.last_left_click = Some((Instant::now(), x, y));
            }
            MouseEventKind::Drag(MouseButton::Left) => {
                if self.mouse_selecting {
                    self.handle_editor_drag(x, y);
                }
            }
            MouseEventKind::Up(MouseButton::Left) => {
                if self.mouse_selecting {
                    self.mouse_selecting = false;
                    if !self.has_selection() {
                        self.clear_selection();
                    }
                }
            }
            MouseEventKind::ScrollUp => self.handle_mouse_scroll(-1, x, y),
            MouseEventKind::ScrollDown => self.handle_mouse_scroll(1, x, y),
            _ => {}
        }
    }

    fn handle_mouse_scroll(&mut self, delta_lines: isize, x: u16, y: u16) {
        if self.palette.is_some() && rect_contains(self.ui.palette_results, x, y) {
            self.move_palette_selection(delta_lines);
            return;
        }

        if rect_contains(self.ui.tree_inner, x, y) {
            self.focus = Focus::FileTree;
            self.scroll_tree_lines(delta_lines);
            return;
        }

        if rect_contains(self.ui.editor_inner, x, y) {
            self.focus = Focus::Editor;
            let view_height = self.ui.editor_inner.height as usize;
            let previous_scroll = self.editor_scroll;
            self.scroll_editor_lines(delta_lines, view_height);

            if !self.lines.is_empty() {
                let effective_delta = self.editor_scroll as isize - previous_scroll as isize;
                let max_line = self.lines.len().saturating_sub(1) as isize;
                let next_line = self.cursor_line as isize + effective_delta;
                self.cursor_line = next_line.clamp(0, max_line.max(0)) as usize;
                let max_col = line_len_chars(&self.lines[self.cursor_line]);
                self.cursor_col = self.cursor_col.min(max_col);
                self.preferred_col = self.cursor_col;
            }
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
        self.clear_selection();
        self.set_cursor_from_editor_point(x, y);
    }

    fn handle_editor_press(&mut self, x: u16, y: u16) {
        self.focus = Focus::Editor;
        self.handle_editor_click(x, y);
        self.selection_anchor = Some(self.cursor_pos());
        self.mouse_selecting = true;
        self.mouse_select_mode = MouseSelectMode::Char;
        self.word_select_origin = None;
    }

    fn handle_editor_double_click(&mut self, x: u16, y: u16) {
        self.focus = Focus::Editor;
        self.set_cursor_from_editor_point(x, y);
        let (start_col, end_col) = self.word_bounds_at_cursor(self.cursor_line, self.cursor_col);
        let start = CursorPos {
            line: self.cursor_line,
            col: start_col,
        };
        let end = CursorPos {
            line: self.cursor_line,
            col: end_col,
        };

        self.selection_anchor = Some(start);
        self.cursor_col = end.col;
        self.preferred_col = self.cursor_col;
        self.mouse_selecting = true;
        self.mouse_select_mode = MouseSelectMode::Word;
        self.word_select_origin = Some((start, end));
    }

    fn handle_editor_drag(&mut self, x: u16, y: u16) {
        if self.mouse_select_mode == MouseSelectMode::Word {
            self.handle_editor_word_drag(x, y);
            return;
        }

        if self.selection_anchor.is_none() {
            self.selection_anchor = Some(self.cursor_pos());
        }

        self.set_cursor_from_editor_point(x, y);
    }

    fn handle_editor_word_drag(&mut self, x: u16, y: u16) {
        let Some((origin_start, origin_end)) = self.word_select_origin else {
            self.set_cursor_from_editor_point(x, y);
            return;
        };

        self.set_cursor_from_editor_point(x, y);
        let (start_col, end_col) = self.word_bounds_at_cursor(self.cursor_line, self.cursor_col);
        let current_start = CursorPos {
            line: self.cursor_line,
            col: start_col,
        };
        let current_end = CursorPos {
            line: self.cursor_line,
            col: end_col,
        };

        if (current_start.line, current_start.col) >= (origin_start.line, origin_start.col) {
            self.selection_anchor = Some(origin_start);
            self.cursor_line = current_end.line;
            self.cursor_col = current_end.col;
        } else {
            self.selection_anchor = Some(origin_end);
            self.cursor_line = current_start.line;
            self.cursor_col = current_start.col;
        }
        self.preferred_col = self.cursor_col;
    }

    fn set_cursor_from_editor_point(&mut self, x: u16, y: u16) {
        if self.lines.is_empty() {
            self.cursor_line = 0;
            self.cursor_col = 0;
            self.preferred_col = 0;
            return;
        }

        let inner = self.ui.editor_inner;
        let max_y = inner.y.saturating_add(inner.height.saturating_sub(1));
        let clamped_y = y.clamp(inner.y, max_y);
        let clicked_row = (clamped_y - inner.y) as usize;
        let line = self.editor_scroll + clicked_row;
        self.cursor_line = line.min(self.lines.len() - 1);

        let number_width = self.line_number_width() + 1;
        let col = (x.saturating_sub(inner.x) as usize).saturating_sub(number_width);
        let max_col = line_len_chars(&self.lines[self.cursor_line]);
        self.cursor_col = col.min(max_col);
        self.preferred_col = self.cursor_col;
    }

    fn word_bounds_at_cursor(&self, line_idx: usize, col: usize) -> (usize, usize) {
        let Some(line) = self.lines.get(line_idx) else {
            return (0, 0);
        };

        let chars: Vec<char> = line.chars().collect();
        if chars.is_empty() {
            return (0, 0);
        }

        let mut idx = col.min(chars.len().saturating_sub(1));
        if col >= chars.len() && idx > 0 {
            idx = idx.saturating_sub(1);
        }

        let class = classify_char(chars[idx]);
        let mut start = idx;
        while start > 0 && classify_char(chars[start - 1]) == class {
            start -= 1;
        }

        let mut end = idx + 1;
        while end < chars.len() && classify_char(chars[end]) == class {
            end += 1;
        }

        (start, end)
    }

    fn is_double_click(&self, x: u16, y: u16) -> bool {
        let Some((time, lx, ly)) = self.last_left_click else {
            return false;
        };

        Instant::now().duration_since(time) <= DOUBLE_CLICK_WINDOW && lx == x && ly == y
    }
}

fn classify_char(ch: char) -> u8 {
    if ch.is_whitespace() {
        return 0;
    }
    if ch.is_alphanumeric() || ch == '_' {
        return 1;
    }
    2
}
