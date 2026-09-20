# Cleanup tasks

Findings from the 2026-09-20 repo review, ranked by impact. Delete this file
once the boxes are ticked.

## 1. Bug: the update prompt never shows

`src/lib/updater.ts` calls `ask()` from `@tauri-apps/plugin-dialog`, which
dispatches `plugin:dialog|ask` over IPC. Nothing answers it: the Rust side has
no `tauri-plugin-dialog`. `check()` succeeds, `ask()` throws, and
`App.tsx:65` swallows the error — so an available update is never offered.

Pick one:

- [ ] **Install the plugin:** add `tauri-plugin-dialog` to `src-tauri/Cargo.toml`,
      `.plugin(tauri_plugin_dialog::init())` to `src-tauri/src/lib.rs`, and
      `dialog:default` to `src-tauri/capabilities/default.json`.
- [ ] **Or drop the prompt:** remove the `ask()` call and the
      `@tauri-apps/plugin-dialog` dependency, install updates unprompted.
- [ ] Verify by pointing the updater at a test release and watching it prompt.

## 2. Dead dependencies

- [ ] Remove `tauri-plugin-opener` — registered at `src-tauri/src/lib.rs:31`,
      granted `opener:default` in `capabilities/default.json`, listed in
      `package.json`, and called from nowhere. `open_path` already routes
      through `host().open_path()` (xdg-open / gio / NSWorkspace) and the
      frontend has no `<a href>` or `openUrl` anywhere. Drop all four:
      Cargo.toml dep, `.plugin()` line, capability entry, JS dep.
- [ ] Remove `@tauri-apps/plugin-global-shortcut` from `package.json` — the
      shortcut path is entirely Rust-side; `src/` never imports it.

## 3. Duplicate dependency versions

Both of these compile twice today, once for us and once for a transitive user.

- [ ] Bump `reqwest` from `"0.12"` to `"0.13"` in `src-tauri/Cargo.toml` to
      match `tauri-plugin-updater`'s `0.13.4`. Right now two full
      hyper + rustls stacks build. Only `gifs.rs` and `translate.rs` use it.
- [ ] Bump `png` from `"0.17"` to `"0.18"` to match what `image` and
      `tray-icon` already pull.
- [ ] Confirm with `cargo tree --duplicates | grep -E '^(reqwest|png) v'`.

## 4. Speculative platform

- [ ] Delete `src-tauri/src/platform/windows.rs` — 67 lines, 13 of them TODO
      stubs. `package.json` says macOS and Linux, and `release.yml` builds only
      `macos-latest` and `ubuntu-22.04`, so it is never compiled, tested, or
      shipped. Drop the `#[cfg(target_os = "windows")]` arms in
      `platform/mod.rs` with it and move the TODO list to a GitHub issue.

## 5. Repo hygiene

- [ ] Add `src-tauri/gen/schemas/` to `.gitignore` and `git rm --cached` it.
      `tauri-build` regenerates these every build — that is why `git status`
      shows three modified schemas plus an untracked `linux-schema.json`.

## 6. Duplication in `commands.rs`

- [ ] Extract `fn config_dir(app: &AppHandle) -> PathBuf` — the
      `app.path().app_config_dir().unwrap()` line appears about ten times.
- [ ] Extract `fn update_prefs(app, |p| ...)` — the load / mutate / save
      sequence repeats verbatim in `set_shortcut`, `set_history_limit`,
      `set_image_limit`, `set_translate_prefs`, and `set_gif_prefs`.

## 7. Split `platform/linux.rs` — done

`platform/linux/` and `platform/macos/` are now directories split by job
(`desktop`/`apps`, `icons`, `search`, `clipboard`, `input`), with the shared
path/process helpers in each `mod.rs` and `linux_shortcuts.rs` moved to
`linux/shortcuts.rs`.

## 8. Wayland shortcut rebinding gap

`set_shortcut` (`commands.rs:113`) re-registers the X11 grab but leaves the
portal binding from startup untouched, so on Wayland the Preferences screen
looks more authoritative than it is.

- [ ] Add a note to the Preferences hotkey rows, shown on Wayland, saying the
      compositor owns the binding and it is changed in the system's own
      keyboard settings.
- [ ] Optional: tear down and recreate the portal session on `set_shortcut`
      so the new combo is at least offered as the preferred trigger.
