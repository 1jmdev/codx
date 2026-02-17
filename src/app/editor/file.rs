use std::{fs, io, path::PathBuf};

use crate::app::{App, Focus};

impl App {
    pub(crate) fn open_file(&mut self, path: PathBuf) -> io::Result<()> {
        let content = fs::read_to_string(&path)?;
        let mut lines: Vec<String> = content.split('\n').map(|line| line.to_string()).collect();
        if lines.is_empty() {
            lines.push(String::new());
        }

        self.current_file = Some(path.clone());
        self.lines = lines;
        self.cursor_line = 0;
        self.cursor_col = 0;
        self.preferred_col = 0;
        self.selection_anchor = None;
        self.editor_scroll = 0;
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.dirty = false;
        self.status = format!("Opened {}", path.display());
        self.focus = Focus::Editor;
        self.lsp
            .open_file(&path, self.lines.join("\n"), &mut self.status);
        Ok(())
    }

    pub(crate) fn save_file(&mut self) -> io::Result<()> {
        let Some(path) = self.current_file.clone() else {
            self.status = String::from("No file selected. Open a file from the tree first.");
            return Ok(());
        };

        fs::write(&path, self.lines.join("\n"))?;
        self.dirty = false;
        self.notify_lsp_save();
        self.status = format!("Saved {}", path.display());
        Ok(())
    }
}
