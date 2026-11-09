# pomme-website

Community redesign of the [pomme.rs](https://pomme.rs) website for [Pomme](https://github.com/PommeMC/Client) (the native Minecraft client written from scratch in Rust & Vulkan).

> **Note:** This is an independent community redesign created by [sqrilizz](https://github.com/sqrilizz) and contributors. Not an official Mojang or Microsoft project.

## Preview
Live preview: **[https://sqrilizz.github.io/pomme-website/](https://sqrilizz.github.io/pomme-website/)**

## Why rewrite?

![EVERYTHING RUST](assets/images/purdze_endorsement.png)

The original site was an Astro/Node/Bun setup with heavy dependencies. This remake replaces the entire pipeline with a clean, standalone Rust static site generator:

- **Zero npm / Node / Bun (cuz why not XD)**: compiled and generated entirely in Rust with `cargo run`.
- **In-engine gameplay capture**: 60 FPS optical flow interpolated footage directly from native client builds.
- **Vanilla Parity matrix**: tracks mobs, blocks, redstone, and rendering against upstream client test scripts.
- **Cold start benchmark**: terminal-style race comparison showing Pomme's 1.4s launch time.
- **Live upstream sync**: automatically pulls real releases, SHA-256 hashes, commits, and contributors from `PommeMC/Client` with offline cache fallback.

## Quick Start

```bash
# Build static site to dist/
cargo run

# Build and start local preview server on http://localhost:4321
cargo run -- serve
```

## Structure
- `builder/` — Rust static site generator.
- `assets/` — CSS, client JS, images, 60 FPS gameplay capture.
- `data/` — Vanilla Minecraft parity dataset (`parity.json`).
- `scripts/` — Upstream parity sync tool.
- `dist/` — Compiled production output.

## License
GPL-3.0.
