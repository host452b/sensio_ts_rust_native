# Sensio Taste Reconstruction Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Rebuild Sensio's library and reader UI around a refined private PDF library taste direction using LXGW Neo XiHei.

**Architecture:** Keep the existing React/Tauri behavior and state model. Improve markup semantics where needed, add repo-local font assets, and rebuild the CSS token system, components, responsive rules, and interaction states around the selected product direction.

**Tech Stack:** React 19, TypeScript, Vite, Tauri 2, CSS, PDF.js.

---

### Task 1: Font Asset And License

**Files:**
- Create: `src/assets/fonts/LXGWNeoXiHei.ttf`
- Create: `third_party/lxgw-neoxihei-v1.303/LICENSE.md`
- Create: `third_party/lxgw-neoxihei-v1.303/LICENSE_CHS.md`
- Modify: `src/styles/app.css`

- [x] Download `LXGWNeoXiHei.ttf` from `https://github.com/lxgw/LxgwNeoXiHei/releases/download/v1.303/LXGWNeoXiHei.ttf`.
- [x] Verify SHA-256 is `ba4b135dd03a50f25f690c72ab38eeac37cccbb79c133cd2229902a86e030376`.
- [x] Download upstream IPA Font License files from tag `v1.303`.
- [x] Add `@font-face` in `src/styles/app.css` with `font-display: swap`.
- [x] Set the root UI stack to `"LXGW Neo XiHei", "LXGWNeoXiHei", "PingFang SC", "Hiragino Sans GB", "Microsoft YaHei", system-ui, sans-serif`.

### Task 2: Library Markup Taste Pass

**Files:**
- Modify: `src/components/LibraryPage.tsx`

- [x] Replace the pixel-style brand mark content with a document-seal mark using text elements already available in CSS.
- [x] Add lightweight section headings and helper copy for import/export without changing command behavior.
- [x] Add metadata labels to document facts so size/date are visually ordered.
- [x] Keep all existing handlers and query keys unchanged.

### Task 3: Reader Markup Taste Pass

**Files:**
- Modify: `src/components/ReaderPage.tsx`
- Modify: `src/components/PdfViewer.tsx`

- [x] Add a `reader-back-button` class to the library navigation button.
- [x] Add a `reader-stage` wrapper around full reader `PdfViewer`.
- [x] Add semantic classes to zoom controls and page shells for refined toolbar and paper treatment.
- [x] Keep zoom, render, text selection, and highlight behavior unchanged.

### Task 4: CSS Reconstruction

**Files:**
- Modify: `src/styles/app.css`

- [x] Replace visual tokens with paper, ink, muted green, brass, and elevation tokens.
- [x] Rework sidebar, toolbar, segmented control, document card, empty state, preview, reader header, PDF toolbar, PDF page, and responsive rules.
- [x] Use stable dimensions and no viewport-scaled font sizes.
- [x] Add `prefers-reduced-motion` handling.
- [x] Keep cards at 8px radius or less.

### Task 5: Verification And Iteration

**Commands:**
- `npm run build`
- `npm run test:preferences`
- `npm run dev -- --host 127.0.0.1 --port 1421`
- `CARGO_NET_OFFLINE=true cargo test --offline --manifest-path src-tauri/Cargo.toml`
- `CARGO_NET_OFFLINE=true npm run tauri:build`

- [x] Build passes.
- [x] Preference tests pass.
- [x] Local server smoke check verifies HTML, CSS, and bundled font are served.
- [x] Rust tests pass.
- [x] Tauri production bundle builds at `src-tauri/target/release/bundle/macos/Sensio.app`.
- [x] Tauri bundle includes LXGW Neo XiHei license files under `Contents/Resources/licenses/lxgw-neoxihei-v1.303/`.
- [x] Directly launched release binary stays running as a runtime smoke check.
- [x] Dependency audit reports 0 vulnerabilities.
- [ ] Browser/window screenshot check verifies no incoherent overlap at desktop and mobile widths. Attempted through macOS `osascript`, but blocked by assistive-access permission for `osascript`.
- [x] Fix every detected issue and rerun the relevant verification.
