use std::io::{self, BufRead};

use serde_json::Value;

pub(crate) fn read_message<R: BufRead>(reader: &mut R) -> io::Result<Option<Value>> {
    let mut content_length = None;
    loop {
        let mut line = String::new();
        let bytes = reader.read_line(&mut line)?;
        if bytes == 0 {
            return Ok(None);
        }

        if line == "\r\n" {
            break;
        }

        if let Some(value) = line.strip_prefix("Content-Length:") {
            content_length = value.trim().parse::<usize>().ok();
        }
    }

    let length = content_length.ok_or_else(|| io::Error::other("Missing Content-Length"))?;
    let mut body = vec![0_u8; length];
    std::io::Read::read_exact(reader, &mut body)?;
    let payload: Value = serde_json::from_slice(&body)
        .map_err(|error| io::Error::other(format!("Invalid LSP JSON: {error}")))?;
    Ok(Some(payload))
}
