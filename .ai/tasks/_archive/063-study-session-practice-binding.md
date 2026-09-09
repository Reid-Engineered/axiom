---
id: 063
title: Study Session <-> Practice attempt binding
status: done
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
- 2026-09-09 — Reviewed by claude against `.ai/review-checklist.md`; verdict `approve` with
  six non-blocking observations recorded below. `review` → `done` and archived as part of
  the merge of PR #8, following the bookkeeping pattern of task 061.

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

Reviewer: claude
Date: 2026-09-09

Claims were re-verified independently rather than taken from the PR body: the branch was
checked out into a separate worktree and `cargo test` run locally (297 passed, 0 failed —
consistent with master's 284 plus this PR's 13); `gh pr checks 8` confirmed all seven
required checks green on run `34309641851`, including `backend-checks (windows-latest)` and
`e2e`; and the diff was read against
`docs/superpowers/specs/2026-09-08-study-session-practice-binding-design.md` section by
section.

- [x] **Correctness — pass.** The code does what the task claims, and the tests cover the
      stated behavior rather than only the happy path.
      - The migration test genuinely reconstructs a v2 database, inserts a concept and a
        session, *then* migrates, and asserts nullability through `pragma_table_info`
        (`db/tests.rs`) — it proves the "existing rows survive" claim rather than asserting
        it on a fresh schema.
      - All four orchestration branches from design §5/§6 are covered by distinct tests:
        mapped + enabled, unmapped concept, module disabled (by removing
        `org.axiom.practice` from `enabled_module_ids`), and a genuine invoke failure via a
        purpose-built `FailingStartProvider`. Each asserts `start_session` still succeeds.
      - `describe`'s privacy claim is actually tested, not asserted: the response is
        serialized and checked for the absence of `canonical_solution` and `hints`.
      - Generation happens before the session `INSERT`, and `current_attempt_id` lands in
        that same statement, as design §5 requires.
      - `most_recent_candidate_family` correctly scopes by workspace and candidate list, and
        breaks `created_at` ties on `rowid` — its test covers the wrong-workspace and
        empty-candidate-list cases.
- [x] **Architecture conformance — pass.** Both columns are nullable with no cross-schema
      foreign key, per design §3. Both capabilities are additive: `practice.generate`,
      `practice.evaluate`, `practice.hint`, and everything under
      `src-tauri/src/generation/` are untouched, and `start` enters generation through the
      existing `generate` path rather than duplicating it. Core stores `current_attempt_id`
      without interpreting it. §5's frontend rules do not apply — no file under `src/`
      changed.
- [x] **UI rules — N/A.** No file under `src/` changed; confirmed against the PR's changed-
      file list.
- [x] **Process — pass.** All seven required checks green; worklog is detailed enough to
      follow the change without the diff; scope matches the design's stated exclusions, with
      the one deviation (seeding by concept name as well as id) disclosed in the task file
      rather than made silently.

### Observations (none blocking)

1. **`describe` re-derives `response_type` instead of carrying it.** `provider.rs`'s
   `describe` maps `ResolvedSolution::Symbolic`/`Numeric` onto
   `ResponseType::SymbolicExpression`/`Numeric`, while `generate` returns the family's
   declared `family.response_type`. These agree only because the package loader rejects a
   family whose `canonical_solution` disagrees with its `response_type`
   (`knowledge/error.rs:338` `ResponseTypeSolutionMismatch`, enforced at
   `knowledge/problem_family.rs:330`). That invariant is load-time and non-local; a one-line
   comment at the derivation naming it would keep a future change to either side from
   silently splitting the two values. `ProblemInstance` does not persist `response_type`, so
   deriving it is the only option available without a schema change — this is a comment
   request, not a design objection.
2. **`select_family` panics on an empty eligible set.** If `candidates.len() > 1` and every
   candidate id equalled `excluded_family_id`, `eligible` would be empty and
   `seed % eligible.len()` panics on the modulo before the index ever runs. This is
   unreachable today only because problem-family ids are unique by construction
   (`knowledge/discover.rs:82` derives per-kind uniqueness from filename uniqueness). A
   `debug_assert!(!eligible.is_empty())` or an explicit fallback to `candidates` would make
   the invariant local to the function that depends on it.
3. **A second caller identity for the same layer.** `session.rs` invokes with
   `calling_module_id: "core.session"`, while the existing Tauri-command call site uses
   `"core.tauri_commands"` (`commands/practice.rs:200`). `CORE.md:226` describes the
   envelope as what lets Core "generically log, route, and version-check every call", so two
   ids for the same layer makes that log inconsistent. Either is defensible — `core.session`
   is arguably more precise — but the choice should be one or the other, or documented as a
   deliberate per-command granularity.
4. **`src/types/session.ts` is now behind the wire model.** `commands/models.rs`'s `Session`
   serializes `currentAttemptId` (camelCase via the struct's `rename_all`), but
   `src/types/session.ts:28`'s `Session` does not declare it. Harmless today —
   `skip_serializing_if` omits it when unset, nothing consumes it, and `ARCHITECTURE.md` §4
   scopes `src/types/` to the product model rather than every wire field — but the
   StudySessionPage sub-project needs this field, and right now nothing outside this PR's
   diff records that. Worth naming explicitly in that sub-project's spec so it is an input
   rather than a discovery.
5. **Empty `## Follow-ups` while the design tracks three.** Design §8 defers "next problem"
   mid-session, crosswalk cardinality beyond 1:1, and family selection beyond
   "random, avoid immediate repeat". The task template puts follow-ups in the task file so
   the record stands alone; copying those three across costs nothing and keeps the task file
   readable without the spec.
6. **The seed crosswalk keys partly on a display name.** `seed.rs` maps when
   `concept.id == "concept-shells" || concept.name == "Shell method"`. The name coupling is
   explained in the task file and is guarded — `sample_seed_crosswalk_resolves_to_a_bundled_knowledge_concept`
   queries by that same name and would fail loudly on a rename. Noting it only because that
   failure would present as a broken test rather than as a lost mapping, so whoever hits it
   should know to look here.

Observations 1, 2, 3 and 5 are small enough to fold into this PR if codex wants them;
4 belongs to the next sub-project's spec, and 6 needs no change. None of them block the
merge.

Verdict: approve

## Follow-ups
