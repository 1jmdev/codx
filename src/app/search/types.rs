/// Which input field has focus inside the search/replace panel.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SearchField {
    Search,
    Replace,
}

#[derive(Clone, Debug)]
pub(crate) struct SearchReplaceState {
    pub(crate) query: String,
    pub(crate) replacement: String,
    pub(crate) focused_field: SearchField,
    /// All match positions: (line, col_start, col_end) in char offsets.
    pub(crate) matches: Vec<(usize, usize, usize)>,
    /// Index of the currently highlighted match.
    pub(crate) current_match: usize,
    /// `false` = search only (Ctrl+F).  `true` = search + replace (Ctrl+H).
    pub(crate) show_replace: bool,
}
