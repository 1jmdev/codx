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
            .syntax_for_path(path)
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

    fn syntax_for_path(&self, path: Option<&Path>) -> Option<&SyntaxReference> {
        path.and_then(|value| self.syntax_set.find_syntax_for_file(value).ok().flatten())
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
