//! Wayland-native global shortcuts via the xdg-desktop-portal `GlobalShortcuts`
//! interface. X11 sessions keep `tauri-plugin-global-shortcut`'s key grabs.

use std::path::PathBuf;
use std::time::Duration;

use ashpd::desktop::global_shortcuts::{GlobalShortcuts, NewShortcut};
use ashpd::zbus::zvariant::Value;
use futures_util::StreamExt;
use tauri::{AppHandle, Runtime};

use crate::{preferences, shortcuts};

const BIND_TIMEOUT: Duration = Duration::from_secs(60);

/// Returns immediately; registration failures (no portal, user declines the
/// bind dialog) are logged, never fatal.
pub(crate) fn spawn<R: Runtime>(app: AppHandle<R>, dir: PathBuf) {
    tauri::async_runtime::spawn(async move {
        if let Err(e) = run(&app, &dir).await {
            eprintln!("wayland global shortcuts: {e}");
        }
    });
}

async fn run<R: Runtime>(app: &AppHandle<R>, dir: &std::path::Path) -> ashpd::Result<()> {
    register_app_id(app).await;

    let prefs = preferences::load(dir);
    let portal = GlobalShortcuts::new().await?;
    let session = portal.create_session(Default::default()).await?;
    // Stable ids on purpose: the compositor stores the binding under the id, so
    // a shortcut granted once keeps working without prompting again.
    let bindings = [
        NewShortcut::new("toggle", "Toggle launcher")
            .preferred_trigger(preferred_trigger(&prefs.toggle_shortcut).as_deref()),
        NewShortcut::new("clipboard", "Open clipboard history")
            .preferred_trigger(preferred_trigger(&prefs.clipboard_shortcut).as_deref()),
    ];
    // A consent dialog the user never saw never answers, and this call waits
    // for it - give up rather than hang with nothing bound and nothing logged.
    let request = match tokio::time::timeout(
        BIND_TIMEOUT,
        portal.bind_shortcuts(&session, &bindings, None, Default::default()),
    )
    .await
    {
        Ok(r) => r?,
        Err(_) => {
            eprintln!(
                "wayland global shortcuts: the compositor never answered the bind request \
                 (consent dialog dismissed?); no global shortcut is active this run"
            );
            return Ok(());
        }
    };
    // The compositor picks the final combo, so log what it settled on.
    for s in request.response()?.shortcuts() {
        println!("wayland shortcut: {} bound to {}", s.id(), s.trigger_description());
    }

    let mut activated = portal.receive_activated().await?;
    while let Some(event) = activated.next().await {
        shortcuts::dispatch(app, event.shortcut_id());
    }
    Ok(())
}

/// Some compositors (GNOME) only show non-Flatpak apps in the bind dialog once
/// they announce an app id. Best-effort: safe to skip if it fails.
async fn register_app_id<R: Runtime>(app: &AppHandle<R>) {
    let Ok(conn) = ashpd::zbus::Connection::session().await else { return };
    let options = std::collections::HashMap::<&str, Value>::new();
    let _: Result<(), _> = conn
        .call_method(
            Some("org.freedesktop.portal.Desktop"),
            "/org/freedesktop/portal/desktop",
            Some("org.freedesktop.host.portal.Registry"),
            "Register",
            &(app.package_info().name.as_str(), options),
        )
        .await
        .map(|_| ());
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
