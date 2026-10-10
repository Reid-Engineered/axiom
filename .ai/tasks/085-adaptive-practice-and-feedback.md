---
id: 085
title: Adaptive Practice and diagnostic feedback
status: proposed
owner: codex
stage: 9
depends_on: [083, 084]
---

## Scope

Implement real mastery/pattern updates, difficulty policy, persisted adaptive question selection, timing and evidence-driven feedback behind the adopted Practice screen. Keep hint budgets/costs and checker detail truthful to the locked contracts.

Product authority: reference/UI/AXIOM-HANDOFF.md and
docs/superpowers/specs/2026-10-10-reid-design-adoption.md.

## Plan

- Versioned Practice/verification capability implementation and DB migrations from 083.
- services → hooks → StudySessionPage; shared question/answer/feedback/inspector components.
- Persist phase, position, queue revisions, evaluation effects, partial success and timestamps.
- Show worked solution, Next question, Skip and Why this question with real destinations.

Acceptance:
- A repeated submission cannot apply mastery or queue changes twice.
- Hints, skips, missing attribution and insufficient family content follow disclosed policy.
- Pause/restart resumes the exact attempt and updated queue; domain isolation holds.
- Rust/provider/migration tests, hook/render tests, required CI/native E2E and network-disabled Practice acceptance pass.

## Worklog

- 2026-10-10 — Scoped following the human's full design/product adoption in task 081.

## What was built / tested / left out

Not started. Implementation must name its checks and link its PR.

## Review

Pending implementation and independent review.

## Follow-ups

Split implementation further if a locked contract or file boundary requires it.
