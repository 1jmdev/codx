use std::path::Path;

#[derive(Clone, Copy)]
pub(crate) struct ServerSpec {
    pub(crate) key: &'static str,
    pub(crate) name: &'static str,
    pub(crate) command: &'static str,
    pub(crate) args: &'static [&'static str],
    pub(crate) language_id: &'static str,
}

impl ServerSpec {
    pub(crate) fn from_path(path: &Path) -> Option<Self> {
        let ext = path.extension()?.to_string_lossy();
        match ext.as_ref() {
            "rs" => Some(Self {
                key: "rust",
                name: "rust-analyzer",
                command: "rust-analyzer",
                args: &[],
                language_id: "rust",
            }),
            "py" => Some(Self {
                key: "python",
                name: "pylsp",
                command: "pylsp",
                args: &[],
                language_id: "python",
            }),
            "js" | "ts" | "tsx" | "jsx" => Some(Self {
                key: "ts",
                name: "typescript-language-server",
                command: "typescript-language-server",
                args: &["--stdio"],
                language_id: "typescript",
            }),
            "go" => Some(Self {
                key: "go",
                name: "gopls",
                command: "gopls",
                args: &[],
                language_id: "go",
            }),
            _ => None,
        }
    }
}
