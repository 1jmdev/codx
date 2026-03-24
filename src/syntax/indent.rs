use tree_sitter::Tree;

pub fn compute_indent(
    tree: Option<&Tree>,
    source: &[u8],
    cursor_byte: usize,
    current_indent: &str,
) -> String {
    let Some(tree) = tree else {
        return current_indent.to_owned();
    };

    let root = tree.root_node();
    let node = root.descendant_for_byte_range(cursor_byte.saturating_sub(1), cursor_byte);

    let Some(node) = node else {
        return current_indent.to_owned();
    };

    let should_increase = is_block_opener(node.kind()) || is_block_opener_byte(source, cursor_byte);

    if should_increase {
        format!("{}    ", current_indent)
    } else {
        current_indent.to_owned()
    }
}

fn is_block_opener(kind: &str) -> bool {
    matches!(
        kind,
        "{" | "(" | "[" | ":" | "do" | "then" | "else" | "elif"
    )
}

fn is_block_opener_byte(source: &[u8], cursor_byte: usize) -> bool {
    if cursor_byte == 0 {
        return false;
    }
    let byte = source[cursor_byte - 1];
    matches!(byte, b'{' | b'(' | b'[' | b':')
}
