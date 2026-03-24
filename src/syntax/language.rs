use std::path::Path;

use tree_sitter::Language;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LanguageId {
    Rust,
    JavaScript,
    TypeScript,
    Python,
    Go,
    C,
    Cpp,
    Html,
    Css,
    Json,
    Toml,
    Yaml,
    Bash,
    Lua,
    Markdown,
}

impl LanguageId {
    pub fn ts_language(self) -> Language {
        match self {
            LanguageId::Rust => tree_sitter_rust::LANGUAGE.into(),
            LanguageId::JavaScript => tree_sitter_javascript::LANGUAGE.into(),
            LanguageId::TypeScript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            LanguageId::Python => tree_sitter_python::LANGUAGE.into(),
            LanguageId::Go => tree_sitter_go::LANGUAGE.into(),
            LanguageId::C => tree_sitter_c::LANGUAGE.into(),
            LanguageId::Cpp => tree_sitter_cpp::LANGUAGE.into(),
            LanguageId::Html => tree_sitter_html::LANGUAGE.into(),
            LanguageId::Css => tree_sitter_css::LANGUAGE.into(),
            LanguageId::Json => tree_sitter_json::LANGUAGE.into(),
            LanguageId::Toml => tree_sitter_toml_ng::LANGUAGE.into(),
            LanguageId::Yaml => tree_sitter_yaml::LANGUAGE.into(),
            LanguageId::Bash => tree_sitter_bash::LANGUAGE.into(),
            LanguageId::Lua => tree_sitter_lua::LANGUAGE.into(),
            LanguageId::Markdown => tree_sitter_md::LANGUAGE.into(),
        }
    }

    pub fn highlight_query_source(self) -> &'static str {
        match self {
            LanguageId::Rust => include_str!("../../assets/queries/rust/highlights.scm"),
            LanguageId::JavaScript => include_str!("../../assets/queries/javascript/highlights.scm"),
            LanguageId::TypeScript => include_str!("../../assets/queries/typescript/highlights.scm"),
            LanguageId::Python => include_str!("../../assets/queries/python/highlights.scm"),
            LanguageId::Go => include_str!("../../assets/queries/go/highlights.scm"),
            LanguageId::C => include_str!("../../assets/queries/c/highlights.scm"),
            LanguageId::Cpp => include_str!("../../assets/queries/cpp/highlights.scm"),
            LanguageId::Html => include_str!("../../assets/queries/html/highlights.scm"),
            LanguageId::Css => include_str!("../../assets/queries/css/highlights.scm"),
            LanguageId::Json => include_str!("../../assets/queries/json/highlights.scm"),
            LanguageId::Toml => include_str!("../../assets/queries/toml/highlights.scm"),
            LanguageId::Yaml => include_str!("../../assets/queries/yaml/highlights.scm"),
            LanguageId::Bash => include_str!("../../assets/queries/bash/highlights.scm"),
            LanguageId::Lua => include_str!("../../assets/queries/lua/highlights.scm"),
            LanguageId::Markdown => include_str!("../../assets/queries/markdown/highlights.scm"),
        }
    }
}

pub fn language_for_path(path: &Path) -> Option<LanguageId> {
    let ext = path.extension()?.to_str()?;
    match ext {
        "rs" => Some(LanguageId::Rust),
        "js" | "mjs" | "cjs" => Some(LanguageId::JavaScript),
        "ts" | "mts" | "cts" => Some(LanguageId::TypeScript),
        "py" | "pyi" | "pyw" => Some(LanguageId::Python),
        "go" => Some(LanguageId::Go),
        "c" | "h" => Some(LanguageId::C),
        "cpp" | "cxx" | "cc" | "hpp" | "hxx" | "hh" => Some(LanguageId::Cpp),
        "html" | "htm" => Some(LanguageId::Html),
        "css" => Some(LanguageId::Css),
        "json" | "jsonc" => Some(LanguageId::Json),
        "toml" => Some(LanguageId::Toml),
        "yaml" | "yml" => Some(LanguageId::Yaml),
        "sh" | "bash" | "zsh" => Some(LanguageId::Bash),
        "lua" => Some(LanguageId::Lua),
        "md" | "markdown" => Some(LanguageId::Markdown),
        _ => None,
    }
}
