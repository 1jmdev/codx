use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::editor::Command;

pub fn map_key_event(key_event: KeyEvent) -> Option<Command> {
    let modifiers = key_event.modifiers;
    let extend = modifiers.contains(KeyModifiers::SHIFT);
    let control = modifiers.contains(KeyModifiers::CONTROL);

    match key_event.code {
        KeyCode::PageUp if control => Some(Command::PreviousBuffer),
        KeyCode::PageDown if control => Some(Command::NextBuffer),
        KeyCode::Left if modifiers.contains(KeyModifiers::ALT) => Some(Command::ResizePaneLeft),
        KeyCode::Right if modifiers.contains(KeyModifiers::ALT) => Some(Command::ResizePaneRight),
        KeyCode::Home if control => Some(Command::MoveDocumentStart { extend: false }),
        KeyCode::End if control => Some(Command::MoveDocumentEnd { extend: false }),
        KeyCode::F(3) if extend => Some(Command::SearchPrevious),
        KeyCode::F(3) => Some(Command::SearchNext),
        KeyCode::Left if control => Some(Command::MoveWordLeft { extend }),
        KeyCode::Right if control => Some(Command::MoveWordRight { extend }),
        KeyCode::Left => Some(Command::MoveLeft { extend }),
        KeyCode::Right => Some(Command::MoveRight { extend }),
        KeyCode::Up => Some(Command::MoveUp { extend }),
        KeyCode::Down => Some(Command::MoveDown { extend }),
        KeyCode::Home => Some(Command::MoveLineStart { extend }),
        KeyCode::End => Some(Command::MoveLineEnd { extend }),
        KeyCode::PageUp => Some(Command::PageUp { extend }),
        KeyCode::PageDown => Some(Command::PageDown { extend }),
        KeyCode::Char('a') if control => Some(Command::MoveLineStart { extend: false }),
        KeyCode::Char('e') if control => Some(Command::MoveLineEnd { extend: false }),
        KeyCode::Char('k') if control => Some(Command::DeleteToEndOfLine),
        KeyCode::Backspace if control => Some(Command::DeleteWordBackward),
        KeyCode::Char('q') if control => Some(Command::Quit),
        KeyCode::Char('z') if control => Some(Command::Undo),
        KeyCode::Char('y') if control => Some(Command::Redo),
        KeyCode::Char('r') if control => Some(Command::ReloadChangedFiles),
        KeyCode::Char('f') if control => Some(Command::OpenSearch),
        KeyCode::Char('b') if control => Some(Command::ToggleExplorer),
        KeyCode::Char('p') if control => Some(Command::OpenFilePicker),
        KeyCode::Tab if control => Some(Command::OpenBufferPicker),
        KeyCode::Char('\\') if control && modifiers.contains(KeyModifiers::SHIFT) => {
            Some(Command::SplitHorizontal)
        }
        KeyCode::Char('\\') if control => Some(Command::SplitVertical),
        KeyCode::Char('w') if control => Some(Command::FocusNextPane),
        KeyCode::Char('g') if control => Some(Command::SearchPrevious),
        KeyCode::Char('S') if control => Some(Command::SaveAs),
        KeyCode::Char('s') if control && modifiers.contains(KeyModifiers::SHIFT) => Some(Command::SaveAs),
        KeyCode::Char('s') if control => Some(Command::Save),
        KeyCode::Char('c') if control => Some(Command::CopySelection),
        KeyCode::Char('x') if control => Some(Command::CutSelection),
        KeyCode::Char('v') if control => Some(Command::Paste),
        KeyCode::Char(':') if !control => Some(Command::OpenCommandBar),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    use crate::editor::Command;
    use crate::keymap::map_key_event;

    #[test]
    fn maps_supported_bindings() {
        let save = map_key_event(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL));
        assert_eq!(save, Some(Command::Save));

        let move_left = map_key_event(KeyEvent::new(KeyCode::Left, KeyModifiers::SHIFT));
        assert_eq!(move_left, Some(Command::MoveLeft { extend: true }));

        let undo = map_key_event(KeyEvent::new(KeyCode::Char('z'), KeyModifiers::CONTROL));
        assert_eq!(undo, Some(Command::Undo));
    }
}
