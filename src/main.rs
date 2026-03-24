mod app;
mod core;
mod editor;
mod file;
mod keymap;
mod ui;
mod util;
mod view;

fn main() -> Result<(), app::AppError> {
    app::run()
}
