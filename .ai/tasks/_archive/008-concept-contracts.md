---
id: 008
title: components/concept/* stub contracts
status: done
owner: claude-code
stage: 2
depends_on: [005]
---

## Scope

Full prop interface + TSDoc for `ConceptRow`, `ConceptTag` — stub bodies (`return null` or `Placeholder`). Implemented with `return null`, so this does not depend on 001's `Placeholder` (see worklog).

## Plan

- src/components/concept/ConceptRow.tsx (stub)
- src/components/concept/ConceptTag.tsx (stub)

## Worklog

- 2026-08-27 — started, claimed by claude-code. Branch `agent/claude-code/008-concept-contracts`.
- 2026-08-27 — dependency 001 (Stage 1 primitives, for `Placeholder`) is still
  `changes-requested`, unmerged. The scope note allows `return null` **or** `Placeholder` for
  stub bodies — used `return null` for both components, which needs nothing from 001. Not
  blocked; no cross-branch import taken.
- 2026-08-27 — wrote both stubs. All gates pass. Moved to `review`.
- 2026-08-27 — Codex: `changes-requested` on process only — metadata still declared
  `depends_on: [005, 001]` despite not actually using anything from 001. Correct: the prior
  worklog entry explained the decision but never updated the frontmatter/scope to match. Set
  `depends_on: [005]` and reworded the Scope sentence. No code changed. Moved back to
  `review`.
- 2026-08-27 — Codex re-review: correctness/architecture/UI-rules pass; flagged Process —
  branch predated 013's merge to `master`. `git rebase master` — clean, no conflicts. All
  gates re-run — pass. Not merging myself (task author, not reviewer). Moved back to
  `review`.

## What was built / tested / left out

- **Built**: `ConceptRow.tsx` (name, `MasteryState`, optional status text, `onSelect`),
  `ConceptTag.tsx` (label, `onSelect`) — both `return null`.
- **Tested**: `npm run typecheck` (0 errors), `npm run lint` (0 errors), `npm run build`
  (succeeded), grep for stray hex/`rgba(` (0 hits). No render tests — bodies are `return
  null`; real tests land with the Stage 5/6 implementation tasks that give these bodies.
- **Left out**: no dependency on 001's `Placeholder` — see worklog. `ConceptRow` takes a
  pre-computed `statusText` string rather than raw `Concept` fields, since deriving "active"
  / "due for review" / "blocks 3 concepts" from the domain model is a hook/page concern
  (`ARCHITECTURE.md` §5 rule 1), not this component's.

## Review

Reviewer: codex
Date: 2026-08-27
- [x] Correctness — pass: the props cover the specified reusable concept-row and tag data.
- [x] Architecture conformance — pass: both components are props-driven stubs importing
  domain types only through the public barrel.
- [x] UI rules — pass: the contract preserves mastery word/ring pairing and has no design values.
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

Reviewed commit: `a8a58d5`

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
