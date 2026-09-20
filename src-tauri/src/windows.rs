use tauri::{AppHandle, Manager, Runtime, WindowEvent};

pub fn wire<R: Runtime>(app: &AppHandle<R>) {
    let main = app.get_webview_window("main").unwrap();
    let main2 = main.clone();
    main.on_window_event(move |e| {
        if let WindowEvent::Focused(false) = e {
            // A Wayland compositor shuffles focus for a while after mapping a
            // surface, so focus loss there says nothing about intent — acting
            // on it dismissed the launcher the instant it opened. Escape, a
            // completed action and the toggle shortcut still close it.
            #[cfg(target_os = "linux")]
            if crate::platform::wayland() {
                return;
            }
            // not focus::hide: the user already moved somewhere else
            let _ = main2.hide();
        }
    });

    let pref = app.get_webview_window("preferences").unwrap();
    titlebar_clicks_reach_the_buttons(&pref);
    let pref2 = pref.clone();
    pref.on_window_event(move |e| {
        if let WindowEvent::CloseRequested { api, .. } = e {
            api.prevent_close();
            let _ = pref2.hide();
        }
    });
}

/// tao 0.35 wraps its Wayland client-side titlebar in a GtkEventBox with
/// `above_child` set, which swallows every pointer event before the close and
/// minimise buttons see one. Fixed in tao 0.36; delete this once tauri ships it.
#[cfg(target_os = "linux")]
fn titlebar_clicks_reach_the_buttons<R: Runtime>(window: &tauri::WebviewWindow<R>) {
    use gtk::prelude::*;
    let Ok(gtk_window) = window.gtk_window() else { return };
    if let Some(bar) = gtk_window.titlebar() {
        if let Ok(event_box) = bar.downcast::<gtk::EventBox>() {
            event_box.set_above_child(false);
        }
    }
}

#[cfg(not(target_os = "linux"))]
fn titlebar_clicks_reach_the_buttons<R: Runtime>(_window: &tauri::WebviewWindow<R>) {}
