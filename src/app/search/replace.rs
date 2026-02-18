use crate::app::App;

impl App {
    /// Replace the currently highlighted match and advance to the next one.
    pub(super) fn replace_current(&mut self) {
        let (line_idx, col_start, col_end, replacement) = {
            let Some(sr) = self.search_replace.as_ref() else {
                return;
            };
            let Some(&(l, cs, ce)) = sr.matches.get(sr.current_match) else {
                return;
            };
            (l, cs, ce, sr.replacement.clone())
        };

        self.begin_edit();
        let line = &mut self.lines[line_idx];
        let byte_start = byte_index_for_char(line, col_start);
        let byte_end = byte_index_for_char(line, col_end);
        line.replace_range(byte_start..byte_end, &replacement);
        self.mark_changed();

        self.refresh_matches();
        self.goto_next_match();
        self.status = String::from("Replaced 1 occurrence.");
    }

    /// Replace every match in the document at once.
    pub(crate) fn replace_all(&mut self) {
        let (query, replacement) = {
            let Some(sr) = self.search_replace.as_ref() else {
                return;
            };
            if sr.query.is_empty() {
                return;
            }
            (sr.query.clone(), sr.replacement.clone())
        };

        let q_chars: Vec<char> = query.chars().collect();
        let qlen = q_chars.len();
        let mut total = 0usize;

        self.begin_edit();
        for line in self.lines.iter_mut() {
            let chars: Vec<char> = line.chars().collect();
            if chars.len() < qlen {
                continue;
            }
            let mut new_line = String::new();
            let mut i = 0usize;
            while i + qlen <= chars.len() {
                let hit = chars[i..i + qlen]
                    .iter()
                    .zip(q_chars.iter())
                    .all(|(a, b)| a.eq_ignore_ascii_case(b));
                if hit {
                    new_line.push_str(&replacement);
                    i += qlen;
                    total += 1;
                } else {
                    new_line.push(chars[i]);
                    i += 1;
                }
            }
            while i < chars.len() {
                new_line.push(chars[i]);
                i += 1;
            }
            *line = new_line;
        }

        self.mark_changed();
        self.refresh_matches();
        self.status = format!("Replaced {total} occurrence(s).");
    }
}

fn byte_index_for_char(line: &str, char_index: usize) -> usize {
    line.char_indices()
        .nth(char_index)
        .map(|(i, _)| i)
        .unwrap_or(line.len())
}
