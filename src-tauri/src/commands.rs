//! All `#[tauri::command]` handlers. Each is a thin adapter that delegates to a
//! platform method or a storage module — no business logic lives here.

use std::str::FromStr;
use std::sync::atomic::Ordering;
use tauri::Manager;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

use crate::clipboard;
use crate::platform::{host, AppEntry, Platform};
use crate::preferences;
use crate::snippets;
use crate::state::{ClipState, ShortcutsState};
use crate::translate::{Provider, Translation};

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
pub fn set_history_limit(
    limit: usize,
    state: tauri::State<ClipState>,
) -> Result<(), String> {
    let limit = limit.clamp(1, 10_000); // guard against 0 / absurd values
    state.limit.store(limit, Ordering::Relaxed);
    let mut list = state.list.lock().unwrap();
    if list.len() > limit {
        list.truncate(limit);
    }
    clipboard::save(&state.dir, &list);

    // persist so the limit survives a restart
    let mut prefs = preferences::load(&state.dir);
    prefs.history_limit = limit;
    preferences::save(&state.dir, &prefs)
}

#[tauri::command]
pub async fn translate(
    text: String,
    from: Option<String>,
    to: Option<String>,
    app: tauri::AppHandle,
) -> Result<Translation, String> {
    let prefs = preferences::load(&app.path().app_config_dir().unwrap());
    let from = from.unwrap_or_else(|| "auto".into());
    let to = to.unwrap_or(prefs.translate_target);
    crate::translate::fetch(prefs.translate_provider, &text, &from, &to).await
}

// Both fields are optional: the language picker sets only the target, the
// Preferences window sets only the engine, and neither clobbers the other.
#[tauri::command]
pub fn set_translate_prefs(
    app: tauri::AppHandle,
    target: Option<String>,
    provider: Option<Provider>,
) -> Result<(), String> {
    let dir = app.path().app_config_dir().unwrap();
    let mut prefs = preferences::load(&dir);
    if let Some(t) = target {
        prefs.translate_target = t;
    }
    if let Some(p) = provider {
        prefs.translate_provider = p;
    }
    preferences::save(&dir, &prefs)
}

#[tauri::command]
pub fn check_accessibility() -> bool {
    host().accessibility_granted()
}

#[tauri::command]
pub fn open_accessibility_settings() -> Result<(), String> {
    host().open_accessibility_settings()
}
