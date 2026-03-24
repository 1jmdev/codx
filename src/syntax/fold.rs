use tree_sitter::Tree;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FoldRange {
    pub start_line: usize,
    pub end_line: usize,
}

pub fn compute_folds(tree: &Tree) -> Vec<FoldRange> {
    let root = tree.root_node();
    let mut folds = Vec::new();
    collect_folds(root, &mut folds);
    folds
}

fn is_foldable_node(kind: &str) -> bool {
    matches!(
        kind,
        "block"
            | "function_item"
            | "impl_item"
            | "struct_item"
            | "enum_item"
            | "trait_item"
            | "mod_item"
            | "function_declaration"
            | "function"
            | "method_definition"
            | "class_declaration"
            | "class_body"
            | "if_statement"
            | "for_statement"
            | "while_statement"
            | "object"
            | "array"
            | "table"
            | "section"
    )
}

fn collect_folds(node: tree_sitter::Node<'_>, folds: &mut Vec<FoldRange>) {
    let start = node.start_position().row;
    let end = node.end_position().row;

    if is_foldable_node(node.kind()) && end > start {
        folds.push(FoldRange {
            start_line: start,
            end_line: end,
        });
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_folds(child, folds);
    }
}
