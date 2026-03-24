use std::cell::RefCell;
use std::path::Path;

use crate::app::{App, FocusTarget};
use crate::core::{Document, History};
use crate::syntax::{language_for_path, SyntaxLayer};
use crate::ui::{PickerItem, PickerKind, PickerState, SplitDirection};
use crate::util::DetectedEncoding;

impl App {
    pub(crate) fn open_file_picker(&mut self) {
        self.focus = FocusTarget::Editor;
        self.picker = Some(PickerState::new(PickerKind::Files));
        self.refresh_picker();
    }

    pub(crate) fn open_buffer_picker(&mut self) {
        self.focus = FocusTarget::Editor;
        let mut picker = PickerState::new(PickerKind::Buffers);
        picker.set_buffer_items(self.buffer_picker_items(String::new()));
        self.picker = Some(picker);
    }

    pub(crate) fn close_picker(&mut self) {
        self.picker = None;
    }

    pub(crate) fn refresh_picker(&mut self) {
        let Some(kind) = self.picker.as_ref().map(|picker| picker.kind()) else {
            return;
        };
        match kind {
            PickerKind::Files => {
                let query = self
                    .picker
                    .as_ref()
                    .map(|picker| picker.query())
                    .unwrap_or_default();
                let items = self.file_finder.search(query, 20);
                if let Some(picker) = self.picker.as_mut() {
                    picker.set_file_items(items);
                }
            }
            PickerKind::Buffers => {
                let query = self
                    .picker
                    .as_ref()
                    .map(|picker| picker.query().to_lowercase())
                    .unwrap_or_default();
                let items = self.buffer_picker_items(query);
                if let Some(picker) = self.picker.as_mut() {
                    picker.set_buffer_items(items);
                }
            }
        }
    }

    pub(crate) fn accept_picker_selection(&mut self) -> Result<(), crate::app::AppError> {
        let selected = self
            .picker
            .as_ref()
            .and_then(|picker| picker.selected_item().cloned());
        let Some(item) = selected else {
            return Ok(());
        };

        if let Some(path) = item.path {
            self.open_path_in_active_pane(&path)?;
            if let (Some(line), Some(column)) = (item.line, item.column) {
                let cursor = crate::core::Cursor::new(line, column);
                if let Some(pane) = self.layout.focused_pane_mut() {
                    pane.set_cursor(cursor);
                    pane.set_selection(crate::core::Selection::caret(cursor));
                }
                self.ensure_cursor_visible();
            }
        } else if let Some(buffer_id) = item.buffer_id {
            self.switch_to_buffer(buffer_id);
        }

        self.picker = None;
        Ok(())
    }

    pub(crate) fn next_buffer(&mut self) {
        self.switch_buffer_by_offset(1);
    }

    pub(crate) fn previous_buffer(&mut self) {
        self.switch_buffer_by_offset(-1);
    }

    pub(crate) fn focus_next_pane(&mut self) {
        self.layout.focus_next();
        if let Some(pane) = self.layout.focused_pane() {
            self.active_buffer_id = pane.buffer_id();
        }
    }

    pub(crate) fn split_focused(&mut self, direction: SplitDirection) {
        let buffer_id = self.active_buffer_id;
        if self.layout.split_focused(direction, buffer_id).is_some() {
            self.active_buffer_id = buffer_id;
            self.ensure_cursor_visible();
        }
    }

    pub(crate) fn resize_focused_pane(&mut self, delta: i16) {
        self.layout.resize_focused(delta);
    }

    pub(crate) fn switch_to_buffer(&mut self, buffer_id: u64) {
        if let Some(pane) = self.layout.focused_pane_mut() {
            pane.set_buffer_id(buffer_id);
        }
        self.active_buffer_id = buffer_id;
        self.ensure_cursor_visible();
    }

    pub(crate) fn ensure_nonempty_buffer_set(&mut self) {
        if self.buffers.is_empty() {
            let buffer_id = self.push_buffer(
                Document::new_empty(None),
                History::default(),
                String::new(),
                DetectedEncoding::default(),
            );
            self.switch_to_buffer(buffer_id);
        }
    }

    pub(crate) fn push_buffer(
        &mut self,
        document: Document,
        history: History,
        saved_snapshot: String,
        encoding: DetectedEncoding,
    ) -> u64 {
        let buffer_id = self.next_buffer_id;
        self.next_buffer_id += 1;
        let language_id = document.path().and_then(language_for_path);
        let syntax = SyntaxLayer::new(language_id);
        self.buffers.push(crate::app::BufferState {
            id: buffer_id,
            document,
            history,
            saved_snapshot,
            encoding,
            syntax,
            line_highlight_cache: RefCell::new(crate::app::LineHighlightCache::default()),
            fold_cache: RefCell::new(crate::app::FoldCache::default()),
        });
        buffer_id
    }

    fn switch_buffer_by_offset(&mut self, delta: isize) {
        if self.buffers.is_empty() {
            return;
        }

        let index = self
            .buffers
            .iter()
            .position(|buffer| buffer.id == self.active_buffer_id)
            .unwrap_or(0);
        let len = self.buffers.len();
        let next = if delta.is_negative() {
            (index + len - (delta.unsigned_abs() % len)) % len
        } else {
            (index + delta as usize) % len
        };
        self.switch_to_buffer(self.buffers[next].id);
    }

    fn buffer_picker_items(&self, query: String) -> Vec<PickerItem> {
        self.buffers
            .iter()
            .filter(|buffer| {
                query.is_empty()
                    || buffer
                        .document
                        .path()
                        .map(|path| path.display().to_string().to_lowercase().contains(&query))
                        .unwrap_or_else(|| String::from("[no name]").contains(&query))
            })
            .map(|buffer| PickerItem {
                title: buffer
                    .document
                    .path()
                    .and_then(|path| path.file_name())
                    .map(|name| name.to_string_lossy().into_owned())
                    .unwrap_or_else(|| String::from("[No Name]")),
                subtitle: buffer
                    .document
                    .path()
                    .map(|path| path.display().to_string())
                    .unwrap_or_default(),
                path: buffer.document.path().map(Path::to_path_buf),
                buffer_id: Some(buffer.id),
                line: None,
                column: None,
            })
            .collect()
    }
}
