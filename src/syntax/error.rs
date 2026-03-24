use thiserror::Error;

#[allow(dead_code)]
#[derive(Debug, Error)]
pub enum SyntaxError {
    #[error("unsupported language")]
    UnsupportedLanguage,
    #[error("failed to set language on parser: {0}")]
    LanguageSetFailed(String),
    #[error("failed to compile query: {0}")]
    QueryCompileFailed(String),
    #[error("parse returned no tree")]
    ParseFailed,
}
