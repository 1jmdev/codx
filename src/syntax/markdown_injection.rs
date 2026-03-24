use tree_sitter::{Query, QueryCursor, StreamingIterator, Tree};

use crate::syntax::{
    HighlightSpan, LanguageId, LanguageRegistry, language_for_name, spans_for_line,
};

pub fn markdown_code_block_spans_for_line(
    tree: &Tree,
    source: &[u8],
    line_start_byte: usize,
    line_end_byte: usize,
) -> Vec<HighlightSpan> {
    let mut cursor = QueryCursor::new();
    let query = match markdown_injection_query() {
        Some(query) => query,
        None => return Vec::new(),
    };

    let root = tree.root_node();
    let names = query.capture_names();
    let mut matches = cursor.matches(query, root, source);
    let mut spans = Vec::new();

    while let Some(mat) = matches.next() {
        let mut language_name = None;
        let mut content_range = None;

        for capture in mat.captures {
            let name = names
                .get(capture.index as usize)
                .copied()
                .unwrap_or_default();

            if name == "language" {
                let text = &source[capture.node.start_byte()..capture.node.end_byte()];
                let trimmed = String::from_utf8_lossy(text).trim().to_owned();
                if !trimmed.is_empty() {
                    language_name = Some(trimmed);
                }
            }

            if name == "content" {
                content_range = Some((capture.node.start_byte(), capture.node.end_byte()));
            }
        }

        let Some(lang_name) = language_name else {
            continue;
        };
        let Some((content_start, content_end)) = content_range else {
            continue;
        };

        if content_end <= line_start_byte || content_start >= line_end_byte {
            continue;
        }

        let Some(language_id) = language_for_name(&lang_name) else {
            continue;
        };
        if language_id == LanguageId::Markdown {
            continue;
        }

        let registry = LanguageRegistry::global();
        let Some(lang_query) = registry.highlight_query(language_id) else {
            continue;
        };

        let block_source = &source[content_start..content_end];
        let mut parser = tree_sitter::Parser::new();
        let lang = language_id.ts_language();
        if parser.set_language(&lang).is_err() {
            continue;
        }

        let Some(block_tree) = parser.parse(block_source, None) else {
            continue;
        };

        let overlap_start = line_start_byte.max(content_start) - content_start;
        let overlap_end = line_end_byte.min(content_end) - content_start;

        let mut nested = spans_for_line(
            &block_tree,
            lang_query,
            block_source,
            overlap_start,
            overlap_end,
        );

        let shift = content_start.saturating_sub(line_start_byte);
        for span in &mut nested {
            span.start_byte = span.start_byte.saturating_add(shift);
            span.end_byte = span.end_byte.saturating_add(shift);
        }
        spans.extend(nested);
    }

    spans
}

fn markdown_injection_query() -> Option<&'static Query> {
    static QUERY: std::sync::OnceLock<Option<Query>> = std::sync::OnceLock::new();
    QUERY
        .get_or_init(|| {
            let source = r#"
((fenced_code_block
   (info_string (language) @language)
   (code_fence_content) @content))
"#;
            Query::new(&tree_sitter_md::LANGUAGE.into(), source).ok()
        })
        .as_ref()
}
