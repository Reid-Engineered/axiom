# Study Session UI integration — design

## 1. Scope

`ROADMAP.md` "Remaining Stage 8 scope" names five things as Stage 8's remaining critical
path. Task 063 (merged) built the first: a study session resolves its concept's Knowledge
Package crosswalk, binds a generated attempt at creation, and exposes `practice.start` /
`practice.describe` on the capability runtime. This design covers the second: making that
binding visible and usable in `StudySessionPage` — describe the bound attempt, check a typed
answer, reveal hints, advance to the next problem.

**Does not build:** the visualization pane (stays inert until Stage 9's engine), the tutor
pane (stays mock exchanges — tutor/AI is Stage 9), persistence of the working-area scratch
text, the permanent Practice regression corpus, or the network-disabled offline acceptance
test. No change to `practice.generate` / `practice.evaluate` / `practice.hint`, to the
`ProblemFamily` / `ProblemInstance` schema, or to anything under `src-tauri/src/generation/`.

### A correction to the roadmap's constraint

`ROADMAP.md` scopes this sub-project as "Antigravity, presentation only — no engine/contract
changes". That constraint cannot hold literally, and this design deliberately widens it.

Task 063 registered `practice.start` and `practice.describe` as capabilities on the module
registry, but neither has a `#[tauri::command]` wrapper and neither appears in `lib.rs`'s
`invoke_handler` list — only `generateAttempt`, `evaluateAttempt`, and `requestHint` do. The
frontend therefore has no route to `practice.describe` at all. A hook consuming it requires
new IPC surface, which is a Core change however thin.

The constraint is restated, not abandoned: **no engine and no capability-contract changes.**
063's capability request/response shapes stay frozen; what this design adds is their exposure
to the frontend, plus one new Core command that writes Core's own session row. The
`ROADMAP.md` sentence should be amended to match when this lands.

## 2. Decisions carried in from brainstorming

- **Two new commands, not one.** `describeAttempt` alone would leave a learner stuck on the
  first problem forever. The "next problem" flow that 063 §5 explicitly deferred is folded in
  here, because the alternative is a beta whose practice loop has no second iteration.
- **An unbound session gets an honest empty state.** `concepts.knowledge_concept_id` is
  `NULL` for every concept except the sample's Shell method, so an unbound session is the
  *common* case today, not an edge case. The problem pane says plainly that there is no
  practice content for this concept yet rather than falling back to the hardcoded mock
  problem. This keeps the crosswalk gap visible during beta instead of hiding it behind
  content that looks real and whose Check button would have nothing to call.
- **A wrong answer reveals the next authored hint.** `EvaluateResponse` carries only
  `{correct, status, submission_count}` — no diagnostic text — while
  `reference/UI/AXIOM-HANDOFF.md` line 92 requires feedback to name the misconception and
  forbids a bare "incorrect", a rule `.ai/review-checklist.md` treats as blocking rather than
  cosmetic. Authored per-family hints are the only misconception-bearing text that exists
  today, so an incorrect check surfaces the next one. This costs the learner a hint they did
  not ask for; the alternative is shipping exactly the feedback the copy rule was written to
  prevent. Extending `practice.evaluate` with real diagnostics is the correct long-term fix
  and is filed as a follow-up (§8), not built here.
- **"Problem N", not "Problem N of M", once bound.** Seeded generation over a family has no
  finite problem count, so the mockup's "Problem 3 of 5" total is unrepresentable for real
  content. Unbound and mock sessions keep today's rendering.
- **A dedicated answer field.** `WorkingArea` is freeform scratch, but `evaluateAttempt`
  needs a typed `ResponseValue`. Parsing the learner's notes — the last line, or the whole
  box — silently reinterprets their working as a submission. One extra single-line control is
  a smaller cost than guessing on their behalf.

## 3. New IPC surface

Two commands, both registered in `lib.rs`'s `invoke_handler`.

### `describeAttempt` — `src-tauri/src/commands/practice.rs`

A pass-through to `practice.describe`, structurally identical to the three commands already
in that file: resolve the capability against the managed `ModuleRegistry` and
`ModuleInstallation`, invoke it with the same `core.tauri_commands` calling module id, return
the response. It belongs here rather than in `session.rs` because it touches no Core row.

```rust
#[tauri::command(rename = "describeAttempt", rename_all = "camelCase")]
// input: { workspaceId, attemptId } -> DescribeResponse
```

An unknown `attempt_id` surfaces `PracticeError::AttemptNotFound` unchanged — the existing
defensive path from 063 §6, not an expected one.

### `nextProblem` — `src-tauri/src/commands/session.rs`

Placed in `session.rs`, not `practice.rs`, because it writes Core's own `sessions` row and
can reuse the `start_practice_attempt` helper task 063 already added to that file. It:

1. Loads the session and its concept's `knowledge_concept_id`.
2. Calls `start_practice_attempt` (063's helper, unchanged) to generate a fresh attempt.
3. Updates `sessions.current_attempt_id` to the new id and increments `problem_index` in one
   statement.
4. Returns the updated `Session`, matching every other session mutation command.

063 §6's degrade policy applies in spirit but **not literally**, and the difference matters.
At session *start* there is no prior attempt, so "Practice unavailable" and
`current_attempt_id = NULL` are the same state. A rebind is different: the session already
has an attempt the learner may be part-way through. Setting it to `NULL` because generation
hiccupped once would discard their in-progress problem to report a transient failure.

So on a failed rebind — Practice unresolvable, disabled, or erroring — `nextProblem` changes
**nothing**: the existing `current_attempt_id` is left in place, `problem_index` does not
advance, and the command still succeeds. The learner keeps the problem they were on. On a
successful rebind both fields update together, so the counter never advances past content
that does not exist.

(Amended after review: this section originally said `current_attempt_id` becomes `NULL`,
carried over from 063 §6 without accounting for the prior-attempt case. The implementation
was correct and this text was not; corrected here rather than changing working code to match
a wrong spec. `next_problem_keeps_the_existing_attempt_when_practice_fails` in
`commands/tests.rs` pins the behaviour.)

## 4. Types and services

- `src/types/practice.ts` — add `AttemptDescription`: `prompt`, `responseType`, `hintsTotal`,
  `hintsRevealed`, `status`, `submissionCount`. Re-exported from `types/index.ts` per
  `ARCHITECTURE.md` §4.
- `src/types/session.ts` — add `currentAttemptId?: string` to `Session`. `commands/models.rs`
  has serialized this field since 063 while the TS type did not declare it; this closes that
  divergence, which was recorded as observation 4 in 063's review.
- `src/services/practiceService.ts` — add `describeAttempt(workspaceId, attemptId)`.
- `src/services/sessionService.ts` — add `nextProblem(sessionId)`, returning `Session`.

Both service functions are one-line `invoke` calls in the existing style.

## 5. The hook

One hook, `src/hooks/useAttempt.ts`, owning the attempt's full lifecycle. This follows
`useSession`, which already pairs a read with its mutations rather than splitting them, and
it builds on `useAsyncResource` for the `data` / `loading` / `error` / `refresh` shape rather
than re-implementing it.

```ts
export function useAttempt(
  session: Session | undefined,
  onSessionChange: (session: Session) => void,
): {
  attempt: AttemptDescription | undefined;
  loading: boolean;
  error: Error | null;
  answer: string;
  setAnswer: (value: string) => void;
  revealedHints: string[];
  evaluation: EvaluationResult | undefined;
  check: () => Promise<void>;
  hint: () => Promise<void>;
  next: () => Promise<void>;
};
```

- The loader resolves to `undefined` without an IPC call when `session?.currentAttemptId` is
  absent — the unbound case must not produce a spurious error.
- `check` builds a typed `ResponseValue` from `answer` and the attempt's `responseType`
  (`Number(answer)` for `numeric`, the raw string for `symbolic-expression`), calls
  `evaluateAttempt`, stores the result, and — when `correct` is false and
  `hintsRevealed < hintsTotal` — calls `hint` once so the wrong-answer path carries substance.
- `next` calls `nextProblem` and passes the returned session to `onSessionChange` — the page
  wires this to `useSession`'s existing `setData`, so the toolbar's counter and the hook's
  attempt never disagree — then re-describes; `answer`, `revealedHints`, and `evaluation`
  reset. The hook owns no session state of its own; `useSession` remains the single owner.
- Called only from `StudySessionPage`, never from a component, per `ARCHITECTURE.md` §5
  rule 1. No new context: the two globals `ARCHITECTURE.md` §5 rule 3 allows stay as they are.

## 6. Page changes and the four states

`ProblemPane` loses its hardcoded prompt string and the module-level `shellExpression`
constant, and takes the hook's values as props. A single-line **Answer** field sits below the
`WorkingArea`, typed by `responseType` (`inputMode="decimal"` for numeric). Check and Hint
gain real handlers; `⌘↵` submits, matching the existing "⌘⏎ to check" affordance. The working
area itself stays local component state — the learner's scratch, unpersisted, unchanged.

| State | Rendering |
|---|---|
| Bound, open | Real prompt as prose, answer field, live Check/Hint, "Problem N" with no total |
| Unbound (`currentAttemptId` null) | One line: no practice content for this concept yet. Working area and tutor stay usable; counter keeps today's "N of M" mock rendering |
| Wrong answer | Neutral line plus the next authored hint, auto-revealed. When `hintsRevealed === hintsTotal`, the neutral line stands alone and says no further hints remain |
| Solved | "Correct." — no celebration, per the handoff's copy rule — and a **Next problem** button calling `next` |

All copy follows `AXIOM-HANDOFF.md` line 92: no exclamation marks, no emoji, no scores.

## 7. Error handling

- **Describe fails** — the pane shows the error in the existing `styles.state` role="status"
  treatment the page already uses for session load failures. The session stays usable.
- **Check or hint fails** — an inline message via the page's existing `mutationError` channel,
  the same path `pauseSession` and `addTutorExchange` failures already take. The attempt is
  not discarded.
- **Next fails** — same inline channel. Because `nextProblem` degrades rather than erroring,
  the realistic outcome is a session that returns with `currentAttemptId` null, which renders
  as the unbound state rather than as an error.

Nothing here reaches the network; every path is local IPC to SQLite and in-process
generation.

## 8. Testing

- **`useAttempt`** — `renderHook` against `src/test/mockBackend.ts` covering all four states,
  the auto-hint-on-wrong-answer behavior, the exhausted-hints boundary, `next` resetting
  answer/hints/evaluation, and a describe failure surfacing as `error` rather than throwing.
- **`StudySessionPage`** — the unbound line renders when `currentAttemptId` is absent; a wrong
  check reveals a hint; a correct check shows "Correct." and a working Next problem button.
- **Rust** — `describeAttempt` round-trips a generated attempt; `nextProblem` rebinds to a new
  attempt id and increments `problem_index`; `nextProblem` with Practice disabled returns a
  session with `current_attempt_id` null, `problem_index` unchanged, and no error.

## 9. Follow-ups (out of scope here, tracked for later)

- **Diagnostic evidence in `practice.evaluate`.** The real fix for the copy-rule gap in §2 —
  a misconception field populated by the verifier, so a wrong answer names what went wrong
  instead of borrowing a hint. Its own sub-project; it changes a capability contract.
- **Structured problem expressions.** The mockup's typeset integral well with the highlighted
  `x` and its "Ask about x" affordance has no data behind it: `ProblemInstance.prompt` is a
  plain string. This design renders the prompt as prose and drops that well on the bound path.
  Restoring it needs a structured expression on `ProblemInstance` — a schema change, and the
  one visible departure from `15-system-refinements.png` this design accepts.
- **Crosswalk coverage.** Every concept but the sample's Shell method is unbound, so most
  beta sessions will show the §6 empty state. Widening the crosswalk is content work, not
  engineering, but it governs how much of the beta is actually exercisable.
- **Persisting the working area.** Scratch text is lost on navigation today. Out of scope
  here; worth deciding before beta feedback confuses it for a bug.
