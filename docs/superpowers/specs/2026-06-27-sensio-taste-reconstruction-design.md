# Sensio Taste Reconstruction Design

## Decision Method

When a product decision is needed, Sensio uses three domain-relevant options, scores them, and executes the highest-scoring option. Scores are 1-5, where 5 is best.

| Dimension | Meaning |
| --- | --- |
| Craft | Material, color, proportion, edge radius, thickness, gloss, and finish |
| Feel | Press response, scroll behavior, selection feedback, and perceived tactility |
| Order | Typography, layout, icon language, spacing, and motion hierarchy |
| Emotion | Whether the product feels calm, reassuring, pleasant, and close to the user |
| Brand | Whether Sensio feels opinionated, tasteful, private, and recognizable |
| Generality | Whether the direction works across library, reader, empty, loading, and error states |
| Universality | Whether it remains understandable and usable for broad Chinese PDF workflows |

## Top 3 Routes

| Route | Craft | Feel | Order | Emotion | Brand | Generality | Universality | Total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Refined private PDF library | 5 | 5 | 5 | 5 | 5 | 5 | 5 | 35 |
| Quiet premium reader | 5 | 4 | 5 | 5 | 3 | 4 | 5 | 31 |
| Professional knowledge workbench | 4 | 4 | 5 | 3 | 4 | 5 | 4 | 29 |

Selected route: refined private PDF library.

## Product Direction

Sensio should feel like a refined local PDF cabinet: private, precise, calm, and deliberate. The UI should make documents feel collected and cared for, while preserving the current practical flows of import, search, selection, preview, reading, zoom, highlight, and export.

## Visual System

Appearance uses paper white, warm graphite ink, muted green, and small brass accents. Corners stay at 8px or below. Shadows should suggest paper thickness and physical stacking, not floating marketing cards. The existing pixel-like brand mark is replaced with a restrained document seal.

Typography uses `LXGW Neo XiHei` as the primary UI font, bundled locally from release `v1.303`. The standard asset `LXGWNeoXiHei.ttf` is used instead of the Plus variant because it directly matches the requested family and keeps bundle size lower. The upstream IPA Font License files are stored under `third_party/lxgw-neoxihei-v1.303/`.

## Interaction System

Primary controls have at least 44px height where space allows. Pressed states use a small visual depression through transform and color, without layout-shifting neighbors. Document selection should feel like drawing a document forward: a clear border, soft paper tone, and restrained elevation. Motion stays between 150ms and 220ms and respects reduced motion.

## Layout Scope

The current three-region desktop structure remains:

- Sidebar: library controls, search, import, export, and library path.
- Collection: document grid/list with stable cards and sort/view controls.
- Reading surface: selected-document preview or full reader, with lighter toolbars and page-focused canvas presentation.

Mobile keeps a single-column flow with visible controls, no horizontal scroll, and stable document card dimensions.

## Deliverable Criteria

- UI uses the bundled `LXGW Neo XiHei` font through `@font-face`.
- Library, preview, reader, loading, empty, success, and error states share the same taste direction.
- Existing behavior remains intact: import path, export path, search, sort, layout persistence, selection, open reader, zoom, text selection, highlight persistence.
- `npm run build` passes.
- `npm run test:preferences` passes.
- A local runtime smoke check can load the app without obvious layout breakage.
