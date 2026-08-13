use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::translate::Provider;

// standalone so serde can use it as a per-field default when migrating old files
pub const DEFAULT_HISTORY_LIMIT: usize = 200;
fn default_history_limit() -> usize {
    DEFAULT_HISTORY_LIMIT
}

// Images are kept on their own, much smaller budget: 50 screenshots is tens of
// megabytes, 200 would be hundreds.
pub const DEFAULT_IMAGE_LIMIT: usize = 50;
fn default_image_limit() -> usize {
    DEFAULT_IMAGE_LIMIT
}

fn default_translate_target() -> String {
    "en".into()
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Preferences {
    pub toggle_shortcut: String,
    pub clipboard_shortcut: String,
    // #[serde(default)]: preferences.json written before this field existed still parses
    #[serde(default = "default_history_limit")]
    pub history_limit: usize,
    #[serde(default = "default_image_limit")]
    pub image_limit: usize,
    #[serde(default = "default_translate_target")]
    pub translate_target: String,
    // Provider::default() == Google; #[serde(default)] keeps old files parsing
    #[serde(default)]
    pub translate_provider: Provider,
    // empty means "use the key compiled into gifs.rs"
    #[serde(default)]
    pub klipy_api_key: String,
    // empty means "<app_config_dir>/gifs"
    #[serde(default)]
    pub gif_dir: String,
}

impl Default for Preferences {
    fn default() -> Self {
        Preferences {
            toggle_shortcut: "Super+Space".into(),
            clipboard_shortcut: "Super+Shift+KeyV".into(),
            history_limit: DEFAULT_HISTORY_LIMIT,
            image_limit: DEFAULT_IMAGE_LIMIT,
            translate_target: default_translate_target(),
            translate_provider: Provider::default(),
            klipy_api_key: String::new(),
            gif_dir: String::new(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn old_preferences_file_without_translate_fields_still_parses() {
        // a preferences.json written before translate existed
        let json = r#"{"toggle_shortcut":"Super+Space","clipboard_shortcut":"Super+Shift+KeyV","history_limit":200}"#;
        let p: Preferences = serde_json::from_str(json).unwrap();
        assert_eq!(p.translate_target, "en");
        assert_eq!(p.translate_provider, crate::translate::Provider::Google);
    }

    #[test]
    fn old_preferences_file_without_gif_fields_still_parses() {
        // a preferences.json written before gif mode existed
        let json = r#"{"toggle_shortcut":"Super+Space","clipboard_shortcut":"Super+Shift+KeyV","history_limit":200}"#;
        let p: Preferences = serde_json::from_str(json).unwrap();
        assert_eq!(p.klipy_api_key, "");
        assert_eq!(p.gif_dir, "");
    }

    #[test]
    fn old_preferences_file_without_an_image_limit_still_parses() {
        // a preferences.json written before the clipboard kept images
        let json = r#"{"toggle_shortcut":"Super+Space","clipboard_shortcut":"Super+Shift+KeyV","history_limit":200}"#;
        let p: Preferences = serde_json::from_str(json).unwrap();
        assert_eq!(p.image_limit, DEFAULT_IMAGE_LIMIT);
    }

    #[test]
    fn provider_round_trips_as_lowercase() {
        let p = Preferences::default();
        let s = serde_json::to_string(&p).unwrap();
        assert!(s.contains(r#""translate_provider":"google""#), "got {s}");
    }
}
