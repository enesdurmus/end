use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Clip {
    Text {
        text: String,
    },
    Image {
        path: String,
        name: String,
        width: u32,
        height: u32,
        bytes: u64,
    },
    Files {
        paths: Vec<String>,
    },
}

impl Clip {
    /// Kind-prefixed, so text can never collide with a file list of the same characters.
    pub fn dedupe_key(&self) -> String {
        match self {
            Clip::Text { text } => format!("t:{text}"),
            Clip::Image { path, .. } => format!("i:{path}"),
            Clip::Files { paths } => format!("f:{}", paths.join("\n")),
        }
    }

    fn image_path(&self) -> Option<&str> {
        match self {
            Clip::Image { path, .. } => Some(path),
            _ => None,
        }
    }
}

/// Returns evicted image paths; deleting them is the caller's job.
pub fn push_capped(
    list: &mut Vec<Clip>,
    item: Clip,
    text_cap: usize,
    image_cap: usize,
) -> Vec<String> {
    if list.first().map(Clip::dedupe_key) == Some(item.dedupe_key()) {
        return Vec::new();
    }
    let key = item.dedupe_key();
    list.retain(|c| c.dedupe_key() != key);
    list.insert(0, item);
    trim(list, text_cap, image_cap)
}

/// Returns evicted image paths; deleting them is the caller's job.
pub fn trim(list: &mut Vec<Clip>, text_cap: usize, image_cap: usize) -> Vec<String> {
    let (mut texts, mut images) = (0usize, 0usize);
    let mut evicted = Vec::new();
    list.retain(|c| match c.image_path() {
        Some(path) => {
            images += 1;
            let keep = images <= image_cap;
            if !keep {
                evicted.push(path.to_string());
            }
            keep
        }
        None => {
            texts += 1;
            texts <= text_cap
        }
    });
    evicted
}

const IMAGE_EXTS: [&str; 7] = ["png", "jpg", "jpeg", "gif", "webp", "svg", "avif"];

/// Copied pixels carry no filename of their own.
pub fn image_name(url: Option<&str>, app: Option<&str>) -> String {
    if let Some(name) = image_file_name_of(url.unwrap_or_default()) {
        return name;
    }
    match app {
        Some(a) if !a.is_empty() => a.to_string(),
        _ => "Image".to_string(),
    }
}

/// `None` for a page URL: every row would be titled "index.html".
fn image_file_name_of(url: &str) -> Option<String> {
    let path = url.split(['?', '#']).next()?;
    let name = path.rsplit('/').next()?;
    let ext = name.rsplit_once('.')?.1.to_ascii_lowercase();
    IMAGE_EXTS
        .contains(&ext.as_str())
        .then(|| percent_decode(name))
        .filter(|n| !n.is_empty())
}

fn percent_decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        let hex = (i + 2 < b.len())
            .then(|| std::str::from_utf8(&b[i + 1..i + 3]).ok())
            .flatten()
            .filter(|_| b[i] == b'%')
            .and_then(|h| u8::from_str_radix(h, 16).ok());
        match hex {
            Some(byte) => {
                out.push(byte);
                i += 3;
            }
            None => {
                out.push(b[i]);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

pub fn store_path(app_dir: &Path) -> PathBuf {
    app_dir.join("clipboard.json")
}

pub fn images_dir(app_dir: &Path) -> PathBuf {
    app_dir.join("clipboard-images")
}

pub fn load(app_dir: &Path) -> Vec<Clip> {
    let Ok(s) = std::fs::read_to_string(store_path(app_dir)) else {
        return Vec::new();
    };
    if let Ok(list) = serde_json::from_str::<Vec<Clip>>(&s) {
        return list;
    }
    load_untyped_history(&s)
}

/// History written before entries had a kind: a flat array of strings.
fn load_untyped_history(s: &str) -> Vec<Clip> {
    serde_json::from_str::<Vec<String>>(s)
        .map(|v| v.into_iter().map(|text| Clip::Text { text }).collect())
        .unwrap_or_default()
}

pub fn save(app_dir: &Path, list: &[Clip]) {
    let _ = std::fs::create_dir_all(app_dir);
    if let Ok(s) = serde_json::to_string(list) {
        let _ = std::fs::write(store_path(app_dir), s);
    }
}

/// Content-hashed, so re-copying an image costs a hash, not a second file.
pub fn store_image(
    dir: &Path,
    rgba: &[u8],
    width: u32,
    height: u32,
    name: String,
) -> Result<Clip, String> {
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let mut h = DefaultHasher::new();
    rgba.hash(&mut h);
    width.hash(&mut h);
    height.hash(&mut h);
    let path = dir.join(format!("{:016x}.png", h.finish()));

    let bytes = match std::fs::metadata(&path) {
        Ok(m) => m.len(),
        Err(_) => {
            let png = encode_png(rgba, width, height)?;
            std::fs::write(&path, &png).map_err(|e| e.to_string())?;
            png.len() as u64
        }
    };
    Ok(Clip::Image {
        path: path.to_string_lossy().into_owned(),
        name,
        width,
        height,
        bytes,
    })
}

fn encode_png(rgba: &[u8], width: u32, height: u32) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    let mut enc = png::Encoder::new(&mut out, width, height);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    let mut writer = enc.write_header().map_err(|e| e.to_string())?;
    writer.write_image_data(rgba).map_err(|e| e.to_string())?;
    writer.finish().map_err(|e| e.to_string())?;
    Ok(out)
}

/// Also catches blobs orphaned by a crash between image write and history save.
pub fn sweep_images(dir: &Path, list: &[Clip]) {
    let kept: Vec<&str> = list.iter().filter_map(Clip::image_path).collect();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.extension().is_some_and(|x| x == "png") && !kept.iter().any(|k| Path::new(k) == p) {
            let _ = std::fs::remove_file(p);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const CAP: usize = 500;
    const IMG_CAP: usize = 50;

    fn text(s: &str) -> Clip {
        Clip::Text { text: s.into() }
    }
    fn image(path: &str) -> Clip {
        Clip::Image { path: path.into(), name: "Image".into(), width: 1, height: 1, bytes: 1 }
    }

    #[test]
    fn cap_and_ordering() {
        let mut l = Vec::new();
        for i in 0..600 {
            push_capped(&mut l, text(&i.to_string()), CAP, IMG_CAP);
        }
        assert_eq!(l.len(), CAP);
        assert_eq!(l[0], text("599"));
    }

    #[test]
    fn duplicate_moved_to_front_consecutive_ignored() {
        let mut l = Vec::new();
        push_capped(&mut l, text("a"), CAP, IMG_CAP);
        push_capped(&mut l, text("b"), CAP, IMG_CAP);
        push_capped(&mut l, text("a"), CAP, IMG_CAP);
        assert_eq!(l, vec![text("a"), text("b")]);
        push_capped(&mut l, text("a"), CAP, IMG_CAP);
        assert_eq!(l, vec![text("a"), text("b")]);
    }

    #[test]
    fn images_are_capped_separately_from_text() {
        let mut l = Vec::new();
        for i in 0..10 {
            push_capped(&mut l, text(&i.to_string()), CAP, 2);
            push_capped(&mut l, image(&format!("/img/{i}.png")), CAP, 2);
        }
        assert_eq!(l.iter().filter(|c| c.image_path().is_some()).count(), 2);
        assert_eq!(l.iter().filter(|c| c.image_path().is_none()).count(), 10);
    }

    #[test]
    fn evicted_images_are_reported_for_deletion() {
        let mut l = Vec::new();
        push_capped(&mut l, image("/img/old.png"), CAP, 1);
        let evicted = push_capped(&mut l, image("/img/new.png"), CAP, 1);
        assert_eq!(evicted, vec!["/img/old.png"]);
    }

    #[test]
    fn re_copied_image_is_not_evicted() {
        let mut l = Vec::new();
        push_capped(&mut l, image("/img/a.png"), CAP, 2);
        push_capped(&mut l, text("x"), CAP, 2);
        let evicted = push_capped(&mut l, image("/img/a.png"), CAP, 2);
        assert!(evicted.is_empty(), "the surviving entry still points at that file");
        assert_eq!(l, vec![image("/img/a.png"), text("x")]);
    }

    #[test]
    fn an_image_is_named_after_its_source_url_when_that_url_names_an_image() {
        assert_eq!(image_name(Some("https://x.dev/a/logo.png?v=2"), Some("Safari")), "logo.png");
        assert_eq!(image_name(Some("https://x.dev/my%20logo.png"), None), "my logo.png");
        assert_eq!(image_name(Some("https://x.dev/A/LOGO.JPG"), None), "LOGO.JPG");
    }

    #[test]
    fn an_image_falls_back_to_the_app_when_the_url_is_not_an_image() {
        assert_eq!(image_name(Some("https://x.dev/pricing"), Some("Safari")), "Safari");
        assert_eq!(image_name(Some("https://x.dev/index.html"), Some("Safari")), "Safari");
        assert_eq!(image_name(None, Some("Figma")), "Figma");
        assert_eq!(image_name(None, None), "Image");
        assert_eq!(image_name(None, Some("")), "Image");
    }

    #[test]
    fn a_text_entry_never_collides_with_an_identical_file_list() {
        assert_ne!(
            text("/a\n/b").dedupe_key(),
            Clip::Files { paths: vec!["/a".into(), "/b".into()] }.dedupe_key()
        );
    }

    #[test]
    fn entries_serialize_in_the_shape_the_frontend_expects() {
        let json = serde_json::to_string(&vec![
            text("hi"),
            Clip::Image { path: "/i/a.png".into(), name: "logo.png".into(), width: 2, height: 1, bytes: 9 },
            Clip::Files { paths: vec!["/d/a.txt".into()] },
        ])
        .unwrap();
        assert_eq!(
            json,
            r#"[{"kind":"text","text":"hi"},{"kind":"image","path":"/i/a.png","name":"logo.png","width":2,"height":1,"bytes":9},{"kind":"files","paths":["/d/a.txt"]}]"#,
            "the Clip union in src/types.ts is written against this exact shape"
        );
    }

    #[test]
    fn untyped_history_files_still_load() {
        let dir = std::env::temp_dir().join("launcher-clip-migrate-test");
        let _ = std::fs::create_dir_all(&dir);
        std::fs::write(store_path(&dir), r#"["one","two"]"#).unwrap();
        assert_eq!(load(&dir), vec![text("one"), text("two")]);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn storing_an_image_twice_reuses_the_one_file() {
        let dir = std::env::temp_dir().join("launcher-clip-image-test");
        let _ = std::fs::remove_dir_all(&dir);
        let rgba = vec![255u8; 4 * 2 * 2];
        let first = store_image(&dir, &rgba, 2, 2, "Safari".into()).unwrap();
        let second = store_image(&dir, &rgba, 2, 2, "Safari".into()).unwrap();
        assert_eq!(first, second);
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 1);

        let Clip::Image { path, width, height, bytes, .. } = &first else { panic!("not an image") };
        assert_eq!((*width, *height), (2, 2));
        assert_eq!(*bytes, std::fs::metadata(path).unwrap().len());
        assert_eq!(&std::fs::read(path).unwrap()[..8], b"\x89PNG\r\n\x1a\n", "a real PNG header");

        sweep_images(&dir, &[]);
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 0);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
