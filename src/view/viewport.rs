use ratatui::layout::Size;

use crate::core::Cursor;

const STATUSLINE_HEIGHT: u16 = 1;
const MESSAGE_HEIGHT: u16 = 1;
const PANE_BORDER_HEIGHT: u16 = 2;
const GUTTER_PADDING: u16 = 1;

#[derive(Debug, Clone)]
pub struct Viewport {
    top_line: usize,
    left_column: usize,
    size: Size,
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            top_line: 0,
            left_column: 0,
            size: Size::new(120, 30),
        }
    }
}

impl Viewport {
    pub fn top_line(&self) -> usize {
        self.top_line
    }

    pub fn left_column(&self) -> usize {
        self.left_column
    }

    pub fn set_terminal_size(&mut self, size: Size) {
        self.size = size;
    }

    pub fn text_height(&self) -> usize {
        self.size
            .height
            .saturating_sub(STATUSLINE_HEIGHT + MESSAGE_HEIGHT + PANE_BORDER_HEIGHT)
            .max(1) as usize
    }

    pub fn text_width(&self, line_count: usize) -> usize {
        self.size
            .width
            .saturating_sub(gutter_width(line_count) + GUTTER_PADDING) as usize
    }

    pub fn ensure_cursor_visible(
        &mut self,
        cursor: Cursor,
        display_column: usize,
        line_count: usize,
        text_height: usize,
        text_width: usize,
    ) {
        let scrolloff = (text_height / 4).max(1);

        if cursor.line < self.top_line + scrolloff {
            self.top_line = cursor.line.saturating_sub(scrolloff);
        } else if cursor.line + scrolloff + 1 > self.top_line + text_height {
            self.top_line = (cursor.line + scrolloff + 1).saturating_sub(text_height);
        }

        if display_column < self.left_column {
            self.left_column = display_column;
        } else if display_column >= self.left_column + text_width {
            self.left_column = display_column.saturating_sub(text_width.saturating_sub(1));
        }

        let max_top = line_count.saturating_sub(text_height);
        if self.top_line > max_top {
            self.top_line = max_top;
        }
    }
}

fn gutter_width(line_count: usize) -> u16 {
    let digits = line_count.max(1).to_string().chars().count() as u16;
    digits.saturating_add(2).max(5)
}

#[cfg(test)]
mod tests {
    use crate::core::Cursor;
    use crate::view::Viewport;

    #[test]
    fn moves_viewport_to_follow_cursor() {
        let mut viewport = Viewport::default();
        viewport.ensure_cursor_visible(Cursor::new(40, 0), 25, 100, 10, 20);
        assert_eq!(viewport.top_line(), 33);
        assert_eq!(viewport.left_column(), 6);
    }
}
