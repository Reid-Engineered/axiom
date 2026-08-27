---
id: 004
title: /dev/gallery primitive gallery route
status: proposed
owner: antigravity
stage: 1
depends_on: [001, 002, 003]
---

## Scope

A temporary route rendering every Stage 1 primitive in every documented variant, for visual side-by-side against `00-foundations.png` and `15-system-refinements.png`. Must be removed before Stage 8 or gated behind a dev-only build flag — does not ship in a production build. Does not add any new primitive or variant; consumes what 001-003 built.

## Plan

- src/pages/dev/GalleryPage.tsx (or equivalent dev-only route)
- wiring to reach it only in dev builds (see AppShell/App.tsx entry point)

## Worklog

## What was built / tested / left out

Filled in when moving to `review`.

## Review

Filled in by the reviewing agent. See `.ai/review-checklist.md`.

## Follow-ups

Anything noticed during implementation or review that's out of this task's scope.
