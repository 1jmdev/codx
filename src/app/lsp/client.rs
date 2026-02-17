use std::{
    io::{self, BufReader, Write},
    path::{Path, PathBuf},
    process::{Child, ChildStdin, Command, Stdio},
    sync::mpsc::{self, Receiver},
    thread,
};

use lsp_types::{
    DidChangeTextDocumentParams, DidOpenTextDocumentParams, DidSaveTextDocumentParams,
    InitializeParams, InitializedParams, PublishDiagnosticsParams, TextDocumentContentChangeEvent,
    TextDocumentIdentifier, TextDocumentItem, Uri, VersionedTextDocumentIdentifier,
    WorkspaceFolder,
    notification::Notification,
    notification::{
        DidChangeTextDocument, DidOpenTextDocument, DidSaveTextDocument, Initialized,
        PublishDiagnostics,
    },
    request::{Initialize, Request},
};
use serde_json::{Value, json};

use super::{
    protocol::read_message,
    server_spec::ServerSpec,
    uri::{path_to_uri, uri_to_path},
};

pub(crate) enum ServerEvent {
    Diagnostics {
        path: PathBuf,
        version: Option<i32>,
        diagnostics: Vec<lsp_types::Diagnostic>,
    },
}

pub(crate) struct LspClient {
    _child: Child,
    stdin: ChildStdin,
    receiver: Receiver<ServerEvent>,
    next_id: i64,
    open_uri: Option<Uri>,
    version: i32,
    language_id: &'static str,
}

impl LspClient {
    pub(crate) fn start(root: &Path, spec: &ServerSpec) -> io::Result<Self> {
        let mut child = Command::new(spec.command)
            .args(spec.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| io::Error::other("Failed to capture LSP stdin"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| io::Error::other("Failed to capture LSP stdout"))?;

        let (tx, receiver) = mpsc::channel();
        thread::spawn(move || {
            let mut reader = BufReader::new(stdout);
            while let Ok(message) = read_message(&mut reader) {
                let Some(value) = message else {
                    break;
                };

                if value.get("method").and_then(Value::as_str) == Some(PublishDiagnostics::METHOD)
                    && let Some(params) = value.get("params")
                    && let Ok(payload) =
                        serde_json::from_value::<PublishDiagnosticsParams>(params.clone())
                    && let Some(path) = uri_to_path(&payload.uri)
                {
                    let _ = tx.send(ServerEvent::Diagnostics {
                        path,
                        version: payload.version,
                        diagnostics: payload.diagnostics,
                    });
                }
            }
        });

        let mut client = Self {
            _child: child,
            stdin,
            receiver,
            next_id: 1,
            open_uri: None,
            version: 1,
            language_id: spec.language_id,
        };

        client.initialize(root)?;
        Ok(client)
    }

    fn initialize(&mut self, root: &Path) -> io::Result<()> {
        let root_uri =
            path_to_uri(root).ok_or_else(|| io::Error::other("Invalid workspace path for LSP"))?;

        let params = InitializeParams {
            process_id: Some(std::process::id()),
            workspace_folders: Some(vec![WorkspaceFolder {
                uri: root_uri,
                name: String::from("workspace"),
            }]),
            ..Default::default()
        };

        let initialize = serde_json::to_value(params)
            .map_err(|error| io::Error::other(format!("LSP init serialize failed: {error}")))?;
        self.send_request(Initialize::METHOD, initialize)?;

        let initialized = serde_json::to_value(InitializedParams {})
            .map_err(|error| io::Error::other(format!("LSP init serialize failed: {error}")))?;
        self.send_notification(Initialized::METHOD, initialized)?;
        Ok(())
    }

    pub(crate) fn open_document(&mut self, path: &Path, text: String) -> Option<i32> {
        if let Some(uri) = path_to_uri(path) {
            self.open_uri = Some(uri.clone());
            self.version = 1;
            let params = DidOpenTextDocumentParams {
                text_document: TextDocumentItem {
                    uri,
                    language_id: self.language_id.to_string(),
                    version: self.version,
                    text,
                },
            };
            let _ = self.send_notification(
                DidOpenTextDocument::METHOD,
                serde_json::to_value(params).unwrap_or(Value::Null),
            );
            return Some(self.version);
        }

        None
    }

    pub(crate) fn did_change(&mut self, path: &Path, text: String) -> i32 {
        let uri = match &self.open_uri {
            Some(uri) => uri.clone(),
            None => match path_to_uri(path) {
                Some(uri) => uri,
                None => return self.version,
            },
        };

        self.version += 1;
        let params = DidChangeTextDocumentParams {
            text_document: VersionedTextDocumentIdentifier {
                uri,
                version: self.version,
            },
            content_changes: vec![TextDocumentContentChangeEvent {
                range: None,
                range_length: None,
                text,
            }],
        };

        let _ = self.send_notification(
            DidChangeTextDocument::METHOD,
            serde_json::to_value(params).unwrap_or(Value::Null),
        );

        self.version
    }

    pub(crate) fn did_save(&mut self, path: &Path) {
        let uri = match &self.open_uri {
            Some(uri) => uri.clone(),
            None => match path_to_uri(path) {
                Some(uri) => uri,
                None => return,
            },
        };

        let params = DidSaveTextDocumentParams {
            text_document: TextDocumentIdentifier { uri },
            text: None,
        };

        let _ = self.send_notification(
            DidSaveTextDocument::METHOD,
            serde_json::to_value(params).unwrap_or(Value::Null),
        );
    }

    pub(crate) fn poll(&mut self) -> Vec<ServerEvent> {
        let mut out = Vec::new();
        while let Ok(item) = self.receiver.try_recv() {
            out.push(item);
        }
        out
    }

    fn send_request(&mut self, method: &str, params: Value) -> io::Result<()> {
        let id = self.next_id;
        self.next_id += 1;
        let payload = json!({"jsonrpc":"2.0","id":id,"method":method,"params":params});
        self.send_payload(payload)
    }

    fn send_notification(&mut self, method: &str, params: Value) -> io::Result<()> {
        let payload = json!({"jsonrpc":"2.0","method":method,"params":params});
        self.send_payload(payload)
    }

    fn send_payload(&mut self, payload: Value) -> io::Result<()> {
        let body = serde_json::to_vec(&payload)
            .map_err(|error| io::Error::other(format!("LSP serialize failed: {error}")))?;
        let header = format!("Content-Length: {}\r\n\r\n", body.len());
        self.stdin.write_all(header.as_bytes())?;
        self.stdin.write_all(&body)?;
        self.stdin.flush()
    }
}
