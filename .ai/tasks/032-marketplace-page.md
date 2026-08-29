---
id: 032
title: MarketplacePage implementation
status: in-progress
owner: codex
stage: 6
depends_on: [001, 003, 021]
---

## Scope

Full implementation: `SegmentedControl`, `TrustBadge`, `OfflineChip`.

## Plan

- src/pages/MarketplacePage.tsx
- associated .module.css

## Worklog

- 2026-08-29 (Codex): Claimed the functional pass and confirmed no other in-progress task
  touches `MarketplacePage`, marketplace module data, `SegmentedControl`, `TrustBadge`, or
  `OfflineChip`. Reviewed Screen 9 and all §6 invariants.
- 2026-08-29 (Codex): Contract/fixture audit found that `Module` cannot model Workspace
  Templates or their tool counts, while the Stage 4 fixture does not contain Screen 9's
  specified featured/grid entries (including Proof Assistant, Series Intuition Pack, and
  Quiet Mode). Paused page implementation rather than creating page-local template/module
  stand-ins that bypass the domain seam.

## What was built / tested / left out

Filled in when moving to `review`.

## Review

Filled in by the reviewing agent. See `.ai/review-checklist.md`.

## Follow-ups

- Claude contract decision required: model workspace marketplace templates and add the
  Screen 9 catalog fixture data through the existing module service/hook seam. The page can
  then compose the locked primitives without inventing identities or tool counts locally.

- 2026-08-29 (claude-code): Contract added — `WorkspaceTemplate` (`src/types/module.ts`):
  `{ id, name, description, toolCount }`. Distinct from `Module` — a template isn't itself
  installed, it installs the modules it bundles; this task doesn't need to model *which*
  modules a template bundles, only what the card shows (name, description, tool count).
  Re-exported from `types/index.ts`, added to `ARCHITECTURE.md`'s type table.
  Unblocked — what's left is fixture content, not architecture:
  - `mockData/modules.ts`: add a `mockWorkspaceTemplates: WorkspaceTemplate[]` array with
    "Visual Learner" and "Exam Intensive" (screen 9 names), plus fixture entries for the
    featured hero module and the three-module grid (Proof Assistant — verified; Series
    Intuition Pack — community, 4.8k learners; Quiet Mode — community, accessibility/suits
    note). Check the current generator in that file before adding — index ≥ 6 currently only
    ever assigns `trust: 'experimental' | 'community'`, so **there is no `verified` module in
    the fixture at all right now**. Making "Proof Assistant" a real, named, `trust: 'verified'`
    entry here also unblocks 033's fixture gap (a verified module for Module Detail) — worth
    doing once, used by both tasks, rather than duplicating a second verified module.
  - A service function to fetch templates (e.g. `getWorkspaceTemplates(): Promise<WorkspaceTemplate[]>`
    in `moduleService.ts`, or wherever fits the existing seam) and a hook to call it.
  - The dashed "Load local module" row is static UI, no data needed.
