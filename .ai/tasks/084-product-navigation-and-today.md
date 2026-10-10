---
id: 084
title: Reid product navigation and Today
status: proposed
owner: codex
stage: 9
depends_on: [082, 083]
---

## Scope

Implement domain-scoped Today / Practice / Error profile / Concept map / History navigation and the editable session plan. Preserve all existing destinations, exact resume context, zero-domain launch and explicit sample exploration.

Product authority: reference/UI/AXIOM-HANDOFF.md and
docs/superpowers/specs/2026-10-10-reid-design-adoption.md.

## Plan

- NavigationContext route variants; App.tsx page wiring; props-driven Sidebar contracts.
- Home/Today page and hooks/services for real plan/reasons/readiness.
- Domain selection backed by existing workspace ids; Settings/tools/material/Marketplace access.
- Concept map page/layout against the locked graph/readiness contract.

Acceptance:
- Fresh installs have no phantom domains, scores or history.
- Switching domains scopes all counts, goals and actions; unready pages show honest empty/unavailable states.
- Resuming uses the bound attempt; a new plan cannot overwrite active work.
- Hook/conditional component tests and native launch/create/switch/resume checks pass.

## Worklog

- 2026-10-10 — Scoped following the human's full design/product adoption in task 081.

## What was built / tested / left out

Not started. Implementation must name its checks and link its PR.

## Review

Pending implementation and independent review.

## Follow-ups

Split implementation further if a locked contract or file boundary requires it.
