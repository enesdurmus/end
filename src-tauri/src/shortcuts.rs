use std::error::Error;
use std::str::FromStr;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager, Runtime};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutEvent, ShortcutState};

use crate::focus;
use crate::preferences;
use crate::state::ShortcutsState;

pub fn register<R: Runtime>(
    app: &AppHandle<R>,
    dir: std::path::PathBuf,
) -> Result<ShortcutsState, Box<dyn Error>> {
    let prefs = preferences::load(&dir);
    let toggle = Shortcut::from_str(&prefs.toggle_shortcut)?;
    let clipboard = Shortcut::from_str(&prefs.clipboard_shortcut)?;
    app.global_shortcut().register(toggle)?;
    app.global_shortcut().register(clipboard)?;
    Ok(ShortcutsState {
        toggle: Mutex::new(toggle),
        clipboard: Mutex::new(clipboard),
        dir,
    })
}

pub fn on_press<R: Runtime>(app: &AppHandle<R>, shortcut: &Shortcut, event: ShortcutEvent) {
    if event.state() != ShortcutState::Pressed {
        return;
    }
    let state = app.state::<ShortcutsState>();
    let is_toggle = *shortcut == *state.toggle.lock().unwrap();
    let is_clipboard = *shortcut == *state.clipboard.lock().unwrap();
    let w = app.get_webview_window("main").unwrap();

    if is_toggle {
        if w.is_visible().unwrap_or(false) {
            focus::hide(app);
        } else {
            let _ = w.emit("focus-search", ());
        }
    } else if is_clipboard {
        let _ = w.emit("clipboard-mode", ());
    }
}
