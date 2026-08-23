# Latch

[![CI](https://github.com/enesdurmus/latch/actions/workflows/ci.yml/badge.svg)](https://github.com/enesdurmus/latch/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

A fast, keyboard-driven app launcher for macOS and Linux with no telemetry and
no backend of its own — built with Tauri, React, and TypeScript.

## Why Latch

I was a happy user of a well-known macOS launcher, but my company's IT policy
blocked it — it phones home to a vendor's servers and sends telemetry, which
didn't fly on a work machine. Rather than go without, I built Latch: a
launcher that runs entirely on your machine, doesn't talk to any backend of
its own, and works the same way on Linux as it does on macOS.

## Features

- **App launcher** — fuzzy-search and open installed applications
- **Clipboard history** — searchable history of what you've copied, images included
- **File search** — find and open files by name without leaving the keyboard
- **Translate mode** — type in one language, get the translation instantly
- **Snippets** — save and paste reusable text with a few keystrokes
- **GIF mode** — search and paste GIFs from your own library or a remote provider
- **Global hotkey** — summon Latch from anywhere, no mouse required
- **No telemetry, no backend** — preferences, clipboard history, and snippets
  are stored on disk; Latch has no server of its own and never phones home.
  Translate and GIF search do call their respective third-party APIs directly
  from your machine — that's the only outbound traffic Latch generates
- **Cross-platform** — the same app, the same shortcuts, on macOS and Linux

## Install

Grab the latest build for your platform from the [Releases](https://github.com/enesdurmus/latch/releases) page:

- **macOS**: download the `.dmg`, drag Latch into Applications. The build isn't
  notarized by Apple, so on first launch Gatekeeper will refuse to open it —
  right-click the app and choose **Open**, or run:
  ```sh
  xattr -cr /Applications/Latch.app
  ```
- **Linux**: download the `.AppImage`, make it executable, and run it:
  ```sh
  chmod +x Latch_*.AppImage
  ./Latch_*.AppImage
  ```
  A `.deb` is also published for Debian/Ubuntu-based systems.

Latch checks for updates on startup and will prompt you to install them
in-app (AppImage and macOS builds only — the `.deb` package updates through
your usual package manager).

## Releasing

1. Bump `version` in `package.json` and `src-tauri/tauri.conf.json`.
2. Commit, then tag: `git tag vX.Y.Z && git push origin vX.Y.Z`.
3. GitHub Actions builds macOS and Linux artifacts and opens a **draft**
   release with the binaries and `latest.json`.
4. Review the draft on GitHub and publish it — that's what makes it visible
   to users and to the in-app updater.

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

## Contributing

Contributions are welcome — see [CONTRIBUTING.md](CONTRIBUTING.md) for dev setup and
[CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) for community guidelines.

## License

[MIT](LICENSE)
