use std::fs;
use std::path::PathBuf;

fn collect_rs_files(dir: &PathBuf, acc: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_rs_files(&path, acc);
            } else if let Some(ext) = path.extension() {
                if ext == "rs" {
                    acc.push(path);
                }
            }
        }
    }
}

#[test]
fn enforce_error_policy_use_anyhow() {
    // Disallow references to the removed typed error helpers
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src_dir = manifest_dir.join("src");

    let mut files = Vec::new();
    collect_rs_files(&src_dir, &mut files);

    let mut violations = Vec::new();

    for file in files {
        if let Ok(content) = fs::read_to_string(&file) {
            if content.contains("AutoversionError") || content.contains("utils::errors") {
                violations.push(file.display().to_string());
            }
        }
    }

    if !violations.is_empty() {
        panic!(
            "Found references to deprecated typed errors in: {:?}\nPlease use anyhow::Result / anyhow::Error as the canonical error type.",
            violations
        );
    }
}
