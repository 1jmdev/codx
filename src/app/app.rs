use std::path::{Path, PathBuf};

use crate::config::Theme;
use crate::core::{Document, History};
use crate::file::{ExplorerState, FileFinder, FileWatcher, RecentFiles};
use crate::syntax::{HighlightSpan, LanguageRegistry, SyntaxLayer, spans_for_line};
use crate::ui::{LayoutState, PickerState};
use crate::util::{Clipboard, DetectedEncoding};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    Editing,
    ConfirmQuit,
    ConfirmDeleteExplorerEntry,
    ExternalChangeConflict,
    CommandBar(CommandBarMode),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandBarMode {
    SaveAs,
    Search,
    Command,
    ExplorerCreateFile,
    ExplorerCreateDirectory,
    ExplorerRename,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageKind {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusTarget {
    Editor,
    Explorer,
}

#[derive(Debug, Default)]
pub(crate) struct CommandBarState {
    pub(crate) input: String,
    pub(crate) original_search_query: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct Message {
    pub(crate) text: String,
    pub(crate) kind: MessageKind,
}

pub struct BufferState {
    pub id: u64,
    pub document: Document,
    pub history: History,
    pub saved_snapshot: String,
    pub encoding: DetectedEncoding,
    pub syntax: SyntaxLayer,
}

pub struct App {
    pub(crate) workspace_root: PathBuf,
    pub(crate) buffers: Vec<BufferState>,
    pub(crate) active_buffer_id: u64,
    pub(crate) next_buffer_id: u64,
    pub(crate) layout: LayoutState,
    pub(crate) explorer: ExplorerState,
    pub(crate) file_finder: FileFinder,
    pub(crate) recent_files: RecentFiles,
    pub(crate) watcher: Option<FileWatcher>,
    pub(crate) picker: Option<PickerState>,
    pub(crate) pending_conflict_paths: Vec<PathBuf>,
    pub(crate) clipboard: Option<Clipboard>,
    pub(crate) focus: FocusTarget,
    pub(crate) mode: AppMode,
    pub(crate) should_quit: bool,
    pub(crate) pending_quit_after_save: bool,
    pub(crate) message: Option<Message>,
    pub(crate) command_bar: CommandBarState,
    pub(crate) active_theme: Theme,
}

impl App {
    pub fn open(path: Option<PathBuf>) -> Result<Self, crate::app::AppError> {
        crate::app::open_app(path)
    }

    pub fn run(&mut self) -> Result<(), crate::app::AppError> {
        crate::app::run_app(self)
    }

    pub fn active_buffer(&self) -> &BufferState {
        self.buffer_by_id(self.active_buffer_id)
            .unwrap_or_else(|| panic!("active buffer should exist"))
    }

    pub fn active_document(&self) -> &Document {
        &self.active_buffer().document
    }

    pub fn mode(&self) -> AppMode {
        self.mode
    }

    pub fn message(&self) -> Option<&str> {
        self.message.as_ref().map(|message| message.text.as_str())
    }

    pub fn message_kind(&self) -> MessageKind {
        self.message
            .as_ref()
            .map(|message| message.kind)
            .unwrap_or(MessageKind::Info)
    }

    pub fn command_bar_prefix(&self) -> Option<&'static str> {
        match self.mode {
            AppMode::CommandBar(CommandBarMode::SaveAs) => Some("Save as: "),
            AppMode::CommandBar(CommandBarMode::Search) => Some("Find: "),
            AppMode::CommandBar(CommandBarMode::Command) => Some(":"),
            AppMode::CommandBar(CommandBarMode::ExplorerCreateFile) => Some("New file: "),
            AppMode::CommandBar(CommandBarMode::ExplorerCreateDirectory) => Some("New dir: "),
            AppMode::CommandBar(CommandBarMode::ExplorerRename) => Some("Rename to: "),
            _ => None,
        }
    }

    pub fn command_bar_input(&self) -> Option<&str> {
        match self.mode {
            AppMode::CommandBar(_) => Some(&self.command_bar.input),
            _ => None,
        }
    }

    pub fn active_pane_id(&self) -> u64 {
        self.layout.focused_pane_id()
    }

    pub fn workspace_root(&self) -> &Path {
        &self.workspace_root
    }

    pub fn picker(&self) -> Option<&PickerState> {
        self.picker.as_ref()
    }

    pub fn explorer(&self) -> &ExplorerState {
        &self.explorer
    }

    pub fn focus(&self) -> FocusTarget {
        self.focus
    }

    pub fn current_conflict_path(&self) -> Option<&std::path::Path> {
        self.pending_conflict_paths.first().map(|p| p.as_path())
    }

    pub(crate) fn buffer_by_id(&self, buffer_id: u64) -> Option<&BufferState> {
        self.buffers.iter().find(|buffer| buffer.id == buffer_id)
    }

    pub(crate) fn buffer_by_id_mut(&mut self, buffer_id: u64) -> Option<&mut BufferState> {
        self.buffers.iter_mut().find(|buffer| buffer.id == buffer_id)
    }

    pub fn update_dirty_syntax_layers(&mut self) {
        for buffer in &mut self.buffers {
            if buffer.syntax.is_dirty() {
                let source = buffer.document.text();
                let _ = buffer.syntax.reparse(source.as_bytes());
            }
        }
    }

    pub fn syntax_spans_for_line(&self, buffer_id: u64, line_index: usize) -> Vec<HighlightSpan> {
        let Some(buffer) = self.buffer_by_id(buffer_id) else {
            return Vec::new();
        };
        let Some(tree) = buffer.syntax.tree() else {
            return Vec::new();
        };
        let Some(lang_id) = buffer.syntax.language_id() else {
            return Vec::new();
        };
        let registry = LanguageRegistry::global();
        let Some(query) = registry.highlight_query(lang_id) else {
            return Vec::new();
        };

        let source = buffer.document.text();
        let source_bytes = source.as_bytes();
        let line_start = buffer.document.line_to_byte(line_index);
        let line_end = if line_index + 1 < buffer.document.line_count() {
            buffer.document.line_to_byte(line_index + 1)
        } else {
            source_bytes.len()
        };

        spans_for_line(tree, query, source_bytes, line_start, line_end)
    }

    pub fn switch_theme(&mut self, name: &str) -> bool {
        match Theme::load_embedded(name) {
            Some(theme) => {
                self.active_theme = theme;
                true
            }
            None => false,
        }
    }

    pub fn active_theme(&self) -> &Theme {
        &self.active_theme
    }

    pub fn active_theme_name(&self) -> &str {
        &self.active_theme.name
    }

    pub(crate) fn active_pane(&self) -> &crate::ui::Pane {
        self.layout
            .focused_pane()
            .unwrap_or_else(|| panic!("active pane should exist"))
    }

    pub(crate) fn active_pane_mut(&mut self) -> &mut crate::ui::Pane {
        self.layout
            .focused_pane_mut()
            .unwrap_or_else(|| panic!("active pane should exist"))
    }
}
