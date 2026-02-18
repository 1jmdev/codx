use std::{
    collections::HashMap,
    io::{self, BufReader, Write},
    path::{Path, PathBuf},
    process::{Child, ChildStdin, Command, Stdio},
    sync::{
        Arc, Mutex,
        mpsc::{self, Receiver},
    },
    thread,
};

use lsp_types::{
    CompletionParams, CompletionResponse, DidChangeTextDocumentParams, DidOpenTextDocumentParams,
    DidSaveTextDocumentParams, InitializeParams, InitializedParams, Position,
    PublishDiagnosticsParams, TextDocumentContentChangeEvent, TextDocumentIdentifier,
    TextDocumentItem, TextDocumentPositionParams, Uri, VersionedTextDocumentIdentifier,
    WorkspaceFolder,
    notification::Notification,
    notification::{
        DidChangeTextDocument, DidOpenTextDocument, DidSaveTextDocument, Initialized,
        PublishDiagnostics,
    },
    request::Completion,
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
    Completion {
        path: PathBuf,
        version: i32,
        line: usize,
        col: usize,
        items: Vec<lsp_types::CompletionItem>,
    },
}

enum PendingRequest {
    Completion {
        path: PathBuf,
        version: i32,
        line: usize,
        col: usize,
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
    pending_requests: Arc<Mutex<HashMap<i64, PendingRequest>>>,
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

        let pending_requests = Arc::new(Mutex::new(HashMap::<i64, PendingRequest>::new()));
        let pending_reader = Arc::clone(&pending_requests);
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

                let response_id = value.get("id").and_then(Value::as_i64);
                let result = value.get("result");
                if let (Some(id), Some(result)) = (response_id, result) {
                    let pending = pending_reader
                        .lock()
                        .ok()
                        .and_then(|mut locked| locked.remove(&id));
                    if let Some(PendingRequest::Completion {
                        path,
                        version,
                        line,
                        col,
                    }) = pending
                    {
                        let items = completion_items_from_result(result);
                        let _ = tx.send(ServerEvent::Completion {
                            path,
                            version,
                            line,
                            col,
                            items,
                        });
                    }
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
            pending_requests,
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

    pub(crate) fn request_completion(
        &mut self,
        path: &Path,
        line: usize,
        col: usize,
        version: i32,
    ) -> io::Result<()> {
        let uri = match &self.open_uri {
            Some(uri) => uri.clone(),
            None => match path_to_uri(path) {
                Some(uri) => uri,
                None => return Ok(()),
            },
        };

        let params = CompletionParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri },
                position: Position::new(line as u32, col as u32),
            },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
            context: Some(lsp_types::CompletionContext {
                trigger_kind: lsp_types::CompletionTriggerKind::INVOKED,
                trigger_character: None,
            }),
        };

        let payload = serde_json::to_value(params).map_err(|error| {
            io::Error::other(format!("LSP completion serialize failed: {error}"))
        })?;
        let id = self.next_id;
        self.next_id += 1;
        if let Ok(mut pending) = self.pending_requests.lock() {
            pending.insert(
                id,
                PendingRequest::Completion {
                    path: path.to_path_buf(),
                    version,
                    line,
                    col,
                },
            );
        }

        let request = json!({"jsonrpc":"2.0","id":id,"method":Completion::METHOD,"params":payload});
        if let Err(error) = self.send_payload(request) {
            if let Ok(mut pending) = self.pending_requests.lock() {
                pending.remove(&id);
            }
            return Err(error);
        }

        Ok(())
    }

    fn send_request(&mut self, method: &str, params: Value) -> io::Result<i64> {
        let id = self.next_id;
        self.next_id += 1;
        let payload = json!({"jsonrpc":"2.0","id":id,"method":method,"params":params});
        self.send_payload(payload)?;
        Ok(id)
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

fn completion_items_from_result(result: &Value) -> Vec<lsp_types::CompletionItem> {
    if result.is_null() {
        return Vec::new();
    }

    let Ok(response) = serde_json::from_value::<CompletionResponse>(result.clone()) else {
        return Vec::new();
    };

    match response {
        CompletionResponse::Array(items) => items,
        CompletionResponse::List(list) => list.items,
    }
}
