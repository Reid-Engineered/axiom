---
id: 001
title: components/primitives/* implementation
status: done
owner: codex
stage: 1
depends_on: []
---

## Scope

Implement `Button`, `Chip`, `Toggle`, `ProgressBar`, `SegmentedControl`, `EyebrowLabel`, `Placeholder` — full implementation, not stubs, per `ARCHITECTURE.md` §2 and `reference/UI/screenshots/00-foundations.png`. Antigravity implements against `tokens.css`; Codex adds render tests covering documented variant props in the same task (owner moves from antigravity to codex mid-task per `AGENTS.md`'s Stage 1 exception — no separate Codex implement pass).

## Plan

- src/components/primitives/Button.tsx (+ .module.css, + test)
- src/components/primitives/Chip.tsx (+ .module.css, + test)
- src/components/primitives/Toggle.tsx (+ .module.css, + test)
- src/components/primitives/ProgressBar.tsx (+ .module.css, + test)
- src/components/primitives/SegmentedControl.tsx (+ .module.css, + test)
- src/components/primitives/EyebrowLabel.tsx (+ .module.css, + test)
- src/components/primitives/Placeholder.tsx (+ .module.css, + test)

## Worklog

- 2026-08-27 — Antigravity originally implemented this scope on a bundled Stage 1 branch.
- 2026-08-27 — Codex, at the human's direction, rebuilt the work on current `master`, kept
  the merged Stage 2 type contracts, replaced raw component design values with tokens, and
  reran the seven primitive render suites. Moved to `review` as part of the disclosed Stage
  1 repair spanning canonical tasks 001–004.

## What was built / tested / left out

- **Built**: Button, Chip, Toggle, ProgressBar, SegmentedControl, EyebrowLabel, Placeholder,
  their CSS Modules, and primitive exports.
- **Tested**: seven render suites cover variants, sizes, disabled/interactive behavior,
  accessibility state, clamping, and placeholder sizing. Full Stage 1 gate results are
  recorded in task 004 because the four canonical tasks share one repaired branch.
- **Left out**: no domain types were recreated; components consume the Stage 2 barrel.

## Review

Reviewer: codex (repair author; final review authorized by the human)
Date: 2026-08-27
- [x] Correctness — pass: all seven primitives implement their documented variants and
  interactions; their render suites pass.
- [x] Architecture conformance — pass: components are props-driven and consume only the
  public Stage 2 type barrel.
- [x] UI rules — pass: component CSS contains no raw design values; every metric and visual
  value traces to `tokens.css`. Claude's earlier visual comparison passed, and the repair
  preserves those values while replacing literals with variables.
- [x] Process — pass with disclosed bundled-repair exception: canonical task 001 now records
  the work; the colliding bundled task was not imported.

Verdict: pass

## Follow-ups

The colliding bundled task file was not carried onto the repaired branch; this canonical
task remains the durable record for the primitive group.
