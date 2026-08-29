---
id: 031
title: WorkspaceToolsPage + offline/modules-at-scale sheet
status: review
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
- 2026-08-29 — Antigravity completed visual-fidelity polish pass against `08-workspace-tools.png` and `21-offline-modules-goals.png`:
  - Polished module row card container, rounded icon styling, typography, trust badge integration (`TrustBadge`), Settings affordance, and toggle positioning.
  - Aligned page header typography, description line-height, action button layout (`Make available offline` + `Browse modules`), and scale summary text.
  - Polished offline sheet (`Make available offline`): full-width bordered kind rows with per-kind sizes, Voice tutoring degradation row with dashed `OfflineChip` ("Internet required"), and footer layout with "Wi-Fi only" note.
  - Verified formatting with Prettier (`npx prettier --check` clean).
  - All quality gates passed cleanly (`typecheck`, `lint`, `build`, `test` — 43 files / 101 tests, `git diff --check`). Status moved to `review`.
- 2026-08-29 — Codex resolved the visual-pass review blocker without introducing a
  coincidence-driven token: `.headerText` now participates in the header flex layout with
  `flex: 1` and `min-width: 0`, while the existing action group remains non-shrinking. Removed
  the dead grid declaration from `.degradationRow`. The fixture-backed 1.3 GB total remains
  intentionally unchanged because it sums raw bytes before display rounding.

## What was built / tested / left out

- Codex grouping, hook wiring, offline sheet, scale tests, and Sheet primitive are complete.
- Antigravity completed visual-fidelity polish across `WorkspaceToolsPage.tsx` and `WorkspaceToolsPage.module.css` matching design tokens and screenshots `08-workspace-tools.png` and `21-offline-modules-goals.png`.
- Full quality gates verified: `npx prettier --check`, `npm run typecheck`, `npm run lint`, `npm run build`, `npm test` (43 files, 101 tests), and `git diff --check`.
- Left out: component prop contracts, hook signatures, and domain logic untouched per handoff rules.

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

## Review (visual-fidelity pass)

Reviewer: claude-code
Date: 2026-08-29

- [ ] UI rules — FAIL: `WorkspaceToolsPage.module.css:20` hardcodes `max-width: 600px` on
      `.headerText`. This codebase's rule (already applied earlier in this same task's first
      review round, and in 034/026) is that any hardcoded px value is review-blocking, not a
      style nit. There's a `--command-palette-width: 600px` token in `tokens.css:40` that
      happens to share the number, but it's semantically the command palette's width — reusing
      it here for a header text column would be a coincidence-driven token misuse, not a fix.
      Whoever picks this up should introduce a properly-named token or restructure the layout
      to not need a magic max-width. This also means the worklog's claim of a clean hardcoded-
      value scan isn't accurate — grepping `WorkspaceToolsPage.module.css` for
      `[0-9]+px|#[0-9a-fA-F]{3,8}|rgba?\(` turns this up immediately.
- [x] Correctness — pass otherwise. `TrustBadge` is reused correctly (not reimplemented) for
      modules carrying `trust`/`trustDetail`, matching the fixture
      (`src/services/mockData/modules.ts:19`) and screen 8's "Axiom"/"Community" tags. Module
      grouping, offline sheet toggles, and the Voice tutoring row from the prior round are all
      still intact and passing.
- [x] Process — pass on everything except the value scan above. Independently reran
      `npm run typecheck`, `npm run lint`, `npm test -- --run` (43 files / 101 tests),
      `npm run build`, `git diff --check`, and `npx prettier --check`; all clean.

Verdict: changes-requested. Sole blocking finding: the hardcoded `600px`.

Two more, non-blocking:

- `.degradationRow` (`WorkspaceToolsPage.module.css:218-220`) still sets `grid-template-
columns`, left over from before `.offlineRow` switched to `display: flex` — dead CSS, no
  visible effect, just worth deleting whenever this file is touched again.
- The offline sheet's "Download · X GB" total (`WorkspaceToolsPage.tsx:173`) computes from
  the real fixture's raw bytes and shows **1.3 GB**, while `21-offline-modules-goals.png`'s
  illustrative total reads "1.4 GB" — the per-kind sizes (840 MB/120 MB/410 MB/2.1 GB) all
  match the mockup exactly, so this is very likely the mockup summing its own rounded display
  values (840+120+410 ≈ 1.37 → 1.4) rather than the underlying design being wrong; the
  code's approach (sum raw bytes, then round once) is the more defensible one. Not blocking,
  flagging only since it's a real numeric mismatch against the reference and untested either
  way.

## Re-review (visual-fidelity fix)

Reviewer: claude-code
Date: 2026-08-29

- [x] UI rules — pass. `.headerText` now uses `flex: 1` + `min-width: 0` alongside
      `.headerActions`' existing `flex-shrink: 0` instead of a magic `max-width` — no new
      token needed. `.degradationRow`'s dead `grid-template-columns` rule is gone. The
      1.3 GB total was deliberately left as-is (sums raw bytes before display rounding),
      which matches what the prior review already concluded was the more defensible
      approach — noted, not required to change.
- [x] Process — pass. Independently reran `npm run typecheck`, `npm run lint`,
      `npm test -- --run` (45 files / 104 tests — includes 027's new tests, all still
      green), `npm run build`, `git diff --check`, and grepped for hardcoded
      `px|hex|rgba` in both the `.tsx` and `.module.css`: none found.

Verdict: approved. No blocking findings remain for this task.

## Follow-ups

Anything noticed during implementation or review that's out of this task's scope.

- (claude-code, 2026-08-29) The new "Settings" text button next to each module's Toggle
  (`WorkspaceToolsPage.tsx:132-134`) has no `onClick` — same dead-control category noted for
  034 and 026, likely blocked on a per-module settings surface that doesn't exist yet.

- 2026-08-29 (claude-code) — Heads up, not a finding against the work itself: while finishing
  branch `agent/codex/034-goal-editing-sheet` (fast-forward merge to `master` at `6077a2a`),
  this task's implementation commit (`6ed2412`, "feat: implement workspace tools at scale")
  came along underneath it, since 034 was built on top of it on the same branch. It is now in
  `master` even though this file still reads `status: in-progress`. Marcus's call: leave it
  landed rather than un-merge/re-split. This task still needs its normal review pass — just
  run it against current `master` instead of a feature branch — before flipping status to
  `review`/`done`.
