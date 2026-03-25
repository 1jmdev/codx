use std::collections::HashMap;
use std::path::Path;

use serde::Deserialize;

use crate::lsp::client::ServerConfig;
use crate::syntax::LanguageId;

#[derive(Debug, Deserialize)]
struct LanguagesConfig {
    #[serde(default)]
    lsp: LspSection,
}

#[derive(Debug, Default, Deserialize)]
struct LspSection {
    #[serde(default)]
    servers: HashMap<String, ServerTomlConfig>,
}

#[derive(Debug, Deserialize)]
struct ServerTomlConfig {
    command: String,
    #[serde(default)]
    args: Vec<String>,
    #[serde(default)]
    initialization_options: Option<serde_json::Value>,
}

pub fn load_server_config(workspace_root: &Path) -> HashMap<LanguageId, ServerConfig> {
    let mut map = default_servers();
    let config_path = workspace_root.join("assets/languages.toml");
    let text = match std::fs::read_to_string(config_path) {
        Ok(text) => text,
        Err(_) => return map,
    };

    let parsed: LanguagesConfig = match toml::from_str(&text) {
        Ok(parsed) => parsed,
        Err(_) => return map,
    };

    for (language_name, server) in parsed.lsp.servers {
        let Some(language) = language_name_to_id(&language_name) else {
            continue;
        };
        map.insert(
            language,
            ServerConfig {
                language_id: language_name,
                command: server.command,
                args: server.args,
                initialization_options: server.initialization_options,
            },
        );
    }
    map
}

fn language_name_to_id(name: &str) -> Option<LanguageId> {
    match name.trim().to_ascii_lowercase().as_str() {
        "rust" => Some(LanguageId::Rust),
        "javascript" => Some(LanguageId::JavaScript),
        "typescript" => Some(LanguageId::TypeScript),
        "python" => Some(LanguageId::Python),
        "go" => Some(LanguageId::Go),
        "c" => Some(LanguageId::C),
        "cpp" | "c++" => Some(LanguageId::Cpp),
        "html" => Some(LanguageId::Html),
        "css" => Some(LanguageId::Css),
        "json" => Some(LanguageId::Json),
        "toml" => Some(LanguageId::Toml),
        "yaml" => Some(LanguageId::Yaml),
        "bash" | "shell" => Some(LanguageId::Bash),
        "lua" => Some(LanguageId::Lua),
        "markdown" => Some(LanguageId::Markdown),
        _ => None,
    }
}

fn default_servers() -> HashMap<LanguageId, ServerConfig> {
    let mut map = HashMap::new();
    map.insert(
        LanguageId::Rust,
        ServerConfig {
            language_id: String::from("rust"),
            command: String::from("rust-analyzer"),
            args: Vec::new(),
            initialization_options: Some(serde_json::json!({
                "cargo": {
                    "autoreload": true,
                    "buildScripts": {
                        "enable": true
                    }
                },
                "procMacro": {
                    "enable": true
                },
                "checkOnSave": {
                    "enable": true
                }
            })),
        },
    );
    map.insert(
        LanguageId::Python,
        ServerConfig {
            language_id: String::from("python"),
            command: String::from("pyright-langserver"),
            args: vec![String::from("--stdio")],
            initialization_options: None,
        },
    );
    map.insert(
        LanguageId::TypeScript,
        ServerConfig {
            language_id: String::from("typescript"),
            command: String::from("typescript-language-server"),
            args: vec![String::from("--stdio")],
            initialization_options: None,
        },
    );
    map.insert(
        LanguageId::JavaScript,
        ServerConfig {
            language_id: String::from("javascript"),
            command: String::from("typescript-language-server"),
            args: vec![String::from("--stdio")],
            initialization_options: None,
        },
    );
    map
}
