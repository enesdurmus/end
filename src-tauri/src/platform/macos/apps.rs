//! `.app` bundle discovery and launching.

use super::AppEntry;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

pub(super) fn list_apps() -> Vec<AppEntry> {
    let mut out = Vec::new();
    scan_apps(PathBuf::from("/Applications"), &mut out);
    scan_apps(PathBuf::from("/System/Applications"), &mut out);
    scan_apps(PathBuf::from("/System/Applications/Utilities"), &mut out);
    if let Some(home) = std::env::var_os("HOME") {
        scan_apps(PathBuf::from(home).join("Applications"), &mut out);
    }
    out.sort_by_key(|a| a.name.to_lowercase());
    out
}

pub(super) fn open(path: &str) -> Result<(), String> {
    Command::new("open").arg(path).spawn().map(|_| ()).map_err(|e| e.to_string())
}

fn scan_apps(dir: PathBuf, out: &mut Vec<AppEntry>) {
    if let Ok(entries) = fs::read_dir(&dir) {
        for e in entries.flatten() {
            let p = e.path();
            if p.extension().is_some_and(|x| x == "app") {
                if let Some(name) = p.file_stem().and_then(|s| s.to_str()) {
                    out.push(AppEntry {
                        name: name.to_string(),
                        path: p.to_string_lossy().to_string(),
                    });
                }
            }
        }
    }
}
