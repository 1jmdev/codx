use std::path::{Path, PathBuf};

use crate::app::{App, AppError, AppMode, BufferState, FocusTarget, MessageKind, Theme};
use crate::file::{load_document, ExplorerState, FileFinder, FileWatcher, RecentFiles};
use crate::syntax::{SyntaxLayer, language_for_path};
use crate::util::{Clipboard, DetectedEncoding};

pub(crate) fn open_app(path: Option<PathBuf>) -> Result<App, AppError> {
    let workspace_root = resolve_workspace_root(path.as_deref());
    let clipboard = Clipboard::new().ok();
    let recent_files = RecentFiles::load();
    let watcher = FileWatcher::new(&workspace_root).ok();

    let (document, encoding) = match path {
        Some(path) if path.exists() => {
            let loaded = load_document(&path)?;
            (loaded.document, loaded.encoding)
        }
        Some(path) => (crate::core::Document::new_empty(Some(path)), DetectedEncoding::default()),
        None => (crate::core::Document::new_empty(None), DetectedEncoding::default()),
    };

    let saved_snapshot = document.text();
    let language_id = document.path().and_then(language_for_path);
    let mut syntax = SyntaxLayer::new(language_id);
    let _ = syntax.reparse(saved_snapshot.as_bytes());

    let initial_buffer = BufferState {
        id: 1,
        document,
        history: crate::core::History::default(),
        saved_snapshot,
        encoding,
        syntax,
    };

    let active_theme = Theme::default_theme();

    let mut app = App {
        workspace_root: workspace_root.clone(),
        active_buffer_id: 1,
        next_buffer_id: 2,
        layout: crate::ui::LayoutState::new(1),
        explorer: ExplorerState::new(workspace_root.clone()),
        file_finder: FileFinder::new(workspace_root),
        recent_files,
        watcher,
        pending_conflict_paths: Vec::new(),
        buffers: vec![initial_buffer],
        picker: None,
        clipboard,
        focus: FocusTarget::Editor,
        mode: AppMode::Editing,
        should_quit: false,
        pending_quit_after_save: false,
        message: None,
        command_bar: crate::app::CommandBarState::default(),
        active_theme,
    };

    if let Some(path) = app.active_document().path().map(Path::to_path_buf) {
        app.recent_files.record(&path);
    }

    if app.clipboard.is_none() {
        app.set_message(
            "System clipboard is unavailable in this environment",
            MessageKind::Warning,
        );
    }

    Ok(app)
}

fn resolve_workspace_root(path: Option<&Path>) -> PathBuf {
    match path {
        Some(path) if path.is_dir() => path.to_path_buf(),
        Some(path) => path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))),
        None => std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
    }
}
