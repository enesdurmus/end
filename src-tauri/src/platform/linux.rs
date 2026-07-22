use super::{AppEntry, Platform};
use std::process::Command;

pub struct Linux;

// Skeleton impl: `open_path` works today; the rest are stubbed (return empty /
// no-op, never panic) so the app runs. Each TODO names the native mechanism.
impl Platform for Linux {
    fn list_apps(&self) -> Vec<AppEntry> {
        Vec::new() // TODO: parse /usr/share/applications and ~/.local/share/applications *.desktop
    }

    fn app_icon(&self, _app_path: &str) -> Option<String> {
        None // TODO: freedesktop icon-theme lookup for the .desktop's Icon= key
    }

    fn open_path(&self, path: &str) -> Result<(), String> {
        Command::new("xdg-open").arg(path).spawn().map(|_| ()).map_err(|e| e.to_string())
    }

    fn search_files(&self, _query: &str) -> Vec<AppEntry> {
        Vec::new() // TODO: plocate/locate, or a walk of common dirs
    }

    fn frontmost_app(&self) -> Option<String> {
        None // TODO: xdotool getactivewindow (X11) / no portable Wayland equivalent
    }

    fn paste(&self, _prev: Option<String>) {
        // TODO: ydotool/xdotool key ctrl+v after refocusing prev
    }

    fn accessibility_granted(&self) -> bool {
        true // X11/Wayland have no equivalent permission gate
    }

    fn open_accessibility_settings(&self) -> Result<(), String> {
        Ok(())
    }
}
