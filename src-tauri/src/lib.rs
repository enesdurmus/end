mod apps;
mod clipboard;
mod files;
mod snippets;

use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{Emitter, Manager};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

struct ClipState {
    list: Mutex<Vec<String>>,
    dir: PathBuf,
    // bundle id of the app that was frontmost before we stole focus
    prev_app: Mutex<Option<String>>,
}

// ponytail: osascript is the cheapest way to read/set frontmost app on macOS
fn frontmost_bundle_id() -> Option<String> {
    let out = std::process::Command::new("osascript")
        .args(["-e", "tell application \"System Events\" to get bundle identifier of first application process whose frontmost is true"])
        .output()
        .ok()?;
    let id = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if id.is_empty() { None } else { Some(id) }
}

#[tauri::command]
fn list_apps() -> Vec<apps::AppEntry> {
    apps::list()
}

#[tauri::command]
fn clipboard_history(state: tauri::State<ClipState>) -> Vec<String> {
    state.list.lock().unwrap().clone()
}

#[tauri::command]
fn paste_text(text: String, state: tauri::State<ClipState>) -> Result<(), String> {
    let mut cb = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    cb.set_text(text).map_err(|e| e.to_string())?;
    // reactivate the app that was frontmost before us, then Cmd+V into it
    let prev = state.prev_app.lock().unwrap().take();
    // ponytail: activation is async on macOS, so poll until the target is actually
    // frontmost (bounded) instead of gambling on a fixed delay, then Cmd+V.
    let script = match prev {
        Some(id) => format!(
            "tell application id \"{id}\" to activate\n\
             repeat 25 times\n\
               tell application \"System Events\" to if bundle identifier of first application process whose frontmost is true is \"{id}\" then exit repeat\n\
               delay 0.02\n\
             end repeat\n\
             tell application \"System Events\" to keystroke \"v\" using command down",
        ),
        None => "delay 0.15\ntell application \"System Events\" to keystroke \"v\" using command down".to_string(),
    };
    std::process::Command::new("osascript")
        .args(["-e", &script])
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn open_path(path: String) -> Result<(), String> {
    std::process::Command::new("open").arg(&path).spawn().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn search_files(query: String) -> Vec<apps::AppEntry> {
    files::search(&query)
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let cmd_space = Shortcut::new(Some(Modifiers::SUPER), Code::Space);
    let cmd_shift_v = Shortcut::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::KeyV);
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(move |app, shortcut, event| {
                    if event.state() != ShortcutState::Pressed {
                        return;
                    }
                    let w = app.get_webview_window("main").unwrap();
                    if shortcut == &cmd_space {
                        if w.is_visible().unwrap_or(false) {
                            let _ = w.hide();
                        } else {
                            let _ = w.center();
                            let _ = w.show();
                            let _ = w.set_focus();
                            let _ = w.emit("focus-search", ());
                        }
                    } else if shortcut == &cmd_shift_v {
                        // remember who was frontmost before we take focus
                        let prev = frontmost_bundle_id();
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
            app.global_shortcut().register(cmd_space)?;
            app.global_shortcut().register(cmd_shift_v)?;
            let w = app.get_webview_window("main").unwrap();
            let w2 = w.clone();
            w.on_window_event(move |e| {
                if let tauri::WindowEvent::Focused(false) = e {
                    let _ = w2.hide();
                }
            });

            let dir = app.path().app_config_dir().unwrap();
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
            open_path,
            search_files,
            clipboard_history,
            paste_text,
            list_snippets,
            save_snippets
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
