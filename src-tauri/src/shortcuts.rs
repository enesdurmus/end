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

    // X11 key grabs (above) don't reach a pure Wayland session; the portal
    // path is the only way a global shortcut fires there. See docs/linux.md.
    #[cfg(target_os = "linux")]
    if crate::platform::wayland() {
        crate::platform::linux::shortcuts::spawn(app.clone(), dir.clone());
    }

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
    let id = if *shortcut == *state.toggle.lock().unwrap() {
        "toggle"
    } else if *shortcut == *state.clipboard.lock().unwrap() {
        "clipboard"
    } else {
        return;
    };
    dispatch(app, id);
}

/// Shared by the X11 grab handler above and the Wayland portal listener.
pub(crate) fn dispatch<R: Runtime>(app: &AppHandle<R>, id: &str) {
    let w = app.get_webview_window("main").unwrap();
    match id {
        "toggle" => {
            if w.is_visible().unwrap_or(false) {
                focus::hide(app);
            } else {
                let _ = w.emit("focus-search", ());
            }
        }
        "clipboard" => {
            let _ = w.emit("clipboard-mode", ());
        }
        _ => {}
    }
}
