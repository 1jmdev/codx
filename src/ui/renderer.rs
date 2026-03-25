use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Widget};
use ratatui::Frame;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::app::{App, AppMode, FocusTarget, MessageKind};
use crate::lsp::{DiagnosticItem, DiagnosticSeverityView};
use crate::syntax::HighlightSpan;
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
    render_completion_overlay(frame, app);
    render_hover_overlay(frame, app);
    render_signature_overlay(frame, app);
    render_diagnostics_overlay(frame, app);
    render_delete_confirm_overlay(frame, app);
    render_external_change_conflict_overlay(frame, app);

    let cursor = screen_cursor_position(app, frame.area(), areas[0], areas[2]);
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

    let viewport_height = inner.height as usize;
    let entries = app.explorer().entries();
    let scroll_offset = app.explorer().scroll_offset();

    let items = entries
        .iter()
        .enumerate()
        .skip(scroll_offset)
        .take(viewport_height)
        .map(|(index, entry)| {
            let indent = "  ".repeat(entry.depth);
            let is_expanded = entry.is_dir && app.explorer().is_expanded(&entry.path);
            let icon = if entry.is_dir {
                if is_expanded {
                    "\u{f07c} "
                } else {
                    "\u{f07b} "
                }
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

fn render_buffer_view(buffer: &mut Buffer, area: Rect, app: &App, buffer_id: u64, pane_id: u64) {
    let palette = Palette::mocha().styles();
    let Some(buffer_state) = app.buffer_by_id(buffer_id) else {
        return;
    };
    let Some(pane) = app.layout.pane(pane_id) else {
        return;
    };

    let folds = app.fold_ranges_for_buffer(buffer_id);

    let gutter_width =
        gutter_width(buffer_state.document.line_count()).min(area.width.saturating_sub(1));
    let text_area = Rect {
        x: area.x.saturating_add(gutter_width),
        y: area.y,
        width: area.width.saturating_sub(gutter_width),
        height: area.height,
    };
    let diagnostics = buffer_state
        .document
        .path()
        .map(|path| app.lsp.diagnostics_for_path(path))
        .unwrap_or(&[]);

    for row in 0..area.height as usize {
        let line_index = pane.viewport().top_line() + row;
        let is_foldable = folds
            .iter()
            .any(|f| f.start_line == line_index && f.end_line > line_index);
        let line_number = if line_index < buffer_state.document.line_count() {
            if is_foldable {
                format!("{:>3}\u{25be} ", line_index + 1)
            } else {
                format!("{:>4} ", line_index + 1)
            }
        } else {
            String::from("~    ")
        };
        let style = if line_index == pane.cursor().line {
            palette.gutter_current
        } else if line_index < buffer_state.document.line_count() {
            match line_max_diagnostic_severity(diagnostics, line_index) {
                Some(DiagnosticSeverityView::Error) => palette.diagnostic_error,
                Some(DiagnosticSeverityView::Warning) => palette.diagnostic_warning,
                Some(DiagnosticSeverityView::Information) => palette.diagnostic_information,
                Some(DiagnosticSeverityView::Hint) => palette.diagnostic_hint,
                None => palette.gutter,
            }
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

    let theme = app.active_theme();
    let editor_style = Style::default()
        .fg(Color::Rgb(
            theme.foreground.r,
            theme.foreground.g,
            theme.foreground.b,
        ))
        .bg(Color::Rgb(
            theme.background.r,
            theme.background.g,
            theme.background.b,
        ));

    let mut lines = Vec::with_capacity(area.height as usize);
    for row in 0..text_area.height as usize {
        let line_index = pane.viewport().top_line() + row;
        lines.push(render_text_line(
            app,
            buffer_id,
            pane_id,
            line_index,
            text_area.width as usize,
        ));
    }
    Paragraph::new(lines)
        .style(editor_style)
        .render(text_area, buffer);
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
    let syntax_spans = app.syntax_spans_for_line(buffer_id, line_index);
    let diagnostics = buffer_state
        .document
        .path()
        .map(|path| app.lsp.diagnostics_for_path(path))
        .unwrap_or(&[]);
    let line_severity = line_max_diagnostic_severity(diagnostics, line_index);
    let line_message = diagnostics
        .iter()
        .filter(|diag| diag.line == line_index)
        .max_by_key(|diag| diag.severity.rank())
        .map(|diag| diag.message.as_str());
    let theme = app.active_theme();
    let plain_style = Style::default().fg(Color::Rgb(
        theme.foreground.r,
        theme.foreground.g,
        theme.foreground.b,
    ));

    let mut spans = Vec::new();
    let mut display_column = 0usize;
    let mut char_column = 0usize;
    let mut byte_offset = 0usize;

    for grapheme in raw_line.graphemes(true) {
        let expanded = if grapheme == "\t" { "    " } else { grapheme };
        let grapheme_width = expanded.width().max(1);
        let next_display = display_column + grapheme_width;
        let grapheme_chars = grapheme.chars().count().max(1);
        let grapheme_bytes = grapheme.len();

        if next_display <= pane.viewport().left_column() {
            display_column = next_display;
            char_column += grapheme_chars;
            byte_offset += grapheme_bytes;
            continue;
        }

        if display_column >= pane.viewport().left_column() + width {
            break;
        }

        let mut base_style = if pane.selection().contains(line_index, char_column) {
            palette.selection
        } else if pane.search().is_active_match_at(line_index, char_column) {
            palette.active_search_match
        } else if pane.search().is_match_at(line_index, char_column) {
            palette.search_match
        } else {
            let syntax_style = find_span_style(&syntax_spans, byte_offset, theme);
            syntax_style.unwrap_or(plain_style)
        };

        if line_severity == Some(DiagnosticSeverityView::Error) {
            base_style = base_style.add_modifier(Modifier::UNDERLINED);
        }

        spans.push(Span::styled(expanded.to_string(), base_style));
        display_column = next_display;
        char_column += grapheme_chars;
        byte_offset += grapheme_bytes;
    }

    if pane
        .selection()
        .starts_at(line_index, raw_line.chars().count())
    {
        spans.push(Span::styled(" ", palette.selection));
    }

    if let Some(message) = line_message {
        let visible_len = pane.viewport().left_column() + width;
        if raw_line.chars().count() < visible_len {
            let mut lens = String::from("      // ");
            lens.push_str(&single_line_diagnostic_message(message));
            lens.push(' ');

            let style = match line_severity {
                Some(DiagnosticSeverityView::Error) => palette.diagnostic_lens_error,
                Some(DiagnosticSeverityView::Warning) => palette.diagnostic_lens_warning,
                Some(DiagnosticSeverityView::Information) => palette.diagnostic_lens_information,
                Some(DiagnosticSeverityView::Hint) | None => palette.diagnostic_lens_hint,
            };
            spans.push(Span::styled(lens, style));
        }
    }

    Line::from(spans)
}

fn find_span_style(
    spans: &[HighlightSpan],
    byte_offset: usize,
    theme: &crate::config::Theme,
) -> Option<Style> {
    let capture = spans
        .iter()
        .rev()
        .find(|s| s.start_byte <= byte_offset && byte_offset < s.end_byte)
        .map(|s| s.capture)?;

    let theme_style = theme.for_capture(capture)?;

    let mut style = Style::default();
    if let Some(fg) = theme_style.fg {
        style = style.fg(Color::Rgb(fg.r, fg.g, fg.b));
    }
    if let Some(bg) = theme_style.bg {
        style = style.bg(Color::Rgb(bg.r, bg.g, bg.b));
    }
    if theme_style.bold {
        style = style.add_modifier(Modifier::BOLD);
    }
    if theme_style.italic {
        style = style.add_modifier(Modifier::ITALIC);
    }
    if theme_style.underline {
        style = style.add_modifier(Modifier::UNDERLINED);
    }
    Some(style)
}

fn line_max_diagnostic_severity(
    diagnostics: &[DiagnosticItem],
    line_index: usize,
) -> Option<DiagnosticSeverityView> {
    diagnostics
        .iter()
        .filter(|diag| diag.line == line_index)
        .map(|diag| diag.severity)
        .max_by_key(|severity: &DiagnosticSeverityView| severity.rank())
}

fn single_line_diagnostic_message(message: &str) -> String {
    message
        .lines()
        .next()
        .unwrap_or_default()
        .trim()
        .chars()
        .take(96)
        .collect()
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

fn render_delete_confirm_overlay(frame: &mut Frame<'_>, app: &App) {
    if !matches!(app.mode(), AppMode::ConfirmDeleteExplorerEntry) {
        return;
    }

    let palette = Palette::mocha();

    // All styles here are fg-only so they sit cleanly on the popup background
    let blue = Style::default().fg(palette.blue);
    let bold_blue = Style::default()
        .fg(palette.blue)
        .add_modifier(Modifier::BOLD);
    let bold_fg = Style::default()
        .fg(palette.text)
        .add_modifier(Modifier::BOLD);
    let dim = Style::default().fg(palette.subtle);
    let border = Style::default().bg(palette.mantle).fg(palette.blue);

    let entry_name = app
        .explorer()
        .selected_entry()
        .and_then(|e| e.path.file_name())
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| String::from("this entry"));

    let popup = centered_rect(frame.area(), 40, 20);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border)
        .style(Style::default().bg(palette.mantle))
        .title(Span::styled(" \u{f1f8} Delete? ", bold_blue));
    let inner = block.inner(popup);
    let areas = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .split(inner);

    Clear.render(popup, frame.buffer_mut());
    block.render(popup, frame.buffer_mut());

    Paragraph::new(Line::from(vec![
        Span::styled("\u{f15b} ", blue),
        Span::styled(entry_name, bold_fg),
    ]))
    .render(areas[0], frame.buffer_mut());

    Paragraph::new(Line::from(Span::styled(
        "will be permanently deleted.",
        dim,
    )))
    .render(areas[1], frame.buffer_mut());

    Paragraph::new(Line::from(vec![
        Span::styled("y", bold_blue),
        Span::styled(" confirm  ", dim),
        Span::styled("n/Esc", bold_blue),
        Span::styled(" cancel", dim),
    ]))
    .render(areas[2], frame.buffer_mut());
}

fn render_external_change_conflict_overlay(frame: &mut Frame<'_>, app: &App) {
    if !matches!(app.mode(), AppMode::ExternalChangeConflict) {
        return;
    }

    let palette = Palette::mocha();
    let bold_blue = Style::default()
        .fg(palette.blue)
        .add_modifier(Modifier::BOLD);
    let bold_yellow = Style::default()
        .fg(palette.yellow)
        .add_modifier(Modifier::BOLD);
    let bold_fg = Style::default()
        .fg(palette.text)
        .add_modifier(Modifier::BOLD);
    let dim = Style::default().fg(palette.subtle);
    let border = Style::default().bg(palette.mantle).fg(palette.yellow);

    let file_name = app
        .current_conflict_path()
        .and_then(|p| p.file_name())
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| String::from("unknown file"));

    let popup = centered_rect(frame.area(), 44, 22);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border)
        .style(Style::default().bg(palette.mantle))
        .title(Span::styled(" \u{f071} File Changed on Disk ", bold_yellow));
    let inner = block.inner(popup);
    let areas = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .split(inner);

    Clear.render(popup, frame.buffer_mut());
    block.render(popup, frame.buffer_mut());

    Paragraph::new(Line::from(vec![
        Span::styled("\u{f15b} ", bold_yellow),
        Span::styled(file_name, bold_fg),
    ]))
    .render(areas[0], frame.buffer_mut());

    Paragraph::new(Line::from(Span::styled(
        "was modified outside the editor.",
        dim,
    )))
    .render(areas[1], frame.buffer_mut());

    Paragraph::new(Line::from(Span::styled(
        "You have unsaved changes in this buffer.",
        dim,
    )))
    .render(areas[2], frame.buffer_mut());

    Paragraph::new(Line::from(vec![
        Span::styled("y", bold_blue),
        Span::styled(" reload from disk  ", dim),
        Span::styled("n/Esc", bold_blue),
        Span::styled(" keep my changes", dim),
    ]))
    .render(areas[3], frame.buffer_mut());
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

    let viewport_height = areas[1].height as usize;
    let scroll_offset = picker.scroll_offset();

    let items = picker
        .items()
        .iter()
        .enumerate()
        .skip(scroll_offset)
        .take(viewport_height)
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

fn render_completion_overlay(frame: &mut Frame<'_>, app: &App) {
    if !app.completion_active() {
        return;
    }
    let area = frame.area();
    let Some((_, pane_area, pane)) = app
        .layout
        .leaves_in_area(if app.explorer().visible() {
            Layout::horizontal([Constraint::Length(30), Constraint::Min(1)]).split(area)[1]
        } else {
            area
        })
        .into_iter()
        .find(|(pane_id, _, _)| *pane_id == app.active_pane_id())
    else {
        return;
    };

    let max_items = app.lsp.completion.items.len().min(8);
    let widest = app
        .lsp
        .completion
        .items
        .iter()
        .take(max_items)
        .map(|item| item.label.chars().count() + item.detail.chars().count().min(28) + 3)
        .max()
        .unwrap_or(12) as u16;
    let width = widest.clamp(20, 48).min(area.width.saturating_sub(2));
    let height = (max_items as u16 + 2).clamp(3, 10);

    let gutter = gutter_width(app.active_document().line_count());
    let cursor_x = pane_area
        .x
        .saturating_add(1)
        .saturating_add(gutter)
        .saturating_add(
            app.active_document()
                .display_column(pane.cursor())
                .saturating_sub(pane.viewport().left_column()) as u16,
        );
    let cursor_y = pane_area.y.saturating_add(1).saturating_add(
        pane.cursor()
            .line
            .saturating_sub(pane.viewport().top_line()) as u16,
    );

    let x = cursor_x
        .saturating_sub(1)
        .min(area.right().saturating_sub(width));
    let mut y = cursor_y.saturating_add(1);
    if y.saturating_add(height) > area.bottom() {
        y = cursor_y.saturating_sub(height);
    }
    let popup = Rect::new(x, y.max(area.y), width, height);

    let block = Block::default().borders(Borders::ALL);
    let inner = block.inner(popup);
    Clear.render(popup, frame.buffer_mut());
    block.render(popup, frame.buffer_mut());

    let palette = Palette::mocha().styles();
    let items = app
        .lsp
        .completion
        .items
        .iter()
        .take(max_items)
        .enumerate()
        .map(|(index, item)| {
            let style = if index == app.lsp.completion.selected {
                palette.selection
            } else {
                palette.editor
            };
            let mut text = item.label.clone();
            if !item.detail.is_empty() {
                text.push_str("  ");
                let detail = if item.detail.chars().count() > 28 {
                    item.detail.chars().take(28).collect::<String>()
                } else {
                    item.detail.clone()
                };
                text.push_str(&detail);
            }
            ListItem::new(Line::from(Span::styled(text, style)))
        })
        .collect::<Vec<_>>();
    List::new(items).render(inner, frame.buffer_mut());
}

fn render_hover_overlay(frame: &mut Frame<'_>, app: &App) {
    if !app.lsp.hover.visible {
        return;
    }
    let popup = centered_rect(frame.area(), 60, 24);
    let block = Block::default().borders(Borders::ALL).title(" Hover ");
    let inner = block.inner(popup);
    Clear.render(popup, frame.buffer_mut());
    block.render(popup, frame.buffer_mut());
    Paragraph::new(app.lsp.hover.contents.clone()).render(inner, frame.buffer_mut());
}

fn render_signature_overlay(frame: &mut Frame<'_>, app: &App) {
    if !app.lsp.signature.visible {
        return;
    }
    let popup = centered_rect(frame.area(), 56, 14);
    let block = Block::default().borders(Borders::ALL).title(" Signature ");
    let inner = block.inner(popup);
    Clear.render(popup, frame.buffer_mut());
    block.render(popup, frame.buffer_mut());
    Paragraph::new(app.lsp.signature.label.clone()).render(inner, frame.buffer_mut());
}

fn render_diagnostics_overlay(frame: &mut Frame<'_>, app: &App) {
    if !app.lsp.diagnostics_panel_open {
        return;
    }
    let popup = centered_rect(frame.area(), 70, 40);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Diagnostics ");
    let inner = block.inner(popup);
    Clear.render(popup, frame.buffer_mut());
    block.render(popup, frame.buffer_mut());
    let lines = app
        .active_document()
        .path()
        .map(|path| {
            app.lsp
                .diagnostics_for_path(path)
                .iter()
                .map(|diag| {
                    format!(
                        "{}:{} {:?} {}",
                        diag.line + 1,
                        diag.column + 1,
                        diag.severity,
                        diag.message
                    )
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let content = if lines.is_empty() {
        String::from("No diagnostics")
    } else {
        lines.join("\n")
    };
    Paragraph::new(content).render(inner, frame.buffer_mut());
}

fn screen_cursor_position(
    app: &App,
    frame_area: Rect,
    editor_area: Rect,
    message_area: Rect,
) -> (u16, u16) {
    if app.picker().is_some() {
        let popup = centered_rect(frame_area, 70, 60);
        let x = popup.x.saturating_add(
            1 + app
                .picker()
                .map(|picker| picker.query().chars().count() as u16)
                .unwrap_or(0),
        );
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
    let y = area.y.saturating_add(1).saturating_add(
        pane.cursor()
            .line
            .saturating_sub(pane.viewport().top_line()) as u16,
    );
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
