mod app;
mod error;
mod startup;
mod terminal;
mod tests;

pub use crate::config::Theme;
pub(crate) use crate::file::open_app;
pub(crate) use crate::ui::run_app;
pub use app::App;
pub use app::AppMode;
pub(crate) use app::BufferState;
#[allow(unused_imports)]
pub use app::CommandBarMode;
pub(crate) use app::CommandBarState;
pub use app::FocusTarget;
pub(crate) use app::FoldCache;
pub(crate) use app::LineHighlightCache;
pub(crate) use app::Message;
pub use app::MessageKind;
pub use error::AppError;
pub use startup::run;
pub(crate) use terminal::TerminalSession;
