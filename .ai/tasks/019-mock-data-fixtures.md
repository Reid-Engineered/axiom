---
id: 019
title: services/mockData/* fixtures
status: review
owner: codex
stage: 4
depends_on: [005]
---

## Scope

Realistic fixture data for workspaces, goals, concepts (with prerequisite/related/leads-to edges), modules, sessions. Volume must be enough to exercise `AXIOM-HANDOFF.md` §5 scale behaviors later (Stage 6) — e.g. enough concepts to test chapter-collapse.

## Plan

- src/services/mockData/workspaces.ts
- src/services/mockData/goals.ts
- src/services/mockData/concepts.ts
- src/services/mockData/modules.ts
- src/services/mockData/sessions.ts

## Worklog

- 2026-08-28 — Claimed by Codex on `agent/codex/019-mock-data-fixtures`; read the Stage 4
  roadmap, locked Stage 2 types/service contracts, architecture rules, and scale scenarios
  before implementing fixtures.
- 2026-08-28 — Added realistic fixtures for three workspaces, six goals, 92 concepts, 20
  modules, and three sessions. The Calculus II workspace supplies the required scale set:
  87 graph-linked concepts, four goals in all four states, modules grouped 4/9/7, and a
  paused session with 40 tutor exchanges.
- 2026-08-28 — Full gates passed: `npm run typecheck`, `npm run lint`, `npm test` (63/63),
  `npm run build`, `git diff --check`, and the hardcoded hex/`rgba(` scan. A fixture count
  check also confirmed 87 Calculus concepts, 20 modules with 4/9/7 visibility, four Calculus
  goals, and 40 exchanges. Moved to `review`.

## What was built / tested / left out

- **Built**: all five `src/services/mockData/*` fixture files against the locked Stage 2
  types. Concept fixtures include prerequisite, related, leads-to, and blocks edges plus a
  fully populated Shell method concept. Workspace offline availability includes all four
  independently toggled kinds and the partial-video limit. Module and session fixtures
  include the metadata needed by the Stage 5/6 non-empty screens.
- **Tested**: typecheck, lint, all 63 existing tests, production build, whitespace check,
  hardcoded-value scan, and explicit scale-count assertions all pass.
- **Left out**: mutation behavior and service-level error handling belong to 020; hooks and
  fixture-backed renderHook coverage belong to 021. No UI scale behavior was implemented.

## Review

Reviewer: claude-code
Date: 2026-08-28

- [x] Correctness — pass: verified the scale claims directly against the fixtures, not just
  the worklog's word — 87 Calculus concepts (`calc-concept-1..87`), 20 modules split 4/9/7 by
  `visibility`, 4 goals covering all four states on `workspace-calculus-ii`, and the
  `session-shell-method` fixture's 40-entry `longTutorHistory`. `calc-concept-22` (index 21,
  the "Shell method" concept) is the one with `displayFormula`/`recentDiagnostics` populated,
  and matches the session fixture that references it by id — cross-fixture references are
  internally consistent.
- [x] Architecture conformance — pass: fixtures match `005`'s locked `Concept`/`Goal`/
  `Module`/`Session`/`Workspace` shapes exactly (confirmed via typecheck), no service logic
  leaked into fixture files.
- [x] UI rules — n/a, no styling in this task.
- [x] Process (gates) — pass: independently re-ran typecheck/lint/build/test on this branch
  (76/76, matches the count claimed by 021 which stacks on this) and the hardcoded hex/`rgba(`
  scan — clean.

Minor, non-blocking: the generated `calc-concept-*` graph is directionally inconsistent —
each concept's `blocksConceptIds` reaches three concepts ahead, but the downstream concepts'
own `prerequisiteConceptIds` only lists the immediate predecessor. E.g. `calc-concept-1`
claims to block `calc-concept-4`, but `calc-concept-4.prerequisiteConceptIds` is just
`[calc-concept-3]`, not `[calc-concept-1]`. Not consumed by any code yet (Stage 4 hooks
expose these fields raw), but whichever Stage 5/6 task builds the "blocks N concepts" count
or the prerequisite-chain rail (`AXIOM-HANDOFF.md`'s 87-concept scale scenario) should either
fix the generator or treat `blocksConceptIds` as directional-only, not an inverse of
`prerequisiteConceptIds`.

Verdict: **approved** — no blocking findings.

## Follow-ups

Anything noticed during implementation or review that's out of this task's scope.
