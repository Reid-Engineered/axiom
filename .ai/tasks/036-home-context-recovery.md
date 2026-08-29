---
id: 036
title: HomePage "returning after time away" context recovery
status: in-progress
owner: codex
stage: 6
depends_on: [024]
---

## Scope

Layer the context-recovery scale behavior from `AXIOM-HANDOFF.md` §5 onto 024's `HomePage` implementation.

## Plan

- src/pages/HomePage.tsx (behavior addition on top of 024)

## Worklog

- 2026-08-29 (Codex): Claimed the behavior pass and confirmed no other in-progress task
  touches `HomePage` context recovery or workspace activity timestamps. Reviewed Screen 16
  and all §6 invariants.
- 2026-08-29 (Codex): Data audit confirmed long absence can derive from
  `Workspace.lastActivityAt`, while named decay can derive only when a `Concept` has
  `wasMasteryState`/`decayedAt`. The fixture currently has no decayed Trig Substitution
  entry, and there is no modeled source for the required three dated “While you were away”
  events. Paused rather than hardcoding an activity feed inside `HomePage`.

## What was built / tested / left out

Filled in when moving to `review`.

## Review

Filled in by the reviewing agent. See `.ai/review-checklist.md`.

## Follow-ups

- Claude contract/fixture decision required: provide the bounded three-event recovery data
  through a domain fixture/hook and update the real Calculus II fixture with an old-enough
  activity timestamp plus the intended decayed concept. Then Home can replace—not append
  to—the Continue card using real data.

- 2026-08-29 (claude-code): Contract added — `WorkspaceActivityEvent`
  (`src/types/workspace.ts`): `{ id, workspaceId, occurredAt, summary }`, deliberately
  bounded (doc comment says so explicitly) — callers show at most three, oldest first, no
  pagination, matching "a semester never becomes a feed." Re-exported from `types/index.ts`,
  added to `ARCHITECTURE.md`'s type table.
  Did **not** touch `workspace-calculus-ii`'s `lastActivityAt` — it's `2026-08-27` (2 days
  ago) and `HomePage.test.tsx`'s existing default-variant test renders against exactly that
  workspace expecting the normal Continue card; aging it would break an already-approved
  test. Use **`workspace-physics`** instead for this scale behavior —
  `lastActivityAt: '2026-05-12T16:45:00.000Z'` (~3.5 months old) is already sitting there in
  the fixture, unused by anything else, and reads like it was seeded in Stage 4 anticipating
  exactly this. `wasMasteryState`/`decayedAt` already exist on `Concept` (unused so far) —
  pick one of workspace-physics's own concepts to decay, no need to reuse Calculus II's
  "Trig substitution" specifically; the screen's exact name is illustrative, not a fixture
  requirement.
  Unblocked — what's left is fixture content, not architecture: decay one workspace-physics
  concept, add 2-3 `WorkspaceActivityEvent` entries for workspace-physics (bounded to what's
  actually shown — three, per the invariant above), and a service/hook to fetch them
  (`getRecentActivity(workspaceId): Promise<WorkspaceActivityEvent[]>` fits the existing
  seam). The main recovery card's "three lines tied to mastery states" (what held/didn't/
  changed) and the "Faded while away" rail are both derivable from existing `Concept` fields
  once a workspace has decayed concepts — that's page-level selection logic, not a new type.
