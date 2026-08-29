---
id: 031
title: WorkspaceToolsPage + offline/modules-at-scale sheet
status: in-progress
owner: antigravity
stage: 6
depends_on: [003, 021]
---

## Scope

Full implementation: module rows, `Toggle`, `SuggestionPanel`. Includes the offline/modules-at-scale sheet behavior from `AXIOM-HANDOFF.md` §5.

## Plan

- src/pages/WorkspaceToolsPage.tsx
- associated .module.css
- complete the locked `Sheet` overlay body used by the offline flow

## Worklog

- 2026-08-28 — Claimed by Codex. Starting 20-module visibility grouping, per-workspace enable toggles, four-goal context, and the per-kind offline sheet. Global `Module.visibility` will be consumed as locked, without a per-workspace workaround.
- 2026-08-28 — Codex implementation pass complete. Built module rows with real per-workspace enabled toggles, grouped modules strictly by the locked global `visibility` field, wired marketplace/module-detail navigation and the suggestion, completed the locked `Sheet` body, and built the four-kind offline flow with honest totals and the fixture's partial-video limitation. Demonstrated against the actual Calculus II fixtures: 20 modules grouped 4 In the workspace / 9 Appear when relevant / 7 Off, four goals, and four offline-kind switches. No per-workspace visibility field or workaround was introduced; the existing global behavior does not make this single-workspace screen internally contradictory. Full gates passed: typecheck, lint, production build, 95/95 tests, hardcoded color scan, component-to-service import scan, and `git diff --check`. Handed to Antigravity for fidelity against `08-workspace-tools.png` and `21-offline-modules-goals.png`; status remains `in-progress`.
- 2026-08-29 — Codex addressed the review findings. Formatted the page, CSS Module, and
  tests with Prettier; all three now pass `npx prettier --check`. Reworked Voice tutoring as
  a fifth offline-sheet row with its own heading, degradation sentence, and the shared
  dashed `Internet required` badge, with render coverage. Typecheck, lint, build, the full
  43-file/101-test suite, token scan, and diff check pass. Status returns to `in-progress` for
  the remaining Antigravity screenshot-fidelity pass.

## What was built / tested / left out

Codex grouping, hook wiring, offline sheet, scale tests, and Sheet primitive are complete. Antigravity screenshot polish remains before review.

## Review

Reviewer: claude-code
Date: 2026-08-29

Reviewing what's actually sitting in `master` today, not waiting for a formal `review` state
— per the earlier Follow-up note, this task's code landed there ahead of the pipeline. Status
was `in-progress`; moved to `changes-requested` since that's the accurate signal (reviewed,
blocking issues found), even though it skipped a `review` state formally.

- [x] Correctness — pass on data/logic. The three-way grouping (`In the workspace` /
      `Appear when relevant` / `Off in this workspace`, counted 4/9/7) and the offline sheet's
      four per-kind toggles with the `9 of 32 downloadable` partial case both match
      `AXIOM-HANDOFF.md:198` and `screenshots/21-offline-modules-goals.png` exactly, and are
      asserted directly by `WorkspaceToolsPage.test.tsx`. `screenshots/08-workspace-tools.png`
      shows a simpler flat ON/OFF layout, but that's the pre-scale example (6 modules); §5's
      pressure-tested grouping is the correct one to build against a 20-module fixture, and
      the task's own scope names it explicitly.
- [ ] Correctness — FAIL: `WorkspaceToolsPage.tsx` and `WorkspaceToolsPage.module.css` (and
      the test file) fail `npx prettier --check`, unlike every other file touched this session
      — the whole component body (`WorkspaceToolsPage.tsx:31`), `ModuleGroup` (`:35`), and
      `OfflineSheet` (`:42`) are each a single several-hundred-character line. No other page in
      this codebase looks like this; `npx prettier --check` on `GoalEditingSheet.tsx` and
      `StudySessionPage.tsx` both pass clean for comparison. `npm run lint` doesn't catch this
      (ESLint isn't configured with a Prettier rule here), which is presumably how it slipped
      through the stated gates. Real defect: unreadable and unreviewable in this state.
- [x] Architecture conformance — pass. Domain data via `useModules`/`useGoals`/`useWorkspaces`
      hooks, called only from the page; no new types; no new global state.
- [ ] UI rules — incomplete, not a fail against Codex's pass specifically: the worklog is
      honest that Antigravity's fidelity pass against these two screenshots hasn't happened
      yet, and it shows. One concrete gap for whoever does that pass: the offline sheet's
      "Voice tutoring" degradation notice (`OfflineSheet`, `WorkspaceToolsPage.tsx:42`,
      `.degradation` in the CSS) is one small paragraph tacked below the four toggle rows,
      but `21-offline-modules-goals.png` gives it the same row treatment as the four toggles
      — its own "Voice tutoring" heading plus a dashed "Internet required" badge — and the
      task's own Plan names "complete the locked Sheet overlay body used by the offline flow"
      directly. No hardcoded px/hex/rgba in either touched `.module.css` file, for what it's
      worth.
- [x] Process — pass on the automated gates. Independently reran `npm run typecheck`,
      `npm run lint`, `npm test -- --run` (43 files / 101 tests), and `npm run build`; all
      clean.

Verdict: changes-requested. Blocking: the Prettier-formatting defect. Non-blocking but
real: the offline sheet still needs its Antigravity fidelity pass (the Voice tutoring row is
the concrete gap to start from); the worklog already says this honestly, so this isn't new
information, just confirmation it's still true today.

## Re-review

Reviewer: claude-code
Date: 2026-08-29

- [x] Correctness — pass. `npx prettier --check` on all three files now passes clean.
- [x] UI rules — pass on the concrete gap raised. "Voice tutoring" is now its own row
      (`WorkspaceToolsPage.tsx:193-199`) with a heading, the degradation sentence, and
      `<OfflineChip status="required" />` (`src/components/badges/OfflineChip.tsx`) — reusing
      the existing shared badge rather than a bespoke one, correctly matching
      `21-offline-modules-goals.png`'s dashed "Internet required" treatment.
      `WorkspaceToolsPage.test.tsx:35-36` asserts the new heading and badge text directly.
- [x] Process — pass. Independently reran `npm run typecheck`, `npm run lint`,
      `npm test -- --run` (43 files / 101 tests), `npm run build`, and `git diff --check`; all
      clean. Grepped `WorkspaceToolsPage.module.css` for hardcoded px/hex/rgba: none found.

Verdict: approved. Both findings resolved. Status correctly stays `in-progress` — the
remaining Antigravity screenshot-fidelity pass (full polish against `08-workspace-tools.png`
and `21-offline-modules-goals.png`, beyond the one row fixed here) is still outstanding, per
the worklog.

## Follow-ups

Anything noticed during implementation or review that's out of this task's scope.

- 2026-08-29 (claude-code) — Heads up, not a finding against the work itself: while finishing
  branch `agent/codex/034-goal-editing-sheet` (fast-forward merge to `master` at `6077a2a`),
  this task's implementation commit (`6ed2412`, "feat: implement workspace tools at scale")
  came along underneath it, since 034 was built on top of it on the same branch. It is now in
  `master` even though this file still reads `status: in-progress`. Marcus's call: leave it
  landed rather than un-merge/re-split. This task still needs its normal review pass — just
  run it against current `master` instead of a feature branch — before flipping status to
  `review`/`done`.
