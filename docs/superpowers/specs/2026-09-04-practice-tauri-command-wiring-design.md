# Practice Tauri command + frontend service wiring — design

## 1. Scope

Roadmap Stage 8, the next unblocked sub-project after the Practice Core Utility
(`.ai/tasks/_archive/057-practice-core-utility.md`, merged). Wires `practice.generate@1`,
`practice.evaluate@1`, `practice.hint@1` — real capabilities on the module-capability
runtime, but so far only ever invoked from test fixtures — through to something the
frontend can actually call: `#[tauri::command]` handlers, real app-startup construction of
the `ModuleRegistry`, and matching `src/services/*` functions.

**Does not build:** Study Session UI (a separate, later sub-project — this ships the
contract, not a consumer of it); per-workspace module enable/disable wired into capability
resolution (see §4 — a fixed, always-enabled installation is used instead, a deliberate,
documented simplification); adaptive family/difficulty selection; anything in
`CORE.md`'s explicitly-deferred permissions/storage-abstraction subsystems.

## 2. Decisions carried in from brainstorming

- **Real `knowledge-package/` content, bundled as a Tauri resource, not the test fixture.**
  `load_knowledge_package` has never been called from production code — only from tests,
  against `src/knowledge/tests/fixtures/canonical`. This task adds the missing piece: the
  repo-root `knowledge-package/` directory (the real shell-method reference package)
  ships inside the app bundle and gets loaded once at startup.
- **A translation boundary in the command layer, not a naming change to `practice::types`.**
  `practice::types`' structs (`GenerateRequest`, etc.) are snake_case to match the
  module-capability runtime's JSON wire contract, already locked in by task 057's tests —
  any capability caller, Tauri or otherwise, sees that same contract. Every existing
  Tauri-facing type in this codebase (`commands/models.rs`) is `camelCase`, matching the
  frontend's own convention. Rather than bending the capability contract to fit one caller,
  `commands/practice.rs` gets its own small camelCase request/response structs and converts
  explicitly — the same job `commands/*.rs` already does translating between SQL rows and
  wire types.
- **Frontend contract built now, ahead of any consuming page.** Matches this repo's
  Stage 2 precedent (component/service contracts locked before the pages that use them
  exist). `src/types/practice.ts`, `src/services/practiceService.ts`, and `mockBackend.ts`
  wiring all land in this task, each tested the same way every other service is —through
  mocked IPC (`ARCHITECTURE.md` §5 rule 2), not by mocking the service module itself —
  even though no page calls any of it yet.
- **Fixed, global `ModuleInstallation`, not per-workspace enablement.** Both first-party
  modules (`core.math_verify`, `org.axiom.practice`) are always enabled for every
  workspace in this task. Real per-workspace enable/disable wired into capability
  resolution would mean reading the `workspace_modules` table (Stage 7) per capability
  call — a real feature, but one nothing in the codebase does yet for *any* capability, and
  arguably touches `CORE.md`'s explicitly-deferred permissions subsystem. Flagged rather
  than silently expanded into; see §7 Follow-ups.

## 3. App-startup wiring

`src-tauri/tauri.conf.json`'s `bundle` gains:

```json
"resources": ["../knowledge-package"]
```

(relative to `src-tauri/`, Tauri v2's bundle-resource convention — the directory is copied
into the app bundle and becomes resolvable at runtime via the same `app.path()` API
`lib.rs` already uses for `app_data_dir()`.)

In `lib.rs`'s `setup` closure, alongside the existing `app.manage(commands::Database::open(...))`:

```rust
let knowledge_package_dir = app.path().resource_dir()?.join("knowledge-package");
let knowledge_package = crate::knowledge::load_knowledge_package(&knowledge_package_dir)
    .expect("bundled knowledge-package must load — a broken bundle is a build problem");

let mut registry = crate::modules::ModuleRegistry::new();
registry
    .register(
        crate::modules::parse(crate::capabilities::math_verify::MANIFEST_TOML)
            .expect("math_verify manifest must parse"),
        Box::new(crate::capabilities::math_verify::MathVerifyProvider),
    )
    .expect("math_verify must register");
let registry = std::sync::Arc::new(tauri::async_runtime::RwLock::new(registry));

let installation = crate::modules::ModuleInstallation {
    workspace_id: String::new(), // unused by resolve()/invoke() beyond echoing into
                                  // CallEnvelope; real per-call workspace_id is threaded
                                  // through each command's own input instead (§4) — see
                                  // §7 for why this field isn't load-bearing yet
    enabled_module_ids: vec![
        crate::modules::ModuleId::new("core.math_verify").expect("static id is valid"),
        crate::modules::ModuleId::new("org.axiom.practice").expect("static id is valid"),
    ],
};

let store = crate::practice::PracticeStore::new(crate::db::open(
    app_data_dir.join("axiom.sqlite3"),
)?);
let practice_provider = crate::practice::PracticeProvider::new(
    store,
    knowledge_package,
    std::sync::Arc::clone(&registry),
    installation.clone(),
);
registry
    .blocking_write()
    .register(
        crate::modules::parse(crate::practice::MANIFEST_TOML)
            .expect("practice manifest must parse"),
        Box::new(practice_provider),
    )
    .expect("practice must register");

app.manage(registry);
app.manage(installation);
```

`PracticeStore` opens its own connection to the same `axiom.sqlite3` file `Database`
already opens (matching how `rusqlite`/SQLite handles multiple connections to one file —
not sharing `Database`'s `Mutex<Connection>` directly, since `PracticeStore` is
`practice/`'s own encapsulated store, per task 057's design). `.expect()`/`.blocking_write()`
panics are deliberate at startup: a broken bundled manifest or package is a build-time
defect, not a runtime condition to recover from — matching the existing precedent of
`app_data_dir` creation using `?` inside `setup` (a closure that already returns
`tauri::Result`).

Note: `installation.workspace_id` is a placeholder empty string, never read by
`resolve`/`invoke` for any purpose beyond being available on the `ModuleInstallation`
value itself — the real per-request workspace id flows through `CallEnvelope.workspace_id`
(set from each command's own input, §4), not through this fixed installation. This is safe
today only because resolution is capability-id-based, not workspace-id-based (§7 flags the
real per-workspace gap this leaves).

## 4. Command layer

New `src-tauri/src/commands/practice.rs`, following the existing `commands/*.rs` shape
(a `*_handler` free function taking already-typed arguments, plus a thin
`#[tauri::command]` wrapper doing IPC deserialization) — see `commands/note.rs` for the
precedent this follows exactly.

```rust
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tauri::async_runtime::RwLock;
use tauri::State;

use crate::knowledge::ResponseType;
use crate::modules::{
    CallEnvelope, CapabilityCall, CapabilityId, CapabilityRequirement, ModuleId,
    ModuleInstallation, ModuleRegistry, RegistryError,
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

#[derive(Serialize)]
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

#[derive(Serialize)]
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

#[derive(Serialize)]
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
    Input: serde::Serialize,
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

`invoke_practice` is the one shared helper — resolves then invokes through the registry
exactly as `practice::evaluate` itself does internally (task 057 §4), just one layer up.
`GenerateRequest.seed` is always `None` here: only tests pass an explicit seed (task 057
spec §5); no command surface exposes it, matching "no adaptive/UI-facing policy" scope
discipline carried through this whole initiative.

Three commands added to `lib.rs`'s `tauri::generate_handler![...]` list:
`commands::practice::generate_attempt`, `commands::practice::evaluate_attempt`,
`commands::practice::request_hint`.

## 5. Frontend service layer

`src/types/practice.ts` (new file, re-exported from `src/types/index.ts`):

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

`src/services/practiceService.ts` (new file, matching every existing `*Service.ts`'s shape):

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

## 6. `mockBackend.ts` wiring

Three new `case` arms in `handleMockInvoke`, backed by a small in-memory attempt map (the
same shape `sessions`'s existing mutable module-level state already uses):

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

const mockAttempts = new Map<string, MockAttempt>();

// One hardcoded family, standing in for a real generation call the mock never needs to
// reproduce (mockBackend exercises shape and state transitions, not Rust's math):
const MOCK_FAMILY = {
  prompt: 'A region is bounded by y = 4x - x^2 and the x-axis. Find the volume when revolved about the y-axis.',
  responseType: 'numeric' as const,
  correctValue: 42.7,
  hints: ['Set up the shell method integral.', 'Integrate from x = 0 to x = 4.'],
};
```

`generateAttempt` creates and stores a `MockAttempt`, returns its `Attempt` shape.
`evaluateAttempt` looks up the attempt, compares `response.value` to `MOCK_FAMILY.correctValue`
(numeric tolerance `1e-6`, matching the real `math.verify` numeric path's spirit without
reimplementing it), records the submission, flips `status` to `'solved'` on a correct
answer, throws (matching every other mock handler's `Error` convention) if the attempt is
missing or already solved. `requestHint` increments `hintsRevealed` and returns the next
`hintTexts` entry, throwing once exhausted — mirroring `NoMoreHints`/`AlreadySolved`'s
shape as thrown errors, the same way `pauseSession` etc. already throw on invalid
transitions rather than returning a typed error.

## 7. Testing

Rust: `commands::practice::tests` — input-to-`practice::types` translation
(`GenerateAttemptInput` → `GenerateRequest`, camelCase↔snake_case field mapping),
response translation (`GenerateResponse` → `Attempt`, confirming `attempt_id` becomes
`attemptId` structurally in the serialized JSON, not just the Rust field name), and error
mapping (`RegistryError::NoCompatibleProvider` etc. → a non-empty `String`). One test
constructs a real in-process registry (mirroring `practice::tests`' round-trip fixture) and
drives a full `generate_attempt_handler` → `evaluate_attempt_handler` → `request_hint_handler`
sequence, proving the command layer's translation is correct end-to-end, not just
unit-correct per function.

Frontend: `src/services/practiceService.test.ts`, matching `moduleService.test.ts`'s shape
— calls the service functions against `handleMockInvoke` (via the existing Tauri IPC mock
setup every service test already uses), never mocking `practiceService.ts` itself. Covers:
generate returns an `Attempt` with the mock family's `hintsTotal`; evaluate with the wrong
value returns `status: 'open'`; evaluate with the correct value returns `status: 'solved'`;
hint returns sequential text and eventually throws; evaluate/hint on an unknown
`attemptId` throws.

## 8. Follow-ups (out of scope here, tracked for later)

- Per-workspace module enable/disable wired into capability resolution (§2, §3) — real
  work, deferred because nothing in the codebase does this for any capability yet, and it
  touches `CORE.md`'s deferred permissions subsystem.
- Study Session UI integration — the actual page(s) calling `practiceService.ts`.
- `GenerateRequest.seed` exposure for any purpose (e.g. a "regenerate this problem"
  affordance) — no command surface needs it yet.
- The network-disabled offline acceptance test spanning the whole Practice path, per
  `ROADMAP.md`'s Stage 8 list — depends on Study Session UI existing first.
