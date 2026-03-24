use encoding_rs::{Encoding, UTF_8, UTF_16BE, UTF_16LE, WINDOWS_1252};
use thiserror::Error;

#[derive(Debug, Clone, Copy)]
pub struct DetectedEncoding {
    encoding: &'static Encoding,
    bom_len: usize,
}

impl Default for DetectedEncoding {
    fn default() -> Self {
        Self {
            encoding: UTF_8,
            bom_len: 0,
        }
    }
}

impl DetectedEncoding {
    pub fn encoding(self) -> &'static Encoding {
        self.encoding
    }

    pub fn bom_len(self) -> usize {
        self.bom_len
    }

    pub fn label(self) -> &'static str {
        self.encoding.name()
    }
}

#[derive(Debug, Error)]
pub enum EncodingError {
    #[error("file content could not be decoded cleanly")]
    Decode,
    #[error("file content could not be encoded as {0}")]
    Encode(&'static str),
}

pub fn detect_encoding(bytes: &[u8]) -> DetectedEncoding {
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return DetectedEncoding {
            encoding: UTF_8,
            bom_len: 3,
        };
    }

    if bytes.starts_with(&[0xFF, 0xFE]) {
        return DetectedEncoding {
            encoding: UTF_16LE,
            bom_len: 2,
        };
    }

    if bytes.starts_with(&[0xFE, 0xFF]) {
        return DetectedEncoding {
            encoding: UTF_16BE,
            bom_len: 2,
        };
    }

    if std::str::from_utf8(bytes).is_ok() {
        return DetectedEncoding::default();
    }

    DetectedEncoding {
        encoding: WINDOWS_1252,
        bom_len: 0,
    }
}

pub fn decode_text(bytes: &[u8]) -> Result<(String, DetectedEncoding), EncodingError> {
    let detected = detect_encoding(bytes);
    let (text, _, had_errors) = detected.encoding().decode(&bytes[detected.bom_len()..]);
    if had_errors {
        return Err(EncodingError::Decode);
    }

    Ok((text.into_owned(), detected))
}

pub fn encode_text(text: &str, detected: DetectedEncoding) -> Result<Vec<u8>, EncodingError> {
    let (encoded, _, had_errors) = detected.encoding().encode(text);
    if had_errors {
        return Err(EncodingError::Encode(detected.label()));
    }

    let mut bytes = Vec::with_capacity(detected.bom_len() + encoded.len());
    if detected.encoding() == UTF_8 && detected.bom_len() == 3 {
        bytes.extend_from_slice(&[0xEF, 0xBB, 0xBF]);
    } else if detected.encoding() == UTF_16LE && detected.bom_len() == 2 {
        bytes.extend_from_slice(&[0xFF, 0xFE]);
    } else if detected.encoding() == UTF_16BE && detected.bom_len() == 2 {
        bytes.extend_from_slice(&[0xFE, 0xFF]);
    }
    bytes.extend_from_slice(&encoded);
    Ok(bytes)
}
