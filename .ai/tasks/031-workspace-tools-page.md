---
id: 031
title: WorkspaceToolsPage + offline/modules-at-scale sheet
status: in-progress
owner: antigravity
stage: 6
depends_on: [003, 021]
---

## Scope

Full implementation: module rows, `Toggle`, `SuggestionPanel`. Includes the offline/modules-at-scale sheet behavior from `AXIOM-HANDOFF.md` §5.

## Plan

- src/pages/WorkspaceToolsPage.tsx
- associated .module.css
- complete the locked `Sheet` overlay body used by the offline flow

## Worklog

- 2026-08-28 — Claimed by Codex. Starting 20-module visibility grouping, per-workspace enable toggles, four-goal context, and the per-kind offline sheet. Global `Module.visibility` will be consumed as locked, without a per-workspace workaround.
- 2026-08-28 — Codex implementation pass complete. Built module rows with real per-workspace enabled toggles, grouped modules strictly by the locked global `visibility` field, wired marketplace/module-detail navigation and the suggestion, completed the locked `Sheet` body, and built the four-kind offline flow with honest totals and the fixture's partial-video limitation. Demonstrated against the actual Calculus II fixtures: 20 modules grouped 4 In the workspace / 9 Appear when relevant / 7 Off, four goals, and four offline-kind switches. No per-workspace visibility field or workaround was introduced; the existing global behavior does not make this single-workspace screen internally contradictory. Full gates passed: typecheck, lint, production build, 95/95 tests, hardcoded color scan, component-to-service import scan, and `git diff --check`. Handed to Antigravity for fidelity against `08-workspace-tools.png` and `21-offline-modules-goals.png`; status remains `in-progress`.

## What was built / tested / left out

Codex grouping, hook wiring, offline sheet, scale tests, and Sheet primitive are complete. Antigravity screenshot polish remains before review.

## Review

Filled in by the reviewing agent. See `.ai/review-checklist.md`.

## Follow-ups

Anything noticed during implementation or review that's out of this task's scope.
