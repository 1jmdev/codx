use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Widget};
use ratatui::Frame;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::app::{App, AppMode, FocusTarget, MessageKind};
use crate::ui::Palette;
use crate::view::build_statusline;

pub fn render(frame: &mut Frame<'_>, app: &App) {
    let areas = Layout::vertical([
        Constraint::Min(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .split(frame.area());

    render_workspace(frame, areas[0], app);
    render_statusline(frame.buffer_mut(), areas[1], app);
    render_message_or_command_bar(frame.buffer_mut(), areas[2], app);
    render_picker_overlay(frame, app);

    let cursor = screen_cursor_position(app, areas[0], areas[2]);
    frame.set_cursor_position(cursor);
}

fn render_workspace(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let palette = Palette::mocha().styles();
    let chunks = if app.explorer().visible() {
        Layout::horizontal([Constraint::Length(30), Constraint::Min(1)]).split(area)
    } else {
        Layout::horizontal([Constraint::Min(1)]).split(area)
    };

    let editor_area = if app.explorer().visible() {
        render_explorer(frame.buffer_mut(), chunks[0], app);
        chunks[1]
    } else {
        chunks[0]
    };

    for (pane_id, pane_area, pane) in app.layout.leaves_in_area(editor_area) {
        let active = pane_id == app.active_pane_id();
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(if active {
                palette.statusline
            } else {
                palette.editor
            });
        let inner = block.inner(pane_area);
        block.render(pane_area, frame.buffer_mut());

        if let Some(buffer_state) = app.buffer_by_id(pane.buffer_id()) {
            render_buffer_view(frame.buffer_mut(), inner, app, buffer_state.id, pane_id);
        }
    }
}

fn render_explorer(buffer: &mut Buffer, area: Rect, app: &App) {
    let palette = Palette::mocha().styles();
    let is_focused = app.focus() == FocusTarget::Explorer;

    let border_style = if is_focused {
        palette.explorer_border_focused
    } else {
        palette.explorer_border
    };

    let title_style = if is_focused {
        palette.explorer_border_focused.add_modifier(Modifier::BOLD)
    } else {
        palette.explorer_border
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(Span::styled(" \u{f07b} Explorer ", title_style));
    let inner = block.inner(area);
    block.render(area, buffer);

    let items = app
        .explorer()
        .entries()
        .iter()
        .enumerate()
        .map(|(index, entry)| {
            let indent = "  ".repeat(entry.depth);
            let is_expanded = entry.is_dir && app.explorer().is_expanded(&entry.path);
            let icon = if entry.is_dir {
                if is_expanded { "\u{f07c} " } else { "\u{f07b} " }
            } else {
                "\u{f15b} "
            };
            let name = entry
                .path
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_else(|| entry.path.display().to_string());
            let is_selected = index == app.explorer().selected();
            let style = if is_selected {
                if entry.is_dir {
                    palette.explorer_dir_selected
                } else {
                    palette.explorer_file_selected
                }
            } else if entry.is_dir {
                palette.explorer_dir
            } else {
                palette.editor
            };
            ListItem::new(Line::from(Span::styled(
                format!("{indent}{icon}{name}"),
                style,
            )))
        })
        .collect::<Vec<_>>();

    List::new(items).render(inner, buffer);
}

fn render_buffer_view(
    buffer: &mut Buffer,
    area: Rect,
    app: &App,
    buffer_id: u64,
    pane_id: u64,
) {
    let palette = Palette::mocha().styles();
    let Some(buffer_state) = app.buffer_by_id(buffer_id) else {
        return;
    };
    let Some(pane) = app.layout.pane(pane_id) else {
        return;
    };

    let gutter_width =
        gutter_width(buffer_state.document.line_count()).min(area.width.saturating_sub(1));
    let text_area = Rect {
        x: area.x.saturating_add(gutter_width),
        y: area.y,
        width: area.width.saturating_sub(gutter_width),
        height: area.height,
    };

    for row in 0..area.height as usize {
        let line_index = pane.viewport().top_line() + row;
        let line_number = if line_index < buffer_state.document.line_count() {
            format!("{:>4} ", line_index + 1)
        } else {
            String::from("~    ")
        };
        let style = if line_index == pane.cursor().line {
            palette.gutter_current
        } else if line_index < buffer_state.document.line_count() {
            palette.gutter
        } else {
            palette.tilde
        };
        let line = Line::from(Span::styled(line_number, style));
        line.render(
            Rect {
                x: area.x,
                y: area.y + row as u16,
                width: gutter_width,
                height: 1,
            },
            buffer,
        );
    }

    let mut lines = Vec::with_capacity(area.height as usize);
    for row in 0..text_area.height as usize {
        let line_index = pane.viewport().top_line() + row;
        lines.push(render_text_line(app, buffer_id, pane_id, line_index, text_area.width as usize));
    }
    Paragraph::new(lines).style(palette.editor).render(text_area, buffer);
}

fn render_text_line(
    app: &App,
    buffer_id: u64,
    pane_id: u64,
    line_index: usize,
    width: usize,
) -> Line<'static> {
    let palette = Palette::mocha().styles();
    let Some(buffer_state) = app.buffer_by_id(buffer_id) else {
        return Line::from("");
    };
    let Some(pane) = app.layout.pane(pane_id) else {
        return Line::from("");
    };

    if line_index >= buffer_state.document.line_count() {
        return Line::from(Span::styled("", palette.editor));
    }

    let raw_line = buffer_state.document.line_text(line_index);
    let mut spans = Vec::new();
    let mut display_column = 0usize;
    let mut char_column = 0usize;

    for grapheme in raw_line.graphemes(true) {
        let expanded = if grapheme == "\t" { "    " } else { grapheme };
        let grapheme_width = expanded.width().max(1);
        let next_display = display_column + grapheme_width;
        let grapheme_chars = grapheme.chars().count().max(1);

        if next_display <= pane.viewport().left_column() {
            display_column = next_display;
            char_column += grapheme_chars;
            continue;
        }

        if display_column >= pane.viewport().left_column() + width {
            break;
        }

        let style = style_for_position(app, pane_id, line_index, char_column, palette.editor);
        spans.push(Span::styled(expanded.to_string(), style));
        display_column = next_display;
        char_column += grapheme_chars;
    }

    if pane.selection().starts_at(line_index, raw_line.chars().count()) {
        spans.push(Span::styled(" ", palette.selection));
    }

    Line::from(spans)
}

fn style_for_position(
    app: &App,
    pane_id: u64,
    line: usize,
    column: usize,
    default: Style,
) -> Style {
    let palette = Palette::mocha().styles();
    let Some(pane) = app.layout.pane(pane_id) else {
        return default;
    };
    if pane.selection().contains(line, column) {
        palette.selection
    } else if pane.search().is_active_match_at(line, column) {
        palette.active_search_match
    } else if pane.search().is_match_at(line, column) {
        palette.search_match
    } else {
        default
    }
}

fn render_statusline(buffer: &mut Buffer, area: Rect, app: &App) {
    let palette = Palette::mocha().styles();
    Paragraph::new(build_statusline(app))
        .style(palette.statusline)
        .render(area, buffer);
}

fn render_message_or_command_bar(buffer: &mut Buffer, area: Rect, app: &App) {
    let palette = Palette::mocha().styles();

    if let (Some(prefix), Some(input)) = (app.command_bar_prefix(), app.command_bar_input()) {
        let line = Line::from(vec![
            Span::styled(prefix, palette.statusline),
            Span::styled(input.to_owned(), palette.command_bar),
        ]);
        Paragraph::new(line)
            .style(palette.command_bar)
            .render(area, buffer);
        return;
    }

    let content = match app.mode() {
        AppMode::ConfirmQuit => app
            .message()
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| String::from("Unsaved changes: y quit, n cancel, s save")),
        _ => app.message().map(ToOwned::to_owned).unwrap_or_default(),
    };

    let style = match app.message_kind() {
        MessageKind::Info => palette.message,
        MessageKind::Warning => palette.warning,
        MessageKind::Error => palette.error,
    };

    Paragraph::new(content).style(style).render(area, buffer);
}

fn render_picker_overlay(frame: &mut Frame<'_>, app: &App) {
    let Some(picker) = app.picker() else {
        return;
    };

    let popup = centered_rect(frame.area(), 70, 60);
    let palette = Palette::mocha().styles();

    let block = Block::default()
        .borders(Borders::ALL)
        .title(match picker.kind() {
            crate::ui::PickerKind::Files => " Files ",
            crate::ui::PickerKind::Buffers => " Buffers ",
        });
    let inner = block.inner(popup);
    let areas = Layout::vertical([Constraint::Length(1), Constraint::Min(1)]).split(inner);

    Clear.render(popup, frame.buffer_mut());
    block.render(popup, frame.buffer_mut());

    Paragraph::new(picker.query().to_owned())
        .style(palette.command_bar)
        .render(areas[0], frame.buffer_mut());

    let items = picker
        .items()
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let style = if index == picker.selected() {
                palette.selection
            } else {
                palette.editor
            };
            let text = if item.subtitle.is_empty() {
                item.title.clone()
            } else {
                format!("{}  {}", item.title, item.subtitle)
            };
            ListItem::new(Line::from(Span::styled(text, style)))
        })
        .collect::<Vec<_>>();
    List::new(items).render(areas[1], frame.buffer_mut());
}

fn screen_cursor_position(app: &App, editor_area: Rect, message_area: Rect) -> (u16, u16) {
    if app.picker().is_some() {
        let popup = centered_rect(editor_area, 70, 60);
        let x = popup.x.saturating_add(1 + app.picker().map(|picker| picker.query().chars().count() as u16).unwrap_or(0));
        return (x, popup.y.saturating_add(1));
    }

    if let (Some(prefix), Some(input)) = (app.command_bar_prefix(), app.command_bar_input()) {
        let x = message_area
            .x
            .saturating_add(prefix.chars().count() as u16)
            .saturating_add(input.chars().count() as u16);
        return (x, message_area.y);
    }

    let pane_area = app
        .layout
        .leaves_in_area(if app.explorer().visible() {
            Layout::horizontal([Constraint::Length(30), Constraint::Min(1)]).split(editor_area)[1]
        } else {
            editor_area
        })
        .into_iter()
        .find(|(pane_id, _, _)| *pane_id == app.active_pane_id());

    let Some((_, area, pane)) = pane_area else {
        return (editor_area.x, editor_area.y);
    };
    let gutter_width = gutter_width(app.active_document().line_count());
    let x = area
        .x
        .saturating_add(1)
        .saturating_add(gutter_width)
        .saturating_add(
            app.active_document()
                .display_column(pane.cursor())
                .saturating_sub(pane.viewport().left_column()) as u16,
        );
    let y = area
        .y
        .saturating_add(1)
        .saturating_add(pane.cursor().line.saturating_sub(pane.viewport().top_line()) as u16);
    (x, y)
}

fn centered_rect(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([
        Constraint::Percentage((100u16.saturating_sub(percent_y)) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100u16.saturating_sub(percent_y)) / 2),
    ])
    .split(area);
    Layout::horizontal([
        Constraint::Percentage((100u16.saturating_sub(percent_x)) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100u16.saturating_sub(percent_x)) / 2),
    ])
    .split(vertical[1])[1]
}

fn gutter_width(line_count: usize) -> u16 {
    let digits = line_count.max(1).to_string().chars().count() as u16;
    digits.saturating_add(2).max(5)
}
