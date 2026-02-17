use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::app::App;

const WIDTH: u16 = 72;
const HEIGHT: u16 = 12;

impl App {
    pub(super) fn draw_palette(&self, frame: &mut ratatui::Frame) {
        let Some(view) = self.palette_view() else {
            return;
        };

        let area = palette_area(frame.area());
        frame.render_widget(Clear, area);

        let block = Block::default()
            .title(format!(" {} ", view.title))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let mut lines = vec![Line::from(Span::styled(
            format!("> {}", view.query),
            Style::default().fg(Color::White),
        ))];

        for (idx, row) in view.rows.iter().enumerate() {
            let style = if idx == view.selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };
            lines.push(Line::from(Span::styled(format!("  {row}"), style)));
        }

        frame.render_widget(Paragraph::new(Text::from(lines)), inner);
    }

    pub(super) fn palette_cursor_position(&self, root: Rect) -> Option<(u16, u16)> {
        let view = self.palette_view()?;
        let area = palette_area(root);
        let x = area.x + 3 + view.query.chars().count() as u16;
        let y = area.y + 1;
        Some((x, y))
    }
}

fn palette_area(root: Rect) -> Rect {
    let width = WIDTH.min(root.width.saturating_sub(2)).max(20);
    let height = HEIGHT.min(root.height.saturating_sub(2)).max(6);
    let x = root.x + (root.width.saturating_sub(width)) / 2;
    let y = root.y + 1;
    Rect::new(x, y, width, height)
}
