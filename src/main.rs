use std::{fs, io, io::ErrorKind, path::PathBuf};

mod app;

use app::App;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

fn main() -> io::Result<()> {
    let (cwd, file_to_open) = launch_target()?;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let mut app = App::new(cwd);
    if let Some(path) = file_to_open {
        app.sidebar_open = false;
        app.open_file(path)?;
    }
    let result = app.run(&mut terminal);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

fn launch_target() -> io::Result<(PathBuf, Option<PathBuf>)> {
    let mut args = std::env::args_os();
    let _ = args.next();

    let cwd = std::env::current_dir()?;
    let Some(arg) = args.next() else {
        return Ok((cwd, None));
    };

    if args.next().is_some() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "expected at most one path argument",
        ));
    }

    let path = PathBuf::from(arg);
    let resolved = if path.is_absolute() {
        path
    } else {
        cwd.join(path)
    };

    match fs::metadata(&resolved) {
        Ok(metadata) if metadata.is_dir() => Ok((resolved, None)),
        Ok(metadata) if metadata.is_file() => {
            let project_dir = resolved.parent().map(PathBuf::from).ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "file has no parent")
            })?;
            Ok((project_dir, Some(resolved)))
        }
        Ok(_) => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "path must be a file or directory",
        )),
        Err(error) if error.kind() == ErrorKind::NotFound => {
            let project_dir = resolved.parent().map(PathBuf::from).ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "file has no parent")
            })?;
            if project_dir.is_dir() {
                Ok((project_dir, Some(resolved)))
            } else {
                Err(io::Error::new(
                    ErrorKind::NotFound,
                    format!("parent directory does not exist: {}", project_dir.display()),
                ))
            }
        }
        Err(error) => Err(error),
    }
}
