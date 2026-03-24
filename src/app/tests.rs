#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    use ratatui::layout::Size;

    use crate::app::{App, AppMode};

    #[test]
    fn dirty_document_requires_confirmation() {
        let mut app = App::open(None).unwrap_or_else(|error| panic!("{error}"));
        app.set_terminal_size(Size {
            width: 120,
            height: 30,
        });
        app.handle_key_event(KeyEvent::from(KeyCode::Char('a')))
            .unwrap_or_else(|error| panic!("{error}"));
        app.handle_key_event(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CONTROL))
            .unwrap_or_else(|error| panic!("{error}"));
        assert!(matches!(app.mode(), AppMode::ConfirmQuit));
    }

    #[test]
    fn open_new_path_creates_unsaved_document() {
        let app =
            App::open(Some(PathBuf::from("new.txt"))).unwrap_or_else(|error| panic!("{error}"));
        assert_eq!(
            app.active_document()
                .path()
                .map(|path: &std::path::Path| path.to_string_lossy().into_owned()),
            Some(String::from("new.txt"))
        );
        assert!(!app.active_document().is_dirty());
    }

    #[test]
    fn command_mode_opens_from_colon() {
        let mut app = App::open(None).unwrap_or_else(|error| panic!("{error}"));
        app.handle_key_event(KeyEvent::from(KeyCode::Char(':')))
            .unwrap_or_else(|error| panic!("{error}"));
        assert!(matches!(app.mode(), AppMode::CommandBar(_)));
    }
}
