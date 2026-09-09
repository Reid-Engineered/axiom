# Study Session UI Integration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make `StudySessionPage` show and drive the Practice attempt that task 063 binds to a session — real prompt, typed answer, hints, and advancing to the next problem.

**Architecture:** Two thin Tauri commands expose task 063's existing `practice.describe` capability and add a `nextProblem` session mutation. One hook, `useAttempt`, owns the attempt lifecycle and is called only from the page. `ProblemPane` loses its hardcoded problem and renders four states off that hook. No capability contract and no generation engine changes.

**Tech Stack:** Rust (Tauri v2, rusqlite), TypeScript, React, Vitest + Testing Library, `@tauri-apps/api/mocks` for IPC in tests.

**Spec:** `docs/superpowers/specs/2026-09-09-study-session-ui-integration-design.md`

## Global Constraints

- **No capability-contract changes.** `practice.generate` / `practice.evaluate` / `practice.hint` / `practice.start` / `practice.describe` request and response shapes are frozen as task 063 left them. Nothing under `src-tauri/src/generation/` or `src-tauri/src/knowledge/` changes.
- **Hooks fetch, pages call hooks** (`ARCHITECTURE.md` §5 rule 1). No component may call a service directly; only `StudySessionPage` may call `useAttempt`.
- **No new global state** beyond `NavigationContext` / `WorkspaceContext` (`ARCHITECTURE.md` §5 rule 3).
- **All new types go in `src/types/*` and are re-exported from `src/types/index.ts`**; components import from `../types`, never a sibling file (`ARCHITECTURE.md` §4).
- **No hardcoded design values.** Every color, radius, shadow, and spacing value traces to `src/styles/tokens.css` (`AGENTS.md`).
- **Copy rules** (`reference/UI/AXIOM-HANDOFF.md` line 92): no exclamation marks, no emoji, no celebration. "Correct." not "Well done". Feedback names the misconception rather than saying only "incorrect". Numbers appear as context, never as scores.
- **Commit after every task.** Branch from `master`; do not commit to `master` directly.

---

### Task 1: `describeAttempt` Tauri command

Exposes task 063's `practice.describe` capability to the frontend. Without this the hook in Task 4 has nothing to call.

**Files:**
- Modify: `src-tauri/src/commands/practice.rs` (add input/output structs, handler, command)
- Modify: `src-tauri/src/lib.rs:65-67` (register the command in `invoke_handler`)
- Test: `src-tauri/src/commands/practice.rs` (its existing `#[cfg(test)] mod tests`)

**Interfaces:**
- Consumes: `crate::practice::{DescribeRequest, DescribeResponse}` — frozen, from task 063.
- Produces: `describe_attempt_handler(registry, installation, DescribeAttemptInput) -> CommandResult<AttemptDescription>`; the command is named `describeAttempt` over IPC and takes a single `input` argument.

- [ ] **Step 1: Write the failing test**

Add to the existing `mod tests` in `src-tauri/src/commands/practice.rs`. The helpers `registry_and_installation()` and `fixture_knowledge_package()` already exist in that module — reuse them rather than writing new ones.

```rust
#[test]
fn describe_attempt_returns_the_current_state_of_a_generated_attempt() {
    let (registry, installation) = registry_and_installation();
    let generated = tauri::async_runtime::block_on(generate_attempt_handler(
        &registry,
        &installation,
        GenerateAttemptInput {
            workspace_id: "ws-1".to_owned(),
            family_id: "problem.shell_y_poly".to_owned(),
        },
    ))
    .unwrap();

    let described = tauri::async_runtime::block_on(describe_attempt_handler(
        &registry,
        &installation,
        DescribeAttemptInput {
            workspace_id: "ws-1".to_owned(),
            attempt_id: generated.attempt_id.clone(),
        },
    ))
    .unwrap();

    assert_eq!(described.prompt, generated.prompt);
    assert_eq!(described.hints_total, generated.hints_total);
    assert_eq!(described.hints_revealed, 0);
    assert_eq!(described.status, AttemptStatus::Open);
    assert_eq!(described.submission_count, 0);
}

#[test]
fn describe_attempt_for_an_unknown_id_is_an_error() {
    let (registry, installation) = registry_and_installation();

    let result = tauri::async_runtime::block_on(describe_attempt_handler(
        &registry,
        &installation,
        DescribeAttemptInput {
            workspace_id: "ws-1".to_owned(),
            attempt_id: "attempt-missing".to_owned(),
        },
    ));

    assert!(result.is_err());
}
```

If `registry_and_installation()` does not exist under exactly that name in the test module, use whichever helper the neighbouring `generate_attempt` tests already use to build a registry — do not add a second one.

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path src-tauri/Cargo.toml describe_attempt`
Expected: FAIL — `cannot find function describe_attempt_handler` / `cannot find struct DescribeAttemptInput`.

- [ ] **Step 3: Write minimal implementation**

Add to `src-tauri/src/commands/practice.rs`, mirroring the `request_hint` pair exactly. Extend the existing `use crate::practice::{...}` import with `DescribeRequest, DescribeResponse`.

```rust
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DescribeAttemptInput {
    pub workspace_id: String,
    pub attempt_id: String,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AttemptDescription {
    pub prompt: String,
    pub response_type: ResponseType,
    pub hints_total: u32,
    pub hints_revealed: u32,
    pub status: AttemptStatus,
    pub submission_count: u32,
}

impl From<DescribeResponse> for AttemptDescription {
    fn from(response: DescribeResponse) -> Self {
        Self {
            prompt: response.prompt,
            response_type: response.response_type.into(),
            hints_total: response.hints_total,
            hints_revealed: response.hints_revealed,
            status: response.status.into(),
            submission_count: response.submission_count,
        }
    }
}

pub async fn describe_attempt_handler(
    registry: &Arc<RwLock<ModuleRegistry>>,
    installation: &ModuleInstallation,
    input: DescribeAttemptInput,
) -> CommandResult<AttemptDescription> {
    let response: DescribeResponse = invoke_practice(
        registry,
        installation,
        "practice.describe",
        input.workspace_id.clone(),
        DescribeRequest {
            workspace_id: input.workspace_id,
            attempt_id: input.attempt_id,
        },
    )
    .await
    .map_err(practice_error)?;
    Ok(response.into())
}

#[tauri::command(rename = "describeAttempt", rename_all = "camelCase")]
pub async fn describe_attempt(
    registry: State<'_, Arc<RwLock<ModuleRegistry>>>,
    installation: State<'_, ModuleInstallation>,
    input: DescribeAttemptInput,
) -> CommandResult<AttemptDescription> {
    describe_attempt_handler(&registry, &installation, input).await
}
```

`ResponseType` already exists in this file with a `From<crate::knowledge::ResponseType>` impl used by `Attempt` — reuse it; do not define a second one.

Then register the command in `src-tauri/src/lib.rs`, after `commands::practice::request_hint`:

```rust
            commands::practice::describe_attempt,
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --manifest-path src-tauri/Cargo.toml`
Expected: PASS, including both new tests. Total count should be 299 (297 before this task).

- [ ] **Step 5: Check formatting and lints**

Run: `cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check && cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`
Expected: both exit 0.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/commands/practice.rs src-tauri/src/lib.rs
git commit -m "feat(064): expose practice.describe as the describeAttempt command"
```

---

### Task 2: `nextProblem` Tauri command

Rebinds a session to a freshly generated attempt so a learner can advance past the first problem. Lives in `session.rs` because it writes Core's own `sessions` row.

**Files:**
- Modify: `src-tauri/src/commands/session.rs` (handler + command; reuses `start_practice_attempt` from task 063)
- Modify: `src-tauri/src/lib.rs` (register in `invoke_handler`)
- Test: `src-tauri/src/commands/tests.rs`

**Interfaces:**
- Consumes: `start_practice_attempt(registry, installation, workspace_id, concept_id) -> Option<String>` — already in `session.rs` from task 063, unchanged. Test helpers `map_concept_to_knowledge_package`, `practice_connection`, `fixture_knowledge_package`, `session_input` already exist in `commands/tests.rs` from task 063.
- Produces: `next_problem_handler(database, registry, installation, session_id) -> CommandResult<Session>`; IPC name `nextProblem`, argument `sessionId`.

- [ ] **Step 1: Write the failing test**

Add to `src-tauri/src/commands/tests.rs`:

```rust
#[test]
fn next_problem_rebinds_the_session_and_advances_the_counter() {
    let database = database();
    let workspace = create_workspace(&database);
    insert_concept(&database, &workspace.id, "concept-shells", "Shell method");
    map_concept_to_knowledge_package(&database, "concept-shells");
    let (registry, installation) = crate::commands::practice::build_practice_registry(
        fixture_knowledge_package(),
        practice_connection(&workspace.id),
    );
    let started = tauri::async_runtime::block_on(session::start_session_handler(
        &database,
        &registry,
        &installation,
        session_input(&workspace.id),
    ))
    .unwrap();
    let first_attempt = started.current_attempt_id.clone().unwrap();

    let advanced = tauri::async_runtime::block_on(session::next_problem_handler(
        &database,
        &registry,
        &installation,
        &started.id,
    ))
    .unwrap();

    assert_ne!(advanced.current_attempt_id, Some(first_attempt));
    assert!(advanced.current_attempt_id.is_some());
    assert_eq!(advanced.problem_index, Some(2));
}

#[test]
fn next_problem_leaves_the_counter_alone_when_practice_is_unavailable() {
    let database = database();
    let workspace = create_workspace(&database);
    insert_concept(&database, &workspace.id, "concept-shells", "Shell method");
    map_concept_to_knowledge_package(&database, "concept-shells");
    let registry = Arc::new(RwLock::new(ModuleRegistry::new()));
    let installation = ModuleInstallation {
        workspace_id: workspace.id.clone(),
        enabled_module_ids: Vec::new(),
    };
    let started = tauri::async_runtime::block_on(session::start_session_handler(
        &database,
        &registry,
        &installation,
        session_input(&workspace.id),
    ))
    .unwrap();

    let advanced = tauri::async_runtime::block_on(session::next_problem_handler(
        &database,
        &registry,
        &installation,
        &started.id,
    ))
    .unwrap();

    assert_eq!(advanced.current_attempt_id, None);
    assert_eq!(advanced.problem_index, started.problem_index);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path src-tauri/Cargo.toml next_problem`
Expected: FAIL — `cannot find function next_problem_handler in module session`.

- [ ] **Step 3: Write minimal implementation**

Add to `src-tauri/src/commands/session.rs`. Note the ordering rule inherited from task 063 §5: generate first, then write, so there is never a half-updated session row.

```rust
pub async fn next_problem_handler(
    database: &Database,
    registry: &Arc<RwLock<ModuleRegistry>>,
    installation: &ModuleInstallation,
    session_id: &str,
) -> CommandResult<Session> {
    let (workspace_id, knowledge_concept_id) = {
        let connection = database.connection()?;
        connection
            .query_row(
                "SELECT sessions.workspace_id, concepts.knowledge_concept_id
                 FROM sessions
                 JOIN concepts ON concepts.id = sessions.concept_id
                 WHERE sessions.id = ?1",
                [session_id],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?)),
            )
            .optional()
            .map_err(database_error)?
            .ok_or_else(|| format!("Session not found: {session_id}"))?
    };
    let attempt_id = match knowledge_concept_id {
        Some(concept_id) => {
            start_practice_attempt(registry, installation, &workspace_id, concept_id).await
        }
        None => None,
    };

    let connection = database.connection()?;
    if attempt_id.is_some() {
        connection
            .execute(
                "UPDATE sessions
                 SET current_attempt_id = ?2, problem_index = COALESCE(problem_index, 1) + 1
                 WHERE id = ?1",
                params![session_id, attempt_id],
            )
            .map_err(database_error)?;
    }
    load_session(&connection, session_id)?
        .ok_or_else(|| format!("Session not found: {session_id}"))
}

#[tauri::command(rename = "nextProblem")]
pub async fn next_problem(
    database: State<'_, Database>,
    registry: State<'_, Arc<RwLock<ModuleRegistry>>>,
    installation: State<'_, ModuleInstallation>,
    session_id: String,
) -> CommandResult<Session> {
    next_problem_handler(&database, &registry, &installation, &session_id).await
}
```

The `if attempt_id.is_some()` guard is what makes the second test pass: when Practice is unavailable the row is left entirely alone, so the counter never advances past content that does not exist.

Register in `src-tauri/src/lib.rs` next to the other session commands:

```rust
            commands::session::next_problem,
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --manifest-path src-tauri/Cargo.toml`
Expected: PASS, 301 tests total.

- [ ] **Step 5: Check formatting and lints**

Run: `cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check && cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`
Expected: both exit 0.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/commands/session.rs src-tauri/src/commands/tests.rs src-tauri/src/lib.rs
git commit -m "feat(064): add the nextProblem session command"
```

---

### Task 3: Frontend types, services, and mock backend

Gives Tasks 4 and 5 something to call, and teaches the test IPC mock about the two new commands.

**Files:**
- Modify: `src/types/practice.ts` (add `AttemptDescription`)
- Modify: `src/types/session.ts:28-47` (add `currentAttemptId`)
- Modify: `src/types/index.ts` (re-export `AttemptDescription`)
- Modify: `src/services/practiceService.ts` (add `describeAttempt`)
- Modify: `src/services/sessionService.ts` (add `nextProblem`)
- Modify: `src/test/mockBackend.ts` (bind an attempt on `startSession`; handle `describeAttempt` and `nextProblem`)
- Test: `src/services/practiceService.test.ts`

**Interfaces:**
- Produces: `AttemptDescription`; `describeAttempt(workspaceId: string, attemptId: string): Promise<AttemptDescription>`; `nextProblem(sessionId: string): Promise<Session>`; `Session.currentAttemptId?: string`.

- [ ] **Step 1: Write the failing test**

Add to `src/services/practiceService.test.ts`:

```ts
it('describes a generated attempt in its current state', async () => {
  const generated = await generateAttempt('workspace-calculus', 'problem.shell_y_poly');
  const described = await describeAttempt('workspace-calculus', generated.attemptId);

  expect(described.prompt).toBe(generated.prompt);
  expect(described.responseType).toBe(generated.responseType);
  expect(described.hintsTotal).toBe(generated.hintsTotal);
  expect(described.hintsRevealed).toBe(0);
  expect(described.status).toBe('open');
  expect(described.submissionCount).toBe(0);
});
```

Import `describeAttempt` alongside the existing imports at the top of that file.

- [ ] **Step 2: Run test to verify it fails**

Run: `npm test -- src/services/practiceService.test.ts`
Expected: FAIL — `describeAttempt is not a function`.

- [ ] **Step 3: Write minimal implementation**

`src/types/practice.ts` — append:

```ts
/** An attempt's current learner-facing state, the single hydration path for a session. */
export interface AttemptDescription {
  prompt: string;
  responseType: ResponseType;
  hintsTotal: number;
  hintsRevealed: number;
  status: AttemptStatus;
  submissionCount: number;
}
```

`src/types/session.ts` — add to `Session`, after `conceptId`:

```ts
  /** The Practice attempt bound to this session, if the concept has practice content. */
  currentAttemptId?: string;
```

`src/types/index.ts` — add `AttemptDescription` to the existing `export type { ... } from './practice'` list.

`src/services/practiceService.ts` — append, and add `AttemptDescription` to the type import:

```ts
export async function describeAttempt(
  workspaceId: string,
  attemptId: string,
): Promise<AttemptDescription> {
  return invoke<AttemptDescription>('describeAttempt', { input: { workspaceId, attemptId } });
}
```

`src/services/sessionService.ts` — append:

```ts
/** Binds the session to a freshly generated problem and advances its counter. */
export async function nextProblem(sessionId: string): Promise<Session> {
  return invoke<Session>('nextProblem', { sessionId });
}
```

`src/test/mockBackend.ts` — three changes.

First, a helper and a shared attempt factory. Add near `MOCK_PRACTICE_FAMILY`:

```ts
/** Mirrors the Rust seed's crosswalk: only the Shell method concept has practice content. */
const MAPPED_CONCEPT_NAMES = new Set(['Shell method']);

function createMockAttempt(): MockAttempt {
  const attempt: MockAttempt = {
    id: `attempt-${crypto.randomUUID()}`,
    prompt: MOCK_PRACTICE_FAMILY.prompt,
    responseType: MOCK_PRACTICE_FAMILY.responseType,
    hintTexts: [...MOCK_PRACTICE_FAMILY.hints],
    hintsRevealed: 0,
    status: 'open',
    submissionCount: 0,
  };
  mockAttempts.set(attempt.id, attempt);
  return attempt;
}
```

Second, in `case 'startSession'`, bind an attempt for a mapped concept. Replace the `const session: Session = {` literal's construction so it includes:

```ts
        currentAttemptId: MAPPED_CONCEPT_NAMES.has(concept.name)
          ? createMockAttempt().id
          : undefined,
        problemIndex: MAPPED_CONCEPT_NAMES.has(concept.name) ? 1 : undefined,
```

Third, two new cases before `default:`:

```ts
    case 'describeAttempt': {
      const input = parameters.input as { workspaceId: string; attemptId: string };
      const attempt = mockAttempts.get(input.attemptId);
      if (!attempt) throw new Error(`Attempt not found: ${input.attemptId}`);
      return {
        prompt: attempt.prompt,
        responseType: attempt.responseType,
        hintsTotal: attempt.hintTexts.length,
        hintsRevealed: attempt.hintsRevealed,
        status: attempt.status,
        submissionCount: attempt.submissionCount,
      };
    }
    case 'nextProblem': {
      const session = findSession(parameters.sessionId as string);
      const concept = findConcept(session.conceptId);
      if (MAPPED_CONCEPT_NAMES.has(concept.name)) {
        session.currentAttemptId = createMockAttempt().id;
        session.problemIndex = (session.problemIndex ?? 1) + 1;
      }
      return structuredClone(session);
    }
```

Also refactor `case 'generateAttempt'` to call `createMockAttempt()` rather than repeating the literal, so the two paths cannot drift.

- [ ] **Step 4: Run tests to verify they pass**

Run: `npm test -- src/services/practiceService.test.ts && npm run typecheck`
Expected: PASS and exit 0.

- [ ] **Step 5: Run the full frontend suite for regressions**

Run: `npm test && npm run lint`
Expected: PASS. The `startSession` change touches existing session tests — if any assert an exact session object, update them to accept the new optional fields rather than removing the binding.

- [ ] **Step 6: Commit**

```bash
git add src/types src/services src/test/mockBackend.ts
git commit -m "feat(064): add attempt description types, services, and mock IPC"
```

---

### Task 4: The `useAttempt` hook

The whole attempt lifecycle in one hook, called only from the page.

**Files:**
- Create: `src/hooks/useAttempt.ts`
- Test: `src/hooks/useAttempt.test.tsx`

**Interfaces:**
- Consumes: `describeAttempt`, `evaluateAttempt`, `requestHint` from `../services/practiceService`; `nextProblem` from `../services/sessionService`; `useAsyncResource` from `./useAsyncResource`.
- Produces: `useAttempt(session: Session | undefined, onSessionChange: (session: Session) => void)` returning `{ attempt, loading, error, answer, setAnswer, revealedHints, evaluation, check, hint, next }`.

- [ ] **Step 1: Write the failing test**

Create `src/hooks/useAttempt.test.tsx`. The mock backend's correct value is `42.7` and it authors two hints (`src/test/mockBackend.ts`).

```tsx
import { act, renderHook, waitFor } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import { startSession } from '../services/sessionService';
import type { Session } from '../types';
import { useAttempt } from './useAttempt';

async function boundSession(): Promise<Session> {
  return startSession({
    workspaceId: 'workspace-calculus',
    conceptId: 'concept-shells',
    intent: { activity: 'Practising', targetMinutes: 8 },
  });
}

describe('useAttempt', () => {
  it('describes the session-bound attempt on mount', async () => {
    const session = await boundSession();
    const { result } = renderHook(() => useAttempt(session, vi.fn()));

    await waitFor(() => expect(result.current.attempt).toBeDefined());
    expect(result.current.attempt?.status).toBe('open');
    expect(result.current.attempt?.hintsRevealed).toBe(0);
    expect(result.current.error).toBeNull();
  });

  it('stays empty and error-free when the session has no bound attempt', async () => {
    const session = { ...(await boundSession()), currentAttemptId: undefined };
    const { result } = renderHook(() => useAttempt(session, vi.fn()));

    await waitFor(() => expect(result.current.loading).toBe(false));
    expect(result.current.attempt).toBeUndefined();
    expect(result.current.error).toBeNull();
  });

  it('reveals the next hint when an answer is wrong', async () => {
    const session = await boundSession();
    const { result } = renderHook(() => useAttempt(session, vi.fn()));
    await waitFor(() => expect(result.current.attempt).toBeDefined());

    act(() => result.current.setAnswer('1'));
    await act(async () => {
      await result.current.check();
    });

    expect(result.current.evaluation?.correct).toBe(false);
    expect(result.current.revealedHints).toHaveLength(1);
    expect(result.current.attempt?.hintsRevealed).toBe(1);
  });

  it('stops revealing hints once they are exhausted', async () => {
    const session = await boundSession();
    const { result } = renderHook(() => useAttempt(session, vi.fn()));
    await waitFor(() => expect(result.current.attempt).toBeDefined());

    act(() => result.current.setAnswer('1'));
    for (let attempt = 0; attempt < 3; attempt += 1) {
      await act(async () => {
        await result.current.check();
      });
    }

    expect(result.current.revealedHints).toHaveLength(2);
    expect(result.current.error).toBeNull();
  });

  it('marks the attempt solved on a correct answer', async () => {
    const session = await boundSession();
    const { result } = renderHook(() => useAttempt(session, vi.fn()));
    await waitFor(() => expect(result.current.attempt).toBeDefined());

    act(() => result.current.setAnswer('42.7'));
    await act(async () => {
      await result.current.check();
    });

    expect(result.current.evaluation?.correct).toBe(true);
    expect(result.current.attempt?.status).toBe('solved');
  });

  it('rebinds and resets when advancing to the next problem', async () => {
    const session = await boundSession();
    const onSessionChange = vi.fn();
    const { result, rerender } = renderHook(
      ({ current }) => useAttempt(current, onSessionChange),
      { initialProps: { current: session } },
    );
    await waitFor(() => expect(result.current.attempt).toBeDefined());
    act(() => result.current.setAnswer('42.7'));
    await act(async () => {
      await result.current.check();
    });

    await act(async () => {
      await result.current.next();
    });
    const advanced = onSessionChange.mock.calls.at(-1)?.[0] as Session;
    expect(advanced.currentAttemptId).not.toBe(session.currentAttemptId);
    expect(advanced.problemIndex).toBe(2);

    rerender({ current: advanced });
    await waitFor(() => expect(result.current.attempt?.status).toBe('open'));
    expect(result.current.answer).toBe('');
    expect(result.current.revealedHints).toEqual([]);
    expect(result.current.evaluation).toBeUndefined();
  });

  it('surfaces a describe failure as an error rather than throwing', async () => {
    const session = { ...(await boundSession()), currentAttemptId: 'attempt-missing' };
    const { result } = renderHook(() => useAttempt(session, vi.fn()));

    await waitFor(() => expect(result.current.error).not.toBeNull());
    expect(result.current.attempt).toBeUndefined();
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `npm test -- src/hooks/useAttempt.test.tsx`
Expected: FAIL — cannot resolve `./useAttempt`.

- [ ] **Step 3: Write minimal implementation**

Create `src/hooks/useAttempt.ts`:

```ts
import { useCallback, useState } from 'react';

import { describeAttempt, evaluateAttempt, requestHint } from '../services/practiceService';
import { nextProblem } from '../services/sessionService';
import type { AttemptDescription, EvaluationResult, ResponseValue, Session } from '../types';
import { useAsyncResource } from './useAsyncResource';

/**
 * One session's Practice attempt: its current state, the learner's answer, revealed
 * hints, and advancing to the next problem. Called only from StudySessionPage.
 */
export function useAttempt(
  session: Session | undefined,
  onSessionChange: (session: Session) => void,
) {
  const workspaceId = session?.workspaceId;
  const attemptId = session?.currentAttemptId;
  const load = useCallback(
    async () =>
      workspaceId && attemptId ? describeAttempt(workspaceId, attemptId) : undefined,
    [attemptId, workspaceId],
  );
  const resource = useAsyncResource<AttemptDescription | undefined>(load);
  const { setData } = resource;
  const [answer, setAnswer] = useState('');
  const [revealedHints, setRevealedHints] = useState<string[]>([]);
  const [evaluation, setEvaluation] = useState<EvaluationResult>();

  const hint = useCallback(async () => {
    if (!workspaceId || !attemptId) return;
    const revealed = await requestHint(workspaceId, attemptId);
    setRevealedHints((hints) => [...hints, revealed.hintText]);
    setData((current) =>
      current ? { ...current, hintsRevealed: revealed.hintsRevealed } : current,
    );
  }, [attemptId, setData, workspaceId]);

  const check = useCallback(async () => {
    const attempt = resource.data;
    if (!workspaceId || !attemptId || !attempt) return;
    const response: ResponseValue =
      attempt.responseType === 'numeric'
        ? { responseType: 'numeric', value: Number(answer) }
        : { responseType: 'symbolic-expression', value: answer };
    const result = await evaluateAttempt(workspaceId, attemptId, response);
    setEvaluation(result);
    setData((current) =>
      current
        ? { ...current, status: result.status, submissionCount: result.submissionCount }
        : current,
    );
    if (!result.correct && attempt.hintsRevealed < attempt.hintsTotal) await hint();
  }, [answer, attemptId, hint, resource.data, setData, workspaceId]);

  const next = useCallback(async () => {
    if (!session) return;
    const advanced = await nextProblem(session.id);
    setAnswer('');
    setRevealedHints([]);
    setEvaluation(undefined);
    onSessionChange(advanced);
  }, [onSessionChange, session]);

  return {
    attempt: resource.data,
    loading: resource.loading,
    error: resource.error,
    answer,
    setAnswer,
    revealedHints,
    evaluation,
    check,
    hint,
    next,
  };
}
```

Note `check` reads `attempt.hintsRevealed` from the value captured before the evaluate call, which is why the exhausted-hints test passes: after two reveals `hintsRevealed === hintsTotal` and no third request is made.

- [ ] **Step 4: Run tests to verify they pass**

Run: `npm test -- src/hooks/useAttempt.test.tsx`
Expected: PASS, 7 tests.

- [ ] **Step 5: Typecheck and lint**

Run: `npm run typecheck && npm run lint`
Expected: both exit 0.

- [ ] **Step 6: Commit**

```bash
git add src/hooks/useAttempt.ts src/hooks/useAttempt.test.tsx
git commit -m "feat(064): add the useAttempt hook"
```

---

### Task 5: Rewire `ProblemPane` to the hook

Removes the last hardcoded problem content from the session screen and renders the four states.

**Files:**
- Modify: `src/pages/StudySessionPage.tsx:19-23` (delete `shellExpression`), `:146-180` (`ProblemPane`), and the `StudySessionPage` body to call the hook
- Modify: `src/pages/StudySessionPage.module.css` (classes for the answer field, feedback line, hint list)
- Test: `src/pages/StudySessionPage.test.tsx`

**Interfaces:**
- Consumes: `useAttempt` from Task 4; `useSession`'s existing `setData` for `onSessionChange`.

- [ ] **Step 1: Write the failing test**

Add to `src/pages/StudySessionPage.test.tsx` (create it if absent, following the render harness used by the other page tests):

```tsx
it('renders the bound attempt prompt and no problem total', async () => {
  const session = await startSession({
    workspaceId: 'workspace-calculus',
    conceptId: 'concept-shells',
    intent: { activity: 'Practising', targetMinutes: 8 },
  });
  render(<StudySessionPage sessionId={session.id} />);

  expect(await screen.findByText(/revolved about the y-axis/i)).toBeVisible();
  expect(screen.getByRole('textbox', { name: 'Answer' })).toBeVisible();
  expect(screen.queryByText(/of 1/)).not.toBeInTheDocument();
});

it('states plainly when a concept has no practice content', async () => {
  const session = await startSession({
    workspaceId: 'workspace-linear-algebra',
    conceptId: 'linear-concept-1',
    intent: { activity: 'Practising', targetMinutes: 8 },
  });
  render(<StudySessionPage sessionId={session.id} />);

  expect(await screen.findByText(/no practice content for this concept yet/i)).toBeVisible();
  expect(screen.queryByRole('textbox', { name: 'Answer' })).not.toBeInTheDocument();
});

it('offers a hint as the explanation when an answer is wrong', async () => {
  const session = await startSession({
    workspaceId: 'workspace-calculus',
    conceptId: 'concept-shells',
    intent: { activity: 'Practising', targetMinutes: 8 },
  });
  render(<StudySessionPage sessionId={session.id} />);
  const answer = await screen.findByRole('textbox', { name: 'Answer' });

  await userEvent.type(answer, '1');
  await userEvent.click(screen.getByRole('button', { name: 'Check' }));

  expect(await screen.findByText('Set up the shell method integral.')).toBeVisible();
});

it('shows a plain confirmation and a next problem action once solved', async () => {
  const session = await startSession({
    workspaceId: 'workspace-calculus',
    conceptId: 'concept-shells',
    intent: { activity: 'Practising', targetMinutes: 8 },
  });
  render(<StudySessionPage sessionId={session.id} />);
  const answer = await screen.findByRole('textbox', { name: 'Answer' });

  await userEvent.type(answer, '42.7');
  await userEvent.click(screen.getByRole('button', { name: 'Check' }));

  expect(await screen.findByText('Correct.')).toBeVisible();
  await userEvent.click(screen.getByRole('button', { name: 'Next problem' }));
  await waitFor(() => expect(screen.getByText('Problem 2')).toBeVisible());
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `npm test -- src/pages/StudySessionPage.test.tsx`
Expected: FAIL — no Answer textbox; the hardcoded prompt renders instead.

- [ ] **Step 3: Write minimal implementation**

In `StudySessionPage`, delete the module-level `shellExpression` constant and the `MathDisplay` import if it becomes unused. Call the hook in the page body, wiring `onSessionChange` to `useSession`'s `setData`:

```tsx
  const { session, loading, error, pauseSession, resumeSession, addTutorExchange, setData } =
    useSession(sessionId);
  const attempt = useAttempt(session, setData);
```

Pass it down: `problem={<ProblemPane session={session} attempt={attempt} working={working} onWorkingChange={setWorking} />}`.

Replace `ProblemPane` with:

```tsx
function ProblemPane({
  session,
  attempt,
  working,
  onWorkingChange,
}: {
  session: Session;
  attempt: ReturnType<typeof useAttempt>;
  working: string;
  onWorkingChange: (value: string) => void;
}) {
  const solved = attempt.attempt?.status === 'solved';
  const counter = attempt.attempt
    ? `Problem ${session.problemIndex ?? 1}`
    : `Problem ${session.problemIndex ?? 1} of ${session.problemCount ?? 1}`;

  return (
    <section className={styles.problemPane} aria-labelledby="problem-heading">
      <p className={styles.eyebrow} id="problem-heading">
        {counter}
      </p>
      {attempt.error ? (
        <p className={styles.state} role="status">
          {attempt.error.message}
        </p>
      ) : null}
      {!attempt.attempt && !attempt.loading && !attempt.error ? (
        <p className={styles.problemText}>
          There is no practice content for this concept yet.
        </p>
      ) : null}
      {attempt.attempt ? (
        <>
          <p className={styles.problemText}>{attempt.attempt.prompt}</p>
          <WorkingArea value={working} onChange={onWorkingChange} />
          <label className={styles.answerLabel}>
            <span className={styles.answerLabelText}>Answer</span>
            <input
              className={styles.answerInput}
              value={attempt.answer}
              inputMode={attempt.attempt.responseType === 'numeric' ? 'decimal' : 'text'}
              onChange={(event) => attempt.setAnswer(event.target.value)}
              onKeyDown={(event) => {
                if (event.key === 'Enter' && event.metaKey) void attempt.check();
              }}
            />
          </label>
          {attempt.evaluation && !attempt.evaluation.correct ? (
            <p className={styles.feedback}>That does not match yet.</p>
          ) : null}
          {solved ? <p className={styles.feedback}>Correct.</p> : null}
          {attempt.revealedHints.length ? (
            <ul className={styles.hintList}>
              {attempt.revealedHints.map((text) => (
                <li key={text} className={styles.hintItem}>
                  {text}
                </li>
              ))}
            </ul>
          ) : null}
          {attempt.evaluation &&
          !attempt.evaluation.correct &&
          attempt.attempt.hintsRevealed >= attempt.attempt.hintsTotal ? (
            <p className={styles.feedback}>No further hints remain for this problem.</p>
          ) : null}
          <div className={styles.problemActions}>
            {solved ? (
              <Button onClick={() => void attempt.next()}>Next problem</Button>
            ) : (
              <>
                <Button onClick={() => void attempt.check()}>Check</Button>
                <Button variant="secondary" onClick={() => void attempt.hint()}>
                  Hint
                </Button>
                <span className={styles.shortcutHint}>⌘↵ to check</span>
              </>
            )}
          </div>
        </>
      ) : null}
    </section>
  );
}
```

Add the three new classes to `StudySessionPage.module.css` using existing tokens only — no literal colors or spacings. Follow the neighbouring rules for exact token names:

```css
.answerLabel {
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
}
.answerLabelText {
  font-size: var(--font-size-eyebrow);
  color: var(--color-text-secondary);
}
.answerInput {
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  padding: var(--space-2);
  font: inherit;
}
.feedback {
  font-size: var(--font-size-body);
  color: var(--color-text-secondary);
}
.hintList {
  margin: 0;
  padding-left: var(--space-4);
  color: var(--color-text-secondary);
}
.hintItem {
  font-size: var(--font-size-body);
}
```

If any token name above does not exist in `src/styles/tokens.css`, use the nearest one that does — do not invent a literal value.

- [ ] **Step 4: Run tests to verify they pass**

Run: `npm test -- src/pages/StudySessionPage.test.tsx`
Expected: PASS, 4 tests.

- [ ] **Step 5: Run the full suite, typecheck, and lint**

Run: `npm test && npm run typecheck && npm run lint && npm run build`
Expected: all exit 0.

- [ ] **Step 6: Verify the token gate**

Run: the hardcoded-value grep from `.ai/quality-gates.md`
Expected: no matches in `StudySessionPage.module.css`.

- [ ] **Step 7: Commit**

```bash
git add src/pages/StudySessionPage.tsx src/pages/StudySessionPage.module.css src/pages/StudySessionPage.test.tsx
git commit -m "feat(064): drive the problem pane from the bound practice attempt"
```

---

### Task 6: Amend the roadmap and file the follow-ups

The spec deliberately widens a `ROADMAP.md` constraint and defers four things. Both need recording, or the next agent reads a roadmap that contradicts the merged code.

**Files:**
- Modify: `ROADMAP.md` ("Remaining Stage 8 scope" paragraph)
- Modify: `.ai/tasks/064-beta-checkpoint.md` (mark this sub-project done, per its Plan section)
- Create: `.ai/tasks/065-evaluate-diagnostics.md`, `.ai/tasks/066-structured-problem-expressions.md`

- [ ] **Step 1: Amend the roadmap constraint**

In `ROADMAP.md`'s "Remaining Stage 8 scope", replace "(Antigravity, presentation only — no engine/contract changes)" with "(presentation and its IPC surface — no engine or capability-contract changes)", and add one sentence recording why: task 063's capabilities had no Tauri command wrappers, so the frontend could not reach `practice.describe` without new IPC.

- [ ] **Step 2: File the two follow-up tasks**

Copy `.ai/tasks/TEMPLATE.md` for each, `status: proposed`, `stage: 8`, `depends_on: [064]`:

- **065 — diagnostic evidence in `practice.evaluate`.** `EvaluateResponse` carries no misconception text, so a wrong answer borrows a hint instead of naming what went wrong (spec §2). Changes a capability contract; needs its own brainstorm.
- **066 — structured problem expressions.** `ProblemInstance.prompt` is a plain string, so the mockup's typeset integral well with the highlighted `x` and its "Ask about x" affordance has no data behind it (spec §9). Schema change.

Also note in `064-beta-checkpoint.md`'s Plan section that the StudySessionPage sub-project is complete, leaving the regression corpus and the offline acceptance test.

- [ ] **Step 3: Commit**

```bash
git add ROADMAP.md .ai/tasks
git commit -m "docs(064): amend the Stage 8 UI constraint and file evaluate/expression follow-ups"
```

---

## Self-Review

**Spec coverage:** §3's two commands → Tasks 1 and 2. §4's types and services → Task 3. §5's hook → Task 4. §6's four states and the answer field → Task 5. §7's error handling → the `attempt.error` branch in Task 5 plus the describe-failure test in Task 4. §8's testing → Tasks 1, 2, 4, 5. §9's follow-ups → Task 6. §1's roadmap correction → Task 6. No gaps.

**Type consistency:** `AttemptDescription` carries the same six fields in Rust (Task 1), TypeScript (Task 3), and the mock backend (Task 3). `next_problem_handler` / `nextProblem` / `nextProblem(sessionId)` agree across Tasks 2, 3, and 4. `useAttempt(session, onSessionChange)` matches the spec's §5 signature and its Task 5 call site.

**Known risk:** Task 5's CSS token names are written from the neighbouring rules' conventions rather than verified against `src/styles/tokens.css`; the step says to substitute the nearest real token rather than invent a literal. Task 3's `startSession` change may require updating existing session tests that assert exact objects.
