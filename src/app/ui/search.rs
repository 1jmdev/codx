use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::app::{search::SearchField, App};

/// Width of the search/replace panel (without borders).
const PANEL_INNER_W: u16 = 36;
/// Height when only the search row is shown.
const PANEL_H_SEARCH: u16 = 3; // border-top + input row + border-bottom
/// Height when the replace row is also shown.
const PANEL_H_REPLACE: u16 = 5; // + label + input row

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

        // Outer border
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

        // ── Search row ────────────────────────────────────────────────────────
        let search_focused = sr.focused_field == SearchField::Search;
        let match_info = if sr.matches.is_empty() && !sr.query.is_empty() {
            " No results".to_string()
        } else if !sr.matches.is_empty() {
            format!(" {}/{}", sr.current_match + 1, sr.matches.len())
        } else {
            String::new()
        };

        let search_label_style = if search_focused {
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        let search_prefix = Span::styled("  ", search_label_style);
        let search_text = Span::styled(
            format!(
                "{:<width$}",
                sr.query,
                width = inner.width.saturating_sub(6) as usize
            ),
            if search_focused {
                Style::default().fg(Color::White)
            } else {
                Style::default().fg(Color::Gray)
            },
        );
        let match_span = Span::styled(
            match_info,
            Style::default().fg(if sr.matches.is_empty() && !sr.query.is_empty() {
                Color::Red
            } else {
                Color::DarkGray
            }),
        );

        let search_row = Rect::new(inner.x, inner.y, inner.width, 1);
        frame.render_widget(
            Paragraph::new(Line::from(vec![search_prefix, search_text, match_span])),
            search_row,
        );

        // ── Replace row (only when show_replace) ─────────────────────────────
        if sr.show_replace {
            let replace_focused = sr.focused_field == SearchField::Replace;
            let replace_label_style = if replace_focused {
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            let replace_prefix = Span::styled("  ", replace_label_style);
            let replace_text = Span::styled(
                format!(
                    "{:<width$}",
                    sr.replacement,
                    width = inner.width.saturating_sub(4) as usize
                ),
                if replace_focused {
                    Style::default().fg(Color::White)
                } else {
                    Style::default().fg(Color::Gray)
                },
            );

            // Divider between the two rows
            let divider_row = Rect::new(inner.x, inner.y + 1, inner.width, 1);
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(
                    "─".repeat(inner.width as usize),
                    Style::default().fg(Color::Rgb(60, 60, 60)),
                ))),
                divider_row,
            );

            let replace_row = Rect::new(inner.x, inner.y + 2, inner.width, 1);
            frame.render_widget(
                Paragraph::new(Line::from(vec![replace_prefix, replace_text])),
                replace_row,
            );

            // Hint line at the bottom
            let hint_row = Rect::new(inner.x, inner.y + 3, inner.width, 1);
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(
                    " Alt+R replace  Alt+A all  Tab switch",
                    Style::default().fg(Color::Rgb(90, 90, 90)),
                ))),
                hint_row,
            );
        } else {
            // Search-only hint
            let hint_row = Rect::new(inner.x, inner.y + 1, inner.width, 1);
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(
                    " Enter next  Shift+Enter prev  Ctrl+H replace",
                    Style::default().fg(Color::Rgb(90, 90, 90)),
                ))),
                hint_row,
            );
        }
    }

    /// Returns the cursor position to place inside the active search/replace field.
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

        // 2 chars for the "  " prefix
        let cursor_x = (inner.x + 2 + text.chars().count() as u16).min(inner.x + inner.width - 1);
        Some((cursor_x, field_y))
    }
}

/// Position the panel in the top-right corner of the editor inner area.
fn search_panel_area(root: Rect, editor_inner: Rect, panel_h: u16) -> Rect {
    // Use editor_inner if it has been computed, otherwise fall back to root.
    let base = if editor_inner.width > 0 {
        editor_inner
    } else {
        root
    };

    let w = (PANEL_INNER_W + 2).min(base.width); // +2 for borders
    let x = base.x + base.width.saturating_sub(w);
    let y = base.y;
    Rect::new(x, y, w, panel_h.min(base.height))
}
