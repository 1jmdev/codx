use std::{
    fs,
    path::{Path, PathBuf},
};

pub(super) fn collect_project_files(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    walk_files(root, root, &mut out);
    out.sort();
    out
}

fn walk_files(root: &Path, dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let Ok(ft) = entry.file_type() else { continue };
        if ft.is_symlink() {
            continue;
        }
        if ft.is_dir() {
            let name = entry.file_name().to_string_lossy().to_string();
            if is_ignored_dir(&name) || is_gitignored(root, &path) {
                continue;
            }
            walk_files(root, &path, out);
        } else if ft.is_file() && !is_gitignored(root, &path) {
            out.push(path);
        }
    }
}

fn is_ignored_dir(name: &str) -> bool {
    matches!(name, ".git" | "target" | "node_modules" | ".svn" | ".hg")
}

fn is_gitignored(root: &Path, path: &Path) -> bool {
    let Ok(rel) = path.strip_prefix(root) else {
        return false;
    };

    let gitignore = root.join(".gitignore");
    let Ok(content) = fs::read_to_string(&gitignore) else {
        return false;
    };

    let rel_str = rel.to_string_lossy().replace('\\', "/");

    for line in content.lines() {
        let pattern = line.trim();
        if pattern.is_empty() || pattern.starts_with('#') {
            continue;
        }
        let pattern = pattern.trim_start_matches('/');
        if rel_str == pattern
            || rel_str.starts_with(&format!("{}/", pattern))
            || rel_str.ends_with(&format!("/{}", pattern))
            || path
                .file_name()
                .is_some_and(|n| n.to_string_lossy() == pattern)
        {
            return true;
        }
    }

    false
}

pub(super) fn grep_project(root: &Path, query: &str) -> Vec<super::types::GrepMatch> {
    if query.is_empty() {
        return Vec::new();
    }
    let q_chars: Vec<char> = query.chars().collect();
    let qlen = q_chars.len();
    let mut results = Vec::new();
    let mut files = Vec::new();
    walk_files(root, root, &mut files);

    for path in files {
        if is_binary(&path) {
            continue;
        }
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        for (line_idx, line) in text.lines().enumerate() {
            let chars: Vec<char> = line.chars().collect();
            if chars.len() < qlen {
                continue;
            }
            let mut col = 0;
            while col + qlen <= chars.len() {
                let hit = chars[col..col + qlen]
                    .iter()
                    .zip(q_chars.iter())
                    .all(|(a, b)| a.eq_ignore_ascii_case(b));
                if hit {
                    results.push(super::types::GrepMatch {
                        path: path.clone(),
                        line_number: line_idx,
                        line_text: line.to_string(),
                        col_start: col,
                    });
                    col += qlen;
                } else {
                    col += 1;
                }
            }
        }
    }

    results
}

fn is_binary(path: &Path) -> bool {
    let Ok(bytes) = fs::read(path) else {
        return true;
    };
    let sample = &bytes[..bytes.len().min(512)];
    sample.iter().any(|&b| b == 0)
}
