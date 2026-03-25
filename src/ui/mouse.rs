use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::{Constraint, Layout, Rect};

use crate::app::{App, AppError, AppMode, FocusTarget};
use crate::core::{Cursor, Selection};

const WHEEL_STEP: isize = 1;
const WHEEL_SLOWDOWN: u8 = 3;

pub(crate) fn handle_mouse_event(app: &mut App, mouse_event: MouseEvent) -> Result<(), AppError> {
    if !matches!(app.mode(), AppMode::Editing) {
        return Ok(());
    }

    if app.picker().is_some() {
        return handle_picker_mouse_event(app, mouse_event);
    }

    match mouse_event.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            handle_left_pointer_event(app, mouse_event, false)
        }
        MouseEventKind::Drag(MouseButton::Left) => {
            handle_left_pointer_event(app, mouse_event, true)
        }
        MouseEventKind::ScrollUp => {
            if pointer_in_explorer(app, mouse_event.column, mouse_event.row) {
                if !consume_wheel_tick(&mut app.mouse_wheel_explorer_accum) {
                    return Ok(());
                }
                app.focus = FocusTarget::Explorer;
                let height = explorer_viewport_height(app);
                app.explorer.scroll_by(-WHEEL_STEP, height);
                keep_explorer_selection_under_pointer(app, mouse_event.row);
            } else if let Some((pane_id, _)) =
                pane_under_pointer(app, mouse_event.column, mouse_event.row)
            {
                if !consume_wheel_tick(&mut app.mouse_wheel_editor_accum) {
                    return Ok(());
                }
                focus_editor_pane(app, pane_id);
                if let Some(pane) = app.layout.pane_mut(pane_id) {
                    let top = pane.viewport().top_line();
                    pane.viewport_mut()
                        .set_top_line(top.saturating_sub(WHEEL_STEP as usize));
                }
            }
            Ok(())
        }
        MouseEventKind::ScrollDown => {
            if pointer_in_explorer(app, mouse_event.column, mouse_event.row) {
                if !consume_wheel_tick(&mut app.mouse_wheel_explorer_accum) {
                    return Ok(());
                }
                app.focus = FocusTarget::Explorer;
                let height = explorer_viewport_height(app);
                app.explorer.scroll_by(WHEEL_STEP, height);
                keep_explorer_selection_under_pointer(app, mouse_event.row);
            } else if let Some((pane_id, _)) =
                pane_under_pointer(app, mouse_event.column, mouse_event.row)
            {
                if !consume_wheel_tick(&mut app.mouse_wheel_editor_accum) {
                    return Ok(());
                }
                focus_editor_pane(app, pane_id);
                let line_count = app.active_document().line_count();
                if let Some(pane) = app.layout.pane_mut(pane_id) {
                    let next = pane
                        .viewport()
                        .top_line()
                        .saturating_add(WHEEL_STEP as usize);
                    let max_top = line_count.saturating_sub(pane.viewport().text_height());
                    pane.viewport_mut().set_top_line(next.min(max_top));
                }
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

fn handle_left_pointer_event(
    app: &mut App,
    mouse_event: MouseEvent,
    extend_selection: bool,
) -> Result<(), AppError> {
    if pointer_in_explorer(app, mouse_event.column, mouse_event.row) {
        app.focus = FocusTarget::Explorer;
        click_explorer(app, mouse_event.column, mouse_event.row)?;
        return Ok(());
    }

    if let Some((pane_id, pane_area)) = pane_under_pointer(app, mouse_event.column, mouse_event.row)
    {
        focus_editor_pane(app, pane_id);
        let position = mouse_to_cursor(app, pane_area, mouse_event.column, mouse_event.row);
        if let Some(cursor) = position {
            let selection = if extend_selection {
                app.active_pane().selection().with_active(cursor)
            } else {
                Selection::caret(cursor)
            };
            if let Some(pane) = app.layout.pane_mut(pane_id) {
                pane.set_cursor(cursor);
                pane.set_selection(selection);
            }
            app.ensure_cursor_visible_minimal();
        }
    }

    Ok(())
}

fn click_explorer(app: &mut App, column: u16, row: u16) -> Result<(), AppError> {
    let Some((outer, inner)) = explorer_areas(app) else {
        return Ok(());
    };
    if !contains(outer, column, row) || !contains(inner, column, row) {
        return Ok(());
    }

    let index = app
        .explorer
        .scroll_offset()
        .saturating_add(row.saturating_sub(inner.y) as usize);
    if index >= app.explorer.entries().len() {
        return Ok(());
    }

    app.explorer.set_selected(index);
    app.open_selected_explorer_entry()
}

fn handle_picker_mouse_event(app: &mut App, mouse_event: MouseEvent) -> Result<(), AppError> {
    match mouse_event.kind {
        MouseEventKind::ScrollUp => {
            if !consume_wheel_tick(&mut app.mouse_wheel_picker_accum) {
                return Ok(());
            }
            let height = picker_viewport_height(app);
            if let Some(picker) = app.picker.as_mut() {
                picker.scroll_by(-WHEEL_STEP, height);
            }
            keep_picker_selection_under_pointer(app, mouse_event.row);
            Ok(())
        }
        MouseEventKind::ScrollDown => {
            if !consume_wheel_tick(&mut app.mouse_wheel_picker_accum) {
                return Ok(());
            }
            let height = picker_viewport_height(app);
            if let Some(picker) = app.picker.as_mut() {
                picker.scroll_by(WHEEL_STEP, height);
            }
            keep_picker_selection_under_pointer(app, mouse_event.row);
            Ok(())
        }
        MouseEventKind::Down(MouseButton::Left) => {
            click_picker(app, mouse_event.column, mouse_event.row)
        }
        MouseEventKind::Moved => {
            hover_picker(app, mouse_event.column, mouse_event.row);
            Ok(())
        }
        _ => Ok(()),
    }
}

fn hover_picker(app: &mut App, column: u16, row: u16) {
    let Some((list_area, scroll_offset)) = picker_list_area(app) else {
        return;
    };
    if !contains(list_area, column, row) {
        return;
    }

    let index = scroll_offset.saturating_add(row.saturating_sub(list_area.y) as usize);
    if let Some(picker) = app.picker.as_mut() {
        if index < picker.items().len() {
            picker.set_selected(index);
        }
    }
}

fn keep_explorer_selection_under_pointer(app: &mut App, row: u16) {
    let Some((_, inner)) = explorer_areas(app) else {
        return;
    };
    if row < inner.y || row >= inner.bottom() {
        return;
    }

    let index = app
        .explorer
        .scroll_offset()
        .saturating_add(row.saturating_sub(inner.y) as usize);
    if index < app.explorer.entries().len() {
        app.explorer.set_selected(index);
    }
}

fn keep_picker_selection_under_pointer(app: &mut App, row: u16) {
    let Some((list_area, scroll_offset)) = picker_list_area(app) else {
        return;
    };
    if row < list_area.y || row >= list_area.bottom() {
        return;
    }

    let index = scroll_offset.saturating_add(row.saturating_sub(list_area.y) as usize);
    if let Some(picker) = app.picker.as_mut() {
        if index < picker.items().len() {
            picker.set_selected(index);
        }
    }
}

fn click_picker(app: &mut App, column: u16, row: u16) -> Result<(), AppError> {
    let Some((list_area, scroll_offset)) = picker_list_area(app) else {
        return Ok(());
    };
    if !contains(list_area, column, row) {
        return Ok(());
    }

    let index = scroll_offset.saturating_add(row.saturating_sub(list_area.y) as usize);
    if let Some(picker) = app.picker.as_mut() {
        if index >= picker.items().len() {
            return Ok(());
        }
        picker.set_selected(index);
    }
    app.accept_picker_selection()
}

fn picker_list_area(app: &App) -> Option<(Rect, usize)> {
    let picker = app.picker()?;
    let workspace = workspace_area(app);
    let popup = centered_rect(workspace, 70, 60);
    if popup.width < 3 || popup.height < 3 {
        return None;
    }
    let inner = Rect {
        x: popup.x.saturating_add(1),
        y: popup.y.saturating_add(1),
        width: popup.width.saturating_sub(2),
        height: popup.height.saturating_sub(2),
    };
    let list_area = Layout::vertical([Constraint::Length(1), Constraint::Min(1)]).split(inner)[1];
    let adjusted = Rect {
        x: list_area.x,
        y: list_area.y.saturating_add(1),
        width: list_area.width,
        height: list_area.height.saturating_sub(1),
    };
    Some((adjusted, picker.scroll_offset()))
}

fn picker_viewport_height(app: &App) -> usize {
    picker_list_area(app)
        .map(|(area, _)| area.height as usize)
        .unwrap_or(0)
}

fn explorer_viewport_height(app: &App) -> usize {
    explorer_areas(app)
        .map(|(_, inner)| inner.height as usize)
        .unwrap_or(0)
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

fn mouse_to_cursor(app: &App, pane_area: Rect, column: u16, row: u16) -> Option<Cursor> {
    if pane_area.width < 3 || pane_area.height < 3 {
        return None;
    }

    let inner = Rect {
        x: pane_area.x.saturating_add(1),
        y: pane_area.y.saturating_add(1),
        width: pane_area.width.saturating_sub(2),
        height: pane_area.height.saturating_sub(2),
    };
    if !contains(inner, column, row) {
        return None;
    }

    let gutter = gutter_width(app.active_document().line_count());
    let text_x = inner.x.saturating_add(gutter);
    let line = app
        .active_pane()
        .viewport()
        .top_line()
        .saturating_add(row.saturating_sub(inner.y) as usize)
        .min(app.active_document().last_line_index());
    if column < text_x {
        return Some(Cursor::new(line, 0));
    }

    let display_column = app
        .active_pane()
        .viewport()
        .left_column()
        .saturating_add(column.saturating_sub(text_x) as usize);
    let char_column = app
        .active_document()
        .column_for_display(line, display_column);
    Some(Cursor::new(line, char_column).with_preferred_column(display_column))
}

fn pane_under_pointer(app: &App, column: u16, row: u16) -> Option<(u64, Rect)> {
    let editor_area = workspace_editor_area(app)?;
    app.layout
        .leaves_in_area(editor_area)
        .into_iter()
        .find(|(_, pane_area, _)| contains(*pane_area, column, row))
        .map(|(pane_id, pane_area, _)| (pane_id, pane_area))
}

fn pointer_in_explorer(app: &App, column: u16, row: u16) -> bool {
    explorer_areas(app)
        .map(|(outer, _)| contains(outer, column, row))
        .unwrap_or(false)
}

fn explorer_areas(app: &App) -> Option<(Rect, Rect)> {
    if !app.explorer.visible() {
        return None;
    }
    let workspace = workspace_area(app);
    let chunks = Layout::horizontal([Constraint::Length(30), Constraint::Min(1)]).split(workspace);
    let outer = chunks[0];
    if outer.width < 3 || outer.height < 3 {
        return Some((outer, outer));
    }
    let inner = Rect {
        x: outer.x.saturating_add(1),
        y: outer.y.saturating_add(1),
        width: outer.width.saturating_sub(2),
        height: outer.height.saturating_sub(2),
    };
    Some((outer, inner))
}

fn workspace_editor_area(app: &App) -> Option<Rect> {
    let workspace = workspace_area(app);
    if app.explorer.visible() {
        let chunks =
            Layout::horizontal([Constraint::Length(30), Constraint::Min(1)]).split(workspace);
        return Some(chunks[1]);
    }
    Some(workspace)
}

fn workspace_area(app: &App) -> Rect {
    let size = terminal_size(app);
    let root = Rect::new(0, 0, size.width, size.height);
    Layout::vertical([
        Constraint::Min(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .split(root)[0]
}

fn terminal_size(app: &App) -> ratatui::layout::Size {
    app.layout
        .focused_pane()
        .map(|pane| pane.viewport().terminal_size())
        .unwrap_or_else(|| ratatui::layout::Size::new(120, 30))
}

fn focus_editor_pane(app: &mut App, pane_id: u64) {
    app.focus = FocusTarget::Editor;
    app.layout.focus_pane(pane_id);
    if let Some(buffer_id) = app.layout.pane(pane_id).map(|pane| pane.buffer_id()) {
        app.active_buffer_id = buffer_id;
    }
}

fn contains(rect: Rect, column: u16, row: u16) -> bool {
    column >= rect.x && column < rect.right() && row >= rect.y && row < rect.bottom()
}

fn gutter_width(line_count: usize) -> u16 {
    let digits = line_count.max(1).to_string().chars().count() as u16;
    digits.saturating_add(2).max(5)
}

fn consume_wheel_tick(counter: &mut u8) -> bool {
    *counter = counter.saturating_add(1);
    if *counter >= WHEEL_SLOWDOWN {
        *counter = 0;
        true
    } else {
        false
    }
}
