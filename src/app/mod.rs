mod app;
mod error;
mod startup;
mod terminal;
mod tests;

pub use app::App;
pub use app::AppMode;
pub(crate) use app::BufferState;
pub use crate::config::Theme;
#[allow(unused_imports)]
pub use app::CommandBarMode;
pub use app::FocusTarget;
pub use app::MessageKind;
pub(crate) use app::CommandBarState;
pub(crate) use app::Message;
pub use error::AppError;
pub(crate) use crate::file::open_app;
pub(crate) use terminal::TerminalSession;
pub(crate) use crate::ui::run_app;
pub use startup::run;
