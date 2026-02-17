use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::App;

use super::{
    files::collect_project_files,
    types::{PaletteAction, PaletteCommand, PaletteKind, PaletteState, PaletteView},
};

impl App {
    pub(crate) fn open_palette(&mut self, kind: PaletteKind) {
        if kind == PaletteKind::Files {
            self.file_picker_cache = collect_project_files(&self.cwd);
        }

        self.palette = Some(PaletteState {
            kind,
            query: String::new(),
            selected: 0,
        });
    }

    pub(crate) fn palette_view(&self) -> Option<PaletteView> {
        let state = self.palette.as_ref()?;
        let rows = self
            .palette_matches(state)
            .into_iter()
            .map(|item| item.label)
            .collect();

        Some(PaletteView {
            title: if state.kind == PaletteKind::Files {
                "Go To File"
            } else {
                "Command Palette"
            },
            query: state.query.clone(),
            rows,
            selected: state.selected,
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
            PaletteAction::Command(PaletteCommand::ReloadLsp) => self.reload_lsp_server(),
        }
    }
}
