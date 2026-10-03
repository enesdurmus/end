//! Bundle icon extraction, cached as PNG in the temp dir.

use super::super::base64_encode;
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::process::Command;

pub(super) fn app_icon(app_path: &str) -> Option<String> {
    let icns = find_icns(app_path)?;
    let mut h = DefaultHasher::new();
    app_path.hash(&mut h);
    let cache = std::env::temp_dir().join("launcher-icons");
    let _ = fs::create_dir_all(&cache);
    let png = cache.join(format!("{:x}.png", h.finish()));
    if !png.exists() {
        let ok = Command::new("sips")
            .args(["-s", "format", "png", "-Z", "64"])
            .arg(&icns)
            .arg("--out")
            .arg(&png)
            .output()
            .ok()?;
        if !ok.status.success() {
            return None;
        }
    }
    let bytes = fs::read(&png).ok()?;
    Some(format!("data:image/png;base64,{}", base64_encode(&bytes)))
}

fn find_icns(app_path: &str) -> Option<PathBuf> {
    let res = PathBuf::from(app_path).join("Contents/Resources");
    let plist = PathBuf::from(app_path).join("Contents/Info.plist");
    if let Ok(out) = Command::new("plutil")
        .args(["-extract", "CFBundleIconFile", "raw", "-o", "-"])
        .arg(&plist)
        .output()
    {
        if out.status.success() {
            let mut name = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !name.is_empty() {
                if !name.ends_with(".icns") {
                    name.push_str(".icns");
                }
                let cand = res.join(&name);
                if cand.exists() {
                    return Some(cand);
                }
            }
        }
    }
    fs::read_dir(&res)
        .ok()?
        .flatten()
        .map(|e| e.path())
        .find(|p| p.extension().is_some_and(|x| x == "icns"))
}
