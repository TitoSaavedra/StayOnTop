# StayOnTop

A lightweight Windows utility to pin any application window on top of all others, with opacity and click-through control.

Built with [Tauri 2](https://tauri.app) + [SvelteKit](https://kit.svelte.dev) + TypeScript.

---

## Features

- **Pin any window** — select any running process and keep it always on top
- **Opacity control** — adjust transparency per pinned window (10%–100%)
- **Click-through mode** — let mouse events pass through a pinned window
- **Image overlay** — pin images directly on screen with opacity and scale control
- **Excluded processes** — hide specific processes from the list permanently
- **Global hotkey** — pin/unpin the selected window from anywhere
- **Auto-start** — launch with Windows on startup
- **Auto-update** — receives updates silently from GitHub Releases
- **Multilanguage** — English, Español, Português (BR)
- **System tray** — lives in the tray when closed

---

## Download

Grab the latest installer from the [Releases](https://github.com/TitoSaavedra/StayOnTop/releases/latest) page.

---

## Development

### Prerequisites

- [Node.js](https://nodejs.org) v22+
- [pnpm](https://pnpm.io) v10+
- [Rust](https://rustup.rs)
- [Tauri prerequisites for Windows](https://tauri.app/start/prerequisites/)

### Setup

```bash
git clone https://github.com/TitoSaavedra/StayOnTop.git
cd StayOnTop
pnpm install
```

### Run in dev mode

```bash
pnpm tauri dev
```

### Build

```bash
pnpm tauri build
```

---

## Releasing a new version

1. Bump `version` in `src-tauri/tauri.conf.json` and `src-tauri/Cargo.toml`
2. Push to the `releases` branch:

```bash
git add .
git commit -m "feat: vX.X.X"
git push origin master:releases
```

GitHub Actions will compile, sign, and publish the release automatically. Users with the app installed will see the update available in **Settings → About**.

---

## Tech stack

| Layer | Technology |
|---|---|
| Desktop shell | Tauri 2 |
| Frontend | SvelteKit 2 + Svelte 5 |
| Styling | SCSS |
| Language | TypeScript / Rust |
| Auto-update | tauri-plugin-updater |
| CI/CD | GitHub Actions |

---

## Author

Made by [Tito Saavedra](https://github.com/TitoSaavedra)

---

## License

MIT
