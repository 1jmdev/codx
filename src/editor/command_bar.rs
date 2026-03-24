use std::path::{Path, PathBuf};

use crate::app::{App, AppError, AppMode, CommandBarMode, MessageKind};

impl App {
    pub(crate) fn begin_save_as_prompt(&mut self) {
        self.command_bar.input = self
            .active_document()
            .path()
            .map(Path::to_path_buf)
            .map(|path| path.to_string_lossy().into_owned())
            .unwrap_or_default();
        self.command_bar.original_search_query = None;
        self.mode = AppMode::CommandBar(CommandBarMode::SaveAs);
    }

    pub(crate) fn begin_search_prompt(&mut self) {
        self.command_bar.input = self.active_pane().search().confirmed_query().to_owned();
        self.command_bar.original_search_query = Some(self.command_bar.input.clone());
        self.with_active_search_state(|search, document, cursor| {
            search.begin_preview(search.confirmed_query().to_owned(), document, cursor);
        });
        self.mode = AppMode::CommandBar(CommandBarMode::Search);
    }

    pub(crate) fn begin_command_prompt(&mut self) {
        self.command_bar.input.clear();
        self.command_bar.original_search_query = None;
        self.mode = AppMode::CommandBar(CommandBarMode::Command);
    }

    pub(crate) fn cancel_command_bar(&mut self) {
        if matches!(self.mode, AppMode::CommandBar(CommandBarMode::Search)) {
            if let Some(original) = self.command_bar.original_search_query.clone() {
                self.with_active_search_state(|search, document, cursor| {
                    search.begin_preview(original.clone(), document, cursor);
                    search.confirm_preview();
                    search.restore_confirmed(document, cursor);
                });
            } else {
                self.active_pane_mut().search_mut().clear();
            }
        }

        self.command_bar.input.clear();
        self.command_bar.original_search_query = None;
        self.mode = AppMode::Editing;
    }

    pub(crate) fn submit_command_bar(&mut self, mode: CommandBarMode) -> Result<(), AppError> {
        match mode {
            CommandBarMode::SaveAs => {
                let path = PathBuf::from(self.command_bar.input.trim());
                if path.as_os_str().is_empty() {
                    self.set_message("Save path cannot be empty", MessageKind::Error);
                    return Ok(());
                }
                self.save_to_path(&path)?;
                self.mode = AppMode::Editing;
            }
            CommandBarMode::Search => {
                if self.command_bar.input.is_empty() {
                    self.active_pane_mut().search_mut().clear();
                } else {
                    let query = self.command_bar.input.clone();
                    self.with_active_search_state(|search, document, cursor| {
                        search.update_preview(query.clone(), document, cursor);
                        search.confirm_preview();
                    });
                    self.jump_to_active_search_match();
                }
                self.mode = AppMode::Editing;
            }
            CommandBarMode::Command => {
                let command = self.command_bar.input.trim().to_owned();
                self.execute_command_bar_command(&command)?;
                self.mode = AppMode::Editing;
            }
            CommandBarMode::ExplorerCreateFile => {
                let path = self.explorer.create(self.command_bar.input.trim(), false)?;
                self.file_finder.refresh();
                self.explorer.refresh();
                self.open_path_in_active_pane(&path)?;
                self.mode = AppMode::Editing;
            }
            CommandBarMode::ExplorerCreateDirectory => {
                let _ = self.explorer.create(self.command_bar.input.trim(), true)?;
                self.file_finder.refresh();
                self.explorer.refresh();
                self.mode = AppMode::Editing;
            }
            CommandBarMode::ExplorerRename => {
                let path = self.explorer.rename_selected(self.command_bar.input.trim())?;
                self.file_finder.refresh();
                self.explorer.refresh();
                self.set_message(&format!("Renamed to {}", path.display()), MessageKind::Info);
                self.mode = AppMode::Editing;
            }
        }

        self.command_bar.input.clear();
        self.command_bar.original_search_query = None;
        Ok(())
    }

    pub(crate) fn update_search_preview(&mut self) {
        if self.command_bar.input.is_empty() {
            self.active_pane_mut().search_mut().clear();
            return;
        }

        let query = self.command_bar.input.clone();
        self.with_active_search_state(|search, document, cursor| {
            search.update_preview(query.clone(), document, cursor);
        });
    }

    fn execute_command_bar_command(&mut self, command: &str) -> Result<(), AppError> {
        match command {
            "w" => self.save_or_prompt()?,
            "q" => self.request_quit(),
            "wq" => {
                self.pending_quit_after_save = true;
                self.save_or_prompt()?;
            }
            "explorer" => self.toggle_explorer(),
            "files" => self.open_file_picker(),
            "buffers" => self.open_buffer_picker(),
            "split" => self.split_focused(crate::ui::SplitDirection::Vertical),
            "vsplit" => self.split_focused(crate::ui::SplitDirection::Horizontal),
            "reload" => self.reload_changed_files()?,
            _ if command.starts_with("find ") => {
                let query = command.trim_start_matches("find ").to_owned();
                self.command_bar.input = query.clone();
                self.with_active_search_state(|search, document, cursor| {
                    search.begin_preview(query.clone(), document, cursor);
                    search.confirm_preview();
                });
                self.jump_to_active_search_match();
            }
            _ => self.set_message("Unknown command", MessageKind::Warning),
        }

        Ok(())
    }

    fn with_active_search_state(
        &mut self,
        mut apply: impl FnMut(
            &mut crate::editor::SearchState,
            &crate::core::Document,
            crate::core::Cursor,
        ),
    ) {
        let pane_id = self.active_pane_id();
        let cursor = self.active_pane().cursor();
        let buffer_id = self.active_buffer_id;
        let (layout, buffers) = (&mut self.layout, &self.buffers);
        if let (Some(pane), Some(buffer)) = (
            layout.pane_mut(pane_id),
            buffers.iter().find(|buffer| buffer.id == buffer_id),
        ) {
            apply(pane.search_mut(), &buffer.document, cursor);
        }
    }
}
