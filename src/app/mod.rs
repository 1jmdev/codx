mod completion;
mod editor;
mod input;
mod lsp;
mod palette;
pub(crate) mod search;
mod state;
mod syntax;
mod tree;
mod ui;

pub use state::App;
pub(crate) use state::{Focus, TreeItem, rect_contains};
