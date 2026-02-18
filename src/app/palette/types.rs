use std::path::PathBuf;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PaletteKind {
    Files,
    Commands,
    GrepSearch,
    GrepReplace,
}

#[derive(Clone, Debug)]
pub(crate) struct PaletteState {
    pub(crate) kind: PaletteKind,
    pub(crate) query: String,
    pub(crate) replace_text: String,
    pub(crate) selected: usize,
}

#[derive(Clone, Debug)]
pub(crate) struct PaletteView {
    pub(crate) title: &'static str,
    pub(crate) query: String,
    pub(crate) replace_text: String,
    pub(crate) rows: Vec<String>,
    pub(crate) selected: usize,
    pub(crate) scroll: usize,
    pub(crate) total_matches: usize,
    pub(crate) show_replace: bool,
}

#[derive(Clone, Copy, Debug)]
pub(super) enum PaletteCommand {
    ReloadLsp,
}

#[derive(Clone, Debug)]
pub(super) struct GrepMatch {
    pub(super) path: PathBuf,
    pub(super) line_number: usize,
    pub(super) line_text: String,
    pub(super) col_start: usize,
}

#[derive(Clone, Debug)]
pub(super) enum PaletteAction {
    OpenFile(PathBuf),
    OpenFileAt(PathBuf, usize, usize),
    Command(PaletteCommand),
}

#[derive(Clone, Debug)]
pub(super) struct PaletteMatch {
    pub(super) label: String,
    pub(super) score: i64,
    pub(super) action: PaletteAction,
}
