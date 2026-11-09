# Pomme Website (Community Redesign)

An unofficial community redesign and static site implementation for [Pomme](https://github.com/PommeMC/Client) — the from-scratch, native Minecraft client written in Rust & Vulkan.

> **Notice:** This repository is an independent community redesign of [pomme.rs](https://pomme.rs) created by [purdze](https://github.com/Purdze), [sqrilizz](https://github.com/sqrilizz) and contributors. It is not the official Mojang or Microsoft project.

## Architecture & Features

- **Native Rust Static Generator**: The entire site is compiled by a custom Rust generator in [`builder/`](builder/) with zero Node.js/Bun dependencies.
- **Live Upstream Sync**: Fetches live GitHub release tags, binary checksums (SHA-256), contributors, and recent commits directly from `PommeMC/Client` with automatic offline cache fallback (`builder/cached_data.json`).
- **In-Engine 60 FPS Presentation**: Includes high frame rate optical flow interpolated gameplay capture (PR #717) showcasing hardware Vulkan frame presentation.
- **Granular Vanilla Parity Matrix**: Live status tracker across 1,000+ mobs, blocks, items, redstone mechanics, and rendering systems, backed by `data/parity.json` and real test harnesses.
- **Replayable Cold Start Benchmark**: Interactive client-side race visualizer comparing Pomme's ~1.4s launch time against Vanilla Java (11.8s) and Lunar Client (14.5s).
- **Fast & Accessible**: Clean semantic HTML5, zero CSS frameworks, progressive enhancement JS (`assets/app.js`), full responsive design for desktop and mobile devices.

## Repository Layout

```
.
├── Cargo.toml          # Workspace manifest
├── builder/            # Rust static generator crate
│   ├── Cargo.toml
│   ├── src/main.rs     # Page templates & GitHub API sync
│   └── cached_data.json# Offline snapshot for rate-limit protection
├── assets/             # Raw stylesheets, vanilla JS, images, fonts
│   ├── style.css       # Clean design system
│   ├── app.js          # Lightweight interactive enhancements
│   └── images/         # 60 FPS gameplay capture & media
├── data/
│   └── parity.json     # Vanilla Minecraft parity database
├── scripts/
│   └── sync_parity.py  # Upstream parity sync utility
└── dist/               # Generated static output (HTML, assets, sitemap)
```

## Quick Start

### Prerequisites
- [Rust toolchain](https://www.rust-lang.org/tools/install) (stable)

### Build the static site
```bash
cargo run
```
The production bundle will be generated into `dist/`.

### Run local preview server
```bash
cargo run -- serve
```
Or specify a custom port:
```bash
cargo run -- serve --port 8899
```

## Deployment

The static output in `dist/` is ready for any host:
- **GitHub Pages**: Automatically deployed via GitHub Actions ([`.github/workflows/pages.yaml`](.github/workflows/pages.yaml)).
- **Cloudflare Pages**: Point build output to `dist/`. Security headers and redirects are included automatically via `_headers` and `_redirects`.

## License

GPL-3.0. See [LICENSE](LICENSE) for details.
