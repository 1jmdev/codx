use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThemeColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl ThemeColor {
    fn from_hex(s: &str) -> Option<Self> {
        let s = s.trim_start_matches('#');
        if s.len() != 6 {
            return None;
        }
        let r = u8::from_str_radix(&s[0..2], 16).ok()?;
        let g = u8::from_str_radix(&s[2..4], 16).ok()?;
        let b = u8::from_str_radix(&s[4..6], 16).ok()?;
        Some(Self { r, g, b })
    }
}

#[derive(Debug, Clone, Default)]
pub struct ThemeStyle {
    pub fg: Option<ThemeColor>,
    pub bg: Option<ThemeColor>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
}

#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub background: ThemeColor,
    pub foreground: ThemeColor,
    highlights: HashMap<String, ThemeStyle>,
}

impl Theme {
    pub fn for_capture(&self, capture: &str) -> Option<&ThemeStyle> {
        if let Some(style) = self.highlights.get(capture) {
            return Some(style);
        }

        let fallback = match capture {
            "comment.documentation" => ["comment.doc", "comment"].as_slice(),
            "string.special" | "string.special.key" => ["string"].as_slice(),
            "constructor" => ["type", "constant"].as_slice(),
            "function.call" | "function.special" => ["function"].as_slice(),
            "method" | "method.call" => ["function.method", "function"].as_slice(),
            "field" => ["property"].as_slice(),
            "parameter" => ["variable.parameter", "variable"].as_slice(),
            "embedded" => ["special"].as_slice(),
            "repeat" | "conditional" => ["keyword.control", "keyword"].as_slice(),
            "preproc" => ["keyword.storage", "keyword"].as_slice(),
            "tag" => ["type", "keyword"].as_slice(),
            "tag.error" => ["error", "tag"].as_slice(),
            "delimiter" => ["punctuation.delimiter", "punctuation"].as_slice(),
            "text.title" => ["special", "keyword"].as_slice(),
            "text.literal" => ["string"].as_slice(),
            "text.uri" | "text.reference" => ["special", "string"].as_slice(),
            "text.emphasis" | "text.strong" => ["special", "keyword"].as_slice(),
            _ => &[],
        };

        for key in fallback {
            if let Some(style) = self.highlights.get(*key) {
                return Some(style);
            }
        }

        if let Some((prefix, _)) = capture.rsplit_once('.') {
            return self.highlights.get(prefix);
        }

        None
    }

    pub fn load_embedded(name: &str) -> Option<Self> {
        let source = match name {
            "catppuccin" => include_str!("../../assets/themes/catppuccin.toml"),
            "gruvbox" => include_str!("../../assets/themes/gruvbox.toml"),
            "dracula" => include_str!("../../assets/themes/dracula.toml"),
            "onedark" => include_str!("../../assets/themes/onedark.toml"),
            "solarized" => include_str!("../../assets/themes/solarized.toml"),
            "tokyonight" => include_str!("../../assets/themes/tokyonight.toml"),
            _ => return None,
        };
        parse_theme_toml(source)
    }

    pub fn available_names() -> &'static [&'static str] {
        &[
            "catppuccin",
            "gruvbox",
            "dracula",
            "onedark",
            "solarized",
            "tokyonight",
        ]
    }

    pub fn default_theme() -> Self {
        Self::load_embedded("catppuccin").unwrap_or_else(empty_theme)
    }
}

fn empty_theme() -> Theme {
    Theme {
        name: String::from("default"),
        background: ThemeColor {
            r: 30,
            g: 30,
            b: 46,
        },
        foreground: ThemeColor {
            r: 205,
            g: 214,
            b: 244,
        },
        highlights: HashMap::new(),
    }
}

#[derive(Deserialize)]
struct TomlTheme {
    name: String,
    palette: TomlPalette,
    #[serde(default)]
    highlights: HashMap<String, TomlStyle>,
}

#[derive(Deserialize)]
struct TomlPalette {
    background: String,
    foreground: String,
}

#[derive(Deserialize, Default)]
struct TomlStyle {
    fg: Option<String>,
    bg: Option<String>,
    #[serde(default)]
    bold: bool,
    #[serde(default)]
    italic: bool,
    #[serde(default)]
    underline: bool,
}

fn parse_theme_toml(source: &str) -> Option<Theme> {
    let toml: TomlTheme = toml::from_str(source).ok()?;

    let background = ThemeColor::from_hex(&toml.palette.background).unwrap_or(ThemeColor {
        r: 30,
        g: 30,
        b: 46,
    });
    let foreground = ThemeColor::from_hex(&toml.palette.foreground).unwrap_or(ThemeColor {
        r: 205,
        g: 214,
        b: 244,
    });

    let mut highlights = HashMap::new();
    for (key, style) in toml.highlights {
        let ts = ThemeStyle {
            fg: style.fg.as_deref().and_then(ThemeColor::from_hex),
            bg: style.bg.as_deref().and_then(ThemeColor::from_hex),
            bold: style.bold,
            italic: style.italic,
            underline: style.underline,
        };
        highlights.insert(key, ts);
    }

    Some(Theme {
        name: toml.name,
        background,
        foreground,
        highlights,
    })
}
