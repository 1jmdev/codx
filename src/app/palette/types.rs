use std::path::PathBuf;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PaletteKind {
    Files,
    Commands,
}

#[derive(Clone, Debug)]
pub(crate) struct PaletteState {
    pub(crate) kind: PaletteKind,
    pub(crate) query: String,
    pub(crate) selected: usize,
}

#[derive(Clone, Debug)]
pub(crate) struct PaletteView {
    pub(crate) title: &'static str,
    pub(crate) query: String,
    pub(crate) rows: Vec<String>,
    pub(crate) selected: usize,
}

#[derive(Clone, Copy, Debug)]
pub(super) enum PaletteCommand {
    ReloadLsp,
}

#[derive(Clone, Debug)]
pub(super) enum PaletteAction {
    OpenFile(PathBuf),
    Command(PaletteCommand),
}

#[derive(Clone, Debug)]
pub(super) struct PaletteMatch {
    pub(super) label: String,
    pub(super) score: i64,
    pub(super) action: PaletteAction,
}
