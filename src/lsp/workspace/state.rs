use std::collections::HashMap;
use std::path::Path;

use lsp_types::Position;
use tokio::runtime::{Builder, Runtime};

use crate::lsp::client::LspClient;
use crate::lsp::completion::{CompletionContext, CompletionItemView};
use crate::lsp::diagnostics::{DiagnosticItem, DiagnosticStore};
use crate::lsp::hover::HoverView;
use crate::lsp::progress::ProgressState;
use crate::lsp::signature::SignatureHelpView;
use crate::lsp::workspace::config::load_server_config;
use crate::syntax::{language_for_path, LanguageId};
use crate::ui::{PickerItem, PickerState};

#[derive(Debug, Default)]
pub struct LspWorkspace {
    runtime: Option<Runtime>,
    clients: HashMap<LanguageId, LspClient>,
    servers: HashMap<LanguageId, crate::lsp::client::ServerConfig>,
    diagnostics: DiagnosticStore,
    pub completion: CompletionContext,
    pub hover: HoverView,
    pub signature: SignatureHelpView,
    pub progress: ProgressState,
    pub diagnostics_panel_open: bool,
}

impl LspWorkspace {
    pub fn new(workspace_root: &Path) -> Self {
        let runtime = Builder::new_current_thread().enable_all().build().ok();
        let servers = load_server_config(workspace_root);
        Self {
            runtime,
            clients: HashMap::new(),
            servers,
            diagnostics: DiagnosticStore::default(),
            completion: CompletionContext::default(),
            hover: HoverView::default(),
            signature: SignatureHelpView::default(),
            progress: ProgressState::default(),
            diagnostics_panel_open: false,
        }
    }

    pub fn diagnostics_for_path(&self, path: &Path) -> &[DiagnosticItem] {
        self.diagnostics.for_path(path)
    }

    pub fn diagnostics_count(&self, path: Option<&Path>) -> usize {
        path.map(|p| self.diagnostics.for_path(p).len())
            .unwrap_or(0)
    }

    pub fn toggle_diagnostics_panel(&mut self) {
        self.diagnostics_panel_open = !self.diagnostics_panel_open;
    }

    pub fn ensure_client_for_path(&mut self, path: &Path, workspace_root: &Path) {
        let Some(language) = language_for_path(path) else {
            return;
        };
        if self.clients.contains_key(&language) {
            return;
        }
        let Some(server) = self.servers.get(&language).cloned() else {
            return;
        };
        let Some(runtime) = self.runtime.as_mut() else {
            return;
        };
        if let Ok(client) = runtime.block_on(LspClient::launch(&server, workspace_root)) {
            self.clients.insert(language, client);
        }
    }

    pub fn request_completion(
        &mut self,
        path: &Path,
        workspace_root: &Path,
        line: usize,
        character: usize,
    ) -> Vec<CompletionItemView> {
        self.ensure_client_for_path(path, workspace_root);
        let Some(language) = language_for_path(path) else {
            return Vec::new();
        };
        let Some(client) = self.clients.get_mut(&language) else {
            return Vec::new();
        };
        if !client.capabilities.completion {
            return Vec::new();
        }
        let Some(runtime) = self.runtime.as_mut() else {
            return Vec::new();
        };

        let params = serde_json::json!({
            "textDocument": { "uri": file_uri(path) },
            "position": { "line": line, "character": character },
            "context": { "triggerKind": 1 }
        });

        let value = match runtime.block_on(client.request("textDocument/completion", params)) {
            Ok(value) => value,
            Err(_) => return Vec::new(),
        };
        parse_completion_items(value)
    }
}

fn file_uri(path: &Path) -> String {
    format!("file://{}", path.to_string_lossy())
}

fn parse_file_uri(uri: &str) -> Option<std::path::PathBuf> {
    uri.strip_prefix("file://").map(std::path::PathBuf::from)
}

fn parse_position(value: &serde_json::Value) -> (usize, usize) {
    let line = value
        .get("line")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0) as usize;
    let character = value
        .get("character")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0) as usize;
    (line, character)
}

impl LspWorkspace {
    pub fn did_open(&mut self, path: &Path, text: &str, workspace_root: &Path) {
        self.ensure_client_for_path(path, workspace_root);
        let Some(language) = language_for_path(path) else {
            return;
        };
        let Some(client) = self.clients.get_mut(&language) else {
            return;
        };
        let Some(runtime) = self.runtime.as_mut() else {
            return;
        };
        let params = serde_json::json!({
            "textDocument": {
                "uri": file_uri(path),
                "languageId": client.language_id,
                "version": 1,
                "text": text
            }
        });
        let _ = runtime.block_on(client.notify("textDocument/didOpen", params));
    }

    pub fn did_change(&mut self, path: &Path, text: &str, version: i32, workspace_root: &Path) {
        self.ensure_client_for_path(path, workspace_root);
        let Some(language) = language_for_path(path) else {
            return;
        };
        let Some(client) = self.clients.get_mut(&language) else {
            return;
        };
        let Some(runtime) = self.runtime.as_mut() else {
            return;
        };
        let params = serde_json::json!({
            "textDocument": { "uri": file_uri(path), "version": version },
            "contentChanges": [ { "text": text } ]
        });
        let _ = runtime.block_on(client.notify("textDocument/didChange", params));
    }

    pub fn did_save(&mut self, path: &Path, text: &str, workspace_root: &Path) {
        self.ensure_client_for_path(path, workspace_root);
        let Some(language) = language_for_path(path) else {
            return;
        };
        let Some(client) = self.clients.get_mut(&language) else {
            return;
        };
        let Some(runtime) = self.runtime.as_mut() else {
            return;
        };
        let params = serde_json::json!({
            "textDocument": { "uri": file_uri(path) },
            "text": text
        });
        let _ = runtime.block_on(client.notify("textDocument/didSave", params));
    }

    pub fn request_hover(
        &mut self,
        path: &Path,
        workspace_root: &Path,
        line: usize,
        character: usize,
    ) -> Option<String> {
        self.ensure_client_for_path(path, workspace_root);
        let language = language_for_path(path)?;
        let client = self.clients.get_mut(&language)?;
        if !client.capabilities.hover {
            return None;
        }
        let runtime = self.runtime.as_mut()?;
        let params = serde_json::json!({
            "textDocument": { "uri": file_uri(path) },
            "position": { "line": line, "character": character }
        });
        let value = runtime
            .block_on(client.request("textDocument/hover", params))
            .ok()?;
        if let Some(text) = value
            .get("contents")
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned)
        {
            return Some(text);
        }
        value
            .get("contents")
            .and_then(|v| v.get("value"))
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned)
    }

    pub fn request_signature(
        &mut self,
        path: &Path,
        workspace_root: &Path,
        line: usize,
        character: usize,
    ) -> Option<(String, Option<u32>)> {
        self.ensure_client_for_path(path, workspace_root);
        let language = language_for_path(path)?;
        let client = self.clients.get_mut(&language)?;
        if !client.capabilities.signature_help {
            return None;
        }
        let runtime = self.runtime.as_mut()?;
        let params = serde_json::json!({
            "textDocument": { "uri": file_uri(path) },
            "position": { "line": line, "character": character }
        });
        let value = runtime
            .block_on(client.request("textDocument/signatureHelp", params))
            .ok()?;
        let sigs = value.get("signatures")?.as_array()?;
        let first = sigs.first()?;
        let label = first
            .get("label")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_owned();
        let active = value
            .get("activeParameter")
            .and_then(serde_json::Value::as_u64)
            .map(|v| v as u32);
        Some((label, active))
    }

    pub fn goto_definition(
        &mut self,
        path: &Path,
        workspace_root: &Path,
        line: usize,
        character: usize,
    ) -> Option<(std::path::PathBuf, usize, usize)> {
        self.goto_request(
            "textDocument/definition",
            path,
            workspace_root,
            line,
            character,
        )
    }

    pub fn goto_references(
        &mut self,
        path: &Path,
        workspace_root: &Path,
        line: usize,
        character: usize,
    ) -> Vec<(std::path::PathBuf, usize, usize)> {
        self.ensure_client_for_path(path, workspace_root);
        let Some(language) = language_for_path(path) else {
            return Vec::new();
        };
        let Some(client) = self.clients.get_mut(&language) else {
            return Vec::new();
        };
        if !client.capabilities.references {
            return Vec::new();
        }
        let Some(runtime) = self.runtime.as_mut() else {
            return Vec::new();
        };
        let params = serde_json::json!({
            "textDocument": { "uri": file_uri(path) },
            "position": { "line": line, "character": character },
            "context": { "includeDeclaration": true }
        });
        let value = match runtime.block_on(client.request("textDocument/references", params)) {
            Ok(value) => value,
            Err(_) => return Vec::new(),
        };
        value
            .as_array()
            .map(|list| {
                list.iter()
                    .filter_map(|item| {
                        let uri = item.get("uri")?.as_str()?;
                        let path = parse_file_uri(uri)?;
                        let start = item.get("range")?.get("start")?;
                        let (line, column) = parse_position(start);
                        Some((path, line, column))
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    }

    pub fn workspace_symbols(&mut self, query: &str, workspace_root: &Path) -> Vec<PickerItem> {
        let Some(language) = self
            .clients
            .keys()
            .next()
            .copied()
            .or(Some(LanguageId::Rust))
        else {
            return Vec::new();
        };
        if !self.clients.contains_key(&language) {
            let fake = workspace_root.join("src").join("main.rs");
            self.ensure_client_for_path(&fake, workspace_root);
        }
        let Some(client) = self.clients.get_mut(&language) else {
            return Vec::new();
        };
        if !client.capabilities.workspace_symbols {
            return Vec::new();
        }
        let Some(runtime) = self.runtime.as_mut() else {
            return Vec::new();
        };
        let value = match runtime
            .block_on(client.request("workspace/symbol", serde_json::json!({ "query": query })))
        {
            Ok(value) => value,
            Err(_) => return Vec::new(),
        };
        value
            .as_array()
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| {
                        let name = item.get("name")?.as_str()?.to_owned();
                        let loc = item.get("location")?;
                        let uri = loc.get("uri")?.as_str()?;
                        let path = parse_file_uri(uri)?;
                        let start = loc.get("range")?.get("start")?;
                        let (line, column) = parse_position(start);
                        Some(PickerItem {
                            title: name,
                            subtitle: path.display().to_string(),
                            path: Some(path),
                            buffer_id: None,
                            line: Some(line),
                            column: Some(column),
                        })
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    }

    pub fn open_workspace_symbols(
        &mut self,
        query: &str,
        workspace_root: &Path,
        picker: &mut Option<PickerState>,
    ) {
        let mut state = PickerState::new(crate::ui::PickerKind::Files);
        state.set_buffer_items(self.workspace_symbols(query, workspace_root));
        if state.items().is_empty() {
            *picker = None;
        } else {
            *picker = Some(state);
        }
    }

    pub fn format_document(
        &mut self,
        path: &Path,
        workspace_root: &Path,
        text: &str,
    ) -> Option<String> {
        self.ensure_client_for_path(path, workspace_root);
        let language = language_for_path(path)?;
        let client = self.clients.get_mut(&language)?;
        if !client.capabilities.formatting {
            return None;
        }
        let runtime = self.runtime.as_mut()?;
        let params = serde_json::json!({
            "textDocument": { "uri": file_uri(path) },
            "options": { "tabSize": 4, "insertSpaces": true }
        });
        let edits = runtime
            .block_on(client.request("textDocument/formatting", params))
            .ok()?;
        apply_text_edits(text, &edits)
    }

    pub fn request_code_actions(
        &mut self,
        path: &Path,
        workspace_root: &Path,
        line: usize,
        character: usize,
    ) -> Vec<String> {
        self.ensure_client_for_path(path, workspace_root);
        let Some(language) = language_for_path(path) else {
            return Vec::new();
        };
        let Some(client) = self.clients.get_mut(&language) else {
            return Vec::new();
        };
        if !client.capabilities.code_action {
            return Vec::new();
        }
        let Some(runtime) = self.runtime.as_mut() else {
            return Vec::new();
        };
        let range = serde_json::json!({
            "start": { "line": line, "character": character },
            "end": { "line": line, "character": character }
        });
        let params = serde_json::json!({
            "textDocument": { "uri": file_uri(path) },
            "range": range,
            "context": { "diagnostics": [] }
        });
        let value = match runtime.block_on(client.request("textDocument/codeAction", params)) {
            Ok(value) => value,
            Err(_) => return Vec::new(),
        };
        value
            .as_array()
            .map(|items| {
                items
                    .iter()
                    .filter_map(|i| i.get("title").and_then(serde_json::Value::as_str))
                    .map(ToOwned::to_owned)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    }

    pub fn rename(
        &mut self,
        path: &Path,
        workspace_root: &Path,
        line: usize,
        character: usize,
        new_name: &str,
    ) -> Vec<(std::path::PathBuf, String)> {
        self.ensure_client_for_path(path, workspace_root);
        let Some(language) = language_for_path(path) else {
            return Vec::new();
        };
        let Some(client) = self.clients.get_mut(&language) else {
            return Vec::new();
        };
        if !client.capabilities.rename {
            return Vec::new();
        }
        let Some(runtime) = self.runtime.as_mut() else {
            return Vec::new();
        };
        let params = serde_json::json!({
            "textDocument": { "uri": file_uri(path) },
            "position": { "line": line, "character": character },
            "newName": new_name
        });
        let value = match runtime.block_on(client.request("textDocument/rename", params)) {
            Ok(value) => value,
            Err(_) => return Vec::new(),
        };
        let changes = value.get("changes").and_then(serde_json::Value::as_object);
        let Some(changes) = changes else {
            return Vec::new();
        };
        changes
            .iter()
            .filter_map(|(uri, edits)| {
                let path = parse_file_uri(uri)?;
                let original = std::fs::read_to_string(&path).ok()?;
                let updated = apply_text_edits(&original, edits)?;
                Some((path, updated))
            })
            .collect()
    }

    pub fn poll_server_messages(&mut self) {
        let Some(runtime) = self.runtime.as_mut() else {
            return;
        };
        let mut updates = Vec::new();
        for client in self.clients.values_mut() {
            let message = runtime.block_on(client.try_read_notification());
            if let Some(message) = message {
                updates.push(message);
            }
        }
        for update in updates {
            if let Some(method) = update.get("method").and_then(serde_json::Value::as_str) {
                match method {
                    "textDocument/publishDiagnostics" => {
                        let Some(params) = update.get("params") else {
                            continue;
                        };
                        let Some(uri) = params.get("uri").and_then(serde_json::Value::as_str)
                        else {
                            continue;
                        };
                        let Some(path) = parse_file_uri(uri) else {
                            continue;
                        };
                        let Some(diags) = params
                            .get("diagnostics")
                            .and_then(serde_json::Value::as_array)
                            .cloned()
                        else {
                            continue;
                        };
                        let parsed = diags
                            .into_iter()
                            .filter_map(|d| serde_json::from_value::<lsp_types::Diagnostic>(d).ok())
                            .collect::<Vec<_>>();
                        self.diagnostics.apply_publish(path, parsed);
                    }
                    "$/progress" => {
                        let Some(params) = update.get("params") else {
                            continue;
                        };
                        if let Some(value) = params.get("value") {
                            self.progress.title = value
                                .get("title")
                                .and_then(serde_json::Value::as_str)
                                .unwrap_or_default()
                                .to_owned();
                            self.progress.message = value
                                .get("message")
                                .and_then(serde_json::Value::as_str)
                                .unwrap_or_default()
                                .to_owned();
                            self.progress.percentage = value
                                .get("percentage")
                                .and_then(serde_json::Value::as_u64)
                                .map(|v| v as u32);
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    fn goto_request(
        &mut self,
        method: &str,
        path: &Path,
        workspace_root: &Path,
        line: usize,
        character: usize,
    ) -> Option<(std::path::PathBuf, usize, usize)> {
        self.ensure_client_for_path(path, workspace_root);
        let language = language_for_path(path)?;
        let client = self.clients.get_mut(&language)?;
        let runtime = self.runtime.as_mut()?;
        let params = serde_json::json!({
            "textDocument": { "uri": file_uri(path) },
            "position": { "line": line, "character": character }
        });
        let value = runtime.block_on(client.request(method, params)).ok()?;
        let first = value.as_array()?.first()?.clone();
        let uri = first.get("uri")?.as_str()?;
        let p = parse_file_uri(uri)?;
        let start = first.get("range")?.get("start")?;
        let (line, column) = parse_position(start);
        Some((p, line, column))
    }
}

fn apply_text_edits(text: &str, edits_value: &serde_json::Value) -> Option<String> {
    let edits = edits_value.as_array()?;
    let mut parsed = edits
        .iter()
        .filter_map(|v| serde_json::from_value::<lsp_types::TextEdit>(v.clone()).ok())
        .collect::<Vec<_>>();
    parsed.sort_by(|a, b| {
        let al = (a.range.start.line, a.range.start.character);
        let bl = (b.range.start.line, b.range.start.character);
        bl.cmp(&al)
    });
    let mut output = text.to_owned();
    for edit in parsed {
        let start = position_to_byte(&output, edit.range.start)?;
        let end = position_to_byte(&output, edit.range.end)?;
        if start <= end && end <= output.len() {
            output.replace_range(start..end, &edit.new_text);
        }
    }
    Some(output)
}

fn position_to_byte(text: &str, pos: Position) -> Option<usize> {
    let mut byte = 0usize;
    let mut line = 0u32;
    for segment in text.split_inclusive('\n') {
        if line == pos.line {
            let mut col = 0u32;
            for (offset, ch) in segment.char_indices() {
                if col == pos.character {
                    return Some(byte + offset);
                }
                if ch == '\n' {
                    break;
                }
                col += 1;
            }
            return Some(byte + segment.len());
        }
        byte += segment.len();
        line += 1;
    }
    if line == pos.line {
        Some(text.len())
    } else {
        None
    }
}

fn parse_completion_items(value: serde_json::Value) -> Vec<CompletionItemView> {
    let items_value = if value.get("items").is_some() {
        value
            .get("items")
            .cloned()
            .unwrap_or(serde_json::Value::Null)
    } else {
        value
    };
    let Some(list) = items_value.as_array() else {
        return Vec::new();
    };

    list.iter()
        .filter_map(|item| {
            let label = item.get("label").and_then(serde_json::Value::as_str)?;
            let detail = item
                .get("detail")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default();
            let insert_text = item
                .get("insertText")
                .and_then(serde_json::Value::as_str)
                .unwrap_or(label);
            let documentation = match item.get("documentation") {
                Some(serde_json::Value::String(text)) => text.to_owned(),
                Some(serde_json::Value::Object(object)) => object
                    .get("value")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_default()
                    .to_owned(),
                _ => String::new(),
            };
            let is_snippet = item
                .get("insertTextFormat")
                .and_then(serde_json::Value::as_u64)
                == Some(2);

            Some(CompletionItemView {
                label: label.to_owned(),
                detail: detail.to_owned(),
                documentation,
                insert_text: insert_text.to_owned(),
                is_snippet,
            })
        })
        .collect()
}
