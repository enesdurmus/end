//! Persisted translation history. Same shape as `clipboard.rs`, but entries are
//! structs and dedupe is keyed on (source, to).

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub const HISTORY_LIMIT: usize = 100;

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Entry {
    pub source: String,
    pub translated: String,
    pub from: String,
    pub to: String,
}

pub fn push_capped(list: &mut Vec<Entry>, e: Entry, cap: usize) {
    // same text to the same language is one entry, moved back to the front
    list.retain(|x| !(x.source == e.source && x.to == e.to));
    list.insert(0, e);
    if list.len() > cap {
        list.truncate(cap);
    }
}

pub fn store_path(app_dir: &Path) -> PathBuf {
    app_dir.join("translations.json")
}

pub fn load(app_dir: &Path) -> Vec<Entry> {
    std::fs::read_to_string(store_path(app_dir)).ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save(app_dir: &Path, list: &[Entry]) {
    let _ = std::fs::create_dir_all(app_dir);
    if let Ok(s) = serde_json::to_string(list) {
        let _ = std::fs::write(store_path(app_dir), s);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn e(source: &str, to: &str) -> Entry {
        Entry {
            source: source.into(),
            translated: format!("{source}-{to}"),
            from: "tr".into(),
            to: to.into(),
        }
    }

    #[test]
    fn newest_first_and_capped() {
        let mut l = Vec::new();
        for i in 0..(HISTORY_LIMIT + 20) {
            push_capped(&mut l, e(&i.to_string(), "en"), HISTORY_LIMIT);
        }
        assert_eq!(l.len(), HISTORY_LIMIT);
        assert_eq!(l[0].source, (HISTORY_LIMIT + 19).to_string());
    }

    #[test]
    fn same_source_and_target_moves_to_front_instead_of_duplicating() {
        let mut l = Vec::new();
        push_capped(&mut l, e("a", "en"), HISTORY_LIMIT);
        push_capped(&mut l, e("b", "en"), HISTORY_LIMIT);
        push_capped(&mut l, e("a", "en"), HISTORY_LIMIT);
        assert_eq!(l.len(), 2);
        assert_eq!(l[0].source, "a");
    }

    #[test]
    fn same_source_different_target_is_a_separate_entry() {
        let mut l = Vec::new();
        push_capped(&mut l, e("a", "en"), HISTORY_LIMIT);
        push_capped(&mut l, e("a", "de"), HISTORY_LIMIT);
        assert_eq!(l.len(), 2);
    }
}
