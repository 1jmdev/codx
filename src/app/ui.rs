use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
};

use super::{App, Focus};

impl App {
    pub(crate) fn draw(&mut self, frame: &mut ratatui::Frame) {
        let root = frame.area();
        let vertical = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(1)])
            .split(root);

        let content = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(74), Constraint::Percentage(26)])
            .split(vertical[0]);

        self.draw_editor(frame, content[0]);
        self.draw_tree(frame, content[1]);
        self.draw_status(frame, vertical[1]);

        if self.focus == Focus::Editor {
            let inner = self.ui.editor_inner;
            if inner.height > 0 && inner.width > 0 {
                let number_width = self.line_number_width();
                let x = inner
                    .x
                    .saturating_add(number_width as u16 + 1 + self.cursor_col as u16);
                let y = inner
                    .y
                    .saturating_add(self.cursor_line.saturating_sub(self.editor_scroll) as u16);
                let max_x = inner.x.saturating_add(inner.width.saturating_sub(1));
                let max_y = inner.y.saturating_add(inner.height.saturating_sub(1));
                frame.set_cursor_position((x.min(max_x), y.min(max_y)));
            }
        }
    }

    fn draw_editor(&mut self, frame: &mut ratatui::Frame, area: Rect) {
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

        let number_width = self.line_number_width();
        let mut rendered = Vec::with_capacity(inner.height as usize);
        for row in 0..inner.height as usize {
            let line_index = self.editor_scroll + row;
            if line_index >= self.lines.len() {
                break;
            }

            let number = format!("{:>width$}", line_index + 1, width = number_width);
            let text = &self.lines[line_index];
            let row_style = if line_index == self.cursor_line {
                Style::default().bg(Color::Rgb(25, 35, 45))
            } else {
                Style::default()
            };

            rendered.push(Line::from(vec![
                Span::styled(
                    number,
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled(text.clone(), row_style),
            ]));
        }

        frame.render_widget(Paragraph::new(Text::from(rendered)), inner);
    }

    fn draw_tree(&mut self, frame: &mut ratatui::Frame, area: Rect) {
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

    fn draw_status(&self, frame: &mut ratatui::Frame, area: Rect) {
        let focus = match self.focus {
            Focus::Editor => "EDITOR",
            Focus::FileTree => "FILES",
        };

        let content = format!(
            " {focus} | Ctrl+S Save | Ctrl+Q Quit | Tab Switch | Enter Open/Toggle | {}",
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
