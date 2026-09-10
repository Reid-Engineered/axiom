---
id: 075
title: Workspace curriculum provisioning via practice.concepts@1
status: done
owner: codex
stage: 8
depends_on: []
---

## Scope

Implement §6.3 of `docs/superpowers/specs/2026-09-09-mock-and-dead-end-containment-design.md`.

`create_workspace_handler` inserts a workspace, a goal, and offline rows — no concepts. A
learner-created workspace therefore cannot reach Practice at all, and the loop the Windows
walk exercised only worked because it ran inside the imported sample. `064` criterion 1 means
a **learner-created** workspace reaches a real generated problem, so this is on the beta path.

Does **not** build: the general `knowledge.query@1` capability (explicitly out of scope for
this beta), any change to `practice.generate` / `evaluate` / `hint` / `start` / `describe`, or
any frontend change.

## Plan

- `src-tauri/src/practice/types.rs` — `ConceptsRequest { workspace_id }`,
  `ConceptsResponse { concepts: Vec<ConceptDescriptor> }`, and:

  ```rust
  pub struct ConceptDescriptor {
      pub concept_id: String,  // opaque to Core — the knowledge-concept crosswalk value
      pub name: String,
      pub topic: String,       // stored as concepts.chapter  — NOT NULL, so Practice supplies it
      pub summary: String,     // stored as concepts.meaning   — NOT NULL, likewise
  }
  ```

  `topic` and `summary` are in the contract because `concepts.chapter` and `concepts.meaning`
  are `NOT NULL` in `0001_initial.sql`. Core inventing either would be Core authoring subject
  content; having the module supply both keeps Core storing opaque strings.

- `src-tauri/src/practice/provider.rs` — `practice.concepts` handler, registered as an
  additive sixth capability alongside the existing five.
- `src-tauri/src/commands/workspace.rs` — `create_workspace_handler` resolves and invokes the
  capability through `ModuleRegistry` with `calling_module_id: "core.workspace"` (matching
  `core.session`'s per-command granularity), and inserts the returned concepts with
  `knowledge_concept_id` set. Core supplies only its own domain defaults: a generated Core
  concept id, `mastery_state = 'New'`, `on_exam = 0`, no diagnostics, no notes.
- `CORE.md` — document this boundary in the capability section.
- `ARCHITECTURE.md` — record that `concepts` rows may originate from a capability response as
  well as from the learner or the sample import.
- `src-tauri/src/practice/tests/mod.rs`, `src-tauri/src/commands/tests.rs` — tests below.

**The approved boundary.** Core may store these strings and the knowledge-concept crosswalk.
Core must **not** interpret subject knowledge and must **not** read `knowledge-package/`
directly — resolution goes through `ModuleRegistry`, as `start_session` already does.

**Degradation.** When the capability is absent, the module is disabled, or the invocation
fails, workspace creation still succeeds with zero concepts — the philosophy `063` established
for `start_session`, for the same reason: workspace creation is Core's own primary function
and must not fail because Practice is unavailable.

**Tests.**

- `practice.concepts` returns the bundled package's three concepts.
- Workspace creation inserts them with `knowledge_concept_id` set and `mastery_state = 'New'`.
- Capability absent → workspace created, zero concepts, no error.
- Module disabled → same.
- Forced invoke failure → same.
- End-to-end: a created workspace's Shell method concept resolves a family through
  `practice.start` and binds a real attempt.

## Worklog

- 2026-09-09 — Filed by claude from the approved design, `proposed` for codex. The capability
  shape and the Core boundary are approved as written above; claude locks `CORE.md` and
  `ARCHITECTURE.md` as contract owner before this merges.
- 2026-09-09 — Claimed by codex. Implementing the locked `practice.concepts@1` contract and
  workspace-provisioning degradation behavior test-first on `agent/codex/075-workspace-curriculum-provisioning`.
- 2026-09-09 — Implementation complete. Gates run by claude on a Linux (WSL) toolchain; the
  Windows MSVC linker is unavailable here and the GNU toolchain cannot link the Tauri
  cdylib, so no Windows result was treated as a pass.
- 2026-09-09 — Contract lock: claude reviewed the `CORE.md` and `ARCHITECTURE.md` additions
  as contract owner and accepted them as written. They state the approved boundary
  accurately — Core stores opaque strings and the crosswalk, resolves through
  `ModuleRegistry` as `core.workspace`, never reads `knowledge-package/`, and degrades to a
  zero-concept workspace. No edit was needed.
- 2026-09-09 — All applicable gates green; moved to `review`. claude made no code changes to
  this branch. CI on the PR is the source of truth for the full required set.

## What was built / tested / left out

- Added `practice.concepts@1` as an additive sixth Practice capability: `ConceptsRequest`,
  `ConceptsResponse` and `ConceptDescriptor { concept_id, name, topic, summary }` in
  `practice/types.rs`, a handler in `practice/provider.rs`, and a `[[provides]]` entry in
  `practice/module.toml`. The existing five capabilities are unchanged.
- `create_workspace_handler` became async and resolves the capability through
  `ModuleRegistry` with `calling_module_id: "core.workspace"`, inserting the returned
  concepts with `knowledge_concept_id` set and Core's own defaults (`mastery_state = 'New'`,
  `on_exam = 0`, no learner history).
- `topic` and `summary` are carried in the contract because `concepts.chapter` and
  `concepts.meaning` are `NOT NULL`; Core inventing either would be Core authoring subject
  content, which the approved boundary forbids.
- Workspace creation is split into two transactions so that a Practice failure cannot fail
  workspace creation. Absent, disabled and failing capability all yield a usable
  zero-concept workspace.
- `CORE.md` and `ARCHITECTURE.md` document the boundary, locked by claude as contract owner.

**Tested** (run by claude on Linux; exact commands and results in the Review section): six
new Rust tests covering the capability's response against the bundled package, provisioning
with opaque crosswalks, all three degradation branches, and an end-to-end assertion that a
created workspace's Shell method concept starts a real attempt. `cargo test` 311 passed / 0
failed; `cargo clippy --all-targets -- -D warnings` clean; `cargo fmt --check` clean.
Frontend gates are not applicable — this task changes no file under `src/`.

**Left out as scoped:** `knowledge.query@1` as a general capability (explicitly out of scope
for this beta), any frontend change, and every other task on the beta path.

## Review

Reviewer: claude
Date: 2026-09-10

Reviewed as contract owner as well as gate reviewer, since this task adds capability surface.
Gates re-run on a Linux toolchain: `cargo test` 311 passed / 0 failed, clippy and fmt clean.
claude made no code change to this branch.

- [x] **Correctness — pass.** All three degradation branches are covered by distinct tests —
      capability absent, module disabled, invocation failing — and each asserts workspace
      creation still succeeds with zero concepts. The end-to-end test earns its place: a
      created workspace's Shell method concept resolves a family and binds a real attempt,
      which is the actual claim `064` criterion 1 makes and the reason this task is on the
      beta path at all.
- [x] **Architecture conformance — pass, and the boundary holds.** Core resolves through
      `ModuleRegistry` with `core.workspace` as the caller, matching `core.session`'s
      per-command granularity from `063`. Nothing in `commands/` reads `knowledge-package/`;
      package access stays inside the Practice provider. Core writes only its own record
      defaults (`mastery_state = 'New'`, `on_exam = 0`, generated id) and stores the module's
      strings without interpreting them. The capability is genuinely additive — the existing
      five and the generation engine are untouched.
- [x] **UI rules — N/A.** No file under `src/` changed.
- [x] **Process — pass.** All seven checks green. `CORE.md` and `ARCHITECTURE.md` are updated
      in the same change rather than left to drift, and `knowledge.query@1` was correctly left
      unbuilt.

### Contract lock

`CORE.md` and `ARCHITECTURE.md` are accepted as written. They state the approved boundary
accurately, including the part most likely to be misread later: `topic` and `summary` are in
the contract because `concepts.chapter` and `concepts.meaning` are `NOT NULL`, so Core
inventing either would be Core authoring subject content. That reasoning is now recorded where
the next person will look for it.

### Observations (none blocking)

1. **`concepts()` discards the request's `workspace_id`** via `let _workspace_id = ...`. Harmless
   — the envelope already carries it and the bundled curriculum is workspace-independent — but
   a one-line comment saying so would stop a future reader treating the discard as an oversight.
2. **A concept with no `topic` would store an empty `chapter`.** `topic.clone().unwrap_or_default()`
   satisfies the `NOT NULL` constraint with `""`, which would render as an empty chapter heading
   in `ConceptsListPage`. Unreachable today — all three bundled concepts declare a topic — but
   the failure mode would be silent rather than loud.
3. **Concept insertion is not covered by the same transaction as the workspace.** Deliberate and
   correct, since a Practice failure must not fail workspace creation. Worth noting that a
   constraint violation *inside* the concept loop would still propagate and fail the call, so
   the degradation contract covers capability failure, not insert failure.

Verdict: approve

## Follow-ups
