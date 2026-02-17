use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
};

use crate::app::editor::byte_index_for_char;
use crate::app::{App, Focus};

impl App {
    pub(crate) fn draw_editor(&mut self, frame: &mut ratatui::Frame, area: Rect) {
        let title = match &self.current_file {
            Some(path) => {
                let marker = if self.dirty { " *" } else { "" };
                let label = path
                    .strip_prefix(&self.cwd)
                    .unwrap_or(path)
                    .to_string_lossy()
                    .replace('\\', "/");
                format!(" Codx: {}{} ", label, marker)
            }
            None => String::from(" Codx: [select a file from tree] "),
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
            let selection = self.selection_cols_for_line(line_index);

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

            let row_style = Style::default();
            let selection_style = Style::default().bg(Color::Rgb(34, 45, 58));
            let mut col_offset = 0;
            for span in spans {
                push_selected_span(
                    &mut line_spans,
                    span.patch_style(row_style),
                    &mut col_offset,
                    selection,
                    selection_style,
                );
            }
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

fn push_selected_span(
    target: &mut Vec<Span<'static>>,
    span: Span<'static>,
    col_offset: &mut usize,
    selection: Option<(usize, usize)>,
    selection_style: Style,
) {
    let Some((sel_start, sel_end)) = selection else {
        *col_offset += span.content.chars().count();
        target.push(span);
        return;
    };

    let text = span.content.into_owned();
    let style = span.style;
    let len = text.chars().count();
    let start = *col_offset;
    let end = start + len;
    *col_offset = end;

    if len == 0 || end <= sel_start || start >= sel_end {
        target.push(Span::styled(text, style));
        return;
    }

    let selected_start = sel_start.max(start);
    let selected_end = sel_end.min(end);
    let before_len = selected_start.saturating_sub(start);
    let selected_len = selected_end.saturating_sub(selected_start);

    if before_len > 0 {
        let before_byte = byte_index_for_char(&text, before_len);
        target.push(Span::styled(text[..before_byte].to_string(), style));
    }

    if selected_len > 0 {
        let selected_start_byte = byte_index_for_char(&text, before_len);
        let selected_end_byte = byte_index_for_char(&text, before_len + selected_len);
        target.push(Span::styled(
            text[selected_start_byte..selected_end_byte].to_string(),
            style.patch(selection_style),
        ));
    }

    let after_start = before_len + selected_len;
    if after_start < len {
        let after_start_byte = byte_index_for_char(&text, after_start);
        target.push(Span::styled(text[after_start_byte..].to_string(), style));
    }
}
