use std::str::FromStr;
use std::sync::atomic::Ordering;
use tauri::Manager;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

use crate::clipboard::{self, Clip};
use crate::focus;
use crate::gifs::{self, Gif};
use crate::platform::{host, AppEntry, Platform};
use crate::preferences;
use crate::snippets;
use crate::state::{ClipState, ShortcutsState, TranslateState};
use crate::translate::{Provider, Translation};
use crate::translate_history::{self, Entry};

#[tauri::command]
pub fn list_apps() -> Vec<AppEntry> {
    host().list_apps()
}

#[tauri::command]
pub fn clipboard_history(state: tauri::State<ClipState>) -> Vec<Clip> {
    state.list.lock().unwrap().clone()
}

fn copy_files_still_on_disk(paths: &[String]) -> Result<(), String> {
    let live: Vec<String> =
        paths.iter().filter(|p| std::path::Path::new(p).exists()).cloned().collect();
    if live.is_empty() {
        return Err("those files are no longer on disk".into());
    }
    host().copy_files(&live)
}

/// Images and files go back as file references, not pixels, so pasting into a
/// chat attaches the file the way copying it in Finder would.
#[tauri::command]
pub fn paste_clip(clip: Clip, app: tauri::AppHandle) -> Result<(), String> {
    match &clip {
        Clip::Text { text } => set_clipboard(text.clone())?,
        Clip::Image { path, .. } => copy_files_still_on_disk(std::slice::from_ref(path))?,
        Clip::Files { paths } => copy_files_still_on_disk(paths)?,
    }
    focus::hide_and_paste(&app);
    Ok(())
}

fn set_clipboard(text: String) -> Result<(), String> {
    let mut cb = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    cb.set_text(text).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn paste_text(text: String, app: tauri::AppHandle) -> Result<(), String> {
    set_clipboard(text)?;
    focus::hide_and_paste(&app);
    Ok(())
}

#[tauri::command]
pub fn write_clipboard(text: String, app: tauri::AppHandle) -> Result<(), String> {
    set_clipboard(text)?;
    focus::hide(&app);
    Ok(())
}

#[tauri::command]
pub fn close_launcher(app: tauri::AppHandle) {
    focus::hide(&app);
}

/// Called by the frontend once it has applied the shortcut's mode.
#[tauri::command]
pub fn show_launcher(app: tauri::AppHandle) {
    focus::show(&app);
}

#[tauri::command]
pub fn open_path(path: String, app: tauri::AppHandle) -> Result<(), String> {
    host().open_path(&path)?;
    focus::hide_window(&app); // the opened app takes focus; don't fight it
    Ok(())
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
    // On Wayland the X11 grab is only a fallback and may never have been
    // registered (see shortcuts::register), so its failures are not fatal.
    let grab = |r: Result<(), tauri_plugin_global_shortcut::Error>| match r {
        Err(e) if !crate::shortcuts::x11_grab_is_optional() => Err(e.to_string()),
        Err(e) => {
            eprintln!("x11 shortcut grab failed: {e}");
            Ok(())
        }
        Ok(()) => Ok(()),
    };
    // the old one is already gone if recording paused the grabs
    let _ = app.global_shortcut().unregister(*current);
    grab(app.global_shortcut().register(new_shortcut))?;
    *current = new_shortcut;

    let mut prefs = preferences::load(&state.dir);
    if kind == "toggle" {
        prefs.toggle_shortcut = accelerator;
    } else {
        prefs.clipboard_shortcut = accelerator;
    }
    preferences::save(&state.dir, &prefs)?;

    // the portal binding (the real shortcut on Wayland) is separate from the grab above
    #[cfg(target_os = "linux")]
    if crate::platform::wayland() {
        crate::platform::linux::shortcuts::spawn(app.clone(), state.dir.clone());
    }
    Ok(())
}

/// While Preferences records a new shortcut, the current ones must not fire
/// (or even be grabbed, or the key never reaches the Preferences window).
#[tauri::command]
pub fn pause_shortcuts(app: tauri::AppHandle) {
    let state = app.state::<ShortcutsState>();
    for slot in [&state.toggle, &state.clipboard] {
        let _ = app.global_shortcut().unregister(*slot.lock().unwrap());
    }
    #[cfg(target_os = "linux")]
    if crate::platform::wayland() {
        crate::platform::linux::shortcuts::stop();
    }
}

/// Undoes `pause_shortcuts` from the saved state; holding the slot locks
/// serialises it with a concurrent `set_shortcut`, so the newest value wins.
#[tauri::command]
pub fn resume_shortcuts(app: tauri::AppHandle) {
    let state = app.state::<ShortcutsState>();
    let (toggle, clipboard) = (state.toggle.lock().unwrap(), state.clipboard.lock().unwrap());
    for s in [*toggle, *clipboard] {
        let _ = app.global_shortcut().unregister(s);
        if let Err(e) = app.global_shortcut().register(s) {
            eprintln!("resume shortcut: {e}");
        }
    }
    #[cfg(target_os = "linux")]
    if crate::platform::wayland() {
        crate::platform::linux::shortcuts::spawn(app.clone(), state.dir.clone());
    }
}

#[tauri::command]
pub fn set_history_limit(
    limit: usize,
    state: tauri::State<ClipState>,
) -> Result<(), String> {
    let limit = limit.clamp(1, 10_000);
    state.limit.store(limit, Ordering::Relaxed);
    apply_caps(&state);

    let mut prefs = preferences::load(&state.dir);
    prefs.history_limit = limit;
    preferences::save(&state.dir, &prefs)
}

#[tauri::command]
pub fn set_image_limit(limit: usize, state: tauri::State<ClipState>) -> Result<(), String> {
    let limit = limit.clamp(0, 1_000); // 0 means keep no images at all
    state.image_limit.store(limit, Ordering::Relaxed);
    apply_caps(&state);

    let mut prefs = preferences::load(&state.dir);
    prefs.image_limit = limit;
    preferences::save(&state.dir, &prefs)
}

/// Lowering a limit frees the disk now, rather than at the next copy.
fn apply_caps(state: &ClipState) {
    let text_cap = state.limit.load(Ordering::Relaxed);
    let image_cap = state.image_limit.load(Ordering::Relaxed);
    let mut list = state.list.lock().unwrap();
    let evicted = clipboard::trim(&mut list, text_cap, image_cap);
    clipboard::save(&state.dir, &list);
    drop(list);
    for path in evicted {
        let _ = std::fs::remove_file(path);
    }
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

/// Both fields are optional: the language picker sets only the target, the
/// Preferences window only the engine.
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
pub fn set_gif_prefs(
    klipy_api_key: Option<String>,
    gif_dir: Option<String>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let dir = app.path().app_config_dir().unwrap();
    let mut prefs = preferences::load(&dir);
    if let Some(k) = klipy_api_key {
        prefs.klipy_api_key = k;
    }
    if let Some(d) = gif_dir {
        prefs.gif_dir = d;
        // thumbnails only render from a folder the asset protocol knows about
        let resolved = gifs::dir(&dir, &prefs.gif_dir);
        std::fs::create_dir_all(&resolved).map_err(|e| e.to_string())?;
        app.asset_protocol_scope()
            .allow_directory(&resolved, false)
            .map_err(|e| e.to_string())?;
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

#[tauri::command]
pub fn translate_history(state: tauri::State<TranslateState>) -> Vec<Entry> {
    state.list.lock().unwrap().clone()
}

#[tauri::command]
pub fn record_translation(entry: Entry, state: tauri::State<TranslateState>) -> Result<(), String> {
    let mut list = state.list.lock().unwrap();
    translate_history::push_capped(&mut list, entry, translate_history::HISTORY_LIMIT);
    translate_history::save(&state.dir, &list);
    Ok(())
}

#[tauri::command]
pub fn clear_translate_history(state: tauri::State<TranslateState>) -> Result<(), String> {
    let mut list = state.list.lock().unwrap();
    list.clear();
    translate_history::save(&state.dir, &list);
    Ok(())
}

fn gif_dir(app: &tauri::AppHandle) -> std::path::PathBuf {
    let dir = app.path().app_config_dir().unwrap();
    let prefs = preferences::load(&dir);
    gifs::dir(&dir, &prefs.gif_dir)
}

#[tauri::command]
pub fn gif_library(app: tauri::AppHandle) -> Vec<Gif> {
    gifs::list_local(&gif_dir(&app))
}

#[tauri::command]
pub async fn gif_search(query: String, app: tauri::AppHandle) -> Result<Vec<Gif>, String> {
    let dir = app.path().app_config_dir().unwrap();
    let key = preferences::load(&dir).klipy_api_key;
    gifs::search_klipy(&query, &key).await
}

#[tauri::command]
pub async fn paste_gif(gif: Gif, app: tauri::AppHandle) -> Result<(), String> {
    let path = match gif.source {
        gifs::Source::Local => std::path::PathBuf::from(&gif.url),
        gifs::Source::Remote => gifs::download_temp(&gif.id, &gif.url).await?,
    };
    host().copy_files(&[path.to_string_lossy().into_owned()])?;
    focus::hide_and_paste(&app);
    Ok(())
}

#[tauri::command]
pub async fn favorite_gif(gif: Gif, app: tauri::AppHandle) -> Result<Gif, String> {
    gifs::download(&gif_dir(&app), &gif.url, &gif.title).await
}

#[tauri::command]
pub fn open_gif_dir(app: tauri::AppHandle) -> Result<(), String> {
    let dir = gif_dir(&app);
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    host().open_path(&dir.to_string_lossy())
}
