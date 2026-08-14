use std::path::PathBuf;
use std::sync::atomic::AtomicUsize;
use std::sync::Mutex;
use tauri_plugin_global_shortcut::Shortcut;

pub struct ClipState {
    pub list: Mutex<Vec<crate::clipboard::Clip>>,
    pub dir: PathBuf,
    pub limit: AtomicUsize,
    pub image_limit: AtomicUsize,
}

pub struct ShortcutsState {
    pub toggle: Mutex<Shortcut>,
    pub clipboard: Mutex<Shortcut>,
    pub dir: PathBuf,
}

pub struct TranslateState {
    pub list: Mutex<Vec<crate::translate_history::Entry>>,
    pub dir: PathBuf,
}
