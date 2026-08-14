use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct AppEntry {
    pub name: String,
    pub path: String,
}

pub trait Platform: Send + Sync {
    fn list_apps(&self) -> Vec<AppEntry>;
    fn app_icon(&self, app_path: &str) -> Option<String>;
    fn open_path(&self, path: &str) -> Result<(), String>;
    fn search_files(&self, query: &str) -> Vec<AppEntry>;
    fn frontmost_app_other_than(&self, own: &str) -> Option<String>;
    fn restore_focus(&self, prev: Option<String>);
    fn paste(&self, prev: Option<String>);
    fn copy_files(&self, paths: &[String]) -> Result<(), String>;
    fn clipboard_files(&self) -> Vec<String>;
    fn clipboard_source_url(&self) -> Option<String>;
    fn app_name(&self, bundle_id: &str) -> Option<String>;
    fn clipboard_change_count(&self) -> Option<u64>;
    fn accessibility_granted(&self) -> bool;
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

pub fn host() -> Host {
    Host
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
pub(crate) fn base64_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut s = String::with_capacity(data.len().div_ceil(3) * 4);
    for c in data.chunks(3) {
        let b = [c[0], *c.get(1).unwrap_or(&0), *c.get(2).unwrap_or(&0)];
        let n = (b[0] as u32) << 16 | (b[1] as u32) << 8 | b[2] as u32;
        s.push(ALPHABET[(n >> 18 & 63) as usize] as char);
        s.push(ALPHABET[(n >> 12 & 63) as usize] as char);
        s.push(if c.len() > 1 { ALPHABET[(n >> 6 & 63) as usize] as char } else { '=' });
        s.push(if c.len() > 2 { ALPHABET[(n & 63) as usize] as char } else { '=' });
    }
    s
}
