---
id: 027
title: FullVisualizationPage inert placeholder
status: proposed
owner: codex
stage: 6
depends_on: [021]
---

## Scope

Static/inert placeholder — no real 3D engine (Stage 8). Must still satisfy "composed from verified primitives, not generated as images" (`AXIOM-HANDOFF.md` §4, Screen 6). Treat the placeholder's data shape (coordinate system, function, region, axis, revolution, shell, annotation) as real API surface now so Stage 8's engine slots in without a page-level rewrite.

## Plan

- src/pages/FullVisualizationPage.tsx
- associated .module.css
- placeholder primitive data-shape types (coordinate system, function, region, axis, revolution, shell, annotation)

## Worklog

## What was built / tested / left out

Filled in when moving to `review`.

## Review

Filled in by the reviewing agent. See `.ai/review-checklist.md`.

## Follow-ups

Anything noticed during implementation or review that's out of this task's scope.
