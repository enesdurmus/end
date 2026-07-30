//! All `#[tauri::command]` handlers. Each is a thin adapter that delegates to a
//! platform method or a storage module — no business logic lives here.

use std::str::FromStr;
use tauri::Manager;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

use crate::platform::{host, AppEntry, Platform};
use crate::preferences;
use crate::snippets;
use crate::state::{ClipState, ShortcutsState};

#[tauri::command]
pub fn list_apps() -> Vec<AppEntry> {
    host().list_apps()
}

#[tauri::command]
pub fn clipboard_history(state: tauri::State<ClipState>) -> Vec<String> {
    state.list.lock().unwrap().clone()
}

#[tauri::command]
pub fn paste_text(text: String, state: tauri::State<ClipState>) -> Result<(), String> {
    let mut cb = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    cb.set_text(text).map_err(|e| e.to_string())?;
    // reactivate the app that was frontmost before us, then paste into it
    let prev = state.prev_app.lock().unwrap().take();
    host().paste(prev);
    Ok(())
}

#[tauri::command]
pub fn open_path(path: String) -> Result<(), String> {
    host().open_path(&path)
}

#[tauri::command]
pub fn app_icon(path: String) -> Option<String> {
    host().app_icon(&path)
}

#[tauri::command]
pub fn search_files(query: String) -> Vec<AppEntry> {
    host().search_files(&query)
}

#[tauri::command]
pub fn list_snippets(app: tauri::AppHandle) -> Vec<snippets::Snippet> {
    let dir = app.path().app_config_dir().unwrap();
    snippets::load(&dir)
}

#[tauri::command]
pub fn save_snippets(app: tauri::AppHandle, items: Vec<snippets::Snippet>) -> Result<(), String> {
    let dir = app.path().app_config_dir().unwrap();
    snippets::save(&dir, &items)
}

#[tauri::command]
pub fn get_preferences(app: tauri::AppHandle) -> preferences::Preferences {
    preferences::load(&app.path().app_config_dir().unwrap())
}

#[tauri::command]
pub fn set_shortcut(app: tauri::AppHandle, kind: String, accelerator: String) -> Result<(), String> {
    let new_shortcut = Shortcut::from_str(&accelerator).map_err(|e| e.to_string())?;
    let state = app.state::<ShortcutsState>();
    let slot = match kind.as_str() {
        "toggle" => &state.toggle,
        "clipboard" => &state.clipboard,
        _ => return Err(format!("unknown shortcut kind: {kind}")),
    };
    let mut current = slot.lock().unwrap();
    app.global_shortcut().unregister(*current).map_err(|e| e.to_string())?;
    app.global_shortcut().register(new_shortcut).map_err(|e| e.to_string())?;
    *current = new_shortcut;

    let mut prefs = preferences::load(&state.dir);
    if kind == "toggle" {
        prefs.toggle_shortcut = accelerator;
    } else {
        prefs.clipboard_shortcut = accelerator;
    }
    preferences::save(&state.dir, &prefs)
}

#[tauri::command]
pub fn check_accessibility() -> bool {
    host().accessibility_granted()
}

#[tauri::command]
pub fn open_accessibility_settings() -> Result<(), String> {
    host().open_accessibility_settings()
}
