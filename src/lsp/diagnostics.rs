use std::collections::HashMap;
use std::path::{Path, PathBuf};

use lsp_types::DiagnosticSeverity;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticSeverityView {
    Error,
    Warning,
    Information,
    Hint,
}

impl DiagnosticStore {
    pub fn apply_publish(
        &mut self,
        path: PathBuf,
        diagnostics: impl IntoIterator<Item = lsp_types::Diagnostic>,
    ) {
        let mapped = diagnostics
            .into_iter()
            .map(|d| DiagnosticItem {
                line: d.range.start.line as usize,
                column: d.range.start.character as usize,
                severity: map_severity(d.severity),
                message: d.message,
            })
            .collect::<Vec<_>>();
        self.set(path, mapped);
    }
}

#[derive(Debug, Clone)]
pub struct DiagnosticItem {
    pub line: usize,
    pub column: usize,
    pub severity: DiagnosticSeverityView,
    pub message: String,
}

#[derive(Debug, Default)]
pub struct DiagnosticStore {
    by_path: HashMap<PathBuf, Vec<DiagnosticItem>>,
}

impl DiagnosticStore {
    pub fn set(&mut self, path: PathBuf, diagnostics: Vec<DiagnosticItem>) {
        self.by_path.insert(path, diagnostics);
    }

    pub fn for_path(&self, path: &Path) -> &[DiagnosticItem] {
        self.by_path.get(path).map(Vec::as_slice).unwrap_or(&[])
    }

    pub fn all(&self) -> impl Iterator<Item = (&PathBuf, &Vec<DiagnosticItem>)> {
        self.by_path.iter()
    }

    pub fn clear_path(&mut self, path: &Path) {
        self.by_path.remove(path);
    }
}

pub fn map_severity(severity: Option<DiagnosticSeverity>) -> DiagnosticSeverityView {
    match severity {
        Some(DiagnosticSeverity::ERROR) => DiagnosticSeverityView::Error,
        Some(DiagnosticSeverity::WARNING) => DiagnosticSeverityView::Warning,
        Some(DiagnosticSeverity::INFORMATION) => DiagnosticSeverityView::Information,
        _ => DiagnosticSeverityView::Hint,
    }
}
