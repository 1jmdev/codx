use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
};

use crate::app::{App, Focus};

impl App {
    pub(crate) fn draw_editor(&mut self, frame: &mut ratatui::Frame, area: Rect) {
        let title = match &self.current_file {
            Some(path) => {
                let marker = if self.dirty { " *" } else { "" };
                format!(" Editor: {}{} ", path.display(), marker)
            }
            None => String::from(" Editor: [select a file from tree] "),
        };

        let border_style = if self.focus == Focus::Editor {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::Gray)
        };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(border_style);
        let inner = block.inner(area);
        self.ui.editor_inner = inner;
        frame.render_widget(block, area);

        self.ensure_cursor_visible(inner.height as usize);
        let start = self.editor_scroll;
        let end = (start + inner.height as usize).min(self.lines.len());
        let highlighted =
            self.syntax
                .highlight_visible(self.current_file.as_deref(), &self.lines, start, end);

        let number_width = self.line_number_width();
        let mut rendered = Vec::new();
        for (row, spans) in highlighted.into_iter().enumerate() {
            let line_index = start + row;
            let mut line_spans = Vec::new();

            let diagnostic = self
                .lsp
                .diagnostic_hint_for_line(self.current_file.as_ref(), line_index);
            let num_color = match diagnostic {
                Some((_, true)) => Color::Yellow,
                Some((_, false)) => Color::Red,
                None => Color::DarkGray,
            };
            let number = format!("{:>width$}", line_index + 1, width = number_width);
            line_spans.push(Span::styled(
                number,
                Style::default().fg(num_color).add_modifier(Modifier::BOLD),
            ));
            line_spans.push(Span::raw(" "));

            let row_style = if line_index == self.cursor_line {
                Style::default().bg(Color::Rgb(25, 35, 45))
            } else {
                Style::default()
            };
            line_spans.extend(spans.into_iter().map(|span| span.patch_style(row_style)));
            if let Some((message, is_warning)) = diagnostic {
                let message_color = if is_warning {
                    Color::Yellow
                } else {
                    Color::Red
                };
                line_spans.push(
                    Span::styled(
                        format!("  // {}", message),
                        Style::default()
                            .fg(message_color)
                            .add_modifier(Modifier::ITALIC),
                    )
                    .patch_style(row_style),
                );
            }
            rendered.push(Line::from(line_spans));
        }

        frame.render_widget(Paragraph::new(Text::from(rendered)), inner);
    }
}
