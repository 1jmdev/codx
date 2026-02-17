use std::{
    fs,
    path::{Path, PathBuf},
};

pub(super) fn collect_project_files(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    collect_project_files_into(root, &mut out);
    out.sort();
    out
}

fn collect_project_files_into(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let Ok(file_type) = entry.file_type() else {
            continue;
        };

        if file_type.is_symlink() {
            continue;
        }

        if file_type.is_dir() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name == ".git" || name == "target" {
                continue;
            }
            collect_project_files_into(&path, out);
            continue;
        }

        if file_type.is_file() {
            out.push(path);
        }
    }
}
