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

pub(crate) fn previous_word_boundary(line: &str, col: usize) -> usize {
    let chars: Vec<char> = line.chars().collect();
    let mut idx = col.min(chars.len());
    while idx > 0 && chars[idx - 1].is_whitespace() {
        idx -= 1;
    }
    if idx == 0 {
        return 0;
    }

    let mode = is_word_char(chars[idx - 1]);
    while idx > 0 {
        let ch = chars[idx - 1];
        if ch.is_whitespace() || is_word_char(ch) != mode {
            break;
        }
        idx -= 1;
    }
    idx
}

fn is_word_char(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_'
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
