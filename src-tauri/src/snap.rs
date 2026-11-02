//! Window-snapping layouts and their keys. Pure geometry:
//! the platform backends read the focused window and the screens, ask `target`
//! where the window should go, and move it.

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Action {
    /// x, y, w, h in sixths of the screen: halves, thirds and two-thirds share one table.
    Grid(u8, u8, u8, u8),
    Center,
    NextDisplay,
    PrevDisplay,
}

pub struct Binding {
    pub id: &'static str,
    pub label: &'static str,
    pub accelerator: &'static str,
    pub action: Action,
}

use Action::*;

const fn bind(id: &'static str, label: &'static str, accelerator: &'static str, action: Action) -> Binding {
    Binding { id, label, accelerator, action }
}

pub const BINDINGS: &[Binding] = &[
    bind("snap-left-half", "Snap window to the left half", "Control+Alt+ArrowLeft", Grid(0, 0, 3, 6)),
    bind("snap-right-half", "Snap window to the right half", "Control+Alt+ArrowRight", Grid(3, 0, 3, 6)),
    bind("snap-top-half", "Snap window to the top half", "Control+Alt+ArrowUp", Grid(0, 0, 6, 3)),
    bind("snap-bottom-half", "Snap window to the bottom half", "Control+Alt+ArrowDown", Grid(0, 3, 6, 3)),
    bind("snap-top-left", "Snap window to the top left", "Control+Alt+KeyU", Grid(0, 0, 3, 3)),
    bind("snap-top-right", "Snap window to the top right", "Control+Alt+KeyI", Grid(3, 0, 3, 3)),
    bind("snap-bottom-left", "Snap window to the bottom left", "Control+Alt+KeyJ", Grid(0, 3, 3, 3)),
    bind("snap-bottom-right", "Snap window to the bottom right", "Control+Alt+KeyK", Grid(3, 3, 3, 3)),
    bind("snap-first-third", "Snap window to the first third", "Control+Alt+KeyD", Grid(0, 0, 2, 6)),
    bind("snap-center-third", "Snap window to the center third", "Control+Alt+KeyF", Grid(2, 0, 2, 6)),
    bind("snap-last-third", "Snap window to the last third", "Control+Alt+KeyG", Grid(4, 0, 2, 6)),
    bind("snap-first-two-thirds", "Snap window to the first two thirds", "Control+Alt+KeyE", Grid(0, 0, 4, 6)),
    bind("snap-center-two-thirds", "Snap window to the center two thirds", "Control+Alt+KeyR", Grid(1, 0, 4, 6)),
    bind("snap-last-two-thirds", "Snap window to the last two thirds", "Control+Alt+KeyT", Grid(2, 0, 4, 6)),
    bind("snap-maximize", "Maximize window", "Control+Alt+Enter", Grid(0, 0, 6, 6)),
    bind("snap-center", "Center window", "Control+Alt+KeyC", Center),
    bind("snap-next-display", "Move window to the next display", "Control+Alt+Super+ArrowRight", NextDisplay),
    bind("snap-prev-display", "Move window to the previous display", "Control+Alt+Super+ArrowLeft", PrevDisplay),
];

pub fn find(id: &str) -> Option<&'static Binding> {
    BINDINGS.iter().find(|b| b.id == id)
}

/// Where `win` should go. `screens` are work areas (panels and docks excluded);
/// the window belongs to the one holding its center.
pub fn target(action: Action, win: Rect, screens: &[Rect]) -> Option<Rect> {
    let (cx, cy) = (win.x + win.w / 2, win.y + win.h / 2);
    let cur = screens
        .iter()
        .position(|s| (s.x..s.x + s.w).contains(&cx) && (s.y..s.y + s.h).contains(&cy))
        .unwrap_or(0);
    let a = *screens.get(cur)?;
    // both edges are rounded separately so neighbouring cells never leave a pixel gap
    let edge = |origin: i32, len: i32, n: u8| origin + len * n as i32 / 6;
    Some(match action {
        Grid(gx, gy, gw, gh) => {
            let (x, y) = (edge(a.x, a.w, gx), edge(a.y, a.h, gy));
            Rect { x, y, w: edge(a.x, a.w, gx + gw) - x, h: edge(a.y, a.h, gy + gh) - y }
        }
        Center => Rect {
            x: a.x + (a.w - win.w).max(0) / 2,
            y: a.y + (a.h - win.h).max(0) / 2,
            ..win
        },
        NextDisplay | PrevDisplay => {
            let n = screens.len();
            let to = screens[if action == NextDisplay { (cur + 1) % n } else { (cur + n - 1) % n }];
            // keep the window's place on the screen, scaled to the new one
            Rect {
                x: to.x + (win.x - a.x) * to.w / a.w,
                y: to.y + (win.y - a.y) * to.h / a.h,
                w: win.w * to.w / a.w,
                h: win.h * to.h / a.h,
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    const SCREEN: Rect = Rect { x: 0, y: 27, w: 1920, h: 1053 };
    const WIN: Rect = Rect { x: 100, y: 100, w: 800, h: 600 };

    fn grid(id: &str) -> Action {
        find(id).unwrap().action
    }

    #[test]
    fn halves_fill_the_work_area() {
        assert_eq!(target(grid("snap-left-half"), WIN, &[SCREEN]), Some(Rect { x: 0, y: 27, w: 960, h: 1053 }));
        assert_eq!(target(grid("snap-right-half"), WIN, &[SCREEN]), Some(Rect { x: 960, y: 27, w: 960, h: 1053 }));
    }

    #[test]
    fn thirds_tile_without_gaps() {
        let w = |id| target(grid(id), WIN, &[SCREEN]).unwrap();
        let (a, b, c) = (w("snap-first-third"), w("snap-center-third"), w("snap-last-third"));
        assert_eq!((a.x + a.w, b.x + b.w), (b.x, c.x));
        assert_eq!(c.x + c.w, 1920);
    }

    #[test]
    fn maximize_is_the_whole_work_area() {
        assert_eq!(target(grid("snap-maximize"), WIN, &[SCREEN]), Some(SCREEN));
    }

    #[test]
    fn center_keeps_the_size() {
        assert_eq!(target(Center, WIN, &[SCREEN]), Some(Rect { x: 560, y: 253, w: 800, h: 600 }));
    }

    #[test]
    fn displays_wrap_and_follow_the_window() {
        let right = Rect { x: 1920, y: 0, w: 1920, h: 1080 };
        let screens = [SCREEN, right];
        let moved = target(NextDisplay, WIN, &screens).unwrap();
        assert!(moved.x >= right.x);
        assert_eq!(target(NextDisplay, moved, &screens).unwrap().x, WIN.x);
        assert_eq!(target(PrevDisplay, WIN, &screens).unwrap().x, moved.x);
        // left half lands on the screen the window is on
        let on_right = Rect { x: 2000, ..WIN };
        assert_eq!(target(grid("snap-left-half"), on_right, &screens).unwrap().x, 1920);
    }

    #[test]
    fn no_screens_means_no_target() {
        assert_eq!(target(Center, WIN, &[]), None);
    }

    #[test]
    fn every_accelerator_parses_and_is_unique() {
        let mut seen = Vec::new();
        for b in BINDINGS {
            let s = tauri_plugin_global_shortcut::Shortcut::from_str(b.accelerator).expect(b.accelerator);
            assert!(!seen.contains(&s), "{} is bound twice", b.accelerator);
            seen.push(s);
        }
    }
}
