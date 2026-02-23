use lsp_types::{CompletionItem as LspCompletionItem, CompletionTextEdit, InsertTextFormat};

pub(super) fn completion_insert_text(item: &LspCompletionItem) -> String {
    let from_text_edit = match item.text_edit.as_ref() {
        Some(CompletionTextEdit::Edit(edit)) => Some(edit.new_text.clone()),
        Some(CompletionTextEdit::InsertAndReplace(edit)) => Some(edit.new_text.clone()),
        None => None,
    };

    let base = from_text_edit
        .or_else(|| item.insert_text.clone())
        .unwrap_or_else(|| item.label.clone());

    if item.insert_text_format == Some(InsertTextFormat::SNIPPET) {
        strip_snippet_markers(&base)
    } else {
        base
    }
}

pub(super) fn completion_replace_range(
    item: &LspCompletionItem,
    line: usize,
    default_start: usize,
    default_end: usize,
) -> Option<(usize, usize)> {
    match item.text_edit.as_ref() {
        Some(CompletionTextEdit::Edit(edit)) => {
            if edit.range.start.line as usize != line || edit.range.end.line as usize != line {
                return None;
            }
            Some((
                edit.range.start.character as usize,
                edit.range.end.character as usize,
            ))
        }
        Some(CompletionTextEdit::InsertAndReplace(edit)) => {
            if edit.insert.start.line as usize != line || edit.insert.end.line as usize != line {
                return None;
            }
            Some((
                edit.insert.start.character as usize,
                edit.insert.end.character as usize,
            ))
        }
        None => Some((default_start, default_end)),
    }
}

fn strip_snippet_markers(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '$' {
            match chars.peek() {
                Some('{') => {
                    let _ = chars.next();
                    let mut placeholder = String::new();
                    for part in chars.by_ref() {
                        if part == '}' {
                            break;
                        }
                        placeholder.push(part);
                    }
                    if let Some((_, value)) = placeholder.split_once(':') {
                        out.push_str(value);
                    }
                }
                Some(next) if next.is_ascii_digit() => {
                    while let Some(next) = chars.peek() {
                        if next.is_ascii_digit() {
                            let _ = chars.next();
                        } else {
                            break;
                        }
                    }
                }
                _ => out.push(ch),
            }
        } else {
            out.push(ch);
        }
    }

    out
}
