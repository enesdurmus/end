use std::error::Error;
use std::str::FromStr;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager, Runtime};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutEvent, ShortcutState};

use crate::focus;
use crate::platform::Platform;
use crate::preferences;
use crate::state::ShortcutsState;

pub fn register<R: Runtime>(
    app: &AppHandle<R>,
    dir: std::path::PathBuf,
) -> Result<ShortcutsState, Box<dyn Error>> {
    let prefs = preferences::load(&dir);
    let toggle = Shortcut::from_str(&prefs.toggle_shortcut)?;
    let clipboard = Shortcut::from_str(&prefs.clipboard_shortcut)?;
    // On Wayland the X11 grab is only a fallback and can legitimately fail
    // (no XWayland, key already taken); the portal below must still get its turn.
    let screenshot = Shortcut::from_str(&prefs.screenshot_shortcut)?;
    for s in [toggle, clipboard, screenshot] {
        if let Err(e) = app.global_shortcut().register(s) {
            if !x11_grab_is_optional() {
                return Err(e.into());
            }
            eprintln!("x11 shortcut grab failed on Wayland: {e}");
        }
    }

    // X11 key grabs (above) don't reach a pure Wayland session; the portal
    // path is the only way a global shortcut fires there. See docs/linux.md.
    #[cfg(target_os = "linux")]
    if crate::platform::wayland() {
        crate::platform::linux::shortcuts::spawn(app.clone(), dir.clone());
    }

    Ok(ShortcutsState {
        toggle: Mutex::new(toggle),
        clipboard: Mutex::new(clipboard),
        screenshot: Mutex::new(screenshot),
        dir,
    })
}

/// True on Wayland, where the portal is the real shortcut path.
pub fn x11_grab_is_optional() -> bool {
    #[cfg(target_os = "linux")]
    return crate::platform::wayland();
    #[cfg(not(target_os = "linux"))]
    false
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
    } else if *shortcut == *state.screenshot.lock().unwrap() {
        "screenshot"
    } else {
        return;
    };
    dispatch(app, id);
}

#[cfg(target_os = "macos")]
fn screenshot_dir<R: Runtime>(app: &AppHandle<R>, configured: &str) -> std::path::PathBuf {
    if !configured.trim().is_empty() {
        return configured.into();
    }
    match app.path().picture_dir() {
        Ok(pictures) => pictures.join("End"),
        Err(_) => app.path().app_config_dir().unwrap().join("screenshots"),
    }
}

#[cfg(target_os = "macos")]
fn prune_screenshots(dir: &std::path::Path, limit: usize) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    // names embed a millisecond timestamp, so sorting by name is sorting by age
    let mut shots: Vec<_> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.starts_with("screenshot-") && n.ends_with(".png"))
        })
        .collect();
    shots.sort();
    let excess = shots.len().saturating_sub(limit);
    for p in &shots[..excess] {
        let _ = std::fs::remove_file(p);
    }
}

/// Shared by the X11 grab handler above and the Wayland portal listener.
pub(crate) fn dispatch<R: Runtime>(app: &AppHandle<R>, id: &str) {
    if id == "screenshot" {
        // blocks until the user has picked a region, so keep it off the caller's thread
        #[cfg(target_os = "macos")]
        let app = app.clone();
        std::thread::spawn(move || {
            #[cfg(target_os = "macos")]
            let prefs = crate::preferences::load(&app.path().app_config_dir().unwrap());
            #[cfg(target_os = "macos")]
            let save = Some(screenshot_dir(&app, &prefs.screenshot_dir));
            #[cfg(not(target_os = "macos"))]
            let save: Option<std::path::PathBuf> = None;
            if let Err(e) = crate::platform::host().screenshot_region(save.as_deref()) {
                eprintln!("screenshot: {e}");
            }
            #[cfg(target_os = "macos")]
            prune_screenshots(save.as_deref().unwrap(), prefs.screenshot_limit);
        });
        return;
    }
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
