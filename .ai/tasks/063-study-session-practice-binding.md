---
id: 063
title: Study Session <-> Practice attempt binding
status: proposed
owner: unassigned
stage: 8
depends_on: [059]
---

## Scope

Implement the design in `docs/superpowers/specs/2026-09-08-study-session-practice-binding-design.md`:
lock how a study session finds its problem family and resumes its current attempt. This is
the first of Stage 8's five remaining critical-path pieces (`ROADMAP.md` "Remaining Stage 8
scope") and the prerequisite the other four depend on.

Backend work, deliberately: two nullable schema columns, two new additive Practice
capabilities, and orchestration inside the existing `start_session` command. Does **not**
build: any `StudySessionPage` UI or frontend hook (next sub-project, presentation-only per
the roadmap), the "next problem" flow once an attempt is solved (design §5/§8 names the seam,
doesn't build it), the permanent regression corpus, or the offline acceptance test.

## Plan

Files expected to be created or touched (see the design doc for full detail on each):

- `src-tauri/src/db/migrations/000X_*.sql` — new migration: `ALTER TABLE concepts ADD COLUMN
  knowledge_concept_id TEXT`; `ALTER TABLE sessions ADD COLUMN current_attempt_id TEXT`.
- `src-tauri/src/commands/seed.rs` (or wherever the sample workspace is populated, task 041's
  territory) — set `knowledge_concept_id = 'shell.method_vertical_axis'` on the
  `concept-shells` row.
- `src-tauri/src/practice/types.rs` — `StartRequest`/`StartResponse`,
  `DescribeRequest`/`DescribeResponse`.
- `src-tauri/src/practice/provider.rs` — `practice.start` and `practice.describe` capability
  handlers; register both alongside the existing three.
- `src-tauri/src/practice/store.rs` — new query: given a workspace id and a candidate
  family-id list, the most recent attempt's `family_id` among them, if any (needed for the
  "avoid immediate repeat" rotation policy).
- `src-tauri/src/commands/session.rs` — `start_session_handler`: resolve the concept's
  `knowledge_concept_id`, call `practice.start` through `ModuleRegistry` if resolvable, and
  set `current_attempt_id` in the same `INSERT` (design §5's ordering matters: generate
  before insert, not after).
- Tests per design §7: migration, `practice.start` (including a multi-family property test
  using a test fixture, not the single bundled family), `practice.describe`, `start_session`
  orchestration (mapped/unmapped concept, module disabled, forced invoke error), and the
  crosswalk regression guard (`concept-shells`'s seeded id actually resolves in the bundled
  package).

If this list grows materially once work starts, that's a signal to split rather than expand
scope silently — see `.ai/lifecycle.md`.

## Worklog

- 2026-09-08 — Brainstormed and spec'd by claude (`docs/superpowers/specs/2026-09-08-study-session-practice-binding-design.md`),
  approved by the human. Filed as `proposed` for whichever agent picks it up.

## What was built / tested / left out

## Review

## Follow-ups
