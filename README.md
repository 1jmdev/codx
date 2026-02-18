# Codx

Codx is a fast terminal code editor written in Rust. It focuses on keyboard-first editing, lightweight navigation, and a simple TUI workflow for working directly inside your project directory.

## Features

- Terminal UI powered by Ratatui + Crossterm
- Project file tree and quick file picker (`Ctrl+P`)
- Command palette (`Ctrl+Space` or `F1`)
- Basic code editing with undo/redo, duplicate/delete line, and indentation controls
- Syntax highlighting support
- LSP integration with reload command from the command palette
- Nerd Font file and folder icons

## Installation

### Install from local clone

```bash
git clone https://github.com/1jmdev/codx
cd codx
cargo install --path .
```

After install, run:

```bash
codx
```

### Run without installing (development)

```bash
cargo run
```

## Usage

Codx supports three launch modes:

Opens Codx in the current directory.
```bash
codx
```

Opens only that file and starts with the sidebar hidden.
```bash
codx path/file.rs
```

Opens Codx rooted in that folder.
```bash
codx path/to/folder
```

In directory mode, you can browse/open files from the sidebar and file palette.

For the best icons, use a Nerd Font (for example, JetBrainsMono Nerd Font or FiraCode Nerd Font) in your terminal.

## Default Keybindings

- `Ctrl+Q` quit
- `Ctrl+S` save current file
- `Ctrl+P` open file palette
- `Ctrl+Space` / `F1` open command palette
- `Ctrl+B` toggle sidebar
- `Esc` focus file tree
- `Ctrl+Z` undo
- `Ctrl+Y` redo
- `Ctrl+D` duplicate line or selection
- `Ctrl+Shift+K` delete line or selection

## License

This project is licensed under the MIT License. See `LICENSE` for details.
