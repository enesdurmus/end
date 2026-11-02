# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.5.0](https://github.com/enesdurmus/end/compare/v0.4.0...v0.5.0) (2026-11-02)


### Features

* add window snapping ([62843a7](https://github.com/enesdurmus/end/commit/62843a728e0131c7bc18e15cf6ce971a539eda0e))
* add window snapping ([#63](https://github.com/enesdurmus/end/issues/63)) ([cdba068](https://github.com/enesdurmus/end/commit/cdba068164c75adecb86cb603666ef01aec672ed))


### Dependencies

* bump arboard from 3.6.1 to 3.7.0 in /src-tauri ([#59](https://github.com/enesdurmus/end/issues/59)) ([b8dc5bb](https://github.com/enesdurmus/end/commit/b8dc5bb8eb85370dc18c27d37a9895427337a2ab))
* bump objc2 from 0.6.4 to 0.6.5 in /src-tauri ([afd66b4](https://github.com/enesdurmus/end/commit/afd66b45ee20559f29bb7db2930d81272d4dbfb9))
* bump react-dom from 19.3.0 to 19.3.1 ([104e525](https://github.com/enesdurmus/end/commit/104e525c95e32622aa9ef11b1c3f4a0071d066e9))
* bump serde_json from 1.0.151 to 1.0.154 in /src-tauri ([7646b85](https://github.com/enesdurmus/end/commit/7646b855502824d30dceb601e67d703e85c65b22))
* bump tauri-plugin-updater in /src-tauri in the tauri group ([2668368](https://github.com/enesdurmus/end/commit/266836840e48046aae5b3f7ba2e129945c83a436))
* bump the tauri group across 1 directory with 3 updates ([950cdb5](https://github.com/enesdurmus/end/commit/950cdb5899efcf0b3d7af950e5eb0b2a574679ea))
* bump the tauri group across 1 directory with 5 updates ([#53](https://github.com/enesdurmus/end/issues/53)) ([2563323](https://github.com/enesdurmus/end/commit/2563323f7d5096775dcac55d294f6a4e4b65cc29))
* bump the tauri group across 1 directory with 6 updates ([#54](https://github.com/enesdurmus/end/issues/54)) ([b240deb](https://github.com/enesdurmus/end/commit/b240deba87364c87dc5a4109bf5cd99f6838a66c))
* bump tokio from 1.53.1 to 1.54.1 in /src-tauri ([6797ccc](https://github.com/enesdurmus/end/commit/6797cccd339398b597eea8bf18ba558c91e71c66))

## [0.4.0](https://github.com/enesdurmus/end/compare/v0.3.1...v0.4.0) (2026-10-11)


### Features

* **screenshot:** region screenshot shortcut that copies to the clipboard ([3050284](https://github.com/enesdurmus/end/commit/3050284e2ac6d8401a39b12712c4affce4e05d99))
* **screenshot:** region screenshot shortcut that copies to the clipboard ([4f5fe3b](https://github.com/enesdurmus/end/commit/4f5fe3bf6cede75be3adeb9ec4b342048bcb1957))

## [0.3.1](https://github.com/enesdurmus/end/compare/v0.3.0...v0.3.1) (2026-10-05)


### Bug Fixes

* **ui:** keep window shadow inside the gutter and drop the native one ([#36](https://github.com/enesdurmus/end/issues/36)) ([db3012f](https://github.com/enesdurmus/end/commit/db3012f6a751d908f8283c2bd96abb9ddf6c0780))

## [0.3.0](https://github.com/enesdurmus/end/compare/v0.2.1...v0.3.0) (2026-10-04)


### Features

* **translate:** show every rendering and pick source/target language ([93764e7](https://github.com/enesdurmus/end/commit/93764e7f0028acbb095265a6eccd472a31955d23))
* **ui:** reference redesign, detail pane and in-panel preferences ([8b55999](https://github.com/enesdurmus/end/commit/8b559992f59d3885074b838bbecd8eeac093fb77))
* **ui:** reference redesign, detail pane and in-panel preferences ([f0a466f](https://github.com/enesdurmus/end/commit/f0a466fd4bd11c6b609deffad943899ec2c0fd89))


### Styles

* **ui:** neutral grey panel surfaces and drop the ⌘K chip ([cdd0334](https://github.com/enesdurmus/end/commit/cdd033470556eab51373350480525a7ae273188b))

## [0.2.1](https://github.com/enesdurmus/end/compare/v0.2.0...v0.2.1) (2026-10-04)


### Dependencies

* **deps:** bump @tauri-apps/plugin-global-shortcut from 2.3.2 to 2.4.0 ([#30](https://github.com/enesdurmus/end/issues/30)) ([2963c01](https://github.com/enesdurmus/end/commit/2963c01a45439b99f2f3bbe5add90cfb75a85ff9))

## [0.2.0](https://github.com/enesdurmus/end/compare/v0.1.2...v0.2.0) (2026-10-04)


### Features

* add End brand icons ([59366b2](https://github.com/enesdurmus/end/commit/59366b27d7d2d359e9d65de772887afe8536dcaa))

## [0.1.2](https://github.com/enesdurmus/End/compare/v0.1.1...v0.1.2) (2026-10-03)


### Bug Fixes

* **deps:** align [@tauri-apps](https://github.com/tauri-apps) npm packages with Rust crates ([edde845](https://github.com/enesdurmus/End/commit/edde8459657fe878ed48be8d6a775ec8424964ea))

## [0.1.1](https://github.com/enesdurmus/End/compare/v0.1.0...v0.1.1) (2026-10-03)


### Bug Fixes

* **deps:** align plugin-react and react-dom with Vite 8 and React 19.3 ([df266ea](https://github.com/enesdurmus/End/commit/df266ea5eff3cdadc1cc1fdd4b2493f0a6a6f3f6))

## [Unreleased]

### Added

- macOS and Linux release builds via GitHub Actions, with in-app auto-update.

## [0.1.0]

- Initial release: app launcher, clipboard history, file search, translate mode,
  GIF mode, and snippets.

[Unreleased]: https://github.com/enesdurmus/End/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/enesdurmus/End/releases/tag/v0.1.0
