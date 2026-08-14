use super::{AppEntry, Platform};
use std::process::Command;

pub struct Windows;

impl Platform for Windows {
    fn list_apps(&self) -> Vec<AppEntry> {
        Vec::new() // TODO: scan Start Menu Programs *.lnk (all-users + per-user)
    }

    fn app_icon(&self, _app_path: &str) -> Option<String> {
        None // TODO: ExtractIconEx / SHGetFileInfo → PNG
    }

    fn open_path(&self, path: &str) -> Result<(), String> {
        let window_title = "";
        Command::new("cmd")
            .args(["/C", "start", window_title, path])
            .spawn()
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    fn search_files(&self, _query: &str) -> Vec<AppEntry> {
        Vec::new() // TODO: Windows Search (ISearchQueryHelper) or Everything SDK
    }

    fn frontmost_app_other_than(&self, _own: &str) -> Option<String> {
        None // TODO: GetForegroundWindow + GetWindowThreadProcessId
    }

    fn restore_focus(&self, _prev: Option<String>) {
        // TODO: SetForegroundWindow(prev)
    }

    fn paste(&self, _prev: Option<String>) {
        // TODO: restore_focus, then SendInput Ctrl+V
    }

    fn accessibility_granted(&self) -> bool {
        true
    }

    fn open_accessibility_settings(&self) -> Result<(), String> {
        Ok(())
    }

    fn copy_files(&self, _paths: &[String]) -> Result<(), String> {
        Ok(()) // TODO: OleSetClipboard with a CF_HDROP drop-file descriptor
    }

    fn clipboard_files(&self) -> Vec<String> {
        Vec::new() // TODO: GetClipboardData(CF_HDROP) + DragQueryFileW
    }

    fn clipboard_source_url(&self) -> Option<String> {
        None // TODO: CF_HTML / CFSTR_INETURL clipboard formats
    }

    fn app_name(&self, _bundle_id: &str) -> Option<String> {
        None // TODO: GetFileVersionInfo FileDescription
    }

    fn clipboard_change_count(&self) -> Option<u64> {
        None // TODO: GetClipboardSequenceNumber
    }
}
