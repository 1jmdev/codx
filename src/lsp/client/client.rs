use std::path::{Path, PathBuf};

use lsp_types::InitializeResult;
use serde_json::Value;

use crate::lsp::capabilities::{NegotiatedCapabilities, default_client_capabilities, negotiate};
use crate::lsp::client::{RpcResponse, ServerConfig};
use crate::lsp::transport::LspTransport;

#[derive(Debug)]
pub struct LspClient {
    pub language_id: String,
    pub root_path: PathBuf,
    pub capabilities: NegotiatedCapabilities,
    pub initialized: bool,
    transport: LspTransport,
    next_request_id: u64,
}

impl LspClient {
    pub async fn launch(config: &ServerConfig, root_path: &Path) -> Result<Self, String> {
        let transport = LspTransport::spawn(&config.command, &config.args)
            .await
            .map_err(|error| format!("failed to spawn {}: {error}", config.command))?;
        let mut client = Self {
            language_id: config.language_id.clone(),
            root_path: root_path.to_path_buf(),
            capabilities: NegotiatedCapabilities {
                completion: false,
                hover: false,
                signature_help: false,
                goto_definition: false,
                goto_declaration: false,
                goto_type_definition: false,
                goto_implementation: false,
                references: false,
                rename: false,
                code_action: false,
                formatting: false,
                range_formatting: false,
                workspace_symbols: false,
                diagnostics_push: false,
            },
            initialized: false,
            transport,
            next_request_id: 1,
        };
        client.initialize().await?;
        Ok(client)
    }

    async fn initialize(&mut self) -> Result<(), String> {
        let root_uri_text = format!("file://{}", self.root_path.to_string_lossy());
        let init_value = serde_json::json!({
            "processId": std::process::id(),
            "rootUri": root_uri_text,
            "capabilities": default_client_capabilities(),
            "clientInfo": {
                "name": "codx",
                "version": env!("CARGO_PKG_VERSION")
            },
            "workDoneProgressParams": {
                "workDoneToken": null
            }
        });
        let result = self.request("initialize", init_value).await?;
        let init: InitializeResult = serde_json::from_value(result)
            .map_err(|error| format!("invalid initialize response: {error}"))?;
        self.capabilities = negotiate(&init);
        self.notify("initialized", Value::Object(serde_json::Map::new()))
            .await?;
        self.initialized = true;
        Ok(())
    }

    pub async fn request(&mut self, method: &str, params: Value) -> Result<Value, String> {
        let id = self.next_request_id;
        self.next_request_id += 1;
        let payload = super::requests::build_request(id, method, params);
        self.transport
            .write_jsonrpc(&payload.to_string())
            .await
            .map_err(|error| format!("write request failed: {error}"))?;
        loop {
            let Some(message) = self
                .transport
                .read_jsonrpc()
                .await
                .map_err(|error| format!("read response failed: {error}"))?
            else {
                return Err(String::from("language server closed the connection"));
            };
            let parsed: RpcResponse = parse_response(&message)?;
            if parsed.id != id {
                continue;
            }
            if let Some(result) = parsed.result {
                return Ok(result);
            }
            if let Some(error) = parsed.error {
                return Err(format!("lsp request {method} failed: {error}"));
            }
        }
    }

    pub async fn notify(&mut self, method: &str, params: Value) -> Result<(), String> {
        let payload = super::notifications::build_notification(method, params);
        self.transport
            .write_jsonrpc(&payload.to_string())
            .await
            .map_err(|error| format!("write notification failed: {error}"))
    }

    pub async fn try_read_notification(&mut self) -> Option<Value> {
        let message = self.transport.read_jsonrpc().await.ok().flatten()?;
        let parsed = serde_json::from_str::<Value>(&message).ok()?;
        if parsed.get("method").is_some() {
            Some(parsed)
        } else {
            None
        }
    }
}

fn parse_response(message: &str) -> Result<RpcResponse, String> {
    let json: Value =
        serde_json::from_str(message).map_err(|error| format!("invalid json-rpc payload: {error}"))?;
    let id = json
        .get("id")
        .and_then(Value::as_u64)
        .ok_or_else(|| String::from("missing json-rpc response id"))?;
    Ok(RpcResponse {
        id,
        result: json.get("result").cloned(),
        error: json.get("error").cloned(),
    })
}
