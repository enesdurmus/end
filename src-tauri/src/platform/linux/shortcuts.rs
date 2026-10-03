//! Wayland-native global shortcuts via the xdg-desktop-portal `GlobalShortcuts`
//! interface. X11 sessions keep `tauri-plugin-global-shortcut`'s key grabs.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;

use ashpd::desktop::global_shortcuts::{GlobalShortcuts, NewShortcut};
use futures_util::StreamExt;
use tauri::{AppHandle, Runtime};

use crate::{preferences, shortcuts};

const BIND_TIMEOUT: Duration = Duration::from_secs(60);
/// Lets a burst of spawn() calls (save + resume) collapse into the last one
/// before anything reaches the compositor and a dialog opens.
const SETTLE: Duration = Duration::from_millis(400);
/// How often a superseded session notices it should close.
const STALE_CHECK: Duration = Duration::from_millis(250);

/// Bumped by every `spawn`; a running session whose number is no longer current
/// closes itself.
static GENERATION: AtomicU64 = AtomicU64::new(0);

fn stale(generation: u64) -> bool {
    GENERATION.load(Ordering::SeqCst) != generation
}

/// Closes the running session (and so releases the compositor's key grab)
/// without opening a new one.
pub(crate) fn stop() {
    GENERATION.fetch_add(1, Ordering::SeqCst);
}

/// Returns immediately; registration failures (no portal, user declines the
/// bind dialog) are logged, never fatal. Calling it again (shortcut changed)
/// replaces the previous session.
pub(crate) fn spawn<R: Runtime>(app: AppHandle<R>, dir: PathBuf) {
    let generation = GENERATION.fetch_add(1, Ordering::SeqCst) + 1;
    tauri::async_runtime::spawn(async move {
        if let Err(e) = run(&app, &dir, generation).await {
            eprintln!("wayland global shortcuts: {e}");
        }
    });
}

async fn run<R: Runtime>(
    app: &AppHandle<R>,
    dir: &std::path::Path,
    generation: u64,
) -> ashpd::Result<()> {
    tokio::time::sleep(SETTLE).await;
    if stale(generation) {
        return Ok(());
    }
    register_app_id(app).await;

    let prefs = preferences::load(dir);
    let portal = GlobalShortcuts::new().await?;
    let session = portal.create_session(Default::default()).await?;
    // The compositor remembers a binding under its id and treats the preferred
    // trigger as a first-bind hint only, so an id that stayed the same would keep
    // the old key forever. The id therefore carries the trigger ("toggle@LOGO+space"):
    // an unchanged shortcut keeps its grant, a changed one is bound afresh.
    let toggle = preferred_trigger(&prefs.toggle_shortcut);
    let clipboard = preferred_trigger(&prefs.clipboard_shortcut);
    let id = |name: &str, t: &Option<String>| format!("{name}@{}", t.as_deref().unwrap_or(""));
    let bindings = [
        NewShortcut::new(id("toggle", &toggle), "Toggle launcher")
            .preferred_trigger(toggle.as_deref()),
        NewShortcut::new(id("clipboard", &clipboard), "Open clipboard history")
            .preferred_trigger(clipboard.as_deref()),
    ];
    // A consent dialog the user never saw never answers, and this call waits
    // for it - give up rather than hang with nothing bound and nothing logged.
    // Meanwhile a newer spawn() cancels this one, which also dismisses its dialog.
    let mut bind = Box::pin(portal.bind_shortcuts(&session, &bindings, None, Default::default()));
    let started = std::time::Instant::now();
    let request = loop {
        match tokio::time::timeout(STALE_CHECK, &mut bind).await {
            Ok(r) => break r?,
            Err(_) if stale(generation) => {
                drop(bind);
                return session.close().await;
            }
            Err(_) if started.elapsed() >= BIND_TIMEOUT => {
                eprintln!(
                    "wayland global shortcuts: the compositor never answered the bind request \
                     (consent dialog dismissed?); no global shortcut is active this run"
                );
                return Ok(());
            }
            Err(_) => {}
        }
    };
    // The compositor picks the final combo, so log what it settled on.
    for s in request.response()?.shortcuts() {
        println!("wayland shortcut: {} bound to {}", s.id(), s.trigger_description());
    }

    let mut activated = portal.receive_activated().await?;
    loop {
        if GENERATION.load(Ordering::SeqCst) != generation {
            return session.close().await;
        }
        // wake up periodically only to notice that a newer session replaced us
        let Ok(next) = tokio::time::timeout(STALE_CHECK, activated.next()).await else { continue };
        let Some(event) = next else { return Ok(()) };
        if GENERATION.load(Ordering::SeqCst) != generation {
            return session.close().await;
        }
        // Without the compositor's token, Mutter/KWin refuse to focus a window
        // that a global shortcut just showed (focus-stealing prevention).
        if let Some(t) = event.options().get("activation_token") {
            if let Ok(t) = String::try_from(t.clone()) {
                crate::focus::set_activation_token(t);
            }
        }
        let name = event.shortcut_id().split('@').next().unwrap_or_default();
        shortcuts::dispatch(app, name);
    }
}

/// Portals key their stored permissions by app id, and for a non-Flatpak app the
/// only way to give them one is the Registry. It must go over the same D-Bus
/// connection the shortcut calls use (ashpd's shared one), be a valid reverse-DNS
/// id, happen once, and name an installed `<id>.desktop` file - without that the
/// compositor files our shortcuts under whatever terminal launched us. Best-effort.
async fn register_app_id<R: Runtime>(app: &AppHandle<R>) {
    static DONE: AtomicBool = AtomicBool::new(false);
    if DONE.swap(true, Ordering::SeqCst) {
        return;
    }
    let identifier = app.config().identifier.as_str();
    if let Err(e) = ensure_desktop_file(identifier, &app.package_info().name) {
        eprintln!("wayland global shortcuts: could not write {identifier}.desktop: {e}");
    }
    match ashpd::AppID::try_from(identifier) {
        Ok(id) => {
            if let Err(e) = ashpd::register_host_app(id).await {
                eprintln!("wayland global shortcuts: app id registration failed: {e}");
            }
        }
        Err(e) => eprintln!("wayland global shortcuts: identifier is not a valid app id: {e}"),
    }
}

/// A hidden per-user launcher entry for the running binary, rewritten when the
/// binary moves (dev build vs. installed). Packaged installs ship their own.
fn ensure_desktop_file(identifier: &str, name: &str) -> std::io::Result<()> {
    let dir = super::env_dir("XDG_DATA_HOME", super::home().join(".local/share")).join("applications");
    let exe = std::env::current_exe()?;
    let entry = format!(
        "[Desktop Entry]\nType=Application\nName={name}\nExec={}\nNoDisplay=true\nTerminal=false\n",
        exe.display()
    );
    let path = dir.join(format!("{identifier}.desktop"));
    if std::fs::read_to_string(&path).is_ok_and(|old| old == entry) {
        return Ok(());
    }
    std::fs::create_dir_all(&dir)?;
    std::fs::write(path, entry)
}

/// Shortcut preference -> XDG "shortcuts" syntax: "+"-joined uppercase modifier
/// names and an xkbcommon keysym name.
// ponytail: covers the keys the preferences UI can produce; an unmapped key
// just drops the hint and lets the compositor ask the user.
fn preferred_trigger(shortcut: &str) -> Option<String> {
    let mut parts = shortcut.split('+').peekable();
    let mut out: Vec<String> = Vec::new();
    while let Some(part) = parts.next() {
        if parts.peek().is_none() {
            let key = match part {
                "Space" => "space".to_string(),
                "Enter" => "Return".to_string(),
                "Escape" => "Escape".to_string(),
                "Tab" => "Tab".to_string(),
                "Backspace" => "BackSpace".to_string(),
                _ if part.starts_with("Key") && part.len() == 4 => part[3..].to_lowercase(),
                _ if part.starts_with("Digit") && part.len() == 6 => part[5..].to_string(),
                _ if part.starts_with('F') && part[1..].parse::<u8>().is_ok() => part.to_string(),
                _ => return None,
            };
            out.push(key);
        } else {
            let modifier = match part {
                "Super" | "Meta" => "LOGO",
                "Shift" => "SHIFT",
                "Alt" => "ALT",
                "Control" | "Ctrl" | "CmdOrCtrl" => "CTRL",
                _ => return None,
            };
            out.push(modifier.to_string());
        }
    }
    Some(out.join("+"))
}

#[cfg(test)]
mod tests {
    use super::preferred_trigger;

    #[test]
    fn maps_super_space() {
        assert_eq!(preferred_trigger("Super+Space"), Some("LOGO+space".into()));
    }

    #[test]
    fn maps_key_letter_with_modifiers() {
        assert_eq!(preferred_trigger("Super+Shift+KeyV"), Some("LOGO+SHIFT+v".into()));
    }

    #[test]
    fn maps_digit_and_function_keys() {
        assert_eq!(preferred_trigger("Control+Digit1"), Some("CTRL+1".into()));
        assert_eq!(preferred_trigger("Alt+F1"), Some("ALT+F1".into()));
    }

    #[test]
    fn maps_named_keys_to_their_keysym_names() {
        assert_eq!(preferred_trigger("Shift+Tab"), Some("SHIFT+Tab".into()));
        assert_eq!(preferred_trigger("Control+Enter"), Some("CTRL+Return".into()));
    }

    #[test]
    fn returns_none_for_unmappable_key() {
        assert_eq!(preferred_trigger("Super+Unknown123"), None);
    }
}
