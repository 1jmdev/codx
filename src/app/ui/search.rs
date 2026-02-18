use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::app::{search::SearchField, App};

const PANEL_INNER_W: u16 = 40;
const PANEL_H_SEARCH: u16 = 3;
const PANEL_H_REPLACE: u16 = 6;

impl App {
    pub(super) fn draw_search_replace(&self, frame: &mut ratatui::Frame) {
        let Some(sr) = self.search_replace.as_ref() else {
            return;
        };

        let panel_h = if sr.show_replace {
            PANEL_H_REPLACE
        } else {
            PANEL_H_SEARCH
        };

        let area = search_panel_area(frame.area(), self.ui.editor_inner, panel_h);
        frame.render_widget(Clear, area);

        let border_style = Style::default().fg(Color::Cyan);
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(border_style)
            .title(Span::styled(
                " Find ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ));
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let search_focused = sr.focused_field == SearchField::Search;
        let match_info = if sr.matches.is_empty() && !sr.query.is_empty() {
            " No results".to_string()
        } else if !sr.matches.is_empty() {
            format!(" {}/{}", sr.current_match + 1, sr.matches.len())
        } else {
            String::new()
        };

        let search_text = Span::styled(
            format!(
                "  {:<width$}",
                sr.query,
                width = inner.width.saturating_sub(6) as usize
            ),
            if search_focused {
                Style::default().fg(Color::White)
            } else {
                Style::default().fg(Color::Rgb(150, 150, 150))
            },
        );
        let match_span = Span::styled(
            match_info,
            Style::default().fg(if sr.matches.is_empty() && !sr.query.is_empty() {
                Color::Red
            } else {
                Color::Rgb(120, 120, 120)
            }),
        );

        let search_row = Rect::new(inner.x, inner.y, inner.width, 1);
        frame.render_widget(
            Paragraph::new(Line::from(vec![search_text, match_span])),
            search_row,
        );

        if sr.show_replace {
            let replace_focused = sr.focused_field == SearchField::Replace;

            let divider_row = Rect::new(inner.x, inner.y + 1, inner.width, 1);
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(
                    "─".repeat(inner.width as usize),
                    Style::default().fg(Color::Rgb(80, 80, 80)),
                ))),
                divider_row,
            );

            let replace_text = Span::styled(
                format!(
                    "  {:<width$}",
                    sr.replacement,
                    width = inner.width.saturating_sub(4) as usize
                ),
                if replace_focused {
                    Style::default().fg(Color::White)
                } else {
                    Style::default().fg(Color::Rgb(150, 150, 150))
                },
            );

            let replace_row = Rect::new(inner.x, inner.y + 2, inner.width, 1);
            frame.render_widget(Paragraph::new(Line::from(replace_text)), replace_row);

            let hint_divider_row = Rect::new(inner.x, inner.y + 3, inner.width, 1);
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(
                    "─".repeat(inner.width as usize),
                    Style::default().fg(Color::Rgb(80, 80, 80)),
                ))),
                hint_divider_row,
            );

            let hint_row = Rect::new(inner.x, inner.y + 4, inner.width, 1);
            frame.render_widget(
                Paragraph::new(Line::from(vec![
                    Span::styled(
                        " Alt+R ",
                        Style::default()
                            .fg(Color::Rgb(180, 180, 60))
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled("replace  ", Style::default().fg(Color::Rgb(160, 160, 160))),
                    Span::styled(
                        "Alt+A ",
                        Style::default()
                            .fg(Color::Rgb(180, 180, 60))
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled("all", Style::default().fg(Color::Rgb(160, 160, 160))),
                ])),
                hint_row,
            );
        }
    }

    pub(super) fn search_replace_cursor_position(&self, root: Rect) -> Option<(u16, u16)> {
        let sr = self.search_replace.as_ref()?;
        let panel_h = if sr.show_replace {
            PANEL_H_REPLACE
        } else {
            PANEL_H_SEARCH
        };
        let area = search_panel_area(root, self.ui.editor_inner, panel_h);
        let inner = Block::default().borders(Borders::ALL).inner(area);

        let (field_y, text) = match sr.focused_field {
            SearchField::Search => (inner.y, &sr.query),
            SearchField::Replace => (inner.y + 2, &sr.replacement),
        };

        let cursor_x = (inner.x + 2 + text.chars().count() as u16).min(inner.x + inner.width - 1);
        Some((cursor_x, field_y))
    }
}

fn search_panel_area(root: Rect, editor_inner: Rect, panel_h: u16) -> Rect {
    let base = if editor_inner.width > 0 {
        editor_inner
    } else {
        root
    };

    let w = (PANEL_INNER_W + 2).min(base.width);
    let x = base.x + base.width.saturating_sub(w);
    let y = base.y;
    Rect::new(x, y, w, panel_h.min(base.height))
}
