//! Focus handoff: who had focus before the launcher took it, and giving it back.
//!
//! Every show and dismiss path goes through here, so no call site has to remember
//! to deal with focus.

use std::sync::Mutex;
use tauri::{AppHandle, Manager, Runtime};

use crate::platform::{host, Platform};

/// The app to hand focus back to. Read without consuming, so several actions inside
/// one activation all return to the same place.
pub struct Focus {
    prev: Mutex<Option<String>>,
    own: String,
}

impl Focus {
    pub fn new(own: String) -> Self {
        Self { prev: Mutex::new(None), own }
    }

    fn capture(&self) {
        *self.prev.lock().unwrap() = host().app_behind(&self.own);
    }

    fn prev(&self) -> Option<String> {
        self.prev.lock().unwrap().clone()
    }
}

/// Captures the focus target first — it has to happen before we are frontmost.
pub fn show<R: Runtime>(app: &AppHandle<R>) {
    app.state::<Focus>().capture();
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.center();
        let _ = w.show();
        let _ = w.set_focus();
    }
}

pub fn hide<R: Runtime>(app: &AppHandle<R>) {
    hide_window(app);
    host().restore_focus(app.state::<Focus>().prev());
}

/// Clipboard contents are the caller's job.
pub fn hide_and_paste<R: Runtime>(app: &AppHandle<R>) {
    hide_window(app);
    host().paste(app.state::<Focus>().prev());
}

/// Dismiss without restoring, for actions that hand focus elsewhere themselves.
pub fn hide_window<R: Runtime>(app: &AppHandle<R>) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.hide();
    }
}
