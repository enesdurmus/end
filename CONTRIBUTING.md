# Contributing to End

Thanks for taking the time to contribute!

## Development setup

```sh
npm install
npm run tauri dev
```

On Linux, you'll need a few system packages first — see [`docs/linux.md`](docs/linux.md).

## Tests

```sh
npm test              # frontend (vitest)
cd src-tauri && cargo test   # backend
```

Both suites should pass before opening a PR.

## Making a change

1. Fork the repo and create a branch off `main`.
2. Keep changes focused — one PR, one concern.
3. Add or update tests for behavior you change.
4. Run `npm run build` (typecheck) and both test suites locally.
5. Open a PR describing what changed and why.

## Reporting bugs / requesting features

Use the issue templates on the [Issues](https://github.com/enesdurmus/End/issues) page.
