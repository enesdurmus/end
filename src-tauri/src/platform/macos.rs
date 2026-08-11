use super::{AppEntry, Platform};
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_app_kit::{NSApplicationActivationOptions, NSPasteboard, NSRunningApplication};
use objc2_core_graphics::{CGEvent, CGEventFlags, CGKeyCode};
use objc2_foundation::{NSArray, NSString, NSThread, NSURL};
use std::collections::hash_map::DefaultHasher;
use std::ffi::{c_int, c_ulong, c_void};
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

    // `lsappinfo front` cannot answer this: once the launcher is the active app it just
    // returns us. visibleProcessList is front-to-back activation order, so the answer is
    // the first entry that isn't us.
    //
    // ponytail: lsappinfo (~10ms, no Automation permission) instead of osascript+System
    // Events (~400ms). Blocks the shortcut path, so it must stay fast — hence the take(5),
    // since each ASN costs another call and the answer is realistically 1st or 2nd.
    fn app_behind(&self, own: &str) -> Option<String> {
        let out = Command::new("lsappinfo").arg("visibleProcessList").output().ok()?;
        let list = String::from_utf8_lossy(&out.stdout);
        asns(&list).iter().take(5).filter_map(|asn| bundle_id(asn)).find(|id| id != own)
    }

    fn restore_focus(&self, prev: Option<String>) {
        if let Some(app) = prev.as_deref().and_then(running) {
            request_activation_of(&app);
        }
    }

    fn paste(&self, prev: Option<String>) {
        let Some(app) = prev.as_deref().and_then(running) else {
            return;
        };
        let pid = app.processIdentifier();
        request_activation_of(&app);
        if pid != NO_PID {
            send_paste(pid, paste_key());
        }
    }

    fn copy_file(&self, path: &str) -> Result<(), String> {
        let pb = NSPasteboard::generalPasteboard();
        pb.clearContents();
        let url = NSURL::fileURLWithPath(&NSString::from_str(path));
        let obj = ProtocolObject::from_ref(&*url);
        if pb.writeObjects(&NSArray::from_slice(&[obj])) {
            Ok(())
        } else {
            Err("clipboard rejected the file".into())
        }
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

/// Pulls the ASNs out of a `lsappinfo visibleProcessList` line, front-to-back.
/// Entries look like `ASN:0x0-0x19019-"WezTerm":`.
fn asns(list: &str) -> Vec<String> {
    list.split_whitespace()
        .filter_map(|e| e.split_once("-\"").map(|(asn, _)| asn.to_string()))
        .collect()
}

fn bundle_id(asn: &str) -> Option<String> {
    let out = Command::new("lsappinfo").args(["info", "-only", "bundleid", asn]).output().ok()?;
    // output looks like: "CFBundleIdentifier"="com.foo.bar"
    let line = String::from_utf8_lossy(&out.stdout);
    let id = line.rsplit('=').next()?.trim().trim_matches('"').to_string();
    Some(id).filter(|id| !id.is_empty())
}

const PASTE_CHAR: u16 = b'v' as u16;
const ANSI_PASTE_KEY: CGKeyCode = 9; // kVK_ANSI_V
const NO_PID: c_int = -1;

/// Main thread only: HIToolbox traps the process anywhere else, and only inside a real
/// NSApplication — no test can reach it.
fn paste_key() -> CGKeyCode {
    debug_assert!(NSThread::isMainThread_class(), "paste_key touches HIToolbox; main thread only");
    paste_key_in_current_layout().unwrap_or(ANSI_PASTE_KEY)
}

/// A CGEvent carries a key position, and `v` moves with the layout (Dvorak, Turkish-F).
fn paste_key_in_current_layout() -> Option<CGKeyCode> {
    unsafe {
        let src = TISCopyCurrentKeyboardLayoutInputSource();
        if src.is_null() {
            return None;
        }
        let data = TISGetInputSourceProperty(src, kTISPropertyUnicodeKeyLayoutData);
        let layout = if data.is_null() { std::ptr::null() } else { CFDataGetBytePtr(data) };
        // `layout` is owned by `src`.
        let found = (!layout.is_null())
            .then(|| (0..128).find(|&code| types_paste_char(layout, code)))
            .flatten();
        CFRelease(src);
        found
    }
}

unsafe fn types_paste_char(layout: *const u8, code: CGKeyCode) -> bool {
    const ACTION_DISPLAY: u16 = 3; // kUCKeyActionDisplay
    const NO_DEAD_KEYS: u32 = 1; // 1 << kUCKeyTranslateNoDeadKeysBit
    let mut dead = 0u32;
    let mut len: c_ulong = 0;
    let mut buf = [0u16; 4];
    let status = UCKeyTranslate(
        layout,
        code,
        ACTION_DISPLAY,
        0,
        LMGetKbdType() as u32,
        NO_DEAD_KEYS,
        &mut dead,
        buf.len() as c_ulong,
        &mut len,
        buf.as_mut_ptr(),
    );
    status == 0 && len == 1 && buf[0] == PASTE_CHAR
}

#[link(name = "Carbon", kind = "framework")]
extern "C" {
    fn TISCopyCurrentKeyboardLayoutInputSource() -> *mut c_void;
    fn TISGetInputSourceProperty(source: *mut c_void, key: *const c_void) -> *mut c_void;
    static kTISPropertyUnicodeKeyLayoutData: *const c_void;
    fn LMGetKbdType() -> u8;
    #[allow(clippy::too_many_arguments)]
    fn UCKeyTranslate(
        key_layout: *const u8,
        virtual_key_code: u16,
        key_action: u16,
        modifier_key_state: u32,
        keyboard_type: u32,
        options: u32,
        dead_key_state: *mut u32,
        max_string_length: c_ulong,
        actual_string_length: *mut c_ulong,
        unicode_string: *mut u16,
    ) -> i32;
}

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFDataGetBytePtr(data: *mut c_void) -> *const u8;
    fn CFRelease(cf: *mut c_void);
}

/// Only counts while the launcher is still the active app: macOS parks a request from a
/// background process in LaunchServices for a full second.
fn request_activation_of(app: &NSRunningApplication) {
    #[allow(deprecated)] // `activate()` alone is macOS 14+; we still support older
    app.activateWithOptions(NSApplicationActivationOptions::ActivateIgnoringOtherApps);
}

fn running(id: &str) -> Option<Retained<NSRunningApplication>> {
    NSRunningApplication::runningApplicationsWithBundleIdentifier(&NSString::from_str(id))
        .iter()
        .next()
}

fn send_paste(pid: c_int, key: CGKeyCode) {
    for down in [true, false] {
        if let Some(ev) = CGEvent::new_keyboard_event(None, key, down) {
            CGEvent::set_flags(Some(&ev), CGEventFlags::MaskCommand);
            CGEvent::post_to_pid(pid, Some(&ev));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{asns, MacOs, Platform};

    #[test]
    fn asns_are_parsed_front_to_back() {
        // verbatim `lsappinfo visibleProcessList` output
        let list = "ASN:0x0-0x19019-\"WezTerm\": ASN:0x0-0x53053-\"Firefox\": \
                    ASN:0x0-0x367367-\"Launcher\": ASN:0x0-0x2c02c-\"Finder\":";
        assert_eq!(
            asns(list),
            ["ASN:0x0-0x19019", "ASN:0x0-0x53053", "ASN:0x0-0x367367", "ASN:0x0-0x2c02c"]
        );
    }

    #[test]
    fn unparseable_process_list_yields_no_asns() {
        assert!(asns("").is_empty());
        assert!(asns("garbage without quotes").is_empty());
    }

    /// Not `== ANSI_PASTE_KEY`: that only holds on QWERTY-family layouts.
    #[test]
    fn the_active_layout_yields_a_paste_key() {
        assert!(super::paste_key_in_current_layout().is_some());
    }

    #[test]
    fn an_app_that_isnt_running_has_nothing_to_paste_into() {
        assert!(super::running("com.example.definitely-not-running").is_none());
    }

    // Ignored by default: it overwrites the real clipboard of whoever runs the
    // suite. Run with `cargo test -- --ignored`.
    #[test]
    #[ignore]
    fn copy_file_puts_a_file_url_on_the_pasteboard() {
        use objc2_app_kit::NSPasteboard;
        use objc2_foundation::NSString;

        MacOs.copy_file("/tmp/launcher-copy-file-test.gif").unwrap();

        let pb = NSPasteboard::generalPasteboard();
        let got = pb.stringForType(&NSString::from_str("public.file-url"));
        assert_eq!(
            got.map(|s| s.to_string()).as_deref(),
            Some("file:///tmp/launcher-copy-file-test.gif")
        );
    }
}
