//! File search: plocate/locate where installed, a bounded $HOME walk otherwise.

use super::{home, on_path, AppEntry};
use std::fs;
use std::path::Path;
use std::process::Command;

pub(super) fn search_files(query: &str) -> Vec<AppEntry> {
    let query = query.trim();
    if query.is_empty() {
        return Vec::new();
    }
    match ["plocate", "locate"].into_iter().find(|b| on_path(b)) {
        Some(bin) => Command::new(bin)
            .args(["-i", "-l", "20", "--", query])
            .output()
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).lines().map(file_entry).collect())
            .unwrap_or_default(),
        None => walk_home(&query.to_lowercase()),
    }
}

fn file_entry(path: &str) -> AppEntry {
    let name = Path::new(path).file_name().and_then(|s| s.to_str()).unwrap_or(path);
    AppEntry { name: name.to_string(), path: path.to_string() }
}

fn walk_home(needle: &str) -> Vec<AppEntry> {
    const MAX_DEPTH: usize = 3;
    const MAX_RESULTS: usize = 20;
    const MAX_DIRS: usize = 2_000;
    let mut out = Vec::new();
    let mut budget = MAX_DIRS;
    let mut level = vec![home()];
    for _ in 0..MAX_DEPTH {
        let mut next = Vec::new();
        for dir in level {
            if budget == 0 {
                return out;
            }
            budget -= 1;
            let Ok(entries) = fs::read_dir(&dir) else { continue };
            for e in entries.flatten() {
                let name = e.file_name().to_string_lossy().to_string();
                if name.starts_with('.') {
                    continue;
                }
                if name.to_lowercase().contains(needle) {
                    out.push(file_entry(&e.path().to_string_lossy()));
                    if out.len() >= MAX_RESULTS {
                        return out;
                    }
                }
                if e.file_type().is_ok_and(|t| t.is_dir()) {
                    next.push(e.path());
                }
            }
        }
        level = next;
    }
    out
}
