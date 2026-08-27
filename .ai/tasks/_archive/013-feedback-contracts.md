---
id: 013
title: components/feedback/* stub contracts
status: done
owner: claude-code
stage: 2
depends_on: [005]
---

## Scope

Full prop interface + TSDoc for `ReasonedRecommendation`, `SuggestionPanel` — stub bodies. **Flag**: `ARCHITECTURE.md` §2 lists `components/feedback/` in the folder structure, but `ROADMAP.md` Stage 2's deliverable bullet enumerates concept/workspace/session/overlays/math and omits feedback/. Included here for consistency with the architecture doc's component inventory; surfacing the omission for human confirmation rather than silently dropping the folder or silently editing ROADMAP.md without sign-off (`.ai/merge-strategy.md`).

## Plan

- src/components/feedback/ReasonedRecommendation.tsx (stub)
- src/components/feedback/SuggestionPanel.tsx (stub)

## Worklog

- 2026-08-27 — started, claimed by claude-code. Branch `agent/claude-code/013-feedback-contracts`.
  Proceeding with implementation despite the flagged ROADMAP.md wording question (Scope /
  Follow-ups) — building the stubs doesn't foreclose either resolution of that question, and
  it's raised separately to the human rather than blocking on it.
- 2026-08-27 — wrote both stubs. All gates pass. Moved to `review`.

## What was built / tested / left out

- **Built**: `ReasonedRecommendation.tsx` (action, evidence, CTA, optional dated
  observations for "Why this?", optional alternative), `SuggestionPanel.tsx` (message,
  accept/dismiss) — both `return null`.
- **Tested**: `npm run typecheck` (0 errors), `npm run lint` (0 errors), `npm run build`
  (succeeded), grep for stray hex/`rgba(` (0 hits). No render tests — bodies are `return
  null`.
- **Left out**: nothing beyond the two components in scope. The ROADMAP.md wording question
  from Scope/Follow-ups is still open — not resolved by building these, just not blocking on.

## Review

Reviewer: codex
Date: 2026-08-27
- [x] Correctness — pass: both contracts cover the evidence, action, alternative, accept,
  and dismiss behavior specified for recommendation and suggestion surfaces.
- [x] Architecture conformance — pass: components are single-purpose, props-driven stubs.
- [x] UI rules — pass: feedback is evidence-based and introduces no hardcoded design values.
- [x] Process — pass: scope follows the architecture inventory; typecheck, lint, build,
  hardcoded-value grep, and diff check pass.

Verdict: pass

Filled in by the reviewing agent. See `.ai/review-checklist.md`.

## Follow-ups

Open question raised in Scope: confirm with the human whether ROADMAP.md Stage 2's deliverable list should be updated to name components/feedback/ explicitly.
