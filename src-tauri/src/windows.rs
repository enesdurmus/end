//! Window lifecycle wiring: hide the launcher on blur, keep Preferences alive on close.

use tauri::{AppHandle, Manager, Runtime, WindowEvent};

pub fn wire<R: Runtime>(app: &AppHandle<R>) {
    let main = app.get_webview_window("main").unwrap();
    let main2 = main.clone();
    main.on_window_event(move |e| {
        if let WindowEvent::Focused(false) = e {
            let _ = main2.hide();
        }
    });

    let pref = app.get_webview_window("preferences").unwrap();
    let pref2 = pref.clone();
    pref.on_window_event(move |e| {
        if let WindowEvent::CloseRequested { api, .. } = e {
            api.prevent_close();
            let _ = pref2.hide();
        }
    });
}
