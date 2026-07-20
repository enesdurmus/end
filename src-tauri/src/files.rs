use crate::apps::AppEntry;
use std::process::Command;

pub fn search(query: &str) -> Vec<AppEntry> {
    if query.trim().is_empty() {
        return Vec::new();
    }
    let out = match Command::new("mdfind").arg("-name").arg(query).output() {
        Ok(o) => o,
        Err(_) => return Vec::new(),
    };
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .take(20)
        .map(|path| {
            let name = std::path::Path::new(path)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or(path)
                .to_string();
            AppEntry { name, path: path.to_string() }
        })
        .collect()
}
