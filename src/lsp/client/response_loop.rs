use std::io::BufReader;
use std::process::ChildStdout;
use std::sync::mpsc::{self, Receiver};
use std::thread;

use serde_json::Value;

use crate::lsp::client::RpcResponse;

pub(super) enum IncomingMessage {
    Notification(Value),
    ServerRequest(Value),
    Response(RpcResponse),
}

pub(super) fn spawn_response_loop(stdout: ChildStdout) -> Receiver<IncomingMessage> {
    let (tx, receiver) = mpsc::channel::<IncomingMessage>();
    thread::spawn(move || {
        let mut reader = BufReader::new(stdout);
        while let Ok(message) = crate::lsp::protocol::read_message(&mut reader) {
            let Some(payload) = message else {
                break;
            };

            if payload.get("method").is_some() && payload.get("id").is_some() {
                let _ = tx.send(IncomingMessage::ServerRequest(payload));
                continue;
            }

            if payload.get("method").is_some() {
                let _ = tx.send(IncomingMessage::Notification(payload));
                continue;
            }

            let id = payload.get("id").and_then(Value::as_u64);
            if let Some(id) = id {
                let _ = tx.send(IncomingMessage::Response(RpcResponse {
                    id,
                    result: payload.get("result").cloned(),
                    error: payload.get("error").cloned(),
                }));
            }
        }
    });
    receiver
}
