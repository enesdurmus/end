# Running on Linux

The launcher has no Linux-only build steps; everything platform-specific lives in
`src-tauri/src/platform/linux.rs`. What it *does* need is a few helper binaries,
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
- **`ydotool`** needs its daemon running and access to `/dev/uinput`.
- **Global shortcuts** go through X11 key grabs, so on a pure Wayland session
  they only fire while an XWayland surface has focus. Bind the launcher in your
  desktop's own keyboard settings if that bites.
