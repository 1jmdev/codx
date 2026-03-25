use std::collections::HashMap;
use std::path::Path;
use std::process::{Command, Stdio};

use crate::lsp::capabilities::NegotiatedCapabilities;
use crate::lsp::client::response_loop::spawn_response_loop;
use crate::lsp::client::session::LspClient;
use crate::lsp::client::ServerConfig;

pub(super) fn launch_client(config: &ServerConfig, root_path: &Path) -> Result<LspClient, String> {
    let mut child = Command::new(&config.command)
        .args(&config.args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| format!("failed to spawn {}: {error}", config.command))?;

    let stdin = child
        .stdin
        .take()
        .ok_or_else(|| String::from("missing language server stdin"))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| String::from("missing language server stdout"))?;

    let receiver = spawn_response_loop(stdout);

    Ok(LspClient {
        language_id: config.language_id.clone(),
        root_path: root_path.to_path_buf(),
        initialization_options: config.initialization_options.clone(),
        capabilities: default_capabilities(),
        initialized: false,
        child,
        stdin,
        receiver,
        queued_notifications: Vec::new(),
        queued_responses: HashMap::new(),
        next_request_id: 1,
    })
}

fn default_capabilities() -> NegotiatedCapabilities {
    NegotiatedCapabilities {
        completion: false,
        hover: false,
        signature_help: false,
        goto_definition: false,
        references: false,
        rename: false,
        code_action: false,
        formatting: false,
        workspace_symbols: false,
    }
}
