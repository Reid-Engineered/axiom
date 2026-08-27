---
id: 005
title: src/types/* domain model
status: done
owner: claude-code
stage: 2
depends_on: []
---

## Scope

Populate `src/types/` per `ARCHITECTURE.md` §4, matching `AXIOM-HANDOFF.md` §1 exactly: all five mastery states, all goal states, offline statuses, trust levels. This is the first thing locked in Stage 2 — everything else (service signatures, component props, mock fixtures) is written against it. No dependency on Stage 1 primitives.

## Plan

- src/types/common.ts
- src/types/workspace.ts
- src/types/goal.ts
- src/types/concept.ts
- src/types/module.ts
- src/types/session.ts
- src/types/index.ts (barrel re-export)

## Worklog

- 2026-08-27 — started, claimed by claude-code. Branch `agent/claude-code/005-types`.
- 2026-08-27 — read `AXIOM-HANDOFF.md` §1 (product model), §4 (screens 1-12), §5 (scale
  behaviour) in full to ground every field in a specific screen, not invented shape.
- 2026-08-27 — wrote all 7 files. Quality gates run (typecheck, lint, build, hardcoded-value
  grep) — all pass. Moved to `review`.

## What was built / tested / left out

- **Built**: `src/types/common.ts` (the four fixed enums, unchanged from
  `ARCHITECTURE.md` §4's naming), `workspace.ts`, `goal.ts`, `concept.ts`, `module.ts`,
  `session.ts`, `index.ts` barrel. Every field traces to specific handoff text — see inline
  TSDoc comments citing the source screen where the field isn't self-explanatory.
- **Tested**: `npm run typecheck` (0 errors), `npm run lint` (0 errors), `npm run build`
  (succeeded), grep for stray hex/`rgba(` in `src/types` (0 hits — not that a types-only
  change was likely to introduce any). No component/hook tests apply — this task touches
  neither (`.ai/quality-gates.md`'s "Components and hooks" gates don't apply here).
- **Left out / notable decisions**:
  - `Workspace.progress` is a 0-1 fraction, kept purely to drive `ProgressBar`'s fill width —
    never rendered as text, so it doesn't violate the "no percentages" invariant
    (`AXIOM-HANDOFF.md` §6.3).
  - Composite, screen-specific view-models (e.g. Home's Continue card) were deliberately
    **not** modeled as their own types — `Session.resumeSummary` + `Session.thumbnailUrl`
    plus the related `Concept`/`Workspace` give a hook everything it needs to compose that
    view later (Stage 4), consistent with `ARCHITECTURE.md` §4's "nothing else" scope for
    `types/`.
  - `SessionIntent.activity` is `string`, not a fixed union — the handoff text gives
    "Practising"/"Reading"/"Exploring" as examples ("the current activity (...)"), not an
    exhaustive enumeration, unlike `MasteryState`/`GoalState` which are explicitly closed
    lists.
  - Did not touch `src/components/*` — the pre-existing `src/types/common.ts` /
    `src/types/index.ts` on `agent/antigravity/001-design-system-primitives` (added
    out-of-lane in that task, flagged in its review) will collide with this on merge; that
    branch needs to rebase onto this once both land, dropping its own copy in favor of this
    barrel. Not fixed here — not this task's branch to edit.

## Review

Reviewer: codex
Date: 2026-08-27
- [x] Correctness — pass: the six domain interfaces and four fixed unions match the
  product model and provide the screen-backed fields claimed in the handoff.
- [x] Architecture conformance — pass: types live in the prescribed files and are all
  re-exported through `src/types/index.ts`; no data flow or global state was introduced.
- [x] UI rules — pass: the types preserve named mastery states, learner-facing offline
  language, and module trust levels without introducing UI markup or design values.
- [x] Process — pass: scope matches task 005, the worklog records concrete decisions, and
  independent runs of `npm run typecheck`, `npm run lint`, `npm run build`, and the
  hardcoded-value grep all passed.

Verdict: pass

## Follow-ups

Anything noticed during implementation or review that's out of this task's scope.
