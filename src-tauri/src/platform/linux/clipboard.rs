//! Clipboard file transfer through wl-clipboard or xclip.

use super::{on_path, wayland};
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;

pub(super) fn copy_files(paths: &[String]) -> Result<(), String> {
    let uris: String = paths.iter().map(|p| format!("{}\n", file_uri(p))).collect();
    match clip_tool() {
        ClipTool::Wayland => {
            pipe_to(Command::new("wl-copy").args(["--type", "text/uri-list"]), &uris)
        }
        ClipTool::X11 => pipe_to(
            Command::new("xclip").args(["-selection", "clipboard", "-t", "text/uri-list"]),
            &uris,
        ),
        ClipTool::None => {
            Err("no clipboard tool: install wl-clipboard (Wayland) or xclip (X11)".into())
        }
    }
}

pub(super) fn files() -> Vec<String> {
    uri_list()
        .lines()
        .filter_map(|l| l.strip_prefix("file://"))
        .map(percent_decode)
        .filter(|p| !p.is_empty())
        .collect()
}

pub(super) fn source_url() -> Option<String> {
    let list = uri_list();
    let first = list.lines().find(|l| !l.trim().is_empty())?.trim();
    (!first.starts_with("file://")).then(|| first.to_string())
}

pub(super) fn change_count() -> Option<u64> {
    let n = wl_change_counter()?.load(Ordering::Relaxed);
    (n != COUNTER_DEAD).then_some(n)
}

enum ClipTool {
    Wayland,
    X11,
    None,
}

fn clip_tool() -> &'static ClipTool {
    static B: OnceLock<ClipTool> = OnceLock::new();
    B.get_or_init(|| {
        if wayland() && on_path("wl-copy") && on_path("wl-paste") && wl_data_control_works() {
            ClipTool::Wayland
        } else if on_path("xclip") {
            // XWayland bridges the clipboard where wl-clipboard can't reach it
            ClipTool::X11
        } else {
            ClipTool::None
        }
    })
}

fn wl_data_control_works() -> bool {
    match Command::new("wl-paste").arg("-l").output() {
        Ok(o) => !String::from_utf8_lossy(&o.stderr).contains("does not support"),
        Err(_) => false,
    }
}

fn pipe_to(cmd: &mut Command, data: &str) -> Result<(), String> {
    // both helpers daemonize, so the spawned child exits at once and needs reaping
    let mut child =
        cmd.stdin(Stdio::piped()).stdout(Stdio::null()).spawn().map_err(|e| e.to_string())?;
    child.stdin.take().ok_or("no stdin")?.write_all(data.as_bytes()).map_err(|e| e.to_string())?;
    std::thread::spawn(move || {
        let _ = child.wait();
    });
    Ok(())
}

fn uri_list() -> String {
    let out = match clip_tool() {
        ClipTool::Wayland => Command::new("wl-paste").args(["-n", "-t", "text/uri-list"]).output(),
        ClipTool::X11 => Command::new("xclip")
            .args(["-selection", "clipboard", "-t", "text/uri-list", "-o"])
            .output(),
        ClipTool::None => return String::new(),
    };
    match out {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).into_owned(),
        _ => String::new(),
    }
}

const COUNTER_DEAD: u64 = 0;

fn wl_change_counter() -> Option<&'static AtomicU64> {
    static C: OnceLock<Option<&'static AtomicU64>> = OnceLock::new();
    *C.get_or_init(|| {
        if !matches!(clip_tool(), ClipTool::Wayland) {
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
            (b'%', Some(h), Some(l)) => {
                match u8::from_str_radix(&format!("{}{}", *h as char, *l as char), 16) {
                    Ok(byte) => {
                        out.push(byte);
                        i += 3;
                    }
                    Err(_) => {
                        out.push(b'%');
                        i += 1;
                    }
                }
            }
            _ => {
                out.push(b[i]);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let list = "https://example.com/cat.gif\nfile:///tmp/a%20b.txt\n";
        let files: Vec<String> = list
            .lines()
            .filter_map(|l| l.strip_prefix("file://"))
            .map(percent_decode)
            .collect();
        assert_eq!(files, ["/tmp/a b.txt"]);
    }
}
