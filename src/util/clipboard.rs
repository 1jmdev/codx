use arboard::Clipboard as SystemClipboard;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ClipboardError {
    #[error("clipboard is unavailable: {0}")]
    Unavailable(String),
}

pub struct Clipboard {
    system: SystemClipboard,
}

impl Clipboard {
    pub fn new() -> Result<Self, ClipboardError> {
        let system = SystemClipboard::new()
            .map_err(|error| ClipboardError::Unavailable(error.to_string()))?;
        Ok(Self { system })
    }

    pub fn copy(&mut self, text: &str) -> Result<(), ClipboardError> {
        self.system
            .set_text(text.to_owned())
            .map_err(|error| ClipboardError::Unavailable(error.to_string()))
    }

    pub fn paste(&mut self) -> Result<String, ClipboardError> {
        self.system
            .get_text()
            .map_err(|error| ClipboardError::Unavailable(error.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use crate::util::ClipboardError;

    #[test]
    fn clipboard_error_displays_message() {
        let error = ClipboardError::Unavailable(String::from("no provider"));
        assert!(error.to_string().contains("clipboard"));
    }
}
