use std::collections::HashMap;
use std::sync::OnceLock;

use tree_sitter::Query;

use crate::syntax::LanguageId;

struct RegistryEntry {
    highlight_query: Query,
}

pub struct LanguageRegistry {
    entries: HashMap<LanguageId, RegistryEntry>,
}

static GLOBAL: OnceLock<LanguageRegistry> = OnceLock::new();

impl LanguageRegistry {
    pub fn global() -> &'static Self {
        GLOBAL.get_or_init(Self::build)
    }

    fn build() -> Self {
        let langs = [
            LanguageId::Rust,
            LanguageId::JavaScript,
            LanguageId::TypeScript,
            LanguageId::Python,
            LanguageId::Go,
            LanguageId::C,
            LanguageId::Cpp,
            LanguageId::Html,
            LanguageId::Css,
            LanguageId::Json,
            LanguageId::Toml,
            LanguageId::Yaml,
            LanguageId::Bash,
            LanguageId::Lua,
            LanguageId::Markdown,
        ];

        let mut entries = HashMap::new();
        for id in langs {
            let lang = id.ts_language();
            let source = id.highlight_query_source();
            if let Ok(highlight_query) = Query::new(&lang, source) {
                entries.insert(id, RegistryEntry { highlight_query });
            }
        }

        Self { entries }
    }

    pub fn highlight_query(&self, id: LanguageId) -> Option<&Query> {
        self.entries.get(&id).map(|e| &e.highlight_query)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_highlight_queries_compile() {
        let langs = [
            LanguageId::Rust,
            LanguageId::JavaScript,
            LanguageId::TypeScript,
            LanguageId::Python,
            LanguageId::Go,
            LanguageId::C,
            LanguageId::Cpp,
            LanguageId::Html,
            LanguageId::Css,
            LanguageId::Json,
            LanguageId::Toml,
            LanguageId::Yaml,
            LanguageId::Bash,
            LanguageId::Lua,
            LanguageId::Markdown,
        ];
        let mut failed = Vec::new();
        for id in langs {
            let lang = id.ts_language();
            let src = id.highlight_query_source();
            match Query::new(&lang, src) {
                Ok(q) => eprintln!("OK  {:?} ({} captures)", id, q.capture_names().len()),
                Err(e) => {
                    eprintln!("ERR {:?}: {:?}", id, e);
                    failed.push((id, e));
                }
            }
        }
        if !failed.is_empty() {
            panic!("{} queries failed — see stderr above", failed.len());
        }
    }
}
