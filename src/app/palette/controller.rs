use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::App;

use super::{
    files::collect_project_files,
    types::{PaletteAction, PaletteCommand, PaletteKind, PaletteState, PaletteView, PreviewTarget},
};

const VISIBLE_ROWS: usize = 16;

impl App {
    pub(crate) fn open_palette(&mut self, kind: PaletteKind) {
        if kind == PaletteKind::Files {
            self.file_picker_cache = collect_project_files(&self.cwd);
        }

        self.completion = None;

        self.palette = Some(PaletteState {
            kind,
            query: String::new(),
            replace_text: String::new(),
            selected: 0,
        });
    }

    pub(crate) fn palette_view(&self) -> Option<PaletteView> {
        let state = self.palette.as_ref()?;
        let all_matches = self.palette_matches(state);

        let scroll = if state.selected >= VISIBLE_ROWS {
            state.selected - VISIBLE_ROWS + 1
        } else {
            0
        };

        let rows = all_matches
            .iter()
            .skip(scroll)
            .take(VISIBLE_ROWS)
            .map(|item| item.label.clone())
            .collect();

        let visible_selected = state.selected.saturating_sub(scroll);

        let (title, show_replace) = match state.kind {
            PaletteKind::Files => ("Go To File", false),
            PaletteKind::Commands => ("Command Palette", false),
            PaletteKind::GrepSearch => ("Search in Project", false),
            PaletteKind::GrepReplace => ("Replace in Project", true),
        };

        // Build preview target from the selected match
        let preview = all_matches
            .get(state.selected)
            .and_then(|m| match &m.action {
                PaletteAction::OpenFile(path) => Some(PreviewTarget {
                    path: path.clone(),
                    focus_line: 0,
                }),
                PaletteAction::OpenFileAt(path, line, _col) => Some(PreviewTarget {
                    path: path.clone(),
                    focus_line: *line,
                }),
                PaletteAction::Command(_) => None,
            });

        Some(PaletteView {
            title,
            query: state.query.clone(),
            replace_text: state.replace_text.clone(),
            rows,
            selected: visible_selected,
            scroll,
            total_matches: all_matches.len(),
            show_replace,
            preview,
        })
    }

    pub(crate) fn handle_palette_key(&mut self, key: KeyEvent) -> bool {
        if self.palette.is_none() {
            return false;
        }

        match key.code {
            KeyCode::Esc => self.palette = None,
            KeyCode::Up => {
                if let Some(state) = self.palette.as_mut() {
                    state.selected = state.selected.saturating_sub(1);
                }
            }
            KeyCode::Down => {
                let max = self
                    .palette
                    .as_ref()
                    .map(|state| self.palette_matches(state).len().saturating_sub(1))
                    .unwrap_or(0);
                if let Some(state) = self.palette.as_mut() {
                    state.selected = (state.selected + 1).min(max);
                }
            }
            KeyCode::Tab => {
                let is_grep_replace = self
                    .palette
                    .as_ref()
                    .is_some_and(|s| s.kind == PaletteKind::GrepReplace);
                if is_grep_replace {
                    if let Some(state) = self.palette.as_mut() {
                        std::mem::swap(&mut state.query, &mut state.replace_text);
                    }
                }
            }
            KeyCode::Backspace => {
                if let Some(state) = self.palette.as_mut() {
                    state.query.pop();
                    state.selected = 0;
                }
            }
            KeyCode::Enter => self.activate_palette_selection(),
            KeyCode::Char(ch)
                if key.modifiers.is_empty() || key.modifiers == KeyModifiers::SHIFT =>
            {
                if let Some(state) = self.palette.as_mut() {
                    state.query.push(ch);
                    state.selected = 0;
                }
            }
            _ => {}
        }

        true
    }

    fn activate_palette_selection(&mut self) {
        let Some(state) = self.palette.as_ref() else {
            return;
        };
        let matches = self.palette_matches(state);
        let Some(selected) = matches.get(state.selected) else {
            self.status = String::from("No match found.");
            return;
        };

        let action = selected.action.clone();
        self.palette = None;
        match action {
            PaletteAction::OpenFile(path) => {
                if let Err(error) = self.open_file(path) {
                    self.status = format!("Open failed: {error}");
                }
            }
            PaletteAction::OpenFileAt(path, line, col) => {
                if let Err(error) = self.open_file(path) {
                    self.status = format!("Open failed: {error}");
                } else {
                    self.cursor_line = line.min(self.lines.len().saturating_sub(1));
                    self.cursor_col = col;
                    self.preferred_col = col;
                    let view_height = self.ui.editor_inner.height as usize;
                    self.ensure_cursor_visible(view_height);
                }
            }
            PaletteAction::Command(PaletteCommand::ReloadLsp) => self.reload_lsp_server(),
        }
    }
}
