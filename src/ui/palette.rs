use ratatui::style::Color::Reset;
use ratatui::style::{Color, Modifier, Style};

#[derive(Debug, Clone, Copy)]
pub struct Palette {
    pub mantle: Color,
    pub surface: Color,
    pub overlay: Color,
    pub text: Color,
    pub subtle: Color,
    pub lavender: Color,
    pub blue: Color,
    pub yellow: Color,
    pub red: Color,
}

#[derive(Debug, Clone, Copy)]
pub struct PaletteStyles {
    pub editor: Style,
    pub gutter: Style,
    pub gutter_current: Style,
    pub selection: Style,
    pub search_match: Style,
    pub active_search_match: Style,
    pub diagnostic_error: Style,
    pub diagnostic_warning: Style,
    pub diagnostic_information: Style,
    pub diagnostic_hint: Style,
    pub diagnostic_lens_error: Style,
    pub diagnostic_lens_warning: Style,
    pub diagnostic_lens_information: Style,
    pub diagnostic_lens_hint: Style,
    pub statusline: Style,
    pub message: Style,
    pub warning: Style,
    pub error: Style,
    pub command_bar: Style,
    pub tilde: Style,
    pub explorer_border: Style,
    pub explorer_border_focused: Style,
    pub explorer_dir: Style,
    pub explorer_dir_selected: Style,
    pub explorer_file_selected: Style,
}

impl Palette {
    pub fn mocha() -> Self {
        Self {
            mantle: Color::Rgb(17, 17, 27),
            surface: Color::Rgb(49, 50, 68),
            overlay: Color::Rgb(108, 112, 134),
            text: Color::Rgb(205, 214, 244),
            subtle: Color::Rgb(166, 173, 200),
            lavender: Color::Rgb(180, 190, 254),
            blue: Color::Rgb(137, 180, 250),
            yellow: Color::Rgb(249, 226, 175),
            red: Color::Rgb(243, 139, 168),
        }
    }

    pub fn styles(self) -> PaletteStyles {
        PaletteStyles {
            editor: Style::default().bg(Reset).fg(self.text),
            gutter: Style::default().bg(Reset).fg(self.overlay),
            gutter_current: Style::default().bg(Reset).fg(self.lavender),
            selection: Style::default().bg(Color::Rgb(69, 71, 90)).fg(self.text),
            search_match: Style::default().bg(Color::Rgb(88, 74, 102)).fg(self.text),
            active_search_match: Style::default().bg(self.blue).fg(self.mantle),
            diagnostic_error: Style::default().bg(Reset).fg(self.red),
            diagnostic_warning: Style::default().bg(Reset).fg(self.yellow),
            diagnostic_information: Style::default().bg(Reset).fg(self.blue),
            diagnostic_hint: Style::default().bg(Reset).fg(self.subtle),
            diagnostic_lens_error: Style::default().bg(Reset).fg(Color::Rgb(240, 146, 170)),
            diagnostic_lens_warning: Style::default().bg(Reset).fg(Color::Rgb(230, 192, 118)),
            diagnostic_lens_information: Style::default().bg(Reset).fg(Color::Rgb(156, 189, 239)),
            diagnostic_lens_hint: Style::default().bg(Reset).fg(Color::Rgb(166, 173, 200)),
            statusline: Style::default()
                .bg(self.surface)
                .fg(self.text)
                .add_modifier(Modifier::BOLD),
            message: Style::default().bg(self.mantle).fg(self.subtle),
            warning: Style::default().bg(self.mantle).fg(self.yellow),
            error: Style::default().bg(self.mantle).fg(self.red),
            command_bar: Style::default().bg(self.mantle).fg(self.text),
            tilde: Style::default().bg(Reset).fg(self.surface),
            // LazyVim / Tokyo Night inspired explorer styles
            explorer_border: Style::default().fg(Color::Rgb(59, 66, 97)),
            explorer_border_focused: Style::default().fg(Color::Rgb(122, 162, 247)),
            explorer_dir: Style::default().fg(Color::Rgb(122, 162, 247)),
            explorer_dir_selected: Style::default()
                .bg(Color::Rgb(40, 52, 87))
                .fg(Color::Rgb(122, 162, 247))
                .add_modifier(Modifier::BOLD),
            explorer_file_selected: Style::default()
                .bg(Color::Rgb(40, 52, 87))
                .fg(Color::Rgb(192, 202, 245)),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ui::Palette;

    #[test]
    fn palette_builds_styles() {
        let styles = Palette::mocha().styles();
        assert_ne!(styles.editor, styles.statusline);
    }
}
