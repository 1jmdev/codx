mod client;
mod protocol;
mod server_spec;
mod uri;

use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use lsp_types::{Diagnostic, DiagnosticSeverity};

use self::client::{LspClient, ServerEvent};
use self::server_spec::ServerSpec;

#[derive(Clone, Debug)]
pub(crate) struct CompletionUpdate {
    pub(crate) path: PathBuf,
    pub(crate) line: usize,
    pub(crate) col: usize,
    pub(crate) items: Vec<lsp_types::CompletionItem>,
}

pub(crate) struct LspManager {
    root: PathBuf,
    client: Option<LspClient>,
    key: Option<&'static str>,
    diagnostics: HashMap<PathBuf, Vec<Diagnostic>>,
    document_versions: HashMap<PathBuf, i32>,
    completion: Option<CompletionUpdate>,
}

impl LspManager {
    pub(crate) fn new(root: PathBuf) -> Self {
        Self {
            root,
            client: None,
            key: None,
            diagnostics: HashMap::new(),
            document_versions: HashMap::new(),
            completion: None,
        }
    }

    pub(crate) fn open_file(&mut self, path: &Path, text: String, status: &mut String) {
        let normalized = normalize_path(path);
        let Some(spec) = ServerSpec::from_path(path) else {
            self.client = None;
            self.key = None;
            self.diagnostics.clear();
            self.document_versions.clear();
            self.completion = None;
            return;
        };

        if self.key != Some(spec.key) {
            self.diagnostics.clear();
            self.document_versions.clear();
            self.completion = None;
            self.client = match LspClient::start(&self.root, &spec) {
                Ok(client) => {
                    *status = format!("LSP connected: {}", spec.name);
                    self.key = Some(spec.key);
                    Some(client)
                }
                Err(error) => {
                    *status = format!("LSP unavailable ({}): {error}", spec.command);
                    self.key = None;
                    None
                }
            };
        }

        if let Some(client) = self.client.as_mut() {
            if let Some(version) = client.open_document(path, text) {
                self.document_versions.insert(normalized, version);
            }
        }
    }

    pub(crate) fn did_change(&mut self, path: &Path, text: String, _status: &mut String) {
        let normalized = normalize_path(path);
        if let Some(client) = self.client.as_mut() {
            let version = client.did_change(path, text);
            self.document_versions.insert(normalized, version);
        }
    }

    pub(crate) fn did_save(&mut self, path: &Path, _status: &mut String) {
        if let Some(client) = self.client.as_mut() {
            client.did_save(path);
        }
    }

    pub(crate) fn reload_for_file(&mut self, path: &Path, text: String, status: &mut String) {
        self.client = None;
        self.key = None;
        self.diagnostics.clear();
        self.document_versions.clear();
        self.completion = None;
        self.open_file(path, text, status);
    }

    pub(crate) fn request_completion(
        &mut self,
        path: &Path,
        line: usize,
        col: usize,
        status: &mut String,
    ) -> bool {
        let normalized = normalize_path(path);
        let version = self
            .document_versions
            .get(&normalized)
            .copied()
            .unwrap_or(1);
        let Some(client) = self.client.as_mut() else {
            *status = String::from("LSP unavailable for this file.");
            return false;
        };

        match client.request_completion(path, line, col, version) {
            Ok(()) => true,
            Err(error) => {
                *status = format!("Completion request failed: {error}");
                false
            }
        }
    }

    pub(crate) fn poll(&mut self, _status: &mut String) {
        let mut incoming = Vec::new();
        if let Some(client) = self.client.as_mut() {
            incoming = client.poll();
        }

        for event in incoming {
            match event {
                ServerEvent::Diagnostics {
                    path,
                    version,
                    diagnostics,
                } => {
                    let normalized = normalize_path(&path);
                    if let (Some(incoming_version), Some(current_version)) =
                        (version, self.document_versions.get(&normalized).copied())
                        && incoming_version < current_version
                    {
                        continue;
                    }
                    self.diagnostics.insert(normalized, diagnostics);
                }
                ServerEvent::Completion {
                    path,
                    version,
                    line,
                    col,
                    items,
                } => {
                    let normalized = normalize_path(&path);
                    if let Some(current_version) = self.document_versions.get(&normalized).copied()
                        && version < current_version
                    {
                        continue;
                    }

                    self.completion = Some(CompletionUpdate {
                        path: normalized,
                        line,
                        col,
                        items,
                    });
                }
            }
        }
    }

    pub(crate) fn take_completion(&mut self) -> Option<CompletionUpdate> {
        self.completion.take()
    }

    fn first_diagnostic_for_line(
        &self,
        path: Option<&PathBuf>,
        line_idx: usize,
    ) -> Option<&Diagnostic> {
        let path = path?;
        let diagnostics = self.diagnostics.get(path).or_else(|| {
            fs::canonicalize(path)
                .ok()
                .and_then(|real| self.diagnostics.get(&real))
        })?;
        diagnostics
            .iter()
            .find(|item| item.range.start.line as usize == line_idx)
    }

    pub(crate) fn diagnostic_hint_for_line(
        &self,
        path: Option<&PathBuf>,
        line_idx: usize,
    ) -> Option<(&str, bool)> {
        let diagnostic = self.first_diagnostic_for_line(path, line_idx)?;
        let is_warning = matches!(diagnostic.severity, Some(DiagnosticSeverity::WARNING));
        Some((diagnostic.message.as_str(), is_warning))
    }
}

fn normalize_path(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}
