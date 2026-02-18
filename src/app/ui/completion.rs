use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::app::App;

const MIN_WIDTH: u16 = 18;
const MAX_WIDTH: u16 = 48;
const MAX_ROWS: usize = 8;

impl App {
    pub(super) fn draw_completion(&self, frame: &mut ratatui::Frame) {
        let Some(state) = self.completion.as_ref() else {
            return;
        };
        if state.items.is_empty() {
            return;
        }

        let Some(area) = self.completion_area(frame.area()) else {
            return;
        };

        frame.render_widget(Clear, area);
        let block = Block::default()
            .title(" Suggestions ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let mut rows = Vec::new();
        let start = state.scroll.min(state.items.len().saturating_sub(1));
        let end = (start + MAX_ROWS).min(state.items.len());
        for idx in start..end {
            let item = &state.items[idx];
            let style = if idx == state.selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };
            rows.push(Line::from(Span::styled(format!(" {}", item.label), style)));
        }

        frame.render_widget(Paragraph::new(Text::from(rows)), inner);
    }

    fn completion_area(&self, root: Rect) -> Option<Rect> {
        let state = self.completion.as_ref()?;
        let inner = self.ui.editor_inner;
        if inner.width < 6 || inner.height < 3 {
            return None;
        }

        let width = completion_width(state)
            .min(inner.width.saturating_sub(1))
            .max(MIN_WIDTH)
            .min(MAX_WIDTH);
        let row_count = state.items.len().min(MAX_ROWS) as u16;
        let height = (row_count + 2).min(inner.height.max(3));

        let mut x = inner.x + self.line_number_width() as u16 + 2 + state.anchor_col as u16;
        let mut y = inner.y + state.line.saturating_sub(self.editor_scroll) as u16 + 1;
        let max_x = root.x + root.width.saturating_sub(width);
        let max_y = root.y + root.height.saturating_sub(height);
        if x > max_x {
            x = max_x;
        }
        if y > max_y {
            y = max_y;
        }

        Some(Rect::new(x, y, width, height.max(3)))
    }
}

fn completion_width(state: &crate::app::state::CompletionState) -> u16 {
    let widest = state
        .items
        .iter()
        .take(MAX_ROWS)
        .map(|item| item.label.chars().count())
        .max()
        .unwrap_or(0) as u16;
    widest + 3
}
