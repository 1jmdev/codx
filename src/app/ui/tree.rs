use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
};

use crate::app::{App, Focus};

impl App {
    pub(crate) fn draw_tree(&mut self, frame: &mut ratatui::Frame, area: Rect) {
        let border_style = if self.focus == Focus::FileTree {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::Gray)
        };

        let block = Block::default()
            .title(" Files ")
            .borders(Borders::ALL)
            .border_style(border_style);

        let inner = block.inner(area);
        self.ui.tree_inner = inner;
        frame.render_widget(block, area);
        self.ensure_tree_selection_visible(inner.height as usize);

        let mut rendered = Vec::new();
        for row in 0..inner.height as usize {
            let idx = self.tree_scroll + row;
            if idx >= self.tree_items.len() {
                break;
            }

            let item = &self.tree_items[idx];
            let indent = "  ".repeat(item.depth);
            let icon = tree_item_icon(
                &item.name,
                item.is_dir,
                self.expanded_dirs.contains(&item.path),
            );

            let name = if item.is_dir {
                format!("{}/", item.name)
            } else {
                item.name.clone()
            };
            let row_style = if idx == self.tree_selected {
                Style::default().bg(Color::Rgb(30, 50, 70))
            } else {
                Style::default()
            };

            let icon_style = if item.is_dir {
                row_style
            } else {
                row_style.add_modifier(Modifier::DIM)
            };

            rendered.push(Line::from(vec![
                Span::styled(indent, row_style),
                Span::styled(icon, icon_style),
                Span::styled(" ", row_style),
                Span::styled(name, row_style),
            ]));
        }

        frame.render_widget(Paragraph::new(Text::from(rendered)), inner);
    }
}

fn tree_item_icon(name: &str, is_dir: bool, is_expanded: bool) -> &'static str {
    if is_dir {
        if is_expanded { " " } else { " " }
    } else {
        file_icon(name)
    }
}

fn file_icon(name: &str) -> &'static str {
    let lower_name = name.to_ascii_lowercase();

    match lower_name.as_str() {
        "cargo.toml" => return "",
        "cargo.lock" => return "",
        "package.json" => return "",
        "package-lock.json" | "pnpm-lock.yaml" | "yarn.lock" => return "",
        "tsconfig.json" => return "",
        "dockerfile" | "docker-compose.yml" | "docker-compose.yaml" => return "",
        "makefile" => return "",
        ".gitignore" | ".gitattributes" | ".gitmodules" => return "",
        ".env" | ".env.local" | ".env.development" | ".env.production" => return "",
        _ => {}
    }

    if ["readme", "license", "changelog"]
        .iter()
        .any(|keyword| lower_name.starts_with(keyword))
    {
        return "";
    }

    let extension = lower_name.rsplit('.').next();
    if extension == Some(lower_name.as_str()) {
        return "";
    }

    match extension {
        Some("rs") => "",
        Some("c") | Some("h") => "",
        Some("cpp") | Some("cc") | Some("cxx") | Some("hpp") | Some("hh") | Some("hxx") => "",
        Some("go") => "",
        Some("java") => "",
        Some("kt") | Some("kts") => "",
        Some("swift") => "",
        Some("cs") => "󰌛",
        Some("js") | Some("mjs") | Some("cjs") => "",
        Some("ts") => "",
        Some("jsx") => "",
        Some("tsx") => "",
        Some("py") => "",
        Some("rb") => "",
        Some("php") => "",
        Some("lua") => "",
        Some("sh") | Some("bash") | Some("zsh") | Some("fish") => "",
        Some("html") | Some("htm") => "",
        Some("css") => "",
        Some("scss") | Some("sass") => "",
        Some("less") => "",
        Some("json") => "",
        Some("yaml") | Some("yml") => "",
        Some("toml") => "",
        Some("ini") | Some("conf") | Some("cfg") | Some("lock") | Some("xml") => "",
        Some("sql") => "",
        Some("md") | Some("markdown") => "",
        Some("txt") | Some("rtf") => "",
        Some("png") | Some("jpg") | Some("jpeg") | Some("gif") | Some("webp") | Some("bmp")
        | Some("svg") | Some("ico") => "󰈟",
        Some("zip") | Some("tar") | Some("gz") | Some("tgz") | Some("xz") | Some("7z")
        | Some("rar") => "",
        Some("mp3") | Some("wav") | Some("ogg") | Some("flac") | Some("m4a") => "",
        Some("mp4") | Some("mkv") | Some("mov") | Some("avi") | Some("webm") => "",
        Some("pdf") => "",
        Some("log") => "",
        _ => "",
    }
}
