---
id: 006
title: src/services/* function signatures
status: done
owner: claude-code
stage: 2
depends_on: [005]
---

## Scope

Write `services/*Service.ts` function signatures against the types locked in 005 — real return types (`Promise<T>`, per `ARCHITECTURE.md` §5 rule 2), bodies `throw new Error('not implemented')`. Locks the contract Stage 4's hooks build against. No real logic or mock data reads yet.

## Plan

- src/services/workspaceService.ts
- src/services/goalService.ts
- src/services/conceptService.ts
- src/services/moduleService.ts
- src/services/sessionService.ts

## Worklog

- 2026-08-27 — started, claimed by claude-code. Branch `agent/claude-code/006-services-signatures`.
- 2026-08-27 — wrote all 5 service files against 005's types, grounding each function in a
  specific screen action rather than a generic CRUD guess.
- 2026-08-27 — `npm run typecheck` passed, but `npm run lint` failed: this repo's ESLint
  config didn't recognize the `_`-prefix convention TS itself uses for intentionally-unused
  params (needed here because every function is `throw new Error('not implemented')` per
  this stage's own spec). Added `argsIgnorePattern: '^_'` to `@typescript-eslint/no-unused-vars`
  in `eslint.config.js` — every remaining Stage 2 stub task hits this same wall, so fixed it
  at the config level instead of finding a per-file workaround. All gates re-run and pass.
  Moved to `review`.
- 2026-08-27 — Codex reviewed, verdict `changes-requested`, 3 findings. Verified all three
  against the actual code before touching anything (not taken on faith):
  1. Confirmed `setWorkspaceOfflineAvailability(id, enabled)` was a single boolean against
     screen 21's four independent per-kind toggles (textbook / problem banks / visual assets
     / module data). This traced back further than the service signature — `Workspace` (005)
     never modeled per-kind offline state at all, just one `OfflineStatus` + a size. Revised
     `src/types/workspace.ts`: added `OfflineContentKind` and `OfflineKindAvailability`,
     replaced `offlineStatus`/`offlineSizeBytes` with `offlineAvailability:
     OfflineKindAvailability[]` (toolbar chip and sheet total are both derivable from this,
     not separately stored). Updated `setWorkspaceOfflineAvailability` to take a `kind` param,
     one call per toggle — matching `setModuleEnabled`'s existing per-item style. Grepped the
     whole tree for other consumers of the removed fields before changing them — none exist
     yet (nothing past Stage 2 is built), so this didn't ripple anywhere. Full `tsc --noEmit`
     re-run to confirm.
  2. Confirmed `Module.visibility` had no mutator. Added `setModuleVisibility(workspaceId,
     moduleId, visibility)` to `moduleService.ts`, mirroring `setModuleEnabled`'s shape.
  3. Recounted: 23 functions were actually exported, not the 21 the worklog claimed — my
     error, not Codex's. After fix 2 it's 24; corrected below.
  All gates re-run (typecheck, lint, build, hardcoded-value grep) — pass. Moved back to
  `review`.
- 2026-08-27 — Codex re-reviewed, verdict `changes-requested` again: `OfflineContentKind`
  didn't match the authoritative screenshot. Before touching code, actually opened
  `screenshots/21-offline-modules-goals.png` as an image — I'd never viewed it directly;
  the four kinds were inferred from a summary sentence on `15-system-refinements.png`
  instead of the real sheet. Confirmed the finding fully, and it was worse than described:
  the sheet has four rows — **Textbook & lecture notes** (840 MB), **Problem banks**
  (120 MB), **Visual assets & module data** (410 MB, one combined toggle, not two), **Course
  videos** (2.1 GB, partial: "9 of 32 downloadable — the rest are streamed by your school").
  My union had split visual-assets/module-data into two kinds that don't exist as separate
  toggles and omitted course videos entirely. Also noted a fifth row, "Voice tutoring" —
  not a toggle, an "Internet required" badge (a `Module`-level property, not a workspace
  offline-content kind), correctly left out.
  Fixed `src/types/workspace.ts`: `OfflineContentKind` is now
  `'textbookAndLectureNotes' | 'problemBanks' | 'visualAssetsAndModuleData' |
  'courseVideos'`; added an optional `partial: { availableCount, totalCount, limitReason }`
  to `OfflineKindAvailability` for the course-videos case (`limitReason` stated in the
  learner's terms, never as an error, per the screenshot's own design note #3). Confirmed
  the sheet's "Download · 1.4 GB" total is the sum of only the *enabled* kinds' sizes
  (840+120+410, videos off) — matches the derive-don't-store approach already in place.
  All gates re-run — pass. Moved back to `review`.

## What was built / tested / left out

- **Built**: `workspaceService.ts`, `goalService.ts`, `conceptService.ts`, `moduleService.ts`,
  `sessionService.ts` — 24 functions total, every one `async`/`Promise`-returning per
  `ARCHITECTURE.md` §5 rule 2, bodies `throw new Error('not implemented')`. Also fixed
  `eslint.config.js` (see worklog) — a shared-tooling change, called out explicitly since it
  affects every other agent's lint gate, not just this task. Also revised `src/types/
  workspace.ts` (originally 005) to model per-kind offline availability, matching the four
  rows in `screenshots/21-offline-modules-goals.png` exactly (Textbook & lecture notes /
  Problem banks / Visual assets & module data / Course videos, the last with partial-
  availability detail) — see worklog for the two-pass correction; flagging again here since
  a type change is the highest-blast-radius kind of edit (`.ai/quality-gates.md`).
- **Tested**: `npm run typecheck` (0 errors), `npm run lint` (0 errors, after the config fix),
  `npm run build` (succeeded), grep for stray hex/`rgba(` in `src/services` (0 hits). No
  component/hook tests apply — this task touches neither.
- **Left out / notable decisions**:
  - No `commandService` — Command Palette's mixed result groups (Actions, Concepts, notes,
    marketplace) compose `conceptService.searchConcepts` + others inside a hook
    (`useCommandPalette`, already named in `ARCHITECTURE.md` §2), not a dedicated service.
  - Dropped a `getGuidingGoal(workspaceId)` convenience method — redundant with
    `Workspace.guidingGoalId` + `goalService.getGoal(id)`; kept `getGoalsByWorkspace` instead
    since that's the one shape not already reachable off `Workspace` itself.
  - `moduleService`'s `Module` type conflates marketplace-catalog fields (developer, price,
    trust) with per-workspace install state (`enabled`, `visibility`). Works for the
    mock-data phase; if Stage 4/6 finds this awkward for an unowned catalog entry, that's a
    finding routed back here, not something to pre-split speculatively now.
  - Input DTOs (`CreateWorkspaceInput`, `StartSessionInput`) live in their service file, not
    `src/types/` — they're action parameters, not domain nouns, consistent with
    `ARCHITECTURE.md` §4's "nothing else" scope for `types/`.

## Review

Reviewer: codex
Date: 2026-08-27
- [ ] Correctness — FAIL: `setWorkspaceOfflineAvailability(id, enabled)` cannot represent
  screen 21 / task 031's four per-kind offline toggles, and `moduleService` has no operation
  for changing the `workspace | contextual | off` visibility that the locked `Module` type
  and task 031 require. These omissions would force later stages to change this supposedly
  locked service contract. The handoff also reports 21 functions, but 23 are exported.
- [x] Architecture conformance — pass: service signatures import through the types barrel,
  return Promises, and contain only explicit not-implemented stubs.
- [x] UI rules — pass: no UI markup, copy, or hardcoded design values were introduced.
- [x] Process — pass: independent runs of `npm run typecheck`, `npm run lint`,
  `npm run build`, and the hardcoded-value grep all passed; the ESLint configuration change
  is explained in the worklog.

Verdict: changes-requested

### Re-review — 2026-08-27

- [ ] Correctness — FAIL: the module-visibility operation and corrected function count
  resolve two prior findings, but `OfflineContentKind` does not match the authoritative
  `21-offline-modules-goals.png`. The four toggles are Textbook & lecture notes, Problem
  banks, Visual assets & module data, and Course videos; the proposed union instead splits
  visual assets from module data and omits course videos. `OfflineKindAvailability` also
  cannot carry the per-kind explanatory/availability detail needed for the designed partial
  course-video case (for example, only some videos being downloadable).
- [x] Architecture conformance — pass: revised signatures remain Promise-returning stubs,
  use public domain types, and introduce no data-flow violation.
- [x] UI rules — pass: no UI markup, learner-facing copy, or hardcoded design values were
  introduced.
- [x] Process — pass: independent reruns of `npm run typecheck`, `npm run lint`,
  `npm run build`, and the hardcoded-value grep passed; 24 service functions are now
  exported as documented.

Verdict: changes-requested

### Final re-review — 2026-08-27

- [x] Correctness — pass: all prior findings are resolved. The offline contract now matches
  the authoritative four toggles, represents partial course-video availability, exposes the
  module visibility mutator, and documents the correct total of 24 service functions.
- [x] Architecture conformance — pass: domain additions remain in `src/types/workspace.ts`
  and are barrel-exported; all service operations are Promise-returning explicit stubs.
- [x] UI rules — pass: the contract preserves the screenshot's learner-facing offline
  model without introducing markup or hardcoded design values.
- [x] Process — pass: independent runs of `npm run typecheck`, `npm run lint`,
  `npm run build`, the hardcoded-value grep, and `git diff --check` all passed.

Verdict: pass

## Follow-ups

Anything noticed during implementation or review that's out of this task's scope.
