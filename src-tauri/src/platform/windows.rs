use super::{AppEntry, Platform};
use std::process::Command;

pub struct Windows;

// Skeleton impl: `open_path` works today; the rest are stubbed (return empty /
// no-op, never panic) so the app runs. Each TODO names the native mechanism.
impl Platform for Windows {
    fn list_apps(&self) -> Vec<AppEntry> {
        Vec::new() // TODO: scan Start Menu Programs *.lnk (all-users + per-user)
    }

    fn app_icon(&self, _app_path: &str) -> Option<String> {
        None // TODO: ExtractIconEx / SHGetFileInfo → PNG
    }

    fn open_path(&self, path: &str) -> Result<(), String> {
        // empty "" is the window title arg `start` expects before the target
        Command::new("cmd").args(["/C", "start", "", path]).spawn().map(|_| ()).map_err(|e| e.to_string())
    }

    fn search_files(&self, _query: &str) -> Vec<AppEntry> {
        Vec::new() // TODO: Windows Search (ISearchQueryHelper) or Everything SDK
    }

    fn app_behind(&self, _own: &str) -> Option<String> {
        None // TODO: GetForegroundWindow + GetWindowThreadProcessId
    }

    fn restore_focus(&self, _prev: Option<String>) {
        // TODO: SetForegroundWindow(prev)
    }

    fn paste(&self, _prev: Option<String>) {
        // TODO: restore_focus, then SendInput Ctrl+V
    }

    fn accessibility_granted(&self) -> bool {
        true // no equivalent permission gate on Windows
    }

    fn open_accessibility_settings(&self) -> Result<(), String> {
        Ok(())
    }

    fn copy_file(&self, _path: &str) -> Result<(), String> {
        Ok(()) // TODO: OleSetClipboard with a CF_HDROP drop-file descriptor
    }
}
