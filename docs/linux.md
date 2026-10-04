# Running on Linux

The launcher has no Linux-only build steps; everything platform-specific lives in
`src-tauri/src/platform/linux/`. What it *does* need is a few helper binaries,
because X11 and Wayland expose no in-process API for the things it does.

## Build dependencies

The usual Tauri v2 set: `webkit2gtk-4.1`, `libsoup-3.0`, `gtk3`, plus
`libayatana-appindicator3` for the tray icon.

## Runtime tools

| Tool | Needed for | Session |
|------|-----------|---------|
| `xdotool` | focus hand-off + the Ctrl+V keystroke | X11 |
| `xclip` | copying/reading files on the clipboard | X11 |
| `wl-clipboard` (`wl-copy`/`wl-paste`) | same, natively | Wayland |
| `wtype` **or** `ydotool` | the Ctrl+V keystroke | Wayland |
| `plocate` | file search (falls back to a shallow `$HOME` walk) | both |
| `gio` | launching `.desktop` entries properly | both |

Each is optional: a missing tool degrades that one feature, nothing crashes.
Preferences → "Accessibility permission" reports whether a keystroke tool was
found, since that is Linux's equivalent of the macOS permission gate.

## Wayland notes

- **Clipboard.** `wl-clipboard` needs the `wlr-data-control` protocol. On
  compositors that lack it (Mutter, historically) the backend detects the
  refusal and falls back to `xclip` over XWayland, which Mutter bridges.
- **Focus.** A Wayland client cannot ask who is focused or focus someone else, so
  focus handoff is left to the compositor: hiding our window returns focus on its
  own. Only the keystroke is synthesised.
- **Paste timing.** The keystroke is sent ~120ms after the window hides, so it
  lands in the app that got focus back rather than in us. Tune with
  `LAUNCHER_PASTE_DELAY_MS` if your compositor is slower.
- **Pasting without tools.** On Wayland the Ctrl+V is sent first through the `org.freedesktop.portal.RemoteDesktop` portal (keyboard only), so no helper binary is needed on GNOME or KDE. The first paste shows a consent dialog (Preferences → "Open System Settings" opens it ahead of time); the grant is remembered through a restore token in `~/.local/share/end/`.
- **Terminals.** End always sends Ctrl+V, which most terminals take as a literal ^V. Bind Ctrl+V to paste in yours (WezTerm: `config.keys = { { key = 'v', mods = 'CTRL', action = wezterm.action.PasteFrom 'Clipboard' } }`).
- **Key tools.** Mutter (GNOME) has no virtual-keyboard protocol, so `wtype` fails there; the paste tries each installed tool in turn (`wtype`, `ydotool`, `xdotool`) until one succeeds.
- **Dismiss.** Clicking away closes the launcher as on macOS; focus loss in the first 500ms after showing is ignored because compositors shuffle focus then.
- **Focus on show.** The shortcut portal's `activation_token` is handed to GTK before the window is shown, so Mutter/KWin allow it to take focus.
- **`ydotool`** needs its daemon running and access to `/dev/uinput`.
- **Global shortcuts.** `tauri-plugin-global-shortcut` grabs keys through X11,
  so on a pure Wayland session it only fires while an XWayland surface has
  focus. To cover that, the app also binds its shortcuts through the
  `org.freedesktop.portal.GlobalShortcuts` portal whenever `WAYLAND_DISPLAY`
  is set (`src-tauri/src/platform/linux/shortcuts.rs`), independent of the X11
  grab above. This needs a compositor that implements the portal — GNOME 45+
  and KDE Plasma 6+ do; wlroots compositors (Sway, etc.) generally don't yet,
  so they fall back to the X11-grab-while-XWayland-focused behavior described
  above. The first bind shows a one-time system consent dialog, and unlike the
  X11 path the compositor owns the final key combo: it takes the app's
  `toggle_shortcut`/`clipboard_shortcut` preference as a hint only, and the
  user can rebind through their desktop's own shortcut settings instead of
  this app's preferences screen. Nothing crashes if the portal is missing or
  the user declines the dialog — it just falls back to the X11 behavior.
- **Changing a shortcut.** The portal remembers a binding by its id and only treats the preferred key as a first-bind hint, so the app puts the key into the id (`toggle@LOGO+space`). Changing the shortcut in Preferences closes the old portal session and binds a new one; the compositor shows its consent dialog once for the new key.
