//! App activation and the synthetic Cmd-V keystroke.

use objc2::rc::Retained;
use objc2_app_kit::{NSApplicationActivationOptions, NSRunningApplication};
use objc2_core_graphics::{CGEvent, CGEventFlags, CGKeyCode};
use objc2_foundation::{NSString, NSThread};
use std::ffi::{c_int, c_ulong, c_void};
use std::process::Command;

const MAX_ASNS_INSPECTED: usize = 5;
const PASTE_CHAR: u16 = b'v' as u16;
const KVK_ANSI_V: CGKeyCode = 9;
const NO_PID: c_int = -1;

pub(super) fn frontmost_app_other_than(own: &str) -> Option<String> {
    // `lsappinfo front` would answer "us" once we are active; this list is
    // front-to-back, and each ASN costs another call
    let out = Command::new("lsappinfo").arg("visibleProcessList").output().ok()?;
    let list = String::from_utf8_lossy(&out.stdout);
    asns(&list).iter().take(MAX_ASNS_INSPECTED).filter_map(|asn| bundle_id(asn)).find(|id| id != own)
}

pub(super) fn app_name(bundle_id: &str) -> Option<String> {
    Some(running(bundle_id)?.localizedName()?.to_string())
}

pub(super) fn restore_focus(prev: Option<String>) {
    if let Some(app) = prev.as_deref().and_then(running) {
        activate(&app);
    }
}

pub(super) fn paste(prev: Option<String>) {
    let Some(app) = prev.as_deref().and_then(running) else {
        return;
    };
    let pid = app.processIdentifier();
    activate(&app);
    if pid != NO_PID {
        send_paste(pid, paste_key());
    }
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

fn running(id: &str) -> Option<Retained<NSRunningApplication>> {
    NSRunningApplication::runningApplicationsWithBundleIdentifier(&NSString::from_str(id))
        .iter()
        .next()
}

/// Only counts while we are still the active app: macOS parks a request from a
/// background process in LaunchServices for a full second.
fn activate(app: &NSRunningApplication) {
    #[allow(deprecated)] // `activate()` alone is macOS 14+
    app.activateWithOptions(NSApplicationActivationOptions::ActivateIgnoringOtherApps);
}

fn send_paste(pid: c_int, key: CGKeyCode) {
    for down in [true, false] {
        if let Some(ev) = CGEvent::new_keyboard_event(None, key, down) {
            CGEvent::set_flags(Some(&ev), CGEventFlags::MaskCommand);
            CGEvent::post_to_pid(pid, Some(&ev));
        }
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

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
            paste_key_in_current_layout().is_some(),
            "not KVK_ANSI_V: that only holds on QWERTY-family layouts"
        );
    }

    #[test]
    fn an_app_that_isnt_running_has_nothing_to_paste_into() {
        assert!(running("com.example.definitely-not-running").is_none());
    }
}
