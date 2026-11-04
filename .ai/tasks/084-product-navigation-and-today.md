---
id: 084
title: Reid product navigation and Today
status: done
owner: antigravity
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
- 2026-11-04 — Claimed by antigravity after human approval of P1–P7 (`083`) and Track 1 vertical selection (`084` → `085` + `076` → `086`).
- 2026-11-04 — Implemented `errorProfile` and `sessionHistory` routes in `src/hooks/navigationContext.ts` and `src/layouts/RouteContent.tsx`; updated `Sidebar` with active-domain `Today` / `Practice` / `Error profile` / `Concept map` / `History` product navigation; implemented `TodayPlanCard` on `HomePage` with editable phase (`Warm-up`, `Practice`, `Review`), difficulty (`Steady`, `Push`, `Exam`), target duration (`15m`, `25m`, `45m`), and queued slot selection reasons backed by `practice.plan@1`; added Concept learning evidence strip on `ConceptViewPage` backed by `practice.evidence@1`.

## What was built / tested / left out

- Built:
  - `src/hooks/navigationContext.ts`, `src/layouts/RouteContent.tsx`, `src/App.tsx`, `src/components/workspace/Sidebar.tsx`, `src/components/workspace/Sidebar.module.css`
  - `src/pages/HomePage.tsx`, `src/pages/HomePage.module.css` (`TodayPlanCard`)
  - `src/pages/ConceptViewPage.tsx`, `src/pages/ConceptViewPage.module.css` (Concept learning evidence strip)
  - `src-tauri/src/practice/engine.rs` (`create_plan`, `get_concept_evidence`)
- Tested:
  - `src/pages/HomePage.test.tsx`, `src/pages/ConceptViewPage.test.tsx`, `src/App.test.tsx`, `src-tauri/src/commands/tests.rs` (`stage_9_learning_evidence_commands_run_end_to_end`)

## Review

Ready for review.

## Follow-ups

None.
