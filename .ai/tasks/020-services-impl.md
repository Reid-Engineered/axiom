---
id: 020
title: services/*Service.ts real implementations
status: review
owner: codex
stage: 4
depends_on: [006, 019]
---

## Scope

Real implementations against 019's mock data, matching 006's locked signatures exactly — no signature changes.

## Plan

- src/services/workspaceService.ts
- src/services/goalService.ts
- src/services/conceptService.ts
- src/services/moduleService.ts
- src/services/sessionService.ts

## Worklog

- 2026-08-28 — Claimed by Codex on `agent/codex/020-services-impl`, stacked on 019 at
  `6433066`. Re-read all five locked service contracts before replacing stub bodies.
- 2026-08-28 — Implemented all 24 locked async functions against 019's fixtures. Reads return
  clones; mutations validate workspace/domain identities, update the in-memory source, and
  return the locked domain shape. No service signature changed.
- 2026-08-28 — Full gates passed: `npm run typecheck`, `npm run lint`, `npm test` (63/63),
  `npm run build`, `git diff --check`, and the hardcoded hex/`rgba(` scan. Static contract
  count confirms all 24 async exports remain present. Moved to `review`.

## What was built / tested / left out

- **Built**: fixture-backed implementations for workspace, goal, concept, module, and session
  services. This includes scoped reads/search, goal edit/revert, per-kind offline toggles,
  module install/enable/visibility behavior, and the complete session lifecycle with tutor
  exchanges. All functions remain `async` and Promise-returning for the Stage 7 IPC swap.
- **Tested**: typecheck, lint, all 63 existing tests, production build, whitespace check,
  hardcoded-value scan, and an exact count of the 24 locked async exports.
- **Left out**: hook state/loading/error orchestration and renderHook tests belong to 021.
  Mock goal inference remains deliberately minimal (`inferred: {}` for a newly created
  workspace); real inference is a later backend concern and no locked inference contract exists.

## Review

Filled in by the reviewing agent. See `.ai/review-checklist.md`.

## Follow-ups

Anything noticed during implementation or review that's out of this task's scope.
