---
id: 001
title: components/primitives/* implementation
status: proposed
owner: antigravity
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

## What was built / tested / left out

Filled in when moving to `review`.

## Review

Filled in by the reviewing agent. See `.ai/review-checklist.md`.

## Follow-ups

Anything noticed during implementation or review that's out of this task's scope.
