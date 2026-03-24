use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::layout::Size;

use crate::app::{App, AppError, AppMode, CommandBarMode, FocusTarget};
use crate::editor::Command;
use crate::keymap::map_key_event;

pub(crate) fn run_app(app: &mut App) -> Result<(), AppError> {
    let mut terminal_session = crate::app::TerminalSession::enter()?;
    let terminal = terminal_session.terminal_mut();
    let size = terminal.size()?;
    app.set_terminal_size(size);
    app.ensure_cursor_visible();

    loop {
        app.poll_background_tasks();
        app.update_dirty_syntax_layers();
        terminal.draw(|frame| crate::ui::render(frame, app))?;

        if app.should_quit {
            return Ok(());
        }

        if event::poll(Duration::from_millis(250))? {
            match event::read()? {
                Event::Key(key_event)
                    if matches!(key_event.kind, KeyEventKind::Press | KeyEventKind::Repeat) =>
                {
                    app.handle_key_event(key_event)?;
                }
                Event::Resize(width, height) => {
                    app.set_terminal_size(Size::new(width, height));
                    app.ensure_cursor_visible();
                }
                _ => {}
            }
        }
    }
}

impl App {
    pub(crate) fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<(), AppError> {
        if !matches!(
            self.mode,
            AppMode::ConfirmQuit
                | AppMode::ConfirmDeleteExplorerEntry
                | AppMode::ExternalChangeConflict
        ) {
            self.clear_message();
        }

        if self.picker.is_some() {
            return self.handle_picker_key(key_event);
        }

        match self.mode {
            AppMode::ConfirmQuit => self.handle_confirm_quit_key(key_event),
            AppMode::ConfirmDeleteExplorerEntry => self.handle_confirm_delete_explorer_key(key_event),
            AppMode::ExternalChangeConflict => self.handle_external_change_conflict_key(key_event),
            AppMode::CommandBar(mode) => self.handle_command_bar_key(mode, key_event),
            AppMode::Editing => {
                if self.focus == FocusTarget::Explorer && self.explorer.visible() {
                    self.handle_explorer_key(key_event)
                } else {
                    self.handle_editing_key(key_event)
                }
            }
        }
    }

    fn handle_editing_key(&mut self, key_event: KeyEvent) -> Result<(), AppError> {
        if let Some(command) = map_key_event(key_event) {
            self.apply_command(command)?;
            return Ok(());
        }

        match key_event.code {
            KeyCode::Char(ch) if !key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                self.insert_text(&ch.to_string(), true);
            }
            KeyCode::Enter => self.insert_newline_with_indent(),
            KeyCode::Tab => self.insert_text("    ", false),
            KeyCode::Backspace => self.backspace(),
            KeyCode::Delete => self.delete_forward(),
            _ => {}
        }
        Ok(())
    }

    fn handle_explorer_key(&mut self, key_event: KeyEvent) -> Result<(), AppError> {
        match key_event.code {
            KeyCode::Esc => self.focus = FocusTarget::Editor,
            KeyCode::Char('q') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                self.apply_command(crate::editor::Command::Quit)?;
            }
            KeyCode::Char('b') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                self.apply_command(crate::editor::Command::ToggleExplorer)?;
            }
            KeyCode::Up => self.explorer.move_selection(-1),
            KeyCode::Down => self.explorer.move_selection(1),
            KeyCode::Left | KeyCode::Right => self.explorer.toggle_selected_expansion(),
            KeyCode::Enter => self.open_selected_explorer_entry()?,
            KeyCode::Char('a') if !key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                self.begin_explorer_create_file()
            }
            KeyCode::Char('A') => self.begin_explorer_create_directory(),
            KeyCode::Char('r') => self.begin_explorer_rename(),
            KeyCode::Char('d') => {
                if self.explorer.selected_entry().is_some() {
                    self.mode = AppMode::ConfirmDeleteExplorerEntry;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_external_change_conflict_key(&mut self, key_event: KeyEvent) -> Result<(), AppError> {
        match key_event.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                // Discard editor changes and reload from disk
                if let Some(path) = self.pending_conflict_paths.first().cloned() {
                    if let Ok(loaded) = crate::file::load_document(&path) {
                        if let Some(buf) = self
                            .buffers
                            .iter_mut()
                            .find(|b| b.document.path().is_some_and(|p| p == path))
                        {
                            buf.document = loaded.document;
                            buf.encoding = loaded.encoding;
                            buf.saved_snapshot = buf.document.text();
                        }
                    }
                    self.pending_conflict_paths.remove(0);
                }
                if self.pending_conflict_paths.is_empty() {
                    self.mode = AppMode::Editing;
                }
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                // Keep editor changes, dismiss this conflict
                if !self.pending_conflict_paths.is_empty() {
                    self.pending_conflict_paths.remove(0);
                }
                if self.pending_conflict_paths.is_empty() {
                    self.mode = AppMode::Editing;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_confirm_delete_explorer_key(&mut self, key_event: KeyEvent) -> Result<(), AppError> {
        match key_event.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                self.mode = AppMode::Editing;
                self.delete_selected_explorer_entry()?;
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                self.mode = AppMode::Editing;
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_picker_key(&mut self, key_event: KeyEvent) -> Result<(), AppError> {
        match key_event.code {
            KeyCode::Esc => self.close_picker(),
            KeyCode::Up => {
                if let Some(picker) = self.picker.as_mut() {
                    picker.move_selection(-1);
                }
            }
            KeyCode::Down => {
                if let Some(picker) = self.picker.as_mut() {
                    picker.move_selection(1);
                }
            }
            KeyCode::Backspace => {
                if let Some(picker) = self.picker.as_mut() {
                    let mut query = picker.query().to_owned();
                    query.pop();
                    picker.set_query(query);
                }
                self.refresh_picker();
            }
            KeyCode::Enter => self.accept_picker_selection()?,
            KeyCode::Char(ch) if !key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                if let Some(picker) = self.picker.as_mut() {
                    let mut query = picker.query().to_owned();
                    query.push(ch);
                    picker.set_query(query);
                }
                self.refresh_picker();
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_confirm_quit_key(&mut self, key_event: KeyEvent) -> Result<(), AppError> {
        match key_event.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => self.should_quit = true,
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                self.mode = AppMode::Editing;
                self.pending_quit_after_save = false;
                self.set_message("Quit cancelled", crate::app::MessageKind::Info);
            }
            KeyCode::Char('s') | KeyCode::Char('S') => {
                self.pending_quit_after_save = true;
                self.save_or_prompt()?;
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_command_bar_key(
        &mut self,
        mode: CommandBarMode,
        key_event: KeyEvent,
    ) -> Result<(), AppError> {
        match key_event.code {
            KeyCode::Esc => self.cancel_command_bar(),
            KeyCode::Enter => self.submit_command_bar(mode)?,
            KeyCode::Backspace => {
                self.command_bar.input.pop();
                if matches!(mode, CommandBarMode::Search) {
                    self.update_search_preview();
                }
            }
            KeyCode::Char(ch) if !key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                self.command_bar.input.push(ch);
                if matches!(mode, CommandBarMode::Search) {
                    self.update_search_preview();
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn apply_command(&mut self, command: Command) -> Result<(), AppError> {
        match command {
            Command::MoveLeft { extend } => self.move_left(extend),
            Command::MoveRight { extend } => self.move_right(extend),
            Command::MoveUp { extend } => self.move_up(extend),
            Command::MoveDown { extend } => self.move_down(extend),
            Command::MoveLineStart { extend } => self.move_line_start(extend),
            Command::MoveLineEnd { extend } => self.move_line_end(extend),
            Command::MoveDocumentStart { extend } => self.move_document_start(extend),
            Command::MoveDocumentEnd { extend } => self.move_document_end(extend),
            Command::MoveWordLeft { extend } => self.move_word_left(extend),
            Command::MoveWordRight { extend } => self.move_word_right(extend),
            Command::PageUp { extend } => self.page_up(extend),
            Command::PageDown { extend } => self.page_down(extend),
            Command::Save => self.save_or_prompt()?,
            Command::SaveAs => self.begin_save_as_prompt(),
            Command::Quit => self.request_quit(),
            Command::Undo => self.undo(),
            Command::Redo => self.redo(),
            Command::OpenSearch => self.begin_search_prompt(),
            Command::SearchNext => self.search_next(),
            Command::SearchPrevious => self.search_previous(),
            Command::OpenCommandBar => self.begin_command_prompt(),
            Command::DeleteToEndOfLine => self.delete_to_end_of_line(),
            Command::DeleteWordBackward => self.delete_word_backward(),
            Command::CopySelection => self.copy_selection(),
            Command::CutSelection => self.cut_selection(),
            Command::Paste => self.paste(),
            Command::ToggleExplorer => self.toggle_explorer(),
            Command::OpenFilePicker => self.open_file_picker(),
            Command::OpenBufferPicker => self.open_buffer_picker(),
            Command::NextBuffer => self.next_buffer(),
            Command::PreviousBuffer => self.previous_buffer(),
            Command::SplitVertical => self.split_focused(crate::ui::SplitDirection::Horizontal),
            Command::SplitHorizontal => self.split_focused(crate::ui::SplitDirection::Vertical),
            Command::FocusNextPane => self.focus_next_pane(),
            Command::ResizePaneLeft => self.resize_focused_pane(-5),
            Command::ResizePaneRight => self.resize_focused_pane(5),
        }

        self.ensure_cursor_visible();
        Ok(())
    }
}
