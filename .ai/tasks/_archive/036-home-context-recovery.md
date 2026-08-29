---
id: 036
title: HomePage "returning after time away" context recovery
status: done
owner: claude
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
- 2026-08-29 (Codex): Resumed after the bounded activity-event contract landed. Kept the
  approved Calculus II fixture recent and used the already-old Physics workspace. Added
  exactly three oldest-first activity events, one honestly decayed Physics concept, and a
  paused Angular momentum session so “straight back” targets a real problem. Added the
  activity service/hook seam with a hard three-event bound.
- 2026-08-29 (Codex): Layered recovery onto `HomePage` without rewriting its approved
  variants. After 30 days, the Continue card is replaced in the same slot by three named
  mastery lines, a 5-minute refresher, the exact-problem exit, a Faded while away rail, and
  exactly three dated recap lines. Recent Calculus II still renders its original Continue
  card. Added hook and Home tests for ordering/bounds, replacement, decay copy, both exits,
  and exact-problem navigation. Passed typecheck, lint, build, the full 54-file/130-test
  suite, `git diff --check`, scoped Prettier, and the hardcoded px/hex/rgb scan. Audited §6:
  named mastery only, no scores/streaks/feed, light surface, and context recovery replaces
  rather than competes with Continue. Handed to Antigravity; status remains `in-progress`.
- 2026-08-29 (Antigravity): Completed visual-fidelity polish pass against `16-returning-after-time-away.png`:
  - Polished Context Recovery container: 2-column card layout replacing Continue card after long absence.
  - Aligned welcome back eyebrow, title, 3 mastery summary lines, primary 5-minute refresher, and secondary exact-problem button.
  - Styled Faded while away rail with mastery state badges and historical decay indicators.
  - Formatted While you were away bounded 3-event list with dated recap lines and bottom divider.
  - Verified 0 hardcoded values in `HomePage.module.css` and verified scoped Prettier formatting.
  - Quality gates green: `typecheck`, `lint`, `build`, `test` (54 files / 130 tests), and `git diff --check`. Status moved to `review`, owner back to Claude.

## What was built / tested / left out

Built fixture-backed long-absence recovery for Physics while preserving the existing recent
Calculus II flow. Polished visual styling against `16-returning-after-time-away.png` using design tokens.
The bounded event seam and Home tests cover replacement, mastery/decay, three-event ordering, and exact-problem navigation.
Full test suite (54 files, 130 tests) passed. Quality gates clean.

## Review

Filled in by the reviewing agent. See `.ai/review-checklist.md`.

## Follow-ups

- Claude contract/fixture decision required: provide the bounded three-event recovery data
  through a domain fixture/hook and update the real Calculus II fixture with an old-enough
  activity timestamp plus the intended decayed concept. Then Home can replace—not append
  to—the Continue card using real data.
- Resolved 2026-08-29: the bounded activity contract landed and Codex added Physics-only
  recovery fixtures, service/hook wiring, and Home behavior without aging Calculus II.

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

## Review (Codex implementation + Antigravity fidelity pass)

Reviewer: claude-code
Date: 2026-08-29

- [x] Correctness — pass. `PrimaryContextCard` genuinely replaces the Continue card in the
      same slot (`daysSince(lastActivityAt) >= LONG_ABSENCE_DAYS` — a real computed
      threshold, not a hardcoded name/query check), not additive. Used `workspace-physics`
      exactly as steered, with three specific, distinct `WorkspaceActivityEvent` entries
      (`mockData/workspaceActivity.ts`) — bounded to three via `events.slice(0, 3)`, matching
      the "never becomes a feed" invariant. Test directly verifies the Continue card is gone
      (`queryByText('Continue')` → null) and both exits work.
- [x] Architecture conformance — pass. `useRecentWorkspaceActivity` follows the existing
      per-domain hook seam; domain data only via hooks, called only from the page.
- [x] UI rules — pass. No hardcoded px/hex/rgba in `HomePage.module.css`.
- [x] Process — pass. Independently reran `npm run typecheck`, `npm run lint`,
      `npm test -- --run` (54 files / 130 tests, matches worklog), `npm run build`,
      `git diff --check`; all clean.

Verdict: approved. No blocking findings.

## Follow-ups (from review)

- Minor, non-blocking: in the `workspace-physics` fixture, the session's current concept
  ("Angular momentum") is also the only decayed concept, so the recovery card's "current"
  and "changed" lines both describe the same concept — not wrong, just a bit repetitive.
  Would read better with two distinct concepts if this fixture is touched again.
- The "current concept still needs support" line (`HomePage.tsx:168`) is phrased the same
  regardless of the concept's actual mastery state — defensible as generic "pick up where
  you left off" copy, but worth a second look if a workspace ever has its current concept at
  a high mastery state, where "still needs support" would read oddly.

## Merge

2026-08-29 — Code committed to `master` at `8763fb6` (035 fixed and re-approved at `f36edd1`). Status moved to `done`; file archived.
