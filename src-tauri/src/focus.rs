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
    let prev = app.state::<Focus>().prev();
    on_main(app, move |app| {
        host().restore_focus(prev);
        hide_window(app);
    });
}

/// Clipboard contents are the caller's job.
pub fn hide_and_paste<R: Runtime>(app: &AppHandle<R>) {
    let prev = app.state::<Focus>().prev();
    on_main(app, move |app| {
        host().paste(prev);
        hide_window(app);
    });
}

/// Main thread, hiding last: commands arrive on a worker, and hiding our only window is
/// what makes us stop being the active app.
fn on_main<R: Runtime>(app: &AppHandle<R>, f: impl FnOnce(&AppHandle<R>) + Send + 'static) {
    let handle = app.clone();
    let _ = app.run_on_main_thread(move || f(&handle));
}

/// Dismiss without restoring, for actions that hand focus elsewhere themselves.
pub fn hide_window<R: Runtime>(app: &AppHandle<R>) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.hide();
    }
}
