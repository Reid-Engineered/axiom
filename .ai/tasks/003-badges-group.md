---
id: 003
title: components/badges/* implementation
status: proposed
owner: antigravity
stage: 1
depends_on: [001]
---

## Scope

Implement `TrustBadge` (three levels), `OfflineChip`, `DiagnosticDot` — full implementation, per `ARCHITECTURE.md` §2 and `00-foundations.png`. Likely composes primitives from 001 (e.g. `Chip`). Antigravity implements; Codex adds render tests covering `TrustBadge`'s three levels and other variant props, same owner-handoff pattern as 001.

## Plan

- src/components/badges/TrustBadge.tsx (+ .module.css, + test)
- src/components/badges/OfflineChip.tsx (+ .module.css, + test)
- src/components/badges/DiagnosticDot.tsx (+ .module.css, + test)

## Worklog

## What was built / tested / left out

Filled in when moving to `review`.

## Review

Filled in by the reviewing agent. See `.ai/review-checklist.md`.

## Follow-ups

Anything noticed during implementation or review that's out of this task's scope.
