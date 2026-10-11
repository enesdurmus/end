use super::{screenshot, AppEntry, Platform};
use std::process::Command;

mod apps;
mod clipboard;
mod icons;
mod input;
mod search;

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
        Command::new("open")
            .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
            .spawn()
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    fn screenshot_region(&self, save_dir: Option<&std::path::Path>) -> Result<(), String> {
        let Some(dir) = save_dir else {
            // -i: interactive region select, -c: to the clipboard; Esc cancels with exit 0
            Command::new("screencapture").args(["-i", "-c"]).status().map_err(|e| e.to_string())?;
            return Ok(());
        };
        // screencapture can't write a file and the clipboard at once, so capture to the
        // file and copy it ourselves (with -c, a filename is ignored)
        let path = screenshot::new_path(dir)?;
        Command::new("screencapture").args(["-i", "-t", "png"]).arg(&path).status().map_err(|e| e.to_string())?;
        // Esc exits 0 without writing a file
        if !path.exists() {
            return Ok(());
        }
        screenshot::copy_png(&path)
    }
}

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXIsProcessTrusted() -> bool;
}
