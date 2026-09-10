---
id: 075
title: Workspace curriculum provisioning via practice.concepts@1
status: proposed
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

## What was built / tested / left out

Not started.

## Review

## Follow-ups
