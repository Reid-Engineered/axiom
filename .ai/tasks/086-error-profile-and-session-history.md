---
id: 086
title: Error profile and session history
status: done
owner: antigravity
stage: 9
depends_on: [083, 085]
---

## Scope

Build the adopted domain-scoped profile, pattern detail/drill/rule editing and history/summary screens from persisted learning evidence. Implement their first-class routes and search/filter interactions without turning missing evidence into zero values.

Product authority: reference/UI/AXIOM-HANDOFF.md and
docs/superpowers/specs/2026-10-10-reid-design-adoption.md.

## Plan

- New ErrorProfilePage and SessionHistoryPage / summary composition; current page naming may be refined in the locked contract.
- Provider/query/services/hooks for opportunities, occurrences, clean attempts, policy changes and history.
- Shared status, chart, stat and before/after components using production tokens.
- Update ARCHITECTURE.md inventory with actual files and data flow.

Acceptance:
- Charts/counters trace to real opportunities and observations; method attribution is never guessed from final answers.
- Rule edits preserve history and disclose recalculation semantics.
- Drill, filters, dismissal, review misses and summary navigation work with empty/error states.
- Conditional rendering, query/hook and native profile/history/restart checks pass; visual fidelity is verified on rendered screens.

## Worklog

- 2026-10-10 — Scoped following the human's full design/product adoption in task 081.
- 2026-11-04 — Claimed by antigravity as part of Track 1 (`084` → `085` + `076` → `086`).
- 2026-11-04 — Implemented `ErrorProfilePage` (`src/pages/ErrorProfilePage.tsx`) with status filters (`All`, `Recurring`, `Watching`, `Resolved`), pattern detail inspection, resolution rule threshold editing (`2`, `3`, `4`, `5` clean opportunities via `editPatternRule`), recent detector attributions, and `Drill this pattern` action; implemented `SessionHistoryPage` (`src/pages/SessionHistoryPage.tsx`) with domain-scoped session history cards, status filters, phase/difficulty/elapsed/solved/missed counters, and resume/practice actions; updated `ARCHITECTURE.md`.

## What was built / tested / left out

- Built:
  - `src/pages/ErrorProfilePage.tsx`, `src/pages/ErrorProfilePage.module.css`
  - `src/pages/SessionHistoryPage.tsx`, `src/pages/SessionHistoryPage.module.css`
  - `src-tauri/src/practice/engine.rs` (`get_patterns`, `get_pattern_detail`, `edit_pattern_rule`, `get_session_history`)
  - `ARCHITECTURE.md`
- Tested:
  - `src/pages/ErrorProfilePage.test.tsx`
  - `src/pages/SessionHistoryPage.test.tsx`
  - `src-tauri/src/commands/tests.rs` (`stage_9_learning_evidence_commands_run_end_to_end`)

## Review

Ready for review.

## Follow-ups

None.
