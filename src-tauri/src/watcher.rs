//! Background thread that polls the system clipboard and appends new entries.

use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager, Runtime};

use crate::clipboard;
use crate::state::ClipState;

const POLL: Duration = Duration::from_millis(500);
const SAVE_DEBOUNCE: Duration = Duration::from_secs(2);

/// Poll the clipboard every 500ms; push (deduped, capped) new entries.
/// Disk writes are debounced so a burst of copies collapses into one save.
pub fn spawn<R: Runtime>(handle: AppHandle<R>) {
    std::thread::spawn(move || {
        let mut cb = match arboard::Clipboard::new() {
            Ok(c) => c,
            Err(_) => return,
        };
        let mut dirty = false;
        let mut last_save = Instant::now();
        loop {
            if let Ok(txt) = cb.get_text() {
                if !txt.is_empty() {
                    let state = handle.state::<ClipState>();
                    let cap = state.limit.load(Ordering::Relaxed);
                    let mut list = state.list.lock().unwrap();
                    let before = list.first().cloned();
                    clipboard::push_capped(&mut list, txt, cap);
                    if list.first() != before.as_ref() {
                        dirty = true;
                    }
                }
            }
            // ponytail: debounce persistence; at most one write / 2s, so a burst of
            // copies is one save. Trade-off: up to ~2s of history lost on a hard crash.
            if dirty && last_save.elapsed() >= SAVE_DEBOUNCE {
                let state = handle.state::<ClipState>();
                clipboard::save(&state.dir, &state.list.lock().unwrap());
                dirty = false;
                last_save = Instant::now();
            }
            std::thread::sleep(POLL);
        }
    });
}
