use std::path::PathBuf;

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::app::App;

const WIDTH_PERCENT: u16 = 82;
const HEIGHT_PERCENT: u16 = 80;
const WIDTH_PERCENT_NARROW: u16 = 40;

impl App {
    pub(super) fn draw_palette(&mut self, frame: &mut ratatui::Frame) {
        let Some(view) = self.palette_view() else {
            self.ui.palette_inner = Rect::default();
            self.ui.palette_results = Rect::default();
            return;
        };

        let has_preview = view.preview.is_some();
        let area = palette_area(frame.area(), has_preview);
        frame.render_widget(Clear, area);

        let total = view.total_matches;
        let title = if total > 0 {
            format!(
                " {} ({}/{}) ",
                view.title,
                view.scroll + view.selected + 1,
                total
            )
        } else {
            format!(" {} ", view.title)
        };

        let outer = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));
        let inner = outer.inner(area);
        self.ui.palette_inner = inner;
        frame.render_widget(outer, area);

        let (results_area, preview_area) = if has_preview {
            let panes = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(38), Constraint::Percentage(62)])
                .split(inner);
            (panes[0], Some(panes[1]))
        } else {
            (inner, None)
        };
        self.ui.palette_results = results_area;

        {
            let mut lines: Vec<Line> = Vec::new();

            lines.push(Line::from(Span::styled(
                format!("> {}", view.query),
                Style::default().fg(Color::White),
            )));

            if view.show_replace {
                lines.push(Line::from(Span::styled(
                    "─".repeat(results_area.width as usize),
                    Style::default().fg(Color::Rgb(80, 80, 80)),
                )));
                lines.push(Line::from(Span::styled(
                    format!("> {}", view.replace_text),
                    Style::default().fg(Color::Rgb(200, 200, 150)),
                )));
            }

            lines.push(Line::from(Span::styled(
                "─".repeat(results_area.width as usize),
                Style::default().fg(Color::Rgb(60, 60, 60)),
            )));

            for (idx, row) in view.rows.iter().enumerate() {
                let is_selected = idx == view.selected;
                let style = if is_selected {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Rgb(180, 180, 180))
                };
                let max_chars = results_area.width.saturating_sub(3) as usize;
                let label: String = row.chars().take(max_chars).collect();
                lines.push(Line::from(Span::styled(format!("  {label}"), style)));
            }

            frame.render_widget(Paragraph::new(Text::from(lines)), results_area);
        }

        if let (Some(right), Some(preview)) = (preview_area, &view.preview) {
            let sep_x = right.x.saturating_sub(1);
            let sep_lines: Vec<Line> = (0..right.height)
                .map(|_| {
                    Line::from(Span::styled(
                        "│",
                        Style::default().fg(Color::Rgb(60, 60, 60)),
                    ))
                })
                .collect();
            frame.render_widget(
                Paragraph::new(Text::from(sep_lines)),
                Rect::new(sep_x, right.y, 1, right.height),
            );

            self.draw_palette_preview(frame, right, &preview.path, preview.focus_line);
        }
    }

    fn draw_palette_preview(
        &self,
        frame: &mut ratatui::Frame,
        area: Rect,
        path: &PathBuf,
        focus_line: usize,
    ) {
        let visible_rows = area.height as usize;
        if visible_rows == 0 {
            return;
        }

        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => {
                frame.render_widget(
                    Paragraph::new(Span::styled(
                        "  (binary or unreadable)",
                        Style::default().fg(Color::Rgb(100, 100, 100)),
                    )),
                    area,
                );
                return;
            }
        };

        let file_lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
        let total_lines = file_lines.len();
        if total_lines == 0 {
            return;
        }

        let half = visible_rows / 2;
        let scroll_start = focus_line
            .saturating_sub(half)
            .min(total_lines.saturating_sub(visible_rows));
        let scroll_end = (scroll_start + visible_rows).min(total_lines);

        let highlighted = self.syntax.highlight_visible(
            Some(path.as_path()),
            &file_lines,
            scroll_start,
            scroll_end,
        );

        let number_width = if total_lines >= 1000 { 4 } else { 3 };
        let mut rendered: Vec<Line> = Vec::new();

        for (row_idx, spans) in highlighted.into_iter().enumerate() {
            let line_idx = scroll_start + row_idx;
            let is_focus = line_idx == focus_line;

            let num_style = if is_focus {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Rgb(70, 70, 70))
            };

            let mut line_spans: Vec<Span<'static>> = Vec::new();

            line_spans.push(Span::styled(
                format!(" {:>width$} ", line_idx + 1, width = number_width),
                num_style,
            ));

            if is_focus {
                // highlight the entire focus line
                let focus_bg = Style::default().bg(Color::Rgb(40, 50, 65));
                for span in spans {
                    line_spans.push(span.patch_style(focus_bg));
                }
                rendered.push(
                    Line::from(line_spans).style(Style::default().bg(Color::Rgb(40, 50, 65))),
                );
            } else {
                line_spans.extend(spans);
                rendered.push(Line::from(line_spans));
            }
        }

        frame.render_widget(Paragraph::new(Text::from(rendered)), area);
    }

    pub(super) fn palette_cursor_position(&self, root: Rect) -> Option<(u16, u16)> {
        let view = self.palette_view()?;
        let area = palette_area(root, view.preview.is_some());

        let inner_x = area.x + 1;
        let inner_y = area.y + 1;
        let x = inner_x + 2 + view.query.chars().count() as u16;
        Some((x, inner_y))
    }
}

fn palette_area(root: Rect, has_preview: bool) -> Rect {
    let wp = if has_preview {
        WIDTH_PERCENT
    } else {
        WIDTH_PERCENT_NARROW
    };
    let width = (root.width * wp / 100)
        .min(root.width.saturating_sub(4))
        .max(40);
    let height = (root.height * HEIGHT_PERCENT / 100)
        .min(root.height.saturating_sub(4))
        .max(10);
    let x = root.x + root.width.saturating_sub(width) / 2;
    let y = root.y + root.height.saturating_sub(height) / 2;
    Rect::new(x, y, width, height)
}
