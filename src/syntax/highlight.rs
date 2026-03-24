use tree_sitter::{Query, QueryCursor, StreamingIterator, Tree};

const CAPTURE_NAMES: &[&str] = &[
    "comment",
    "comment.doc",
    "keyword",
    "keyword.function",
    "keyword.storage",
    "keyword.type",
    "keyword.operator",
    "keyword.return",
    "keyword.control",
    "string",
    "string.escape",
    "number",
    "float",
    "boolean",
    "operator",
    "punctuation",
    "punctuation.delimiter",
    "punctuation.bracket",
    "function",
    "function.method",
    "function.builtin",
    "function.macro",
    "type",
    "type.builtin",
    "variable",
    "variable.parameter",
    "variable.builtin",
    "constant",
    "constant.builtin",
    "attribute",
    "property",
    "namespace",
    "escape",
    "label",
    "special",
    "error",
];

fn intern_capture_name(name: &str) -> &'static str {
    CAPTURE_NAMES
        .iter()
        .find(|&&n| n == name)
        .copied()
        .unwrap_or("variable")
}

#[derive(Debug, Clone)]
pub struct HighlightSpan {
    pub start_byte: usize,
    pub end_byte: usize,
    pub capture: &'static str,
}

pub fn spans_for_line(
    tree: &Tree,
    query: &Query,
    source: &[u8],
    line_start_byte: usize,
    line_end_byte: usize,
) -> Vec<HighlightSpan> {
    let mut cursor = QueryCursor::new();
    cursor.set_byte_range(line_start_byte..line_end_byte);

    let root = tree.root_node();
    let names = query.capture_names();

    let mut spans = Vec::new();
    let mut matches = cursor.matches(query, root, source);

    while let Some(mat) = matches.next() {
        for capture in mat.captures {
            let node = capture.node;
            let node_start = node.start_byte();
            let node_end = node.end_byte();

            if node_end <= line_start_byte || node_start >= line_end_byte {
                continue;
            }

            let start_byte = node_start.saturating_sub(line_start_byte);
            let end_byte = node_end.saturating_sub(line_start_byte);

            let raw_name = names
                .get(capture.index as usize)
                .copied()
                .unwrap_or("variable");
            let capture_name = intern_capture_name(raw_name);

            spans.push(HighlightSpan {
                start_byte,
                end_byte,
                capture: capture_name,
            });
        }
    }

    spans
}
