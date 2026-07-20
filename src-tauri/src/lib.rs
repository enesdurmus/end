use tauri::Manager;
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
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
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
