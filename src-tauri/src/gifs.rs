//! The GIF library: the folder on disk, and the shape a GIF takes on the way to
//! the frontend. Favouriting a Tenor GIF downloads it here, so "my favourites"
//! and "my local GIFs" are the same folder — there is no separate store to keep
//! in sync with the filesystem.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Source {
    Local,
    Tenor,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Gif {
    pub id: String,
    pub title: String,
    /// What the row thumbnail renders: an https URL for Tenor, a file path for
    /// local entries (the frontend runs it through Tauri's asset protocol).
    pub preview: String,
    /// What gets copied: the full-size media URL, or the file path.
    pub url: String,
    pub source: Source,
}

const EXTENSIONS: [&str; 5] = ["gif", "png", "jpg", "jpeg", "webp"];

/// Every image in the library, newest first — so a GIF favourited a moment ago is
/// the first thing on screen next time the mode opens with an empty query.
/// ponytail: a full re-scan per open. A library is tens of files; add a cache
/// invalidated by folder mtime only if a scan ever shows up in a profile.
pub fn list_local(dir: &Path) -> Vec<Gif> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new(); // no folder yet is an empty library, not a failure
    };

    let mut found: Vec<(std::time::SystemTime, Gif)> = entries
        .flatten()
        .filter_map(|e| {
            let path = e.path();
            let ext = path.extension()?.to_str()?.to_lowercase();
            if !EXTENSIONS.contains(&ext.as_str()) {
                return None;
            }
            let title = path.file_stem()?.to_str()?.to_string();
            let full = path.to_str()?.to_string();
            let modified = e.metadata().ok()?.modified().ok()?;
            Some((
                modified,
                Gif {
                    id: full.clone(),
                    title,
                    preview: full.clone(),
                    url: full,
                    source: Source::Local,
                },
            ))
        })
        .collect();

    found.sort_by(|a, b| b.0.cmp(&a.0));
    found.into_iter().map(|(_, g)| g).collect()
}

/// "Surprised Pikachu!" -> "surprised-pikachu".
pub fn slug(title: &str) -> String {
    let folded: String = title
        .chars()
        .map(|c| match c {
            'ş' | 'Ş' => 's',
            'ı' | 'İ' => 'i',
            'ğ' | 'Ğ' => 'g',
            'ü' | 'Ü' => 'u',
            'ö' | 'Ö' => 'o',
            'ç' | 'Ç' => 'c',
            _ => c,
        })
        .collect();

    let s = folded
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join("-");

    if s.is_empty() { "gif".into() } else { s }
}

/// Appends -2, -3, ... until `taken` says the name is free. Takes a predicate
/// rather than a directory so it is testable without touching disk.
pub fn unique_stem(stem: &str, taken: impl Fn(&str) -> bool) -> String {
    if !taken(stem) {
        return stem.to_string();
    }
    (2..).map(|n| format!("{stem}-{n}")).find(|c| !taken(c)).unwrap()
}

/// The library folder: the preference if set, else `<app_config_dir>/gifs`.
pub fn dir(app_dir: &Path, configured: &str) -> PathBuf {
    if configured.trim().is_empty() {
        app_dir.join("gifs")
    } else {
        PathBuf::from(configured)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slug_lowercases_and_dashes() {
        assert_eq!(slug("Surprised Pikachu!"), "surprised-pikachu");
        assert_eq!(slug("  many   spaces  "), "many-spaces");
    }

    // Turkish titles are the common case for this user, and the folder is
    // synced/browsed in Finder, so names stay ASCII.
    #[test]
    fn slug_folds_turkish_characters() {
        assert_eq!(slug("Şaşkın Güç İçin"), "saskin-guc-icin");
    }

    #[test]
    fn slug_falls_back_when_nothing_survives() {
        assert_eq!(slug("!!!"), "gif");
        assert_eq!(slug(""), "gif");
    }

    #[test]
    fn unique_stem_appends_a_counter_until_free() {
        assert_eq!(unique_stem("cat", |_| false), "cat");
        assert_eq!(unique_stem("cat", |s| s == "cat"), "cat-2");
        assert_eq!(unique_stem("cat", |s| s == "cat" || s == "cat-2"), "cat-3");
    }

    #[test]
    fn list_local_keeps_images_and_skips_everything_else() {
        let dir = std::env::temp_dir().join(format!("gifs-test-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("subdir")).unwrap();
        for name in ["a.gif", "b.PNG", "c.webp", "notes.txt", "no-extension"] {
            std::fs::write(dir.join(name), b"x").unwrap();
        }

        let got = list_local(&dir);
        let mut titles: Vec<_> = got.iter().map(|g| g.title.clone()).collect();
        titles.sort();

        assert_eq!(titles, vec!["a", "b", "c"]);
        assert!(got.iter().all(|g| g.source == Source::Local));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn list_local_on_a_missing_folder_is_empty_not_an_error() {
        assert!(list_local(Path::new("/nonexistent/gif/folder")).is_empty());
    }
}
