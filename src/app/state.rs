use std::{
    collections::HashSet,
    io,
    path::PathBuf,
    time::{Duration, Instant},
};

use crossterm::event::{self, Event};
use ratatui::{Terminal, layout::Rect};

use crate::app::{
    lsp::LspManager, palette::PaletteState, search::SearchReplaceState, syntax::SyntaxEngine,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Focus {
    Editor,
    FileTree,
}

#[derive(Clone, Debug)]
pub(crate) struct TreeItem {
    pub(crate) path: PathBuf,
    pub(crate) name: String,
    pub(crate) depth: usize,
    pub(crate) is_dir: bool,
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct UiGeometry {
    pub(crate) editor_inner: Rect,
    pub(crate) tree_inner: Rect,
    pub(crate) palette_inner: Rect,
    pub(crate) palette_results: Rect,
    pub(crate) palette_preview: Rect,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CursorPos {
    pub(crate) line: usize,
    pub(crate) col: usize,
}

#[derive(Clone, Debug)]
pub(crate) struct EditorSnapshot {
    pub(crate) lines: Vec<String>,
    pub(crate) cursor_line: usize,
    pub(crate) cursor_col: usize,
    pub(crate) preferred_col: usize,
    pub(crate) selection_anchor: Option<CursorPos>,
}

#[derive(Clone, Debug)]
pub(crate) struct CompletionItem {
    pub(crate) label: String,
    pub(crate) insert_text: String,
    pub(crate) match_text: String,
    pub(crate) replace_start_col: usize,
    pub(crate) replace_end_col: usize,
    pub(crate) sort_text: String,
}

#[derive(Clone, Debug)]
pub(crate) struct CompletionState {
    pub(crate) line: usize,
    pub(crate) anchor_col: usize,
    pub(crate) selected: usize,
    pub(crate) scroll: usize,
    pub(crate) items: Vec<CompletionItem>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MouseSelectMode {
    Char,
    Word,
}

pub struct App {
    pub(crate) cwd: PathBuf,
    pub(crate) focus: Focus,
    pub(crate) sidebar_open: bool,
    pub(crate) status: String,
    pub(crate) should_quit: bool,
    pub(crate) tree_items: Vec<TreeItem>,
    pub(crate) expanded_dirs: HashSet<PathBuf>,
    pub(crate) tree_selected: usize,
    pub(crate) tree_scroll: usize,
    pub(crate) current_file: Option<PathBuf>,
    pub(crate) lines: Vec<String>,
    pub(crate) cursor_line: usize,
    pub(crate) cursor_col: usize,
    pub(crate) preferred_col: usize,
    pub(crate) selection_anchor: Option<CursorPos>,
    pub(crate) editor_scroll: usize,
    pub(crate) undo_stack: Vec<EditorSnapshot>,
    pub(crate) redo_stack: Vec<EditorSnapshot>,
    pub(crate) dirty: bool,
    pub(crate) ui: UiGeometry,
    pub(crate) syntax: SyntaxEngine,
    pub(crate) lsp: LspManager,
    pub(crate) palette: Option<PaletteState>,
    pub(crate) completion: Option<CompletionState>,
    pub(crate) file_picker_cache: Vec<PathBuf>,
    pub(crate) search_replace: Option<SearchReplaceState>,
    pub(crate) mouse_selecting: bool,
    pub(crate) mouse_select_mode: MouseSelectMode,
    pub(crate) word_select_origin: Option<(CursorPos, CursorPos)>,
    pub(crate) last_left_click: Option<(Instant, u16, u16)>,
}

impl App {
    pub fn new(cwd: PathBuf) -> Self {
        let mut expanded_dirs = HashSet::new();
        expanded_dirs.insert(cwd.clone());

        let mut app = Self {
            cwd: cwd.clone(),
            focus: Focus::FileTree,
            sidebar_open: true,
            status: String::from("Welcome. Mouse + Tab supported."),
            should_quit: false,
            tree_items: Vec::new(),
            expanded_dirs,
            tree_selected: 0,
            tree_scroll: 0,
            current_file: None,
            lines: vec![String::new()],
            cursor_line: 0,
            cursor_col: 0,
            preferred_col: 0,
            selection_anchor: None,
            editor_scroll: 0,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            dirty: false,
            ui: UiGeometry::default(),
            syntax: SyntaxEngine::new(),
            lsp: LspManager::new(cwd),
            palette: None,
            completion: None,
            file_picker_cache: Vec::new(),
            search_replace: None,
            mouse_selecting: false,
            mouse_select_mode: MouseSelectMode::Char,
            word_select_origin: None,
            last_left_click: None,
        };

        app.rebuild_tree();
        app
    }

    pub fn run<B: ratatui::backend::Backend>(
        &mut self,
        terminal: &mut Terminal<B>,
    ) -> io::Result<()>
    where
        io::Error: From<B::Error>,
    {
        loop {
            self.lsp.poll(&mut self.status);
            self.refresh_completion_from_lsp();
            terminal.draw(|frame| self.draw(frame))?;

            if self.should_quit {
                break;
            }

            if event::poll(Duration::from_millis(100))? {
                match event::read()? {
                    Event::Key(key) => self.on_key(key),
                    Event::Mouse(mouse) => self.on_mouse(mouse),
                    Event::Resize(_, _) => {}
                    _ => {}
                }
            }
        }

        Ok(())
    }

    pub(crate) fn notify_lsp_change(&mut self) {
        if let Some(path) = self.current_file.as_ref() {
            self.lsp
                .did_change(path, self.lines.join("\n"), &mut self.status);
        }
    }

    pub(crate) fn notify_lsp_save(&mut self) {
        if let Some(path) = self.current_file.as_ref() {
            self.lsp.did_save(path, &mut self.status);
        }
    }
}

pub(crate) fn rect_contains(rect: Rect, x: u16, y: u16) -> bool {
    x >= rect.x && x < rect.x + rect.width && y >= rect.y && y < rect.y + rect.height
}
