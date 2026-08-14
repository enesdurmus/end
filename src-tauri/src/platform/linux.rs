//! Linux backend. Mechanisms are picked at runtime: X11 has `xdotool` for focus
//! and keys, Wayland lets a client do neither, so focus is left to the
//! compositor and only the keystroke is synthesised. A missing helper degrades
//! that one feature; nothing here panics or hard-fails.

use super::{b64, AppEntry, Platform};
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;
use std::time::Duration;

pub struct Linux;

impl Platform for Linux {
    fn list_apps(&self) -> Vec<AppEntry> {
        let mut seen = std::collections::HashSet::new();
        let mut out = Vec::new();
        // XDG order: the first file with a given id wins, so ~/.local/share
        // shadows the system entry.
        for dir in data_dirs() {
            let apps = dir.join("applications");
            collect_desktop_entries(&apps, &apps, 3, &mut seen, &mut out);
        }
        out.sort_by_key(|a| a.name.to_lowercase());
        out
    }

    fn app_icon(&self, app_path: &str) -> Option<String> {
        let icon = field(&fs::read_to_string(app_path).ok()?, "Icon")?.to_string();
        // an absolute Icon= is legal, and skips the theme search
        let file = if icon.starts_with('/') {
            PathBuf::from(&icon)
        } else {
            find_icon(&icon)?
        };
        icon_data_uri(&file)
    }

    fn open_path(&self, path: &str) -> Result<(), String> {
        if path.ends_with(".desktop") {
            return launch_desktop(path);
        }
        detach(Command::new("xdg-open").arg(path))
    }

    fn search_files(&self, query: &str) -> Vec<AppEntry> {
        let query = query.trim();
        if query.is_empty() {
            return Vec::new();
        }
        // ponytail: no index installed means no results outside $HOME.
        match ["plocate", "locate"].into_iter().find(|b| has(b)) {
            Some(bin) => Command::new(bin)
                .args(["-i", "-l", "20", "--", query])
                .output()
                .ok()
                .map(|o| String::from_utf8_lossy(&o.stdout).lines().map(entry_for).collect())
                .unwrap_or_default(),
            None => walk_home(&query.to_lowercase()),
        }
    }

    fn app_behind(&self, _own: &str) -> Option<String> {
        if wayland() {
            return None; // no Wayland client may ask who else is focused
        }
        let id = xdotool(&["getactivewindow"])?;
        // excluding ourselves is the point, and only the pid can say
        let pid = xdotool(&["getwindowpid", &id])?;
        (pid != std::process::id().to_string()).then_some(id)
    }

    fn restore_focus(&self, prev: Option<String>) {
        if let Some(id) = prev {
            // not --sync: it blocks until the window activates, which never
            // happens if it closed meanwhile — and this is the main thread.
            xdotool(&["windowactivate", &id]);
        }
    }

    fn paste(&self, prev: Option<String>) {
        self.restore_focus(prev);
        let Some(key) = key_tool() else { return };
        // The key goes wherever focus is, and focus only leaves us once the
        // caller hides our window — right after this returns. So wait it out.
        std::thread::spawn(move || {
            std::thread::sleep(paste_delay());
            key.press_paste();
        });
    }

    fn copy_files(&self, paths: &[String]) -> Result<(), String> {
        let uris: String = paths.iter().map(|p| format!("{}\n", file_uri(p))).collect();
        // ponytail: text/uri-list only, since one helper process can own one
        // target — file managers wanting x-special/gnome-copied-files see nothing.
        match backend() {
            Clip::Wl => feed(Command::new("wl-copy").args(["--type", "text/uri-list"]), &uris),
            Clip::X11 => feed(
                Command::new("xclip").args(["-selection", "clipboard", "-t", "text/uri-list"]),
                &uris,
            ),
            Clip::None => Err("no clipboard tool: install wl-clipboard (Wayland) or xclip (X11)".into()),
        }
    }

    fn clipboard_files(&self) -> Vec<String> {
        uri_list()
            .lines()
            .filter_map(|l| l.strip_prefix("file://"))
            .map(percent_decode)
            .filter(|p| !p.is_empty())
            .collect()
    }

    fn clipboard_url(&self) -> Option<String> {
        // Not a file URI: that is a Files entry already, and its path would be a
        // misleading name for pixels copied from somewhere else.
        let list = uri_list();
        let first = list.lines().find(|l| !l.trim().is_empty())?.trim();
        (!first.starts_with("file://")).then(|| first.to_string())
    }

    fn app_name(&self, window_id: &str) -> Option<String> {
        // `app_behind` hands back a window id, not a bundle id
        xdotool(&["getwindowclassname", window_id])
    }

    fn clipboard_change_count(&self) -> Option<u64> {
        // Only Wayland has a change event to count; X11 gets the read-every-tick
        // path the trait documents.
        let n = wl_change_counter()?.load(Ordering::Relaxed);
        (n != COUNTER_DEAD).then_some(n)
    }

    fn accessibility_granted(&self) -> bool {
        // No permission gate exists here, so answer what the UI is really
        // asking: can we synthesise a keystroke at all.
        key_tool().is_some()
    }

    fn open_accessibility_settings(&self) -> Result<(), String> {
        Err(format!(
            "no keystroke tool found. Install {} to enable pasting.",
            if wayland() { "wtype or ydotool" } else { "xdotool" }
        ))
    }
}


fn wayland() -> bool {
    static W: OnceLock<bool> = OnceLock::new();
    *W.get_or_init(|| std::env::var_os("WAYLAND_DISPLAY").is_some())
}

fn home() -> PathBuf {
    std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default()
}

fn env_dir(key: &str, default: PathBuf) -> PathBuf {
    match std::env::var_os(key) {
        Some(v) if !v.is_empty() => PathBuf::from(v),
        _ => default,
    }
}

/// `$XDG_DATA_HOME` then `$XDG_DATA_DIRS`, in lookup order.
fn data_dirs() -> Vec<PathBuf> {
    let mut dirs = vec![env_dir("XDG_DATA_HOME", home().join(".local/share"))];
    let sys = std::env::var("XDG_DATA_DIRS").unwrap_or_default();
    let sys = if sys.is_empty() { "/usr/local/share:/usr/share".to_string() } else { sys };
    dirs.extend(sys.split(':').filter(|s| !s.is_empty()).map(PathBuf::from));
    dirs
}

/// A binary on `$PATH`, scanned rather than shelled out to `which` — this sits
/// on the paste path.
fn has(bin: &str) -> bool {
    std::env::var_os("PATH")
        .map(|path| std::env::split_paths(&path).any(|d| d.join(bin).is_file()))
        .unwrap_or(false)
}


/// The value of `key` in the `[Desktop Entry]` group. Localised variants
/// (`Name[tr]`) deliberately don't match.
fn field<'a>(text: &'a str, key: &str) -> Option<&'a str> {
    text.lines()
        .skip_while(|l| l.trim() != "[Desktop Entry]")
        .skip(1)
        .take_while(|l| !l.trim_start().starts_with('['))
        .find_map(|l| {
            let (k, v) = l.split_once('=')?;
            (k.trim() == key).then(|| v.trim())
        })
}

fn is_true(text: &str, key: &str) -> bool {
    field(text, key) == Some("true")
}

/// Walks `dir` for `*.desktop`, deduped by spec id (the path relative to
/// `root`), so an earlier XDG dir shadows a later one.
fn collect_desktop_entries(
    dir: &Path,
    root: &Path,
    depth: usize,
    seen: &mut std::collections::HashSet<String>,
    out: &mut Vec<AppEntry>,
) {
    let Ok(entries) = fs::read_dir(dir) else { return };
    for e in entries.flatten() {
        let path = e.path();
        if path.is_dir() {
            // is_dir follows symlinks, so the depth cap is also the loop guard
            if depth > 0 {
                collect_desktop_entries(&path, root, depth - 1, seen, out);
            }
            continue;
        }
        if path.extension().is_none_or(|x| x != "desktop") {
            continue;
        }
        let id = path.strip_prefix(root).unwrap_or(&path).to_string_lossy().replace('/', "-");
        if !seen.insert(id) {
            continue;
        }
        let Ok(text) = fs::read_to_string(&path) else { continue };
        if field(&text, "Type") != Some("Application")
            || is_true(&text, "NoDisplay")
            || is_true(&text, "Hidden")
        {
            continue;
        }
        if let Some(name) = field(&text, "Name") {
            out.push(AppEntry {
                name: name.to_string(),
                path: path.to_string_lossy().to_string(),
            });
        }
    }
}

fn launch_desktop(path: &str) -> Result<(), String> {
    // only gio applies the entry properly: field codes, Terminal=, D-Bus activation
    if has("gio") {
        return detach(Command::new("gio").arg("launch").arg(path));
    }
    let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let exec = field(&text, "Exec").ok_or("desktop entry has no Exec=")?;
    let mut argv = exec_argv(exec).into_iter();
    let bin = argv.next().ok_or("desktop entry has an empty Exec=")?;
    detach(Command::new(bin).args(argv))
}

/// `Exec=` minus its field codes (`%f`, `%U`, …) — we launch with no arguments.
///
/// ponytail: quoted arguments with spaces come apart here; gio, tried first,
/// handles them.
fn exec_argv(exec: &str) -> Vec<String> {
    exec.split_whitespace()
        .filter(|t| !(t.starts_with('%') && t.len() == 2))
        .map(|t| t.trim_matches('"').to_string())
        .collect()
}


/// Freedesktop icon lookup as a flat candidate list, covering both layouts in
/// the wild (`theme/48x48/apps/x.png`, `theme/apps/48/x.png`) plus pixmaps.
///
/// ponytail: no index.theme and no Inherits= chain — misses fall through to
/// hicolor, which the spec requires every icon to live in anyway.
fn find_icon(name: &str) -> Option<PathBuf> {
    const SIZES: [&str; 7] = ["64x64", "48x48", "128x128", "256x256", "32x32", "scalable", "symbolic"];
    const EXTS: [&str; 2] = ["png", "svg"];

    let mut bases = vec![home().join(".icons"), env_dir("XDG_DATA_HOME", home().join(".local/share")).join("icons")];
    bases.extend(data_dirs().into_iter().map(|d| d.join("icons")));

    let user = user_icon_theme();
    let themes: Vec<&str> = [user.as_deref(), Some("hicolor"), Some("Adwaita")]
        .into_iter()
        .flatten()
        .collect();

    let files: Vec<String> = EXTS.iter().map(|ext| format!("{name}.{ext}")).collect();
    for base in &bases {
        if !base.is_dir() {
            continue;
        }
        for theme in &themes {
            // pruning here keeps a miss to a few stats instead of the whole
            // cross product, and every app row asks for an icon
            let dir = base.join(theme);
            if !dir.is_dir() {
                continue;
            }
            for size in SIZES {
                for file in &files {
                    for cand in [
                        dir.join(size).join("apps").join(file),
                        dir.join("apps").join(size.split('x').next().unwrap()).join(file),
                    ] {
                        if cand.is_file() {
                            return Some(cand);
                        }
                    }
                }
            }
        }
    }
    // pixmaps: no theme, no size, just the file
    data_dirs()
        .into_iter()
        .map(|d| d.join("pixmaps"))
        .flat_map(|d| EXTS.iter().map(move |e| d.join(format!("{name}.{e}"))))
        .find(|p| p.is_file())
}

fn user_icon_theme() -> Option<String> {
    static T: OnceLock<Option<String>> = OnceLock::new();
    T.get_or_init(|| {
        let out = Command::new("gsettings")
            .args(["get", "org.gnome.desktop.interface", "icon-theme"])
            .output()
            .ok()?;
        let theme = String::from_utf8_lossy(&out.stdout).trim().trim_matches('\'').to_string();
        (!theme.is_empty()).then_some(theme)
    })
    .clone()
}

/// SVG goes to the webview as-is — it renders vectors, so nothing here needs a
/// rasteriser.
fn icon_data_uri(file: &Path) -> Option<String> {
    let mime = match file.extension()?.to_str()? {
        "png" => "image/png",
        "svg" => "image/svg+xml",
        _ => return None, // .xpm: nothing renders it, nothing modern ships it
    };
    Some(format!("data:{mime};base64,{}", b64(&fs::read(file).ok()?)))
}


fn entry_for(path: &str) -> AppEntry {
    let name = Path::new(path).file_name().and_then(|s| s.to_str()).unwrap_or(path);
    AppEntry { name: name.to_string(), path: path.to_string() }
}

/// Breadth-first, so shallow matches come first. Bounded in depth, results and
/// directories visited: this runs per keystroke, and a home directory with a
/// node_modules in it is effectively unbounded.
fn walk_home(needle: &str) -> Vec<AppEntry> {
    const MAX_DEPTH: usize = 3;
    const MAX_RESULTS: usize = 20;
    const MAX_DIRS: usize = 2_000;
    let mut out = Vec::new();
    let mut budget = MAX_DIRS;
    let mut level = vec![home()];
    for _ in 0..MAX_DEPTH {
        let mut next = Vec::new();
        for dir in level {
            if budget == 0 {
                return out;
            }
            budget -= 1;
            let Ok(entries) = fs::read_dir(&dir) else { continue };
            for e in entries.flatten() {
                let name = e.file_name().to_string_lossy().to_string();
                if name.starts_with('.') {
                    continue;
                }
                if name.to_lowercase().contains(needle) {
                    out.push(entry_for(&e.path().to_string_lossy()));
                    if out.len() >= MAX_RESULTS {
                        return out;
                    }
                }
                if e.file_type().is_ok_and(|t| t.is_dir()) {
                    next.push(e.path());
                }
            }
        }
        level = next;
    }
    out
}


enum Clip {
    Wl,
    X11,
    None,
}

/// Which clipboard helper to talk to, decided once. A Wayland session isn't
/// enough on its own: `wl-clipboard` needs the data-control protocol, which
/// Mutter historically lacks — there XWayland bridges the clipboard and `xclip`
/// works, so the probe asks rather than guesses.
fn backend() -> &'static Clip {
    static B: OnceLock<Clip> = OnceLock::new();
    B.get_or_init(|| {
        if wayland() && has("wl-copy") && has("wl-paste") && wl_data_control_works() {
            Clip::Wl
        } else if has("xclip") {
            Clip::X11
        } else {
            Clip::None
        }
    })
}

/// `wl-paste -l` exits non-zero on an empty clipboard too, so the exit code
/// can't be the signal — the unsupported-protocol message is.
fn wl_data_control_works() -> bool {
    match Command::new("wl-paste").arg("-l").output() {
        Ok(o) => !String::from_utf8_lossy(&o.stderr).contains("does not support"),
        Err(_) => false,
    }
}

fn feed(cmd: &mut Command, data: &str) -> Result<(), String> {
    // Both helpers daemonize, so the child we spawn forks the process that will
    // serve the selection and exits at once. Reaping it keeps every copy from
    // leaving a zombie; dropping stdin below is what sends EOF.
    let mut child = cmd.stdin(Stdio::piped()).stdout(Stdio::null()).spawn().map_err(|e| e.to_string())?;
    child.stdin.take().ok_or("no stdin")?.write_all(data.as_bytes()).map_err(|e| e.to_string())?;
    std::thread::spawn(move || {
        let _ = child.wait();
    });
    Ok(())
}

fn uri_list() -> String {
    let out = match backend() {
        Clip::Wl => Command::new("wl-paste").args(["-n", "-t", "text/uri-list"]).output(),
        Clip::X11 => {
            Command::new("xclip").args(["-selection", "clipboard", "-t", "text/uri-list", "-o"]).output()
        }
        Clip::None => return String::new(),
    };
    match out {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).into_owned(),
        _ => String::new(), // clipboard holds something else; not an error
    }
}

/// Sentinel for "the watcher died, go back to polling" — a real count starts at 1.
const COUNTER_DEAD: u64 = 0;

/// A counter fed by `wl-paste --watch`, turning the 500ms poll into an atomic
/// load: nothing is read, and a big screenshot is never decoded twice.
fn wl_change_counter() -> Option<&'static AtomicU64> {
    static C: OnceLock<Option<&'static AtomicU64>> = OnceLock::new();
    *C.get_or_init(|| {
        if !matches!(backend(), Clip::Wl) {
            return None;
        }
        let counter: &'static AtomicU64 = Box::leak(Box::new(AtomicU64::new(1)));
        let mut child = Command::new("wl-paste")
            .args(["--watch", "echo"])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .ok()?;
        let stdout = child.stdout.take()?;
        std::thread::spawn(move || {
            for _ in BufReader::new(stdout).lines().map_while(Result::ok) {
                counter.fetch_add(1, Ordering::Relaxed);
            }
            counter.store(COUNTER_DEAD, Ordering::Relaxed);
            let _ = child.wait();
        });
        Some(counter)
    })
}

const UNRESERVED: &[u8] = b"-._~/";

fn file_uri(path: &str) -> String {
    let mut s = String::from("file://");
    for &b in path.as_bytes() {
        if b.is_ascii_alphanumeric() || UNRESERVED.contains(&b) {
            s.push(b as char);
        } else {
            s.push_str(&format!("%{b:02X}"));
        }
    }
    s
}

fn percent_decode(s: &str) -> String {
    let b = s.trim().as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        match (b[i], b.get(i + 1), b.get(i + 2)) {
            (b'%', Some(h), Some(l)) => match u8::from_str_radix(&format!("{}{}", *h as char, *l as char), 16) {
                Ok(byte) => {
                    out.push(byte);
                    i += 3;
                }
                Err(_) => {
                    out.push(b'%');
                    i += 1;
                }
            },
            _ => {
                out.push(b[i]);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}


fn xdotool(args: &[&str]) -> Option<String> {
    let out = Command::new("xdotool").args(args).output().ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    (!s.is_empty()).then_some(s)
}

enum Key {
    Xdotool,
    Wtype,
    Ydotool,
}

impl Key {
    fn press_paste(&self) {
        let mut cmd = match self {
            Key::Xdotool => {
                let mut c = Command::new("xdotool");
                c.args(["key", "--clearmodifiers", "ctrl+v"]);
                c
            }
            Key::Wtype => {
                let mut c = Command::new("wtype");
                c.args(["-M", "ctrl", "v", "-m", "ctrl"]);
                c
            }
            // ydotool speaks keycodes: 29 = leftctrl, 47 = v
            Key::Ydotool => {
                let mut c = Command::new("ydotool");
                c.args(["key", "29:1", "47:1", "47:0", "29:0"]);
                c
            }
        };
        let _ = cmd.status();
    }
}

/// Wayland falls back to xdotool: under XWayland it still reaches X11 clients,
/// which is most of them.
fn key_tool() -> Option<Key> {
    let candidates: [(&str, Key); 3] = if wayland() {
        [("wtype", Key::Wtype), ("ydotool", Key::Ydotool), ("xdotool", Key::Xdotool)]
    } else {
        [("xdotool", Key::Xdotool), ("wtype", Key::Wtype), ("ydotool", Key::Ydotool)]
    };
    candidates.into_iter().find(|(bin, _)| has(bin)).map(|(_, k)| k)
}

/// How long to wait for focus to leave us before sending Ctrl+V. Compositors
/// differ, hence the knob.
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

/// Spawn without leaving a zombie behind.
fn detach(cmd: &mut Command) -> Result<(), String> {
    let mut child = cmd.stdout(Stdio::null()).stderr(Stdio::null()).spawn().map_err(|e| e.to_string())?;
    std::thread::spawn(move || {
        let _ = child.wait();
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const ENTRY: &str = "\
[Desktop Entry]
Type=Application
Name=Text Editor
Name[tr]=Metin Düzenleyici
Icon=org.gnome.TextEditor
Exec=gnome-text-editor --gapplication-service %U
NoDisplay=false

[Desktop Action new-window]
Name=New Window
Exec=gnome-text-editor --new-window";

    #[test]
    fn fields_come_from_the_desktop_entry_group_only() {
        assert_eq!(field(ENTRY, "Name"), Some("Text Editor"), "localised keys must not match");
        assert_eq!(field(ENTRY, "Icon"), Some("org.gnome.TextEditor"));
        assert_eq!(field(ENTRY, "Type"), Some("Application"));
        // present only in the trailing action group, so invisible from here
        assert_eq!(field(ENTRY, "Missing"), None);
        assert!(!is_true(ENTRY, "NoDisplay"));
        assert!(!is_true(ENTRY, "Hidden"), "an absent key is not true");
    }

    #[test]
    fn field_codes_are_stripped_from_exec() {
        assert_eq!(
            exec_argv(field(ENTRY, "Exec").unwrap()),
            ["gnome-text-editor", "--gapplication-service"]
        );
        // %% is a literal percent, and a bare %-word is not a field code
        assert_eq!(exec_argv("app %%x %f %ok"), ["app", "%%x", "%ok"]);
    }

    #[test]
    fn file_uris_round_trip_through_percent_escapes() {
        let path = "/home/ada/my files/résumé (1).pdf";
        let uri = file_uri(path);
        assert_eq!(uri, "file:///home/ada/my%20files/r%C3%A9sum%C3%A9%20%281%29.pdf");
        assert_eq!(percent_decode(uri.strip_prefix("file://").unwrap()), path);
    }

    #[test]
    fn a_malformed_escape_survives_decoding() {
        assert_eq!(percent_decode("/tmp/100%"), "/tmp/100%");
        assert_eq!(percent_decode("/tmp/a%zz"), "/tmp/a%zz");
    }

    #[test]
    fn clipboard_files_ignores_non_file_uris() {
        // what `clipboard_files` does to a uri-list, without a live clipboard
        let list = "https://example.com/cat.gif\nfile:///tmp/a%20b.txt\n";
        let files: Vec<String> = list
            .lines()
            .filter_map(|l| l.strip_prefix("file://"))
            .map(percent_decode)
            .collect();
        assert_eq!(files, ["/tmp/a b.txt"]);
    }

    /// The shadowing rule list_apps depends on, plus the entries that must
    /// never be listed at all.
    #[test]
    fn desktop_entries_dedupe_by_id_and_skip_hidden_ones() {
        let root = std::env::temp_dir().join("launcher-desktop-scan-test");
        let (user, sys) = (root.join("user"), root.join("sys"));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(user.join("sub")).unwrap();
        fs::create_dir_all(&sys).unwrap();
        let entry = |name: &str, extra: &str| {
            format!("[Desktop Entry]\nType=Application\nName={name}\n{extra}\n")
        };
        fs::write(user.join("editor.desktop"), entry("Mine", "")).unwrap();
        fs::write(sys.join("editor.desktop"), entry("Theirs", "")).unwrap();
        fs::write(sys.join("daemon.desktop"), entry("Daemon", "NoDisplay=true")).unwrap();
        fs::write(sys.join("old.desktop"), entry("Old", "Hidden=true")).unwrap();
        fs::write(sys.join("link.desktop"), "[Desktop Entry]\nType=Link\nName=Link\n").unwrap();
        fs::write(sys.join("notes.txt"), "not a desktop entry").unwrap();
        fs::write(user.join("sub/nested.desktop"), entry("Nested", "")).unwrap();

        let mut seen = std::collections::HashSet::new();
        let mut out = Vec::new();
        collect_desktop_entries(&user, &user, 3, &mut seen, &mut out);
        collect_desktop_entries(&sys, &sys, 3, &mut seen, &mut out);

        let mut names: Vec<&str> = out.iter().map(|a| a.name.as_str()).collect();
        names.sort();
        assert_eq!(names, ["Mine", "Nested"], "user entry shadows the system one");
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn missing_tools_never_panic() {
        assert!(!has("definitely-not-a-real-binary-name"));
        assert!(Linux.search_files("   ").is_empty());
        assert!(Linux.app_icon("/nonexistent.desktop").is_none());
    }
}
