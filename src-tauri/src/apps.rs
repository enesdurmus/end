use serde::Serialize;
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Clone)]
pub struct AppEntry {
    pub name: String,
    pub path: String,
}

fn scan(dir: PathBuf, out: &mut Vec<AppEntry>) {
    if let Ok(entries) = fs::read_dir(&dir) {
        for e in entries.flatten() {
            let p = e.path();
            if p.extension().map_or(false, |x| x == "app") {
                if let Some(name) = p.file_stem().and_then(|s| s.to_str()) {
                    out.push(AppEntry { name: name.to_string(), path: p.to_string_lossy().to_string() });
                }
            }
        }
    }
}

pub fn list() -> Vec<AppEntry> {
    let mut out = Vec::new();
    scan(PathBuf::from("/Applications"), &mut out);
    if let Some(home) = std::env::var_os("HOME") {
        scan(PathBuf::from(home).join("Applications"), &mut out);
    }
    out.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    out
}
