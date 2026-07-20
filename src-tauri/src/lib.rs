mod apps;
mod clipboard;
mod files;

use std::path::PathBuf;
use std::sync::Mutex;
use tauri::Manager;
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

struct ClipState {
    list: Mutex<Vec<String>>,
    dir: PathBuf,
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
fn paste_text(text: String) -> Result<(), String> {
    let mut cb = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    cb.set_text(text).map_err(|e| e.to_string())?;
    // pano set edildikten sonra öndeki uygulamaya Cmd+V bas
    std::process::Command::new("osascript")
        .args(["-e", "tell application \"System Events\" to keystroke \"v\" using command down"])
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let cmd_space = Shortcut::new(Some(Modifiers::SUPER), Code::Space);
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(move |app, shortcut, event| {
                    if shortcut == &cmd_space && event.state() == ShortcutState::Pressed {
                        let w = app.get_webview_window("main").unwrap();
                        if w.is_visible().unwrap_or(false) {
                            let _ = w.hide();
                        } else {
                            let _ = w.center();
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                })
                .build(),
        )
        .setup(move |app| {
            app.global_shortcut().register(cmd_space)?;
            let w = app.get_webview_window("main").unwrap();
            let w2 = w.clone();
            w.on_window_event(move |e| {
                if let tauri::WindowEvent::Focused(false) = e {
                    let _ = w2.hide();
                }
            });

            let dir = app.path().app_config_dir().unwrap();
            let initial = clipboard::load(&dir);
            app.manage(ClipState { list: Mutex::new(initial), dir: dir.clone() });
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
            paste_text
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
