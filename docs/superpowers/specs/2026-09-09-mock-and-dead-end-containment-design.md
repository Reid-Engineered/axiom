# Mock and dead-end containment — design

## 1. Scope

The Windows release-build walk of `064` proved the Practice loop works and, in the same
pass, proved the rendered app mixes two incompatible sources of truth. A real generated
`shell_y_poly` problem evaluates, hints, and advances correctly; alongside it, Home
advertises "Calculus II — Shell method · Problem 6 of 12" with a "Resume session" button
that opens a screen reading "There is no practice content for this concept yet."

This design covers the containment of every production-reachable path whose data is mock,
hardcoded, or seeded-without-a-corresponding-event. It is the work that makes `064`'s
criteria 1, 2 and 6 honestly checkable rather than nominally unblocked.

**Does not build:** Stage 9 scope in any form — no real tutor, no 3D visualization engine,
no marketplace backend, no module sandboxing, no additional problem families, no offline
sync. It also does not build `067` (regression corpus) or `068` (offline acceptance test),
which remain their own tasks; §8 here adds rendered coverage those two do not provide and
were never scoped to provide.

**Does not remove** the "Explore a sample workspace" feature. Sample data stays. What
changes is what the sample is allowed to contain.

## 2. The rule this design reduces to

> **The rendered app must never present, as the learner's own history, anything the
> learner did not do.**

Everything below is one of three mechanisms enforcing it:

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

## 3. Source-of-truth, before and after

**Today.** Two paths reach `StudySessionPage`, and only one of them works.

```
 src/services/mockData/*
   3 workspaces · 87 concepts · 4 sessions (40 fabricated tutor exchanges)
   goals · modules · material · notes · activity events
        │
        │ sampleWorkspaceService.ts ships the WHOLE fixture set as `seed`
        ▼
 importSampleWorkspace (Rust)  ──►  SQLite
                                     seeded rows and learner rows are
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

 Reached by neither path — hardcoded in the page itself:
   visualization pane copy · fullVisualizationScene.ts · prefilled "your working"
   WorkspaceOverview Recent + ReasonedRecommendation · CreateWorkspace inferences
```

**After.**

```
 knowledge-package/ ──practice.concepts@1──► create_workspace ─┐
   3 concepts, 1 family                                        │
                                                               ▼
 mockData/* (curriculum only) ──importSampleWorkspace──►   concepts rows
   workspaces · concepts · goals · modules · material          (knowledge_concept_id)
   ✗ sessions ✗ exchanges ✗ diagnostics ✗ notes                │
   ✗ progress ✗ activity events ✗ mastery history              │
                                                               ▼
                                        "Practice this" / Home Continue
                                                    │
                             startSession REUSES the open session for
                             (workspace, concept); creates only if none
                                                    │
                                     sessions.current_attempt_id
                                                    │  practice.describe
                                                    ▼
                        app_state('lastRoute', 'activeWorkspaceId')
                                                    │
                                  boot ──► restore ──► the same attempt
```

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
"command does not exist" class of seam defect; it does not and cannot catch "the command
exists and returns the wrong row."

Nothing in the suite crosses render → IPC → SQLite → render. §8 adds that.

## 5. Inventory and classification

### 5.1 Remove — false affordance, dead end, or fabricated history

| Item | Location | Why |
|---|---|---|
| Seeded sessions, 40 tutor exchanges, settled conclusions, `problemIndex 6 / problemCount 12` | `src/services/mockData/sessions.ts` via seed | Activity the learner did not perform, persisted as if they had |
| Seeded `workspaceActivity` events ("While you were away") | `mockData/workspaceActivity.ts` via seed | Same |
| Seeded `notes` ("From your work" palette group, "Your notes" rail) | `mockData/notes.ts` via seed | Claims learner authorship |
| Per-concept `recentDiagnostics` ("Recent practice") | `mockData/concepts.ts` via seed | Claims learner attempts |
| Per-concept `learnerHeuristic` / `heuristicEvidence` ("Your notes say you think of it as…") | `mockData/concepts.ts` via seed | Claims learner authorship |
| Workspace `progress: 0.58` and `lastActivityAt` | `mockData/workspaces.ts` via seed | A progress figure with nothing behind it; drives Home's "3 days ago since your last session" |
| Concept `masteryState` / `wasMasteryState` above `New` | `mockData/concepts.ts` via seed | Mastery is learner-derived; pre-set states assert a history. See §7 decision C |
| `ORDER BY rowid` active-session selection | `src-tauri/src/commands/session.rs:100` | §4.2 |
| `initialRoute={{ type: 'firstLaunch' }}` with no boot check | `src/App.tsx:187` | §4.1 |
| Prefilled working `r = x, h = x² − 1 …` | `src/pages/StudySessionPage.tsx:27` | Authored working beside a real attempt it does not correspond to |
| Visualization pane copy naming `y = x² − 1 on [1, 3]` | `src/pages/StudySessionPage.tsx:169` | Actively contradicts the real generated problem on the same screen |
| `shellMethodScene` as the only scene `FullVisualizationPage` renders | `src/pages/fullVisualizationScene.ts` | Same, full-screen; plus "contributes about 6% of the total volume" |
| `ReasonedRecommendation` — fabricated Tuesday/Thursday evidence, "Start · 8 min" → seeded session | `src/pages/WorkspaceOverviewPage.tsx:92` | Fabricated evidence *and* a dead destination |
| "Recent" rail — hardcoded note and visualization lines | `src/pages/WorkspaceOverviewPage.tsx:55` | Fabricated history |
| `conceptStatus()` returning "2 days ago" / "active" by array index | `src/pages/WorkspaceOverviewPage.tsx:129` | Fabricated per-concept recency |
| `selectConceptsInPlay()` preferring four hardcoded concept names | `src/pages/WorkspaceOverviewPage.tsx:123` | Mock-shaped assumption in production selection logic |
| "Axiom read that as" inference chips; inert comfort / materials / pacing controls | `src/pages/CreateWorkspacePage.tsx:17` | Presents fabricated inference as system output |
| `FALLBACK_WORKSPACE_ID = 'workspace-calculus-ii'` | `src/App.tsx:28` | A mock id as a production fallback |
| `useCommandPalette(workspaceId = 'workspace-calculus-ii')` | `src/hooks/useCommandPalette.ts:26` | Same |
| "Use template", "Try it first", "Add to another workspace" | Marketplace / ModuleDetail | Buttons with no destination |
| "See in concept map", "Save to concept notes", "Pin to notes", "Share", "N more notes" | ConceptView / StudySession / FullVis | Same |
| Canned tutor answer written into `tutor_exchanges` | `src-tauri/src/commands/session.rs:328` | Writes fiction into the real database. See §7 decision D |

One near-miss, checked and left alone: `Stage3StubRouteMenu` navigates to a
nonexistent `sessionId: 'session-1'` (`src/components/workspace/Stage3StubRouteMenu.tsx:17`),
but returns `null` unless `import.meta.env.DEV`, so it never ships. No change.

### 5.2 Wire to persisted domain state

| Item | Why |
|---|---|
| Boot route and active workspace | The only way criterion 2 is true rather than nominally supported |
| Concepts in a learner-created workspace | `create_workspace_handler` inserts a workspace, a goal, and offline rows — no concepts. Practice is therefore unreachable outside the sample today |
| `startSession` reuse semantics | "Practice this" must reopen the concept's open session, not reset it |
| `sessions.last_activity_at` | There is currently nothing correct to order active-session selection by |
| `sessions.elapsed_minutes` | Written once as `0`, never updated; the toolbar reads `0′ of 20′` forever. See §7 decision F |

### 5.3 Legitimate sample content — keep

The 87 Calculus II concepts with names and chapters, three sample workspaces, guiding goals,
the module catalog, workspace templates, material and its chapter segments. All of it is
curriculum: pre-authored content a learner could plausibly have installed, making no claim
about what they did. It stays, behind the same explicit opt-in it has today.

### 5.4 Inert chrome — keep, visibly disabled

Rotate / Slice / Revolve / Cross-section, zoom controls, bounds sliders, the Inspector. These
read as tool chrome rather than as claims about the learner, and `AXIOM-HANDOFF.md` §4
Screen 6 explicitly specifies the visualization page as inert until an engine exists. They
get a disabled state and no invented copy.

### 5.5 Deferred — Stage 9

Real 3D rendering, a real tutor capability, marketplace backend, module sandboxing, offline
sync, problem families beyond `shell_y_poly`.

## 6. The work

Eight tasks. Two of them carry the ids the walk used, on the same subjects, so they
supersede those observations rather than duplicating them.

| id | title | owner | depends on | beta criteria |
|---|---|---|---|---|
| 070 | Restart restores the learner's context | claude | 071, 072 | **2**, 6 |
| 071 | Active-session selection and session reuse | codex | — | **1, 2, 6** |
| 072 | `app_state` persistence for route and workspace | codex | — | 2 |
| 073 | Sample seed carries curriculum, not activity | codex | — | **6** |
| 074 | `practice.concepts@1` and concepts on workspace create | codex | — | **1** |
| 075 | Study Session — remove unbacked session state | antigravity | 071 | **6** |
| 076 | Remove false affordances outside the session | antigravity | 073 | **6** |
| 077 | Native regression — practice loop across restart | claude | 070–076 | verifies 1, 2, 6 |

Order: `072 ∥ 071 ∥ 073 ∥ 074` → `070 ∥ 075 ∥ 076` → `077`.

`071` and `072` both add a migration and will collide on the next free number. The
numbers below are indicative; whichever lands second renumbers its file, and neither
task treats a specific migration number as part of its contract.

### 070 — Restart restores the learner's context

Supersedes the walk's task 070. Frontend boot sequencing.

Boot blocks first paint on `getWorkspaces()` and `getAppState()`. Zero workspaces →
`firstLaunch`, as today. Otherwise restore the persisted route; if that route no longer
resolves (its session row is gone or `completed`), fall back to `home` — landing on a valid
screen, not on a generic "unavailable" message. `activeWorkspaceId` is restored alongside and
written on every change.

Touches `src/App.tsx`, `src/hooks/NavigationProvider.tsx`, `src/hooks/WorkspaceProvider.tsx`,
a new `src/services/appStateService.ts`, and a new `src/hooks/useRestoredContext.ts`.
Removes `FALLBACK_WORKSPACE_ID`.

**Acceptance:** relaunching with at least one persisted workspace never shows First Launch;
relaunching from a bound Study Session lands back on that session with the same prompt;
relaunching with zero workspaces still shows First Launch; a persisted route pointing at a
deleted session lands on Home.

### 071 — Active-session selection and session reuse

Supersedes the walk's task 071. Rust.

Adds `sessions.last_activity_at` (migration `0004`), written on start, resume, pause,
`nextProblem`, and attempt evaluation. `get_active_session_by_workspace_handler` orders by it
descending, tie-broken on `rowid` descending, so the newest activity wins rather than the
oldest row.

`start_session_handler` gains reuse: given `(workspace_id, concept_id)` with an existing
non-completed session, return that session instead of inserting. If its `current_attempt_id`
is null or its attempt is solved, bind a fresh attempt onto it — the session persists, the
problem advances. This is what makes "Practice this" a resume rather than a reset.

Touches `src-tauri/src/db/migrations/0004_session_activity.sql`,
`src-tauri/src/commands/session.rs`, `src-tauri/src/commands/models.rs`, and the session
tests.

**Acceptance:** with a seeded session present in the same workspace, a newly started session
is the one Home returns; pressing "Practice this" twice yields one session row and the same
prompt both times; pressing it after solving yields the same session with a new attempt; the
existing `start_session` degradation branches (unmapped concept, module disabled, invoke
failure) still succeed.

### 072 — `app_state` persistence for route and workspace

Rust. Migration `0005` adds `app_state(key TEXT PRIMARY KEY, value TEXT NOT NULL)`, plus
`getAppState` / `setAppState` commands. Deliberately a key/value table and not typed columns:
what the frontend chooses to restore is presentation state, and Core should not grow a schema
migration each time that set changes.

**Acceptance:** a value written, the connection dropped, the database file reopened, and the
value read back identically; an unknown key returns null rather than erroring.

### 073 — Sample seed carries curriculum, not activity

Rust and fixtures. Removes `sessions` from `SampleWorkspaceSeed` (`models.rs:284`), from
`import_seed`, and from `sampleWorkspaceService.ts`'s payload. Same for `workspaceActivity`
and `notes`. Strips `recentDiagnostics`, `learnerHeuristic`, `heuristicEvidence`, and
non-`New` mastery states from `mockData/concepts.ts`; zeroes `progress` and drops
`lastActivityAt` in `mockData/workspaces.ts`.

**`src/test/mockBackend.ts` is updated in the same task.** It seeds itself from `mockSessions`
at line 83, which is precisely the divergence that let the frontend suite stay green through
this defect. Leaving the double richer than the real backend would preserve the blind spot.

`mockData/sessions.ts` is not deleted — `mockBackend` may still construct sessions for tests
that need one; it simply stops shipping them as seed.

**Acceptance:** immediately after sample import, `getActiveSessionByWorkspace` returns null
for every sample workspace and Home renders no Continue card; the Shell method concept still
resolves `shell.method_vertical_axis` and "Practice this" still generates a real problem; the
existing crosswalk guard still passes.

### 074 — `practice.concepts@1` and concepts on workspace create

Rust. A learner-created workspace currently has no concepts, so Practice is reachable only
inside the sample. This task makes the beta loop true for a real learner.

Adds an additive sixth Practice capability, `practice.concepts@1`, returning the concept ids
and names the module can serve — the same shape as the existing five, resolved through
`ModuleRegistry`. `create_workspace_handler` invokes it and inserts the returned concepts with
`knowledge_concept_id` set, degrading to zero concepts when Practice is absent, disabled, or
failing, exactly as `start_session` already degrades.

Core stores strings the module produced; it does not read `knowledge-package/` and gains no
subject knowledge, preserving `CORE.md` §1. See §7 decision B — this needs sign-off before
implementation, and `CORE.md` plus `ARCHITECTURE.md` are updated as part of the task.

**Acceptance:** a workspace created through First Launch contains the bundled package's
concepts; its Shell method concept opens and "Practice this" generates a real problem; with
the Practice module disabled, workspace creation still succeeds with zero concepts.

### 075 — Study Session: remove unbacked session state

Frontend. Empties the prefilled working default. Drops the visualization pane's hardcoded
region copy in favour of an inert placeholder that names no specific region. Makes
`SessionToolbar`'s `problemCount` optional so the five-dash indicator renders only when a real
count exists, and the counter reads "Problem 3" rather than "Problem 3 of 1". Handles the
elapsed-time readout per §7 decision F, and the tutor pane per §7 decision D.

**Acceptance:** the working area opens empty; no on-screen copy names a region that
contradicts the current attempt; no problem count is displayed unless persisted; the
generate / evaluate / hint / next behaviour observed in the walk is byte-for-byte unchanged.

### 076 — Remove false affordances outside the session

Frontend. Everything in §5.1 belonging to `WorkspaceOverviewPage`, `CreateWorkspacePage`,
`MarketplacePage`, `ModuleDetailPage`, `ConceptViewPage`, and `useCommandPalette`'s hardcoded
default. Buttons with no destination are removed, not relabelled.

**Acceptance:** no button on any production-reachable screen navigates nowhere or to a
nonexistent record; no screen displays evidence, recency, or recommendation text not derived
from a persisted row; a grep for `workspace-calculus-ii` outside `mockData/` and `test/`
returns nothing.

### 077 — Native regression: practice loop across restart

Extends `e2e/` — the one existing harness that crosses render → IPC → SQLite. One new flow:

sample import → Concepts → Shell method → "Practice this" → assert a prompt with no
unsubstituted placeholders → submit a wrong answer → assert "That does not match yet." and a
hint → submit the correct answer → assert "Correct." → "Next problem" → assert the prompt
changed → **terminate and relaunch against the same `XDG_DATA_HOME`** → assert the app is not
on First Launch and the same prompt is on screen.

Plus two negative assertions that would each have caught this class directly: immediately
after sample import Home renders no Continue card, and a freshly created workspace reaches a
real problem (post-`074`).

This is the coverage `067` and `068` do not provide. `067` is a backend corpus; `068` is a
network-disabled acceptance test. Neither renders a page.

## 7. Decisions requiring human approval before implementation

**A — `app_state` table as the home for persisted UI state.** Rather than `localStorage`.
The e2e harness controls persistence through `XDG_DATA_HOME`, which the SQLite file honours
and webview storage does not reliably; and restored state that must agree with database rows
is better stored beside them. Adds a new persistence surface; `ARCHITECTURE.md` §4/§5 updated.

**B — Core writing `concepts` rows from a module capability's output.** `CORE.md` §1 states
Core has no knowledge of any subject's internals. This preserves the letter of it (Core stores
opaque strings the module produced) but is new boundary surface. `CORE.md` documents
`knowledge.query@1` as a required capability that no code implements; building that general
capability is Stage 9 scope, so this proposes the narrow `practice.concepts@1` alongside the
existing five instead. If you would rather implement `knowledge.query@1` properly, `074`
becomes materially larger and should move out of the beta path.

**C — Sample concepts' mastery set to `New`, with diagnostics, heuristics, and notes
dropped.** This is the rule in §2 applied consistently: a mastery state is a claim about the
learner. It visibly thins the sample and diverges from several `AXIOM-HANDOFF.md` screenshots,
which show a workspace mid-journey. The alternative — keeping the states and labelling the
workspace as sample — was considered and rejected as the "generic fallback copy" failure mode.
This is the decision most worth overriding if you disagree.

**D — Tutor input disabled for beta; the canned Rust answer removed.**
`add_tutor_exchange_handler` currently persists a fixed sentence as the tutor's reply,
indistinguishable in the database from a real one. Of everything here it is the only defect
that writes fiction into the learner's own record. Disabling the input diverges from
`AXIOM-HANDOFF.md` Screen 5. Keeping it requires accepting fabricated persisted content.

**E — `startSession` reopens rather than creates.** A behaviour change to a command locked in
Stage 7. It is what "Practice this" has to mean if `063`'s binding is to survive a second
press, but it changes the meaning of an existing contract and should be an explicit decision
rather than a bug fix.

**F — `elapsedMinutes`: wire it, or remove the readout.** Recommendation: wire it on pause,
resume, and `nextProblem`. Elapsed time against an intended target is context, not a score,
so `AGENTS.md:22` permits it — and the five-dash indicator derived from `problemIndex` is the
element that genuinely reads as scoring, which `075` removes regardless.

## 8. Testing

Each task carries unit coverage in its own layer, as the repo already requires. What is new
is the seam:

- **Rust** — `071`'s selection ordering and reuse branches; `072`'s reopen-the-file
  round-trip; `073`'s "no session after sample import" assertion; `074`'s three degradation
  branches.
- **Frontend** — `070`'s four boot cases against `mockBackend`; `075` and `076` asserting
  absence, which is the harder direction and the one that was missing.
- **Native** — `077`, the only test in the repo that will exercise render → IPC → SQLite →
  restart → render.

`073`'s update to `mockBackend.ts` is load-bearing for all of this. A test double richer than
the backend it doubles is how this defect survived a green pipeline, and the fix is to make
the double honest rather than to add tests around it.

## 9. What this design commits to

**Removed:** seeded sessions, tutor exchanges, activity events, notes, diagnostics,
heuristics, seeded progress and mastery history; the `rowid` selection; the hardcoded first
launch route; prefilled working; contradictory visualization copy; fabricated recommendation
evidence and recency; the "Axiom read that as" chips; the canned tutor answer; every button
with no destination; both `workspace-calculus-ii` production fallbacks.

**Labelled:** nothing. Every item considered for labelling was either removable or genuinely
inert chrome. Adding a "sample" badge to make dishonest content acceptable was rejected.

**Wired to real data:** boot route and active workspace; active-session selection; session
reuse on "Practice this"; concepts in a created workspace; session last-activity and elapsed
time.

**Deferred:** the real tutor, the visualization engine, marketplace backend, `knowledge.query@1`
as a general capability, and every other Stage 9 item — unchanged by this design, and none of
them a prerequisite for the beta gate.
