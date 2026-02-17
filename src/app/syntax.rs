use std::path::Path;

use ratatui::{
    style::{Color, Modifier, Style},
    text::Span,
};
use syntect::{
    easy::HighlightLines,
    highlighting::{FontStyle, Style as SynStyle, Theme, ThemeSet},
    parsing::{SyntaxReference, SyntaxSet},
};

pub(crate) struct SyntaxEngine {
    syntax_set: SyntaxSet,
    theme: Theme,
}

impl SyntaxEngine {
    pub(crate) fn new() -> Self {
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let themes = ThemeSet::load_defaults();
        let theme = themes
            .themes
            .get("base16-eighties.dark")
            .cloned()
            .or_else(|| themes.themes.values().next().cloned())
            .unwrap_or_default();

        Self { syntax_set, theme }
    }

    pub(crate) fn highlight_visible(
        &self,
        path: Option<&Path>,
        lines: &[String],
        start: usize,
        end: usize,
    ) -> Vec<Vec<Span<'static>>> {
        if lines.is_empty() || start >= end {
            return Vec::new();
        }

        let syntax = self
            .syntax_for_path(path, lines)
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());
        let mut highlighter = HighlightLines::new(syntax, &self.theme);
        let mut out = Vec::new();

        for (idx, line) in lines.iter().enumerate() {
            let ranges = highlighter
                .highlight_line(line, &self.syntax_set)
                .unwrap_or_default();
            if idx >= start && idx < end {
                if ranges.is_empty() {
                    out.push(vec![Span::raw(line.clone())]);
                } else {
                    out.push(
                        ranges
                            .into_iter()
                            .map(|(style, text)| Span::styled(text.to_string(), syn_to_tui(style)))
                            .collect(),
                    );
                }
            }
            if idx >= end {
                break;
            }
        }

        out
    }

    fn syntax_for_path(&self, path: Option<&Path>, lines: &[String]) -> Option<&SyntaxReference> {
        let by_path =
            path.and_then(|value| self.syntax_set.find_syntax_for_file(value).ok().flatten());
        if by_path.is_some() {
            return by_path;
        }

        if let Some(value) = path {
            if let Some(by_name) = self.syntax_for_filename(value) {
                return Some(by_name);
            }
        }

        lines
            .first()
            .and_then(|line| self.syntax_set.find_syntax_by_first_line(line))
    }

    fn syntax_for_filename(&self, path: &Path) -> Option<&SyntaxReference> {
        let name = path.file_name()?.to_str()?;
        let ext = path.extension().and_then(|value| value.to_str());

        if let Some(ext) = ext {
            if let Some(syntax) = self.syntax_set.find_syntax_by_extension(ext) {
                return Some(syntax);
            }
        }

        let mapped = match name {
            "Dockerfile" | "Containerfile" => Some("Dockerfile"),
            "Makefile" | "makefile" | "GNUmakefile" => Some("Makefile"),
            ".gitignore" | ".ignore" | ".dockerignore" | ".editorconfig" | ".gitattributes"
            | ".npmrc" | ".yarnrc" | ".env" | ".env.local" | ".env.development"
            | ".env.production" | ".env.test" => Some("INI"),
            "Cargo" | "Cargo.toml" | "Pipfile" | "pyproject.toml" | "poetry.lock" | "bun.lock"
            | "bunfig.toml" => Some("TOML"),
            "package.json" | "tsconfig.json" | "composer.json" | "deno.json" | "deno.jsonc" => {
                Some("JSON")
            }
            "README" | "README.md" | "CHANGELOG.md" | "CONTRIBUTING.md" => Some("Markdown"),
            _ => match ext.unwrap_or_default().to_ascii_lowercase().as_str() {
                "rs" => Some("Rust"),
                "js" | "mjs" | "cjs" => Some("JavaScript"),
                "jsx" => Some("JavaScript (JSX)"),
                "ts" | "mts" | "cts" => Some("TypeScript"),
                "tsx" => Some("TypeScriptReact"),
                "py" | "pyi" => Some("Python"),
                "go" => Some("Go"),
                "java" => Some("Java"),
                "kt" | "kts" => Some("Kotlin"),
                "swift" => Some("Swift"),
                "dart" => Some("Dart"),
                "php" => Some("PHP"),
                "rb" => Some("Ruby"),
                "lua" => Some("Lua"),
                "r" => Some("R"),
                "c" | "h" => Some("C"),
                "cc" | "cpp" | "cxx" | "hpp" | "hh" | "hxx" => Some("C++"),
                "cs" => Some("C#"),
                "sh" | "bash" => Some("Bourne Again Shell (bash)"),
                "zsh" => Some("Zsh"),
                "fish" => Some("Fish"),
                "ps1" => Some("PowerShell"),
                "json" | "jsonc" => Some("JSON"),
                "toml" => Some("TOML"),
                "yaml" | "yml" => Some("YAML"),
                "xml" | "xsd" | "xsl" | "svg" => Some("XML"),
                "html" | "htm" => Some("HTML"),
                "css" => Some("CSS"),
                "scss" => Some("SCSS"),
                "sass" => Some("Sass"),
                "sql" => Some("SQL"),
                "md" | "markdown" => Some("Markdown"),
                "ini" | "cfg" | "conf" | "properties" => Some("INI"),
                "csv" | "tsv" => Some("CSV"),
                _ => None,
            },
        };

        mapped.and_then(|name| self.syntax_set.find_syntax_by_name(name))
    }
}

fn syn_to_tui(style: SynStyle) -> Style {
    let (r, g, b) = boost(style.foreground.r, style.foreground.g, style.foreground.b);
    let mut out = Style::default().fg(Color::Rgb(r, g, b));
    if style.font_style.contains(FontStyle::BOLD) {
        out = out.add_modifier(Modifier::BOLD);
    }
    out
}

fn boost(r: u8, g: u8, b: u8) -> (u8, u8, u8) {
    (scale(r), scale(g), scale(b))
}

fn scale(v: u8) -> u8 {
    let boosted = ((v as u16 * 130) / 100) + 12;
    boosted.min(255) as u8
}
