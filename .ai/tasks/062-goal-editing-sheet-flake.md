---
id: 062
title: GoalEditingSheet.test.tsx flake — assert on facet chips before they render
status: review
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

## Follow-ups
