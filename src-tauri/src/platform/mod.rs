use serde::Serialize;

/// A launchable/openable entry (an app or a file) surfaced to the UI.
#[derive(Serialize, Clone)]
pub struct AppEntry {
    pub name: String,
    pub path: String,
}

/// Everything the launcher needs from the host OS.
///
/// One implementation per platform, selected at compile time below. Adding a
/// new OS means adding a module with a `impl Platform` — no call site changes.
pub trait Platform: Send + Sync {
    /// Installed applications, sorted for display.
    fn list_apps(&self) -> Vec<AppEntry>;
    /// App icon as a `data:image/png;base64,...` URI, if one can be resolved.
    fn app_icon(&self, app_path: &str) -> Option<String>;
    /// Open a file / app / URL with the OS default handler.
    fn open_path(&self, path: &str) -> Result<(), String>;
    /// Search the system file index by name.
    fn search_files(&self, query: &str) -> Vec<AppEntry>;
    /// The frontmost app that isn't `own`, i.e. where focus should go when we step
    /// aside. Excluding ourselves is the point: by the time the launcher is asking,
    /// it may already be the active app.
    fn app_behind(&self, own: &str) -> Option<String>;
    /// Bring `prev` (if any) back to the front.
    fn restore_focus(&self, prev: Option<String>);
    /// Bring `prev` back to the front, then synthesize the paste keystroke.
    /// Clipboard contents are set by the caller beforehand.
    fn paste(&self, prev: Option<String>);
    /// Whether the OS has granted the input-synthesis / accessibility permission.
    fn accessibility_granted(&self) -> bool;
    /// Open the OS settings pane where the user grants that permission.
    fn open_accessibility_settings(&self) -> Result<(), String>;
}

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::MacOs as Host;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::Linux as Host;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use windows::Windows as Host;

/// The platform backend for the current target. Unit struct — zero cost.
pub fn host() -> Host {
    Host
}
