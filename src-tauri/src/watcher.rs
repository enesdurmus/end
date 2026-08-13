//! Background thread that polls the system clipboard and appends new entries.

use std::path::Path;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager, Runtime};

use crate::clipboard::{self, Clip};
use crate::platform::{host, Platform};
use crate::state::ClipState;

const POLL: Duration = Duration::from_millis(500);
const SAVE_DEBOUNCE: Duration = Duration::from_secs(2);

/// What the clipboard holds right now, or `None` if it holds nothing we keep.
///
/// Order is by cost and by intent: file URLs are a few strings; text is cheap
/// and, when an app offers both, is what the user meant to copy (a spreadsheet
/// selection ships a rendered image alongside its text); an image is decoded
/// only when nothing cheaper is on offer.
fn capture(cb: &mut arboard::Clipboard, images: &Path, image_cap: usize, own: &str) -> Option<Clip> {
    let files = host().clipboard_files();
    if !files.is_empty() {
        return Some(Clip::Files { paths: files });
    }
    if let Ok(text) = cb.get_text() {
        if !text.is_empty() {
            return Some(Clip::Text { text });
        }
    }
    if image_cap == 0 {
        return None; // images turned off: don't decode one just to delete it
    }
    let img = cb.get_image().ok()?;
    // Both lookups only run when an image is actually being stored — the app one
    // shells out, so it must not sit on the idle path.
    let app = host().app_behind(own).and_then(|id| host().app_name(&id));
    let name = clipboard::image_name(host().clipboard_url().as_deref(), app.as_deref());
    match clipboard::store_image(images, &img.bytes, img.width as u32, img.height as u32, name) {
        Ok(clip) => Some(clip),
        Err(e) => {
            eprintln!("clipboard: storing a copied image failed: {e}");
            None
        }
    }
}

/// Poll the clipboard every 500ms; push (deduped, capped) new entries.
/// Disk writes are debounced so a burst of copies collapses into one save.
pub fn spawn<R: Runtime>(handle: AppHandle<R>) {
    std::thread::spawn(move || {
        let mut cb = match arboard::Clipboard::new() {
            Ok(c) => c,
            Err(_) => return,
        };
        let images = clipboard::images_dir(&handle.state::<ClipState>().dir);
        // our own bundle id, so "which app was this copied from" can exclude us
        let own = handle.config().identifier.clone();
        let mut seen_change: Option<u64> = None;
        let mut dirty = false;
        let mut last_save = Instant::now();
        loop {
            // The OS counter turns an idle tick into one FFI call: nothing is read,
            // and a multi-megabyte screenshot is never decoded twice. Platforms
            // without a counter report None and get the old read-every-tick path.
            let change = host().clipboard_change_count();
            if change.is_none() || change != seen_change {
                seen_change = change;
                let state = handle.state::<ClipState>();
                let cap = state.limit.load(Ordering::Relaxed);
                let image_cap = state.image_limit.load(Ordering::Relaxed);
                if let Some(clip) = capture(&mut cb, &images, image_cap, &own) {
                    let mut list = state.list.lock().unwrap();
                    let before = list.first().map(Clip::key);
                    let evicted = clipboard::push_capped(&mut list, clip, cap, image_cap);
                    if list.first().map(Clip::key) != before {
                        dirty = true;
                    }
                    drop(list);
                    for path in evicted {
                        let _ = std::fs::remove_file(path);
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
