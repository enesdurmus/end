use tauri::{AppHandle, Manager, Runtime, WindowEvent};

pub fn wire<R: Runtime>(app: &AppHandle<R>) {
    let main = app.get_webview_window("main").unwrap();
    let main2 = main.clone();
    main.on_window_event(move |e| {
        if let WindowEvent::Focused(false) = e {
            // A Wayland compositor shuffles focus for a moment after mapping a
            // surface, so focus loss right after showing says nothing about
            // intent (it dismissed the launcher the instant it opened). Later
            // loss is a real click-away, as on macOS.
            #[cfg(target_os = "linux")]
            if crate::platform::wayland() && crate::focus::just_shown() {
                return;
            }
            // not focus::hide: the user already moved somewhere else
            let _ = main2.hide();
        }
    });
}
