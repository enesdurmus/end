mod apps;
mod files;

use tauri::Manager;
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

#[tauri::command]
fn list_apps() -> Vec<apps::AppEntry> {
    apps::list()
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
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![list_apps, open_path, search_files])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
