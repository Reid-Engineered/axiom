---
id: 083
title: Lock mastery diagnostic and adaptive-session contracts
status: proposed
owner: claude
stage: 9
depends_on: [081]
---

## Scope

Design/version provider-owned mastery, Bloom/readiness, misconception opportunities, difficulty costs/hint budgets, resolution rules, persisted plans/queues, timing and verification evidence. Resolve actual policy values before implementation. No subject-specific policy in Core.

Product authority: reference/UI/AXIOM-HANDOFF.md and
docs/superpowers/specs/2026-10-10-reid-design-adoption.md.

## Plan

- CORE.md and ARCHITECTURE.md addenda; capability and Knowledge Package specs under docs/superpowers/specs/.
- Inventory existing src/types/*, src-tauri/src/{practice,knowledge,capabilities,commands,db}.
- Lock payloads, error/unavailable states, migrations, session resume and idempotency boundaries.
- Coordinate existing task 076 timing scope; retain Stage 8 regression/offline work.

Acceptance:
- Every adopted field has a real owner, provenance, version and persistence rule.
- No invented numeric conversion of legacy mastery; define absent/historical evidence behavior.
- Opportunity attribution differs from observed misconception; policies disclose costs.
- Conformance/test plan covers repeated submissions, skips, domain isolation, restart, insufficient content and numerical verification limits.

## Worklog

- 2026-10-10 — Scoped following the human's full design/product adoption in task 081.

## What was built / tested / left out

Not started. Implementation must name its checks and link its PR.

## Review

Pending implementation and independent review.

## Follow-ups

Split implementation further if a locked contract or file boundary requires it.
