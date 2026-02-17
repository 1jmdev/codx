use std::{collections::HashSet, io, path::PathBuf, time::Duration};

use crossterm::event::{self, Event};
use ratatui::{layout::Rect, Terminal};

mod editor;
mod input;
mod tree;
mod ui;

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
}

#[derive(Debug)]
pub struct App {
    pub(crate) cwd: PathBuf,
    pub(crate) focus: Focus,
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
    pub(crate) editor_scroll: usize,
    pub(crate) dirty: bool,

    pub(crate) ui: UiGeometry,
}

impl App {
    pub fn new(cwd: PathBuf) -> Self {
        let mut expanded_dirs = HashSet::new();
        expanded_dirs.insert(cwd.clone());

        let mut app = Self {
            cwd,
            focus: Focus::FileTree,
            status: String::from("Welcome. Use mouse or Tab to switch focus."),
            should_quit: false,
            tree_items: Vec::new(),
            expanded_dirs,
            tree_selected: 0,
            tree_scroll: 0,
            current_file: None,
            lines: vec![String::new()],
            cursor_line: 0,
            cursor_col: 0,
            editor_scroll: 0,
            dirty: false,
            ui: UiGeometry::default(),
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
}

pub(crate) fn rect_contains(rect: Rect, x: u16, y: u16) -> bool {
    x >= rect.x && x < rect.x + rect.width && y >= rect.y && y < rect.y + rect.height
}
