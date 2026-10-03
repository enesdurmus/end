use super::{AppEntry, Platform};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::OnceLock;

mod clipboard;
mod desktop;
mod icons;
mod input;
mod search;
pub(crate) mod shortcuts;

pub struct Linux;

impl Platform for Linux {
    fn list_apps(&self) -> Vec<AppEntry> {
        desktop::list_apps()
    }

    fn app_icon(&self, app_path: &str) -> Option<String> {
        icons::app_icon(app_path)
    }

    fn open_path(&self, path: &str) -> Result<(), String> {
        desktop::open(path)
    }

    fn search_files(&self, query: &str) -> Vec<AppEntry> {
        search::search_files(query)
    }

    fn frontmost_app_other_than(&self, _own: &str) -> Option<String> {
        input::frontmost_window()
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

    fn app_name(&self, window_id: &str) -> Option<String> {
        input::window_class(window_id)
    }

    fn clipboard_change_count(&self) -> Option<u64> {
        clipboard::change_count()
    }

    fn accessibility_granted(&self) -> bool {
        input::can_paste()
    }

    fn open_accessibility_settings(&self) -> Result<(), String> {
        // On Wayland the portal asks the user itself, so just trigger its dialog.
        if wayland() {
            return tauri::async_runtime::block_on(input::portal::ensure());
        }
        Err(format!(
            "no keystroke tool found. Install {} to enable pasting.",
            if wayland() { "wtype or ydotool" } else { "xdotool" }
        ))
    }
}

pub(crate) fn wayland() -> bool {
    static W: OnceLock<bool> = OnceLock::new();
    *W.get_or_init(|| std::env::var_os("WAYLAND_DISPLAY").is_some())
}

pub(super) fn home() -> PathBuf {
    std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default()
}

pub(super) fn env_dir(key: &str, default: PathBuf) -> PathBuf {
    match std::env::var_os(key) {
        Some(v) if !v.is_empty() => PathBuf::from(v),
        _ => default,
    }
}

pub(super) fn xdg_data_dirs_in_lookup_order() -> Vec<PathBuf> {
    let mut dirs = vec![env_dir("XDG_DATA_HOME", home().join(".local/share"))];
    let sys = std::env::var("XDG_DATA_DIRS").unwrap_or_default();
    let sys = if sys.is_empty() { "/usr/local/share:/usr/share".to_string() } else { sys };
    dirs.extend(sys.split(':').filter(|s| !s.is_empty()).map(PathBuf::from));
    dirs
}

pub(super) fn on_path(bin: &str) -> bool {
    std::env::var_os("PATH")
        .map(|path| std::env::split_paths(&path).any(|d| d.join(bin).is_file()))
        .unwrap_or(false)
}

pub(super) fn spawn_detached(cmd: &mut Command) -> Result<(), String> {
    let mut child =
        cmd.stdout(Stdio::null()).stderr(Stdio::null()).spawn().map_err(|e| e.to_string())?;
    std::thread::spawn(move || {
        let _ = child.wait();
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_tools_never_panic() {
        assert!(!on_path("definitely-not-a-real-binary-name"));
        assert!(Linux.search_files("   ").is_empty());
        assert!(Linux.app_icon("/nonexistent.desktop").is_none());
    }
}

