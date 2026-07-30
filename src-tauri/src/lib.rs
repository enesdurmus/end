mod clipboard;
mod commands;
mod platform;
mod preferences;
mod shortcuts;
mod snippets;
mod state;
mod tray;
mod watcher;
mod windows;

use std::sync::Mutex;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(shortcuts::on_press)
                .build(),
        )
        .setup(|app| {
            let dir = app.path().app_config_dir().unwrap();
            let handle = app.handle();

            app.manage(shortcuts::register(handle, dir.clone())?);
            tray::build(handle)?;
            windows::wire(handle);

            let initial = clipboard::load(&dir);
            app.manage(state::ClipState {
                list: Mutex::new(initial),
                dir: dir.clone(),
                prev_app: Mutex::new(None),
            });
            watcher::spawn(handle.clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_apps,
            commands::app_icon,
            commands::open_path,
            commands::search_files,
            commands::clipboard_history,
            commands::paste_text,
            commands::list_snippets,
            commands::save_snippets,
            commands::get_preferences,
            commands::set_shortcut,
            commands::check_accessibility,
            commands::open_accessibility_settings
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
