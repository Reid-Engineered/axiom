---
id: 086
title: Error profile and session history
status: proposed
owner: codex
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

## What was built / tested / left out

Not started. Implementation must name its checks and link its PR.

## Review

Pending implementation and independent review.

## Follow-ups

Split implementation further if a locked contract or file boundary requires it.
