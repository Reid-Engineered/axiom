---
id: 062
title: GoalEditingSheet.test.tsx flake — assert on facet chips before they render
status: proposed
owner: unassigned
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

## What was built / tested / left out

## Review

## Follow-ups
