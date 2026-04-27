# Sensio

Local-first PDF library built with React 19, Vite, TypeScript, Tauri 2, Rust, SQLite FTS5, and PDF.js.

## Stack

- UI: React 19, Vite, TypeScript
- UI state: Zustand
- Server/app data: TanStack Query calling typed Tauri commands
- Desktop backend: Tauri 2, Rust, tokio
- IPC typing: `tauri-specta` / `specta`
- Database: bundled SQLite with FTS5 `unicode61` and trigram search
- PDF reader: `pdfjs-dist` Display API, canvas pages, text layer, custom highlight overlay
- PDF export: `lopdf`, always writes a copy and never modifies the original file
- Storage: filesystem PDF copies plus SQLite metadata

## Run Locally

```bash
npm install
npm run tauri:dev
```

When dependency commands need network access, clear stale proxy variables first:

```bash
env -u HTTP_PROXY -u HTTPS_PROXY -u ALL_PROXY -u http_proxy -u https_proxy -u all_proxy npm install
env -u HTTP_PROXY -u HTTPS_PROXY -u ALL_PROXY -u http_proxy -u https_proxy -u all_proxy cargo check
```

## Build And Test

```bash
npm run build
npm run test:preferences
CARGO_NET_OFFLINE=true cargo test --offline --manifest-path src-tauri/Cargo.toml
CARGO_NET_OFFLINE=true npm run tauri:build
```

The macOS app bundle is produced at:

```text
src-tauri/target/release/bundle/macos/Sensio.app
```

## Current Scope

- Library shell with Chinese UI, grid/list view, persisted sort and layout preferences.
- Import PDF by file picker or path. Imports copy the PDF into the local library.
- Search and list documents through SQLite metadata and FTS indexes.
- Open reader windows through Tauri commands.
- Render original PDFs through PDF.js with selectable text and persisted highlight rectangles.
- Export a separate PDF copy through Rust without mutating the original file.

## Storage Model

The app stores `library.json` under the Tauri app data directory. That bootstrap file only points to the active library path.

Inside the library, `AppState.db` owns document metadata, highlights, search indexes, tags, and persisted app preferences such as `layoutMode` and `sortKey`. PDF files live on the filesystem and are referenced by SQLite metadata.
