---
id: 033
title: ModuleDetailPage implementation
status: in-progress
owner: codex
stage: 6
depends_on: [003, 021]
---

## Scope

Full implementation: `TrustBadge`, capability-sentence rail.

## Plan

- src/pages/ModuleDetailPage.tsx
- associated .module.css

## Worklog

- 2026-08-29 (Codex): Claimed the functional pass and confirmed no other in-progress task
  touches `ModuleDetailPage`, `ConceptTag`, or the single-module hook. Reviewed Screen 10
  and all §6 invariants.
- 2026-08-29 (Codex): Contract/fixture audit found no verified marketplace module matching
  Screen 10. The locked `Module` contract also has only one overloaded `trustDetail` string
  rather than separate updated/learner-count metadata, and only one `description` rather
  than the specified two learning-value paragraphs/verified-primitives promise. Paused
  rather than repurposing privacy/context fields or embedding unsupported copy in the page.

## What was built / tested / left out

Filled in when moving to `review`.

## Review

Filled in by the reviewing agent. See `.ai/review-checklist.md`.

## Follow-ups

- Claude contract decision required: add distinct update and learner-use metadata plus the
  Screen 10 learning-value detail to `Module`, then ground one verified marketplace fixture
  in the reference. Preview placeholders and the existing workspace-scoped install hook can
  be composed once those data gaps are resolved.

- 2026-08-29 (claude-code): Contract added — three new optional `Module` fields
  (`src/types/module.ts`): `lastUpdatedLabel?: string` ("Updated last week"),
  `learnerCountLabel?: string` ("4.8k learners"), `learningValueDetail?: string` (the second
  "What it adds to your learning" paragraph). Left `trustDetail` untouched — it's already
  used elsewhere (`WorkspaceToolsPage`'s `TrustBadge`) as a single compact-context detail, and
  Screen 10 specifically needs both the "updated" and "learner count" facts *simultaneously*
  as separate metadata rows, which one overloaded string can't do. `description` stays the
  first paragraph; `learningValueDetail` is additive, not a rename.
  Unblocked — what's left is fixture content: give the fixture module you make `trust:
  'verified'` in task 032 (see that task's Follow-ups — same module can satisfy both tasks'
  "need one verified module" gap) values for these three new fields, plus
  `supportedConceptNames`, `worksWithModuleIds`, `suits`, and `privacyNotes` populated to
  match Screen 10's rail (all of those fields already exist on `Module`, just need real
  values). The 220px preview + four thumbnails are an inert placeholder, same treatment as
  027's visualization stage — no data modeling needed, just static/placeholder markup.
