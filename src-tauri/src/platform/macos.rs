use super::{AppEntry, Platform};
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};

pub struct MacOs;

impl Platform for MacOs {
    fn list_apps(&self) -> Vec<AppEntry> {
        let mut out = Vec::new();
        scan(PathBuf::from("/Applications"), &mut out);
        scan(PathBuf::from("/System/Applications"), &mut out);
        scan(PathBuf::from("/System/Applications/Utilities"), &mut out);
        if let Some(home) = std::env::var_os("HOME") {
            scan(PathBuf::from(home).join("Applications"), &mut out);
        }
        out.sort_by_key(|a| a.name.to_lowercase());
        out
    }

    fn app_icon(&self, app_path: &str) -> Option<String> {
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
        Some(format!("data:image/png;base64,{}", b64(&bytes)))
    }

    fn open_path(&self, path: &str) -> Result<(), String> {
        Command::new("open").arg(path).spawn().map(|_| ()).map_err(|e| e.to_string())
    }

    fn search_files(&self, query: &str) -> Vec<AppEntry> {
        if query.trim().is_empty() {
            return Vec::new();
        }
        // ponytail: mdfind can return tens of thousands of lines for common words;
        // stream stdout and stop at the first 20 → child dies via SIGPIPE.
        let mut child = match Command::new("mdfind").arg("-name").arg(query).stdout(Stdio::piped()).spawn() {
            Ok(c) => c,
            Err(_) => return Vec::new(),
        };
        let reader = match child.stdout.take() {
            Some(o) => BufReader::new(o),
            None => return Vec::new(),
        };
        let out: Vec<AppEntry> = reader
            .lines()
            .map_while(Result::ok)
            .take(20)
            .map(|path| {
                let name = std::path::Path::new(&path)
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or(&path)
                    .to_string();
                AppEntry { name, path }
            })
            .collect();
        let _ = child.kill();
        let _ = child.wait();
        out
    }

    // ponytail: lsappinfo (~10ms, no Automation permission) instead of
    // osascript+System Events (~400ms). Blocks the shortcut path, so it must stay fast.
    fn frontmost_app(&self) -> Option<String> {
        let asn = Command::new("lsappinfo").arg("front").output().ok()?;
        let asn = String::from_utf8_lossy(&asn.stdout).trim().to_string();
        if asn.is_empty() {
            return None;
        }
        let out = Command::new("lsappinfo")
            .args(["info", "-only", "bundleid", &asn])
            .output()
            .ok()?;
        // output looks like: "CFBundleIdentifier"="com.foo.bar"
        let line = String::from_utf8_lossy(&out.stdout);
        let id = line.rsplit('=').next()?.trim().trim_matches('"');
        if id.is_empty() {
            None
        } else {
            Some(id.to_string())
        }
    }

    fn paste(&self, prev: Option<String>) {
        // ponytail: activation is async on macOS, so poll until the target is actually
        // frontmost (bounded) instead of gambling on a fixed delay, then Cmd+V.
        let script = match prev {
            Some(id) => format!(
                "tell application id \"{id}\" to activate\n\
                 repeat 25 times\n\
                   tell application \"System Events\" to if (bundle identifier of first application process whose frontmost is true) is \"{id}\" then exit repeat\n\
                   delay 0.02\n\
                 end repeat\n\
                 tell application \"System Events\" to keystroke \"v\" using command down",
            ),
            None => "delay 0.15\ntell application \"System Events\" to keystroke \"v\" using command down".to_string(),
        };
        std::thread::spawn(move || {
            let _ = Command::new("osascript").args(["-e", &script]).output();
        });
    }

    fn accessibility_granted(&self) -> bool {
        // ponytail: AXIsProcessTrusted is one FFI call, not worth a crate
        unsafe { AXIsProcessTrusted() }
    }

    fn open_accessibility_settings(&self) -> Result<(), String> {
        Command::new("open")
            .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
            .spawn()
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
}

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXIsProcessTrusted() -> bool;
}

fn scan(dir: PathBuf, out: &mut Vec<AppEntry>) {
    if let Ok(entries) = fs::read_dir(&dir) {
        for e in entries.flatten() {
            let p = e.path();
            if p.extension().is_some_and(|x| x == "app") {
                if let Some(name) = p.file_stem().and_then(|s| s.to_str()) {
                    out.push(AppEntry { name: name.to_string(), path: p.to_string_lossy().to_string() });
                }
            }
        }
    }
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
    // fallback: first .icns in Resources
    fs::read_dir(&res)
        .ok()?
        .flatten()
        .map(|e| e.path())
        .find(|p| p.extension().is_some_and(|x| x == "icns"))
}

fn b64(data: &[u8]) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut s = String::with_capacity(data.len().div_ceil(3) * 4);
    for c in data.chunks(3) {
        let b = [c[0], *c.get(1).unwrap_or(&0), *c.get(2).unwrap_or(&0)];
        let n = (b[0] as u32) << 16 | (b[1] as u32) << 8 | b[2] as u32;
        s.push(T[(n >> 18 & 63) as usize] as char);
        s.push(T[(n >> 12 & 63) as usize] as char);
        s.push(if c.len() > 1 { T[(n >> 6 & 63) as usize] as char } else { '=' });
        s.push(if c.len() > 2 { T[(n & 63) as usize] as char } else { '=' });
    }
    s
}
