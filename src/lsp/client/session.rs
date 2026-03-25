use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Child;
use std::sync::mpsc::{Receiver, Sender};
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
    pub initialization_options: Option<Value>,
    pub capabilities: NegotiatedCapabilities,
    pub initialized: bool,
    pub(super) child: Child,
    pub(super) sender: Sender<String>,
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
            "initializationOptions": self.initialization_options.clone().unwrap_or(Value::Null),
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
        if let Some(options) = self.initialization_options.clone() {
            self.notify(
                "workspace/didChangeConfiguration",
                serde_json::json!({
                    "settings": options
                }),
            )
            .await?;
        }
        self.initialized = true;
        Ok(())
    }

    pub async fn request(&mut self, method: &str, params: Value) -> Result<Value, String> {
        let id = self.send_request(method, params)?;

        if let Some(response) = self.queued_responses.remove(&id) {
            return response.into_result(method);
        }

        loop {
            let incoming = self.receiver.recv_timeout(Duration::from_secs(5));
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
                        return response.into_result(method);
                    }
                    self.queued_responses.insert(response.id, response);
                }
            }
        }
    }

    pub fn send_request(&mut self, method: &str, params: Value) -> Result<u64, String> {
        let id = self.next_request_id;
        self.next_request_id += 1;
        let payload = super::requests::build_request(id, method, params);
        self.write_jsonrpc(&payload.to_string())
            .map_err(|error| format!("write request failed: {error}"))?;
        Ok(id)
    }

    pub async fn notify(&mut self, method: &str, params: Value) -> Result<(), String> {
        let payload = super::notifications::build_notification(method, params);
        self.write_jsonrpc(&payload.to_string())
            .map_err(|error| format!("write notification failed: {error}"))
    }

    pub fn drain_notifications(&mut self) -> Vec<Value> {
        self.pump_incoming();

        std::mem::take(&mut self.queued_notifications)
    }

    pub fn drain_responses(&mut self) -> Vec<RpcResponse> {
        self.pump_incoming();
        self.queued_responses.drain().map(|(_, response)| response).collect()
    }

    fn pump_incoming(&mut self) {
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
    }

    fn write_jsonrpc(&mut self, payload: &str) -> io::Result<()> {
        let mut message = String::with_capacity(payload.len() + 32);
        message.push_str("Content-Length: ");
        message.push_str(&payload.len().to_string());
        message.push_str("\r\n\r\n");
        message.push_str(payload);
        self.sender.send(message).map_err(|error| {
            io::Error::new(
                io::ErrorKind::BrokenPipe,
                format!("language server writer closed: {error}"),
            )
        })
    }
}

impl Drop for LspClient {
    fn drop(&mut self) {
        let _ = self.child.kill();
    }
}
