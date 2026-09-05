# Practice Tauri Command + Frontend Service Wiring Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make `practice.generate`/`evaluate`/`hint` — real capabilities on the module-capability runtime since task 057, but only ever invoked from test fixtures — callable from the real app: real startup construction of the registry against the bundled `knowledge-package/`, a camelCase `#[tauri::command]` layer, and matching `src/services/practiceService.ts` + `mockBackend.ts` wiring.

**Architecture:** A new `build_practice_registry` helper (testable independent of Tauri's app context) builds the `ModuleRegistry` + `ModuleInstallation`; `lib.rs`'s `setup` closure calls it with the real bundled knowledge package. A new `commands/practice.rs` translates between that registry and camelCase wire types, the same job every other `commands/*.rs` file already does between SQL rows and wire types. The frontend gets the usual `types.ts` + `service.ts` + `mockBackend.ts` triad every other domain already has.

**Tech Stack:** Rust (`tauri` 2.11.5, `async-trait`, `rusqlite`), TypeScript/React (Vitest, `@tauri-apps/api`). No new dependencies.

**Spec:** [docs/superpowers/specs/2026-09-04-practice-tauri-command-wiring-design.md](../specs/2026-09-04-practice-tauri-command-wiring-design.md)

## Global Constraints

- **Base this work on latest `master`, not this doc-only branch.** As of this plan, `master` has task 057's `src-tauri/src/practice/` merged; this plan's worktree/branch does not. Start implementation from a checkout of current `master`.
- `PracticeStore` (`src-tauri/src/practice/store.rs`) is currently a private module with no public re-export (`practice/mod.rs` only exports `PracticeError`, `PracticeProvider`, the `types::*` names, and `MANIFEST_TOML`). Task 1 adds `pub use store::PracticeStore;` — a small, justified addition to already-merged code, needed because this is the first real caller outside `practice/` that constructs a store directly (the same category of gap task 057 itself hit and fixed with `math_verify`'s missing serde derives).
- Fixed, global `ModuleInstallation` (both first-party modules always enabled, `workspace_id` unused/empty) per the spec §2/§3 — not per-workspace enablement. Don't wire `workspace_modules` into capability resolution in this task.
- Command-facing types are camelCase (`#[serde(rename_all = "camelCase")]`), matching `commands/models.rs`'s existing convention — never rename `practice::types`' own snake_case capability contract to match.
- `GenerateRequest.seed` is always `None` from any command — no command surface exposes a seed parameter.
- No async `#[tauri::command]` exists anywhere in this codebase yet (`grep -rn "async fn" src-tauri/src/commands/*.rs` returns nothing) — this plan's Task 4/5 are the first. If `State<'_, T>` extraction behaves unexpectedly in an async command when you reach that step, that's a real finding to note in the task file's Worklog, not a silent workaround — Tauri v2's documented pattern is exactly what Task 4 specifies, but this codebase has never exercised it.
- Gates: this task touches both `src-tauri/` and `src/`, so both gate sets from `.ai/quality-gates.md` apply — `cargo check`/`test`/`clippy`/`fmt` **and** `npm run typecheck`/`lint`/`build`/`test`.
- Task file: `.ai/tasks/058-practice-tauri-wiring.md` (`stage: 8`, `depends_on: [57]`).

---

### Task 1: Create the task file, export `PracticeStore`

**Files:**
- Create: `.ai/tasks/058-practice-tauri-wiring.md`
- Modify: `src-tauri/src/practice/mod.rs`

**Interfaces:**
- Produces: `crate::practice::PracticeStore` becomes a public path (constructible from `commands/practice.rs` in Task 3).

- [ ] **Step 1: Write the task file**

```markdown
---
id: 058
title: Practice Tauri command + frontend service wiring
status: in-progress
owner: claude-code
stage: 8
depends_on: [57]
---

## Scope

Wire `practice.generate@1`/`practice.evaluate@1`/`practice.hint@1` — real capabilities on
the module-capability runtime since task 057, invoked so far only from test fixtures —
through to something the frontend can call: real app-startup construction of the
`ModuleRegistry` against the bundled `knowledge-package/`, `#[tauri::command]` handlers
translating to/from `practice::types`' snake_case contract, and matching
`src/services/practiceService.ts` + `src/test/mockBackend.ts` wiring. Does not build: Study
Session UI, per-workspace module enable/disable wired into capability resolution — see
`docs/superpowers/specs/2026-09-04-practice-tauri-command-wiring-design.md` §1/§8.

## Plan

- `src-tauri/src/practice/mod.rs` (export `PracticeStore`)
- `src-tauri/tauri.conf.json` (bundle `knowledge-package/` as a resource)
- `src-tauri/src/commands/practice.rs` (new), `src-tauri/src/commands/mod.rs`,
  `src-tauri/src/lib.rs`
- `src/types/practice.ts` (new), `src/types/index.ts`
- `src/services/practiceService.ts` (new), `src/services/practiceService.test.ts` (new)
- `src/test/mockBackend.ts`

See `docs/superpowers/plans/2026-09-04-practice-tauri-wiring.md` for the task-by-task plan.

## Worklog

- 2026-09-04 — started, claimed by claude-code

## What was built / tested / left out

(filled in at the final task)

## Review

(filled in by reviewer)

## Follow-ups

(filled in if anything is noticed during implementation/review)
```

- [ ] **Step 2: Export `PracticeStore`**

Modify `src-tauri/src/practice/mod.rs`:

```rust
mod error;
mod provider;
mod store;
mod types;

pub use error::PracticeError;
pub use provider::PracticeProvider;
pub use store::PracticeStore;
pub use types::{
    AttemptStatus, EvaluateRequest, EvaluateResponse, GenerateRequest, GenerateResponse,
    HintRequest, HintResponse, ResponseValue,
};

/// The embedded first-party manifest for this module.
pub const MANIFEST_TOML: &str = include_str!("module.toml");

#[cfg(test)]
mod tests;
```

- [ ] **Step 3: Confirm the crate still builds**

Run: `cd src-tauri && cargo check`
Expected: succeeds (a pure visibility widening — nothing depends on `PracticeStore` staying
private).

- [ ] **Step 4: Commit**

```bash
git add .ai/tasks/058-practice-tauri-wiring.md src-tauri/src/practice/mod.rs
git commit -m "task(058): open Practice Tauri wiring; export PracticeStore"
```

---

### Task 2: `build_practice_registry` — a testable startup-wiring helper

**Files:**
- Create: `src-tauri/src/commands/practice.rs`
- Modify: `src-tauri/src/commands/mod.rs`

**Interfaces:**
- Consumes: `crate::knowledge::KnowledgePackage`, `rusqlite::Connection`,
  `crate::practice::{PracticeStore, PracticeProvider, MANIFEST_TOML}`,
  `crate::capabilities::math_verify::{MANIFEST_TOML as MATH_VERIFY_MANIFEST_TOML, MathVerifyProvider}`,
  `crate::modules::{parse, ModuleRegistry, ModuleId, ModuleInstallation}`.
- Produces: `pub fn build_practice_registry(knowledge_package: KnowledgePackage, connection: rusqlite::Connection) -> (Arc<tauri::async_runtime::RwLock<ModuleRegistry>>, ModuleInstallation)`
  — used by Task 3's command handlers (constructing test registries) and Task 5's `lib.rs`
  startup wiring (constructing the real one).

`lib.rs`'s `setup` closure — where this eventually gets called for real — can't run a
`rusqlite::Connection`/`KnowledgePackage` through a normal `#[test]`, since it needs a real
`tauri::App` context to resolve paths. Factoring the actual registry-building logic out into
this plain, dependency-injected function is what makes it testable at all: this task tests
the mechanism directly; Task 5 only has to prove the *wiring* (that `lib.rs` calls this
function with the right real inputs), not the mechanism itself again.

- [ ] **Step 1: Write the failing test**

Create `src-tauri/src/commands/practice.rs` with just enough to compile the test (the real
function body is Step 3):

```rust
use std::sync::Arc;

use tauri::async_runtime::RwLock;

use crate::capabilities::math_verify::MathVerifyProvider;
use crate::knowledge::KnowledgePackage;
use crate::modules::{parse, ModuleId, ModuleInstallation, ModuleRegistry};
use crate::practice::PracticeProvider;

pub fn build_practice_registry(
    knowledge_package: KnowledgePackage,
    connection: rusqlite::Connection,
) -> (Arc<RwLock<ModuleRegistry>>, ModuleInstallation) {
    todo!()
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::modules::{
        CallEnvelope, CapabilityCall, CapabilityId, CapabilityRequirement,
    };

    use super::*;

    fn fixture_package() -> KnowledgePackage {
        let fixture_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src/knowledge/tests/fixtures/canonical");
        crate::knowledge::load_knowledge_package(&fixture_root).unwrap()
    }

    fn seeded_connection() -> rusqlite::Connection {
        let mut connection = crate::db::open_in_memory().unwrap();
        let transaction = connection.transaction().unwrap();
        transaction
            .execute(
                "INSERT INTO workspaces (id, name, guiding_goal_id, progress, paused)
                 VALUES ('ws-1', 'Test', 'goal-1', 0.0, 0)",
                [],
            )
            .unwrap();
        transaction
            .execute(
                "INSERT INTO goals (id, workspace_id, text, state, created_at, updated_at)
                 VALUES ('goal-1', 'ws-1', 'Test goal', 'Guiding', ?1, ?1)",
                ["2026-09-04T12:00:00Z"],
            )
            .unwrap();
        transaction.commit().unwrap();
        connection
    }

    #[test]
    fn build_practice_registry_registers_both_first_party_modules() {
        let (registry, installation) = build_practice_registry(fixture_package(), seeded_connection());

        let math_verify_handle = tauri::async_runtime::block_on(registry.read()).resolve(
            &installation,
            &CapabilityRequirement {
                id: CapabilityId::new("math.verify").unwrap(),
                min_version: 1,
            },
        );
        assert!(math_verify_handle.is_ok());

        let practice_handle = tauri::async_runtime::block_on(registry.read()).resolve(
            &installation,
            &CapabilityRequirement {
                id: CapabilityId::new("practice.generate").unwrap(),
                min_version: 1,
            },
        );
        assert!(practice_handle.is_ok());
    }

    #[test]
    fn build_practice_registry_can_actually_generate_an_attempt() {
        let (registry, installation) = build_practice_registry(fixture_package(), seeded_connection());
        let handle = tauri::async_runtime::block_on(registry.read())
            .resolve(
                &installation,
                &CapabilityRequirement {
                    id: CapabilityId::new("practice.generate").unwrap(),
                    min_version: 1,
                },
            )
            .unwrap();

        let call = CapabilityCall {
            envelope: CallEnvelope {
                workspace_id: "ws-1".to_owned(),
                capability_id: CapabilityId::new("practice.generate").unwrap(),
                version: 1,
                calling_module_id: ModuleId::new("core.test_caller").unwrap(),
            },
            input: serde_json::json!({
                "workspace_id": "ws-1",
                "family_id": "problem.shell_y_poly",
                "seed": 42,
            }),
        };

        let output: serde_json::Value = tauri::async_runtime::block_on(async {
            let registry = registry.read().await;
            registry.invoke(&handle, &installation, call).await
        })
        .unwrap();

        assert!(output["attempt_id"].is_string());
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd src-tauri && cargo test commands::practice::tests`
Expected: FAIL — `todo!()` panics ("not yet implemented"), or a compile error if `mod
practice;` isn't declared in `commands/mod.rs` yet. Declare it now:

Modify `src-tauri/src/commands/mod.rs`, adding `pub mod practice;` alongside the other
`pub mod` lines (after `pub mod note;`, before `pub mod seed;`, keeping the existing
alphabetical ordering).

- [ ] **Step 3: Implement `build_practice_registry`**

Replace the `todo!()` body in `src-tauri/src/commands/practice.rs`:

```rust
pub fn build_practice_registry(
    knowledge_package: KnowledgePackage,
    connection: rusqlite::Connection,
) -> (Arc<RwLock<ModuleRegistry>>, ModuleInstallation) {
    let mut registry = ModuleRegistry::new();
    registry
        .register(
            parse(crate::capabilities::math_verify::MANIFEST_TOML)
                .expect("math_verify manifest must parse"),
            Box::new(MathVerifyProvider),
        )
        .expect("math_verify must register");
    let registry = Arc::new(RwLock::new(registry));

    let installation = ModuleInstallation {
        workspace_id: String::new(),
        enabled_module_ids: vec![
            ModuleId::new("core.math_verify").expect("static id is valid"),
            ModuleId::new("org.axiom.practice").expect("static id is valid"),
        ],
    };

    let store = crate::practice::PracticeStore::new(connection);
    let provider = PracticeProvider::new(
        store,
        knowledge_package,
        Arc::clone(&registry),
        installation.clone(),
    );
    registry
        .blocking_write()
        .register(
            parse(crate::practice::MANIFEST_TOML).expect("practice manifest must parse"),
            Box::new(provider),
        )
        .expect("practice must register");

    (registry, installation)
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cd src-tauri && cargo test commands::practice::tests`
Expected: PASS (2 tests). If `.blocking_write()`/`.blocking_read()` aren't available on
`tauri::async_runtime::RwLock`, per the Global Constraints note in task 057's own plan,
wrap the offending call in `tauri::async_runtime::block_on(async { registry.write().await
... })` instead — the production code path (Task 3's async handlers) already uses
`.read().await`/`.write().await` and is unaffected either way.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/commands/practice.rs src-tauri/src/commands/mod.rs
git commit -m "feat(commands): add build_practice_registry startup-wiring helper"
```

---

### Task 3: Command layer — `generate_attempt`, `evaluate_attempt`, `request_hint`

**Files:**
- Modify: `src-tauri/src/commands/practice.rs`

**Interfaces:**
- Consumes: `build_practice_registry` (Task 2), `crate::practice::{GenerateRequest,
  GenerateResponse, EvaluateRequest, EvaluateResponse, HintRequest, HintResponse,
  ResponseValue, AttemptStatus as PracticeAttemptStatus}`, `crate::modules::{CallEnvelope,
  CapabilityCall, CapabilityId, CapabilityRequirement, ModuleId, ModuleInstallation,
  ModuleRegistry, RegistryError}`, `crate::commands::CommandResult`.
- Produces (used by `lib.rs` in Task 5 and, indirectly, by the frontend's expected JSON
  shape in Task 6): `#[tauri::command] generate_attempt`, `evaluate_attempt`,
  `request_hint`, each `async fn(registry: State<'_, Arc<RwLock<ModuleRegistry>>>,
  installation: State<'_, ModuleInstallation>, input: <...Input>) ->
  CommandResult<<...Output>>`. Wire shapes: `Attempt { attempt_id, prompt, response_type,
  hints_total }`, `EvaluationResult { correct, status, submission_count }`, `Hint {
  hint_text, hints_revealed, hints_total }` — all camelCase on the wire via `#[serde(rename_all
  = "camelCase")]`.

- [ ] **Step 1: Write the failing tests**

Add to `src-tauri/src/commands/practice.rs`, above `#[cfg(test)] mod tests`:

```rust
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::knowledge::ResponseType;
use crate::modules::{
    CallEnvelope, CapabilityCall, CapabilityId, CapabilityRequirement, RegistryError,
};
use crate::practice::{
    AttemptStatus as PracticeAttemptStatus, EvaluateRequest, EvaluateResponse, GenerateRequest,
    GenerateResponse, HintRequest, HintResponse, ResponseValue,
};

use super::CommandResult;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateAttemptInput {
    pub workspace_id: String,
    pub family_id: String,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Attempt {
    pub attempt_id: String,
    pub prompt: String,
    pub response_type: ResponseType,
    pub hints_total: u32,
}

impl From<GenerateResponse> for Attempt {
    fn from(response: GenerateResponse) -> Self {
        Self {
            attempt_id: response.attempt_id,
            prompt: response.prompt,
            response_type: response.response_type,
            hints_total: response.hints_total,
        }
    }
}

#[derive(Deserialize)]
#[serde(tag = "responseType", rename_all = "camelCase")]
pub enum ResponseValueInput {
    SymbolicExpression { value: String },
    Numeric { value: f64 },
}

impl From<ResponseValueInput> for ResponseValue {
    fn from(input: ResponseValueInput) -> Self {
        match input {
            ResponseValueInput::SymbolicExpression { value } => {
                ResponseValue::SymbolicExpression { value }
            }
            ResponseValueInput::Numeric { value } => ResponseValue::Numeric { value },
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvaluateAttemptInput {
    pub workspace_id: String,
    pub attempt_id: String,
    pub response: ResponseValueInput,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AttemptStatus {
    Open,
    Solved,
}

impl From<PracticeAttemptStatus> for AttemptStatus {
    fn from(status: PracticeAttemptStatus) -> Self {
        match status {
            PracticeAttemptStatus::Open => AttemptStatus::Open,
            PracticeAttemptStatus::Solved => AttemptStatus::Solved,
        }
    }
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EvaluationResult {
    pub correct: bool,
    pub status: AttemptStatus,
    pub submission_count: u32,
}

impl From<EvaluateResponse> for EvaluationResult {
    fn from(response: EvaluateResponse) -> Self {
        Self {
            correct: response.correct,
            status: response.status.into(),
            submission_count: response.submission_count,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestHintInput {
    pub workspace_id: String,
    pub attempt_id: String,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Hint {
    pub hint_text: String,
    pub hints_revealed: u32,
    pub hints_total: u32,
}

impl From<HintResponse> for Hint {
    fn from(response: HintResponse) -> Self {
        Self {
            hint_text: response.hint_text,
            hints_revealed: response.hints_revealed,
            hints_total: response.hints_total,
        }
    }
}

async fn invoke_practice<Input, Output>(
    registry: &Arc<RwLock<ModuleRegistry>>,
    installation: &ModuleInstallation,
    capability: &str,
    workspace_id: String,
    input: Input,
) -> Result<Output, RegistryError>
where
    Input: Serialize,
    Output: serde::de::DeserializeOwned,
{
    let requirement = CapabilityRequirement {
        id: CapabilityId::new(capability).expect("static capability id is valid"),
        min_version: 1,
    };
    let handle = {
        let registry = registry.read().await;
        registry.resolve(installation, &requirement)?
    };
    let call = CapabilityCall {
        envelope: CallEnvelope {
            workspace_id,
            capability_id: requirement.id.clone(),
            version: 1,
            calling_module_id: ModuleId::new("core.tauri_commands")
                .expect("static module id is valid"),
        },
        input,
    };
    let registry = registry.read().await;
    registry.invoke(&handle, installation, call).await
}

fn practice_error(error: RegistryError) -> String {
    error.to_string()
}

pub async fn generate_attempt_handler(
    registry: &Arc<RwLock<ModuleRegistry>>,
    installation: &ModuleInstallation,
    input: GenerateAttemptInput,
) -> CommandResult<Attempt> {
    let response: GenerateResponse = invoke_practice(
        registry,
        installation,
        "practice.generate",
        input.workspace_id.clone(),
        GenerateRequest {
            workspace_id: input.workspace_id,
            family_id: input.family_id,
            seed: None,
        },
    )
    .await
    .map_err(practice_error)?;
    Ok(response.into())
}

#[tauri::command(rename_all = "camelCase")]
pub async fn generate_attempt(
    registry: State<'_, Arc<RwLock<ModuleRegistry>>>,
    installation: State<'_, ModuleInstallation>,
    input: GenerateAttemptInput,
) -> CommandResult<Attempt> {
    generate_attempt_handler(&registry, &installation, input).await
}

pub async fn evaluate_attempt_handler(
    registry: &Arc<RwLock<ModuleRegistry>>,
    installation: &ModuleInstallation,
    input: EvaluateAttemptInput,
) -> CommandResult<EvaluationResult> {
    let response: EvaluateResponse = invoke_practice(
        registry,
        installation,
        "practice.evaluate",
        input.workspace_id.clone(),
        EvaluateRequest {
            workspace_id: input.workspace_id,
            attempt_id: input.attempt_id,
            response: input.response.into(),
        },
    )
    .await
    .map_err(practice_error)?;
    Ok(response.into())
}

#[tauri::command(rename_all = "camelCase")]
pub async fn evaluate_attempt(
    registry: State<'_, Arc<RwLock<ModuleRegistry>>>,
    installation: State<'_, ModuleInstallation>,
    input: EvaluateAttemptInput,
) -> CommandResult<EvaluationResult> {
    evaluate_attempt_handler(&registry, &installation, input).await
}

pub async fn request_hint_handler(
    registry: &Arc<RwLock<ModuleRegistry>>,
    installation: &ModuleInstallation,
    input: RequestHintInput,
) -> CommandResult<Hint> {
    let response: HintResponse = invoke_practice(
        registry,
        installation,
        "practice.hint",
        input.workspace_id.clone(),
        HintRequest {
            workspace_id: input.workspace_id,
            attempt_id: input.attempt_id,
        },
    )
    .await
    .map_err(practice_error)?;
    Ok(response.into())
}

#[tauri::command(rename_all = "camelCase")]
pub async fn request_hint(
    registry: State<'_, Arc<RwLock<ModuleRegistry>>>,
    installation: State<'_, ModuleInstallation>,
    input: RequestHintInput,
) -> CommandResult<Hint> {
    request_hint_handler(&registry, &installation, input).await
}
```

Add these tests inside the existing `#[cfg(test)] mod tests` block (reusing `fixture_package`
and `seeded_connection` from Task 2):

```rust
#[test]
fn generate_attempt_translates_response_to_camel_case_shape() {
    let (registry, installation) = build_practice_registry(fixture_package(), seeded_connection());

    let attempt = tauri::async_runtime::block_on(generate_attempt_handler(
        &registry,
        &installation,
        GenerateAttemptInput {
            workspace_id: "ws-1".to_owned(),
            family_id: "problem.shell_y_poly".to_owned(),
        },
    ))
    .unwrap();

    assert!(!attempt.attempt_id.is_empty());
    assert!(!attempt.prompt.is_empty());
    assert!(attempt.hints_total >= 1);

    let value = serde_json::to_value(&attempt).unwrap();
    assert!(value.get("attemptId").is_some(), "expected camelCase attemptId key");
    assert!(value.get("attempt_id").is_none(), "must not leak snake_case keys");
}

#[test]
fn generate_attempt_with_unknown_family_is_an_error() {
    let (registry, installation) = build_practice_registry(fixture_package(), seeded_connection());

    let result = tauri::async_runtime::block_on(generate_attempt_handler(
        &registry,
        &installation,
        GenerateAttemptInput {
            workspace_id: "ws-1".to_owned(),
            family_id: "problem.nonexistent".to_owned(),
        },
    ));

    assert!(result.is_err());
}

#[test]
fn full_generate_evaluate_hint_sequence_round_trips_through_the_command_layer() {
    let (registry, installation) = build_practice_registry(fixture_package(), seeded_connection());

    let attempt = tauri::async_runtime::block_on(generate_attempt_handler(
        &registry,
        &installation,
        GenerateAttemptInput {
            workspace_id: "ws-1".to_owned(),
            family_id: "problem.shell_y_poly".to_owned(),
        },
    ))
    .unwrap();

    let hint = tauri::async_runtime::block_on(request_hint_handler(
        &registry,
        &installation,
        RequestHintInput {
            workspace_id: "ws-1".to_owned(),
            attempt_id: attempt.attempt_id.clone(),
        },
    ))
    .unwrap();
    assert_eq!(hint.hints_revealed, 1);

    let evaluation = tauri::async_runtime::block_on(evaluate_attempt_handler(
        &registry,
        &installation,
        EvaluateAttemptInput {
            workspace_id: "ws-1".to_owned(),
            attempt_id: attempt.attempt_id,
            response: ResponseValueInput::SymbolicExpression {
                value: "0".to_owned(),
            },
        },
    ))
    .unwrap();
    assert_eq!(evaluation.status, AttemptStatus::Open);
    assert_eq!(evaluation.submission_count, 1);

    let evaluation_value = serde_json::to_value(&evaluation).unwrap();
    assert!(evaluation_value.get("submissionCount").is_some());
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd src-tauri && cargo test commands::practice::tests`
Expected: FAIL to compile until the code from Step 1 is fully in place (the block above
already is the implementation — Task 3, like task 057's Task 7, front-loads the real code
because the translation layer only makes sense written whole).

- [ ] **Step 3: Run tests to verify they pass**

Run: `cd src-tauri && cargo test commands::practice::tests`
Expected: PASS (5 tests: the 2 from Task 2 plus 3 new ones).

- [ ] **Step 4: Run the full crate test suite and clippy**

Run: `cd src-tauri && cargo test && cargo clippy --lib -- -D warnings && cargo fmt --check`
Expected: all pass (adjust the clippy package selector per task 057's precedent —
`-p axiom --lib`, not `-p axiom_lib` — if the bare `--lib` form errors here too).

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/commands/practice.rs
git commit -m "feat(commands): add generate_attempt/evaluate_attempt/request_hint"
```

---

### Task 4: Bundle `knowledge-package/`, wire real startup, register commands

**Files:**
- Modify: `src-tauri/tauri.conf.json`
- Modify: `src-tauri/src/lib.rs`

**Interfaces:**
- Consumes: `commands::practice::build_practice_registry` (Task 2), `crate::knowledge::load_knowledge_package`, `crate::db::open`.
- Produces: `Arc<tauri::async_runtime::RwLock<ModuleRegistry>>` and `ModuleInstallation`
  available as Tauri managed state to every command; `generate_attempt`/`evaluate_attempt`/
  `request_hint` reachable via `invoke()` from the frontend.

- [ ] **Step 1: Add the bundle resource**

Modify `src-tauri/tauri.conf.json`'s `"bundle"` block:

```json
"bundle": {
    "active": true,
    "targets": "all",
    "resources": ["../knowledge-package"],
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ]
}
```

- [ ] **Step 2: Wire startup construction and register the commands**

Modify `src-tauri/src/lib.rs`:

```rust
pub mod capabilities;
pub mod commands;
pub mod db;
pub mod generation;
pub mod knowledge;
pub mod modules;
pub mod practice;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&app_data_dir)?;
            let database_path = app_data_dir.join("axiom.sqlite3");
            app.manage(commands::Database::open(&database_path)?);

            let knowledge_package_dir = app.path().resource_dir()?.join("knowledge-package");
            let knowledge_package = knowledge::load_knowledge_package(&knowledge_package_dir)
                .expect(
                    "bundled knowledge-package must load -- a broken bundle is a build problem",
                );
            let practice_connection = db::open(&database_path)?;
            let (registry, installation) = commands::practice::build_practice_registry(
                knowledge_package,
                practice_connection,
            );
            app.manage(registry);
            app.manage(installation);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::workspace::get_workspaces,
            commands::workspace::get_workspace,
            commands::workspace::get_recent_activity,
            commands::workspace::create_workspace,
            commands::workspace::set_workspace_offline_availability,
            commands::goal::get_goal,
            commands::goal::get_goals_by_workspace,
            commands::goal::update_goal,
            commands::goal::revert_goal,
            commands::concept::get_concepts_by_workspace,
            commands::concept::get_concept,
            commands::concept::search_concepts,
            commands::module::get_modules_by_workspace,
            commands::module::get_marketplace_modules,
            commands::module::get_workspace_templates,
            commands::module::get_module,
            commands::module::install_module,
            commands::module::set_module_enabled,
            commands::module::set_module_visibility,
            commands::session::get_active_session_by_workspace,
            commands::session::get_session,
            commands::session::start_session,
            commands::session::pause_session,
            commands::session::resume_session,
            commands::session::add_tutor_exchange,
            commands::session::end_session,
            commands::material::get_material,
            commands::material::search_material,
            commands::note::get_recent_notes,
            commands::seed::import_sample_workspace,
            commands::practice::generate_attempt,
            commands::practice::evaluate_attempt,
            commands::practice::request_hint,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

(`pub mod practice;` already exists in `lib.rs` from task 057 — confirm it's still there
rather than duplicating it; the snippet above shows the full intended file for reference.)

- [ ] **Step 3: Verify it builds**

Run: `cd src-tauri && cargo check`
Expected: succeeds. This step is this task's actual test — `setup`'s closure body can't
run outside a real `tauri::App` context, so there is no `#[test]` for the wiring itself;
Task 2 already covered `build_practice_registry`'s behavior directly, and this step
confirms `lib.rs` calls it with arguments of the right types.

If `app.path().resource_dir()` doesn't resolve to the bundled `knowledge-package/` in a
`cargo tauri dev` run (dev-mode resource resolution can differ from a built bundle),
that's a real finding for the task file's Worklog — note it and, if needed, fall back to
resolving the path relative to `std::env::current_exe()` or a dev-only override, whichever
`cargo tauri dev`'s actual behavior calls for once observed.

- [ ] **Step 4: Run the full backend gate**

Run: `cd src-tauri && cargo check && cargo test && cargo clippy --lib -- -D warnings && cargo fmt --check`
Expected: all pass.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/tauri.conf.json src-tauri/src/lib.rs
git commit -m "feat: bundle knowledge-package, wire practice registry at startup"
```

---

### Task 5: Frontend types — `src/types/practice.ts`

**Files:**
- Create: `src/types/practice.ts`
- Modify: `src/types/index.ts`

**Interfaces:**
- Produces: `ResponseType`, `Attempt`, `ResponseValue`, `AttemptStatus`, `EvaluationResult`,
  `Hint` — TypeScript types consumed by `practiceService.ts` (Task 6) and `mockBackend.ts`
  (Task 7).

- [ ] **Step 1: Write the file**

Create `src/types/practice.ts`:

```typescript
export type ResponseType = 'symbolic-expression' | 'numeric';

export interface Attempt {
  attemptId: string;
  prompt: string;
  responseType: ResponseType;
  hintsTotal: number;
}

export type ResponseValue =
  | { responseType: 'symbolic-expression'; value: string }
  | { responseType: 'numeric'; value: number };

export type AttemptStatus = 'open' | 'solved';

export interface EvaluationResult {
  correct: boolean;
  status: AttemptStatus;
  submissionCount: number;
}

export interface Hint {
  hintText: string;
  hintsRevealed: number;
  hintsTotal: number;
}
```

- [ ] **Step 2: Re-export from the barrel file**

Modify `src/types/index.ts`:

```typescript
export * from './common';
export * from './workspace';
export * from './goal';
export * from './concept';
export * from './module';
export * from './session';
export * from './visualization';
export * from './material';
export * from './note';
export * from './practice';
```

- [ ] **Step 3: Typecheck**

Run: `npm run typecheck`
Expected: succeeds (no consumers yet, so this only checks the new file itself compiles).

- [ ] **Step 4: Commit**

```bash
git add src/types/practice.ts src/types/index.ts
git commit -m "feat(types): add Practice attempt/evaluation/hint types"
```

---

### Task 6: `src/services/practiceService.ts`

**Files:**
- Create: `src/services/practiceService.ts`

**Interfaces:**
- Consumes: `Attempt`, `EvaluationResult`, `Hint`, `ResponseValue` (Task 5).
- Produces: `generateAttempt(workspaceId: string, familyId: string): Promise<Attempt>`,
  `evaluateAttempt(workspaceId: string, attemptId: string, response: ResponseValue):
  Promise<EvaluationResult>`, `requestHint(workspaceId: string, attemptId: string):
  Promise<Hint>` — consumed by `practiceService.test.ts` (Task 8) and, later, Study Session
  UI (out of scope here).

- [ ] **Step 1: Write the file**

Create `src/services/practiceService.ts`:

```typescript
import { invoke } from '@tauri-apps/api/core';

import type { Attempt, EvaluationResult, Hint, ResponseValue } from '../types';

export async function generateAttempt(workspaceId: string, familyId: string): Promise<Attempt> {
  return invoke<Attempt>('generateAttempt', { input: { workspaceId, familyId } });
}

export async function evaluateAttempt(
  workspaceId: string,
  attemptId: string,
  response: ResponseValue,
): Promise<EvaluationResult> {
  return invoke<EvaluationResult>('evaluateAttempt', {
    input: { workspaceId, attemptId, response },
  });
}

export async function requestHint(workspaceId: string, attemptId: string): Promise<Hint> {
  return invoke<Hint>('requestHint', { input: { workspaceId, attemptId } });
}
```

- [ ] **Step 2: Typecheck and lint**

Run: `npm run typecheck && npm run lint`
Expected: both succeed.

- [ ] **Step 3: Commit**

```bash
git add src/services/practiceService.ts
git commit -m "feat(services): add practiceService"
```

---

### Task 7: `mockBackend.ts` wiring

**Files:**
- Modify: `src/test/mockBackend.ts`

**Interfaces:**
- Consumes: `ResponseType`, `AttemptStatus` (Task 5).
- Produces: `handleMockInvoke` handles `'generateAttempt'`, `'evaluateAttempt'`,
  `'requestHint'` — used by `practiceService.test.ts` (Task 8) through the existing
  `beforeEach`/`mockIPC` setup (`src/test/setup.ts`, unchanged).

- [ ] **Step 1: Add the mock attempt state and family fixture**

Modify `src/test/mockBackend.ts`. Add `AttemptStatus` and `ResponseType` to the file's
*existing* `import type { ... } from '../types';` block (do not add a second, separate
import statement from `'../types'` — that would trip the repo's `eslint` import-dedup
rule) — the block becomes:

```typescript
import type {
  AttemptStatus,
  Concept,
  Goal,
  Material,
  MaterialResult,
  Module,
  Note,
  ResponseType,
  Session,
  Workspace,
  WorkspaceActivityEvent,
  WorkspaceTemplate,
} from '../types';
```

Then add, near the other `let`-declared mutable fixtures:

```typescript
interface MockAttempt {
  id: string;
  prompt: string;
  responseType: ResponseType;
  hintTexts: string[];
  hintsRevealed: number;
  status: AttemptStatus;
  submissionCount: number;
}

let mockAttempts: Map<string, MockAttempt>;

const MOCK_PRACTICE_FAMILY = {
  prompt:
    'A region is bounded by y = 4x - x^2 and the x-axis. Find the volume when revolved about the y-axis.',
  responseType: 'numeric' as ResponseType,
  correctValue: 42.7,
  hints: ['Set up the shell method integral.', 'Integrate from x = 0 to x = 4.'],
};
```

Add `mockAttempts = new Map();` inside `resetMockBackend()`, alongside the other
reassignments.

- [ ] **Step 2: Add the three command cases**

Add to the `switch (command)` block in `handleMockInvoke`, before the `default:` arm:

```typescript
case 'generateAttempt': {
  const input = parameters.input as { workspaceId: string; familyId: string };
  findWorkspace(input.workspaceId);
  const id = `attempt-${crypto.randomUUID()}`;
  const attempt: MockAttempt = {
    id,
    prompt: MOCK_PRACTICE_FAMILY.prompt,
    responseType: MOCK_PRACTICE_FAMILY.responseType,
    hintTexts: [...MOCK_PRACTICE_FAMILY.hints],
    hintsRevealed: 0,
    status: 'open',
    submissionCount: 0,
  };
  mockAttempts.set(id, attempt);
  return {
    attemptId: attempt.id,
    prompt: attempt.prompt,
    responseType: attempt.responseType,
    hintsTotal: attempt.hintTexts.length,
  };
}
case 'evaluateAttempt': {
  const input = parameters.input as {
    workspaceId: string;
    attemptId: string;
    response: { responseType: string; value: string | number };
  };
  const attempt = mockAttempts.get(input.attemptId);
  if (!attempt) throw new Error(`Attempt not found: ${input.attemptId}`);
  if (attempt.status === 'solved') throw new Error(`Attempt already solved: ${input.attemptId}`);
  const submitted = Number(input.response.value);
  const correct = Math.abs(submitted - MOCK_PRACTICE_FAMILY.correctValue) <= 1e-6;
  attempt.submissionCount += 1;
  if (correct) attempt.status = 'solved';
  return {
    correct,
    status: attempt.status,
    submissionCount: attempt.submissionCount,
  };
}
case 'requestHint': {
  const input = parameters.input as { workspaceId: string; attemptId: string };
  const attempt = mockAttempts.get(input.attemptId);
  if (!attempt) throw new Error(`Attempt not found: ${input.attemptId}`);
  if (attempt.hintsRevealed >= attempt.hintTexts.length) {
    throw new Error(`No more hints for attempt: ${input.attemptId}`);
  }
  attempt.hintsRevealed += 1;
  return {
    hintText: attempt.hintTexts[attempt.hintsRevealed - 1],
    hintsRevealed: attempt.hintsRevealed,
    hintsTotal: attempt.hintTexts.length,
  };
}
```

- [ ] **Step 3: Typecheck**

Run: `npm run typecheck`
Expected: succeeds.

- [ ] **Step 4: Commit**

```bash
git add src/test/mockBackend.ts
git commit -m "test(mockBackend): add practice attempt/evaluate/hint handling"
```

---

### Task 8: `src/services/practiceService.test.ts`

**Files:**
- Create: `src/services/practiceService.test.ts`

**Interfaces:** None new — exercises Tasks 5-7 together through the mocked IPC boundary
(`src/test/setup.ts`'s existing `beforeEach`/`mockIPC`), matching `moduleService.test.ts`'s
shape.

- [ ] **Step 1: Write the failing tests**

Create `src/services/practiceService.test.ts`:

```typescript
import { describe, expect, it } from 'vitest';

import { evaluateAttempt, generateAttempt, requestHint } from './practiceService';

const workspaceId = 'workspace-calculus-ii';

describe('practiceService', () => {
  it('generates an attempt with the mock family prompt and hint count', async () => {
    const attempt = await generateAttempt(workspaceId, 'problem.shell_y_poly');

    expect(attempt.attemptId).toBeTruthy();
    expect(attempt.prompt).toContain('volume');
    expect(attempt.hintsTotal).toBe(2);
  });

  it('returns status open for an incorrect response', async () => {
    const attempt = await generateAttempt(workspaceId, 'problem.shell_y_poly');

    const result = await evaluateAttempt(workspaceId, attempt.attemptId, {
      responseType: 'numeric',
      value: 0,
    });

    expect(result.correct).toBe(false);
    expect(result.status).toBe('open');
    expect(result.submissionCount).toBe(1);
  });

  it('returns status solved for the correct response', async () => {
    const attempt = await generateAttempt(workspaceId, 'problem.shell_y_poly');

    const result = await evaluateAttempt(workspaceId, attempt.attemptId, {
      responseType: 'numeric',
      value: 42.7,
    });

    expect(result.correct).toBe(true);
    expect(result.status).toBe('solved');
  });

  it('reveals hints in order and throws once exhausted', async () => {
    const attempt = await generateAttempt(workspaceId, 'problem.shell_y_poly');

    const first = await requestHint(workspaceId, attempt.attemptId);
    expect(first.hintsRevealed).toBe(1);
    const second = await requestHint(workspaceId, attempt.attemptId);
    expect(second.hintsRevealed).toBe(2);

    await expect(requestHint(workspaceId, attempt.attemptId)).rejects.toThrow();
  });

  it('throws evaluating or hinting an unknown attempt', async () => {
    await expect(
      evaluateAttempt(workspaceId, 'attempt-unknown', { responseType: 'numeric', value: 1 }),
    ).rejects.toThrow();
    await expect(requestHint(workspaceId, 'attempt-unknown')).rejects.toThrow();
  });
});
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `npm run test -- practiceService`
Expected: FAIL if Task 7's mock cases aren't in place yet (they are, by this point in the
plan) — run this step regardless to confirm the suite is wired up and passing for the right
reason, not accidentally passing with `passWithNoTests`.

- [ ] **Step 3: Run tests to verify they pass**

Run: `npm run test -- practiceService`
Expected: PASS (5 tests).

- [ ] **Step 4: Commit**

```bash
git add src/services/practiceService.test.ts
git commit -m "test(services): add practiceService tests"
```

---

### Task 9: Full gate, close out the task file

**Files:**
- Modify: `.ai/tasks/058-practice-tauri-wiring.md`

**Interfaces:** None new.

- [ ] **Step 1: Run every gate**

Run:
```bash
cd src-tauri && cargo check && cargo test && cargo clippy --lib -- -D warnings && cargo fmt --check
cd .. && npm run typecheck && npm run lint && npm run build && npm run test
```
Expected: all pass. Fix any drift (`cargo fmt`, lint autofix) and re-commit before moving
on. If `GoalEditingSheet.test.tsx` fails and this task touched nothing under
`src/pages/GoalEditingSheet*`, that's the documented flake (`.ai/quality-gates.md`,
confirmed recurring as recently as task 057's own merge) — re-run once and don't block on
it if it goes green.

- [ ] **Step 2: Update the task file**

Modify `.ai/tasks/058-practice-tauri-wiring.md`, replacing the `## What was built / tested
/ left out` section:

```markdown
## What was built / tested / left out

Built: `build_practice_registry` (`commands/practice.rs`) — a testable helper that
constructs the `ModuleRegistry` + fixed `ModuleInstallation`, registering `math_verify`
then `practice` in that order; `#[tauri::command]` handlers `generate_attempt`,
`evaluate_attempt`, `request_hint`, each translating between `practice::types`' snake_case
capability contract and a camelCase wire shape; real startup wiring in `lib.rs` (bundles
`knowledge-package/` as a Tauri resource, loads it via `load_knowledge_package` for the
first time outside a test, manages the registry + installation as Tauri state); the
frontend triad `src/types/practice.ts` + `src/services/practiceService.ts` +
`src/test/mockBackend.ts` wiring, tested through mocked IPC per `ARCHITECTURE.md` §5
rule 2.

Tested: `cargo test` across `commands::practice::tests` (registry construction, command
translation including a structural camelCase-key assertion, a full
generate→hint→evaluate sequence through the command layer); `npm run test` across
`practiceService.test.ts` (generate/evaluate open+solved/hint sequencing/unknown-attempt
errors), exercised through `handleMockInvoke`, never mocking the service module itself.
Gates run: `cargo check`/`test`/`clippy --lib -- -D warnings`/`fmt --check`,
`npm run typecheck`/`lint`/`build`/`test` — both sides, since this task touches `src-tauri/`
and `src/`.

Left out (per spec §1/§8, by design): Study Session UI (no page calls `practiceService.ts`
yet — this ships the contract, not a consumer); per-workspace module enable/disable wired
into capability resolution (fixed global `ModuleInstallation` instead); any `seed`
parameter on the generate command; the network-disabled offline acceptance test (depends
on Study Session UI existing first).
```

Set `status: review` in the frontmatter.

- [ ] **Step 3: Commit**

```bash
git add .ai/tasks/058-practice-tauri-wiring.md
git commit -m "task(058): move to review"
```
