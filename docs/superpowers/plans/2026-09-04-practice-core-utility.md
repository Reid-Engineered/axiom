# Practice Core Utility Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship a first-party Practice module — `practice.generate@1`, `practice.evaluate@1`, `practice.hint@1` — registered on the module-capability runtime, backed by real SQLite persistence, that assembles Knowledge Package data, generation, and `math.verify` into a generate → attempt → evaluate flow.

**Architecture:** New top-level `src-tauri/src/practice/` module (types, error, store, provider), a new SQLite migration adding `practice_attempts`/`practice_submissions`, and a `PracticeProvider` that holds `Arc<tauri::async_runtime::RwLock<ModuleRegistry>>` (the async-aware `RwLock`, not `std::sync::RwLock` — its guards must survive an inner `.await`) to resolve and invoke `math.verify` internally exactly as any external caller would — the first real inter-capability call on the runtime.

**Tech Stack:** Rust, `rusqlite` 0.40.2, `async-trait` 0.1.92, `serde`/`serde_json`, existing `crate::modules` (capability runtime), `crate::knowledge` (Knowledge Package types + loader), `crate::generation` (problem generation engine), `crate::capabilities::math_verify`. No new dependencies.

**Spec:** [docs/superpowers/specs/2026-09-04-practice-core-utility-design.md](../specs/2026-09-04-practice-core-utility-design.md)

## Global Constraints

- Rust-only. No `#[tauri::command]`, no frontend/`src/services/*` wiring, no Study Session UI — those are later sub-projects (spec §1, §9).
- No new Cargo dependency. A "random enough" seed for `practice.generate` uses `std::collections::hash_map::RandomState` (already used internally by Rust's stdlib `HashMap`, draws from OS randomness, zero new dependency) — not the `rand` crate.
- `tauri::async_runtime::RwLock` is assumed to re-export `tokio::sync::RwLock` directly (including its `blocking_read()`/`blocking_write()` methods, used by test helpers that aren't themselves `async fn`). Task 6 Step 5 (`cargo test`/`cargo clippy`) is the first point this assumption gets checked by the compiler — if `blocking_read`/`blocking_write` aren't available on the type Tauri re-exports, replace every `.blocking_read()`/`.blocking_write()` call site (Tasks 7 and 9) with `tauri::async_runtime::block_on(async { registry.read().await ... })`/`block_on(async { registry.write().await ... })` instead; the production code path in `evaluate()` (Task 7) already uses the plain async `.read().await` form regardless and is unaffected.
- Practice calls `math.verify` only through `ModuleRegistry::resolve`/`invoke` — never by calling `MathVerifyProvider`'s Rust code directly (spec §4).
- `canonical_solution` and unrevealed hint text never appear in a `practice.generate`/`practice.hint` JSON response — only in the persisted `practice_attempts.instance_json` row (spec §5).
- Every `practice.*` handler's first step on an existing attempt is loading it filtered by **both** `id` and `workspace_id` — an id from another workspace is `AttemptNotFound`, not a distinguishable error (spec §5).
- Migration numbering follows `src-tauri/src/db/schema.rs`'s existing `MIGRATIONS` array — this plan adds migration `version: 2`, name `"practice_attempts"`, bumping `LATEST_SCHEMA_VERSION` to `2`.
- Module id `org.axiom.practice`; capability ids `practice.generate`, `practice.evaluate`, `practice.hint`, each `version = 1`. `math.verify`'s existing module id is `core.math_verify` (`src-tauri/src/capabilities/math_verify/module.toml`) — do not change it.
- Task file: `.ai/tasks/057-practice-core-utility.md` (`stage: 8`, `depends_on: [45, 46, 47, 48, 54, 55, 56]`), per `.ai/lifecycle.md`.

---

### Task 1: Create the task file

**Files:**
- Create: `.ai/tasks/057-practice-core-utility.md`

**Interfaces:** None — this is bookkeeping, not code.

- [ ] **Step 1: Write the task file**

```markdown
---
id: 057
title: Practice Core Utility
status: in-progress
owner: claude-code
stage: 8
depends_on: [45, 46, 47, 48, 54, 55, 56]
---

## Scope

Add a first-party Practice module (`org.axiom.practice`) providing `practice.generate@1`,
`practice.evaluate@1`, `practice.hint@1` on the module-capability runtime, backed by real
SQLite persistence (`practice_attempts`, `practice_submissions`). Assembles the Knowledge
Package, canonical Problem schema, `math.verify`, and problem generation (tasks 049-056)
into a generate -> attempt -> evaluate flow. Does not build: any Tauri command or frontend
wiring, Study Session UI, adaptive family/difficulty selection, or adaptive hint selection —
see `docs/superpowers/specs/2026-09-04-practice-core-utility-design.md` §1/§9.

## Plan

- `src-tauri/src/db/migrations/0002_practice.sql`, `src-tauri/src/db/schema.rs` (new tables)
- `src-tauri/src/practice/module.toml`, `mod.rs`, `types.rs`, `error.rs`, `store.rs`,
  `provider.rs`, `tests/mod.rs`
- `src-tauri/src/lib.rs` (register `pub mod practice;`)

See `docs/superpowers/plans/2026-09-04-practice-core-utility.md` for the task-by-task
implementation plan.

## Worklog

- 2026-09-04 — started, claimed by claude-code

## What was built / tested / left out

(filled in at Task 9)

## Review

(filled in by reviewer)

## Follow-ups

(filled in if anything is noticed during implementation/review)
```

- [ ] **Step 2: Commit**

```bash
git add .ai/tasks/057-practice-core-utility.md
git commit -m "task(057): open Practice Core Utility"
```

---

### Task 2: Migration — `practice_attempts` and `practice_submissions`

**Files:**
- Create: `src-tauri/src/db/migrations/0002_practice.sql`
- Modify: `src-tauri/src/db/schema.rs`
- Test: `src-tauri/src/db/tests.rs`

**Interfaces:**
- Produces: tables `practice_attempts(id, workspace_id, family_id, seed, instance_json, hints_revealed, status, created_at, updated_at)` and `practice_submissions(id, attempt_id, response_json, correct, submitted_at)`, reachable via `crate::db::open_in_memory()` once migrated.

- [ ] **Step 1: Write the failing test**

Append to `src-tauri/src/db/tests.rs` (matching its existing style — check the file's current imports/helpers first; use the same `open_in_memory` or equivalent helper already used by neighboring tests in that file):

```rust
#[test]
fn practice_tables_exist_after_migration() {
    let connection = crate::db::open_in_memory().unwrap();

    let attempts_columns: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('practice_attempts')",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(attempts_columns, 9);

    let submissions_columns: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('practice_submissions')",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(submissions_columns, 5);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd src-tauri && cargo test practice_tables_exist_after_migration`
Expected: FAIL — `no such table: practice_attempts` (or the `pragma_table_info` count is `0`).

- [ ] **Step 3: Write the migration**

Create `src-tauri/src/db/migrations/0002_practice.sql`:

```sql
CREATE TABLE practice_attempts (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    family_id TEXT NOT NULL,
    seed INTEGER NOT NULL,
    instance_json TEXT NOT NULL,
    hints_revealed INTEGER NOT NULL DEFAULT 0 CHECK (hints_revealed >= 0),
    status TEXT NOT NULL CHECK (status IN ('open', 'solved')),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX practice_attempts_by_workspace
ON practice_attempts(workspace_id, status);

CREATE TABLE practice_submissions (
    id TEXT PRIMARY KEY,
    attempt_id TEXT NOT NULL REFERENCES practice_attempts(id) ON DELETE CASCADE,
    response_json TEXT NOT NULL,
    correct INTEGER NOT NULL CHECK (correct IN (0, 1)),
    submitted_at TEXT NOT NULL
);

CREATE INDEX practice_submissions_by_attempt
ON practice_submissions(attempt_id, submitted_at);
```

Modify `src-tauri/src/db/schema.rs` to register it:

```rust
pub const LATEST_SCHEMA_VERSION: i64 = 2;

pub struct Migration {
    pub version: i64,
    pub name: &'static str,
    pub sql: &'static str,
}

pub const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        name: "initial_domain_schema",
        sql: include_str!("migrations/0001_initial.sql"),
    },
    Migration {
        version: 2,
        name: "practice_attempts",
        sql: include_str!("migrations/0002_practice.sql"),
    },
];
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cd src-tauri && cargo test practice_tables_exist_after_migration`
Expected: PASS

- [ ] **Step 5: Run the full existing db test suite to confirm no regression**

Run: `cd src-tauri && cargo test --lib db::`
Expected: PASS (all prior migration/schema tests still pass — `LATEST_SCHEMA_VERSION` bump doesn't break anything already asserting a literal value; if a test hardcodes `LATEST_SCHEMA_VERSION == 1`, update it to reference the constant instead, not a new literal).

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/db/migrations/0002_practice.sql src-tauri/src/db/schema.rs src-tauri/src/db/tests.rs
git commit -m "feat(db): add practice_attempts and practice_submissions tables"
```

---

### Task 3: `practice/types.rs` — request/response contracts

**Files:**
- Create: `src-tauri/src/practice/types.rs`

**Interfaces:**
- Consumes: `crate::knowledge::ResponseType` (existing enum, `SymbolicExpression | Numeric`).
- Produces: `AttemptStatus { Open, Solved }`, `ResponseValue { SymbolicExpression { value: String }, Numeric { value: f64 } }`, `GenerateRequest { workspace_id: String, family_id: String, seed: Option<u64> }`, `GenerateResponse { attempt_id: String, prompt: String, response_type: ResponseType, hints_total: u32 }`, `EvaluateRequest { workspace_id: String, attempt_id: String, response: ResponseValue }`, `EvaluateResponse { correct: bool, status: AttemptStatus, submission_count: u32 }`, `HintRequest { workspace_id: String, attempt_id: String }`, `HintResponse { hint_text: String, hints_revealed: u32, hints_total: u32 }` — all used by `provider.rs` (Task 6-8) and `store.rs` (Task 4).

- [ ] **Step 1: Write the failing tests**

```rust
use serde::{Deserialize, Serialize};

use crate::knowledge::ResponseType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AttemptStatus {
    Open,
    Solved,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(tag = "response_type", rename_all = "kebab-case")]
pub enum ResponseValue {
    SymbolicExpression { value: String },
    Numeric { value: f64 },
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct GenerateRequest {
    pub workspace_id: String,
    pub family_id: String,
    #[serde(default)]
    pub seed: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct GenerateResponse {
    pub attempt_id: String,
    pub prompt: String,
    pub response_type: ResponseType,
    pub hints_total: u32,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct EvaluateRequest {
    pub workspace_id: String,
    pub attempt_id: String,
    pub response: ResponseValue,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct EvaluateResponse {
    pub correct: bool,
    pub status: AttemptStatus,
    pub submission_count: u32,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct HintRequest {
    pub workspace_id: String,
    pub attempt_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct HintResponse {
    pub hint_text: String,
    pub hints_revealed: u32,
    pub hints_total: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_request_deserializes_without_a_seed() {
        let value = serde_json::json!({
            "workspace_id": "ws-1",
            "family_id": "problem.shell_y_poly",
        });
        let request: GenerateRequest = serde_json::from_value(value).unwrap();
        assert_eq!(request.seed, None);
    }

    #[test]
    fn generate_request_deserializes_with_an_explicit_seed() {
        let value = serde_json::json!({
            "workspace_id": "ws-1",
            "family_id": "problem.shell_y_poly",
            "seed": 42,
        });
        let request: GenerateRequest = serde_json::from_value(value).unwrap();
        assert_eq!(request.seed, Some(42));
    }

    #[test]
    fn generate_response_never_serializes_a_canonical_solution_field() {
        let response = GenerateResponse {
            attempt_id: "attempt-1".to_owned(),
            prompt: "Find the volume.".to_owned(),
            response_type: ResponseType::SymbolicExpression,
            hints_total: 2,
        };
        let value = serde_json::to_value(&response).unwrap();
        assert!(value.get("canonical_solution").is_none());
        assert!(value.get("hints").is_none());
    }

    #[test]
    fn numeric_response_value_round_trips() {
        let value = serde_json::json!({ "response_type": "numeric", "value": 4.0 });
        let response: ResponseValue = serde_json::from_value(value).unwrap();
        assert_eq!(response, ResponseValue::Numeric { value: 4.0 });
    }

    #[test]
    fn symbolic_response_value_round_trips() {
        let value = serde_json::json!({
            "response_type": "symbolic-expression",
            "value": "2*pi",
        });
        let response: ResponseValue = serde_json::from_value(value).unwrap();
        assert_eq!(
            response,
            ResponseValue::SymbolicExpression { value: "2*pi".to_owned() }
        );
    }

    #[test]
    fn hint_response_never_serializes_an_unrevealed_hint_list() {
        let response = HintResponse {
            hint_text: "Identify the radius.".to_owned(),
            hints_revealed: 1,
            hints_total: 3,
        };
        let value = serde_json::to_value(&response).unwrap();
        assert!(value.get("hints").is_none());
        assert_eq!(value["hint_text"], "Identify the radius.");
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd src-tauri && cargo test practice::types::tests`
Expected: FAIL to compile — `crate::practice` module doesn't exist yet.

- [ ] **Step 3: Create the module skeleton so this file compiles**

This step exists only to make Task 3's tests runnable in isolation — Task 5 fills in the real `mod.rs`. Create `src-tauri/src/practice/mod.rs` with just:

```rust
mod types;
```

And add `pub mod practice;` to `src-tauri/src/lib.rs`'s module declaration block (alongside `pub mod capabilities;`, `pub mod commands;`, etc — keep the existing alphabetical-ish ordering, inserting after `pub mod modules;`).

- [ ] **Step 4: Run test to verify it passes**

Run: `cd src-tauri && cargo test practice::types::tests`
Expected: PASS (6 tests)

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/practice/ src-tauri/src/lib.rs
git commit -m "feat(practice): add request/response types"
```

---

### Task 4: `practice/error.rs` — `PracticeError`

**Files:**
- Create: `src-tauri/src/practice/error.rs`
- Modify: `src-tauri/src/practice/mod.rs`

**Interfaces:**
- Consumes: `crate::generation::GenerationError` (existing, `Display`-able).
- Produces: `PracticeError` enum used by `store.rs` (Task 5) and `provider.rs` (Task 6-8):
  `FamilyNotFound { family_id: String }`, `AttemptNotFound { attempt_id: String }`,
  `NoMoreHints { attempt_id: String }`, `AlreadySolved { attempt_id: String }`,
  `ResponseTypeMismatch { attempt_id: String }`, `GenerationFailed(GenerationError)`,
  `VerificationFailed(String)`, `Storage(String)`.

- [ ] **Step 1: Write the failing tests**

```rust
use std::error::Error;
use std::fmt;

use crate::generation::GenerationError;

#[derive(Debug)]
pub enum PracticeError {
    FamilyNotFound { family_id: String },
    AttemptNotFound { attempt_id: String },
    NoMoreHints { attempt_id: String },
    AlreadySolved { attempt_id: String },
    ResponseTypeMismatch { attempt_id: String },
    GenerationFailed(GenerationError),
    VerificationFailed(String),
    Storage(String),
}

impl fmt::Display for PracticeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FamilyNotFound { family_id } => {
                write!(formatter, "no problem family {family_id:?} exists")
            }
            Self::AttemptNotFound { attempt_id } => {
                write!(formatter, "no attempt {attempt_id:?} exists in this workspace")
            }
            Self::NoMoreHints { attempt_id } => write!(
                formatter,
                "attempt {attempt_id:?} has no more hints to reveal"
            ),
            Self::AlreadySolved { attempt_id } => {
                write!(formatter, "attempt {attempt_id:?} is already solved")
            }
            Self::ResponseTypeMismatch { attempt_id } => write!(
                formatter,
                "response shape for attempt {attempt_id:?} does not match its response_type"
            ),
            Self::GenerationFailed(error) => write!(formatter, "generation failed: {error}"),
            Self::VerificationFailed(message) => {
                write!(formatter, "verification failed: {message}")
            }
            Self::Storage(message) => write!(formatter, "storage error: {message}"),
        }
    }
}

impl Error for PracticeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::GenerationFailed(error) => Some(error),
            _ => None,
        }
    }
}

impl From<GenerationError> for PracticeError {
    fn from(error: GenerationError) -> Self {
        Self::GenerationFailed(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::knowledge::GeneratorId;

    #[test]
    fn family_not_found_displays_the_family_id() {
        let error = PracticeError::FamilyNotFound {
            family_id: "problem.nonexistent".to_owned(),
        };
        assert_eq!(
            error.to_string(),
            "no problem family \"problem.nonexistent\" exists"
        );
    }

    #[test]
    fn attempt_not_found_displays_the_attempt_id() {
        let error = PracticeError::AttemptNotFound {
            attempt_id: "attempt-1".to_owned(),
        };
        assert_eq!(
            error.to_string(),
            "no attempt \"attempt-1\" exists in this workspace"
        );
    }

    #[test]
    fn generation_failed_wraps_and_displays_the_underlying_error() {
        let underlying = GenerationError::UnknownGenerator {
            id: GeneratorId::new("gen.nonexistent").unwrap(),
        };
        let error: PracticeError = underlying.into();
        assert_eq!(
            error.to_string(),
            "generation failed: no generator is registered for gen.nonexistent"
        );
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd src-tauri && cargo test practice::error::tests`
Expected: FAIL to compile — `error` module not declared in `practice/mod.rs` yet.

- [ ] **Step 3: Wire the module in**

Modify `src-tauri/src/practice/mod.rs`:

```rust
mod error;
mod types;

pub use error::PracticeError;
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cd src-tauri && cargo test practice::error::tests && cargo clippy -p axiom_lib`
Expected: PASS, zero clippy warnings (remove the placeholder import line from Step 1 once real usage covers both imports — by Task 6 `ProblemFamilyId` will be used elsewhere in this file's tests if needed; if not, delete the unused import per clippy's guidance now).

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/practice/error.rs src-tauri/src/practice/mod.rs
git commit -m "feat(practice): add PracticeError"
```

---

### Task 5: `practice/store.rs` — persistence

**Files:**
- Create: `src-tauri/src/practice/store.rs`
- Modify: `src-tauri/src/practice/mod.rs`

**Interfaces:**
- Consumes: `rusqlite::Connection`, `crate::knowledge::ProblemInstance` (existing, `Serialize`/`Deserialize`), `PracticeError` (Task 4), `AttemptStatus` (Task 3).
- Produces: `pub struct PracticeStore(Mutex<Connection>)` with `PracticeStore::new(Connection) -> Self`; `pub struct AttemptRow { pub id: String, pub workspace_id: String, pub family_id: String, pub seed: u64, pub instance: ProblemInstance, pub hints_revealed: u32, pub status: AttemptStatus, pub created_at: String, pub updated_at: String }`; methods `insert_attempt(&self, id: &str, workspace_id: &str, family_id: &str, seed: u64, instance: &ProblemInstance) -> Result<(), PracticeError>`, `load_attempt(&self, attempt_id: &str, workspace_id: &str) -> Result<AttemptRow, PracticeError>`, `increment_hints_revealed(&self, attempt_id: &str) -> Result<u32, PracticeError>`, `record_submission(&self, attempt_id: &str, response_json: &str, correct: bool) -> Result<(), PracticeError>`, `mark_solved(&self, attempt_id: &str) -> Result<(), PracticeError>`, `count_submissions(&self, attempt_id: &str) -> Result<u32, PracticeError>` — all used by `provider.rs` (Task 6-8).

- [ ] **Step 1: Write the failing tests**

```rust
use std::sync::{Mutex, MutexGuard};

use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::knowledge::ProblemInstance;

use super::error::PracticeError;
use super::types::AttemptStatus;

pub struct PracticeStore(Mutex<Connection>);

#[derive(Debug, Clone, PartialEq)]
pub struct AttemptRow {
    pub id: String,
    pub workspace_id: String,
    pub family_id: String,
    pub seed: u64,
    pub instance: ProblemInstance,
    pub hints_revealed: u32,
    pub status: AttemptStatus,
    pub created_at: String,
    pub updated_at: String,
}

impl PracticeStore {
    pub fn new(connection: Connection) -> Self {
        Self(Mutex::new(connection))
    }

    fn connection(&self) -> Result<MutexGuard<'_, Connection>, PracticeError> {
        self.0
            .lock()
            .map_err(|_| PracticeError::Storage("connection lock is poisoned".to_owned()))
    }

    pub fn insert_attempt(
        &self,
        id: &str,
        workspace_id: &str,
        family_id: &str,
        seed: u64,
        instance: &ProblemInstance,
    ) -> Result<(), PracticeError> {
        let instance_json = serde_json::to_string(instance)
            .map_err(|error| PracticeError::Storage(error.to_string()))?;
        let now = now();
        self.connection()?
            .execute(
                "INSERT INTO practice_attempts
                    (id, workspace_id, family_id, seed, instance_json, hints_revealed,
                     status, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, 0, 'open', ?6, ?6)",
                params![id, workspace_id, family_id, seed as i64, instance_json, now],
            )
            .map_err(|error| PracticeError::Storage(error.to_string()))?;
        Ok(())
    }

    pub fn load_attempt(
        &self,
        attempt_id: &str,
        workspace_id: &str,
    ) -> Result<AttemptRow, PracticeError> {
        self.connection()?
            .query_row(
                "SELECT id, workspace_id, family_id, seed, instance_json, hints_revealed,
                        status, created_at, updated_at
                 FROM practice_attempts WHERE id = ?1 AND workspace_id = ?2",
                params![attempt_id, workspace_id],
                map_attempt_row,
            )
            .optional()
            .map_err(|error| PracticeError::Storage(error.to_string()))?
            .ok_or_else(|| PracticeError::AttemptNotFound {
                attempt_id: attempt_id.to_owned(),
            })
    }

    pub fn increment_hints_revealed(&self, attempt_id: &str) -> Result<u32, PracticeError> {
        let connection = self.connection()?;
        connection
            .execute(
                "UPDATE practice_attempts SET hints_revealed = hints_revealed + 1, updated_at = ?2
                 WHERE id = ?1",
                params![attempt_id, now()],
            )
            .map_err(|error| PracticeError::Storage(error.to_string()))?;
        connection
            .query_row(
                "SELECT hints_revealed FROM practice_attempts WHERE id = ?1",
                params![attempt_id],
                |row| row.get::<_, i64>(0),
            )
            .map(|value| value as u32)
            .map_err(|error| PracticeError::Storage(error.to_string()))
    }

    pub fn record_submission(
        &self,
        attempt_id: &str,
        response_json: &str,
        correct: bool,
    ) -> Result<(), PracticeError> {
        self.connection()?
            .execute(
                "INSERT INTO practice_submissions (id, attempt_id, response_json, correct, submitted_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![new_id(), attempt_id, response_json, correct as i64, now()],
            )
            .map_err(|error| PracticeError::Storage(error.to_string()))?;
        Ok(())
    }

    pub fn mark_solved(&self, attempt_id: &str) -> Result<(), PracticeError> {
        self.connection()?
            .execute(
                "UPDATE practice_attempts SET status = 'solved', updated_at = ?2 WHERE id = ?1",
                params![attempt_id, now()],
            )
            .map_err(|error| PracticeError::Storage(error.to_string()))?;
        Ok(())
    }

    pub fn count_submissions(&self, attempt_id: &str) -> Result<u32, PracticeError> {
        self.connection()?
            .query_row(
                "SELECT COUNT(*) FROM practice_submissions WHERE attempt_id = ?1",
                params![attempt_id],
                |row| row.get::<_, i64>(0),
            )
            .map(|value| value as u32)
            .map_err(|error| PracticeError::Storage(error.to_string()))
    }
}

fn map_attempt_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<AttemptRow> {
    let instance_json: String = row.get(4)?;
    let instance: ProblemInstance = serde_json::from_str(&instance_json).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(4, rusqlite::types::Type::Text, Box::new(error))
    })?;
    let status_text: String = row.get(6)?;
    let status = match status_text.as_str() {
        "solved" => AttemptStatus::Solved,
        _ => AttemptStatus::Open,
    };
    Ok(AttemptRow {
        id: row.get(0)?,
        workspace_id: row.get(1)?,
        family_id: row.get(2)?,
        seed: row.get::<_, i64>(3)? as u64,
        instance,
        hints_revealed: row.get::<_, i64>(5)? as u32,
        status,
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
    })
}

fn now() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

fn new_id() -> String {
    format!("submission-{}", uuid::Uuid::new_v4())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_instance() -> ProblemInstance {
        use std::collections::BTreeMap;
        ProblemInstance {
            family_id: crate::knowledge::ProblemFamilyId::new("problem.shell_y_poly").unwrap(),
            seed: 7,
            resolved_parameters: BTreeMap::from([("coeff".to_owned(), 4.0)]),
            prompt: "Find the volume.".to_owned(),
            canonical_solution: crate::knowledge::ResolvedSolution::Numeric(12.0),
            hints: vec!["Identify the radius.".to_owned()],
        }
    }

    fn store() -> PracticeStore {
        PracticeStore::new(crate::db::open_in_memory().unwrap())
    }

    fn seed_workspace(store: &PracticeStore) {
        store
            .connection()
            .unwrap()
            .execute(
                "INSERT INTO workspaces (id, name, guiding_goal_id, progress, paused)
                 VALUES ('ws-1', 'Test', 'goal-1', 0.0, 0)",
                [],
            )
            .unwrap();
    }

    #[test]
    fn insert_then_load_round_trips_the_full_instance() {
        let store = store();
        seed_workspace(&store);
        let instance = sample_instance();
        store
            .insert_attempt("attempt-1", "ws-1", "problem.shell_y_poly", 7, &instance)
            .unwrap();

        let row = store.load_attempt("attempt-1", "ws-1").unwrap();

        assert_eq!(row.instance, instance);
        assert_eq!(row.hints_revealed, 0);
        assert_eq!(row.status, AttemptStatus::Open);
    }

    #[test]
    fn load_attempt_from_a_different_workspace_is_not_found() {
        let store = store();
        seed_workspace(&store);
        store
            .insert_attempt("attempt-1", "ws-1", "problem.shell_y_poly", 7, &sample_instance())
            .unwrap();

        let result = store.load_attempt("attempt-1", "ws-other");

        assert!(matches!(result, Err(PracticeError::AttemptNotFound { .. })));
    }

    #[test]
    fn increment_hints_revealed_persists_across_loads() {
        let store = store();
        seed_workspace(&store);
        store
            .insert_attempt("attempt-1", "ws-1", "problem.shell_y_poly", 7, &sample_instance())
            .unwrap();

        assert_eq!(store.increment_hints_revealed("attempt-1").unwrap(), 1);
        assert_eq!(store.increment_hints_revealed("attempt-1").unwrap(), 2);
        assert_eq!(store.load_attempt("attempt-1", "ws-1").unwrap().hints_revealed, 2);
    }

    #[test]
    fn record_submission_then_count_reflects_it() {
        let store = store();
        seed_workspace(&store);
        store
            .insert_attempt("attempt-1", "ws-1", "problem.shell_y_poly", 7, &sample_instance())
            .unwrap();

        store.record_submission("attempt-1", "{\"value\":1.0}", false).unwrap();
        store.record_submission("attempt-1", "{\"value\":4.0}", true).unwrap();

        assert_eq!(store.count_submissions("attempt-1").unwrap(), 2);
    }

    #[test]
    fn mark_solved_updates_status() {
        let store = store();
        seed_workspace(&store);
        store
            .insert_attempt("attempt-1", "ws-1", "problem.shell_y_poly", 7, &sample_instance())
            .unwrap();

        store.mark_solved("attempt-1").unwrap();

        assert_eq!(store.load_attempt("attempt-1", "ws-1").unwrap().status, AttemptStatus::Solved);
    }

    #[test]
    fn attempt_surviving_a_fresh_connection_to_the_same_file_reads_back_identically() {
        let dir = std::env::temp_dir().join(format!("axiom-practice-test-{}", uuid::Uuid::new_v4()));
        let db_path = dir.join("axiom.sqlite3");
        std::fs::create_dir_all(&dir).unwrap();

        {
            let store = PracticeStore::new(crate::db::open(&db_path).unwrap());
            seed_workspace(&store);
            store
                .insert_attempt("attempt-1", "ws-1", "problem.shell_y_poly", 7, &sample_instance())
                .unwrap();
        }

        let reopened = PracticeStore::new(crate::db::open(&db_path).unwrap());
        let row = reopened.load_attempt("attempt-1", "ws-1").unwrap();
        assert_eq!(row.instance, sample_instance());

        std::fs::remove_dir_all(&dir).unwrap();
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd src-tauri && cargo test practice::store::tests`
Expected: FAIL to compile — `store` module not declared yet, and `Connection::execute` on a private `self.connection()` used from the test module needs the field/method visible within the crate (it is, since tests live in the same file's `mod tests` — but `store` itself isn't wired into `practice/mod.rs` yet).

- [ ] **Step 3: Wire the module in**

Modify `src-tauri/src/practice/mod.rs`:

```rust
mod error;
mod store;
mod types;

pub use error::PracticeError;
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cd src-tauri && cargo test practice::store::tests`
Expected: PASS (6 tests)

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/practice/store.rs src-tauri/src/practice/mod.rs
git commit -m "feat(practice): add SQLite-backed attempt store"
```

---

### Task 6: `practice/module.toml` and `practice.generate`

**Files:**
- Create: `src-tauri/src/practice/module.toml`
- Create: `src-tauri/src/practice/provider.rs`
- Modify: `src-tauri/src/practice/mod.rs`

**Interfaces:**
- Consumes: `crate::knowledge::{KnowledgePackage, ProblemFamilyId, load_knowledge_package}`, `crate::generation::generate_problem_instance`, `crate::modules::{CapabilityId, CapabilityProvider, InvocationError}`, `PracticeStore`/`PracticeError` (Tasks 4-5), `GenerateRequest`/`GenerateResponse` (Task 3).
- Produces: `pub struct PracticeProvider { .. }` with `PracticeProvider::new(store: PracticeStore, knowledge_package: KnowledgePackage, registry: std::sync::Arc<tauri::async_runtime::RwLock<crate::modules::ModuleRegistry>>, installation: crate::modules::ModuleInstallation) -> Self` and a `CapabilityProvider` impl whose `invoke` currently handles `("practice.generate", 1)` (evaluate/hint land in Tasks 7-8 as `UnknownCapability` for now). `pub const MANIFEST_TOML: &str` re-exported from `mod.rs`. Note: `Arc` here is `std::sync::Arc` (fine to share across threads/clone cheaply); the `RwLock` it wraps is Tauri's async-aware one (`tauri::async_runtime::RwLock`, re-exporting `tokio::sync::RwLock`) — its guards are `Send`, unlike `std::sync::RwLock`'s, which Task 7's inner `.await` while holding the lock requires.

- [ ] **Step 1: Write the manifest**

Create `src-tauri/src/practice/module.toml`:

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

- [ ] **Step 2: Write the failing test**

Create `src-tauri/src/practice/provider.rs`:

```rust
use std::sync::Arc;

use serde_json::Value;
use tauri::async_runtime::RwLock;

use crate::knowledge::{KnowledgePackage, ProblemFamilyId};
use crate::modules::{CapabilityId, CapabilityProvider, InvocationError, ModuleInstallation, ModuleRegistry};

use super::error::PracticeError;
use super::store::PracticeStore;
use super::types::{GenerateRequest, GenerateResponse};

pub struct PracticeProvider {
    store: PracticeStore,
    knowledge_package: KnowledgePackage,
    // Holds the registry it will itself be registered into (Task 6/9's registration test) —
    // an intentional Arc reference cycle, acceptable because ModuleRegistry is a
    // process-lifetime singleton, never freed before exit. See spec §4. Must be the
    // async-aware RwLock (tauri::async_runtime::RwLock, not std::sync::RwLock) because
    // Task 7's evaluate() holds the read guard across an inner `.await`, and only an
    // async-aware lock's guard is Send.
    #[allow(dead_code)]
    registry: Arc<RwLock<ModuleRegistry>>,
    installation: ModuleInstallation,
}

impl PracticeProvider {
    pub fn new(
        store: PracticeStore,
        knowledge_package: KnowledgePackage,
        registry: Arc<RwLock<ModuleRegistry>>,
        installation: ModuleInstallation,
    ) -> Self {
        Self {
            store,
            knowledge_package,
            registry,
            installation,
        }
    }

    async fn handle_generate(&self, input: Value) -> Result<Value, InvocationError> {
        let request: GenerateRequest = serde_json::from_value(input).map_err(|error| {
            InvocationError::InvalidInput {
                capability_id: capability_id("practice.generate"),
                message: error.to_string(),
            }
        })?;
        let response = self
            .generate(request)
            .await
            .map_err(|error| to_invocation_error("practice.generate", error))?;
        serde_json::to_value(response).map_err(|error| InvocationError::Failed {
            message: error.to_string(),
        })
    }

    async fn generate(&self, request: GenerateRequest) -> Result<GenerateResponse, PracticeError> {
        let family = self
            .knowledge_package
            .problem_families
            .iter()
            .find(|family| family.id.as_str() == request.family_id)
            .ok_or_else(|| PracticeError::FamilyNotFound {
                family_id: request.family_id.clone(),
            })?;

        let seed = request.seed.unwrap_or_else(random_seed);
        let instance = crate::generation::generate_problem_instance(family, seed)?;

        let attempt_id = format!("attempt-{}", uuid::Uuid::new_v4());
        self.store.insert_attempt(
            &attempt_id,
            &request.workspace_id,
            request.family_id.as_str(),
            seed,
            &instance,
        )?;

        Ok(GenerateResponse {
            attempt_id,
            prompt: instance.prompt,
            response_type: family.response_type,
            hints_total: instance.hints.len() as u32,
        })
    }
}

#[async_trait::async_trait]
impl CapabilityProvider for PracticeProvider {
    async fn invoke(
        &self,
        capability_id: &CapabilityId,
        version: u32,
        input: Value,
    ) -> Result<Value, InvocationError> {
        match (capability_id.as_str(), version) {
            ("practice.generate", 1) => self.handle_generate(input).await,
            _ => Err(InvocationError::UnknownCapability {
                capability_id: capability_id.clone(),
                version,
            }),
        }
    }
}

fn capability_id(value: &str) -> CapabilityId {
    CapabilityId::new(value).expect("static capability id is valid")
}

fn to_invocation_error(capability: &str, error: PracticeError) -> InvocationError {
    match error {
        PracticeError::FamilyNotFound { .. }
        | PracticeError::AttemptNotFound { .. }
        | PracticeError::NoMoreHints { .. }
        | PracticeError::AlreadySolved { .. }
        | PracticeError::ResponseTypeMismatch { .. } => InvocationError::InvalidInput {
            capability_id: capability_id(capability),
            message: error.to_string(),
        },
        PracticeError::GenerationFailed(_)
        | PracticeError::VerificationFailed(_)
        | PracticeError::Storage(_) => InvocationError::Failed {
            message: error.to_string(),
        },
    }
}

/// A seed with no reproducibility requirement (real `practice.generate` calls, as opposed to
/// tests that pass an explicit `seed`). `RandomState`'s keys are drawn from OS randomness by
/// `std`, with zero new dependency — deliberately not the `rand` crate, matching the project's
/// existing no-new-dependency pattern (see the problem-generation design's own RNG decision).
fn random_seed() -> u64 {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};
    RandomState::new().build_hasher().finish()
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use crate::modules::ModuleId;

    fn fixture_package() -> KnowledgePackage {
        let fixture_root =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("src/knowledge/tests/fixtures/canonical");
        crate::knowledge::load_knowledge_package(&fixture_root).unwrap()
    }

    fn provider() -> PracticeProvider {
        let store = PracticeStore::new(crate::db::open_in_memory().unwrap());
        store
            .connection_for_test()
            .execute(
                "INSERT INTO workspaces (id, name, guiding_goal_id, progress, paused)
                 VALUES ('ws-1', 'Test', 'goal-1', 0.0, 0)",
                [],
            )
            .unwrap();
        let registry = Arc::new(RwLock::new(ModuleRegistry::new())); // tauri::async_runtime::RwLock
        let installation = ModuleInstallation {
            workspace_id: "ws-1".to_owned(),
            enabled_module_ids: vec![ModuleId::new("org.axiom.practice").unwrap()],
        };
        PracticeProvider::new(store, fixture_package(), registry, installation)
    }

    #[test]
    fn generate_with_an_explicit_seed_matches_the_generation_engine_directly() {
        let provider = provider();
        let family = provider
            .knowledge_package
            .problem_families
            .iter()
            .find(|family| family.id.as_str() == "problem.shell_y_poly")
            .unwrap();
        let expected = crate::generation::generate_problem_instance(family, 42).unwrap();

        let request = GenerateRequest {
            workspace_id: "ws-1".to_owned(),
            family_id: "problem.shell_y_poly".to_owned(),
            seed: Some(42),
        };
        let response = tauri::async_runtime::block_on(provider.generate(request)).unwrap();

        assert_eq!(response.prompt, expected.prompt);
        assert_eq!(response.hints_total, expected.hints.len() as u32);
    }

    #[test]
    fn generate_response_never_exposes_the_canonical_solution() {
        let provider = provider();
        let capability_id = CapabilityId::new("practice.generate").unwrap();
        let input = serde_json::json!({
            "workspace_id": "ws-1",
            "family_id": "problem.shell_y_poly",
            "seed": 42,
        });

        let output = tauri::async_runtime::block_on(provider.invoke(&capability_id, 1, input))
            .unwrap();

        assert!(output.get("canonical_solution").is_none());
        assert!(output.get("hints").is_none());
        assert!(output["attempt_id"].is_string());
    }

    #[test]
    fn generate_with_an_unknown_family_id_is_invalid_input() {
        let provider = provider();
        let capability_id = CapabilityId::new("practice.generate").unwrap();
        let input = serde_json::json!({
            "workspace_id": "ws-1",
            "family_id": "problem.nonexistent",
        });

        let result = tauri::async_runtime::block_on(provider.invoke(&capability_id, 1, input));

        assert!(matches!(result, Err(InvocationError::InvalidInput { .. })));
    }

    #[test]
    fn generate_without_a_seed_still_succeeds_and_produces_a_persisted_attempt() {
        let provider = provider();
        let capability_id = CapabilityId::new("practice.generate").unwrap();
        let input = serde_json::json!({
            "workspace_id": "ws-1",
            "family_id": "problem.shell_y_poly",
        });

        let output = tauri::async_runtime::block_on(provider.invoke(&capability_id, 1, input))
            .unwrap();

        let attempt_id = output["attempt_id"].as_str().unwrap();
        assert!(provider.store.load_attempt(attempt_id, "ws-1").is_ok());
    }

    #[test]
    fn unknown_capability_id_is_rejected() {
        let provider = provider();
        let capability_id = CapabilityId::new("practice.other").unwrap();
        let result = tauri::async_runtime::block_on(provider.invoke(&capability_id, 1, Value::Null));
        assert!(matches!(result, Err(InvocationError::UnknownCapability { .. })));
    }
}
```

This references `store.connection_for_test()`, which doesn't exist yet — add it now as a small `#[cfg(test)]`-only accessor in `store.rs` (Task 5's file) so tests can seed a workspace row directly without a public non-test API leaking connection access:

```rust
#[cfg(test)]
impl PracticeStore {
    pub(crate) fn connection_for_test(&self) -> MutexGuard<'_, Connection> {
        self.connection().unwrap()
    }
}
```

- [ ] **Step 3: Run test to verify it fails**

Run: `cd src-tauri && cargo test practice::provider::tests`
Expected: FAIL to compile — `provider` module and `MANIFEST_TOML` not wired into `practice/mod.rs` yet.

- [ ] **Step 4: Wire the module in**

Modify `src-tauri/src/practice/mod.rs`:

```rust
mod error;
mod provider;
mod store;
mod types;

pub use error::PracticeError;
pub use provider::PracticeProvider;
pub use types::{
    AttemptStatus, EvaluateRequest, EvaluateResponse, GenerateRequest, GenerateResponse,
    HintRequest, HintResponse, ResponseValue,
};

/// The embedded first-party manifest for this module.
pub const MANIFEST_TOML: &str = include_str!("module.toml");

#[cfg(test)]
mod tests;
```

- [ ] **Step 5: Run test to verify it passes**

Run: `cd src-tauri && cargo test practice::provider::tests`
Expected: PASS (5 tests)

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/practice/module.toml src-tauri/src/practice/provider.rs src-tauri/src/practice/store.rs src-tauri/src/practice/mod.rs
git commit -m "feat(practice): add module manifest and practice.generate"
```

---

### Task 7: `practice.evaluate` — the inter-capability call to `math.verify`

**Files:**
- Modify: `src-tauri/src/practice/provider.rs`

**Interfaces:**
- Consumes: `crate::capabilities::math_verify::{VerifyRequest, VerifyResult}` (existing, exported), `crate::modules::{CapabilityCall, CallEnvelope, CapabilityRequirement, ModuleId}` (existing), `crate::knowledge::ResolvedSolution` (existing).
- Produces: `PracticeProvider::evaluate(&self, request: EvaluateRequest) -> Result<EvaluateResponse, PracticeError>`, and `("practice.evaluate", 1)` wired into `invoke`.

- [ ] **Step 1: Write the failing tests**

Add to `src-tauri/src/practice/provider.rs`, above the existing `impl PracticeProvider` block's closing brace (add these as new methods) and its `#[cfg(test)] mod tests`:

```rust
use crate::capabilities::math_verify::{MathVerifyProvider, VerifyRequest, VerifyResult};
use crate::knowledge::ResolvedSolution;
use crate::modules::{CallEnvelope, CapabilityCall, CapabilityRequirement, ModuleId};

use super::types::{EvaluateRequest, EvaluateResponse, ResponseValue};
```

(add these to the existing `use` block at the top of the file rather than duplicating imports)

```rust
impl PracticeProvider {
    // ... existing new()/handle_generate()/generate() stay as-is ...

    async fn handle_evaluate(&self, input: Value) -> Result<Value, InvocationError> {
        let request: EvaluateRequest = serde_json::from_value(input).map_err(|error| {
            InvocationError::InvalidInput {
                capability_id: capability_id("practice.evaluate"),
                message: error.to_string(),
            }
        })?;
        let response = self
            .evaluate(request)
            .await
            .map_err(|error| to_invocation_error("practice.evaluate", error))?;
        serde_json::to_value(response).map_err(|error| InvocationError::Failed {
            message: error.to_string(),
        })
    }

    async fn evaluate(&self, request: EvaluateRequest) -> Result<EvaluateResponse, PracticeError> {
        let attempt = self
            .store
            .load_attempt(&request.attempt_id, &request.workspace_id)?;
        if attempt.status == super::types::AttemptStatus::Solved {
            return Err(PracticeError::AlreadySolved {
                attempt_id: request.attempt_id.clone(),
            });
        }

        let verify_request = match (&attempt.instance.canonical_solution, &request.response) {
            (ResolvedSolution::Numeric(canonical_solution), ResponseValue::Numeric { value }) => {
                VerifyRequest::Numeric {
                    canonical_solution: *canonical_solution,
                    student_response: *value,
                }
            }
            (
                ResolvedSolution::Symbolic(canonical_solution),
                ResponseValue::SymbolicExpression { value },
            ) => VerifyRequest::SymbolicExpression {
                canonical_solution: canonical_solution.clone(),
                student_response: value.clone(),
            },
            _ => {
                return Err(PracticeError::ResponseTypeMismatch {
                    attempt_id: request.attempt_id.clone(),
                })
            }
        };

        let requirement = CapabilityRequirement {
            id: capability_id("math.verify"),
            min_version: 1,
        };
        // tauri::async_runtime::RwLock (tokio-backed): read()/write() are async fns
        // returning the guard directly, no poisoning/Result -- unlike std::sync::RwLock.
        let handle = {
            let registry = self.registry.read().await;
            registry
                .resolve(&self.installation, &requirement)
                .map_err(|error| PracticeError::VerificationFailed(error.to_string()))?
        };
        let call = CapabilityCall {
            envelope: CallEnvelope {
                workspace_id: request.workspace_id.clone(),
                capability_id: requirement.id.clone(),
                version: 1,
                calling_module_id: ModuleId::new("org.axiom.practice")
                    .expect("static module id is valid"),
            },
            input: verify_request,
        };
        let result: VerifyResult = {
            // Holding this guard across the .await below is exactly why `registry` must be
            // the async-aware RwLock, not std::sync::RwLock (see PracticeProvider's field
            // comment in Task 6) -- its guard is Send, so this compiles under async_trait's
            // Send-future requirement.
            let registry = self.registry.read().await;
            registry
                .invoke(&handle, &self.installation, call)
                .await
                .map_err(|error| PracticeError::VerificationFailed(error.to_string()))?
        };

        let response_json = serde_json::to_string(&request.response)
            .map_err(|error| PracticeError::Storage(error.to_string()))?;
        self.store
            .record_submission(&request.attempt_id, &response_json, result.is_correct)?;
        if result.is_correct {
            self.store.mark_solved(&request.attempt_id)?;
        }
        let submission_count = self.store.count_submissions(&request.attempt_id)?;

        Ok(EvaluateResponse {
            correct: result.is_correct,
            status: if result.is_correct {
                super::types::AttemptStatus::Solved
            } else {
                super::types::AttemptStatus::Open
            },
            submission_count,
        })
    }
}
```

Update the `CapabilityProvider::invoke` match to add `("practice.evaluate", 1) => self.handle_evaluate(input).await,`.

Add these tests to `mod tests`:

```rust
fn registry_with_math_verify_and_practice() -> (Arc<RwLock<ModuleRegistry>>, ModuleInstallation, PracticeProvider) {
    let math_verify_manifest =
        crate::modules::parse(crate::capabilities::math_verify::MANIFEST_TOML).unwrap();
    let registry = Arc::new(RwLock::new(ModuleRegistry::new())); // tauri::async_runtime::RwLock
    // blocking_write/blocking_read: this helper is a plain sync fn called from #[test]s,
    // not from inside an async task, so the tokio-backed lock's blocking variants apply
    // (the async .read()/.write() are used only from real async code paths, e.g. evaluate()).
    registry
        .blocking_write()
        .register(math_verify_manifest, Box::new(MathVerifyProvider))
        .unwrap();

    let store = PracticeStore::new(crate::db::open_in_memory().unwrap());
    store
        .connection_for_test()
        .execute(
            "INSERT INTO workspaces (id, name, guiding_goal_id, progress, paused)
             VALUES ('ws-1', 'Test', 'goal-1', 0.0, 0)",
            [],
        )
        .unwrap();
    let installation = ModuleInstallation {
        workspace_id: "ws-1".to_owned(),
        enabled_module_ids: vec![
            ModuleId::new("core.math_verify").unwrap(),
            ModuleId::new("org.axiom.practice").unwrap(),
        ],
    };
    let provider = PracticeProvider::new(
        store,
        fixture_package(),
        Arc::clone(&registry),
        installation.clone(),
    );
    (registry, installation, provider)
}

#[test]
fn evaluate_a_correct_response_solves_the_attempt_via_math_verify() {
    // Only math_verify is registered into the shared registry here -- `provider` itself
    // is called directly (not resolved through the registry), matching every other test
    // in this file. Task 9's round-trip test is the one that also registers `practice`
    // into the registry and resolves it from there.
    let (_registry, _installation, provider) = registry_with_math_verify_and_practice();

    let generate_input = serde_json::json!({
        "workspace_id": "ws-1",
        "family_id": "problem.shell_y_poly",
        "seed": 42,
    });
    let generated =
        tauri::async_runtime::block_on(provider.generate(
            serde_json::from_value(generate_input).unwrap(),
        ))
        .unwrap();

    let family = provider
        .knowledge_package
        .problem_families
        .iter()
        .find(|family| family.id.as_str() == "problem.shell_y_poly")
        .unwrap();
    let instance = crate::generation::generate_problem_instance(family, 42).unwrap();
    let ResolvedSolution::Symbolic(expression) = &instance.canonical_solution else {
        panic!("fixture family is symbolic");
    };
    let correct_value = mathcore::MathCore::new().calculate(expression).unwrap();

    let response = tauri::async_runtime::block_on(provider.evaluate(EvaluateRequest {
        workspace_id: "ws-1".to_owned(),
        attempt_id: generated.attempt_id,
        response: ResponseValue::SymbolicExpression {
            value: correct_value.to_string(),
        },
    }))
    .unwrap();

    assert!(response.correct);
    assert_eq!(response.status, super::types::AttemptStatus::Solved);
    assert_eq!(response.submission_count, 1);
}

#[test]
fn evaluate_an_incorrect_response_stays_open_and_counts_the_submission() {
    let (_registry, _installation, provider) = registry_with_math_verify_and_practice();
    let generated = tauri::async_runtime::block_on(provider.generate(GenerateRequest {
        workspace_id: "ws-1".to_owned(),
        family_id: "problem.shell_y_poly".to_owned(),
        seed: Some(42),
    }))
    .unwrap();

    let response = tauri::async_runtime::block_on(provider.evaluate(EvaluateRequest {
        workspace_id: "ws-1".to_owned(),
        attempt_id: generated.attempt_id,
        response: ResponseValue::SymbolicExpression {
            value: "0".to_owned(),
        },
    }))
    .unwrap();

    assert!(!response.correct);
    assert_eq!(response.status, super::types::AttemptStatus::Open);
    assert_eq!(response.submission_count, 1);
}

#[test]
fn evaluate_after_already_solved_is_rejected() {
    let (_registry, _installation, provider) = registry_with_math_verify_and_practice();
    let generated = tauri::async_runtime::block_on(provider.generate(GenerateRequest {
        workspace_id: "ws-1".to_owned(),
        family_id: "problem.shell_y_poly".to_owned(),
        seed: Some(42),
    }))
    .unwrap();
    let family = provider
        .knowledge_package
        .problem_families
        .iter()
        .find(|family| family.id.as_str() == "problem.shell_y_poly")
        .unwrap();
    let instance = crate::generation::generate_problem_instance(family, 42).unwrap();
    let ResolvedSolution::Symbolic(expression) = &instance.canonical_solution else {
        panic!("fixture family is symbolic");
    };
    let correct_value = mathcore::MathCore::new().calculate(expression).unwrap();
    tauri::async_runtime::block_on(provider.evaluate(EvaluateRequest {
        workspace_id: "ws-1".to_owned(),
        attempt_id: generated.attempt_id.clone(),
        response: ResponseValue::SymbolicExpression {
            value: correct_value.to_string(),
        },
    }))
    .unwrap();

    let result = tauri::async_runtime::block_on(provider.evaluate(EvaluateRequest {
        workspace_id: "ws-1".to_owned(),
        attempt_id: generated.attempt_id,
        response: ResponseValue::SymbolicExpression {
            value: correct_value.to_string(),
        },
    }));

    assert!(matches!(result, Err(PracticeError::AlreadySolved { .. })));
}

#[test]
fn evaluate_checks_the_stored_instance_not_a_caller_supplied_solution() {
    // The request has no way to pass a canonical_solution at all (EvaluateRequest has no
    // such field) -- this test documents that guarantee structurally, by confirming
    // evaluate() only ever needs attempt_id + response to reach a verdict.
    let (_registry, _installation, provider) = registry_with_math_verify_and_practice();
    let generated = tauri::async_runtime::block_on(provider.generate(GenerateRequest {
        workspace_id: "ws-1".to_owned(),
        family_id: "problem.shell_y_poly".to_owned(),
        seed: Some(1),
    }))
    .unwrap();

    let result = tauri::async_runtime::block_on(provider.evaluate(EvaluateRequest {
        workspace_id: "ws-1".to_owned(),
        attempt_id: generated.attempt_id,
        response: ResponseValue::SymbolicExpression {
            value: "0".to_owned(),
        },
    }));

    assert!(result.is_ok());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd src-tauri && cargo test practice::provider::tests`
Expected: FAIL to compile (`("practice.evaluate", 1)` arm not yet in `invoke`'s match, `evaluate`/`handle_evaluate` not yet defined) — or, once the code above is pasted in per Step 1, some tests fail on assertions instead (e.g. `AlreadySolved` not yet returned) until Step 3 lands fully. Confirm the specific compiler/assertion errors before proceeding.

- [ ] **Step 3: Confirm implementation matches Step 1**

The code in Step 1 already is the implementation (this task's TDD cycle is unusually front-loaded because the inter-capability call only makes sense written whole — see spec §4's reasoning for why resolve+invoke can't be meaningfully split into a smaller increment). Add the `("practice.evaluate", 1) => self.handle_evaluate(input).await,` arm to `invoke`'s `match`.

- [ ] **Step 4: Run test to verify it passes**

Run: `cd src-tauri && cargo test practice::provider::tests`
Expected: PASS (existing 5 + new 4 = 9 tests)

- [ ] **Step 5: Run full crate test suite to confirm no regression**

Run: `cd src-tauri && cargo test`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/practice/provider.rs
git commit -m "feat(practice): add practice.evaluate, calling math.verify through the registry"
```

---

### Task 8: `practice.hint`

**Files:**
- Modify: `src-tauri/src/practice/provider.rs`

**Interfaces:**
- Consumes: `HintRequest`/`HintResponse` (Task 3), `PracticeStore::increment_hints_revealed` (Task 5).
- Produces: `PracticeProvider::hint(&self, request: HintRequest) -> Result<HintResponse, PracticeError>`, `("practice.hint", 1)` wired into `invoke`.

- [ ] **Step 1: Write the failing tests**

Add to `mod tests` in `provider.rs`:

```rust
#[test]
fn hint_reveals_hints_in_order_and_tracks_the_count() {
    let (_registry, _installation, provider) = registry_with_math_verify_and_practice();
    let generated = tauri::async_runtime::block_on(provider.generate(GenerateRequest {
        workspace_id: "ws-1".to_owned(),
        family_id: "problem.shell_y_poly".to_owned(),
        seed: Some(42),
    }))
    .unwrap();
    assert!(generated.hints_total >= 1, "fixture family must have at least one hint");

    let first = tauri::async_runtime::block_on(provider.hint(HintRequest {
        workspace_id: "ws-1".to_owned(),
        attempt_id: generated.attempt_id.clone(),
    }))
    .unwrap();

    assert_eq!(first.hints_revealed, 1);
    assert_eq!(first.hints_total, generated.hints_total);
    assert!(!first.hint_text.is_empty());
}

#[test]
fn hint_past_the_total_is_no_more_hints() {
    let (_registry, _installation, provider) = registry_with_math_verify_and_practice();
    let generated = tauri::async_runtime::block_on(provider.generate(GenerateRequest {
        workspace_id: "ws-1".to_owned(),
        family_id: "problem.shell_y_poly".to_owned(),
        seed: Some(42),
    }))
    .unwrap();

    for _ in 0..generated.hints_total {
        tauri::async_runtime::block_on(provider.hint(HintRequest {
            workspace_id: "ws-1".to_owned(),
            attempt_id: generated.attempt_id.clone(),
        }))
        .unwrap();
    }

    let result = tauri::async_runtime::block_on(provider.hint(HintRequest {
        workspace_id: "ws-1".to_owned(),
        attempt_id: generated.attempt_id,
    }));

    assert!(matches!(result, Err(PracticeError::NoMoreHints { .. })));
}

#[test]
fn hint_response_never_exposes_the_full_hint_list() {
    let (_registry, _installation, provider) = registry_with_math_verify_and_practice();
    let generated = tauri::async_runtime::block_on(provider.generate(GenerateRequest {
        workspace_id: "ws-1".to_owned(),
        family_id: "problem.shell_y_poly".to_owned(),
        seed: Some(42),
    }))
    .unwrap();
    let capability_id = CapabilityId::new("practice.hint").unwrap();
    let input = serde_json::json!({
        "workspace_id": "ws-1",
        "attempt_id": generated.attempt_id,
    });

    let output = tauri::async_runtime::block_on(provider.invoke(&capability_id, 1, input))
        .unwrap();

    assert!(output.get("hints").is_none());
    assert!(output["hint_text"].is_string());
}

#[test]
fn hint_on_an_attempt_from_another_workspace_is_not_found() {
    let (_registry, _installation, provider) = registry_with_math_verify_and_practice();
    let generated = tauri::async_runtime::block_on(provider.generate(GenerateRequest {
        workspace_id: "ws-1".to_owned(),
        family_id: "problem.shell_y_poly".to_owned(),
        seed: Some(42),
    }))
    .unwrap();

    let result = tauri::async_runtime::block_on(provider.hint(HintRequest {
        workspace_id: "ws-other".to_owned(),
        attempt_id: generated.attempt_id,
    }));

    assert!(matches!(result, Err(PracticeError::AttemptNotFound { .. })));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd src-tauri && cargo test practice::provider::tests`
Expected: FAIL to compile — `hint`/`handle_hint` not defined, `("practice.hint", 1)` not in `invoke`'s match.

- [ ] **Step 3: Implement `hint`/`handle_hint`**

Add to the `impl PracticeProvider` block in `provider.rs`:

```rust
async fn handle_hint(&self, input: Value) -> Result<Value, InvocationError> {
    let request: HintRequest = serde_json::from_value(input).map_err(|error| {
        InvocationError::InvalidInput {
            capability_id: capability_id("practice.hint"),
            message: error.to_string(),
        }
    })?;
    let response = self
        .hint(request)
        .map_err(|error| to_invocation_error("practice.hint", error))?;
    serde_json::to_value(response).map_err(|error| InvocationError::Failed {
        message: error.to_string(),
    })
}

fn hint(&self, request: HintRequest) -> Result<HintResponse, PracticeError> {
    let attempt = self
        .store
        .load_attempt(&request.attempt_id, &request.workspace_id)?;
    let hints_total = attempt.instance.hints.len() as u32;
    if attempt.hints_revealed >= hints_total {
        return Err(PracticeError::NoMoreHints {
            attempt_id: request.attempt_id.clone(),
        });
    }

    let hints_revealed = self.store.increment_hints_revealed(&request.attempt_id)?;
    let hint_text = attempt.instance.hints[(hints_revealed - 1) as usize].clone();

    Ok(HintResponse {
        hint_text,
        hints_revealed,
        hints_total,
    })
}
```

Add `use super::types::{HintRequest, HintResponse};` to the file's imports. Add the `("practice.hint", 1) => self.handle_hint(input).await,` arm to `invoke`'s `match`.

- [ ] **Step 4: Run test to verify it passes**

Run: `cd src-tauri && cargo test practice::provider::tests`
Expected: PASS (9 existing + 4 new = 13 tests)

- [ ] **Step 5: Run full crate test suite and clippy**

Run: `cd src-tauri && cargo test && cargo clippy -p axiom_lib -- -D warnings`
Expected: PASS, zero warnings.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/practice/provider.rs
git commit -m "feat(practice): add practice.hint"
```

---

### Task 9: Full-registry round-trip test, quality gates, close out the task file

**Files:**
- Create: `src-tauri/src/practice/tests/mod.rs`
- Modify: `.ai/tasks/057-practice-core-utility.md`

**Interfaces:** None new — this composes everything from Tasks 2-8 into the one end-to-end proof the spec's §8 testing plan calls for, matching `math_verify`'s own `tests/mod.rs` round-trip pattern.

- [ ] **Step 1: Write the round-trip test**

Create `src-tauri/src/practice/tests/mod.rs`:

```rust
use std::sync::Arc;

use tauri::async_runtime::RwLock;

use crate::capabilities::math_verify::MathVerifyProvider;
use crate::modules::{
    parse, CapabilityId, CapabilityRequirement, ModuleId, ModuleInstallation, ModuleRegistry,
};

use super::store::PracticeStore;
use super::PracticeProvider;

fn fixture_package() -> crate::knowledge::KnowledgePackage {
    let fixture_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src/knowledge/tests/fixtures/canonical");
    crate::knowledge::load_knowledge_package(&fixture_root).unwrap()
}

#[test]
fn practice_manifest_parses_and_declares_math_verify_as_a_requirement() {
    let manifest = parse(super::MANIFEST_TOML).expect("module.toml must parse");
    assert_eq!(manifest.id.as_str(), "org.axiom.practice");
    assert!(manifest
        .requires
        .iter()
        .any(|requirement| requirement.id.as_str() == "math.verify" && requirement.min_version == 1));
    assert_eq!(manifest.provides.len(), 3);
}

#[test]
fn practice_resolves_math_verify_through_a_real_registry_end_to_end() {
    let registry = Arc::new(RwLock::new(ModuleRegistry::new())); // tauri::async_runtime::RwLock
    registry
        .blocking_write()
        .register(
            parse(crate::capabilities::math_verify::MANIFEST_TOML).unwrap(),
            Box::new(MathVerifyProvider),
        )
        .unwrap();

    let store = PracticeStore::new(crate::db::open_in_memory().unwrap());
    store
        .connection_for_test()
        .execute(
            "INSERT INTO workspaces (id, name, guiding_goal_id, progress, paused)
             VALUES ('ws-1', 'Test', 'goal-1', 0.0, 0)",
            [],
        )
        .unwrap();
    let installation = ModuleInstallation {
        workspace_id: "ws-1".to_owned(),
        enabled_module_ids: vec![
            ModuleId::new("core.math_verify").unwrap(),
            ModuleId::new("org.axiom.practice").unwrap(),
        ],
    };
    let provider = PracticeProvider::new(
        store,
        fixture_package(),
        Arc::clone(&registry),
        installation.clone(),
    );
    registry
        .blocking_write()
        .register(parse(super::MANIFEST_TOML).unwrap(), Box::new(provider))
        .unwrap();

    // Resolve practice.generate through the registry, the same way an eventual Tauri
    // command layer would -- not by calling the provider's Rust methods directly.
    let handle = registry
        .blocking_read()
        .resolve(
            &installation,
            &CapabilityRequirement {
                id: CapabilityId::new("practice.generate").unwrap(),
                min_version: 1,
            },
        )
        .expect("practice.generate must resolve");

    let call = crate::modules::CapabilityCall {
        envelope: crate::modules::CallEnvelope {
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
    .expect("practice.generate invocation must succeed");

    assert!(output["attempt_id"].is_string());
    assert!(output.get("canonical_solution").is_none());
}
```

- [ ] **Step 2: Run test to verify it fails, then passes**

Run: `cd src-tauri && cargo test practice::tests`
Expected: first confirm it fails to compile if `tests` isn't yet declared as `#[cfg(test)] mod tests;` in `mod.rs` (it was added in Task 6 Step 4 already — if so this should compile immediately); then `cargo test practice::tests` PASSes (2 tests).

- [ ] **Step 3: Run the entire workspace test suite and clippy**

Run: `cd src-tauri && cargo check && cargo test && cargo clippy -p axiom_lib -- -D warnings && cargo fmt --check`
Expected: all pass. Fix any `cargo fmt` drift with `cargo fmt` and re-commit.

- [ ] **Step 4: Update the task file**

Modify `.ai/tasks/057-practice-core-utility.md`, replacing the `## What was built / tested / left out` section:

```markdown
## What was built / tested / left out

Built: `src-tauri/src/practice/` (types, error, store, provider, module.toml), a new
migration (`practice_attempts`, `practice_submissions`), and `pub mod practice;` in
`lib.rs`. `practice.generate@1`, `practice.evaluate@1`, `practice.hint@1` registered on
the module-capability runtime; `practice.evaluate` resolves and invokes `math.verify`
through `Arc<RwLock<ModuleRegistry>>` rather than calling `math_verify`'s Rust code
directly (spec §4) -- the first real inter-capability call the runtime has carried.

Tested: `cargo test` across `practice::types`, `practice::error`, `practice::store`,
`practice::provider`, `practice::tests` (round-trip through a real registry with both
`math_verify` and `practice` registered) -- generation-matches-the-engine, hidden
canonical-solution/hints in every outward response, workspace isolation on all three
capabilities, multi-submission-until-solved, `AlreadySolved`/`NoMoreHints` edge cases,
attempt persistence surviving a fresh connection to the same on-disk database. Gates run:
`cargo check`, `cargo test`, `cargo clippy -p axiom_lib -- -D warnings`, `cargo fmt --check`
(all `src-tauri/` changes; no `src/` changes in this task, so the npm-side gates in
`.ai/quality-gates.md` don't apply).

Left out (per spec §1/§9, by design): any `#[tauri::command]` or frontend service
wiring, Study Session UI, adaptive family/difficulty selection, adaptive hint selection,
the network-disabled offline acceptance test (depends on Study Session UI existing first).
```

Set `status: review` in the frontmatter.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/practice/tests/ .ai/tasks/057-practice-core-utility.md
git commit -m "test(practice): add full-registry round-trip test; move task to review"
```
