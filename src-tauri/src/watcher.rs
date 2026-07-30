//! Background thread that polls the system clipboard and appends new entries.

use std::time::Duration;
use tauri::{AppHandle, Manager, Runtime};

use crate::clipboard;
use crate::state::ClipState;

/// Poll the clipboard every 500ms; push (deduped, capped) and persist on change.
pub fn spawn<R: Runtime>(handle: AppHandle<R>) {
    std::thread::spawn(move || {
        let mut cb = match arboard::Clipboard::new() {
            Ok(c) => c,
            Err(_) => return,
        };
        loop {
            if let Ok(txt) = cb.get_text() {
                if !txt.is_empty() {
                    let state = handle.state::<ClipState>();
                    let mut list = state.list.lock().unwrap();
                    let before = list.first().cloned();
                    clipboard::push_capped(&mut list, txt, clipboard::CAP);
                    if list.first() != before.as_ref() {
                        clipboard::save(&state.dir, &list);
                    }
                }
            }
            std::thread::sleep(Duration::from_millis(500));
        }
    });
}
