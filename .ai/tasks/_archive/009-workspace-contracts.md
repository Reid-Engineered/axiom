---
id: 009
title: components/workspace/* stub contracts
status: done
owner: claude-code
stage: 2
depends_on: [005]
---

## Scope

Full prop interface + TSDoc for `WorkspaceCard`, `WorkspaceTree` — stub bodies. Implemented with `return null`, so this does not depend on 001's `Placeholder` (see worklog).

## Plan

- src/components/workspace/WorkspaceCard.tsx (stub)
- src/components/workspace/WorkspaceTree.tsx (stub)

## Worklog

- 2026-08-27 — started, claimed by claude-code. Branch `agent/claude-code/009-workspace-contracts`.
- 2026-08-27 — same as 008: 001 still `changes-requested`, unmerged. Used `return null` for
  both bodies, no dependency on `Placeholder` needed.
- 2026-08-27 — wrote both stubs. All gates pass. Moved to `review`.
- 2026-08-27 — Codex: `changes-requested` on process only, same as 008 — metadata still
  declared `depends_on: [005, 001]`. Set `depends_on: [005]`, reworded Scope. No code
  changed. Moved back to `review`.
- 2026-08-27 — Codex re-review: correctness/architecture/UI-rules pass; flagged Process —
  branch predated 013's merge. `git rebase master` — clean, no conflicts. All gates re-run —
  pass. Not merging myself. Moved back to `review`.

## What was built / tested / left out

- **Built**: `WorkspaceCard.tsx` (name, goal sentence, progress, last concept + relative
  time, paused), `WorkspaceTree.tsx` (workspace list, open workspace id, active sub-item,
  select callbacks) — both `return null`.
- **Tested**: `npm run typecheck` (0 errors), `npm run lint` (0 errors), `npm run build`
  (succeeded), grep for stray hex/`rgba(` (0 hits). No render tests — bodies are `return
  null`.
- **Left out**: `WorkspaceTree`'s four sub-items (Overview/Concepts/Material/Tools) are a
  fixed literal union, not data — they're permanent navigation, never fetched
  (`AXIOM-HANDOFF.md` §3). Real expand/collapse and routing wiring is Stage 3's task 017;
  this is contract only. No dependency on 001's `Placeholder` — see worklog.

## Review

Reviewer: codex
Date: 2026-08-27
- [x] Correctness — pass: card and two-level workspace-tree props cover their stated screens.
- [x] Architecture conformance — pass: both components are data-driven stubs with no service access.
- [x] UI rules — pass: modules cannot become tree rows and progress remains unlabelled.
- [ ] Process — FAIL: task metadata still declares dependency 001, which remains unresolved
  and unmerged. If `return null` genuinely removes that dependency, update the locked task
  metadata/scope accordingly; otherwise this task cannot enter review yet.

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

Reviewed commit: `e00c810`

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
