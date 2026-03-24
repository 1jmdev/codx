mod app;
mod config;
mod core;
mod editor;
mod file;
mod keymap;
mod lsp;
mod syntax;
mod ui;
mod util;
mod view;

fn main() -> Result<(), app::AppError> {
    app::run()
}
