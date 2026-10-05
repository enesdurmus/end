# End

[![CI](https://github.com/enesdurmus/End/actions/workflows/ci.yml/badge.svg)](https://github.com/enesdurmus/End/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

A keyboard-driven app launcher for macOS and Linux that runs entirely on your
machine: no telemetry, no account, no backend. Built with Tauri, React and TypeScript.

I used a well-known macOS launcher until my company's IT policy blocked it for
sending telemetry to the vendor's servers. End is the replacement: same workflow,
nothing leaves your machine.

## Features

- App launcher with fuzzy search
- Clipboard history, images included
- File search
- Snippets
- Translate and GIF search (these call their third-party APIs directly from
  your machine, the only outbound traffic End makes)
- Global hotkey

Preferences, history and snippets are stored locally on disk.

## Install

Download a build from [Releases](https://github.com/enesdurmus/End/releases).

- **macOS**: open the `.dmg` and drag End to Applications. The build isn't
  notarized, so run `xattr -cr /Applications/End.app` once, or right-click → Open.
- **Linux**: use the `.AppImage` (`chmod +x End_*.AppImage && ./End_*.AppImage`)
  or the `.deb`. Optional helper tools and Wayland notes: [docs/linux.md](docs/linux.md).

The AppImage and macOS builds update in-app; the `.deb` updates through apt.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

[MIT](LICENSE)
