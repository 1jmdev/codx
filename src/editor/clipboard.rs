use crate::app::{App, MessageKind};

impl App {
    pub(crate) fn copy_selection(&mut self) {
        if let Some(text) = self.selection_text() {
            match self.clipboard.as_mut() {
                Some(clipboard) => match clipboard.copy(&text) {
                    Ok(()) => self.set_message("Selection copied", MessageKind::Info),
                    Err(error) => self.set_message(&error.to_string(), MessageKind::Error),
                },
                None => self.set_message(
                    "System clipboard is unavailable in this environment",
                    MessageKind::Error,
                ),
            }
        }
    }

    pub(crate) fn cut_selection(&mut self) {
        if let Some(text) = self.selection_text() {
            match self.clipboard.as_mut() {
                Some(clipboard) => match clipboard.copy(&text) {
                    Ok(()) => {
                        if let Some((start, end)) = self.active_pane().selection().normalized() {
                            self.apply_edit(start, end, "", false);
                            self.set_message("Selection cut", MessageKind::Info);
                        }
                    }
                    Err(error) => self.set_message(&error.to_string(), MessageKind::Error),
                },
                None => self.set_message(
                    "System clipboard is unavailable in this environment",
                    MessageKind::Error,
                ),
            }
        }
    }

    pub(crate) fn paste(&mut self) {
        match self.clipboard.as_mut() {
            Some(clipboard) => match clipboard.paste() {
                Ok(text) => self.insert_text(&text, false),
                Err(error) => self.set_message(&error.to_string(), MessageKind::Error),
            },
            None => self.set_message(
                "System clipboard is unavailable in this environment",
                MessageKind::Error,
            ),
        }
    }
}
