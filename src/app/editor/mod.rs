mod edit;
mod file;
mod history;
mod selection;
mod viewport;

pub(crate) const TAB_WIDTH: usize = 4;

pub(crate) fn line_len_chars(line: &str) -> usize {
    line.chars().count()
}

pub(crate) fn byte_index_for_char(line: &str, char_index: usize) -> usize {
    line.char_indices()
        .nth(char_index)
        .map(|(index, _)| index)
        .unwrap_or(line.len())
}

fn leading_indent_width(line: &str) -> usize {
    let mut width = 0;
    for ch in line.chars() {
        if ch == ' ' && width < TAB_WIDTH {
            width += 1;
            continue;
        }
        if ch == '\t' {
            return if width == 0 { 1 } else { width };
        }
        break;
    }

    width
}
