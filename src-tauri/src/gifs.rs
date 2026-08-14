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
    /// Thumbnail: an https URL when remote, a file path when local.
    pub preview: String,
    /// What gets copied: the full-size media URL, or the file path.
    pub url: String,
    pub source: Source,
}

const EXTENSIONS: [&str; 5] = ["gif", "png", "jpg", "jpeg", "webp"];

pub fn list_local(dir: &Path) -> Vec<Gif> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
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

pub fn unique_stem(stem: &str, taken: impl Fn(&str) -> bool) -> String {
    if !taken(stem) {
        return stem.to_string();
    }
    (2..).map(|n| format!("{stem}-{n}")).find(|c| !taken(c)).unwrap()
}

pub fn dir(app_dir: &Path, configured: &str) -> PathBuf {
    if configured.trim().is_empty() {
        app_dir.join("gifs")
    } else {
        PathBuf::from(configured)
    }
}

/// Extractable from any binary this ships in; the `klipy_api_key` preference
/// overrides it, and distributing means deleting this and requiring that.
pub const KLIPY_KEY: &str = "oLFOeHPRxnvuhMqzpliTykG5cG9VX2UzBcEyQKshp2AaAmk2pNu2FOGy3FyVlgGc";

const KLIPY_BASE: &str = "https://api.klipy.com/api/v1";

#[derive(Deserialize)]
struct KlipyResponse {
    data: KlipyPage,
}

#[derive(Deserialize)]
struct KlipyPage {
    data: Vec<KlipyItem>,
}

/// No `deny_unknown_fields`: fields KLIPY adds later must not be fatal.
#[derive(Deserialize)]
struct KlipyItem {
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

fn best_gif(
    file: &std::collections::HashMap<String, std::collections::HashMap<String, KlipyMedia>>,
    sizes_by_preference: &[&str],
) -> Option<String> {
    sizes_by_preference.iter().find_map(|size| file.get(*size)?.get("gif")).map(|m| m.url.clone())
}

#[derive(Deserialize)]
struct KlipyError {
    errors: KlipyErrorBody,
}

#[derive(Deserialize)]
struct KlipyErrorBody {
    message: Vec<String>,
}

fn parse_klipy(body: &str) -> Result<Vec<Gif>, String> {
    // md beats hd: measured at 640x640/1.2MB against hd's 498x498/4.0MB
    const FULL_SIZES: [&str; 4] = ["md", "hd", "sm", "xs"];
    const PREVIEW_SIZES: [&str; 4] = ["sm", "xs", "md", "hd"];

    let parsed: KlipyResponse = serde_json::from_str(body).map_err(|e| e.to_string())?;
    Ok(parsed
        .data
        .data
        .into_iter()
        .filter_map(|item| {
            let full = best_gif(&item.file, &FULL_SIZES)?;
            let preview = best_gif(&item.file, &PREVIEW_SIZES).unwrap_or_else(|| full.clone());
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
        // a bad key answers 404, not 401/403, so no status code is worth special-casing
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

/// Never leaves a half-written file at `path` for `download_temp` to reuse.
fn write_atomically(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let tmp = path.with_extension("part");
    let result = std::fs::write(&tmp, bytes)
        .and_then(|()| std::fs::rename(&tmp, path))
        .map_err(|e| e.to_string());
    if result.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    result
}

/// Named after the provider's id, so pasting the same GIF twice reuses the file.
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
        assert!(
            !titles.contains(&"trap".to_string()),
            "a directory named *.gif is not an image"
        );
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn list_local_orders_newest_first_by_mtime() {
        use std::thread;
        use std::time::Duration;

        let dir = std::env::temp_dir().join(format!("gifs-test-order-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();

        for name in ["a.gif", "b.gif", "c.gif"] {
            std::fs::write(dir.join(name), b"x").unwrap();
            thread::sleep(Duration::from_millis(10));
        }

        let got = list_local(&dir);
        let titles: Vec<_> = got.iter().map(|g| g.title.clone()).collect();

        assert_eq!(titles, vec!["c", "b", "a"], "newest first");
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn list_local_on_a_missing_folder_is_empty_not_an_error() {
        assert!(list_local(Path::new("/nonexistent/gif/folder")).is_empty());
    }

    const KLIPY_SEARCH_FIXTURE: &str = include_str!("../tests/fixtures/klipy-search.json");
    const KLIPY_BAD_KEY_FIXTURE: &str = include_str!("../tests/fixtures/klipy-bad-key.json");

    #[test]
    fn klipy_response_maps_to_gifs() {
        let got = parse_klipy(KLIPY_SEARCH_FIXTURE).unwrap();

        assert_eq!(got.len(), 2);
        assert_eq!(got[0].id, "2424550499023490");
        assert_eq!(got[0].title, "Pikachu Shocked Face Stunned - Surprised Meme");
        assert_eq!(got[0].source, Source::Remote);

        assert_eq!(
            got[0].url,
            "https://static.klipy.com/ii/f87f46a2c5aeaeed4c68910815f73eaf/49/8e/pL5ZuAoY.gif",
            "md wins the full-size race: bigger on screen and smaller to send than hd"
        );
        assert_eq!(
            got[0].preview,
            "https://static.klipy.com/ii/f87f46a2c5aeaeed4c68910815f73eaf/49/8e/HbsWTaoP.gif",
            "sm wins the preview race: smallest size that holds up as a thumbnail"
        );
    }

    #[test]
    fn klipy_tolerates_unknown_fields() {
        assert_eq!(
            parse_klipy(KLIPY_SEARCH_FIXTURE).unwrap().len(),
            2,
            "the fixture carries slug/tags/type/blur_preview, none of them modelled"
        );
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

    #[test]
    fn klipy_bad_key_body_surfaces_the_providers_own_message() {
        let parsed: KlipyError = serde_json::from_str(KLIPY_BAD_KEY_FIXTURE).unwrap();
        assert_eq!(
            parsed.errors.message.join(" "),
            "The provided API key is invalid: [BADKEY123]"
        );
    }

    #[test]
    fn write_atomically_leaves_no_partial_file_when_the_write_fails() {
        let never_created = std::env::temp_dir().join(format!("gifs-test-missing-{}", std::process::id()));
        let path = never_created.join("x.gif");

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
        assert!(
            !path.with_extension("part").exists(),
            "the temp file should not survive a successful write"
        );
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
