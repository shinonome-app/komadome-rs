pub mod builder;
pub mod contracts;
pub mod kana;
pub mod templates;

use anyhow::Result;
use std::path::{Path, PathBuf};

/// Recursively collect files under `dir` whose extension equals `ext` (without the dot).
/// Returns an empty vec if `dir` does not exist. Shared by the template (`ntzr`) and
/// contract (`ntzc`) registries.
pub fn find_files_with_ext(dir: &Path, ext: &str) -> Result<Vec<PathBuf>> {
    let mut results = Vec::new();
    if dir.exists() {
        find_files_recursive(dir, ext, &mut results)?;
    }
    Ok(results)
}

fn find_files_recursive(dir: &Path, ext: &str, results: &mut Vec<PathBuf>) -> Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            find_files_recursive(&path, ext, results)?;
        } else if path.extension().and_then(|s| s.to_str()) == Some(ext) {
            results.push(path);
        }
    }
    Ok(())
}
