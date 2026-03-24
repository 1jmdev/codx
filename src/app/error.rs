use crate::file::FileError;
use crate::ui::UiError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("terminal error: {0}")]
    Terminal(#[from] std::io::Error),
    #[error("ui error: {0}")]
    Ui(#[from] UiError),
    #[error("file error: {0}")]
    File(#[from] FileError),
    #[error("application invariant failed: {0}")]
    Invariant(String),
}
