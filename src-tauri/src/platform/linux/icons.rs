//! Icon lookup through the XDG icon theme spec.

use super::super::base64_encode;
use super::{desktop, env_dir, home, xdg_data_dirs_in_lookup_order};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

pub(super) fn app_icon(app_path: &str) -> Option<String> {
    let icon = desktop::desktop_field(&fs::read_to_string(app_path).ok()?, "Icon")?.to_string();
    let file = if icon.starts_with('/') { PathBuf::from(&icon) } else { find_themed_icon(&icon)? };
    icon_data_uri(&file)
}

fn find_themed_icon(name: &str) -> Option<PathBuf> {
    const SIZES: [&str; 7] =
        ["64x64", "48x48", "128x128", "256x256", "32x32", "scalable", "symbolic"];
    const EXTS: [&str; 2] = ["png", "svg"];

    let mut bases = vec![
        home().join(".icons"),
        env_dir("XDG_DATA_HOME", home().join(".local/share")).join("icons"),
    ];
    bases.extend(xdg_data_dirs_in_lookup_order().into_iter().map(|d| d.join("icons")));

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
    xdg_data_dirs_in_lookup_order()
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

fn icon_data_uri(file: &Path) -> Option<String> {
    let mime = match file.extension()?.to_str()? {
        "png" => "image/png",
        "svg" => "image/svg+xml",
        _ => return None,
    };
    Some(format!("data:{mime};base64,{}", base64_encode(&fs::read(file).ok()?)))
}
