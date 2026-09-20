//! Freedesktop `.desktop` entries: discovery and launching.

use super::{on_path, spawn_detached, xdg_data_dirs_in_lookup_order, AppEntry};
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::process::Command;

pub(super) fn list_apps() -> Vec<AppEntry> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for dir in xdg_data_dirs_in_lookup_order() {
        let apps = dir.join("applications");
        collect_desktop_entries(&apps, &apps, 3, &mut seen, &mut out);
    }
    out.sort_by_key(|a| a.name.to_lowercase());
    out
}

pub(super) fn open(path: &str) -> Result<(), String> {
    if path.ends_with(".desktop") {
        return launch_desktop_entry(path);
    }
    spawn_detached(Command::new("xdg-open").arg(path))
}

pub(super) fn desktop_field<'a>(text: &'a str, key: &str) -> Option<&'a str> {
    text.lines()
        .skip_while(|l| l.trim() != "[Desktop Entry]")
        .skip(1)
        .take_while(|l| !l.trim_start().starts_with('['))
        .find_map(|l| {
            let (k, v) = l.split_once('=')?;
            (k.trim() == key).then(|| v.trim())
        })
}

fn desktop_flag(text: &str, key: &str) -> bool {
    desktop_field(text, key) == Some("true")
}

fn collect_desktop_entries(
    dir: &Path,
    root: &Path,
    depth: usize,
    seen: &mut HashSet<String>,
    out: &mut Vec<AppEntry>,
) {
    let Ok(entries) = fs::read_dir(dir) else { return };
    for e in entries.flatten() {
        let path = e.path();
        if path.is_dir() {
            // is_dir follows symlinks, so the depth cap doubles as the loop guard
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
        if desktop_field(&text, "Type") != Some("Application")
            || desktop_flag(&text, "NoDisplay")
            || desktop_flag(&text, "Hidden")
        {
            continue;
        }
        if let Some(name) = desktop_field(&text, "Name") {
            out.push(AppEntry {
                name: name.to_string(),
                path: path.to_string_lossy().to_string(),
            });
        }
    }
}

fn launch_desktop_entry(path: &str) -> Result<(), String> {
    // only gio applies field codes, Terminal= and D-Bus activation
    if on_path("gio") {
        return spawn_detached(Command::new("gio").arg("launch").arg(path));
    }
    let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let exec = desktop_field(&text, "Exec").ok_or("desktop entry has no Exec=")?;
    let mut argv = exec_argv_without_field_codes(exec).into_iter();
    let bin = argv.next().ok_or("desktop entry has an empty Exec=")?;
    spawn_detached(Command::new(bin).args(argv))
}

fn exec_argv_without_field_codes(exec: &str) -> Vec<String> {
    exec.split_whitespace()
        .filter(|t| !(t.starts_with('%') && t.len() == 2))
        .map(|t| t.trim_matches('"').to_string())
        .collect()
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
        assert_eq!(desktop_field(ENTRY, "Name"), Some("Text Editor"), "localised keys must not match");
        assert_eq!(desktop_field(ENTRY, "Icon"), Some("org.gnome.TextEditor"));
        assert_eq!(desktop_field(ENTRY, "Type"), Some("Application"));
        assert_eq!(desktop_field(ENTRY, "Missing"), None, "keys in a trailing action group are invisible");
        assert!(!desktop_flag(ENTRY, "NoDisplay"));
        assert!(!desktop_flag(ENTRY, "Hidden"), "an absent key is not true");
    }

    #[test]
    fn field_codes_are_stripped_from_exec() {
        assert_eq!(
            exec_argv_without_field_codes(desktop_field(ENTRY, "Exec").unwrap()),
            ["gnome-text-editor", "--gapplication-service"]
        );
        assert_eq!(
            exec_argv_without_field_codes("app %%x %f %ok"),
            ["app", "%%x", "%ok"],
            "%% is a literal percent and a bare %-word is not a field code"
        );
    }

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

        let mut seen = HashSet::new();
        let mut out = Vec::new();
        collect_desktop_entries(&user, &user, 3, &mut seen, &mut out);
        collect_desktop_entries(&sys, &sys, 3, &mut seen, &mut out);

        let mut names: Vec<&str> = out.iter().map(|a| a.name.as_str()).collect();
        names.sort();
        assert_eq!(names, ["Mine", "Nested"], "user entry shadows the system one");
        let _ = fs::remove_dir_all(&root);
    }
}
