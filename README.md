# The Artist's Image

A desktop app that turns an album cover image plus lyrics into ASCII art. Pick
a cover, paste (or fetch) the lyrics, choose a font, and export the result as
PNG, JPEG, or SVG.

Built with [Tauri 2](https://tauri.app/) and [SvelteKit](https://kit.svelte.dev/).
The Rust backend handles image sampling, grid layout, and glyph rasterization.
The Svelte frontend drives the workflow and renders a live preview.

## Features

- Drag-and-drop image input with any standard format
- Lyrics from manual paste, `.lrc` / `.txt` file, or fetched from AZLyrics
- Three render methods: **Average**, **Absolute**, **Sharp**
- Resolution-independent grid layout — preview and export share the same grid
- Auto-fit font sizing driven by lyrics length
- Pixel-gap line and char spacing, decoupled from font size
- Transparent space characters
- Live zoomable preview
- Export to PNG, JPEG, or SVG at any resolution

## Getting Started

### Prerequisites

- [Node.js](https://nodejs.org/) 18+
- [Rust](https://www.rust-lang.org/tools/install) (stable)
- Platform deps for [Tauri 2](https://v2.tauri.app/start/prerequisites/)
- Python 3 + `scrapling[fetchers]` (only required for the lyrics-fetch sidecar)

### Install

```bash
npm install
```

### Run in development

```bash
npm run tauri dev
```

This starts Vite for the frontend and launches the Tauri shell.

### Build a release

```bash
npm run tauri build
```

Artifacts are written to `src-tauri/target/release/bundle/`.

## Project Layout

```
src/                  SvelteKit frontend (UI, stores, components)
src-tauri/            Rust backend (Tauri commands, ASCII renderer, export)
  src/ascii/          Grid generation, color sampling, layout
  src/export/         PNG / JPEG / SVG writers
  src/fonts.rs        System font enumeration
  sidecar/            Python AZLyrics scraper (bundled as a Tauri resource)
```

## Lyrics Fetching

The "fetch" tab in the lyrics panel runs a small Python sidecar
(`src-tauri/sidecar/scraper.py`) that scrapes AZLyrics. It is built into the
app via `build.rs` and invoked from Rust over the Tauri sidecar protocol.

```bash
pip install -r src-tauri/sidecar/requirements.txt
```

## License

MIT
