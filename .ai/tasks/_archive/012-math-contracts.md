---
id: 012
title: components/math/* stub contracts
status: done
owner: claude-code
stage: 2
depends_on: [005]
---

## Scope

Full prop interface + TSDoc for `MathInline`, `MathDisplay` (STIX Two Text per `AGENTS.md` UI rules) — stub bodies.

## Plan

- src/components/math/MathInline.tsx (stub)
- src/components/math/MathDisplay.tsx (stub)

## Worklog

- 2026-08-27 — started, claimed by claude-code. Branch `agent/claude-code/012-math-contracts`.
- 2026-08-27 — wrote both stubs. All gates pass. Moved to `review`.
- 2026-08-27 — Codex: `changes-requested` — confirmed against screen 5: "the integral in a
  well with the selected term `x` highlighted and an 'Ask about x' affordance" needs
  per-term structure, which a flat string can't carry. Added a shared `MathSegment { text,
  selected? }` type (in `MathDisplay.tsx`, imported by `MathInline.tsx`) — `expression` is
  now `string | MathSegment[]`, plus an `onSelectTerm` callback. No parser added — segments
  are caller-supplied, same "no math parser in this phase" stance as before, just no longer
  blocking the interaction the screen actually needs. Applied to both components for a
  consistent math-primitive API, even though only the display well currently needs it. All
  gates re-run — pass. Moved back to `review`.
- 2026-08-27 — Codex re-review: correctness/architecture/UI-rules pass; flagged Process —
  branch predated 013's merge. `git rebase master` — clean, no conflicts. All gates re-run —
  pass. Not merging myself. Moved back to `review`.

## What was built / tested / left out

- **Built**: `MathInline.tsx`, `MathDisplay.tsx` — both take `expression: string |
  MathSegment[]` plus an optional `onSelectTerm`, `return null`.
- **Tested**: `npm run typecheck` (0 errors), `npm run lint` (0 errors), `npm run build`
  (succeeded), grep for stray hex/`rgba(` (0 hits). No render tests — bodies are `return
  null`.
- **Left out**: `expression` is a pre-typeset string, not a LaTeX/MathML input — no math
  parser is in `package.json`, and nothing in the mock-data phase requires one (`AGENTS.md`
  Engineering principles: YAGNI). If real rendering later needs a parser, that's a Stage 5/6
  finding against this contract, not something to pull in speculatively now.

## Review

Reviewer: codex
Date: 2026-08-27
- [ ] Correctness — FAIL: a single pre-typeset `expression: string` cannot represent screen
  5's selected term with distinct highlighting and its "Ask about x" interaction. The full
  Stage 2 contract needs a structured selection/interaction prop (without requiring a math
  parser) so later implementation does not change the API.
- [x] Architecture conformance — pass: both components are props-driven stubs.
- [x] UI rules — pass: STIX typography is documented and no design values are introduced.
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

Reviewed commit: `7dbcaf4`

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
