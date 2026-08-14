use std::path::Path;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager, Runtime};

use crate::clipboard::{self, Clip};
use crate::platform::{host, Platform};
use crate::state::ClipState;

const POLL: Duration = Duration::from_millis(500);
const SAVE_DEBOUNCE: Duration = Duration::from_secs(2);

/// Cheapest first, and an app offering both text and an image (a spreadsheet
/// selection) meant the text.
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
        return None;
    }
    let img = cb.get_image().ok()?;
    let app = host().frontmost_app_other_than(own).and_then(|id| host().app_name(&id));
    let name = clipboard::image_name(host().clipboard_source_url().as_deref(), app.as_deref());
    match clipboard::store_image(images, &img.bytes, img.width as u32, img.height as u32, name) {
        Ok(clip) => Some(clip),
        Err(e) => {
            eprintln!("clipboard: storing a copied image failed: {e}");
            None
        }
    }
}

pub fn spawn<R: Runtime>(handle: AppHandle<R>) {
    std::thread::spawn(move || {
        let mut cb = match arboard::Clipboard::new() {
            Ok(c) => c,
            Err(_) => return,
        };
        let images = clipboard::images_dir(&handle.state::<ClipState>().dir);
        let own = handle.config().identifier.clone();
        let mut seen_change: Option<u64> = None;
        let mut dirty = false;
        let mut last_save = Instant::now();
        loop {
            // where the OS offers a counter, an idle tick costs one call and never
            // re-decodes a multi-megabyte screenshot
            let change = host().clipboard_change_count();
            if change.is_none() || change != seen_change {
                seen_change = change;
                let state = handle.state::<ClipState>();
                let cap = state.limit.load(Ordering::Relaxed);
                let image_cap = state.image_limit.load(Ordering::Relaxed);
                if let Some(clip) = capture(&mut cb, &images, image_cap, &own) {
                    let mut list = state.list.lock().unwrap();
                    let before = list.first().map(Clip::dedupe_key);
                    let evicted = clipboard::push_capped(&mut list, clip, cap, image_cap);
                    if list.first().map(Clip::dedupe_key) != before {
                        dirty = true;
                    }
                    drop(list);
                    for path in evicted {
                        let _ = std::fs::remove_file(path);
                    }
                }
            }
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
