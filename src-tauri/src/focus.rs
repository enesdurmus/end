use std::sync::Mutex;
use tauri::{AppHandle, Manager, Runtime};

use crate::platform::{host, Platform};

/// The app to hand focus back to. Read without consuming, so several actions
/// inside one activation all return to the same place.
pub struct Focus {
    prev: Mutex<Option<String>>,
    own: String,
}

impl Focus {
    pub fn new(own: String) -> Self {
        Self { prev: Mutex::new(None), own }
    }

    fn capture(&self) {
        *self.prev.lock().unwrap() = host().frontmost_app_other_than(&self.own);
    }

    fn prev(&self) -> Option<String> {
        self.prev.lock().unwrap().clone()
    }
}

/// Wayland xdg-activation token from the shortcut portal, consumed by the next show().
#[cfg(target_os = "linux")]
static ACTIVATION_TOKEN: Mutex<Option<String>> = Mutex::new(None);

#[cfg(target_os = "linux")]
pub fn set_activation_token(token: String) {
    *ACTIVATION_TOKEN.lock().unwrap() = Some(token);
}

#[cfg(target_os = "linux")]
static SHOWN_AT: Mutex<Option<std::time::Instant>> = Mutex::new(None);

/// True while the compositor may still be settling focus after show().
#[cfg(target_os = "linux")]
pub fn just_shown() -> bool {
    SHOWN_AT.lock().unwrap().is_some_and(|t| t.elapsed() < std::time::Duration::from_millis(500))
}

pub fn show<R: Runtime>(app: &AppHandle<R>) {
    app.state::<Focus>().capture(); // before we become frontmost
    #[cfg(target_os = "linux")]
    {
        *SHOWN_AT.lock().unwrap() = Some(std::time::Instant::now());
    }
    if let Some(w) = app.get_webview_window("main") {
        #[cfg(target_os = "linux")]
        if let Some(token) = ACTIVATION_TOKEN.lock().unwrap().take() {
            // GTK turns the startup id into an xdg-activation request on present
            let gw = w.clone();
            let _ = app.run_on_main_thread(move || {
                use gtk::prelude::GtkWindowExt;
                if let Ok(win) = gw.gtk_window() {
                    win.set_startup_id(&token);
                }
            });
        }
        let _ = w.center();
        let _ = w.show();
        let _ = w.set_focus();
    }
}

pub fn hide<R: Runtime>(app: &AppHandle<R>) {
    let prev = app.state::<Focus>().prev();
    on_main_thread(app, move |app| {
        host().restore_focus(prev);
        hide_window(app);
    });
}

/// Clipboard contents are the caller's job.
pub fn hide_and_paste<R: Runtime>(app: &AppHandle<R>) {
    let prev = app.state::<Focus>().prev();
    on_main_thread(app, move |app| {
        host().paste(prev);
        hide_window(app);
    });
}

/// Commands arrive on a worker thread, and hiding our only window is what makes
/// us stop being the active app — so it has to happen last, here.
fn on_main_thread<R: Runtime>(app: &AppHandle<R>, f: impl FnOnce(&AppHandle<R>) + Send + 'static) {
    let handle = app.clone();
    let _ = app.run_on_main_thread(move || f(&handle));
}

/// Dismiss without restoring, for actions that hand focus elsewhere themselves.
pub fn hide_window<R: Runtime>(app: &AppHandle<R>) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.hide();
    }
}
