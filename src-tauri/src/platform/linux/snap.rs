//! Window snapping. Wayland has no way to move another client's window, so on
//! GNOME End installs a small Shell extension that does it over D-Bus; X11 goes
//! through xdotool and wmctrl. Other Wayland compositors are unsupported.

use std::process::Command;

use ashpd::zbus::Connection;

use crate::snap::{target, Action, Rect};

const UUID: &str = "end-snap@enesdurmus.github.io";
const NAME: &str = "io.github.enesdurmus.EndSnap";
const PATH: &str = "/io/github/enesdurmus/EndSnap";
const EXTENSION: [(&str, &str); 2] = [
    ("metadata.json", include_str!("../../../gnome-extension/metadata.json")),
    ("extension.js", include_str!("../../../gnome-extension/extension.js")),
];

type Wire = (i32, i32, i32, i32);

fn rect((x, y, w, h): Wire) -> Rect {
    Rect { x, y, w, h }
}

fn gnome() -> bool {
    std::env::var("XDG_CURRENT_DESKTOP").is_ok_and(|d| d.contains("GNOME"))
}

pub(super) fn place(action: Action) -> Result<(), String> {
    if !super::wayland() {
        return place_x11(action);
    }
    if !gnome() {
        return Err("window snapping on Wayland needs GNOME".into());
    }
    tauri::async_runtime::block_on(place_gnome(action))
}

pub(super) fn setup() -> Option<String> {
    if !super::wayland() {
        let missing: Vec<_> = ["xdotool", "wmctrl", "xrandr", "xprop"].into_iter().filter(|t| !super::on_path(t)).collect();
        return (!missing.is_empty()).then(|| format!("Install {} to enable window snapping.", missing.join(", ")));
    }
    if !gnome() {
        return Some("Window snapping on Wayland is only supported on GNOME.".into());
    }
    if let Err(e) = install_extension() {
        return Some(format!("Could not install the GNOME extension: {e}"));
    }
    if tauri::async_runtime::block_on(gnome_call("Ping", &())).is_ok() {
        return None;
    }
    let _ = Command::new("gnome-extensions").args(["enable", UUID]).output();
    // GNOME on Wayland only discovers a new extension at login
    Some("Log out and back in once to finish setting up window snapping.".into())
}

fn install_extension() -> std::io::Result<()> {
    let dir = super::env_dir("XDG_DATA_HOME", super::home().join(".local/share"))
        .join("gnome-shell/extensions")
        .join(UUID);
    std::fs::create_dir_all(&dir)?;
    for (file, content) in EXTENSION {
        let path = dir.join(file);
        if std::fs::read_to_string(&path).is_ok_and(|old| old == content) {
            continue;
        }
        std::fs::write(path, content)?;
    }
    Ok(())
}

async fn gnome_call<B: serde::Serialize + ashpd::zbus::zvariant::DynamicType>(
    method: &str,
    body: &B,
) -> Result<ashpd::zbus::Message, String> {
    let conn = Connection::session().await.map_err(|e| e.to_string())?;
    conn.call_method(Some(NAME), PATH, Some(NAME), method, body).await.map_err(|e| e.to_string())
}

async fn place_gnome(action: Action) -> Result<(), String> {
    let reply = gnome_call("Info", &())
        .await
        .map_err(|e| format!("GNOME extension not reachable ({e}); log out and back in once after enabling snapping"))?;
    let info: Vec<Wire> = reply.body().deserialize().map_err(|e| e.to_string())?;
    let Some((win, screens)) = info.split_first() else { return Ok(()) };
    let screens: Vec<Rect> = screens.iter().copied().map(rect).collect();
    let Some(to) = target(action, rect(*win), &screens) else { return Ok(()) };
    gnome_call("MoveResize", &(to.x, to.y, to.w, to.h)).await.map(|_| ())
}

fn run(cmd: &str, args: &[&str]) -> Result<String, String> {
    let out = Command::new(cmd).args(args).output().map_err(|e| format!("{cmd}: {e}"))?;
    if !out.status.success() {
        return Err(format!("{cmd} failed"));
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

fn place_x11(action: Action) -> Result<(), String> {
    let id = run("xdotool", &["getactivewindow"])?.trim().to_string();
    // xdotool reports the client area, so a decorated window can land a
    // title bar off; fix with _NET_FRAME_EXTENTS if it shows
    let win = parse_shell_geometry(&run("xdotool", &["getwindowgeometry", "--shell", &id])?)
        .ok_or("could not read the window geometry")?;
    let work = parse_workarea(&run("xprop", &["-root", "_NET_WORKAREA"])?);
    let screens: Vec<Rect> = run("xrandr", &["--listmonitors"])?
        .lines()
        .filter_map(parse_monitor)
        .filter_map(|m| match work {
            Some(w) => intersect(m, w),
            None => Some(m),
        })
        .collect();
    let Some(to) = target(action, win, &screens) else { return Ok(()) };
    let _ = run("wmctrl", &["-i", "-r", &id, "-b", "remove,maximized_vert,maximized_horz"]);
    run("wmctrl", &["-i", "-r", &id, "-e", &format!("0,{},{},{},{}", to.x, to.y, to.w, to.h)]).map(|_| ())
}

/// `X=1\nY=2\nWIDTH=3\nHEIGHT=4` (plus a WINDOW line)
fn parse_shell_geometry(s: &str) -> Option<Rect> {
    let get = |key: &str| {
        s.lines().find_map(|l| l.strip_prefix(key)?.strip_prefix('=')?.trim().parse::<i32>().ok())
    };
    Some(Rect { x: get("X")?, y: get("Y")?, w: get("WIDTH")?, h: get("HEIGHT")? })
}

/// ` 0: +*DP-1 1920/527x1080/296+0+0  DP-1` -> 1920x1080 at 0,0
fn parse_monitor(line: &str) -> Option<Rect> {
    let geometry = line.split_whitespace().find(|t| t.contains('x') && t.contains('+'))?;
    let (size, pos) = geometry.split_once('+')?;
    let (w, h) = size.split_once('x')?;
    let (x, y) = pos.split_once('+')?;
    let mm = |s: &str| s.split('/').next()?.parse().ok();
    Some(Rect { x: x.parse().ok()?, y: y.parse().ok()?, w: mm(w)?, h: mm(h)? })
}

/// `_NET_WORKAREA(CARDINAL) = 0, 27, 1920, 1053, ...`: the first desktop's area.
fn parse_workarea(s: &str) -> Option<Rect> {
    let n: Vec<i32> = s.split('=').nth(1)?.split(',').filter_map(|v| v.trim().parse().ok()).collect();
    Some(Rect { x: *n.first()?, y: *n.get(1)?, w: *n.get(2)?, h: *n.get(3)? })
}

fn intersect(a: Rect, b: Rect) -> Option<Rect> {
    let (x, y) = (a.x.max(b.x), a.y.max(b.y));
    let (r, bottom) = ((a.x + a.w).min(b.x + b.w), (a.y + a.h).min(b.y + b.h));
    (r > x && bottom > y).then_some(Rect { x, y, w: r - x, h: bottom - y })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_xrandr_monitors() {
        let r = parse_monitor(" 0: +*DP-1 1920/527x1080/296+1920+0  DP-1").unwrap();
        assert_eq!(r, Rect { x: 1920, y: 0, w: 1920, h: 1080 });
        assert_eq!(parse_monitor("Monitors: 2"), None);
    }

    #[test]
    fn parses_workarea_and_shell_geometry() {
        let w = parse_workarea("_NET_WORKAREA(CARDINAL) = 0, 27, 1920, 1053, 0, 27, 1920, 1053").unwrap();
        assert_eq!(w, Rect { x: 0, y: 27, w: 1920, h: 1053 });
        let g = parse_shell_geometry("WINDOW=1\nX=10\nY=20\nWIDTH=300\nHEIGHT=400\nSCREEN=0\n").unwrap();
        assert_eq!(g, Rect { x: 10, y: 20, w: 300, h: 400 });
    }

    #[test]
    fn panel_is_cut_out_of_the_monitor() {
        let m = Rect { x: 0, y: 0, w: 1920, h: 1080 };
        assert_eq!(intersect(m, Rect { x: 0, y: 27, w: 1920, h: 1053 }), Some(Rect { x: 0, y: 27, w: 1920, h: 1053 }));
        assert_eq!(intersect(m, Rect { x: 2000, y: 0, w: 10, h: 10 }), None);
    }
}
