use std::path::PathBuf;
use std::sync::atomic::AtomicUsize;
use std::sync::Mutex;
use tauri_plugin_global_shortcut::Shortcut;

/// Clipboard history plus the focus we need to restore after a paste.
pub struct ClipState {
    pub list: Mutex<Vec<String>>,
    pub dir: PathBuf,
    // bundle id of the app that was frontmost before we stole focus
    pub prev_app: Mutex<Option<String>>,
    // max entries kept; user-editable, read lock-free by the watcher thread
    pub limit: AtomicUsize,
}

/// Live shortcuts + the config dir, shared between the press handler and the
/// `set_shortcut` command so both read/write one source of truth.
pub struct ShortcutsState {
    pub toggle: Mutex<Shortcut>,
    pub clipboard: Mutex<Shortcut>,
    pub dir: PathBuf,
}

/// Persisted translation history, loaded once at startup.
pub struct TranslateState {
    pub list: Mutex<Vec<crate::translate_history::Entry>>,
    pub dir: PathBuf,
}
