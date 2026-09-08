# Study Session ↔ Practice attempt binding — design

## 1. Scope

`ROADMAP.md` "Remaining Stage 8 scope" names five things as Stage 8's remaining critical
path. This design covers the first: "lock how a study session selects a problem family and
resumes its current attempt." It is the prerequisite the other four depend on — wiring
Practice into `StudySessionPage` through a hook, polishing the learner-facing states against
`reference/UI/AXIOM-HANDOFF.md`, the permanent regression corpus, and the network-disabled
offline acceptance test are each their own brainstorm → spec → plan cycle, scoped separately,
and (per the roadmap) presentation-only with no engine/contract changes. This design is the
opposite: it is backend/schema work, deliberately, because it is what makes the
presentation-only work downstream possible without Core or the frontend reaching into
Practice's internals.

**Does not build:** any StudySessionPage UI, any frontend hook consuming these capabilities,
the "next problem" flow once an attempt is solved (the mechanism here supports it — see §4 —
but the UX decision is out of scope), the permanent regression corpus, or the offline
acceptance test.

## 2. Decisions carried in from brainstorming

- **One session, at most one current attempt at a time.** `Session.problemIndex`/
  `problemCount` track position in a larger set the learner is working through (across
  sessions or resumes), not a queue of attempts owned by one session record.
- **Family selection when a concept has more than one `ProblemFamily`: random, avoiding an
  immediate repeat of the same family this workspace just attempted.** Only one family
  (`problem.shell_y_poly`) exists in the bundled package today, so this policy has nothing to
  rotate among yet — implemented now anyway, since the resolution step it rides on (§3) has
  to exist regardless of family count, and building the real policy costs little more than a
  stub that would need revisiting the moment a second family lands.
- **This sub-project may change the backend** — a new nullable column on `sessions`, a new
  nullable column on `concepts`, and two new (additive) Practice capabilities. Practice's
  existing three capabilities (`practice.generate`/`practice.evaluate`/`practice.hint`) and
  the generation engine are untouched. The "presentation only, no engine/contract changes"
  constraint in `ROADMAP.md` scopes the *next* sub-project (`StudySessionPage` wiring), not
  this one.
- **A hidden gap found mid-brainstorm, folded in rather than deferred:** Core's `concepts`
  table (workspace-scoped, backs `ConceptsListPage`, arbitrary ids like `concept-shells`) and
  the Knowledge Package's `Concept` (global dotted slugs like `shell.method_vertical_axis`,
  which is what `ProblemFamily.concept_id` actually references) are two disjoint ID
  namespaces today, with no column anywhere linking them — even though a Core concept named
  "Shell method" and the Knowledge Package concept it clearly corresponds to both already
  exist. This design adds the missing crosswalk (§3) rather than treating family resolution
  as workable without it.
- **Split hydration from creation.** A session-start capability that both creates an attempt
  and returns its full display payload was considered and rejected in favor of two thin
  capabilities (§4): one that ensures an attempt exists, one that reads its current state.
  A freshly started session and a resumed one then hydrate through the identical read path —
  the frontend hook in the next sub-project has one code path to build, not two.
- **Practice-side failures during session start degrade to "no attempt bound," never to a
  failed session creation.** This mirrors `CORE.md`'s existing manifest-loading philosophy
  ("one broken manifest never blocks the rest of the bundle from registering") — a session is
  Core's own primary function and must not fail because Practice, a capability Core does not
  otherwise depend on, is unavailable or broken.

## 3. Data model

Two schema additions, both nullable, both opaque to the side that doesn't own them:

```sql
-- Core's own concepts table (0001_initial.sql)
ALTER TABLE concepts ADD COLUMN knowledge_concept_id TEXT;

-- Core's own sessions table (0001_initial.sql)
ALTER TABLE sessions ADD COLUMN current_attempt_id TEXT;
```

- `concepts.knowledge_concept_id` — the Knowledge Package concept slug this Core concept
  corresponds to (e.g. `"shell.method_vertical_axis"` for the `concept-shells` row), or
  `NULL` if this concept has no known Practice content. `NULL` is the default and expected
  state for nearly every row today — only one pairing exists to populate initially.
- `sessions.current_attempt_id` — the id of the `practice_attempts` row (see
  `0002_practice.sql`) this session is bound to, or `NULL` if none was bound (no mapped
  concept, Practice unavailable, or the concept has zero families). Core stores and returns
  this value but never interprets it — the same treatment `CORE.md` already gives capability
  handles ("opaque to callers").

Neither column carries a foreign-key constraint into the other schema's tables:
`knowledge_concept_id` has nothing to reference (Knowledge Package concepts are loaded
in-process from bundled files, not rows in Core's own database), and `current_attempt_id`
deliberately stays a plain, uninterpreted string on the Core side rather than a cross-schema
`REFERENCES practice_attempts(id)`, consistent with Core having no compile-time dependency on
the Practice crate module.

**Seed data:** wherever the sample workspace is populated (`commands/seed.rs`, task 041's
territory), set `knowledge_concept_id = 'shell.method_vertical_axis'` on the `concept-shells`
row. This is the one pairing that exists today.

## 4. New capability surface (Practice side)

Two new, additive capabilities. Both are new capability ids — neither is a change to
`practice.generate`, `practice.evaluate`, `practice.hint`, or anything in `src-tauri/src/generation/`.

### `practice.start`

Input:

```rust
struct StartRequest {
    workspace_id: String,
    concept_id: String, // a Knowledge Package concept slug, e.g. "shell.method_vertical_axis"
}
```

Behavior:

1. Look up every loaded `ProblemFamily` whose `concept_id` equals the request's `concept_id`.
2. Zero matches → return `StartResponse { attempt_id: None }`. Not an error — this is the
   expected outcome for every concept until more families exist.
3. Exactly one match → generate against it (reusing the same internal path
   `practice.generate` already uses — this is not new generation logic, just a new entry
   point into the existing one) and return its `attempt_id`.
4. More than one match → pick uniformly at random, excluding whichever of the candidate
   families produced this workspace's most-recently-created attempt (if any), then generate
   against the pick. Needs one new `PracticeStore` query: given a workspace id and a list of
   candidate family ids, the most recent attempt's `family_id` among them, if any — nothing
   in `store.rs` filters by a candidate family list today.

Output:

```rust
struct StartResponse {
    attempt_id: Option<String>,
}
```

Deliberately thin — no prompt, no hint count. `practice.start` runs at most once per session
(from Core's `start_session` orchestration, §5); everything needed to *display* an attempt
comes from `practice.describe`.

### `practice.describe`

Input:

```rust
struct DescribeRequest {
    workspace_id: String,
    attempt_id: String,
}
```

Output: the same shape `GenerateResponse` already carries, plus current status —

```rust
struct DescribeResponse {
    prompt: String,
    response_type: ResponseType,
    hints_total: u32,
    hints_revealed: u32,
    status: AttemptStatus,
    submission_count: u32,
}
```

Read-only, safe to call any number of times. Unknown `attempt_id` returns the existing
`PracticeError::AttemptNotFound` — no new error variant. This is the *only* hydration path a
future frontend hook needs: a freshly started session and a resumed one both call
`practice.describe` with whatever `current_attempt_id` the session carries, with no
resume-specific branch anywhere.

## 5. Orchestration flow

Entirely inside Core's existing `start_session` Rust command, in this order:

1. Load the session's Core `concepts` row. If `knowledge_concept_id` is `NULL`, skip to
   step 4 with no attempt.
2. `ModuleRegistry::resolve(workspace_id, "practice.start", ..)`. A `NoCompatibleProvider`
   result (Practice not enabled in this workspace) is not an error — treat it the same as
   step 1's skip. No new "is Practice available" check gets written; this reuses a resolution
   outcome the registry already defines.
3. If resolved, `invoke` it with `{workspace_id, concept_id: knowledge_concept_id}`, getting
   back `Option<attempt_id>`. Any invoke-time error here (as opposed to "no provider") also
   degrades to `None` per the failure policy in §2 — logged, not propagated as a
   `start_session` failure.
4. Insert the `sessions` row with `current_attempt_id` set to whatever steps 1–3 produced
   (`NULL` in every skip/failure case).

Generation happens *before* the session insert, and both the attempt and the session-row
reference to it land from a single `INSERT` — there is no two-phase state where a session
exists but its attempt-or-lack-of-one hasn't been decided yet. The only failure residue
possible is an attempt generated in step 3 whose subsequent session insert fails — an
orphaned, harmless `practice_attempts` row, the same class of imperfection already accepted
elsewhere in this schema (e.g. the temp-dir leaks noted in task 061's follow-ups), not a
correctness bug.

**Resuming needs no changes.** `resumeSession` already exists (`src-tauri/src/commands/session.rs`,
via `useSessions.ts`'s `resumeSession`) and only transitions `status: paused → active`. Once
the next sub-project's hook exists, it hydrates via `practice.describe` on every mount —
fresh or resumed — keyed off `current_attempt_id`. Nothing about resume is Practice-specific
at the Core layer.

**Explicitly deferred, not decided here:** what happens once the bound attempt is solved —
whether or how a "next problem" action replaces `current_attempt_id` mid-session. The
mechanism above (a re-callable `practice.start`, a settable `current_attempt_id` column)
supports that without redesign; the UX and whether it warrants a new Core command belongs to
the `StudySessionPage` wiring sub-project.

## 6. Error handling

One policy, applied uniformly (§2, §5): any Practice-side failure during session start —
unresolvable provider, a concept with zero matching families, or a genuine capability invoke
error — collapses to `current_attempt_id = NULL`. `start_session` never fails because of
Practice. This is a deliberate asymmetry: Core's own function (creating a session) must not
depend on a capability-providing module's health.

`practice.describe` against an unknown `attempt_id` is the one place this design surfaces a
real error, reusing `PracticeError::AttemptNotFound` — there is no scenario in which
`current_attempt_id` points at a row that doesn't exist under correct operation, so this is a
defensive check, not an expected path.

## 7. Testing

Matching this codebase's existing bar (10,000-seed property tests for generation, migration
conformance tests for schema):

- **Migration** — both new columns exist, default `NULL`, existing rows unaffected.
- **`practice.start`** — zero families → `None`; one family → generates and returns an id; a
  property test across many seeds confirming the rotation policy never immediately repeats
  the same family twice in a row for one workspace once more than one family exists (uses a
  multi-family test fixture, not the single bundled family).
- **`practice.describe`** — correct payload for an open attempt and for a solved one; unknown
  `attempt_id` → `AttemptNotFound`.
- **`start_session` orchestration** — mapped concept + enabled Practice → `current_attempt_id`
  matches a real `practice_attempts` row; unmapped concept → `NULL`; Practice module disabled
  → `NULL`, no error surfaced to the caller; a forced `practice.start` invoke error →
  `NULL`, `start_session` still succeeds.
- **Crosswalk regression guard** — `concept-shells`'s seeded `knowledge_concept_id` actually
  matches a concept slug present in the bundled knowledge package. Catches silent drift if
  either side's slug ever changes, rather than letting the mapping fail invisibly at runtime.

No frontend or UI test belongs in this design — those are the `StudySessionPage` wiring
sub-project's, and the regression corpus / offline acceptance test are their own
already-separated sub-projects per `ROADMAP.md`.

## 8. Follow-ups (out of scope here, tracked for later)

- **"Next problem" mid-session.** Deferred in §5 — needs its own small design once the
  learner-facing UX for a solved attempt is decided.
- **Crosswalk cardinality beyond 1:1.** This design assumes one Core concept maps to at most
  one Knowledge Package concept. If a Core concept ever needs to span multiple Knowledge
  Package concepts (or vice versa), `knowledge_concept_id` as a single nullable column stops
  being sufficient and would need a join table — not built now because no such case exists.
- **Family selection beyond "random, avoid immediate repeat."** If difficulty-aware or
  adaptive selection is wanted later, `practice.start`'s internal policy can change without
  touching its request/response contract or anything upstream of it.
