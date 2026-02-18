use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::Paragraph,
};

use crate::app::{App, Focus};

impl App {
    pub(crate) fn draw(&mut self, frame: &mut ratatui::Frame) {
        let root = frame.area();
        let vertical = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(1)])
            .split(root);

        let content = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(if self.sidebar_open {
                [Constraint::Percentage(78), Constraint::Percentage(22)]
            } else {
                [Constraint::Percentage(100), Constraint::Length(0)]
            })
            .split(vertical[0]);

        self.draw_editor(frame, content[0]);
        if self.sidebar_open {
            self.draw_tree(frame, content[1]);
        } else {
            self.ui.tree_inner = Rect::default();
        }
        self.draw_completion(frame);
        self.draw_palette(frame);
        self.draw_search_replace(frame);
        self.draw_status(frame, vertical[1]);
        self.place_cursor(frame);
    }

    fn place_cursor(&self, frame: &mut ratatui::Frame) {
        if let Some(pos) = self.palette_cursor_position(frame.area()) {
            frame.set_cursor_position(pos);
            return;
        }

        if let Some(pos) = self.search_replace_cursor_position(frame.area()) {
            frame.set_cursor_position(pos);
            return;
        }

        if self.focus != Focus::Editor {
            return;
        }

        let inner = self.ui.editor_inner;
        if inner.height == 0 || inner.width == 0 {
            return;
        }

        let x = inner
            .x
            .saturating_add(self.line_number_width() as u16 + 1 + self.cursor_col as u16);
        let y = inner
            .y
            .saturating_add(self.cursor_line.saturating_sub(self.editor_scroll) as u16);
        let max_x = inner.x.saturating_add(inner.width.saturating_sub(1));
        let max_y = inner.y.saturating_add(inner.height.saturating_sub(1));
        frame.set_cursor_position((x.min(max_x), y.min(max_y)));
    }

    fn draw_status(&self, frame: &mut ratatui::Frame, area: Rect) {
        let focus = match self.focus {
            Focus::Editor => "EDITOR",
            Focus::FileTree => "FILES",
        };

        let content = format!(
            " Codx | {focus} | Ctrl+P Files | F1/Ctrl+Shift+P Commands | Esc Tree | Ctrl+S Save | Ctrl+Q Quit | {}",
            self.status
        );

        frame.render_widget(
            Paragraph::new(content).style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Rgb(166, 205, 223)),
            ),
            area,
        );
    }
}
