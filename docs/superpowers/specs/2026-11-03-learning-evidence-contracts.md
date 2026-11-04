# Learning evidence and adaptive-session contracts

Date: 2026-11-03. Task: [083](../../../.ai/tasks/083-learning-evidence-contracts.md).
Status: approved contract (2026-11-04 human sign-off on P1–P7 as `policy.reid_default`
revision 1).

Authorities: [UI handoff](../../../reference/UI/AXIOM-HANDOFF.md),
[adoption](2026-10-10-reid-design-adoption.md), [Core](../../../CORE.md),
[architecture](../../../ARCHITECTURE.md), Roadmap Stage 9. The
[Knowledge evidence extension](2026-11-03-knowledge-evidence-extension.md) defines content
metadata. Subject policy belongs to providers and packages, never Core.

## 1. Inventory and design choice

Inventory inspected at base `66c19400f8bb6fa22180542a3b76ac0fd824cef5`:

- `src/types/common.ts`, `concept.ts`: named mastery, historical named decay and free-text
  diagnostics only. No numeric mastery, objective evidence, Bloom or readiness model.
- `practice.ts`: Attempt/AttemptDescription expose prompt, response type, hint counts,
  open/solved, and submission count. EvaluationResult has only correct/status/count.
  No operation key, partial result, checker trace or diagnostic provenance.
- `session.ts`: one concept, current attempt, active/paused/completed, target minutes,
  elapsedMinutes, optional problem position/count. No plan, queue, working draft or timer.
- `workspace.ts`, `goal.ts`: workspace identity, activity, Guiding goal and inferred deadline
  remain the domain/goal authority. Workspace.progress is not a new mastery source.
- `module.ts`: catalog/per-workspace presentation, not provider evidence identity.
  `visualization.ts` describes primitives, not verification evidence. `material.ts`,
  `note.ts` and `index.ts` need no new learning semantics. All type files were inspected;
  none defines the new engines. Older comments banning numeric presentation are historical.
- `src-tauri/src/practice/{types,provider,store,error}.rs`, `module.toml`: six @1
  capabilities (generate/evaluate/hint/start/describe/concepts). Attempts persist the full
  generated instance/seed and hint count; each evaluation inserts a new submission.
  Submission insertion and mark_solved are separate writes. Solved attempts reject checks;
  duplicate wrong checks are not deduplicated. Verifier error detail is not returned to UI.
- `knowledge/{types,raw,package,loader,validate,problem_family,objective,provenance}.rs`
  and conformance tests: schema 1, semantic package version, provenance refs, objective ids,
  difficulty range, generator/version, constraints, canonical solution, ordered hints.
  Strict raw parsers reject unknown fields. No misconception opportunities or Bloom tags.
- `capabilities/math_verify/{types,provider,error}.rs`: @1 returns is_correct/error.
  Numeric values and mathcore-calculated constant expressions compare with absolute and
  relative tolerance 1e-9. It does not establish symbolic identity, method, substitution,
  antiderivative-family completeness, or arbitrary-domain equivalence.
- `commands/{practice,session,concept,workspace,models,mod}.rs`: registry invocation and
  camelCase DTO boundary; session reuse/current-attempt binding and knowledge crosswalk.
  Current resume can advance a solved attempt; the new contract preserves its feedback
  until explicit advance. Other command domains (goal, material, module, note, seed) retain
  their existing ownership and must not derive learning evidence.
- `db/{mod,schema,tests}.rs`, migrations 0001–0004: Core domain rows, practice attempts and
  submissions, concept crosswalk, session binding/activity. Schema version is 4. No
  numeric snapshots, diagnostic events, policy revisions, receipts, plans, drafts or timing
  checkpoints. elapsed_minutes starts at zero and is not an authoritative measured timer.

Brainstorm alternatives: deriving scores in UI violates provenance; a general event bus
adds a new subsystem without solving atomic persistence. Choose a provider-owned evidence
aggregate with synchronous capability calls and transactional receipts. Core stores opaque
links and validates ownership, not subject classifications. No new global frontend store.
This narrows the transaction/storage design to the in-tree Practice aggregate; it does not
silently design the general module storage API, sandbox or event bus deferred in Core.

## 2. Wire conventions, owners and versioning

The shapes below are normative proposed payloads, not code added by this task. Rust wire
keys use snake_case; command DTOs convert mechanically to camelCase. Ids are opaque strings;
counts/revisions/durations are nonnegative safe integers, scores finite numbers in [0,1],
dates UTC RFC3339. Arrays have stable order. Optional values use explicit null; a missing
observation never becomes zero. Invalid values/references fail validation before mutation.

Every persisted aggregate carries this metadata, and each nested field inherits it unless
its owner is explicitly specified below:

```typescript
type Stamp = {
  schema_version: 1;
  workspace_id: string;
  provider: { module_id: string; module_version: string;
              capability_id: string; capability_version: number };
  policy_ref: { id: string; revision: number; digest: string } | null;
  content_ref: { package_id: string; package_version: string;
                 schema_version: number; digest: string } | null;
  evidence_ids: string[];
  recorded_at: string;
};
type Available<T> = { state: 'available'; value: T; stamp: Stamp }
  | { state: 'unavailable'; reason: 'no-evidence' | 'legacy-only'
      | 'unsupported' | 'provider-unavailable' | 'content-unavailable'
      | 'policy-unapproved'; last_known: { value: T; stamp: Stamp } | null };
```

Provider identity is taken from the resolved registry handle/manifest, not trusted client
input. Content digest covers canonical package files; policy digest covers canonical policy
JSON, both SHA-256 with canonicalization pinned by implementation fixtures. Provider stores
immutable content/policy snapshots with those digests; ids/versions alone cannot reproduce
an edited file. evidence_ids reference same-workspace immutable events. Core owns
workspace/session/goal ids and their existence; Practice owns stamps and all learning
conclusions; package authors own content_ref contents. Core does not interpret policy JSON.

Capability versions describe wire compatibility, policy revisions describe calculations,
package versions describe authored content, and SQLite migrations describe storage. None
substitutes for another. Breaking payloads require a capability major version. The current
registry resolves a minimum version by provider order; callers must also validate the
returned supported version and never deserialize an unknown major as a known one.
Historical snapshots retain their original provider/policy/content versions permanently
until workspace deletion. Switching provider starts a distinct evidence stream; no implicit
cross-provider score merge or retrospective reclassification.

## 3. Evidence, mastery, Bloom and readiness

Practice owns immutable Observation records (schema 1), originating in a committed check,
hint, skip, solution reveal, or explicit timer/policy event:

```typescript
type Observation = {
  id: string; stamp: Stamp; session_id: string | null; attempt_id: string | null;
  submission_id: string | null; ordinal: number;
  kind: 'check' | 'hint' | 'skip' | 'solution' | 'decay' | 'rule-change';
  result: 'correct' | 'incorrect' | 'partial' | 'unverifiable' | null;
  objective_ids: string[]; verification_id: string | null;
  hints_used: number; assisted: boolean;
};
type MasterySnapshot = {
  id: string; concept_id: string; objective_ids: string[];
  score: number; previous_snapshot_id: string | null;
  previous_score: number | null; delta: number | null;
  named_state: 'Developing' | 'Familiar' | 'Strong' | 'Mastered';
  as_of: string; last_practiced_at: string;
  decay: { from_snapshot_id: string; from_score: number;
           elapsed_ms: number; amount: number } | null;
};
type BloomEvidence = {
  demonstrated_level: 'remember' | 'understand' | 'apply' | 'analyze'
    | 'evaluate' | 'create'; objective_ids: string[];
  qualifying_attempt_ids: string[]; previous_level: string | null;
};
type Readiness = {
  state: 'ready' | 'not-ready' | 'unknown';
  prerequisites: { concept_id: string; snapshot_id: string | null;
                   state: 'ready' | 'not-ready' | 'unknown' }[];
  reason: string;
};
```

Each result is returned as Available<T>. Practice persists Observation and snapshots with
Stamp in the same transaction as the originating action. BloomEvidence uses package
objective tags plus qualifying attempts, not a score-to-Bloom conversion. Readiness uses
package prerequisite edges and referenced snapshots; it is not an exam prediction. The
provider persists its as-of projection so charts and plan explanations reference the same
evidence. Delta compares snapshots on the same policy/content stream only; first observation
has null previous/delta. Check/hint/skip/solution records require session and attempt ids;
decay and domain rule-change events have null attempt/submission ids and may have null
session ids. Ordinal is a workspace/provider-stream sequence, never a client timestamp.

No evidence means no score and no Bloom level. A fresh learner may retain the legacy named
New presentation alongside unavailable numeric evidence. Historical named mastery,
wasMasteryState, decayedAt, meaning, and free-text diagnostics remain historical text.
Never backfill numeric scores, pattern observations, clean counts, or Bloom from them.
Old correctness submissions remain reviewable but do not acquire missing provenance.
Goal edits retain history; no automatic score change. All calculations below are P1–P3.

## 4. Opportunity attribution and pattern resolution

An authored opportunity means a problem *could* test a pattern. It is distinct from a
supported observation that the learner exhibited or avoided it. Practice emits one final
attribution per attempt/pattern under a pinned rule revision:

```typescript
type Attribution = {
  id: string; attempt_id: string; pattern_id: string; rule_revision: number;
  opportunity_id: string; eligible: boolean;
  outcome: 'observed' | 'clean' | 'unknown' | 'not-applicable';
  evidence_ids: string[]; detector_id: string | null;
  detector_version: number | null; explanation: string;
};
type PatternSnapshot = {
  id: string; pattern_id: string; name: string;
  status: 'Recurring' | 'Improving' | 'Resolved' | 'Unknown';
  opportunities: number; occurrences: number; unknown_count: number;
  clean_streak: number; first_seen_at: string | null; last_seen_at: string | null;
  recent_attribution_ids: string[]; rule_revision: number;
  resolution: { required_clean: number; eligible_clean: number;
                explanation: string };
};
```

These provider-owned schema-1 records inherit Stamp and immutable persistence. The package
owns opportunity/detector definitions; the provider owns their execution and evidence links.
One attempt contributes at most one opportunity and occurrence per pattern. opportunities
counts finalized eligible attempts, including unknown; occurrences counts observed only.
Charts expose observed/clean/unknown denominators separately and must not imply all
non-occurrences were clean. Skipped/unassessed attempts are unknown if eligible. Ineligible
attempts are not-applicable and excluded. A matching diagnostic on any accepted submission
makes the attempt observed even if later corrected; corrections remain useful feedback but
cannot turn that attempt clean. Unknown never increments a streak.

An observed attribution can finalize immediately on a supported check and trigger one
replacement. Otherwise finalize on solved/skip/end; clean requires a solved unassisted
attempt with affirmative evidence coverage. Ending an open attempted question yields
unknown attribution; unpresented queue slots create no opportunities. Repeated checks
link to the existing finalized attribution, not another denominator entry. The UI may
show provisional diagnostic feedback but cannot count it until its receipt is committed.

Final-expression correctness cannot establish the learner's method or substitution. Clean
requires affirmative detector coverage of required evidence; absence of a match is not
enough. Current constant-value verification alone supports no method-attribution detector.
Partial success requires separately assessed objective evidence, otherwise unverifiable.

Rule editing validates a provider-supported rule, previews consequences, and persists a new
revision plus effective time. It never overwrites prior events/charts. Proposed P4 starts
a new streak at revision change; old statuses remain available by revision. Resolved
patterns retain history and can recur. Drill passes a pattern target to planning; unavailable
content returns content-unavailable, never an unrelated drill disguised as a match.

## 5. Capabilities and mutation boundary

Existing @1 capabilities remain unchanged. Introduce the following only through a reviewed
implementation: `practice.plan@1`, `practice.session@1`, `practice.evidence@1`,
`practice.patterns@1`, and `math.verify@2`. They use the existing serializable registry
boundary. A single session capability owns the Practice aggregate's action protocol so
evaluation cannot commit independently of its mastery, diagnostic and queue effects.

All read requests carry workspace_id. evidence additionally accepts concept_ids and as_of;
patterns accepts status/search, cursor and limit and returns snapshots plus next_cursor.
History is read via session action `read` with session_id or `list` with cursor/limit;
list returns session ids/status/start/end and Available<summary>. Cursors are opaque,
workspace-bound, ordered by recorded_at then id and rejected across domains.
Reads use separate read request variants, not mutation Action values. as_of cannot be in
the future and returns the stored snapshot at or before that instant, never a recalculated
historical score using today's policy. Patterns also supports read-detail {pattern_id,
rule_revision|null}, returning the rule, snapshots and paged attributions with their
attempt/feedback ids; charts use those persisted attributions.

Plan request: workspace_id, goal_id|null, concept_ids, target_ms|null, question_limit|null,
difficulty, pattern_id|null. Return Available<Plan>. Practice verifies all crosswalks and
goal scope. No Guiding goal means no invented deadline. Goal content is supplied through
the command boundary as a versioned immutable snapshot, not direct package access by Core.
Goal snapshot fields are id, text, state, inferred (the existing GoalInferredStructure),
and updated_at; Core owns and persists that source, Practice copies it into the plan.
Plan creation is a mutation with operation_id and request digest/receipt semantics below;
retries return the same plan id/seed choices. Read/preview operations do not create a plan.
All edit previews take the proposed payload/current revision and return its consequences
without committing. An explicit confirm sends the mutation with the previewed revision.

```typescript
type Action = {
  workspace_id: string; session_id: string; operation_id: string;
  expected_revision: number;
  action: 'start' | 'resume' | 'pause' | 'check' | 'hint' | 'skip' | 'advance'
    | 'save-working' | 'reveal-solution' | 'end' | 'edit-plan' | 'edit-rule';
  payload: object;
};
type Receipt = {
  operation_id: string; session_id: string; revision: number;
  event_ids: string[]; attempt: AttemptState | null;
  mastery_changes: Available<MasterySnapshot>[];
  bloom_changes: Available<BloomEvidence>[];
  pattern_changes: Available<PatternSnapshot>[];
  plan: Plan; timing: Available<Timing>; summary: Available<Summary>;
};
```

Payload discriminants are exact: start {plan_id, plan_revision}; resume/pause/end {};
check {attempt_id, response: ResponseValue}; hint {attempt_id, level};
skip/advance/reveal-solution {attempt_id}; save-working {attempt_id, draft_revision,
answer_text, working_text}; edit-plan {plan_revision, target_ms, question_limit, difficulty,
concept_ids}; edit-rule {pattern_id, rule_revision, required_clean}. All fields required,
nullable only where defined. Invalid payloads do not commit. ResponseValue retains numeric
and symbolic-expression variants from practice/types.rs; unsupported response types fail.
Rule edits use session_id for the invoking session, or a dedicated domain-scoped patterns
mutation with the same operation/revision receipt semantics when outside a session.
That patterns mutation carries workspace_id, operation_id, expected_revision (the pattern
aggregate revision), pattern_id, rule_revision and required_clean; its receipt returns the
new PatternSnapshot and rule revision. Start uses a client-allocated session UUID with
expected_revision=0; only that validated creation path may bind a previously absent session.
If another resumable session exists, return revision-conflict with its id for explicit
resume, rather than creating parallel work or returning a receipt for a different session.

Practice owns operation ids as client-supplied UUID deduplication keys, not evidence. A
unique (workspace_id, operation_id) receipt stores the canonical request digest and full
response. Same key/body returns the original receipt even after subsequent actions;
same key/different body returns operation-conflict. Check receipt lookup precedes revision
checking. New keys require the current revision; stale writes return revision-conflict.
No retry regenerates ids or creates another attempt. A deliberate new answer uses a new
key; identical answer on the same attempt with no intervening answer/hint/solution action
returns the existing check outcome with no new learning effect. Distinct retries remain
submission history but only the first assessable check affects mastery under P1.

Verifier/detector computation occurs outside the write transaction against immutable input.
Then one SQLite transaction rechecks workspace, revisions and receipt uniqueness; commits
submission, attempt status, observations, snapshots, queue revision, session current binding,
and receipt together. No await under a DB transaction. A crash yields all or none. Core
and Practice currently have separate connections: cross-connection sequential writes are
not atomic. Implementation must use one provider transaction connection to update its
aggregate and Core's opaque session binding via a narrow transaction helper; Core validates
the supplied ids and never computes learning policy. This is a scoped storage seam, not
permission for arbitrary module SQL. Uncommitted generated instances are discarded.

Errors carry code, retryable, message and current_revision|null: invalid-input, not-found
(including cross-workspace ids), unsupported-version, provider-unavailable,
content-unavailable, policy-unapproved, revision-conflict, operation-conflict,
budget-exhausted, attempt-closed, storage-failed. A verifier unable to assess a valid answer
returns unverifiable, never incorrect. No failed action applies mastery or queue effects.
Read failures retain last_known with its stamp and show unavailable; never silently reroute
an active attempt to a different provider. Provider removal permits history inspection,
not new grading. UI renders only committed receipts and offers a retry for transient errors.

## 6. Persisted plans, queue and exact resume

Practice owns schema-1 Plan and AttemptState, with Stamp and the referenced policy snapshot:

```typescript
type Plan = {
  id: string; revision: number; goal_id: string | null; goal_snapshot: object | null;
  target_ms: number | null; difficulty: 'Steady' | 'Push' | 'Exam';
  policy_ref: { id: string; revision: number; digest: string };
  phases: { id: string; name: 'Warm-up' | 'Stretch' | 'Combined';
            reason: string; evidence_ids: string[] }[];
  queue: { id: string; phase_id: string; concept_id: string; objective_ids: string[];
           family_id: string; seed: string; attempt_id: string | null;
           state: 'pending' | 'current' | 'correct' | 'miss' | 'partial' | 'skipped';
           reason: string; evidence_ids: string[]; replaces_id: string | null }[];
  changes: { event_id: string; removed_slot_id: string; added_slot_id: string;
             reason: string }[];
  availability: 'complete' | 'limited'; limitations: string[];
};
type AttemptState = {
  id: string; slot_id: string; status: 'open' | 'solved' | 'skipped';
  instance_ref: string; prompt: string; response_type: string;
  restrictions: string[]; bloom_target: string | null;
  draft_revision: number; answer_text: string; working_text: string;
  hints_total: number; hints_revealed: number; hint_budget: number;
  submission_ids: string[]; last_feedback_id: string | null;
};
```

Seeds serialize as decimal strings to avoid u64 loss in JavaScript. Family/seed/content
snapshot is pinned at plan creation; full instance, canonical solution and hint ladder are
stored before becoming current. Canonical answers remain backend-private until committed
feedback/solution reveal authorizes disclosure. Selection reason, watched patterns,
prerequisites, prior practice and policy disclosures are returned from these stored refs;
UI never reverse-engineers reasons. Position is queue order; question count is actual
queue length, never the requested count when content is short.

Start is explicit and reuses a resumable session for the same workspace; read/resume never
generate or advance. Correct/partial/miss feedback remains attached to the same question
until advance/skip. Save-working persists ordered drafts; stale draft revisions cannot
overwrite newer work. UI indicates saving and only promises the last acknowledged draft
after a crash. Pause, advance and end flush pending draft before transition. Domain
switching leaves the other session intact. End is idempotent and freezes a summary; no
silent generation. Miss review creates a new plan referencing original attempts, preserving
their answers and outcomes. Target duration is context, not an automatic deadline.

Planning uses only supported verified content. Missing goals/history permit content-based
practice with explicit no-history reasons; missing calibration yields policy-unapproved.
Limited content produces a shorter, editable plan or unavailable if no eligible family.
No invented Bloom progression/Combined phase if no multi-objective family supports it.
P6 defines ordering/replacement. Replacements touch pending slots only, retain prior slot
history and reasons, never overwrite current work, and cannot increase requested length.
Plan edits preview consequences and CAS revision on commit; difficulty changes affect only
pending slots. Existing attempts retain their disclosed costs and budgets.

## 7. Timing, verification and summaries

Task 076 owns timer implementation coordination with 085; its old score-ban question is
resolved by adoption. Core owns measured time; Practice consumes its acknowledged timing
snapshot without using speed as mastery evidence. Timing (schema 1) fields: elapsed_ms,
attempt_elapsed_ms, target_ms|null, checkpoint_sequence, active (boolean), measured_at,
recovered (boolean). Core persists checkpoint sequence/durations with each session action;
provenance is monotonic active-time deltas from the native process, not wall-clock
subtraction or client-asserted duration. P7 proposes checkpoint cadence and activity rules.
Pause/suspend/close exclude time; restart restores acknowledged totals and starts paused.
Clock changes do not alter elapsed time. Crash loss is disclosed, never filled with downtime.
Legacy elapsed_minutes is historical/unmeasured, not a measured zero.

`math.verify@2` request: workspace_id, attempt_id, response, canonical_solution,
response_type, restrictions, content_ref. Canonical data is supplied by Practice, never UI.
Verifier owns immutable Verification (schema 1): id, verdict (correct/incorrect/partial/
unverifiable), method (numeric-tolerance/constant-expression-numeric), checker_version,
absolute_tolerance, relative_tolerance, canonical_value|null, student_value|null,
restrictions, parse_error|null, limitations, objective_results (objective_id, verdict,
evidence), samples|null, terms|null. Each actual sample has bindings/expected/actual/error;
each term has expression/verdict/evidence. Unsupported detail is null, never synthesized.
Practice persists the result with provider/content Stamp and submission link atomically.

Retain @1's 1e-9 numeric tolerances in the @2 proposal; report actual method. The bundled
checker supports finite constant calculations only: samples and terms remain null, partial
is unsupported, domain identities and missing +C diagnoses are unverifiable. A parser
failure is an input explanation, not a misconception. Adding variable sampling, symbolic
proof, partial-credit detectors or altered tolerances requires a separately reviewed
verifier/content contract and regression fixtures. No numerical agreement is called proof.

Summary is provider-owned schema 1 with Stamp: session_id, completed_at,
timing: Available<Timing>, difficulty_history (slot_id/difficulty/policy_ref), correct,
assessed_total, attempted_total, skipped_total, unverifiable_total, hints_used,
mastery_changes: Available<MasterySnapshot>[], repeated_pattern_ids, question_rows
(slot_id/attempt_id/phase_id/prompt/result/feedback_id/before_snapshot_id|null/
after_snapshot_id|null), next_plan: Available<Plan>. Counts use unique attempts, not
submission count. assessed_total includes correct/incorrect/partial; attempted_total
includes any check, including unverifiable. Skips without checks are separate; categories
are disclosed rather than forced into a misleading correct/total. Hints count actual
distinct reveals. Summary and next recommendation reference stored events; absent fields
remain unavailable. A policy mix displays its history, not one misleading difficulty label.

## 8. Proposed policy values — human decisions P1–P7

These are engineering starting hypotheses, not calibrated learning-science claims. They
are provider policy data, absent from Core. Acceptance must pin all values/formulas in an
immutable policy revision with a human decision reference. No implementation may select a
different value silently. Alternative is to keep affected outputs unavailable until a
separate calibration task resolves them.

Practice owns PolicySnapshot (schema 1), persisted immutably: id, revision, digest,
approved_by, decision_ref, effective_at, mastery (trial_weight, partial_credit,
named_thresholds, minimum_mastered_trials, objective_aggregation), decay (half_life_ms,
projection_cadence), readiness (threshold), bloom (attempt_count, session_count,
unassisted_required), resolution (required_clean, editable_min, editable_max,
unknown_resets), difficulty (per-mode hint_budget, cost_per_hint, solution_credit),
planning (default_target_ms, default_question_limit, question_limit_range,
target_ms_range, phase_shares, replacement_limit, ordering), and timing_policy_ref.
These fields encode P1–P7 exactly if approved; reject incomplete or unapproved policies.
Core owns the separate timing policy snapshot/cadence referenced by timing_policy_ref;
Practice stores that opaque reference for reproducible summaries. No learner-controlled
edit can change approval metadata or silently alter another policy field.

- **P1 mastery update and thresholds:** per objective keep weighted assessed successes /
  weighted assessed trials, with no pseudo-count or initial score. First assessable check
  per attempt has weight 1; correct=1, incorrect=0, partial=0.5 only with supported objective
  assessment; unverifiable/skip has no trial. Concept score is equal-weight mean of scoped
  assessable objective scores, unavailable until every scoped assessable objective has
  evidence (unassessable authored objectives are disclosed in limitations). Developing
  [0,0.4), Familiar [0.4,0.7), Strong [0.7,0.9), Mastered [0.9,1] requires at least 5
  assessed attempts per scoped objective; otherwise cap named state at Strong. Rationale:
  auditable observed ratio, limited resistance to one-attempt mastery; no legacy conversion.
- **P2 decay/readiness:** propose exponential decay of the observed score with 30-day
  half-life since last assessed practice; at a new trial discount prior trial weights by
  the elapsed decay factor before adding the new trial and restarting the decay anchor
  (so a miss after decay cannot raise the displayed score). Persist one as-of decay projection per UTC day on
  first evidence read, keyed by concept/policy/day; same-day reads reuse it. Readiness
  requires every prerequisite's available decayed score >=0.7; no prerequisites is ready,
  any missing evidence is unknown unless another prerequisite is already not-ready.
  Rationale: explicit reversible projection, no background scheduler; half-life needs human
  calibration. It is not an exam readiness probability.
- **P3 Bloom:** require 3 first-check correct, unassisted distinct attempts on authored
  objectives at the same level across at least 2 sessions before demonstrating that level.
  Highest supported qualifying level is reported; no claim about untested lower levels.
  No automatic Bloom decay. Rationale: tags alone describe question demand, not ability;
  multiple sessions reduce immediate retry inflation.
- **P4 pattern rule:** 5 consecutive eligible clean attempts to resolve (editable integer
  1–20); observed resets streak to zero and becomes Recurring; 1–4 clean after a recorded
  occurrence is Improving; five is Resolved. No observed history means Unknown, not
  Resolved. Unknown interrupts the streak without adding clean evidence; not-applicable
  leaves it unchanged. An occurrence after resolution reopens Recurring. Edits start a
  new streak with no retroactive reclassification. Rationale: follows the reference's
  five-attempt candidate while making reset and unknown behavior explicit.
- **P5 difficulty budgets/costs:** Steady allows min(3, available hints), Push min(1,
  available hints), Exam 0; effective budgets are disclosed before start. Correct evidence
  credit is max(0, base_credit - cost_per_hint * distinct_hints_before_first_check), with
  base_credit 1 (or supported partial 0.5). Per-hint costs: Steady 0, Push 0.05, Exam 0
  because hints are forbidden. Incorrect credit 0; no extra negative score debit for a miss
  or skip. Solution reveal before assessment gives no trial and no clean credit; any hint
  prevents clean/Bloom qualification. Hints after first check cannot increase credit or
  alter an already committed score. Rationale: disclosed evidence discount rather than
  repeatedly charging arbitrary mastery points; Exam changes access, not punishment.
  This intentionally proposes a meaning for the reference's 0.05 rather than treating it
  as an adopted rule. Policy must disclose that distinction to the learner.
- **P6 planning/adaptation:** propose default 20 minutes, at most 10 questions (learner may
  edit 1–50, target 1–180 minutes). Allocate 30% Warm-up, 40% Stretch, remainder Combined,
  rounding first two down, omitting unsupported phases. Warm-up orders available ready
  concepts by oldest practice, Stretch by lowest supported score, Combined only verified
  multi-objective families; ties sort by concept/family id. No-history candidates sort by
  id. Seed selection is random once then persisted. Steady selects lowest supported family
  difficulty, Push next higher supported tier (or discloses same-tier fallback), Exam highest
  supported tier. On first observed miss per current slot, replace at most the next pending
  slot with a supported pattern opportunity at the same difficulty; choose a different
  family if available, otherwise a new seed with the limitation disclosed. No content means
  no replacement and an explicit reason. Rationale: bounded explainable adaptation; the
  current single family cannot promise diverse phases, diagnosis or increased difficulty.
- **P7 timing:** count active foreground session time including feedback/tools; exclude
  background, pause, suspend and closed app time. Checkpoint every 5 seconds and before
  action/transition; maximum unacknowledged crash loss is 5 seconds during normal operation,
  otherwise show last checkpoint and recovery notice. No auto-end at target. Rationale:
  predictable context without scoring speed; task 076/085 must validate platform events.

## 9. Migration, implementation plan and conformance gates

No runtime/schema/type edits in 083. Implementation order after policy approval:

1. Add typed payloads in `practice/types.rs`, command DTOs, `src/types/practice.ts` and
   `session.ts` with barrel exports; preserve @1 consumers. Add provider policy validation
   and package extension loader per the companion spec. Register only supported majors.
2. Add the next unused SQL migration after current 0004 (coordinate concurrent migrations).
   Introduce immutable policy/content snapshots, observations, verification/attribution,
   mastery/Bloom/pattern snapshots, plans/queue revisions, working drafts and receipts;
   Core owns session timing checkpoints and opaque plan/provider bindings. Use same-workspace
   composite references, receipt unique keys and revision CAS. Package ids alone are not
   workspace keys. Foreign keys cascade workspace deletion; no global learner history.
   Preserve all old rows. New evidence is absent for legacy records, no blanket backfill.
3. Implement transactional session actions and narrow Core binding helper; then matching
   commands/services/hooks, and pages in 084–086. Each new command must be wired and covered
   by the IPC registration guard. No UI-local calculations and no new global store.
4. 076 implements timing with 085 against section 7/P7; 086 consumes immutable profile and
   summary projections. Keep 067's permanent full-instance seed corpus and 068's verified
   network-disabled native loop as outstanding Stage 8 work, not replaced by this spec or
   ordinary CI. Missing generators, detector coverage and richer verification need scoped
   content/implementation work; UI must remain honest in their absence.

Required future conformance cases (tests to write, **not tests run by this docs task**):

- Registry/DTO fixtures for each action, Available variant, malformed enum/range/version,
  module/policy/content change and all unsupported @1/@2 combinations.
- Rust file-backed migration from schema 4 and fresh DB; preserve legacy named states,
  zero fabricated evidence; simulate failure at each write boundary and restart. Race
  duplicate checks/hints/advance/end; one receipt/effect; same key with different payload
  conflicts; new key identical answer does not inflate history; stale revisions fail.
- Correct, miss, supported partial, parse failure and unverifiable response; finite tolerance
  boundaries 1e-9, nonfinite input, constant agreement without symbolic-proof claims,
  unknown method attribution, no invented term/sample traces.
- Eligible observed vs clean vs unknown vs ineligible; repeated checks count once; hints,
  solution reveal, skip, recurrence and rule revision; chart denominator honesty; P1–P5
  thresholds and weights at boundaries; no-evidence objective, mixed versions and decay
  repeated reads, clock rollback/forward.
- Domain isolation for all reads, receipts, draft writes, pattern edits, queue references
  and cursors; overlapping identical package concepts across two workspaces.
- No eligible family, one family, unavailable provider, exhausted hints, missing Bloom or
  detector support, partial plan, plan edit, failed replacement and multiple misses; current
  attempt/working/feedback never replaced before explicit advance.
- Native restart at prompt, draft acknowledgment, feedback, hint and replacement; pause,
  background/suspend, timer checkpoint crash and target overrun. Summary uses unique
  attempts and original versions; goal edit preserves history and active plan snapshot.
- Hooks through existing mock IPC fixtures, conditional component rendering, and native
  flows per Stage 9 acceptance. 067 must pin unchanged generated instances across the
  metadata migration; 068 must prove external network isolation and observe a deliberate
  offline break fail while preserving WebDriver loopback.

Docs-task validation is reference/coverage review plus git diff checks. Full frontend,
Rust and Linux E2E CI remains required before merge; no local runtime pass is claimed here.
