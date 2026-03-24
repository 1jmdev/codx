use std::collections::HashMap;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin};
use std::sync::mpsc::Receiver;
use std::time::Duration;

use lsp_types::{InitializeResult, InitializedParams};
use serde_json::Value;

use crate::lsp::capabilities::{default_client_capabilities, negotiate, NegotiatedCapabilities};
use crate::lsp::client::bootstrap::launch_client;
use crate::lsp::client::response_loop::IncomingMessage;
use crate::lsp::client::uri::file_uri;
use crate::lsp::client::{RpcResponse, ServerConfig};

#[derive(Debug)]
pub struct LspClient {
    pub language_id: String,
    pub root_path: PathBuf,
    pub capabilities: NegotiatedCapabilities,
    pub initialized: bool,
    pub(super) child: Child,
    pub(super) stdin: ChildStdin,
    pub(super) receiver: Receiver<IncomingMessage>,
    pub(super) queued_notifications: Vec<Value>,
    pub(super) queued_responses: HashMap<u64, RpcResponse>,
    pub(super) next_request_id: u64,
}

impl LspClient {
    pub async fn launch(config: &ServerConfig, root_path: &Path) -> Result<Self, String> {
        let mut client = launch_client(config, root_path)?;
        client.initialize().await?;
        Ok(client)
    }

    async fn initialize(&mut self) -> Result<(), String> {
        let init_value = serde_json::json!({
            "processId": std::process::id(),
            "rootUri": file_uri(&self.root_path),
            "capabilities": default_client_capabilities(),
            "workspaceFolders": [
                {
                    "uri": file_uri(&self.root_path),
                    "name": "workspace"
                }
            ],
            "clientInfo": {
                "name": "codx",
                "version": env!("CARGO_PKG_VERSION")
            }
        });
        let result = self.request("initialize", init_value).await?;
        let init: InitializeResult = serde_json::from_value(result)
            .map_err(|error| format!("invalid initialize response: {error}"))?;
        self.capabilities = negotiate(&init);
        let initialized_value = serde_json::to_value(InitializedParams {})
            .map_err(|error| format!("failed to serialize initialized params: {error}"))?;
        self.notify("initialized", initialized_value).await?;
        self.initialized = true;
        Ok(())
    }

    pub async fn request(&mut self, method: &str, params: Value) -> Result<Value, String> {
        let id = self.next_request_id;
        self.next_request_id += 1;
        let payload = super::requests::build_request(id, method, params);
        self.write_jsonrpc(&payload.to_string())
            .map_err(|error| format!("write request failed: {error}"))?;

        if let Some(response) = self.queued_responses.remove(&id) {
            return Self::response_to_result(method, response);
        }

        loop {
            let incoming = self.receiver.recv_timeout(Duration::from_millis(600));
            let incoming = match incoming {
                Ok(message) => message,
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    return Err(format!(
                        "lsp request {method} timed out waiting for server response"
                    ));
                }
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    return Err(String::from("language server closed the connection"));
                }
            };
            match incoming {
                IncomingMessage::Notification(notification) => {
                    self.queued_notifications.push(notification);
                }
                IncomingMessage::Response(response) => {
                    if response.id == id {
                        return Self::response_to_result(method, response);
                    }
                    self.queued_responses.insert(response.id, response);
                }
            }
        }
    }

    pub async fn notify(&mut self, method: &str, params: Value) -> Result<(), String> {
        let payload = super::notifications::build_notification(method, params);
        self.write_jsonrpc(&payload.to_string())
            .map_err(|error| format!("write notification failed: {error}"))
    }

    pub fn drain_notifications(&mut self) -> Vec<Value> {
        while let Ok(incoming) = self.receiver.try_recv() {
            match incoming {
                IncomingMessage::Notification(notification) => {
                    self.queued_notifications.push(notification);
                }
                IncomingMessage::Response(response) => {
                    self.queued_responses.insert(response.id, response);
                }
            }
        }

        std::mem::take(&mut self.queued_notifications)
    }

    pub async fn try_read_notification(&mut self) -> Option<Value> {
        let mut drained = self.drain_notifications();
        if drained.is_empty() {
            None
        } else {
            Some(drained.remove(0))
        }
    }

    fn response_to_result(method: &str, response: RpcResponse) -> Result<Value, String> {
        if let Some(result) = response.result {
            return Ok(result);
        }
        if let Some(error) = response.error {
            return Err(format!("lsp request {method} failed: {error}"));
        }
        Err(format!("lsp request {method} returned no result"))
    }

    fn write_jsonrpc(&mut self, payload: &str) -> io::Result<()> {
        let header = format!("Content-Length: {}\r\n\r\n", payload.len());
        self.stdin.write_all(header.as_bytes())?;
        self.stdin.write_all(payload.as_bytes())?;
        self.stdin.flush()
    }
}

impl Drop for LspClient {
    fn drop(&mut self) {
        let _ = self.child.kill();
    }
}
