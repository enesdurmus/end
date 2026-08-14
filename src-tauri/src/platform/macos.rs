use super::{base64_encode, AppEntry, Platform};
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

const MAX_SEARCH_RESULTS: usize = 20;
const MAX_ASNS_INSPECTED: usize = 5;

impl Platform for MacOs {
    fn list_apps(&self) -> Vec<AppEntry> {
        let mut out = Vec::new();
        scan_apps(PathBuf::from("/Applications"), &mut out);
        scan_apps(PathBuf::from("/System/Applications"), &mut out);
        scan_apps(PathBuf::from("/System/Applications/Utilities"), &mut out);
        if let Some(home) = std::env::var_os("HOME") {
            scan_apps(PathBuf::from(home).join("Applications"), &mut out);
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
        Some(format!("data:image/png;base64,{}", base64_encode(&bytes)))
    }

    fn open_path(&self, path: &str) -> Result<(), String> {
        Command::new("open").arg(path).spawn().map(|_| ()).map_err(|e| e.to_string())
    }

    fn search_files(&self, query: &str) -> Vec<AppEntry> {
        if query.trim().is_empty() {
            return Vec::new();
        }
        // mdfind can return tens of thousands of lines, so stream and kill it early
        let mut child =
            match Command::new("mdfind").arg("-name").arg(query).stdout(Stdio::piped()).spawn() {
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
            .take(MAX_SEARCH_RESULTS)
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

    fn frontmost_app_other_than(&self, own: &str) -> Option<String> {
        // `lsappinfo front` would answer "us" once we are active; this list is
        // front-to-back, and each ASN costs another call
        let out = Command::new("lsappinfo").arg("visibleProcessList").output().ok()?;
        let list = String::from_utf8_lossy(&out.stdout);
        asns(&list).iter().take(MAX_ASNS_INSPECTED).filter_map(|asn| bundle_id(asn)).find(|id| id != own)
    }

    fn restore_focus(&self, prev: Option<String>) {
        if let Some(app) = prev.as_deref().and_then(running) {
            activate(&app);
        }
    }

    fn paste(&self, prev: Option<String>) {
        let Some(app) = prev.as_deref().and_then(running) else {
            return;
        };
        let pid = app.processIdentifier();
        activate(&app);
        if pid != NO_PID {
            send_paste(pid, paste_key());
        }
    }

    fn copy_files(&self, paths: &[String]) -> Result<(), String> {
        let urls: Vec<Retained<NSURL>> = paths
            .iter()
            .map(|p| NSURL::fileURLWithPath(&NSString::from_str(p)))
            .collect();
        let objs: Vec<_> = urls.iter().map(|u| ProtocolObject::from_ref(&**u)).collect();
        let pb = NSPasteboard::generalPasteboard();
        pb.clearContents();
        if pb.writeObjects(&NSArray::from_slice(&objs)) {
            Ok(())
        } else {
            Err("clipboard rejected the files".into())
        }
    }

    fn clipboard_files(&self) -> Vec<String> {
        let pb = NSPasteboard::generalPasteboard();
        let Some(items) = pb.pasteboardItems() else {
            return Vec::new();
        };
        let ty = NSString::from_str("public.file-url");
        items
            .iter()
            .filter_map(|item| item.stringForType(&ty))
            .filter_map(|s| NSURL::URLWithString(&s))
            .filter_map(|url| url.path().map(|p| p.to_string()))
            .collect()
    }

    fn clipboard_source_url(&self) -> Option<String> {
        let pb = NSPasteboard::generalPasteboard();
        let s = pb.stringForType(&NSString::from_str("public.url"))?.to_string();
        (!s.is_empty()).then_some(s)
    }

    fn app_name(&self, bundle_id: &str) -> Option<String> {
        Some(running(bundle_id)?.localizedName()?.to_string())
    }

    fn clipboard_change_count(&self) -> Option<u64> {
        Some(NSPasteboard::generalPasteboard().changeCount() as u64)
    }

    fn accessibility_granted(&self) -> bool {
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

fn scan_apps(dir: PathBuf, out: &mut Vec<AppEntry>) {
    if let Ok(entries) = fs::read_dir(&dir) {
        for e in entries.flatten() {
            let p = e.path();
            if p.extension().is_some_and(|x| x == "app") {
                if let Some(name) = p.file_stem().and_then(|s| s.to_str()) {
                    out.push(AppEntry {
                        name: name.to_string(),
                        path: p.to_string_lossy().to_string(),
                    });
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
    fs::read_dir(&res)
        .ok()?
        .flatten()
        .map(|e| e.path())
        .find(|p| p.extension().is_some_and(|x| x == "icns"))
}

/// `ASN:0x0-0x19019-"WezTerm":` -> `ASN:0x0-0x19019`
fn asns(list: &str) -> Vec<String> {
    list.split_whitespace()
        .filter_map(|e| e.split_once("-\"").map(|(asn, _)| asn.to_string()))
        .collect()
}

/// `"CFBundleIdentifier"="com.foo.bar"` -> `com.foo.bar`
fn bundle_id(asn: &str) -> Option<String> {
    let out = Command::new("lsappinfo").args(["info", "-only", "bundleid", asn]).output().ok()?;
    let line = String::from_utf8_lossy(&out.stdout);
    let id = line.rsplit('=').next()?.trim().trim_matches('"').to_string();
    Some(id).filter(|id| !id.is_empty())
}

const PASTE_CHAR: u16 = b'v' as u16;
const KVK_ANSI_V: CGKeyCode = 9;
const NO_PID: c_int = -1;

fn paste_key() -> CGKeyCode {
    debug_assert!(NSThread::isMainThread_class(), "paste_key touches HIToolbox; main thread only");
    paste_key_in_current_layout().unwrap_or(KVK_ANSI_V)
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
        let found = (!layout.is_null())
            .then(|| (0..128).find(|&code| types_paste_char(layout, code)))
            .flatten();
        CFRelease(src);
        found
    }
}

unsafe fn types_paste_char(layout: *const u8, code: CGKeyCode) -> bool {
    const KUC_KEY_ACTION_DISPLAY: u16 = 3;
    const KUC_KEY_TRANSLATE_NO_DEAD_KEYS: u32 = 1;
    let mut dead = 0u32;
    let mut len: c_ulong = 0;
    let mut buf = [0u16; 4];
    let status = UCKeyTranslate(
        layout,
        code,
        KUC_KEY_ACTION_DISPLAY,
        0,
        LMGetKbdType() as u32,
        KUC_KEY_TRANSLATE_NO_DEAD_KEYS,
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

/// Only counts while we are still the active app: macOS parks a request from a
/// background process in LaunchServices for a full second.
fn activate(app: &NSRunningApplication) {
    #[allow(deprecated)] // `activate()` alone is macOS 14+
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

    const VISIBLE_PROCESS_LIST: &str = "ASN:0x0-0x19019-\"WezTerm\": ASN:0x0-0x53053-\"Firefox\": \
         ASN:0x0-0x367367-\"Launcher\": ASN:0x0-0x2c02c-\"Finder\":";

    #[test]
    fn asns_are_parsed_front_to_back() {
        assert_eq!(
            asns(VISIBLE_PROCESS_LIST),
            ["ASN:0x0-0x19019", "ASN:0x0-0x53053", "ASN:0x0-0x367367", "ASN:0x0-0x2c02c"]
        );
    }

    #[test]
    fn unparseable_process_list_yields_no_asns() {
        assert!(asns("").is_empty());
        assert!(asns("garbage without quotes").is_empty());
    }

    #[test]
    fn the_active_layout_yields_a_paste_key() {
        assert!(
            super::paste_key_in_current_layout().is_some(),
            "not KVK_ANSI_V: that only holds on QWERTY-family layouts"
        );
    }

    #[test]
    fn an_app_that_isnt_running_has_nothing_to_paste_into() {
        assert!(super::running("com.example.definitely-not-running").is_none());
    }

    /// Overwrites the real clipboard, so it is ignored by default: run with
    /// `cargo test -- --ignored`. One test rather than three, because the general
    /// pasteboard is global state and cargo runs tests in parallel.
    #[test]
    #[ignore]
    fn files_round_trip_through_the_pasteboard() {
        use objc2_app_kit::NSPasteboard;
        use objc2_foundation::NSString;

        MacOs.copy_files(&["/tmp/launcher-copy-file-test.gif".to_string()]).unwrap();
        let pb = NSPasteboard::generalPasteboard();
        let got = pb.stringForType(&NSString::from_str("public.file-url"));
        assert_eq!(
            got.map(|s| s.to_string()).as_deref(),
            Some("file:///tmp/launcher-copy-file-test.gif")
        );

        let paths = vec![
            "/tmp/launcher clip test/a b.txt".to_string(),
            "/tmp/launcher clip test/c.txt".to_string(),
        ];
        let before = MacOs.clipboard_change_count().unwrap();
        MacOs.copy_files(&paths).unwrap();
        assert_eq!(MacOs.clipboard_files(), paths, "percent-escapes must be decoded");
        assert!(MacOs.clipboard_change_count().unwrap() > before);

        arboard::Clipboard::new().unwrap().set_text("just text").unwrap();
        assert!(MacOs.clipboard_files().is_empty(), "plain text is not a file list");
    }
}
