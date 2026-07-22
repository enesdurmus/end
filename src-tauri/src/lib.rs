mod clipboard;
mod platform;
mod preferences;
mod snippets;

use platform::{host, AppEntry, Platform};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Mutex;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{Emitter, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

struct ClipState {
    list: Mutex<Vec<String>>,
    dir: PathBuf,
    // bundle id of the app that was frontmost before we stole focus
    prev_app: Mutex<Option<String>>,
}

// live shortcuts + the config dir, shared between the handler and the
// set_shortcut command so both read/write one source of truth
struct ShortcutsState {
    toggle: Mutex<Shortcut>,
    clipboard: Mutex<Shortcut>,
    dir: PathBuf,
}

#[tauri::command]
fn list_apps() -> Vec<AppEntry> {
    host().list_apps()
}

#[tauri::command]
fn clipboard_history(state: tauri::State<ClipState>) -> Vec<String> {
    state.list.lock().unwrap().clone()
}

#[tauri::command]
fn paste_text(text: String, state: tauri::State<ClipState>) -> Result<(), String> {
    let mut cb = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    cb.set_text(text).map_err(|e| e.to_string())?;
    // reactivate the app that was frontmost before us, then paste into it
    let prev = state.prev_app.lock().unwrap().take();
    host().paste(prev);
    Ok(())
}

#[tauri::command]
fn open_path(path: String) -> Result<(), String> {
    host().open_path(&path)
}

#[tauri::command]
fn app_icon(path: String) -> Option<String> {
    host().app_icon(&path)
}

#[tauri::command]
fn search_files(query: String) -> Vec<AppEntry> {
    host().search_files(&query)
}

#[tauri::command]
fn list_snippets(app: tauri::AppHandle) -> Vec<snippets::Snippet> {
    let dir = app.path().app_config_dir().unwrap();
    snippets::load(&dir)
}

#[tauri::command]
fn save_snippets(app: tauri::AppHandle, items: Vec<snippets::Snippet>) -> Result<(), String> {
    let dir = app.path().app_config_dir().unwrap();
    snippets::save(&dir, &items)
}

#[tauri::command]
fn get_preferences(app: tauri::AppHandle) -> preferences::Preferences {
    preferences::load(&app.path().app_config_dir().unwrap())
}

#[tauri::command]
fn set_shortcut(app: tauri::AppHandle, kind: String, accelerator: String) -> Result<(), String> {
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
fn check_accessibility() -> bool {
    host().accessibility_granted()
}

#[tauri::command]
fn open_accessibility_settings() -> Result<(), String> {
    host().open_accessibility_settings()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(move |app, shortcut, event| {
                    if event.state() != ShortcutState::Pressed {
                        return;
                    }
                    let state = app.state::<ShortcutsState>();
                    let is_toggle = *shortcut == *state.toggle.lock().unwrap();
                    let is_clipboard = *shortcut == *state.clipboard.lock().unwrap();
                    let w = app.get_webview_window("main").unwrap();
                    if is_toggle {
                        if w.is_visible().unwrap_or(false) {
                            let _ = w.hide();
                        } else {
                            let _ = w.center();
                            let _ = w.show();
                            let _ = w.set_focus();
                            let _ = w.emit("focus-search", ());
                        }
                    } else if is_clipboard {
                        // remember who was frontmost before we take focus
                        let prev = host().frontmost_app();
                        *app.state::<ClipState>().prev_app.lock().unwrap() = prev;
                        let _ = w.center();
                        let _ = w.show();
                        let _ = w.set_focus();
                        let _ = w.emit("clipboard-mode", ());
                    }
                })
                .build(),
        )
        .setup(move |app| {
            let dir = app.path().app_config_dir().unwrap();
            let prefs = preferences::load(&dir);
            let toggle = Shortcut::from_str(&prefs.toggle_shortcut)?;
            let clipboard_shortcut = Shortcut::from_str(&prefs.clipboard_shortcut)?;
            app.global_shortcut().register(toggle)?;
            app.global_shortcut().register(clipboard_shortcut)?;
            app.manage(ShortcutsState {
                toggle: Mutex::new(toggle),
                clipboard: Mutex::new(clipboard_shortcut),
                dir: dir.clone(),
            });

            let preferences_item = MenuItem::with_id(app, "preferences", "Preferences", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let tray_menu = Menu::with_items(app, &[&preferences_item, &quit_item])?;
            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&tray_menu)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "preferences" => {
                        if let Some(w) = app.get_webview_window("preferences") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .build(app)?;

            let w = app.get_webview_window("main").unwrap();
            let w2 = w.clone();
            w.on_window_event(move |e| {
                if let tauri::WindowEvent::Focused(false) = e {
                    let _ = w2.hide();
                }
            });

            let pref_w = app.get_webview_window("preferences").unwrap();
            let pref_w2 = pref_w.clone();
            pref_w.on_window_event(move |e| {
                if let tauri::WindowEvent::CloseRequested { api, .. } = e {
                    api.prevent_close();
                    let _ = pref_w2.hide();
                }
            });

            let initial = clipboard::load(&dir);
            app.manage(ClipState { list: Mutex::new(initial), dir: dir.clone(), prev_app: Mutex::new(None) });
            let handle = app.handle().clone();
            std::thread::spawn(move || {
                let mut cb = match arboard::Clipboard::new() {
                    Ok(c) => c,
                    Err(_) => return,
                };
                loop {
                    if let Ok(txt) = cb.get_text() {
                        if !txt.is_empty() {
                            let state = handle.state::<ClipState>();
                            let mut list = state.list.lock().unwrap();
                            let before = list.first().cloned();
                            clipboard::push_capped(&mut list, txt, clipboard::CAP);
                            if list.first() != before.as_ref() {
                                clipboard::save(&state.dir, &list);
                            }
                        }
                    }
                    std::thread::sleep(std::time::Duration::from_millis(500));
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_apps,
            app_icon,
            open_path,
            search_files,
            clipboard_history,
            paste_text,
            list_snippets,
            save_snippets,
            get_preferences,
            set_shortcut,
            check_accessibility,
            open_accessibility_settings
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
