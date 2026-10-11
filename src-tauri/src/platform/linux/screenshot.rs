//! Region screenshot through the xdg-desktop-portal `Screenshot` interface
//! (GNOME and KDE show their own region picker), copied to the clipboard.

use ashpd::desktop::screenshot::Screenshot;

use crate::platform::screenshot::copy_png;

pub(super) fn region() -> Result<(), String> {
    tauri::async_runtime::block_on(async {
        let response = Screenshot::request().interactive(true).send().await.map_err(|e| e.to_string())?;
        // cancelling the picker is an error response, not a failure
        let Ok(shot) = response.response() else { return Ok(()) };
        let uri = shot.uri().as_str();
        let path = std::path::PathBuf::from(super::clipboard::percent_decode(
            uri.strip_prefix("file://").ok_or_else(|| format!("not a file uri: {uri}"))?,
        ));
        copy_png(&path)
    })
}
