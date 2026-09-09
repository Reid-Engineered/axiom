---
id: 063
title: Study Session <-> Practice attempt binding
status: review
owner: codex
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
- 2026-09-08 — Claimed by codex on `agent/codex/063-study-session-practice-binding`,
  branched directly from `master`; implementation began only after the approved carrier PR
  landed on `master`.
- 2026-09-09 — Implemented the schema crosswalk, additive Practice capabilities, session
  orchestration, retained-sample binding, and the full backend test matrix from design §7.
- 2026-09-09 — Opened [PR #8](https://github.com/Reid-Engineered/axiom/pull/8); all seven
  required CI checks passed, including `backend-checks (windows-latest)` and E2E.

## What was built / tested / left out

- Added nullable `concepts.knowledge_concept_id` and `sessions.current_attempt_id` columns,
  including migration coverage that starts at schema v2 and proves existing rows survive
  with both new values unset.
- Added `practice.start` and `practice.describe` alongside the existing three Practice
  capabilities. Start handles zero/one/multiple-family selection and excludes the most
  recent candidate family when alternatives exist; describe returns only learner-facing
  attempt state.
- Updated `start_session` to resolve the stored concept crosswalk, invoke Practice before
  the session insert, persist the returned attempt id in that same insert, and degrade to an
  unbound session when Practice is absent, disabled, or fails.
- Seeded the retained sample's Shell method concept with
  `shell.method_vertical_axis` and added a guard that resolves the stored id in the bundled
  Knowledge Package. The importer accepts both the older `concept-shells` fixture id and
  the retained sample's generated Core id; runtime resolution uses only the stored crosswalk.
- Tests cover migration behavior; start with zero, one, and multiple families; the
  no-immediate-repeat property across 10,000 seeds; open/solved/unknown describe behavior;
  mapped/unmapped/disabled/failing session orchestration; and the sample crosswalk guard.
  [PR #8](https://github.com/Reid-Engineered/axiom/pull/8) is the source of truth for the
  passing `cargo check`, `cargo test`, `cargo clippy`, formatting, frontend matrix, and E2E
  gates on Linux, macOS, and Windows where applicable.
- Deliberately left out every design exclusion: `StudySessionPage` and frontend hooks, the
  next-problem flow, the permanent regression corpus, and the offline acceptance test. No
  file under `src/` or `src-tauri/src/generation/` changed, and
  `practice.generate`/`practice.evaluate`/`practice.hint` remain unchanged.

## Review

## Follow-ups
