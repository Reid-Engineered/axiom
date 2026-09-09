---
id: 066
title: Study session problem pane — types, useAttempt hook, and ProblemPane wiring
status: done
owner: antigravity
stage: 8
depends_on: [063]
---

## Scope

Tasks 3, 4 and 5 of `docs/superpowers/plans/2026-09-09-study-session-ui-integration.md`: the
frontend half of the Study Session integration. Adds the `AttemptDescription` type and
`Session.currentAttemptId`, the `describeAttempt` / `nextProblem` service functions, mock-IPC
support for both, the `useAttempt` hook, and the `ProblemPane` rewire that replaces the
screen's hardcoded problem with the session's real bound attempt across four states: bound,
unbound, wrong answer, solved.

Does **not** build: any Rust — the two commands are task `065`, owned by codex and running in
parallel. This task's tests run against `src/test/mockBackend.ts`, so it is not blocked on
`065` landing. Does not touch the visualization pane, the tutor pane, working-area
persistence, or `ROADMAP.md` (task `064`'s Task 6).

## Plan

Follow the plan document's Task 3, Task 4 and Task 5 exactly; it carries the full test code
and implementation for each. Files:

- `src/types/practice.ts`, `src/types/session.ts`, `src/types/index.ts`
- `src/services/practiceService.ts`, `src/services/sessionService.ts`
- `src/test/mockBackend.ts` — bind an attempt on `startSession` for a mapped concept; handle
  `describeAttempt` and `nextProblem`
- `src/hooks/useAttempt.ts` + `src/hooks/useAttempt.test.tsx`
- `src/pages/StudySessionPage.tsx`, `.module.css`, `.test.tsx`

Design source: `docs/superpowers/specs/2026-09-09-study-session-ui-integration-design.md`
§§4–7. Two known risks the plan calls out: Task 5's CSS token names are written from
neighbouring conventions and need checking against `src/styles/tokens.css` (substitute the
nearest real token, never a literal), and Task 3's `startSession` change may require updating
existing session tests that assert exact session objects.

## Worklog

- 2026-09-09 — Filed by claude from the approved plan, split from task `065` so the Rust and
  frontend halves have unambiguous single owners per `.ai/lifecycle.md`. Assigned to
  antigravity, which owns `StudySessionPage` from task 026 and every other page-layer task in
  the archive.
- 2026-09-09 — Task 3 implemented: added `AttemptDescription` to `src/types/practice.ts`,
  `currentAttemptId?: string` to `Session` in `src/types/session.ts` (re-exported in `src/types/index.ts`),
  added `describeAttempt` to `practiceService.ts` and `nextProblem` to `sessionService.ts`, updated
  `src/test/mockBackend.ts` with mapped concept attempt creation on `startSession` and handlers
  for `describeAttempt` and `nextProblem`. Added unit test in `practiceService.test.ts`.
- 2026-09-09 — Task 4 implemented: added `src/hooks/useAttempt.ts` with full attempt lifecycle
  (load/describe, check with typed response, hint revelation on wrong answer, advance with next)
  and 7 unit tests in `src/hooks/useAttempt.test.tsx`.
- 2026-09-09 — Task 5 implemented: rewired `StudySessionPage.tsx` and `ProblemPane` to `useAttempt(session, setData)`,
  removed hardcoded `shellExpression`, rendered four states (bound open, unbound, wrong answer with hint, solved with next problem),
  added token-compliant styles in `StudySessionPage.module.css`, and verified with 4 new tests in `StudySessionPage.test.tsx`.
- 2026-09-09 — Quality gates verified: `npm test` (60 test files, 161 tests passed), `npm run typecheck` (0 errors),
  `npm run lint` (0 errors), `npm run build` (succeeded), and token regex check (0 hardcoded values). Marked for review.
- 2026-09-09 — Review findings addressed: rebased cleanly on `origin/master`; wrapped check, hint, and next actions in `StudySessionPage.tsx` with `setMutationError` to surface failures to the user; switched `hintList` keys to array indices; added test asserting error alert on check failure; documented `vite.config.ts` test exclusion for `.claude/**` to isolate external Claude worktrees.

- 2026-09-09 — Reviewed by claude: `changes-requested` on the first pass, `approve` on the
  second after a clean rebase and fixes for all six findings. All seven required checks green.
  `review` → `done` and archived as part of the merge of PR #9.

## What was built / tested / left out

- **Built:**
  - `AttemptDescription` type interface and `Session.currentAttemptId?: string`
  - `describeAttempt` and `nextProblem` service functions
  - Mock IPC support for `describeAttempt` and `nextProblem`, plus attempt binding on `startSession` for mapped concepts
  - `useAttempt` hook managing attempt state, answer, hints, evaluation, checking, and problem advancement
  - `ProblemPane` rewiring supporting bound attempt prompt, numeric/symbolic answer input, feedback messages, hint list, solved state, and unbound empty state
  - Error state handling in `StudySessionPage` wrapping check, hint, and next actions with `mutationError` alerts
  - CSS module styling using tokens from `tokens.css`
  - Excluded `.claude/**` in `vite.config.ts` `test.exclude` to prevent test runner from picking up files in external Claude worktrees

- **Tested:**
  - `src/services/practiceService.test.ts` (describing generated attempts)
  - `src/hooks/useAttempt.test.tsx` (describing on mount, unbound case, hint reveals on wrong answer, exhausted hints boundary, solved status, next problem advancement, error handling)
  - `src/pages/StudySessionPage.test.tsx` (rendering bound attempt prompt, unbound state message, hint revelation on wrong answer, solved confirmation and next problem action, and error alert on check failure)
  - Full frontend suite: 60 test files / 162 tests passing

- **Left out:**
  - Rust Tauri commands (handled in parallel by task 065)
  - Visualization and tutor pane backend integration (mock by design for Stage 8)
  - Working area scratch persistence (deferred per spec §9)

## Review

Reviewer: claude
Date: 2026-09-09

Reviewed twice. The first pass returned `changes-requested` over two blocking process
findings and one real defect; this section records both passes, since the first one's findings
are part of this task's history rather than something to erase.

Verified independently on each pass rather than from the PR body: the branch was checked out
into a separate worktree and `npm test` run locally (161 tests on the first pass, 162 on the
second, matching the claims exactly), the CSS token names were checked one by one against
`src/styles/tokens.css`, and the check status was read from the API rather than taken on
report.

- [x] **Correctness — pass.** `useAttempt` is faithful to the plan; the four states render as
      specified; `check` reads `hintsRevealed` from the pre-evaluate snapshot, which is what
      makes the exhausted-hints boundary work. Three deliberate improvements on the plan, all
      correct:
      - Real token names substituted for the invented ones in the plan's Task 5 sketch
        (`--color-hairline-strong`, `--radius-control`, `--space-md`, `--space-xl`,
        `--text-secondary`, `--color-content`); all nine verified present in `tokens.css`.
      - The unbound state renders `WorkingArea`, which the plan's sketch omitted. Spec §6 says
        the working area and tutor stay usable in that state, so the deviation is *more*
        faithful to the design than the plan was.
      - `ProblemPane` takes `onCheck` / `onHint` / `onNext` callbacks rather than the hook
        object, so the component no longer holds a hook at all — a cleaner boundary than the
        plan specified.
- [x] **Architecture conformance — pass.** `useAttempt` is called only from
      `StudySessionPage` (§5 rule 1). `AttemptDescription` lives in `src/types/practice.ts`
      and reaches consumers through `index.ts`'s existing `export * from './practice'` (§4).
      No new global state (§5 rule 3). No `ARCHITECTURE.md` update needed: no structural
      change, and the hook follows the existing `useSession` shape.
- [x] **UI rules — pass.** No hardcoded color, radius, shadow, or spacing. Copy follows
      `AXIOM-HANDOFF.md` line 92: "Correct." with no celebration, "That does not match yet."
      rather than a bare "incorrect", no exclamation marks, no emoji.
- [x] **Process — pass on re-review** (failed the first pass; see findings 1 and 2).

### First-pass findings, all since addressed

1. **CI never ran.** `statusCheckRollup` was empty — zero checks, not pending ones —
   making the locally-run gate claims unverifiable by the process meant to verify them.
2. **The branch conflicted with `master`, which is what suppressed CI.** Cut from `010e438`,
   before the spec, plan, and task records landed at `9f5093a`, so it re-added all three as
   new files; GitHub cannot compute a merge ref for a conflicted PR, so the `pull_request`
   workflow never fired. Both resolved by one clean rebase; all seven checks now pass.
3. **Check, hint, and next failures were silently swallowed — the plan's defect, not the
   implementer's.** `void attempt.check()` discarded the rejection, so an IPC failure gave an
   unhandled promise rejection and nothing visible to the learner, while spec §7 routes those
   to the page's existing `mutationError` channel. The plan's Task 5 code block specified
   exactly what was implemented and named no test for the path. Now fixed with three handlers
   structurally identical to the existing `pause`, plus a test asserting the message reaches
   `role="alert"`.
4. **`vite.config.ts` was out of scope and undisclosed.** Adding `'.claude/**'` to the vitest
   `exclude` list is correct and probably necessary — worktrees under `.claude/` carry test
   files vitest would otherwise collect — but needed naming. Now disclosed in both the worklog
   and the "what was built" section.
5. **Nit — `key={text}` on the hint list**, which would collide on duplicate hint text. Now
   keyed by index.
6. **Nit — stray trailing blank lines** in `practiceService.ts` and `sessionService.ts`. Now
   trimmed.

### Note for whoever picks up task 065

This merges the frontend half against `src/test/mockBackend.ts` only. The two Tauri commands
it calls — `describeAttempt` and `nextProblem` — do not exist in Rust yet; they are task 065,
still unstarted. Until 065 lands, this code is fully green in tests and inert in a real build:
`describeAttempt` will reject at runtime, which the error handling added under finding 3
surfaces as a message rather than a crash. That is the parallel-execution split working as
designed, not a regression, but the beta gate's end-to-end criterion is not met until 065
merges.

Verdict: approve

## Follow-ups

- **065** — the Rust half (`describeAttempt`, `nextProblem`). Unstarted; this task is inert
  in a real build until it lands.
