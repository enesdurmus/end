use std::path::{Path, PathBuf};

pub fn push_capped(list: &mut Vec<String>, item: String, cap: usize) {
    if list.first() == Some(&item) {
        return;
    }
    list.retain(|x| x != &item);
    list.insert(0, item);
    if list.len() > cap {
        list.truncate(cap);
    }
}

pub fn store_path(app_dir: &Path) -> PathBuf {
    app_dir.join("clipboard.json")
}

pub fn load(app_dir: &Path) -> Vec<String> {
    let p = store_path(app_dir);
    std::fs::read_to_string(p).ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save(app_dir: &Path, list: &[String]) {
    let _ = std::fs::create_dir_all(app_dir);
    if let Ok(s) = serde_json::to_string(list) {
        let _ = std::fs::write(store_path(app_dir), s);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const CAP: usize = 500;
    #[test]
    fn cap_and_ordering() {
        let mut l = Vec::new();
        for i in 0..600 { push_capped(&mut l, i.to_string(), CAP); }
        assert_eq!(l.len(), CAP);
        assert_eq!(l[0], "599");
    }
    #[test]
    fn duplicate_moved_to_front_consecutive_ignored() {
        let mut l = Vec::new();
        push_capped(&mut l, "a".into(), CAP);
        push_capped(&mut l, "b".into(), CAP);
        push_capped(&mut l, "a".into(), CAP);
        assert_eq!(l, vec!["a", "b"]);
        push_capped(&mut l, "a".into(), CAP);
        assert_eq!(l, vec!["a", "b"]);
    }
}
