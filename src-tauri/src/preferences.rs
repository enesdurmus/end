use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

// standalone so serde can use it as a per-field default when migrating old files
pub const DEFAULT_HISTORY_LIMIT: usize = 200;
fn default_history_limit() -> usize {
    DEFAULT_HISTORY_LIMIT
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Preferences {
    pub toggle_shortcut: String,
    pub clipboard_shortcut: String,
    // #[serde(default)]: preferences.json written before this field existed still parses
    #[serde(default = "default_history_limit")]
    pub history_limit: usize,
}

impl Default for Preferences {
    fn default() -> Self {
        Preferences {
            toggle_shortcut: "Super+Space".into(),
            clipboard_shortcut: "Super+Shift+KeyV".into(),
            history_limit: DEFAULT_HISTORY_LIMIT,
        }
    }
}

fn path(dir: &Path) -> PathBuf {
    dir.join("preferences.json")
}

pub fn load(dir: &Path) -> Preferences {
    std::fs::read_to_string(path(dir)).ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save(dir: &Path, prefs: &Preferences) -> Result<(), String> {
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let s = serde_json::to_string_pretty(prefs).map_err(|e| e.to_string())?;
    std::fs::write(path(dir), s).map_err(|e| e.to_string())
}
