use crate::app::App;
use crate::core::Cursor;

impl App {
    pub(crate) fn trigger_completion(&mut self, trigger: Option<char>) {
        let Some(path) = self.active_document().path().map(|path| path.to_path_buf()) else {
            return;
        };
        let cursor = self.active_pane().cursor();
        self.lsp.request_completion(
            &path,
            &self.workspace_root,
            cursor.line,
            cursor.column,
            trigger,
        );
    }

    pub(crate) fn accept_completion(&mut self) {
        let Some(item) = self.lsp.completion.selected_item().cloned() else {
            return;
        };
        let text = if item.is_snippet {
            crate::editor::snippet::expand_snippet_body(&item.insert_text)
        } else {
            item.insert_text
        };
        let cursor = self.active_pane().cursor();
        let replace_start = completion_prefix_start(self.active_document(), cursor);
        if replace_start != cursor {
            self.apply_edit(replace_start, cursor, &text, false);
        } else {
            self.insert_text(&text, false);
        }
        self.lsp.completion.close();
    }

    pub(crate) fn close_completion(&mut self) {
        self.lsp.completion.close();
    }

    pub(crate) fn completion_active(&self) -> bool {
        self.lsp.completion.active
    }
}

fn completion_prefix_start(document: &crate::core::Document, cursor: Cursor) -> Cursor {
    let line = document.raw_line_text(cursor.line);
    let chars = line.chars().collect::<Vec<_>>();
    let mut column = cursor.column.min(chars.len());

    while column > 0 {
        let character = chars[column - 1];
        if character.is_alphanumeric() || character == '_' {
            column -= 1;
            continue;
        }
        break;
    }

    Cursor::new(cursor.line, column)
}
