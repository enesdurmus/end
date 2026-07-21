use crate::apps::AppEntry;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

pub fn search(query: &str) -> Vec<AppEntry> {
    if query.trim().is_empty() {
        return Vec::new();
    }
    // ponytail: mdfind can return tens of thousands of lines for common words;
    // stream stdout and stop at the first 20 → child dies via SIGPIPE.
    let mut child = match Command::new("mdfind")
        .arg("-name")
        .arg(query)
        .stdout(Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    let reader = match child.stdout.take() {
        Some(o) => BufReader::new(o),
        None => return Vec::new(),
    };
    let out: Vec<AppEntry> = reader
        .lines()
        .map_while(Result::ok)
        .take(20)
        .map(|path| {
            let name = std::path::Path::new(&path)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or(&path)
                .to_string();
            AppEntry { name, path }
        })
        .collect();
    let _ = child.kill();
    let _ = child.wait();
    out
}
