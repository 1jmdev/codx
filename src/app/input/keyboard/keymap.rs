use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::Focus;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum GlobalCommand {
    ShowCommandPalette,
    ShowFilePalette,
    ShowSearch,
    ShowSearchReplace,
    ShowGrepSearch,
    ShowGrepReplace,
    TriggerSuggest,
    ToggleSidebar,
    Save,
    Quit,
    Undo,
    Redo,
    DuplicateLine,
    DeleteLine,
    ActivateTreeItem,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Scope {
    Any,
    Editor,
    FileTree,
}

#[derive(Clone, Copy, Debug)]
struct Binding {
    command: GlobalCommand,
    keys: &'static [&'static str],
    scope: Scope,
}

const DEFAULT_GLOBAL_KEYBINDINGS: &[Binding] = &[
    Binding {
        command: GlobalCommand::ShowCommandPalette,
        keys: &["f1", "ctrl+shift+p"],
        scope: Scope::Any,
    },
    Binding {
        command: GlobalCommand::ShowFilePalette,
        keys: &["ctrl+p"],
        scope: Scope::Any,
    },
    Binding {
        command: GlobalCommand::ShowSearch,
        keys: &["ctrl+f"],
        scope: Scope::Any,
    },
    Binding {
        command: GlobalCommand::ShowSearchReplace,
        keys: &["ctrl+h"],
        scope: Scope::Any,
    },
    Binding {
        command: GlobalCommand::ShowGrepSearch,
        keys: &["ctrl+shift+f"],
        scope: Scope::Any,
    },
    Binding {
        command: GlobalCommand::ShowGrepReplace,
        keys: &["ctrl+shift+h"],
        scope: Scope::Any,
    },
    Binding {
        command: GlobalCommand::TriggerSuggest,
        keys: &["ctrl+space"],
        scope: Scope::Editor,
    },
    Binding {
        command: GlobalCommand::ToggleSidebar,
        keys: &["ctrl+b"],
        scope: Scope::Any,
    },
    Binding {
        command: GlobalCommand::Save,
        keys: &["ctrl+s"],
        scope: Scope::Any,
    },
    Binding {
        command: GlobalCommand::Quit,
        keys: &["ctrl+q"],
        scope: Scope::Any,
    },
    Binding {
        command: GlobalCommand::Undo,
        keys: &["ctrl+z"],
        scope: Scope::Editor,
    },
    Binding {
        command: GlobalCommand::Redo,
        keys: &["ctrl+y"],
        scope: Scope::Editor,
    },
    Binding {
        command: GlobalCommand::DuplicateLine,
        keys: &["ctrl+d"],
        scope: Scope::Editor,
    },
    Binding {
        command: GlobalCommand::DeleteLine,
        keys: &["ctrl+shift+k"],
        scope: Scope::Editor,
    },
    Binding {
        command: GlobalCommand::ActivateTreeItem,
        keys: &["ctrl+shift+e"],
        scope: Scope::FileTree,
    },
];

pub(super) fn resolve_global_command(key: KeyEvent, focus: Focus) -> Option<GlobalCommand> {
    DEFAULT_GLOBAL_KEYBINDINGS
        .iter()
        .find(|binding| scope_matches(binding.scope, focus) && binding_matches(binding, key))
        .map(|binding| binding.command)
}

fn scope_matches(scope: Scope, focus: Focus) -> bool {
    match scope {
        Scope::Any => true,
        Scope::Editor => focus == Focus::Editor,
        Scope::FileTree => focus == Focus::FileTree,
    }
}

fn binding_matches(binding: &Binding, key: KeyEvent) -> bool {
    binding
        .keys
        .iter()
        .filter_map(|spec| parse_keybinding(spec))
        .any(|shortcut| {
            key_code_matches(shortcut.code, key.code) && shortcut.modifiers == key.modifiers
        })
}

fn key_code_matches(left: KeyCode, right: KeyCode) -> bool {
    match (left, right) {
        (KeyCode::Char(a), KeyCode::Char(b)) => a.eq_ignore_ascii_case(&b),
        (KeyCode::Char(' '), KeyCode::Null) => true,
        (KeyCode::Null, KeyCode::Char(' ')) => true,
        _ => left == right,
    }
}

#[derive(Clone, Copy)]
struct Shortcut {
    code: KeyCode,
    modifiers: KeyModifiers,
}

fn parse_keybinding(spec: &str) -> Option<Shortcut> {
    let mut modifiers = KeyModifiers::empty();
    let mut code = None;

    for part in spec.split('+') {
        let token = part.trim().to_ascii_lowercase();
        match token.as_str() {
            "ctrl" | "control" => modifiers |= KeyModifiers::CONTROL,
            "shift" => modifiers |= KeyModifiers::SHIFT,
            "alt" => modifiers |= KeyModifiers::ALT,
            "space" => code = Some(KeyCode::Char(' ')),
            _ => {
                if let Some(function) = token.strip_prefix('f')
                    && let Ok(value) = function.parse::<u8>()
                {
                    code = Some(KeyCode::F(value));
                    continue;
                }

                if token.chars().count() == 1 {
                    code = token.chars().next().map(KeyCode::Char);
                    continue;
                }

                return None;
            }
        }
    }

    code.map(|code| Shortcut { code, modifiers })
}
