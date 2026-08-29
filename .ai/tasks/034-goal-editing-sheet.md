---
id: 034
title: GoalEditingSheet implementation
status: review
owner: antigravity
stage: 6
depends_on: [011, 021]
---

## Scope

Full implementation as a `Sheet` overlay: `Chip`, consequence-preview panel.

## Plan

- src/pages/GoalEditingSheet.tsx
- associated .module.css

## Worklog

- 2026-08-28 — Claimed by Codex on top of 031's tested Sheet primitive. Starting real-goal editing/revert wiring, inferred-facet correction, and consequence preview; task remains in progress for Antigravity fidelity.
- 2026-08-28 — **Interrupted by a Codex rate limit, mid-debug, not handed to Antigravity yet.
  Pick up here:**
  - Uncommitted in the working tree (not lost, just not yet committed): `src/pages/GoalEditingSheet.tsx` (modified), `src/pages/GoalEditingSheet.module.css` (new),
    `src/pages/GoalEditingSheet.test.tsx` (new), plus this file's frontmatter/worklog edit.
  - The sheet itself reads as functionally complete: real `useGoals` edit/revert wiring,
    inferred-facet Chips with per-facet removal, the consequence-preview panel copy, error
    handling on save/revert. `typecheck` and `lint` both pass clean on the current state.
  - **One known failing test, not yet fixed**: `GoalEditingSheet.test.tsx`'s first case
    (`edits and reverts the real guiding-goal fixture...`) fails on
    `expect(field).toHaveValue('Be ready to explain...')`. Root cause: `useGoals(workspaceId)`
    loads asynchronously, so `goal` is `undefined` on the first render and the textarea starts
    empty; `useEffect(() => { if (goal) setText(goal.text); }, [goal])` only populates it once
    the fetch resolves on a later render. The test's `await screen.findByRole('textbox', {
    name: 'Goal' })` resolves as soon as the _element_ exists (immediately — the textarea
    isn't gated on loading), not once its value is populated, so the `toHaveValue` assertion
    runs before the effect has fired. Fix: replace the bare
    `expect(field).toHaveValue(...)` with `await waitFor(() => expect(field).toHaveValue(...))`
    (already imported in the test file). Haven't verified this is the _only_ remaining issue —
    re-run the full file after the fix, not just this one case.
  - Second test in the same file (`allows inferred facets to be corrected...`) passes as-is.
  - Not yet done: full gate suite (typecheck/lint/build/full test run) hasn't been re-run
    since before the interruption; worklog's "what was built/tested/left out" section is
    still template text; no hand-off to Antigravity yet — don't do that until the test suite
    is green and gates are re-run and recorded here.
- 2026-08-29 — Codex resumed the interrupted implementation, fixed the asynchronous fixture
  assertion, added direct coverage of the Revert behavior, and reran every applicable quality
  gate. Functional implementation is complete and ownership passes to Antigravity for visual
  fidelity against `11-goal-editing.png`; status remains `in-progress` through the polish step.
- 2026-08-29 — Antigravity completed visual-fidelity polish pass against `reference/UI/screenshots/11-goal-editing.png`:
  - Aligned typography, spacing, and borders with design tokens (`tokens.css`).
  - Textarea styled with accent active outline, comfortable 2-line height, and smooth focus state.
  - Aligned previous goal note and Revert button row with flex space-between.
  - Styled inferred facet chips and dashed `+ Add` button according to screenshot spec.
  - Styled `WHAT CHANGES` consequence preview section with uppercase eyebrow heading, top hairline divider, and custom accent/muted bullet dots (`--accent-mastery` for recommendations/pacing, `--color-hairline-strong` for tools/invariants).
  - Aligned footer action bar with `Update goal` / `Cancel` buttons on the left and `Goal history` metadata label on the right.
  - All quality gates passed cleanly (`typecheck`, `lint`, `build`, `test`, `git diff --check`). Status moved to `review`.
- 2026-08-29 — Antigravity addressed the two review findings:
  - Correctness: Rendered `GoalInferredStructure.tools` in `GoalEditingSheet.tsx` as an inferred facet chip with remove support, matching `CreateWorkspacePage`.
  - UI rules: Replaced hardcoded `top: 6px` in `GoalEditingSheet.module.css` with `top: var(--space-md-minus)`.
  - Added test assertion and removal check for `Tools · Practice, Visualizer, Tutor` chip in `GoalEditingSheet.test.tsx`.
  - Reran all quality gates (`typecheck`, `lint`, `build`, `test`, `git diff --check`); all passed cleanly. Status moved to `review`.

## What was built / tested / left out

- Built real `useGoals` and `useWorkspaces` wiring for the active goal and workspace name,
  update/revert behavior, removable inferred-facet chips, validation and mutation errors, and
  the non-destructive consequence preview inside the shared `Sheet` overlay.
- Polished visual layer in `GoalEditingSheet.module.css` and `GoalEditingSheet.tsx` to match
  `11-goal-editing.png` screenshot: seamless sheet header/footer styling, custom colored consequence bullets, dashed Add chip, active textarea styling, and token-driven spacing and typography.
- Tested `src/pages/GoalEditingSheet.test.tsx` and the entire test suite (41 files, 97 tests passed).
- Quality gates passed on 2026-08-29: `npm run typecheck`, `npm run lint`, `npm run build`,
  `npm test`, and `git diff --check`.
- Left out: no functional changes to component props, hook return contracts, or business logic.

## Review

Reviewer: claude-code
Date: 2026-08-29

- [ ] Correctness — FAIL: `GoalInferredStructure.tools` (`src/types/goal.ts:12`) is never
      rendered as a chip in `GoalEditingSheet.tsx`'s `facets` array (lines 64-71), which only
      covers `deadline`/`masteryType`/`pacing`/`conceptScope`. The `goal-calculus-exam` fixture
      has `tools: ['Practice', 'Visualizer', 'Tutor']` set
      (`src/services/mockData/goals.ts:14`), so real inferred data is silently dropped from the
      editor. `goal.ts:4-6`'s own doc comment says inferred structure is "shown as removable
      chips on Create Workspace / Goal Editing" and "always inferred and correctable" —
      `CreateWorkspacePage.tsx:19` does surface a Tools chip on the sibling screen, so this
      screen is inconsistent with its counterpart, not just incomplete on its own.
- [x] Architecture conformance — pass. Domain data fetched only via `useGoals`/`useWorkspaces`,
      called only from this page; no new types; no new global state.
- [ ] UI rules — FAIL: `GoalEditingSheet.module.css:156` hardcodes `top: 6px` instead of the
      existing `var(--space-md-minus)` token (`src/styles/tokens.css:55`, also `6px`). AGENTS.md
      is explicit that a hardcoded px value is a review-blocking finding, not a style nit.
- [x] Process — pass. Independently reran `npm run typecheck`, `npm run lint`, `npm test --
      --run` (41 files / 97 tests), and `npm run build`; all clean. Worklog is detailed and
      matches the diff. Scope matches the task as created; no `ARCHITECTURE.md` changes needed.

Verdict: changes-requested

## Follow-ups (from review)

- The "+ Add" chip (`GoalEditingSheet.tsx:127`) has no `onClick`/handler — clicking it does
  nothing. The reference screenshot is static so this can't be confirmed as in-scope for 034;
  noting it here rather than blocking on it. If an "add a facet" flow is wanted, it should be
  its own task rather than folded into this fix.

## Re-review

Reviewer: claude-code
Date: 2026-08-29

- [x] Correctness — pass. `facets` (`GoalEditingSheet.tsx:64-75`) now includes a `tools` entry
      rendering `Tools · Practice, Visualizer, Tutor` for the `goal-calculus-exam` fixture,
      matching `CreateWorkspacePage`'s treatment of the same field, with a `none` fallback when
      empty. `GoalEditingSheet.test.tsx:42-46` asserts it renders and can be removed.
- [x] UI rules — pass. `GoalEditingSheet.module.css:156` now reads
      `top: var(--space-md-minus)`; `grep` for hardcoded px/hex/rgba in the file returns nothing.
- [x] Process — pass. Independently reran `npm run typecheck`, `npm run lint`,
      `npm test -- --run` (41 files / 97 tests), `npm run build`, and `git diff --check`; all
      clean.

Verdict: approved — both prior findings fixed and independently verified. Leaving `status:
review` rather than `done` since per `.ai/lifecycle.md` that transition happens on merge to
`main`, which hasn't happened yet. The "+ Add" follow-up above remains open as non-blocking,
for a future task if wanted.

## Follow-ups

Anything noticed during implementation or review that's out of this task's scope.
