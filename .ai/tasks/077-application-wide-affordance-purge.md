---
id: 077
title: Application-wide affordance purge
status: proposed
owner: unassigned
stage: 8
depends_on: []
---

## Scope

Deferred follow-up from §7.2 of
`docs/superpowers/specs/2026-09-09-mock-and-dead-end-containment-design.md`, which carries the
full inventory with file:line references.

Everything the containment investigation found that is **not** on the beta-critical path:
production-reachable UI presenting fabricated history, evidence, or affordances with no
destination. Real findings, but not consequences of the Windows walk, and deliberately kept
out of `070`–`075` so the beta path stays reviewable.

**Ordered by severity, led by the one that writes to the database:**

1. **The canned tutor answer.** `add_tutor_exchange_handler`
   (`src-tauri/src/commands/session.rs:328`) persists a fixed sentence as the tutor's reply,
   indistinguishable in `tutor_exchanges` from a real one. The only defect in the whole
   inventory that writes fiction into the learner's own record. Do this first.
2. **Seed hygiene beyond sessions** — mastery states above `New`, `recentDiagnostics`,
   `learnerHeuristic` / `heuristicEvidence`, `notes`, `workspaceActivity` events. All claim
   learner activity. `072` removed only the seeded sessions and the two workspace columns the
   boot ordering reads.
3. **`FullVisualizationPage`** — `fullVisualizationScene.ts` is the only scene it ever renders,
   contradicting whatever real problem is open, plus "contributes about 6% of the total volume".
4. **`WorkspaceOverviewPage`** — the hardcoded "Recent" list, `ReasonedRecommendation`'s
   fabricated Tuesday/Thursday evidence and its "Start · 8 min" destination, `conceptStatus()`
   returning "2 days ago" by array index, `selectConceptsInPlay()`'s four hardcoded names.
5. **`CreateWorkspacePage`** — the "Axiom read that as" inference chips and the inert comfort /
   materials / pacing controls.
6. **Remaining dead buttons and defaults** — `useCommandPalette`'s `workspace-calculus-ii`
   default; "Use template", "Try it first", "Add to another workspace", "See in concept map",
   "Save to concept notes", "Pin to notes", "Share", "N more notes".

## Plan

Not planned as one task, and should not be implemented as one. It spans six pages and three
ownership lanes; several items diverge visibly from `reference/UI/AXIOM-HANDOFF.md`
screenshots (the sample workspace shown mid-journey, Screen 5's tutor panel); and dropping
seeded mastery states is a product decision about what a sample workspace is *for*, not a bug
fix.

Expect three or four small tasks once the shape is agreed. Item 1 is separable and worth
filing on its own immediately.

Nothing here is pulled into `070`–`075` unless `074`'s native regression directly reaches it.

## Worklog

- 2026-09-09 — Filed by claude from the approved design, held outside the beta-critical path
  by explicit decision.
- 2026-09-10 — The `useCommandPalette` `workspace-calculus-ii` default and the corresponding missing-workspace query/navigation guards moved to task `070`. Startup restoration cannot remove the production fallback honestly while the hook silently recreates it; the remaining dead buttons and defaults stay in this follow-up.

## What was built / tested / left out

Not started.

## Review

## Follow-ups
