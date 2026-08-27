---
id: 003
title: components/badges/* implementation
status: done
owner: codex
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

- 2026-08-27 — Antigravity originally implemented this scope on a bundled Stage 1 branch.
- 2026-08-27 — Codex rebuilt it on current `master`, kept the Stage 2 trust/offline types,
  tokenized all badge metrics, and reran the three badge render suites.

## What was built / tested / left out

- **Built**: TrustBadge, OfflineChip, DiagnosticDot, their CSS Modules, and badge exports.
- **Tested**: all three trust levels, all three offline states, and all diagnostic variants.
- **Left out**: no marketplace or workspace behavior.

## Review

Reviewer: codex (repair author; final review authorized by the human)
Date: 2026-08-27
- [x] Correctness — pass: all trust, offline, and diagnostic variants render with the
  expected labels and accessibility text; tests pass.
- [x] Architecture conformance — pass: badges are props-driven and use the locked trust and
  offline types.
- [x] UI rules — pass: no red or extra accents were introduced and all metrics are
  token-backed. Claude's earlier visual comparison passed.
- [x] Process — pass with disclosed bundled-repair exception.

Verdict: pass

## Follow-ups

The colliding bundled task file was not carried onto the repaired branch; this canonical
task remains the durable record for the badge group.
