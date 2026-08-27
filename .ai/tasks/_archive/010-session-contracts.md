---
id: 010
title: components/session/* stub contracts
status: done
owner: claude-code
stage: 2
depends_on: [005]
---

## Scope

Full prop interface + TSDoc for `SessionToolbar`, `WorkspaceToolbar`, `WorkingArea` — stub bodies.

## Plan

- src/components/session/SessionToolbar.tsx (stub)
- src/components/session/WorkspaceToolbar.tsx (stub)
- src/components/session/WorkingArea.tsx (stub)

## Worklog

- 2026-08-27 — started, claimed by claude-code. Branch `agent/claude-code/010-session-contracts`.
- 2026-08-27 — wrote all 3 stubs. All gates pass. Moved to `review`.
- 2026-08-27 — Codex: `changes-requested` — confirmed against screen 5: the "five-dash
  session progress" and "12′ of 30′" time readout are separate, adjacent elements, and the
  five dashes mirror the problem pane's "Problem 3 of 5", not the time. `SessionToolbar` had
  no way to receive that. Added `problemIndex`/`problemCount` (matching `Session`'s existing
  fields from 005/006). All gates re-run — pass. Moved back to `review`.
- 2026-08-27 — Codex re-review: correctness/architecture/UI-rules pass; flagged Process —
  branch predated 013's merge. `git rebase master` — clean, no conflicts. All gates re-run —
  pass. Not merging myself. Moved back to `review`.

## What was built / tested / left out

- **Built**: `SessionToolbar.tsx` (concept/subject, `SessionIntent`, change-intent callback,
  problem index/count, elapsed/target minutes, pause), `WorkspaceToolbar.tsx` (workspace
  name, offline-available
  flag, right-aligned `children` slot for page-specific actions), `WorkingArea.tsx`
  (controlled textarea-style value/onChange) — all `return null`.
- **Tested**: `npm run typecheck` (0 errors), `npm run lint` (0 errors), `npm run build`
  (succeeded), grep for stray hex/`rgba(` (0 hits). No render tests — bodies are `return
  null`.
- **Left out**: `WorkspaceToolbar` intentionally takes only `workspaceName` +
  `offlineAvailable` as fixed props and a `children` slot for the rest — the handoff doesn't
  fully spec this component's contents beyond height and the offline chip, so page-specific
  actions are left to whoever implements each page rather than guessed here.

## Review

Reviewer: codex
Date: 2026-08-27
- [ ] Correctness — FAIL: `SessionToolbar` claims ownership of the five-dash session progress
  but accepts no problem/progress value, so it cannot render the designed current position
  (screen 5 shows problem 3 of 5). Add a typed progress input consistent with `Session`.
- [x] Architecture conformance — pass: all components are props-driven stubs using public types.
- [x] UI rules — pass: no hardcoded design values or prohibited navigation model is introduced.
- [x] Process — pass: typecheck, lint, build, hardcoded-value grep, and diff check pass.

Verdict: changes-requested

Filled in by the reviewing agent. See `.ai/review-checklist.md`.

### Re-review — 2026-08-27

- [x] Correctness — pass: the prior task-specific contract finding is resolved.
- [x] Architecture conformance — pass: the revised contract remains within Stage 2 scope.
- [x] UI rules — pass: the revised API preserves the applicable product invariants.
- [ ] Process — FAIL: this branch is based before merged task 013. Compared with current
  `master`, it deletes `src/components/feedback/*`, removes the archived 013 task, and
  restores 013 to the active queue. Per `.ai/merge-strategy.md`, the owner must rebase onto
  current `master` and rerun all gates before re-requesting review.

Verdict: changes-requested

### Exact-HEAD re-review — 2026-08-27

Reviewed commit: `2c621c9`

- [x] Correctness — pass: the original contract finding is fully resolved.
- [x] Architecture conformance — pass: the contract remains props-driven and within the
  locked Stage 2 boundary.
- [x] UI rules — pass: applicable product invariants and token rules are preserved.
- [x] Process — pass: this exact HEAD is rebased on `170061a`; its diff contains only the
  task's intended files. Independent typecheck, lint, build, hardcoded-value grep, and
  diff check all pass.

Verdict: pass

## Follow-ups

Anything noticed during implementation or review that's out of this task's scope.
