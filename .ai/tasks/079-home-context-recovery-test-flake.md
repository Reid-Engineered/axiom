---
id: 079
title: HomePage context-recovery test races the concepts fetch
status: review
owner: claude
stage: 8
depends_on: []
---

## Scope

Flake follow-up, filed per `.ai/quality-gates.md`'s rule that a check which fails once and
passes clean on re-run is treated as a pass **and gets a follow-up task**. That doc records
what happens otherwise: task `052` hit a flake, filed nothing, and the same test went on
flaking through `060` and `066` before `062` finally fixed it.

`HomePage > replaces Continue with bounded context recovery after a long absence`
(`src/pages/HomePage.test.tsx`) failed on `frontend-checks (windows-latest)` during PR #21's
run, then passed clean on an immediate re-run with no code change.

## Diagnosis

Not a mystery, and not caused by `072` or `075` — it is a latent race the two of them made
easier to hit.

The test awaits only the workspace heading:

```ts
const title = await screen.findByRole('heading', {
  name: 'You were working with Angular momentum',
});
```

That heading renders as soon as the *workspace* resolves. The next assertions then use
**synchronous** queries:

```ts
const recoveryLines = within(recovery!)
  .getByText(/held up while you were away/)   // getBy, not findBy
  .closest('ul');
```

`ContextRecovery` builds those lines from `useConcepts(workspace.id)`, a separate async
resource. On a fast runner the concepts have already arrived; on a slower Windows runner they
have not, `concepts` is still empty, `held` is `undefined`, and the line never renders. The CI
DOM dump confirms it — the recovery section rendered with an empty `<ol />` and no mastery
lines.

The same latent race applies to the `While you were away` events list, which reads from
`useRecentWorkspaceActivity`.

## Plan

- `src/pages/HomePage.test.tsx` — await the concept-dependent content rather than assuming it,
  e.g. `await within(recovery!).findByText(/held up while you were away/)` before the
  `closest('ul')` walk, and the same for the away-events section.
- Check the sibling tests in that file for the same pattern; the fix is only worth doing once.

Deliberately a test-only change. `ContextRecovery` itself is correct — it renders what it has
and renders nothing for data that has not arrived.

## Worklog

- 2026-09-10 — Filed by claude after the flake surfaced on PR #21 and cleared on re-run.
- 2026-09-10 — **Escalated and fixed immediately rather than left proposed.** It recurred twice
  more within the hour, on PR #23 — a docs-only change that touches nothing but `.ai/tasks/`.
  Three consecutive runs, once each on `windows-latest`, `macos-latest` and `ubuntu-latest`,
  every one the same assertion. That is not an occasional flake the quality-gates re-run rule
  covers; it was blocking unrelated work on every platform, so claude claimed and fixed it.
- 2026-09-10 — Likely why it got worse: `071` merged between the first sighting and the rest.
  `mockBackend`'s `startSession` now also writes `workspaces.lastActivityAt`, which shifts the
  ordering of the async work this test races against. The race was always present; `071` made
  it the likely outcome rather than the unlikely one.

## What was built / tested / left out

- `src/pages/HomePage.test.tsx` only. Replaced the synchronous `getByText(/held up while you
  were away/)` with `await findByText`, wrapped the two `getAllByRole('listitem')` length
  assertions in `waitFor`, and made the decayed-concept assertion a `findByText`. A comment
  records *why* — those three assertions read from `useConcepts` and
  `useRecentWorkspaceActivity`, which resolve after the workspace that renders the heading the
  test already awaited.
- No production code changed. `ContextRecovery` was always correct: it renders nothing for data
  that has not arrived, which is exactly what the test was failing to wait for.
- Tested: the file run five times consecutively, 4/4 passing each time, then the full suite —
  `npx vitest run` 61 files / 165 tests — plus `npm run typecheck`, `npm run lint` and
  `npm run build`. Rust gates are not applicable; no file under `src-tauri/` changed.
- Left out: the sibling tests in the file were checked for the same pattern and do not have it.
  They either await the element they assert on or assert only workspace-derived content.

## Review

## Follow-ups
