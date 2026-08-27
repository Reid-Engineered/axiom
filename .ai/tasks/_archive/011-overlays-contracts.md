---
id: 011
title: components/overlays/* stub contracts
status: done
owner: claude-code
stage: 2
depends_on: [005]
---

## Scope

Full prop interface + TSDoc for `Sheet`, `Popover`, `Inspector`, `CommandPalette` — stub bodies. `CommandPalette` gets real results in Stage 6 (035); this task is contract only.

## Plan

- src/components/overlays/Sheet.tsx (stub)
- src/components/overlays/Popover.tsx (stub)
- src/components/overlays/Inspector.tsx (stub)
- src/components/overlays/CommandPalette.tsx (stub)

## Worklog

- 2026-08-27 — started, claimed by claude-code. Branch `agent/claude-code/011-overlays-contracts`.
- 2026-08-27 — wrote all 4 stubs. All gates pass. Moved to `review`.
- 2026-08-27 — Codex: `changes-requested` — `CommandPaletteResultItem`'s flat label/detail
  couldn't carry a Concepts result's mastery glyph or a marketplace result's trust badge.
  Read task 035's own scope before fixing rather than guessing the shape: it explicitly
  reuses `ConceptRow` directly for Concepts results, not a bare mastery ring — so a first
  attempt at a `leading?: ReactNode` icon-slot next to `label` would have left `ConceptRow`'s
  internal name/status rendering duplicated against the item's own `label`. Replaced
  `label`/`detail` with a single `content: ReactNode` — 035 can now pass a plain string, or
  `<ConceptRow .../>`, or `<TrustBadge .../>` + label directly. All gates re-run — pass.
  Moved back to `review`.
- 2026-08-27 — Codex re-review: correctness/architecture/UI-rules pass; flagged Process —
  branch predated 013's merge. `git rebase master` — clean, no conflicts. All gates re-run —
  pass. Not merging myself. Moved back to `review`.

## What was built / tested / left out

- **Built**: `Sheet.tsx` (eyebrow/title/footer slots, open/onClose), `Popover.tsx`
  (anchor ref, open/onClose), `Inspector.tsx` (title, open/onClose), `CommandPalette.tsx`
  (query, grouped results with `content: ReactNode` per item, scope label) — all
  `return null`.
- **Tested**: `npm run typecheck` (0 errors), `npm run lint` (0 errors), `npm run build`
  (succeeded), grep for stray hex/`rgba(` (0 hits). No render tests — bodies are `return
  null`.
- **Left out**: `CommandPalette`'s `groups: CommandPaletteResultGroup[]` accepts an empty
  array without issue — Stage 3's task 018 wires stub-level open/close with empty results;
  Stage 6's task 035 is the one that populates real groups. No result-fetching logic here,
  per this task's own scope note.

## Review

Reviewer: codex
Date: 2026-08-27
- [ ] Correctness — FAIL: `CommandPaletteResultItem` only supports plain label/detail text.
  It cannot carry the mastery state/status required for Concept results or the trust level
  required for the marketplace result, so task 035 could not compose its specified
  `ConceptRow` and trust badge without changing this locked contract.
- [x] Architecture conformance — pass: overlays remain props-driven and fetch no data.
- [x] UI rules — pass: no hardcoded design values or prohibited copy is introduced.
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

Reviewed commit: `3bd51aa`

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
