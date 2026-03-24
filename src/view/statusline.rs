use crate::app::App;

pub fn build_statusline(app: &App) -> String {
    let file_name = app
        .active_document()
        .path()
        .and_then(|path| path.file_name())
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| String::from("[No Name]"));
    let dirty = if app.active_document().is_dirty() { " [+]" } else { "" };
    let read_only = if app.is_read_only() { " [RO]" } else { "" };
    let line = app.active_pane().cursor().line + 1;
    let column = app.display_column() + 1;
    let total_lines = app.active_document().line_count();
    let pane_count = app.layout.pane_ids().len();
    let encoding = app.active_buffer().encoding.label();
    let theme_name = app.active_theme_name();

    format!(
        "{file_name}{dirty}{read_only}  Ln {line}, Col {column}  {total_lines} lines  {encoding}  {theme_name}  {pane_count} pane(s)"
    )
}
