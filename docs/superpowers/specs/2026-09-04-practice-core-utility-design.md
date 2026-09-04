# Practice Core Utility — design

## 1. Scope

Roadmap Stage 8, "Practice Core Utility" sub-project (`ROADMAP.md` "Remaining Stage 8
scope"), following directly on the canonical Problem schema
(`.ai/tasks/_archive/054-canonical-problem-schema.md`), `math.verify`
(`.ai/tasks/_archive/055-math-verify.md`), problem generation
(`.ai/tasks/_archive/056-problem-generation.md`), and the module-capability runtime
(`.ai/tasks/_archive/045-048`), all merged.

Add a first-party Practice module providing three capabilities —
`practice.generate@1`, `practice.evaluate@1`, `practice.hint@1` — that assemble the pieces
built by those four prior sub-projects into a learner-facing generate → attempt → evaluate
flow, invoked through the module-capability runtime with no Practice-specific logic in
Core.

**Does not build:** any Tauri command or frontend service wiring; Study Session UI (a
later, separate sub-project — "minimal Study Session UI integration," presentation only);
adaptive family/difficulty selection policy (the caller picks a `family_id`; Practice
doesn't decide "what problem serves this objective"); partial credit, error
classification, or adaptive hint selection beyond fixed authored order (Tutor's future
concern, per the domain-boundary doc's "Practice owns hint selection, reveal timing, and
state" — this sub-project implements the simplest policy that sentence allows, not the
richest one); Core's general storage abstraction (CORE.md's own scope note) — this
sub-project adds two Practice-specific tables, not a generalized mechanism other modules
would reuse.

## 2. Decisions carried in from brainstorming

- **Real capabilities on the runtime, not a plain Rust API.** Practice registers as a real
  module (`module.toml`, `CapabilityProvider`) providing all three capabilities, matching
  CORE.md §6 ("first-party modules are third-party modules"). This is deliberately the
  second real consumer of the module-capability runtime (`math.verify` was the first),
  proving the runtime generalizes rather than having been shaped around its one existing
  user.
- **Minimal SQLite persistence now, not deferred.** An attempt's state (which problem, how
  many hints revealed, submission history) is real, persisted data from this sub-project
  on — not in-memory-only scaffolding that Study Session UI integration would have to
  redesign around. Two new tables, following Stage 7's migration pattern.
- **Sequential hint reveal only.** `practice.hint` reveals the next hint in the family's
  authored order (`Hint.level`), tracking a count. No adaptive selection.
- **Caller picks the family explicitly.** `practice.generate`'s input is `family_id`, not
  a concept or objective id — Practice does not implement "which family fits this learner
  right now" selection policy. That's future adaptive-policy work, layered on top of this
  contract, not inside it.
- **Multiple submissions per instance, until correct.** `practice.evaluate` may be called
  repeatedly against the same `attempt_id`; the attempt stays `open` until a correct
  response arrives or the caller starts a new attempt via `practice.generate`.
- **Persisted attempts store the full generated `ProblemInstance`, not just `family_id` +
  `seed`.** Regeneration from a seed is deterministic in principle, but the stored row is
  the actual source of truth a read uses — no re-invocation of the generation engine on
  every read, and no correctness dependency on knowledge-package content staying byte-
  identical between generate and a later read.
- **Practice calls `math.verify` through the module registry, not directly.** The one new
  mechanism this sub-project adds to the runtime itself: a capability provider invoking
  another capability the way any external caller would (resolve, then invoke), rather than
  reaching into `math_verify`'s Rust internals because both happen to live in the same
  binary today. See §4.

## 3. Location and registration

New top-level module `src-tauri/src/practice/`, sibling to `knowledge/`, `capabilities/`,
`generation/`, `modules/`, `db/`. Follows the file-split convention `math_verify`
established: `types.rs` (request/response shapes), `error.rs` (`PracticeError`),
`provider.rs` (the `CapabilityProvider` impl), `store.rs` (persistence queries against
`practice_attempts`/`practice_submissions`), `mod.rs` (public exports), inline
`#[cfg(test)] mod tests` per file plus a `tests/` directory for the full-registry
round-trip test.

Ships one first-party manifest, embedded via `include_str!`, matching the
`math_verify`/`EmbeddedManifestSource` pattern:

```toml
manifest_version = 1
id = "org.axiom.practice"
name = "Axiom Practice"
version = "0.1.0"
minimum_axiom_version = "0.1.0"
offline = "full"

[[provides]]
id = "practice.generate"
version = 1

[[provides]]
id = "practice.evaluate"
version = 1

[[provides]]
id = "practice.hint"
version = 1

[[requires]]
id = "math.verify"
min_version = 1
```

This is the first manifest in the codebase with a non-empty `[[requires]]` — sub-project 1
defined the field and sub-project 1's conformance suite (task 048) already exercises
resolution against it with fixture manifests, but no real module has depended on another
real module's capability until now.

## 4. The inter-capability call

`practice.evaluate` needs `math.verify`'s correctness verdict. Three ways to get it, in
order of preference:

**Chosen: Practice resolves and invokes `math.verify` through the same `ModuleRegistry`
path any external caller would use.** `ModuleRegistry` is constructed and wrapped in
`Arc<RwLock<ModuleRegistry>>` at app wiring time (`src-tauri/src/lib.rs` or equivalent
setup code, wherever Tauri managed state is assembled). Registration order: register
`math_verify` first, then construct `PracticeProvider` holding a clone of that `Arc` plus
the `ModuleInstallation` context it needs for resolution, then register `practice` into the
same registry. Inside the `practice.evaluate` handler, Practice calls
`registry.resolve(&installation, &CapabilityRequirement { id: "math.verify", min_version:
1 })` to get a `CapabilityHandle`, then `registry.invoke(&handle, &installation, call)` —
identical to how Core or a future frontend-facing orchestration layer would call
`math.verify` directly. No hardcoded reference to `MathVerifyProvider`; if a workspace ever
had a different `math.verify`-providing module enabled ahead of the first-party one in its
`enabled_module_ids`, Practice would transparently call that one instead.

**Rejected: call `math_verify`'s internal Rust function directly.** Same binary, so
mechanically trivial, but this is exactly the "we control both sides today" shortcut
CORE.md §6 warns against: a real third-party Practice-alternative module built against this
same contract could never do this, so if first-party code takes the shortcut, the contract
is quietly weaker than it claims to be.

**Rejected: push the `math.verify` call out of Practice, into whatever orchestrates a
session.** Would invert the domain-boundary doc's own diagram — Practice sits above
`math.verify`, calling *into* it to answer "what do we give this learner now?" — and would
leak correctness-computation responsibility into a layer meant to stay UI-facing.

This makes `Arc<RwLock<ModuleRegistry>>` construction order load-bearing for the first
time: `math_verify` must be registered before the `Arc` is cloned into `PracticeProvider`.
Registration itself (`&mut self`) still requires the write lock; `resolve`/`invoke` calls
from within `practice.evaluate` take only the read lock, so a workspace's Practice calls
never contend with each other on the registry lock, only (briefly, at startup) with
registration.

## 5. Capability contracts

Transported as `serde_json::Value` through the existing generic `CapabilityProvider::invoke`
— no change to that trait, matching `math_verify`'s precedent.

```rust
// practice.generate@1
#[derive(Deserialize)]
pub struct GenerateRequest {
    pub workspace_id: String,
    pub family_id: ProblemFamilyId,
    /// Test-only determinism hook. Omitted in real use — Practice draws a fresh seed
    /// from the OS RNG (`rand::rngs::OsRng` or equivalent single-use source; distinct
    /// from generation's own deterministic SplitMix64, which seeds *from* this value).
    #[serde(default)]
    pub seed: Option<u64>,
}
#[derive(Serialize)]
pub struct GenerateResponse {
    pub attempt_id: String,
    pub prompt: String,
    pub response_type: ResponseType,
    pub hints_total: u32,
}

// practice.evaluate@1
#[derive(Deserialize)]
pub struct EvaluateRequest {
    pub workspace_id: String,
    pub attempt_id: String,
    pub response: ResponseValue, // mirrors math.verify's VerifyRequest response shapes:
                                  // numeric f64 or a symbolic-expression string, matching
                                  // the stored instance's response_type
}
#[derive(Serialize)]
pub struct EvaluateResponse {
    pub correct: bool,
    pub status: AttemptStatus, // "open" | "solved"
    pub submission_count: u32,
}

// practice.hint@1
#[derive(Deserialize)]
pub struct HintRequest {
    pub workspace_id: String,
    pub attempt_id: String,
}
#[derive(Serialize)]
pub struct HintResponse {
    pub hint_text: String,
    pub hints_revealed: u32,
    pub hints_total: u32,
}
```

`canonical_solution` and any not-yet-revealed hint text never appear in a response —
`GenerateResponse` carries only what a learner should see before attempting the problem,
and `HintResponse` carries only the one hint just revealed, not the full list. Everything
sensitive lives solely in the persisted `practice_attempts` row (§6), read only by
`practice.evaluate`/`practice.hint`'s own handlers.

`attempt_id` scoping: every handler's first step is loading the attempt row filtered by
*both* `id` and `workspace_id` — an `attempt_id` from workspace A is simply not found
(`PracticeError::AttemptNotFound`, not a distinguishable "wrong workspace" error) when
looked up under workspace B. No cross-workspace leakage, no information disclosure about
whether the id exists elsewhere.

## 6. Persistence

Two new tables, added via a migration in `src-tauri/src/db/migrations/`, following Stage
7's existing migration numbering/pattern:

```sql
CREATE TABLE practice_attempts (
    id              TEXT PRIMARY KEY,
    workspace_id    TEXT NOT NULL REFERENCES workspaces(id),
    family_id       TEXT NOT NULL,
    seed            INTEGER NOT NULL,
    instance_json   TEXT NOT NULL,   -- serialized ProblemInstance, incl. canonical_solution
    hints_revealed  INTEGER NOT NULL DEFAULT 0,
    status          TEXT NOT NULL CHECK (status IN ('open', 'solved')),
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL
);

CREATE TABLE practice_submissions (
    id            TEXT PRIMARY KEY,
    attempt_id    TEXT NOT NULL REFERENCES practice_attempts(id),
    response_json TEXT NOT NULL,
    correct       INTEGER NOT NULL,  -- 0/1
    submitted_at  TEXT NOT NULL
);
```

`instance_json` stores the full `ProblemInstance` (prompt, resolved parameters, canonical
solution, hint text) exactly as `generate_problem_instance` produced it — the row is the
source of truth `practice.evaluate`/`practice.hint` read from; the generation engine is
never re-invoked on a read path. `seed` is retained alongside the full instance for
auditability, not as the thing a read reconstructs from.

`store.rs` exposes plain query functions (`insert_attempt`, `load_attempt`,
`increment_hints_revealed`, `record_submission`, `mark_solved`) taking a DB connection —
the same shape Stage 7's `db/` module already uses elsewhere, no new persistence
abstraction introduced.

## 7. Error handling

New `PracticeError` enum (mirrors `KnowledgeError`/`MathVerifyError`'s shape):

```rust
pub enum PracticeError {
    FamilyNotFound { family_id: String },
    AttemptNotFound { attempt_id: String },     // includes wrong-workspace lookups (§5)
    NoMoreHints { attempt_id: String },
    AlreadySolved { attempt_id: String },        // evaluate called again after solved
    GenerationFailed(generation::GenerationError),
    VerificationFailed(String),                  // math.verify invocation itself errored
                                                  // (not "wrong answer" -- that's Ok(false))
    Storage(rusqlite::Error),                    // or whatever Stage 7's db error type is
}
```

Mapped to `InvocationError::InvalidInput` (bad/missing ids, calling `evaluate` on a solved
attempt) vs. `InvocationError::Failed` (generation or storage failure, an internal
`math.verify` invocation error) at the `CapabilityProvider::invoke` boundary — same split
`math_verify` uses between "the request was malformed" and "something genuinely broke."
A parseable-but-wrong response is never an error: `practice.evaluate` returns
`Ok(EvaluateResponse { correct: false, status: "open", .. })`, exactly mirroring
`math.verify`'s own "wrongness is not failure" rule.

## 8. Testing

`cargo test`, per `store.rs`/`provider.rs` split:

- **Generation**: `practice.generate` with an explicit seed produces the same
  `attempt_id`-addressable prompt as calling `generate_problem_instance` directly with that
  seed (proves no drift between Practice's wiring and the engine); an unknown `family_id`
  yields `FamilyNotFound`; the response never contains `canonical_solution` or hint text
  (a structural assertion on the serialized JSON, not just the Rust struct's fields).
- **Evaluation**: a correct response transitions `open` → `solved`; an incorrect response
  stays `open` and increments `submission_count`; calling `evaluate` again after `solved`
  returns `AlreadySolved`; a response is checked against the *stored* instance regardless
  of what the caller claims (can't be tricked by resubmitting a different response shape).
- **Hints**: sequential reveal matches `Hint.level` order; `hints_revealed` persists across
  calls; requesting past `hints_total` returns `NoMoreHints`.
- **Workspace isolation**: an `attempt_id` created under workspace A is `AttemptNotFound`
  under workspace B, for all three capabilities.
- **Inter-capability call**: a fixture `ModuleRegistry` with both `math_verify` and
  `practice` registered, invoked through `practice.evaluate` end-to-end — proving §4's
  resolve-then-invoke path actually reaches real `math.verify` code, not a stub.
- **Persistence round-trip**: attempt survives a fresh connection to the same on-disk
  database (matching Stage 7's restart-persistence precedent, task 042).

## 9. Follow-ups (out of scope here, tracked for later)

- Tauri command wiring and the frontend service that calls it (`src/services/*`) — next
  sub-project, per `ARCHITECTURE.md` §5's swap-in pattern.
- Study Session UI integration (presentation only, per `ROADMAP.md`).
- Adaptive family/difficulty selection policy (which family serves this learner right now)
  — layered on top of `practice.generate`'s explicit `family_id` input, not inside it.
- Adaptive or diagnostic hint selection beyond fixed authored order — Tutor's eventual
  concern.
- The network-disabled offline acceptance test spanning the whole Practice path
  end-to-end — the last item in `ROADMAP.md`'s Stage 8 sub-project list, depends on Study
  Session UI existing first.
