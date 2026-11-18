use super::{AppEntry, Platform};
use std::ffi::c_void;
use std::process::Command;

mod apps;
mod clipboard;
mod icons;
mod input;
mod search;
mod snap;

pub struct MacOs;

impl Platform for MacOs {
    fn list_apps(&self) -> Vec<AppEntry> {
        apps::list_apps()
    }

    fn app_icon(&self, app_path: &str) -> Option<String> {
        icons::app_icon(app_path)
    }

    fn open_path(&self, path: &str) -> Result<(), String> {
        apps::open(path)
    }

    fn search_files(&self, query: &str) -> Vec<AppEntry> {
        search::search_files(query)
    }

    fn frontmost_app_other_than(&self, own: &str) -> Option<String> {
        input::frontmost_app_other_than(own)
    }

    fn restore_focus(&self, prev: Option<String>) {
        input::restore_focus(prev);
    }

    fn paste(&self, prev: Option<String>) {
        input::paste(prev);
    }

    fn copy_files(&self, paths: &[String]) -> Result<(), String> {
        clipboard::copy_files(paths)
    }

    fn clipboard_files(&self) -> Vec<String> {
        clipboard::files()
    }

    fn clipboard_source_url(&self) -> Option<String> {
        clipboard::source_url()
    }

    fn app_name(&self, bundle_id: &str) -> Option<String> {
        input::app_name(bundle_id)
    }

    fn clipboard_change_count(&self) -> Option<u64> {
        clipboard::change_count()
    }

    fn accessibility_granted(&self) -> bool {
        unsafe { AXIsProcessTrusted() }
    }

    fn open_accessibility_settings(&self) -> Result<(), String> {
        request_accessibility();
        Command::new("open")
            .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
            .spawn()
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    fn screenshot_region(&self) -> Result<(), String> {
        // -i: interactive region select, -c: to the clipboard; Esc cancels with exit 0
        Command::new("screencapture").args(["-i", "-c"]).status().map_err(|e| e.to_string())?;
        Ok(())
    }

    fn place_window(&self, action: crate::snap::Action) -> Result<(), String> {
        snap::place(action)
    }

    fn setup_window_management(&self) -> Option<String> {
        (!self.accessibility_granted()).then(|| "Grant the Accessibility permission below, window snapping needs it.".into())
    }
}

fn request_accessibility() {
    unsafe {
        let keys = [kAXTrustedCheckOptionPrompt];
        let values = [kCFBooleanTrue];
        let options = CFDictionaryCreate(
            std::ptr::null(),
            keys.as_ptr(),
            values.as_ptr(),
            1,
            &raw const kCFTypeDictionaryKeyCallBacks,
            &raw const kCFTypeDictionaryValueCallBacks,
        );
        if options.is_null() {
            return;
        }
        AXIsProcessTrustedWithOptions(options);
        CFRelease(options);
    }
}

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXIsProcessTrusted() -> bool;
    fn AXIsProcessTrustedWithOptions(options: *const c_void) -> bool;
    static kAXTrustedCheckOptionPrompt: *const c_void;
}

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    static kCFBooleanTrue: *const c_void;
    static kCFTypeDictionaryKeyCallBacks: c_void;
    static kCFTypeDictionaryValueCallBacks: c_void;
    fn CFDictionaryCreate(
        allocator: *const c_void,
        keys: *const *const c_void,
        values: *const *const c_void,
        num_values: isize,
        key_callbacks: *const c_void,
        value_callbacks: *const c_void,
    ) -> *mut c_void;
    fn CFRelease(cf: *mut c_void);
}
