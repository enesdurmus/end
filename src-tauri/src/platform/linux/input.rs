//! Window focus and synthetic keystrokes via the RemoteDesktop portal or xdotool / wtype / ydotool.

use super::{on_path, wayland};
use std::process::Command;
use std::sync::OnceLock;
use std::time::Duration;

pub(super) fn frontmost_window() -> Option<String> {
    if wayland() {
        return None;
    }
    let id = xdotool(&["getactivewindow"])?;
    let pid = xdotool(&["getwindowpid", &id])?;
    (pid != std::process::id().to_string()).then_some(id)
}

pub(super) fn window_class(window_id: &str) -> Option<String> {
    xdotool(&["getwindowclassname", window_id])
}

pub(super) fn restore_focus(prev: Option<String>) {
    if let Some(id) = prev {
        // no --sync: it blocks forever if the window closed meanwhile
        xdotool(&["windowactivate", &id]);
    }
}

pub(super) fn paste(prev: Option<String>) {
    restore_focus(prev);
    // focus only leaves us once the caller hides our window, right after this returns
    std::thread::spawn(move || {
        std::thread::sleep(paste_delay());
        // GNOME's Mutter has neither the virtual-keyboard protocol nor, usually,
        // any helper tool installed, but every Wayland desktop with a
        // RemoteDesktop portal can inject keys without one.
        if wayland() && tauri::async_runtime::block_on(portal::paste()) {
            return;
        }
        // wtype exits non-zero on Mutter; keep going until one tool succeeds.
        for tool in key_tools() {
            if tool.press_paste() {
                break;
            }
        }
    });
}

fn xdotool(args: &[&str]) -> Option<String> {
    let out = Command::new("xdotool").args(args).output().ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    (!s.is_empty()).then_some(s)
}

pub(super) enum KeyTool {
    Xdotool,
    Wtype,
    Ydotool,
}

impl KeyTool {
    fn press_paste(&self) -> bool {
        let mut cmd = match self {
            KeyTool::Xdotool => {
                let mut c = Command::new("xdotool");
                c.args(["key", "--clearmodifiers", "ctrl+v"]);
                c
            }
            KeyTool::Wtype => {
                let mut c = Command::new("wtype");
                c.args(["-M", "ctrl", "v", "-m", "ctrl"]);
                c
            }
            KeyTool::Ydotool => {
                let mut c = Command::new("ydotool");
                // keycodes: 29 = leftctrl, 47 = v
                c.args(["key", "29:1", "47:1", "47:0", "29:0"]);
                c
            }
        };
        cmd.status().is_ok_and(|s| s.success())
    }
}

fn key_tools() -> Vec<KeyTool> {
    // xdotool still reaches most Wayland clients through XWayland
    let order: [(&str, KeyTool); 3] = if wayland() {
        [("wtype", KeyTool::Wtype), ("ydotool", KeyTool::Ydotool), ("xdotool", KeyTool::Xdotool)]
    } else {
        [("xdotool", KeyTool::Xdotool), ("wtype", KeyTool::Wtype), ("ydotool", KeyTool::Ydotool)]
    };
    order.into_iter().filter(|(bin, _)| on_path(bin)).map(|(_, k)| k).collect()
}

/// Whether pasting can work at all: a helper tool, or the portal on Wayland.
pub(super) fn can_paste() -> bool {
    key_tool().is_some() || wayland()
}

pub(super) fn key_tool() -> Option<KeyTool> {
    key_tools().into_iter().next()
}

fn paste_delay() -> Duration {
    static D: OnceLock<Duration> = OnceLock::new();
    *D.get_or_init(|| {
        let ms = std::env::var("LAUNCHER_PASTE_DELAY_MS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(120);
        Duration::from_millis(ms)
    })
}

/// Ctrl+V through the xdg-desktop-portal RemoteDesktop interface.
pub(super) mod portal {
    use ashpd::desktop::remote_desktop::{DeviceType, KeyState, RemoteDesktop, SelectDevicesOptions};
    use ashpd::desktop::{PersistMode, Session};
    use std::time::Duration;
    use tokio::sync::Mutex;

    type Live = (RemoteDesktop, Session<RemoteDesktop>);

    /// One session for the whole run: opening it is what shows the consent dialog.
    static LIVE: Mutex<Option<Live>> = Mutex::const_new(None);
    const CONSENT_TIMEOUT: Duration = Duration::from_secs(60);

    /// The restore token lets the compositor skip the dialog on later runs.
    fn token_path() -> std::path::PathBuf {
        super::super::env_dir("XDG_DATA_HOME", super::super::home().join(".local/share"))
            .join("latch/remote-desktop-token")
    }

    async fn connect() -> ashpd::Result<Live> {
        let rd = RemoteDesktop::new().await?;
        let session = rd.create_session(Default::default()).await?;
        let token = std::fs::read_to_string(token_path()).ok();
        rd.select_devices(
            &session,
            SelectDevicesOptions::default()
                .set_devices(Some(DeviceType::Keyboard.into()))
                .set_persist_mode(PersistMode::ExplicitlyRevoked)
                .set_restore_token(token.as_deref().map(str::trim)),
        )
        .await?;
        let started = rd.start(&session, None, Default::default()).await?.response()?;
        if let Some(t) = started.restore_token() {
            let path = token_path();
            if let Some(dir) = path.parent() {
                let _ = std::fs::create_dir_all(dir);
            }
            let _ = std::fs::write(path, t);
        }
        Ok((rd, session))
    }

    async fn press_paste(rd: &RemoteDesktop, session: &Session<RemoteDesktop>) -> ashpd::Result<()> {
        // evdev keycodes: 29 = leftctrl, 47 = v
        for (code, state) in [
            (29, KeyState::Pressed),
            (47, KeyState::Pressed),
            (47, KeyState::Released),
            (29, KeyState::Released),
        ] {
            rd.notify_keyboard_keycode(session, code, state, Default::default()).await?;
        }
        Ok(())
    }

    /// Opens the session if needed (this is where the consent dialog appears).
    pub async fn ensure() -> Result<(), String> {
        let mut live = LIVE.lock().await;
        if live.is_none() {
            let opened = tokio::time::timeout(CONSENT_TIMEOUT, connect())
                .await
                .map_err(|_| "the compositor never answered the keyboard request".to_string())?
                .map_err(|e| e.to_string())?;
            *live = Some(opened);
        }
        Ok(())
    }

    pub async fn paste() -> bool {
        for _ in 0..2 {
            if let Err(e) = ensure().await {
                eprintln!("portal paste: {e}");
                return false;
            }
            let mut live = LIVE.lock().await;
            let Some((rd, session)) = live.as_ref() else { continue };
            match press_paste(rd, session).await {
                Ok(()) => return true,
                // session died (portal restarted, access revoked): reconnect once
                Err(e) => {
                    eprintln!("portal paste: {e}");
                    *live = None;
                }
            }
        }
        false
    }
}

