---
id: 004
title: /dev/gallery primitive gallery route
status: done
owner: codex
stage: 1
depends_on: [001, 002, 003]
---

## Scope

A temporary route rendering every Stage 1 primitive in every documented variant, for visual side-by-side against `00-foundations.png` and `15-system-refinements.png`. Must be removed before Stage 8 or gated behind a dev-only build flag — does not ship in a production build. Does not add any new primitive or variant; consumes what 001-003 built.

## Plan

- src/pages/dev/GalleryPage.tsx (or equivalent dev-only route)
- wiring to reach it only in dev builds (see AppShell/App.tsx entry point)

## Worklog

- 2026-08-27 — Antigravity originally implemented the gallery on a bundled Stage 1 branch;
  review found that it became the production default route and that the bundled task id
  collided with canonical tasks 001–004.
- 2026-08-27 — Codex rebuilt the gallery on current `master`, gated rendering with
  `import.meta.env.DEV`, restored the empty AppShell as the default route, removed inline
  spacing, added route render coverage, and preserved these four canonical task records.

## What was built / tested / left out

- **Built**: DevGalleryPage and CSS Module plus hash routing in App. Every Stage 1 component
  and documented variant is represented.
- **Tested**: App render tests cover the empty default shell and the development gallery
  hash. Production build inspection finds no gallery heading or route markers. Across tasks
  001–004: typecheck, lint, 13 test files / 40 tests, production build, hardcoded color/rgba
  grep, raw CSS design-value grep, and diff check pass.
- **Left out**: permanent navigation; the gallery remains a temporary dev-only surface.

## Review

Reviewer: codex (repair author; final review authorized by the human)
Date: 2026-08-27
- [x] Correctness — pass: the empty AppShell remains the default; the gallery renders only
  for `#/dev/gallery` in development and shows every Stage 1 component variant.
- [x] Architecture conformance — pass: the temporary route adds no routing library or global
  domain state.
- [x] UI rules — pass: gallery styling is fully token-backed; its production markers are
  removed by the build-time development guard.
- [x] Process — pass with disclosed bundled-repair exception: 13 test files / 40 tests,
  typecheck, lint, build, color/rgba grep, numeric CSS design-value grep, production marker
  inspection, and diff check pass. Canonical tasks 001–004 replace the colliding bundled
  task record.

Verdict: pass

## Follow-ups

The colliding bundled task file was not carried onto the repaired branch. Tasks 001–004 are
the sole Stage 1 records, and this task records the cross-task acceptance gates.
