---
id: 062
title: GoalEditingSheet.test.tsx flake — assert on facet chips before they render
status: done
owner: claude
stage: 8
depends_on: []
---

## Scope

Fix the `GoalEditingSheet.test.tsx` flake that `.ai/quality-gates.md` already names by file
as its flaky-check precedent from task 052, so `frontend-checks` stops failing intermittently
on loaded CI runners. This is the follow-up task that the flaky-checks policy in
`.ai/quality-gates.md` calls for and that was never actually filed.

Does **not** build: any change to `GoalEditingSheet` itself, or a general audit of
`getByText`-after-`findBy*` across the suite. If that audit looks warranted once this one is
fixed, it becomes its own task.

## Evidence

`src/pages/GoalEditingSheet.test.tsx:41-42`, in
`allows inferred facets to be corrected without changing the goal text`:

```ts
await screen.findByRole('textbox', { name: 'Goal' });
expect(screen.getByText('Tools · Practice, Visualizer, Tutor')).toBeVisible();
```

The `await` settles as soon as the goal textbox exists, which does not guarantee the inferred
facet chips have rendered. On a slower runner the synchronous `getByText` on the next line
runs a tick too early.

CI (run `33991157438`, job `101373518620`, PR #4):

```
FAIL src/pages/GoalEditingSheet.test.tsx > GoalEditingSheet >
  allows inferred facets to be corrected without changing the goal text
TestingLibraryElementError: Unable to find an element with the text:
  Tools · Practice, Visualizer, Tutor.
Test Files  1 failed | 58 passed (59)
      Tests  1 failed | 148 passed (149)
```

Environment-dependent, which is what makes it a flake rather than a break: it failed on
`ubuntu-latest` while `frontend-checks` passed on `macos-latest` and `windows-latest` in the
same run, and the full suite passes locally (59 files, 149 tests). Task 052 saw the same test
fail once and go green on an immediate re-run, which is the precedent `.ai/quality-gates.md`
records.

## Plan

Files to be touched:

- `src/pages/GoalEditingSheet.test.tsx` — await the assertion target itself
  (`await screen.findByText('Tools · Practice, Visualizer, Tutor')`) rather than a different
  element that happens to appear earlier. Check the sibling assertions in the same file
  (`:23`, `:24`) for the same shape while here.
- `.ai/quality-gates.md` — once this is fixed and green, update the flaky-checks paragraph so
  it stops citing this file as a live precedent, or point it at this task id.

## Worklog

- 2026-09-05 — created as a follow-up from task 060's review of PR #4.
- 2026-09-09 — Claimed by claude and pulled forward: the flake stopped being a PR annoyance
  and failed on `master` itself (run `34375033315`, `frontend-checks (macos-latest)`, same
  test and same error), where there is no PR to re-run. Fixed and moved to `review`.

- 2026-09-09 — Independently reviewed by codex: verdict `pass`, no findings. It verified
  against `GoalEditingSheet.tsx` that the component renders its textarea before the async goal
  loads, confirming the old await was invalid synchronization rather than a masked
  render-ordering defect. `review` → `done` and archived as part of the merge of PR #10.

## What was built / tested / left out

Awaited the assertion targets themselves instead of a different element that happens to
render earlier:

- `src/pages/GoalEditingSheet.test.tsx:41` — replaced the `findByRole('textbox')` await plus
  a synchronous `getByText` with a single `await screen.findByText('Tools · Practice,
  Visualizer, Tutor')`. The old await settled as soon as the goal textbox existed, which
  never guaranteed the inferred facet chips had rendered.
- `src/pages/GoalEditingSheet.test.tsx:23-24` — the same latent shape in the sibling test, as
  this task's Plan section called for: both consequence assertions now use `findByText`. They
  had not been observed failing, but they depended on the same "an earlier element implies a
  later one" assumption.
- `.ai/quality-gates.md:12-15` — the flaky-checks paragraph no longer cites this file as a
  live precedent. It now records why the follow-up task matters: task 052 saw this flake and
  filed nothing, so it recurred through tasks 060 and 066 and reached `master`.

Tested: `npm test -- src/pages/GoalEditingSheet.test.tsx` passes (2 tests). Local passing
proves little for a flake — it passed locally before this change too — so the argument is the
mechanism, not the run: no assertion in this file now depends on a different element having
rendered first. CI on the PR is the gate.

Deliberately left out: the repo-wide audit of `getByText`-after-`findBy*` that this task's
Scope explicitly excluded. If that audit still looks warranted, it is its own task.

## Review

Reviewer: codex
Date: 2026-09-09

### Correctness

- [x] **Built behavior — pass.** The three changed assertions now await their own targets
  (`src/pages/GoalEditingSheet.test.tsx:23-24,41`); the textarea still renders before the
  asynchronously loaded goal (`src/pages/GoalEditingSheet.tsx:21-30,102-107`), so the old
  textbox await was invalid synchronization rather than evidence of a component-ordering bug.
- [x] **Test coverage — pass.** The focused file passes both cases, and the assertions still
  cover consequence copy, inferred facets, removal, revert, and save
  (`src/pages/GoalEditingSheet.test.tsx:17-49`) rather than merely exercising render.
- [x] **Edge cases — N/A.** This PR changes test synchronization and process documentation
  only; it changes no component props or production behavior
  (`src/pages/GoalEditingSheet.test.tsx:1-51`).

### Architecture conformance (`ARCHITECTURE.md`)

- [x] **Hook/page data ownership — pass.** No production import or data-flow rule changed;
  the test continues to render the page-level component directly
  (`src/pages/GoalEditingSheet.test.tsx:4,9-16,33-40`).
- [x] **Type placement/re-export — N/A.** No file under `src/types/` changed; the only source
  edit is `src/pages/GoalEditingSheet.test.tsx:17-49`.
- [x] **Async service contract — N/A.** No service function changed; the PR only waits for
  UI results of the existing async load (`src/pages/GoalEditingSheet.test.tsx:17-24,41`).
- [x] **Global state — N/A.** No state ownership changed; the test-only diff is confined to
  `src/pages/GoalEditingSheet.test.tsx:23-24,41`.

### UI rules (`AGENTS.md`)

- [x] **Design tokens — N/A.** No CSS or production UI value changed
  (`src/pages/GoalEditingSheet.test.tsx:23-24,41`).
- [x] **Markup reuse — N/A.** No component markup changed
  (`src/pages/GoalEditingSheet.test.tsx:23-24,41`).
- [x] **Screen fidelity — N/A.** The rendered screen is untouched; only how its existing
  output is awaited changed (`src/pages/GoalEditingSheet.test.tsx:23-24,41`).
- [x] **Copy rules — pass.** The PR adds no learner-facing copy and preserves the existing
  assertion strings verbatim (`src/pages/GoalEditingSheet.test.tsx:23-24,41`).

### Process

- [x] **Quality gates — pass.** PR #10's head `b6d3a4e` has all seven required checks green;
  the focused test also passes independently (2 tests), and `git diff --check` is clean.
- [x] **Worklog — pass.** The incident, exact synchronization change, validation limit, and
  deliberately excluded repo-wide audit are all recorded
  (`.ai/tasks/062-goal-editing-sheet-flake.md:65-93`).
- [x] **Scope — pass.** The sibling assertions are a justified defensive change, not scope
  creep: they had the identical cross-element timing dependency and were explicitly named in
  the Plan (`.ai/tasks/062-goal-editing-sheet-flake.md:54-61,79-82`). The diff touches only
  the planned test, this task record, and the flake policy.
- [x] **Architecture documentation — N/A.** No folder structure, top-level directory, shared
  type, or data-flow rule changed (`src/pages/GoalEditingSheet.test.tsx:23-24,41`).

### Findings

No blocking or non-blocking findings. The mechanical claim is true: every asynchronous
positive assertion awaits its own target (`src/pages/GoalEditingSheet.test.tsx:17-24,26,29,41`),
while the two negative assertions follow synchronous `fireEvent` state updates
(`src/pages/GoalEditingSheet.test.tsx:42-49`). The flake policy remains general—the immediate
re-run unblocks a PR and filing a follow-up is mandatory—with this incident serving only as
the rationale (`.ai/quality-gates.md:12-17`).

Verdict: pass

## Follow-ups
