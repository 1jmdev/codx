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

impl DiagnosticSeverityView {
    pub fn rank(self) -> u8 {
        match self {
            Self::Error => 4,
            Self::Warning => 3,
            Self::Information => 2,
            Self::Hint => 1,
        }
    }
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

    pub fn counts_for_path(&self, path: &Path) -> DiagnosticCounts {
        let mut counts = DiagnosticCounts::default();
        if let Some(items) = self.by_path.get(path) {
            for diagnostic in items {
                match diagnostic.severity {
                    DiagnosticSeverityView::Error => counts.errors += 1,
                    DiagnosticSeverityView::Warning => counts.warnings += 1,
                    DiagnosticSeverityView::Information => counts.information += 1,
                    DiagnosticSeverityView::Hint => counts.hints += 1,
                }
            }
        }
        counts
    }

    pub fn clear_path(&mut self, path: &Path) {
        self.by_path.remove(path);
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct DiagnosticCounts {
    pub errors: usize,
    pub warnings: usize,
    pub information: usize,
    pub hints: usize,
}

pub fn map_severity(severity: Option<DiagnosticSeverity>) -> DiagnosticSeverityView {
    match severity {
        Some(DiagnosticSeverity::ERROR) => DiagnosticSeverityView::Error,
        Some(DiagnosticSeverity::WARNING) => DiagnosticSeverityView::Warning,
        Some(DiagnosticSeverity::INFORMATION) => DiagnosticSeverityView::Information,
        _ => DiagnosticSeverityView::Hint,
    }
}
