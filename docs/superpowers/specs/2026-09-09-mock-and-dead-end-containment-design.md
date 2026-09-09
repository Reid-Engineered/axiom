# Mock and dead-end containment — design

## 1. Scope

The Windows release-build walk of `064` proved the Practice loop works and, in the same
pass, proved the rendered app mixes two incompatible sources of truth. A real generated
`shell_y_poly` problem evaluates, hints, and advances correctly; alongside it, Home
advertises "Calculus II — Shell method · Problem 6 of 12" with a "Resume session" button
that opens a screen reading "There is no practice content for this concept yet."

This design has two parts:

- **§6 — the beta-critical path.** Six tasks: active-session selection and reuse, sample
  seed correction, workspace curriculum provisioning, startup restoration, Study Session
  contradictory-state removal, and one native regression across two restarts. This is the
  work `064`'s criteria 1, 2 and 6 wait on, and there are **no unresolved decisions left on
  it** — every question this design raised has been answered and folded in below.
- **§7 — separately approved follow-ups.** Findings that are real but are not consequences
  of the walk: real elapsed-time tracking, and the application-wide affordance purge. They
  stay inventoried and stay out of the beta path.

**Criteria 1, 2 and 6 remain unmet until all six tasks in §6 land and `074`'s native
regression passes.** No subset of them settles a criterion, and no criterion is claimed on
the strength of a passing unit suite. `064`'s status table should carry them as unmet until
that regression is green on a real build.

**Does not build:** Stage 9 scope in any form — no real tutor, no 3D visualization engine,
no marketplace backend, no module sandboxing, no additional problem families, no offline
sync, and not the general `knowledge.query@1` capability. It also does not build `067`
(regression corpus) or `068` (offline acceptance test), which remain their own tasks; §6.6
adds rendered coverage neither provides nor was scoped to.

**Does not remove** the "Explore a sample workspace" feature. Sample data stays. What
changes is what the sample is allowed to contain.

**Does not add** a route-persistence mechanism. No `app_state` table, no `localStorage`, no
stored UI position. Startup restoration is derived from domain state (§6.4).

## 2. The rule this design reduces to

> **The rendered app must never present, as the learner's own history, anything the
> learner did not do.**

Three mechanisms enforce it:

1. **Seed curriculum, never activity.** Concepts, goals, material, and the module catalog
   are *content* — they can legitimately ship pre-authored. Sessions, tutor exchanges,
   diagnostics, notes, mastery states, and progress are *records of activity* and may only
   exist because something happened.
2. **"The active session" must mean the session the learner is actually in.** Today it
   means the lowest `rowid` in the table.
3. **Unbacked UI is removed when it is an affordance, and kept only when it is visibly
   inert chrome.** No generic fallback copy substituted for a missing destination.

The rule is deliberately narrower than "remove all mock data." A pre-authored Calculus II
curriculum is honest. A pre-authored claim that the learner reached problem 6 of 12 is not.

Mechanism 3 is also where the beta path stops. Applying it exhaustively across every screen
is §7.2 — the walk found dead ends in the practice loop, and that is what the beta gate is
about. A §7.2 item is pulled forward only if a beta regression in §6.6 actually reaches it.

## 3. Source-of-truth, before and after

**Today.** Two paths reach `StudySessionPage`, and only one of them works. A learner-created
workspace reaches neither, because it has no concepts at all.

```
 src/services/mockData/*
   3 workspaces · 87 concepts · 4 sessions (40 fabricated tutor exchanges)
   goals · modules · material · notes · activity events
        │
        │ sampleWorkspaceService.ts ships the WHOLE fixture set as `seed`
        ▼
 importSampleWorkspace (Rust)  ──►  SQLite  ◄── create_workspace
                                     │           workspace + goal + offline rows
        seeded and learner rows are  │           NO CONCEPTS ── criterion 1 unreachable
        INDISTINGUISHABLE once written
        ┌────────────────────────────┴────────────────────────────┐
        │                                                         │
 getActiveSessionByWorkspace                            startSession
 ORDER BY rowid LIMIT 1                                 practice.start → real attempt
 → always the seeded row                                → sets current_attempt_id
        │                                                         │
        ▼                                                         ▼
 Home "Resume session"                              ConceptView "Practice this"
 current_attempt_id = NULL                          current_attempt_id = real
        │                                                         │
        ▼                                                         ▼
 "There is no practice content                      prompt · hint · evaluate · next
  for this concept yet."   ── DEAD END                        works

 Reached by no path — hardcoded in the page itself:
   visualization pane copy · fullVisualizationScene.ts · prefilled "your working"
   `0′ of 20′` · five-dash indicator · WorkspaceOverview Recent + recommendation
```

**After the beta path (§6).** Both entry points reach the same real attempt.

```
 knowledge-package/ ──► Practice module ──practice.concepts@1──► create_workspace   [075]
   (Core never reads the package)         opaque {concept_id, name, topic, summary}
                                                                       │
 mockData/* ──importSampleWorkspace──► concepts · goals · modules ·    │  concepts rows
   ✗ sessions ✗ exchanges ✗ settled conclusions          material      │  + crosswalk
   ✗ workspaces.progress ✗ workspaces.last_activity_at    [072]        │
                              │                                        │
                              └───────────────┬────────────────────────┘
                                              ▼
                              "Practice this" / Home Continue
                                              │
                       startSession REUSES the open session for       [071]
                       (workspace, concept); creates only if none
                                              │
        sessions.last_activity_at ────────────┴──────── workspaces.last_activity_at
                                              │         (written on real activity only)
                                              ▼
                                 sessions.current_attempt_id
                                              │  practice.describe
                                              ▼
 boot ─► workspaces.length ? home : firstLaunch ─► Continue ─► the same attempt  [070]
         activeWorkspaceId = §6.4's deterministic ordering, no stored route
```

Restoration is a *query*, not a stored blob. Nothing about the learner's position is written
anywhere except as a side effect of activity that genuinely happened — mechanism 1 applied to
the restoration problem itself.

## 4. Root-cause analysis for the two observed failures

The walk recorded them as two tasks. They are one defect seen from two ends: **the
application has no representation of "the session the learner is currently in."**

### 4.1 Restart does not restore the active attempt

`src/App.tsx:187` mounts `NavigationProvider` with `initialRoute={{ type: 'firstLaunch' }}`,
unconditionally. Nothing anywhere re-routes on boot, and `WorkspaceProvider` holds
`activeWorkspaceId` in a plain `useState` with no persistence. So restart restores nothing —
not the route, not the workspace, not the session.

The sidebar looked populated because `Application` calls `useWorkspaces()` regardless of
route, so real persisted workspaces render beside a First Launch screen that asserts there
are none. The database binding from `063` was never the problem; it was never read, because
no code path after boot asks for it.

The second half of the observation — "Practice this produced a fresh `f(x) = 3x - x²`
problem" — is a distinct defect with the same root. `start_session_handler`
(`src-tauri/src/commands/session.rs:115`) unconditionally `INSERT`s a new session row and
calls `practice.start` for it. There is no "reopen the session this concept already has"
path anywhere in the stack, so every press of "Practice this" abandons the previous attempt
and generates another.

### 4.2 The Home "Resume session" continuation is a dead end

`get_active_session_by_workspace_handler` (`src-tauri/src/commands/session.rs:100`):

```sql
SELECT id FROM sessions
WHERE workspace_id = ?1 AND status <> 'completed'
ORDER BY rowid LIMIT 1
```

`session-shell-method` is the first row `insert_sessions` writes during sample import, so it
holds the lowest `rowid` and wins this query permanently — over every real session the
learner subsequently starts. Its `current_attempt_id` is `NULL`, because the fixture in
`src/services/mockData/sessions.ts` has no such field and nothing ever bound one. `useAttempt`
therefore resolves to `undefined` and `ProblemPane` renders its unbound branch.

The same query backs `WorkspaceOverviewPage`'s Continue card, that page's
`ReasonedRecommendation` "Start · 8 min" button, and `useCommandPalette`'s session actions.
All four affordances point at the same dead session.

`ORDER BY rowid` is not merely a wrong tiebreak. There is no column on `sessions` that
records when the learner last touched one, so there is currently nothing correct to order by.

### 4.3 Why frontend tests were green

`src/test/mockBackend.ts:83` seeds its own session list from `mockSessions` and implements
`startSession` by binding an attempt for a mapped concept. The double therefore satisfies
`getActiveSessionByWorkspace` with a session the double itself bound — the exact condition
the real backend fails to produce. Rust tests, in the other direction, prove
`practice.start`/`describe` and the `current_attempt_id` round-trip without ever rendering a
page or choosing which session Home displays. `069`'s command-registration guard closed the
"command does not exist" class of seam defect; it cannot catch "the command exists and
returns the wrong row."

Nothing in the suite crosses render → IPC → SQLite → render. §6.6 adds that.

## 5. Inventory and classification

Complete, so the record stands alone. **Path** says whether an item is on the beta-critical
path (§6) or a separately approved follow-up (§7).

### 5.1 Remove — false affordance, dead end, or fabricated history

| Item | Location | Why | Path |
|---|---|---|---|
| Seeded sessions, 40 tutor exchanges, settled conclusions, `problemIndex 6 / problemCount 12` | `mockData/sessions.ts` via seed | Directly causes the Home dead end (§4.2) | **072** |
| Workspace `progress: 0.58` and `lastActivityAt` | `mockData/workspaces.ts` via seed | A progress figure with nothing behind it — and `last_activity_at` is what §6.4's boot ordering reads, so a seeded value fabricates an active workspace | **072** |
| `ORDER BY rowid` active-session selection | `src-tauri/src/commands/session.rs:100` | §4.2 | **071** |
| `initialRoute={{ type: 'firstLaunch' }}` with no boot check | `src/App.tsx:187` | §4.1 | **070** |
| `FALLBACK_WORKSPACE_ID = 'workspace-calculus-ii'` | `src/App.tsx:28` | A mock id as a production fallback; exists only because nothing establishes a real active workspace at boot | **070** |
| Prefilled working `r = x, h = x² − 1 …` | `src/pages/StudySessionPage.tsx:27` | Authored working beside a real attempt it does not correspond to | **073** |
| Visualization pane copy naming `y = x² − 1 on [1, 3]` | `src/pages/StudySessionPage.tsx:169` | Contradicts the real generated problem on the same screen | **073** |
| `Problem N of 1` and the five-dash indicator derived from it | `SessionToolbar.tsx:37`, `StudySessionPage.tsx:198` | A position claim with no persisted total behind it | **073** |
| `0′ of 20′` frozen elapsed readout | `SessionToolbar.tsx:65` | `elapsed_minutes` is written once as `0` and never updated. The **readout** goes now; the tracking behind it is §7.1 | **073** |
| Canned tutor answer written into `tutor_exchanges` | `src-tauri/src/commands/session.rs:328` | The only defect here that writes fiction into the learner's own record. First item in §7.2 | §7.2 |
| Seeded `workspaceActivity` events ("While you were away") | `mockData/workspaceActivity.ts` via seed | Fabricated history, reachable only via the 30-day context-recovery variant | §7.2 |
| Seeded `notes` ("From your work", "Your notes") | `mockData/notes.ts` via seed | Claims learner authorship | §7.2 |
| Per-concept `recentDiagnostics` ("Recent practice") | `mockData/concepts.ts` via seed | Claims learner attempts | §7.2 |
| Per-concept `learnerHeuristic` / `heuristicEvidence` | `mockData/concepts.ts` via seed | Claims learner authorship | §7.2 |
| Concept `masteryState` / `wasMasteryState` above `New` | `mockData/concepts.ts` via seed | Mastery is learner-derived; pre-set states assert a history | §7.2 |
| `shellMethodScene` as the only scene `FullVisualizationPage` renders | `src/pages/fullVisualizationScene.ts` | Same contradiction as the session pane, full-screen | §7.2 |
| `ReasonedRecommendation` — fabricated Tuesday/Thursday evidence, "Start · 8 min" | `WorkspaceOverviewPage.tsx:92` | Fabricated evidence *and* a dead destination | §7.2 |
| "Recent" rail — hardcoded note and visualization lines | `WorkspaceOverviewPage.tsx:55` | Fabricated history | §7.2 |
| `conceptStatus()` returning "2 days ago" / "active" by array index | `WorkspaceOverviewPage.tsx:129` | Fabricated per-concept recency | §7.2 |
| `selectConceptsInPlay()` preferring four hardcoded concept names | `WorkspaceOverviewPage.tsx:123` | Mock-shaped assumption in production selection logic | §7.2 |
| "Axiom read that as" inference chips; inert comfort / materials / pacing controls | `CreateWorkspacePage.tsx:17` | Presents fabricated inference as system output | §7.2 |
| `useCommandPalette(workspaceId = 'workspace-calculus-ii')` | `src/hooks/useCommandPalette.ts:26` | A second mock-id production default | §7.2 |
| "Use template", "Try it first", "Add to another workspace", "See in concept map", "Save to concept notes", "Pin to notes", "Share", "N more notes" | Marketplace / ModuleDetail / ConceptView / StudySession / FullVis | Buttons with no destination | §7.2 |

One near-miss, checked and left alone: `Stage3StubRouteMenu` navigates to a nonexistent
`sessionId: 'session-1'` (`src/components/workspace/Stage3StubRouteMenu.tsx:17`), but returns
`null` unless `import.meta.env.DEV`, so it never ships. No change.

### 5.2 Wire to persisted domain state

| Item | Why | Path |
|---|---|---|
| `sessions.last_activity_at` | There is nothing correct to order active-session selection by | **071** |
| `workspaces.last_activity_at`, written on real activity | Makes the active workspace derivable at boot without stored UI state | **071** |
| `workspaces.created_at` | The column does not exist; §6.4's fallback ordering needs it | **071** |
| `startSession` reuse semantics | "Practice this" must reopen the concept's open session, not reset it | **071** |
| Concepts in a learner-created workspace | Criterion 1 is unreachable outside the sample without them | **075** |
| Boot workspace and route, derived | The only way criterion 2 is true rather than nominally supported | **070** |

### 5.3 Legitimate sample content — keep

The 87 Calculus II concepts with names and chapters, three sample workspaces, guiding goals,
the module catalog, workspace templates, material and its chapter segments. All of it is
curriculum: pre-authored content a learner could plausibly have installed, making no claim
about what they did. It stays, behind the same explicit opt-in it has today.

### 5.4 Inert chrome — keep

Rotate / Slice / Revolve / Cross-section, zoom controls, bounds sliders, the Inspector. These
read as tool chrome rather than as claims about the learner, and `AXIOM-HANDOFF.md` §4
Screen 6 specifies the visualization page as inert until an engine exists. Unchanged by §6.

### 5.5 Deferred — Stage 9

Real 3D rendering, a real tutor capability, marketplace backend, module sandboxing, offline
sync, problem families beyond `shell_y_poly`, and `knowledge.query@1` as a general capability.

## 6. The beta-critical path

Six tasks. `070` and `071` keep the ids and subjects the walk used, so they supersede those
observations rather than duplicating them.

**Ownership.** Functional implementation and test work is Codex's, per `AGENTS.md`. Claude
owns contract locking and independent review, and is the reviewer — not a co-implementer — on
every task below, so no task is implemented and reviewed by the same agent. Antigravity is
not assigned data or prop behaviour anywhere in this path; see §6.5 for the one bounded
visual pass it may pick up.

| id | title | owner | reviewer | depends on | criteria |
|---|---|---|---|---|---|
| 071 | Active-session selection and reuse | codex | claude | — | 1, 2, 6 |
| 072 | Sample seed carries curriculum, not activity | codex | claude | — | 6 |
| 075 | Workspace curriculum provisioning | codex | claude | — | 1 |
| 070 | Startup restoration from domain state | codex | claude | 071, 072 | 2 |
| 073 | Study Session contradictory-state removal | codex | claude | 071 | 6 |
| 074 | Native regression across restart | codex | claude | 070, 071, 072, 073, 075 | verifies 1, 2, 6 |

**Order.** `071 ∥ 072 ∥ 075` run concurrently — three independent tasks, no shared files.
`070` starts after both `071` and `072` merge. `073` depends on `071`: its gate is the moment
`071`'s session contract is locked in review, not `071`'s merge, so `073` may begin against
the locked contract while `071` is still in flight — their touched files are disjoint
(`073` is `src/pages` and `src/components/session`; `071` is `src-tauri`). `073` and `070`
may overlap for the same reason. `074` starts only after all five have merged.

`073` is therefore **not** parallel with `071` in the sense of being independent of it; the
table's dependency is real and the overlap is a scheduling allowance on disjoint files, not
an absence of dependency.

### 6.1 — `071` Active-session selection and reuse

Supersedes the walk's task 071. Owner codex, reviewer claude.

**Files:** new `src-tauri/src/db/migrations/0004_session_activity.sql`,
`src-tauri/src/commands/session.rs`, `src-tauri/src/commands/workspace.rs`,
`src-tauri/src/commands/models.rs`, `src-tauri/src/commands/tests.rs`,
`src/types/session.ts`, `src/types/workspace.ts`, `src/test/mockBackend.ts`.

**Schema.** `sessions.last_activity_at TEXT`, `workspaces.created_at TEXT`, both nullable so
existing rows migrate untouched. `workspaces.last_activity_at` already exists and starts
being written rather than only seeded. `workspaces.created_at` is set on creation and on
sample import; pre-existing rows keep `NULL`, which §6.4's ordering handles explicitly.
`Workspace`'s wire model gains `createdAt`, since `070`'s comparator runs in the frontend.

**Activity timestamps.** `sessions.last_activity_at` and the owning workspace's
`last_activity_at` are both written on session start, resume, pause, `nextProblem`, and
attempt evaluation. These are *timestamps of activity*, not durations —
`sessions.elapsed_minutes` is untouched by this task and by this whole path (§7.1).

**Selection.** `get_active_session_by_workspace_handler` orders
`ORDER BY last_activity_at IS NULL, last_activity_at DESC, rowid DESC` — non-null activity
first, most recent wins, oldest-row-wins replaced by newest-row-wins on ties.

**Reuse.** `start_session_handler`, given `(workspace_id, concept_id)` with an existing
non-completed session, returns that session instead of inserting. If its `current_attempt_id`
is null or its attempt is solved, a fresh attempt is bound onto it — the session persists, the
problem advances. This is the approved change to `startSession`'s Stage 7 semantics: "Practice
this" is a resume, not a reset.

**Tests.** Migration coverage proving existing rows survive with both new columns unset;
selection ordering proving that recent real activity wins over **an older open session
created by test setup** — the fixture is constructed in the test, not imported from
`mockData`, so the assertion does not depend on production sample fixtures that `072`
removes; reuse on second and third `startSession` for the same pair; reuse after solving
binding a new attempt to the same session row; `workspaces.last_activity_at` advancing on
session activity and on nothing else; all four existing degradation branches (unmapped
concept, module disabled, invoke failure, missing workspace) still succeeding.

**Acceptance.** Two `startSession` calls for one `(workspace, concept)` yield one row and the
identical prompt. A session touched later wins selection over an older open one regardless of
`rowid`. `elapsed_minutes` is unchanged by this task.

### 6.2 — `072` Sample seed carries curriculum, not activity

Owner codex, reviewer claude.

**Files:** `src-tauri/src/commands/models.rs`, `src-tauri/src/commands/seed.rs`,
`src-tauri/src/commands/tests.rs`, `src/services/sampleWorkspaceService.ts`,
`src/services/mockData/workspaces.ts`, `src/test/mockBackend.ts`.

Removes `sessions` from `SampleWorkspaceSeed` (`models.rs:284`), from `import_seed`, and from
`sampleWorkspaceService.ts`'s payload — along with the `tutor_exchanges` and
`session_settled_conclusions` rows `insert_sessions` writes. Zeroes `progress` and drops
`lastActivityAt` in `mockData/workspaces.ts`.

Scope is the seeded *sessions* and the two workspace columns `070` reads — not seed hygiene
in general. Mastery states, diagnostics, notes, heuristics and activity events are also
fabricated history, but none causes the dead end and none is read by the boot ordering; they
are §7.2.

**`src/test/mockBackend.ts` is updated in this task.** It seeds itself from `mockSessions` at
line 83, which is precisely the divergence that let the frontend suite stay green through this
defect (§4.3). Leaving the double richer than the real backend preserves the blind spot.
`mockData/sessions.ts` is not deleted — `mockBackend` and individual tests may still construct
sessions; it stops shipping them as seed.

**Tests.** After sample import, `getActiveSessionByWorkspace` returns `None` for all three
sample workspaces and no `tutor_exchanges` or `session_settled_conclusions` rows exist; the
Shell method concept still resolves `shell.method_vertical_axis`; the existing crosswalk guard
still passes; a frontend test asserting Home renders no Continue card immediately after import.

**Acceptance.** Importing the sample produces zero session rows and a workspace with
`progress = 0` and `last_activity_at IS NULL`, while "Practice this" on Shell method still
generates a real problem.

### 6.3 — `075` Workspace curriculum provisioning

Owner codex, reviewer claude. On the beta path because criterion 1, as written in `064`,
means a **learner-created** workspace reaches a real generated problem — not only the
imported sample.

**Files:** `src-tauri/src/practice/types.rs`, `src-tauri/src/practice/provider.rs`,
`src-tauri/src/commands/workspace.rs`, `src-tauri/src/practice/tests/mod.rs`,
`src-tauri/src/commands/tests.rs`, `CORE.md`, `ARCHITECTURE.md`.

**The approved capability.** `practice.concepts@1`, an additive sixth Practice capability
alongside `generate`, `evaluate`, `hint`, `start` and `describe`. Request carries
`workspace_id`; response is a list of descriptors:

```rust
pub struct ConceptDescriptor {
    pub concept_id: String,  // opaque to Core — the knowledge-concept crosswalk value
    pub name: String,
    pub topic: String,       // stored as concepts.chapter — NOT NULL, so Practice supplies it
    pub summary: String,     // stored as concepts.meaning  — NOT NULL, likewise
}
```

`topic` and `summary` are in the contract because `concepts.chapter` and `concepts.meaning`
are `NOT NULL`. Core inventing either would be Core authoring subject content; having the
module supply both keeps Core storing opaque strings.

**The boundary, as approved.** Core may store these strings and the knowledge-concept
crosswalk. Core must not interpret subject knowledge and must not read the Knowledge Package
directly — resolution goes through `ModuleRegistry` exactly as `start_session` already does,
with `calling_module_id: "core.workspace"` matching `core.session`'s per-command granularity.
Core supplies only its own domain defaults on insert: a generated Core concept id,
`mastery_state = 'New'`, `on_exam = 0`, no diagnostics, no notes. This is **not** expanded
into `knowledge.query@1`.

**Degradation.** `create_workspace_handler` invokes the capability and inserts what comes
back. When the capability is absent, the module is disabled, or the invocation fails,
workspace creation still succeeds with zero concepts — the same philosophy `063` established
for `start_session`, and the same reason: workspace creation is Core's own primary function.

**Documentation.** `CORE.md` gains this boundary in its capability section; `ARCHITECTURE.md`
records that `concepts` rows may originate from a capability response as well as from the
learner or the sample import. Claude locks both as the contract-owning agent before `075`
merges.

**Tests.** The capability returning the bundled package's three concepts; workspace creation
inserting them with `knowledge_concept_id` set and `mastery_state = 'New'`; the three
degradation branches each leaving a usable zero-concept workspace; and an end-to-end Rust
assertion that a created workspace's Shell method concept resolves a family through
`practice.start`.

**Acceptance.** A workspace created through `createWorkspace` contains the bundled package's
concepts, its Shell method concept is mapped, and `startSession` on it binds a real attempt.
With Practice disabled, creation still returns a workspace.

### 6.4 — `070` Startup restoration from domain state

Supersedes the walk's task 070. Owner codex, reviewer claude. Frontend only — no new
persistence, no stored route, no new command.

**Files:** `src/App.tsx`, `src/hooks/WorkspaceProvider.tsx`, new
`src/hooks/useRestoredContext.ts`, new `src/hooks/selectStartupWorkspace.ts`,
`src/App.test.tsx` / `src/test/App.test.tsx`.

**The locked boot rule.**

1. Boot blocks first paint on `getWorkspaces()`, so the app never renders First Launch and
   then corrects itself.
2. Zero workspaces → `firstLaunch`.
3. Otherwise → `home`, with `activeWorkspaceId` chosen by this total ordering over the
   returned list, applied as a single comparator:

   ```
   1. workspaces with non-null last_activity_at sort before those with null
   2. among those, greatest last_activity_at first
   3. if every workspace has null last_activity_at:
        non-null created_at before null created_at
        greatest created_at first
   4. final tiebreak: id descending
   ```

   Step 4 exists to make the order **total and reproducible**, not to approximate recency —
   workspace ids are random, so the id tiebreak is arbitrary but deterministic. `getWorkspaces`
   keeps its existing `ORDER BY rowid` so the sidebar's listing order is unchanged; the
   comparator runs in the frontend over that list, which is why `071` exposes `createdAt` on
   the wire model.
4. **Immediately after a create or import action, the workspace returned by that action is
   used directly**; the boot fallback is not re-run. A freshly created workspace has null
   activity and would otherwise be chosen only by the `created_at` branch, and a freshly
   imported sample would compete with existing workspaces on timestamps it does not have.
5. Home's Continue resolves the selected workspace's most recently active non-completed
   session via `getActiveSessionByWorkspace`, and reopens that session's bound attempt
   unchanged — no change to `useAttempt` or the attempt lifecycle.

`FALLBACK_WORKSPACE_ID` is removed here. `workspace-calculus-ii` is not reintroduced as a
fallback anywhere.

**Criterion 2 wording.** This restores the correct in-progress attempt and puts it one click
from launch; it does not land the learner *inside* the session. `064`'s criterion 2 is
recorded in those terms.

**Tests.** `selectStartupWorkspace` is a pure function with unit coverage for each ordering
branch: mixed null and non-null activity; all-null activity falling through to `created_at`;
all-null activity *and* all-null `created_at` falling through to the id tiebreak; a single
workspace; an empty list. Plus boot tests against `mockBackend`: zero workspaces renders First
Launch; one or more renders Home; the restored workspace is the one last worked in rather than
`workspaces[0]`; the post-create and post-import paths use the returned workspace rather than
the comparator.

**Acceptance.** Relaunching with at least one persisted workspace never shows First Launch;
relaunching with zero still does; Home's Continue reopens the exact bound attempt from before
the restart.

### 6.5 — `073` Study Session contradictory-state removal

Owner codex, reviewer claude. Functional removal and prop-shape change — not a visual task.

**Files:** `src/pages/StudySessionPage.tsx`, `src/components/session/SessionToolbar.tsx`,
`src/pages/StudySessionPage.test.tsx`, `src/components/session/SessionComponents.test.tsx`.

Empties the prefilled working default. Replaces the visualization pane's hardcoded region copy
with an inert placeholder naming no specific region. Makes `SessionToolbar`'s `problemCount`
optional so the five-dash indicator renders only when a real persisted total exists and the
counter reads "Problem 3" rather than "Problem 3 of 1". **Removes the `0′ of 20′` elapsed
readout entirely** — this task adds no Rust timing behaviour and does not modify pause, resume
or `nextProblem` for time tracking. Real elapsed-time tracking is §7.1.

The tutor pane is not in this task; its canned persisted answer is a fabrication rather than a
contradiction, and it heads §7.2.

**Tests — absence must be asserted.** "Existing tests pass untouched" is not acceptance
evidence for removed content. `073` adds explicit negative assertions:

- the working area renders with an empty value on mount;
- no element contains the string `y = x² − 1` or `[1, 3]` anywhere on the session screen;
- with `problemCount` absent, no five-dash indicator renders and no "of N" appears in the
  problem counter or its `aria-label`;
- no elapsed-time claim renders — no `′` readout, no `of {targetMinutes}` text;
- generate / evaluate / hint / next behaviour is unchanged, asserted by the *positive*
  path re-tested in this file: a bound attempt renders its prompt, a wrong answer produces
  "That does not match yet." plus a hint, a correct answer produces "Correct.", and Next
  problem resets the answer and hint list.

**Acceptance.** All five assertions above pass, and no removed string appears anywhere in the
rendered session screen.

**Bounded visual pass, if needed.** If removing these elements leaves the toolbar or
visualization pane visually broken — collapsed spacing, an empty region with no layout — file
`078` as a bounded Antigravity polish task against the resulting screenshots. It covers CSS
and layout only; the data and prop behaviour stays in `073` and is not reassigned.

### 6.6 — `074` Native regression across restart

Owner codex, reviewer claude. Extends `e2e/`, the one harness crossing render → IPC → SQLite.

**Files:** new `e2e/practice-loop-restart.test.mjs`, `e2e/README.md`.

**Main flow — two restarts, deliberately.** The first proves open-attempt restoration; the
second proves submission and hint-state restoration.

1. Create a workspace through First Launch.
2. Navigate to its Shell method concept.
3. Choose "Practice this".
4. Assert a generated prompt with no unsubstituted placeholders.
5. Record the exact prompt text.
6. Terminate and relaunch against the same application-data directory.
7. Assert the app opens Home, not First Launch.
8. Use Continue; assert the exact same prompt and an open attempt.
9. Submit a deliberately wrong answer.
10. Assert "That does not match yet." and the authored hint.
11. Terminate and relaunch again.
12. Continue; assert the same prompt, the wrong-submission state, and the revealed hint all
    remain.
13. Submit the correct answer.
14. Assert "Correct."
15. Choose "Next problem".
16. Assert the prompt changes and the answer and hints reset.

Steps 1–3 exercise `075`: this flow reaches a real generated problem in a **learner-created**
workspace, which is what criterion 1 requires and what no existing test covers.

**Retained negative assertion.** Separately, immediately after importing the sample workspace,
Home must not show a fabricated Continue card.

This is the coverage `067` and `068` do not provide. `067` is a backend corpus; `068` is a
network-disabled acceptance test. Neither renders a page.

**Acceptance.** Both flows green on Linux CI's existing `e2e` job.

## 7. Separately approved follow-ups

Filed as `proposed`, outside the beta path, and not treated as consequences of the walk. A
§7 item is pulled forward only if a `074` regression directly reaches it.

### 7.1 — `076` Real elapsed-time tracking

`073` removes the frozen `0′ of 20′` readout; nothing replaces it. If session timing is still
wanted, it needs `sessions.elapsed_minutes` actually maintained across pause, resume and
`nextProblem`, plus a decision about whether elapsed-against-target reads as context or as a
score under `AGENTS.md:22`. Not required for criteria 1, 2 or 6.

### 7.2 — `077` Application-wide affordance purge

Everything in §5.1 marked §7.2: the canned tutor answer first, since it is the only defect
that writes fiction into the learner's record; then the seed's mastery states, diagnostics,
notes, heuristics and activity events; `FullVisualizationPage`'s single hardcoded scene;
`WorkspaceOverviewPage`'s Recent list, fabricated recommendation and index-derived recency;
`CreateWorkspacePage`'s inference chips; `useCommandPalette`'s `workspace-calculus-ii` default;
and every button with no destination.

It spans six pages and three ownership lanes; several items diverge visibly from
`AXIOM-HANDOFF.md` screenshots; and dropping seeded mastery states is a product decision about
what a sample workspace is *for*. Expect three or four small tasks once the shape is agreed,
led by the tutor item. Marketplace, ModuleDetail, generic notes, mastery-history removal and
unrelated page cleanup stay here and are not pulled into `070`–`075`.

## 8. Testing summary

| Layer | Task | What it proves |
|---|---|---|
| Rust | 071 | Selection ordering with an older test-created open session; reuse; activity timestamps; migration safety |
| Rust | 072 | No sessions, exchanges or conclusions after sample import; crosswalk intact |
| Rust | 075 | Capability response; concepts inserted with crosswalk; three degradation branches; created workspace resolves a family |
| Frontend | 070 | `selectStartupWorkspace` per ordering branch; boot routing; post-create and post-import selection |
| Frontend | 072, 073 | Home shows no Continue after import; five explicit absence assertions plus the re-tested positive path |
| Native | 074 | Created-workspace practice loop across two restarts, plus the sample-import negative assertion |

`072`'s update to `mockBackend.ts` is load-bearing for all of it. A test double richer than the
backend it doubles is how this defect survived a green pipeline; the fix is to make the double
honest, not to add tests around it.

## 9. What this design commits to

**Removed:** seeded sessions, their tutor exchanges and settled conclusions; seeded workspace
progress and last-activity; the `rowid` session selection; the hardcoded first-launch route;
`FALLBACK_WORKSPACE_ID`; prefilled working; contradictory visualization copy; unbacked problem
totals and the five-dash indicator; the frozen elapsed readout.

**Wired to real data:** session and workspace activity timestamps; `workspaces.created_at`;
active-session selection; session reuse on "Practice this"; concepts in a learner-created
workspace via `practice.concepts@1`; boot workspace and route, derived.

**Labelled:** nothing. Every item considered for labelling was either removable or genuinely
inert chrome. A "sample" badge to make dishonest content acceptable was rejected.

**Not persisted:** the route. No `app_state`, no `localStorage`, no stored UI position.

**Deferred:** real elapsed-time tracking (§7.1); the application-wide affordance purge,
including the canned tutor answer and the sample's mastery history (§7.2); and all Stage 9
scope, `knowledge.query@1` included.

**Unmet until §6 lands and `074` passes:** criteria 1, 2 and 6.
