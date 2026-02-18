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

        let total = view.total_matches;
        let title = if total > view.rows.len() || view.scroll > 0 {
            format!(
                " {} ({}/{}) ",
                view.title,
                view.scroll + view.selected + 1,
                total
            )
        } else {
            format!(" {} ", view.title)
        };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let mut lines = vec![Line::from(Span::styled(
            format!("> {}", view.query),
            Style::default().fg(Color::White),
        ))];

        if view.show_replace {
            lines.push(Line::from(Span::styled(
                "─".repeat(inner.width as usize),
                Style::default().fg(Color::Rgb(80, 80, 80)),
            )));
            lines.push(Line::from(Span::styled(
                format!("> {}", view.replace_text),
                Style::default().fg(Color::Rgb(200, 200, 150)),
            )));
            lines.push(Line::from(Span::styled(
                "─".repeat(inner.width as usize),
                Style::default().fg(Color::Rgb(80, 80, 80)),
            )));
        }

        if view.scroll > 0 {
            lines.push(Line::from(Span::styled(
                format!("  \u{25b2} {} more above", view.scroll),
                Style::default().fg(Color::Rgb(100, 100, 100)),
            )));
        }

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

        let remaining = total.saturating_sub(view.scroll + view.rows.len());
        if remaining > 0 {
            lines.push(Line::from(Span::styled(
                format!("  \u{25bc} {} more below", remaining),
                Style::default().fg(Color::Rgb(100, 100, 100)),
            )));
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
