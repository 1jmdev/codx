mod editor;
mod input;
mod lsp;
mod state;
mod syntax;
mod tree;
mod ui;

pub use state::App;
pub(crate) use state::{Focus, TreeItem, rect_contains};
