//! The GIF library: the folder on disk, and the shape a GIF takes on the way to
//! the frontend. Favouriting a remote GIF downloads it here, so "my favourites"
//! and "my local GIFs" are the same folder — there is no separate store to keep
//! in sync with the filesystem.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Source {
    Local,
    Remote,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Gif {
    pub id: String,
    pub title: String,
    /// What the row thumbnail renders: an https URL for a remote GIF, a file path for
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
            let metadata = e.metadata().ok()?;
            if !metadata.is_file() {
                return None;
            }
            let path = e.path();
            let ext = path.extension()?.to_str()?.to_lowercase();
            if !EXTENSIONS.contains(&ext.as_str()) {
                return None;
            }
            let title = path.file_stem()?.to_str()?.to_string();
            let full = path.to_str()?.to_string();
            let modified = metadata.modified().ok()?;
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

/// Shipped so the app works with no setup. A key inside a distributed binary is
/// extractable — acceptable while this is not distributed, and `klipy_api_key`
/// in preferences is the escape hatch. Distributing means deleting this const
/// and requiring the preference.
pub const KLIPY_KEY: &str = "oLFOeHPRxnvuhMqzpliTykG5cG9VX2UzBcEyQKshp2AaAmk2pNu2FOGy3FyVlgGc";

/// KLIPY puts the key in the path rather than a query parameter.
const KLIPY_BASE: &str = "https://api.klipy.com/api/v1";

#[derive(Deserialize)]
struct KlipyResponse {
    data: KlipyPage,
}

#[derive(Deserialize)]
struct KlipyPage {
    data: Vec<KlipyItem>,
}

#[derive(Deserialize)]
struct KlipyItem {
    /// A JSON number on the wire, not a string — the real shape, confirmed against
    /// a live response, differs from the third-party docs this was first built from.
    id: i64,
    #[serde(default)]
    title: String,
    #[serde(default)]
    file: std::collections::HashMap<String, std::collections::HashMap<String, KlipyMedia>>,
}

#[derive(Deserialize)]
struct KlipyMedia {
    url: String,
}

/// `size` ordered by preference, first `.gif` entry found under it wins.
fn best_gif(
    file: &std::collections::HashMap<String, std::collections::HashMap<String, KlipyMedia>>,
    sizes: &[&str],
) -> Option<String> {
    sizes.iter().find_map(|size| file.get(*size)?.get("gif")).map(|m| m.url.clone())
}

#[derive(Deserialize)]
struct KlipyError {
    errors: KlipyErrorBody,
}

#[derive(Deserialize)]
struct KlipyErrorBody {
    message: Vec<String>,
}

/// Deliberately tolerant: `#[serde(default)]` on `file` and no `deny_unknown_fields`
/// mean fields KLIPY adds later (or ones this struct doesn't model, like `slug`,
/// `tags`, `type`, `blur_preview`) are ignored rather than fatal, and an item with
/// no usable GIF is dropped rather than turned into a row that cannot be pasted.
fn parse_klipy(body: &str) -> Result<Vec<Gif>, String> {
    let parsed: KlipyResponse = serde_json::from_str(body).map_err(|e| e.to_string())?;
    Ok(parsed
        .data
        .data
        .into_iter()
        .filter_map(|item| {
            // Full size (copied/pasted): md before hd. Measured on a real item, md is
            // 640x640 at 1.2MB while hd is only 498x498 at 4.0MB — md is both bigger
            // on screen and far cheaper to send, so it wins despite the name.
            let full = best_gif(&item.file, &["md", "hd", "sm", "xs"])?;
            // Preview (row thumbnail): sm is 220x220 at ~97KB, the smallest size that
            // still looks right on a retina thumbnail row.
            let preview = best_gif(&item.file, &["sm", "xs", "md", "hd"]).unwrap_or_else(|| full.clone());
            Some(Gif {
                id: item.id.to_string(),
                title: item.title,
                preview,
                url: full,
                source: Source::Remote,
            })
        })
        .collect())
}

pub async fn search_klipy(query: &str, key: &str) -> Result<Vec<Gif>, String> {
    let key = if key.trim().is_empty() { KLIPY_KEY } else { key };
    let res = reqwest::Client::new()
        .get(format!("{KLIPY_BASE}/{key}/gifs/search"))
        .query(&[("q", query), ("per_page", "24")])
        .send()
        .await
        .map_err(|_| "klipy'ye ulaşılamadı".to_string())?;

    if !res.status().is_success() {
        let code = res.status().as_u16();
        let body = res.text().await.unwrap_or_default();
        // KLIPY returns 404 (not 401/403) for a bad key, so there is no status code
        // reliable enough to special-case — surface the provider's own message
        // instead, and fall back to a generic one only if the body doesn't parse.
        return Err(match serde_json::from_str::<KlipyError>(&body) {
            Ok(e) if !e.errors.message.is_empty() => e.errors.message.join(" "),
            _ => format!("klipy hatası ({code})"),
        });
    }

    let body = res.text().await.map_err(|e| e.to_string())?;
    parse_klipy(&body)
}

async fn fetch_bytes(url: &str) -> Result<Vec<u8>, String> {
    let res = reqwest::get(url).await.map_err(|_| "indirilemedi".to_string())?;
    if !res.status().is_success() {
        return Err(format!("indirilemedi ({})", res.status().as_u16()));
    }
    Ok(res.bytes().await.map_err(|e| e.to_string())?.to_vec())
}

/// Downloads into the library and returns the entry as it now exists on disk, so
/// the caller can show it as local without re-listing the folder.
pub async fn download(dir: &Path, url: &str, title: &str) -> Result<Gif, String> {
    let bytes = fetch_bytes(url).await?;
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;

    let stem = unique_stem(&slug(title), |s| dir.join(format!("{s}.gif")).exists());
    let path = dir.join(format!("{stem}.gif"));
    std::fs::write(&path, bytes).map_err(|e| e.to_string())?;

    let full = path.to_string_lossy().to_string();
    Ok(Gif {
        id: full.clone(),
        title: stem,
        preview: full.clone(),
        url: full,
        source: Source::Local,
    })
}

/// Writes `bytes` to `path` via a sibling temp file + rename, so a write that
/// fails partway (disk full, permissions) never leaves a partial file sitting
/// at `path` — `download_temp`'s existence check would otherwise treat that
/// partial file as a finished download forever.
fn write_atomically(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let tmp = path.with_extension("part");
    let result = std::fs::write(&tmp, bytes)
        .and_then(|()| std::fs::rename(&tmp, path))
        .map_err(|e| e.to_string());
    if result.is_err() {
        let _ = std::fs::remove_file(&tmp); // best-effort; the error above is what matters
    }
    result
}

/// For pasting a remote GIF without keeping it. Named by the provider's id, so
/// pasting the same GIF twice reuses the file instead of littering temp.
pub async fn download_temp(id: &str, url: &str) -> Result<PathBuf, String> {
    let path = std::env::temp_dir().join(format!("launcher-gif-{}.gif", slug(id)));
    if path.exists() {
        return Ok(path);
    }
    let bytes = fetch_bytes(url).await?;
    write_atomically(&path, &bytes)?;
    Ok(path)
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
        std::fs::create_dir_all(dir.join("trap.gif")).unwrap();
        for name in ["a.gif", "b.PNG", "c.webp", "notes.txt", "no-extension"] {
            std::fs::write(dir.join(name), b"x").unwrap();
        }

        let got = list_local(&dir);
        let mut titles: Vec<_> = got.iter().map(|g| g.title.clone()).collect();
        titles.sort();

        assert_eq!(titles, vec!["a", "b", "c"]);
        assert!(got.iter().all(|g| g.source == Source::Local));
        assert!(!titles.contains(&"trap".to_string()), "directory with .gif extension should be excluded");
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn list_local_orders_newest_first_by_mtime() {
        use std::thread;
        use std::time::Duration;

        let dir = std::env::temp_dir().join(format!("gifs-test-order-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();

        // Create files with distinct mtimes by creating them with delays.
        // Each write has at least 10ms between, which guarantees different mtimes on most filesystems.
        for name in ["a.gif", "b.gif", "c.gif"] {
            std::fs::write(dir.join(name), b"x").unwrap();
            thread::sleep(Duration::from_millis(10));
        }

        let got = list_local(&dir);
        let titles: Vec<_> = got.iter().map(|g| g.title.clone()).collect();

        // Newest first means c (created last), b (middle), a (created first).
        assert_eq!(titles, vec!["c", "b", "a"], "list_local should return files newest first by mtime");
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn list_local_on_a_missing_folder_is_empty_not_an_error() {
        assert!(list_local(Path::new("/nonexistent/gif/folder")).is_empty());
    }

    // Real response for q=pikachu&per_page=2, captured against the live API — the
    // parser must handle exactly this shape, not an invented approximation of it.
    const KLIPY_SEARCH_FIXTURE: &str = include_str!("../tests/fixtures/klipy-search.json");
    const KLIPY_BAD_KEY_FIXTURE: &str = include_str!("../tests/fixtures/klipy-bad-key.json");

    #[test]
    fn klipy_response_maps_to_gifs() {
        let got = parse_klipy(KLIPY_SEARCH_FIXTURE).unwrap();

        assert_eq!(got.len(), 2);
        assert_eq!(got[0].id, "2424550499023490");
        assert_eq!(got[0].title, "Pikachu Shocked Face Stunned - Surprised Meme");
        assert_eq!(got[0].source, Source::Remote);

        // md wins the full-size race: bigger on screen (640x640) and far smaller
        // to send (1.2MB) than hd (498x498 at 4.0MB) on this fixture's real item.
        assert_eq!(
            got[0].url,
            "https://static.klipy.com/ii/f87f46a2c5aeaeed4c68910815f73eaf/49/8e/pL5ZuAoY.gif"
        );
        // sm wins the preview race: smallest size that still holds up as a thumbnail.
        assert_eq!(
            got[0].preview,
            "https://static.klipy.com/ii/f87f46a2c5aeaeed4c68910815f73eaf/49/8e/HbsWTaoP.gif"
        );
    }

    // the real shape came from third-party docs first, so fields this struct never
    // modeled (slug, tags, type, blur_preview, and anything KLIPY adds later) must
    // never be fatal
    #[test]
    fn klipy_tolerates_unknown_fields() {
        assert_eq!(parse_klipy(KLIPY_SEARCH_FIXTURE).unwrap().len(), 2);
    }

    #[test]
    fn klipy_drops_items_with_no_usable_gif_rather_than_faking_one() {
        let body = r#"{"result":true,"data":{"data":[
          {"id":1,"title":"only webp","file":{"md":{"webp":{"url":"https://x/x.webp","width":1,"height":1,"size":1}}}}
        ]}}"#;
        assert!(parse_klipy(body).unwrap().is_empty());
    }

    #[test]
    fn klipy_falls_back_to_a_worse_size_when_the_preferred_one_is_missing() {
        let body = r#"{"result":true,"data":{"data":[
          {"id":1,"title":"x","file":{"hd":{"gif":{"url":"https://x/hd.gif","width":1,"height":1,"size":1}}}}
        ]}}"#;
        let got = parse_klipy(body).unwrap();
        assert_eq!(got[0].url, "https://x/hd.gif");
        assert_eq!(got[0].preview, "https://x/hd.gif");
    }

    #[test]
    fn klipy_empty_results_is_an_empty_list_not_an_error() {
        let body = r#"{"result":true,"data":{"data":[],"current_page":1}}"#;
        assert!(parse_klipy(body).unwrap().is_empty());
    }

    #[test]
    fn klipy_garbage_is_an_error_not_a_panic() {
        assert!(parse_klipy("not json").is_err());
    }

    // a bad key is a 404 on the real API, not 401/403 — the error path must not
    // special-case a status code and must surface the provider's own message
    #[test]
    fn klipy_bad_key_body_surfaces_the_providers_own_message() {
        let parsed: KlipyError = serde_json::from_str(KLIPY_BAD_KEY_FIXTURE).unwrap();
        assert_eq!(
            parsed.errors.message.join(" "),
            "The provided API key is invalid: [BADKEY123]"
        );
    }

    // item 6: a write that fails partway must not leave a file at the final path
    // for `download_temp`'s `path.exists()` check to mistake for a real download.
    #[test]
    fn write_atomically_leaves_no_partial_file_when_the_write_fails() {
        let bad_dir = std::env::temp_dir().join(format!("gifs-test-missing-{}", std::process::id()));
        let path = bad_dir.join("x.gif"); // bad_dir is never created, so the write fails

        let err = write_atomically(&path, b"partial-bytes");

        assert!(err.is_err());
        assert!(!path.exists(), "a failed write must not leave a file behind");
    }

    #[test]
    fn write_atomically_writes_the_final_file_on_success() {
        let dir = std::env::temp_dir().join(format!("gifs-test-atomic-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("x.gif");

        write_atomically(&path, b"hello").unwrap();

        assert_eq!(std::fs::read(&path).unwrap(), b"hello");
        assert!(!path.with_extension("part").exists(), "the temp file should not survive a successful write");
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
