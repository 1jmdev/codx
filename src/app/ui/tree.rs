use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
};

use crate::app::{App, Focus};

impl App {
    pub(crate) fn draw_tree(&mut self, frame: &mut ratatui::Frame, area: Rect) {
        let border_style = if self.focus == Focus::FileTree {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::Gray)
        };

        let block = Block::default()
            .title(" Files ")
            .borders(Borders::ALL)
            .border_style(border_style);

        let inner = block.inner(area);
        self.ui.tree_inner = inner;
        frame.render_widget(block, area);
        self.ensure_tree_selection_visible(inner.height as usize);

        let mut rendered = Vec::new();
        for row in 0..inner.height as usize {
            let idx = self.tree_scroll + row;
            if idx >= self.tree_items.len() {
                break;
            }

            let item = &self.tree_items[idx];
            let indent = "  ".repeat(item.depth);
            let icon = if item.is_dir {
                if self.expanded_dirs.contains(&item.path) {
                    "v"
                } else {
                    ">"
                }
            } else {
                "-"
            };

            let name = if item.is_dir {
                format!("{}/", item.name)
            } else {
                item.name.clone()
            };
            let row_style = if idx == self.tree_selected {
                Style::default().bg(Color::Rgb(30, 50, 70))
            } else {
                Style::default()
            };

            rendered.push(Line::from(Span::styled(
                format!("{indent}{icon} {name}"),
                row_style,
            )));
        }

        frame.render_widget(Paragraph::new(Text::from(rendered)), inner);
    }
}
