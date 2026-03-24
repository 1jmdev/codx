use std::io::{self, Stdout};

use crossterm::ExecutableCommand;
use crossterm::cursor::Show;
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::execute;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

use crate::app::AppError;

pub(crate) struct TerminalSession {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl TerminalSession {
    pub(crate) fn enter() -> Result<Self, AppError> {
        let terminal = ratatui::try_init()?;
        let mut session = Self { terminal };
        session
            .terminal
            .backend_mut()
            .execute(crossterm::terminal::SetTitle("codx"))?;
        session.terminal.backend_mut().execute(EnableMouseCapture)?;
        session.terminal.clear()?;
        Ok(session)
    }

    pub(crate) fn terminal_mut(&mut self) -> &mut Terminal<CrosstermBackend<Stdout>> {
        &mut self.terminal
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        let _ = ratatui::restore();
        let _ = execute!(io::stdout(), DisableMouseCapture, Show);
    }
}
