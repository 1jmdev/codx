use thiserror::Error;

#[allow(dead_code)]
#[derive(Debug, Error)]
pub enum UiError {
    #[error("ui failure: {0}")]
    Message(String),
}
