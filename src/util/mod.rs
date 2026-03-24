mod clipboard;
mod encoding;

#[allow(unused_imports)]
pub use clipboard::{Clipboard, ClipboardError};
pub use encoding::{decode_text, encode_text, DetectedEncoding, EncodingError};
