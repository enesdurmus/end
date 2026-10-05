# Linux

Platform code lives in `src-tauri/src/platform/linux/`. X11 and Wayland expose no
in-process API for focus, paste or clipboard files, so End uses a few helper tools.
Each is optional: if one is missing, only that feature degrades.

## Build dependencies

The usual Tauri v2 set: `webkit2gtk-4.1`, `libsoup-3.0`, `gtk3`, `libayatana-appindicator3`.

## Runtime tools

| Tool | Used for | Session |
|------|----------|---------|
| `xdotool` | focus hand-off, Ctrl+V | X11 |
| `xclip` | file clipboard | X11 |
| `wl-clipboard` | file clipboard | Wayland |
| `wtype` or `ydotool` | Ctrl+V | Wayland |
| `plocate` | file search (falls back to a shallow `$HOME` walk) | both |
| `gio` | launching `.desktop` entries | both |

Preferences → "Accessibility permission" shows whether a paste tool was found.

## Wayland

- **Clipboard:** `wl-clipboard` needs `wlr-data-control`. Where it's missing
  (Mutter), End falls back to `xclip` over XWayland.
- **Paste:** Ctrl+V goes through the `RemoteDesktop` portal first, so GNOME and KDE
  need no helper tool. The first paste asks for consent; the grant is stored in
  `~/.local/share/end/`. Otherwise End tries `wtype`, `ydotool`, `xdotool` in turn
  (`wtype` doesn't work on Mutter; `ydotool` needs its daemon and `/dev/uinput`).
- **Paste timing:** the keystroke is sent ~120ms after the window hides. Raise
  `LAUNCHER_PASTE_DELAY_MS` if your compositor is slow.
- **Focus:** the compositor returns focus when End hides. Focus loss in the first
  500ms after showing is ignored.
- **Global shortcuts:** with `WAYLAND_DISPLAY` set, End binds through the
  `GlobalShortcuts` portal (GNOME 45+, KDE Plasma 6+; most wlroots compositors
  lack it). The compositor owns the final key, so rebind it in your desktop's
  settings; changing it in Preferences opens a new consent dialog. Without the
  portal, the shortcut only fires while an XWayland window has focus.
- **Terminals:** End sends Ctrl+V, which most terminals read as a literal `^V`.
  Bind Ctrl+V to paste in yours.
