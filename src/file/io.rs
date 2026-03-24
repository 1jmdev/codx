use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use tempfile::NamedTempFile;
use thiserror::Error;

use crate::core::Document;
use crate::util::{DetectedEncoding, EncodingError, decode_text, encode_text};

#[derive(Debug)]
pub struct LoadedDocument {
    pub document: Document,
    pub encoding: DetectedEncoding,
}

#[derive(Debug, Error)]
pub enum FileError {
    #[error("path is a directory: {0}")]
    Directory(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("encoding error for {path}: {source}")]
    Encoding {
        path: String,
        #[source]
        source: EncodingError,
    },
    #[error("notify error: {0}")]
    Notify(#[from] notify::Error),
}

pub fn load_document(path: &Path) -> Result<LoadedDocument, FileError> {
    if path.is_dir() {
        return Err(FileError::Directory(path.display().to_string()));
    }

    let bytes = fs::read(path)?;
    let (text, encoding) = decode_text(&bytes).map_err(|source| FileError::Encoding {
        path: path.display().to_string(),
        source,
    })?;
    Ok(LoadedDocument {
        document: Document::from_text(Some(path.to_path_buf()), &text),
        encoding,
    })
}

pub fn save_document(
    path: &Path,
    document: &Document,
    encoding: DetectedEncoding,
) -> Result<(), FileError> {
    if path.is_dir() {
        return Err(FileError::Directory(path.display().to_string()));
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let bytes = encode_text(&document.text(), encoding).map_err(|source| FileError::Encoding {
        path: path.display().to_string(),
        source,
    })?;
    atomic_write(path, &bytes)?;
    Ok(())
}

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), FileError> {
    let parent = path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    let mut temp = NamedTempFile::new_in(parent)?;
    temp.write_all(bytes)?;
    temp.flush()?;
    temp.persist(path)
        .map_err(|error| FileError::Io(error.error))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::core::{Cursor, Document};
    use crate::file::{load_document, save_document};
    use crate::util::DetectedEncoding;

    #[test]
    fn load_and_save_round_trip_utf8() {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("codx-{suffix}.txt"));
        let mut document = Document::new_empty(Some(path.clone()));
        document.insert_text(Cursor::new(0, 0), "hello\nworld");
        save_document(&path, &document, DetectedEncoding::default())
            .unwrap_or_else(|error| panic!("{error}"));
        let loaded = load_document(&path).unwrap_or_else(|error| panic!("{error}"));
        assert_eq!(loaded.document.text(), "hello\nworld");
        let _ = fs::remove_file(path);
    }
}
