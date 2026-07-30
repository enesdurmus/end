use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Clone)]
pub struct Snippet {
    pub keyword: String,
    pub text: String,
}

fn path(dir: &Path) -> PathBuf {
    dir.join("snippets.json")
}

pub fn load(dir: &Path) -> Vec<Snippet> {
    std::fs::read_to_string(path(dir)).ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save(dir: &Path, items: &[Snippet]) -> Result<(), String> {
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let s = serde_json::to_string_pretty(items).map_err(|e| e.to_string())?;
    std::fs::write(path(dir), s).map_err(|e| e.to_string())
}
