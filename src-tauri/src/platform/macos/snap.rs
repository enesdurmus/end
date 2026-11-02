//! Window snapping through System Events (needs the Accessibility permission).

use std::process::Command;

use crate::snap::{target, Action, Rect};

/// The frontmost window and every screen's work area, in the top-left-origin
/// coordinates System Events uses (AppKit's are bottom-left).
const INFO: &str = r#"
ObjC.import('AppKit');
const p = Application('System Events').processes.whose({frontmost: true})[0];
const w = p.windows[0];
const screens = $.NSScreen.screens;
const top = screens.objectAtIndex(0).frame.size.height;
const areas = [];
for (let i = 0; i < screens.count; i++) {
  const v = screens.objectAtIndex(i).visibleFrame;
  areas.push([v.origin.x, top - v.origin.y - v.size.height, v.size.width, v.size.height]);
}
JSON.stringify({win: [...w.position(), ...w.size()], screens: areas});
"#;

// size first, then position, then size again: a window moved onto a smaller screen gets clamped
const MOVE: &str = r#"
function run(a) {
  const p = Application('System Events').processes.whose({frontmost: true})[0];
  const w = p.windows[0];
  w.size = [+a[2], +a[3]];
  w.position = [+a[0], +a[1]];
  w.size = [+a[2], +a[3]];
}
"#;

fn osascript(script: &str, args: &[String]) -> Result<String, String> {
    let out = Command::new("osascript")
        .args(["-l", "JavaScript", "-e", script])
        .args(args)
        .output()
        .map_err(|e| e.to_string())?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

fn rect(v: &serde_json::Value) -> Option<Rect> {
    let n = |i: usize| v.get(i)?.as_f64().map(|f| f.round() as i32);
    Some(Rect { x: n(0)?, y: n(1)?, w: n(2)?, h: n(3)? })
}

pub(super) fn place(action: Action) -> Result<(), String> {
    let info: serde_json::Value =
        serde_json::from_str(osascript(INFO, &[])?.trim()).map_err(|e| e.to_string())?;
    let win = rect(&info["win"]).ok_or("could not read the window geometry")?;
    let screens: Vec<Rect> = info["screens"].as_array().into_iter().flatten().filter_map(rect).collect();
    let Some(to) = target(action, win, &screens) else { return Ok(()) };
    osascript(MOVE, &[to.x, to.y, to.w, to.h].map(|n| n.to_string())).map(|_| ())
}
