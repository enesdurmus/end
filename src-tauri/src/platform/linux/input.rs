//! Window focus and synthetic keystrokes via xdotool / wtype / ydotool.

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
    let Some(key) = key_tool() else { return };
    // focus only leaves us once the caller hides our window, right after this returns
    std::thread::spawn(move || {
        std::thread::sleep(paste_delay());
        key.press_paste();
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
    fn press_paste(&self) {
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
        let _ = cmd.status();
    }
}

pub(super) fn key_tool() -> Option<KeyTool> {
    // xdotool still reaches most Wayland clients through XWayland
    let candidates: [(&str, KeyTool); 3] = if wayland() {
        [("wtype", KeyTool::Wtype), ("ydotool", KeyTool::Ydotool), ("xdotool", KeyTool::Xdotool)]
    } else {
        [("xdotool", KeyTool::Xdotool), ("wtype", KeyTool::Wtype), ("ydotool", KeyTool::Ydotool)]
    };
    candidates.into_iter().find(|(bin, _)| on_path(bin)).map(|(_, k)| k)
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
