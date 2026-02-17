mod client;
mod protocol;
mod server_spec;
mod uri;

use std::{collections::HashMap, path::{Path, PathBuf}};

use lsp_types::Diagnostic;

use self::client::{LspClient, ServerEvent};
use self::server_spec::ServerSpec;

pub(crate) struct LspManager {
    root: PathBuf,
    client: Option<LspClient>,
    key: Option<&'static str>,
    diagnostics: HashMap<PathBuf, Vec<Diagnostic>>,
}

impl LspManager {
    pub(crate) fn new(root: PathBuf) -> Self {
        Self {
            root,
            client: None,
            key: None,
            diagnostics: HashMap::new(),
        }
    }

    pub(crate) fn open_file(&mut self, path: &Path, text: String, status: &mut String) {
        let Some(spec) = ServerSpec::from_path(path) else {
            self.client = None;
            self.key = None;
            return;
        };

        if self.key != Some(spec.key) {
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
            client.open_document(path, text);
        }
    }

    pub(crate) fn did_change(&mut self, path: &Path, text: String, _status: &mut String) {
        if let Some(client) = self.client.as_mut() {
            client.did_change(path, text);
        }
    }

    pub(crate) fn poll(&mut self, _status: &mut String) {
        let mut incoming = Vec::new();
        if let Some(client) = self.client.as_mut() {
            incoming = client.poll();
        }

        for event in incoming {
            let ServerEvent::Diagnostics { path, diagnostics } = event;
            self.diagnostics.insert(path, diagnostics);
        }
    }

    pub(crate) fn first_diagnostic_for_line(
        &self,
        path: Option<&PathBuf>,
        line_idx: usize,
    ) -> Option<String> {
        let path = path?;
        let diagnostics = self.diagnostics.get(path)?;
        diagnostics
            .iter()
            .find(|item| item.range.start.line as usize == line_idx)
            .map(|item| item.message.clone())
    }

    pub(crate) fn line_has_diagnostic(&self, path: Option<&PathBuf>, line_idx: usize) -> bool {
        self.first_diagnostic_for_line(path, line_idx).is_some()
    }
}
