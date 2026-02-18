use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::App;

use super::{
    files::collect_project_files,
    types::{PaletteAction, PaletteCommand, PaletteKind, PaletteState, PaletteView, PreviewTarget},
};

const MIN_VISIBLE_ROWS: usize = 1;
const FALLBACK_VISIBLE_ROWS: usize = 16;

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
            preview_active: false,
            preview_scroll: 0,
        });
    }

    pub(crate) fn palette_view(&self) -> Option<PaletteView> {
        let state = self.palette.as_ref()?;
        let all_matches = self.palette_matches(state);
        let visible_rows = self.palette_visible_rows(state);

        let scroll = if state.selected >= visible_rows {
            state.selected - visible_rows + 1
        } else {
            0
        };

        let rows = all_matches
            .iter()
            .skip(scroll)
            .take(visible_rows)
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

        let preview_active = state.preview_active && preview.is_some();

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
            preview_active,
            preview_scroll: state.preview_scroll,
        })
    }

    pub(crate) fn move_palette_selection(&mut self, delta_rows: isize) {
        let preview_active = self
            .palette
            .as_ref()
            .is_some_and(|state| state.preview_active);
        if preview_active {
            self.scroll_palette_preview(delta_rows);
            return;
        }

        let max = self
            .palette
            .as_ref()
            .map(|state| self.palette_matches(state).len().saturating_sub(1))
            .unwrap_or(0);
        if let Some(state) = self.palette.as_mut() {
            let next = state.selected as isize + delta_rows;
            state.selected = next.clamp(0, max as isize) as usize;
            state.preview_scroll = 0;
        }
    }

    pub(crate) fn scroll_palette_preview(&mut self, delta_lines: isize) {
        let has_preview = self.palette_has_preview_target();
        if !has_preview {
            return;
        }

        let default_scroll = self.palette_default_preview_scroll();

        let Some(state) = self.palette.as_mut() else {
            return;
        };

        if !state.preview_active {
            state.preview_scroll = default_scroll;
        }
        state.preview_active = true;
        if delta_lines < 0 {
            state.preview_scroll = state.preview_scroll.saturating_sub((-delta_lines) as usize);
        } else {
            state.preview_scroll = state.preview_scroll.saturating_add(delta_lines as usize);
        }
    }

    pub(crate) fn handle_palette_key(&mut self, key: KeyEvent) -> bool {
        if self.palette.is_none() {
            return false;
        }

        match key.code {
            KeyCode::Esc => self.palette = None,
            KeyCode::Up => {
                if let Some(state) = self.palette.as_mut() {
                    if state.preview_active {
                        state.preview_scroll = state.preview_scroll.saturating_sub(1);
                    } else {
                        state.selected = state.selected.saturating_sub(1);
                        state.preview_scroll = 0;
                    }
                }
            }
            KeyCode::Down => {
                let max = self.palette_max_selection_index();
                if let Some(state) = self.palette.as_mut() {
                    if state.preview_active {
                        state.preview_scroll = state.preview_scroll.saturating_add(1);
                    } else {
                        state.selected = (state.selected + 1).min(max);
                        state.preview_scroll = 0;
                    }
                }
            }
            KeyCode::Left => {
                if let Some(state) = self.palette.as_mut() {
                    state.preview_active = false;
                }
            }
            KeyCode::Right => {
                let has_preview = self.palette_has_preview_target();
                if has_preview {
                    let default_scroll = self.palette_default_preview_scroll();
                    if let Some(state) = self.palette.as_mut() {
                        if !state.preview_active {
                            state.preview_scroll = default_scroll;
                        }
                        state.preview_active = true;
                    }
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
                        state.preview_active = false;
                        state.preview_scroll = 0;
                    }
                }
            }
            KeyCode::Backspace => {
                if let Some(state) = self.palette.as_mut() {
                    state.query.pop();
                    state.selected = 0;
                    state.preview_active = false;
                    state.preview_scroll = 0;
                }
            }
            KeyCode::Enter => self.activate_palette_selection(),
            KeyCode::Char(ch)
                if key.modifiers.is_empty() || key.modifiers == KeyModifiers::SHIFT =>
            {
                if let Some(state) = self.palette.as_mut() {
                    state.query.push(ch);
                    state.selected = 0;
                    state.preview_active = false;
                    state.preview_scroll = 0;
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

    fn palette_max_selection_index(&self) -> usize {
        self.palette
            .as_ref()
            .map(|state| self.palette_matches(state).len().saturating_sub(1))
            .unwrap_or(0)
    }

    fn palette_has_preview_target(&self) -> bool {
        let Some(state) = self.palette.as_ref() else {
            return false;
        };
        let matches = self.palette_matches(state);
        let Some(selected) = matches.get(state.selected) else {
            return false;
        };
        matches!(
            selected.action,
            PaletteAction::OpenFile(_) | PaletteAction::OpenFileAt(_, _, _)
        )
    }

    fn palette_default_preview_scroll(&self) -> usize {
        let Some(state) = self.palette.as_ref() else {
            return 0;
        };

        let matches = self.palette_matches(state);
        let Some(selected) = matches.get(state.selected) else {
            return 0;
        };

        let focus_line = match selected.action {
            PaletteAction::OpenFile(_) => 0,
            PaletteAction::OpenFileAt(_, line, _) => line,
            PaletteAction::Command(_) => return 0,
        };

        let visible_rows = self.ui.palette_preview.height.max(1) as usize;
        focus_line.saturating_sub(visible_rows / 2)
    }
}

impl App {
    fn palette_visible_rows(&self, state: &PaletteState) -> usize {
        let panel_rows = self.ui.palette_results.height as usize;
        if panel_rows == 0 {
            return FALLBACK_VISIBLE_ROWS;
        }

        let fixed_rows = if state.kind == PaletteKind::GrepReplace {
            3
        } else {
            2
        };

        panel_rows.saturating_sub(fixed_rows).max(MIN_VISIBLE_ROWS)
    }
}
