# Mock and dead-end containment — design

## 1. Scope

The Windows release-build walk of `064` proved the Practice loop works and, in the same
pass, proved the rendered app mixes two incompatible sources of truth. A real generated
`shell_y_poly` problem evaluates, hints, and advances correctly; alongside it, Home
advertises "Calculus II — Shell method · Problem 6 of 12" with a "Resume session" button
that opens a screen reading "There is no practice content for this concept yet."

This design has two parts, deliberately separated:

- **§6 — the beta-critical path.** Five tasks that follow directly from the walk: seeded
  activity removal, active-session selection and reuse, startup restoration, Study Session
  contradictions, and one native restart regression. These are the work `064`'s criteria 1,
  2 and 6 actually wait on.
- **§7 — separately approved follow-ups.** Two proposals the investigation surfaced that are
  real, but are *not* consequences of the walk and must not ride in on its authority:
  workspace curriculum provisioning, and the application-wide affordance purge. Each needs
  its own decision before it is filed as anything but `proposed`.

The inventory in §5 covers everything found, so the record is complete; the split in §6/§7
governs what gets built now.

**Does not build:** Stage 9 scope in any form — no real tutor, no 3D visualization engine,
no marketplace backend, no module sandboxing, no additional problem families, no offline
sync. It also does not build `067` (regression corpus) or `068` (offline acceptance test),
which remain their own tasks; §8 adds rendered coverage neither provides nor was scoped to.

**Does not remove** the "Explore a sample workspace" feature. Sample data stays. What
changes is what the sample is allowed to contain.

**Does not add** a generic route-persistence mechanism. An earlier draft proposed an
`app_state(key, value)` table for storing the last route; it is dropped. Restoration is
derived from domain state instead (§6.3), which needs no new persistence surface.

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

Mechanism 3 is also where the beta path stops. Applying it exhaustively across every screen
is §7.2, not §6 — the walk found dead ends in the practice loop, and that is what the beta
gate is about.

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

**After the beta path (§6).**

```
 mockData/* ──importSampleWorkspace──►  concepts · goals · modules · material
   ✗ sessions ✗ exchanges ✗ settled conclusions          (knowledge_concept_id)
   ✗ workspaces.progress ✗ workspaces.last_activity_at            │
                                                                  ▼
                                          "Practice this" / Home Continue
                                                    │
                             startSession REUSES the open session for
                             (workspace, concept); creates only if none
                                                    │
                          sessions.last_activity_at ─┴─ workspaces.last_activity_at
                                                    │      (written on real activity)
                                                    ▼
                                     sessions.current_attempt_id
                                                    │  practice.describe
                                                    ▼
   boot ─► workspaces.length ? home : firstLaunch ─► Continue ─► the same attempt
           activeWorkspaceId = max(workspaces.last_activity_at)
```

Restoration is a *query*, not a stored blob. Nothing about the learner's position is written
anywhere except as a side effect of activity that genuinely happened — which is mechanism 1
applied to the restoration problem itself.

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

Nothing in the suite crosses render → IPC → SQLite → render. §6.5 adds that.

## 5. Inventory and classification

Complete, so the record stands alone. The **Path** column says whether an item is on the
beta-critical path (§6) or a separately approved follow-up (§7).

### 5.1 Remove — false affordance, dead end, or fabricated history

| Item | Location | Why | Path |
|---|---|---|---|
| Seeded sessions, 40 tutor exchanges, settled conclusions, `problemIndex 6 / problemCount 12` | `mockData/sessions.ts` via seed | Directly causes the Home dead end (§4.2) | **§6.1** |
| Workspace `progress: 0.58` and `lastActivityAt` | `mockData/workspaces.ts` via seed | A progress figure with nothing behind it — and `last_activity_at` is what §6.3's boot derivation reads, so a seeded value fabricates an active workspace | **§6.1** |
| `ORDER BY rowid` active-session selection | `src-tauri/src/commands/session.rs:100` | §4.2 | **§6.2** |
| `initialRoute={{ type: 'firstLaunch' }}` with no boot check | `src/App.tsx:187` | §4.1 | **§6.3** |
| Prefilled working `r = x, h = x² − 1 …` | `src/pages/StudySessionPage.tsx:27` | Authored working beside a real attempt it does not correspond to | **§6.4** |
| Visualization pane copy naming `y = x² − 1 on [1, 3]` | `src/pages/StudySessionPage.tsx:169` | Contradicts the real generated problem on the same screen | **§6.4** |
| `Problem N of 1` and the five-dash indicator derived from it | `SessionToolbar.tsx:37`, `StudySessionPage.tsx:198` | A position claim with no persisted count behind it | **§6.4** |
| `0′ of 20′` frozen elapsed readout | `SessionToolbar.tsx:65` | `elapsed_minutes` is written once as `0` and never updated | **§6.4** |
| Seeded `workspaceActivity` events ("While you were away") | `mockData/workspaceActivity.ts` via seed | Fabricated history, but reachable only via the 30-day context-recovery variant | §7.2 |
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
| `FALLBACK_WORKSPACE_ID = 'workspace-calculus-ii'` | `src/App.tsx:28` | A mock id as a production fallback | **§6.3** |
| `useCommandPalette(workspaceId = 'workspace-calculus-ii')` | `src/hooks/useCommandPalette.ts:26` | Same | §7.2 |
| "Use template", "Try it first", "Add to another workspace", "See in concept map", "Save to concept notes", "Pin to notes", "Share", "N more notes" | Marketplace / ModuleDetail / ConceptView / StudySession / FullVis | Buttons with no destination | §7.2 |
| Canned tutor answer written into `tutor_exchanges` | `src-tauri/src/commands/session.rs:328` | **The only defect here that writes fiction into the learner's own record.** First item in §7.2 | §7.2 |

One near-miss, checked and left alone: `Stage3StubRouteMenu` navigates to a nonexistent
`sessionId: 'session-1'` (`src/components/workspace/Stage3StubRouteMenu.tsx:17`), but returns
`null` unless `import.meta.env.DEV`, so it never ships. No change.

### 5.2 Wire to persisted domain state

| Item | Why | Path |
|---|---|---|
| `sessions.last_activity_at` | There is nothing correct to order active-session selection by | **§6.2** |
| `workspaces.last_activity_at`, written on real activity | Makes the active workspace derivable at boot without stored UI state | **§6.2** |
| `startSession` reuse semantics | "Practice this" must reopen the concept's open session, not reset it | **§6.2** |
| Boot route and active workspace, derived | The only way criterion 2 is true rather than nominally supported | **§6.3** |
| `sessions.elapsed_minutes` | See §9 decision B | **§6.4** |
| Concepts in a learner-created workspace | `create_workspace_handler` inserts a workspace, a goal, and offline rows — no concepts, so Practice is unreachable outside the sample | §7.1 |

### 5.3 Legitimate sample content — keep

The 87 Calculus II concepts with names and chapters, three sample workspaces, guiding goals,
the module catalog, workspace templates, material and its chapter segments. All of it is
curriculum: pre-authored content a learner could plausibly have installed, making no claim
about what they did. It stays, behind the same explicit opt-in it has today.

### 5.4 Inert chrome — keep

Rotate / Slice / Revolve / Cross-section, zoom controls, bounds sliders, the Inspector. These
read as tool chrome rather than as claims about the learner, and `AXIOM-HANDOFF.md` §4
Screen 6 specifies the visualization page as inert until an engine exists. Unchanged by §6;
§7.2 decides whether they get an explicit disabled state.

### 5.5 Deferred — Stage 9

Real 3D rendering, a real tutor capability, marketplace backend, module sandboxing, offline
sync, problem families beyond `shell_y_poly`.

## 6. The beta-critical path

Five tasks, in execution order. `070` and `071` keep the ids and subjects the walk used, so
they supersede those observations rather than duplicating them.

| step | id | title | owner | depends on | criteria |
|---|---|---|---|---|---|
| 1 | 072 | Sample seed carries curriculum, not activity | codex | — | **6** |
| 2 | 071 | Active-session selection and reuse | codex | — | **1, 2, 6** |
| 3 | 070 | Startup restoration from domain state | claude | 071, 072 | **2** |
| 4 | 073 | Study Session — remove contradictory content | antigravity | 071 | **6** |
| 5 | 074 | Native regression — mid-attempt restart | claude | 070–073 | verifies 1, 2, 6 |

`072`, `071` and `073` are independent enough to run in parallel if three agents are free;
the sequence above is the safe serial order and the one the dependency column enforces.

### 6.1 — `072` Sample seed carries curriculum, not activity

Removes `sessions` from `SampleWorkspaceSeed` (`models.rs:284`), from `import_seed`, and from
`sampleWorkspaceService.ts`'s payload — along with the `tutor_exchanges` and
`session_settled_conclusions` rows `insert_sessions` writes. Zeroes `progress` and drops
`lastActivityAt` in `mockData/workspaces.ts`.

Scope is deliberately the seeded *sessions* and the two workspace columns §6.3 reads, not
seed hygiene in general. Mastery states, diagnostics, notes, heuristics and activity events
are also fabricated history, but none of them causes the dead end and none is read by the
boot derivation — they are §7.2.

**`src/test/mockBackend.ts` is updated in the same task.** It seeds itself from `mockSessions`
at line 83, which is precisely the divergence that let the frontend suite stay green through
this defect (§4.3). Leaving the double richer than the real backend preserves the blind spot.
`mockData/sessions.ts` is not deleted — `mockBackend` may still construct sessions for tests
that need one; it stops shipping them as seed.

**Acceptance:** immediately after sample import, `getActiveSessionByWorkspace` returns null
for every sample workspace and Home renders no Continue card; the Shell method concept still
resolves `shell.method_vertical_axis` and "Practice this" still generates a real problem; the
existing crosswalk guard still passes.

### 6.2 — `071` Active-session selection and reuse

Supersedes the walk's task 071.

Adds `sessions.last_activity_at` and starts writing `workspaces.last_activity_at`, both on
session start, resume, pause, `nextProblem`, and attempt evaluation.
`get_active_session_by_workspace_handler` orders by `sessions.last_activity_at` descending,
tie-broken on `rowid` descending — newest activity wins rather than oldest row.

`start_session_handler` gains reuse: given `(workspace_id, concept_id)` with an existing
non-completed session, return that session instead of inserting. If its `current_attempt_id`
is null or its attempt is solved, bind a fresh attempt onto it — the session persists, the
problem advances. This is what makes "Practice this" a resume rather than a reset, and it is
§9 decision A.

Touches a new migration, `src-tauri/src/commands/session.rs`, `commands/models.rs`,
`commands/workspace.rs`, and the session tests.

**Acceptance:** with a seeded session present in the same workspace, a newly started session
is the one Home returns; pressing "Practice this" twice yields one session row and the same
prompt both times; pressing it after solving yields the same session with a new attempt;
`workspaces.last_activity_at` advances on session activity and on nothing else; the existing
`start_session` degradation branches (unmapped concept, module disabled, invoke failure)
still succeed.

### 6.3 — `070` Startup restoration from domain state

Supersedes the walk's task 070. Frontend only — no new persistence, no stored route.

```
launch
 └─ workspaces.length === 0  → firstLaunch
 └─ else → home
      activeWorkspaceId = the workspace with the greatest last_activity_at
      Home's Continue card = getActiveSessionByWorkspace(activeWorkspaceId)
                           → studySession → describeAttempt(current_attempt_id)
```

Boot blocks first paint on `getWorkspaces()` so the app never renders First Launch and then
corrects itself. `FALLBACK_WORKSPACE_ID` is removed as part of this — the fallback exists
only because nothing establishes a real active workspace at boot.

Touches `src/App.tsx`, `src/hooks/WorkspaceProvider.tsx`, and a new
`src/hooks/useRestoredContext.ts`.

**Criterion 2 wording.** This restores the correct in-progress attempt and puts it one click
from launch; it does not land the learner *inside* the session. `064`'s criterion 2 should be
recorded in those terms rather than left to read as the stronger claim. Booting directly into
the active session was considered and rejected as a product decision that does not belong in
a defect fix.

**Acceptance:** relaunching with at least one persisted workspace never shows First Launch;
relaunching with zero workspaces still does; the restored active workspace is the one last
worked in, not `workspaces[0]`; Home's Continue reopens the exact bound attempt from before
the restart.

### 6.4 — `073` Study Session: remove contradictory content

Empties the prefilled working default. Replaces the visualization pane's hardcoded region
copy with an inert placeholder naming no specific region. Makes `SessionToolbar`'s
`problemCount` optional so the five-dash indicator renders only when a real count exists and
the counter reads "Problem 3" rather than "Problem 3 of 1". Resolves the frozen elapsed
readout per §9 decision B.

Scope is contradictions and unbacked numbers *on this screen*. The tutor pane is not in this
task — its canned persisted answer is a fabrication rather than a contradiction, and
disabling the input is a product call, so it heads §7.2.

**Acceptance:** the working area opens empty; no on-screen copy names a region that
contradicts the current attempt; no problem count or position is displayed unless persisted;
the generate / evaluate / hint / next behaviour observed in the walk is unchanged, asserted by
the existing `StudySessionPage` and `useAttempt` tests passing untouched.

### 6.5 — `074` Native regression: mid-attempt restart

Extends `e2e/` — the one harness that crosses render → IPC → SQLite. One flow:

sample import → Concepts → Shell method → "Practice this" → assert a prompt with no
unsubstituted placeholders → wrong answer → assert "That does not match yet." and a hint →
correct answer → assert "Correct." → "Next problem" → assert the prompt changed → **terminate
mid-attempt and relaunch against the same `XDG_DATA_HOME`** → assert the app is not on First
Launch, and that Home's Continue reopens the same prompt.

Plus one negative assertion that would have caught this class on its own: immediately after
sample import, Home renders no Continue card.

This is the coverage `067` and `068` do not provide. `067` is a backend corpus; `068` is a
network-disabled acceptance test. Neither renders a page.

### 6.6 What the beta path does *not* settle

**Criterion 1 is met in the sample workspace only.** A learner-created workspace has no
concepts (`create_workspace_handler` inserts a workspace, a goal, and offline rows), so
"Practice this" is unreachable in one. Fixing that is §7.1, deliberately outside this path.
`064`'s criterion 1 should record the caveat explicitly rather than read as a general claim —
that is the honest status after §6 lands, and stating it is cheaper than discovering it at
the next walk.

## 7. Separately approved follow-ups

Filed as `proposed`, blocked on their own decision, and **not** treated as consequences of
the Windows walk.

### 7.1 — `075` Workspace curriculum provisioning

A learner-created workspace has no concepts, so the beta loop is demonstrable only inside
sample data (§6.6). The proposal: an additive sixth Practice capability, `practice.concepts@1`,
returning the concept ids and names the module can serve; `create_workspace_handler` invokes
it through `ModuleRegistry` and inserts the returned concepts with `knowledge_concept_id` set,
degrading to zero concepts when Practice is absent or failing, as `start_session` already does.

**Why it needs its own approval.** `CORE.md` §1 states Core has no knowledge of any subject's
internals. Core storing opaque strings a module produced preserves the letter of that, but it
is new boundary surface, and `CORE.md` already documents `knowledge.query@1` as a required
capability nothing implements. Choosing the narrow `practice.concepts@1` over building
`knowledge.query@1` properly is a real architectural fork with Stage 9 consequences, and it
should be decided as one — not adopted because a beta walk happened to expose the gap.

### 7.2 — `076` Application-wide affordance purge

Everything in §5.1 marked §7.2: the canned tutor answer first (it is the only defect that
writes fiction into the learner's record), then the seed's mastery states, diagnostics, notes,
heuristics and activity events; `FullVisualizationPage`'s single hardcoded scene;
`WorkspaceOverviewPage`'s Recent list, fabricated recommendation and index-derived recency;
`CreateWorkspacePage`'s inference chips; `useCommandPalette`'s `workspace-calculus-ii` default;
and every button with no destination.

**Why it needs its own approval, and probably splitting.** It spans six pages and three
ownership lanes; several items diverge visibly from `AXIOM-HANDOFF.md` screenshots (the sample
workspace shown mid-journey, Screen 5's tutor panel); and dropping seeded mastery states is a
product decision about what a sample workspace is *for*, not a bug fix. Expect this to become
three or four small tasks once the shape is agreed, led by the tutor item.

## 8. Testing

Each task carries coverage in its own layer. What is new is the seam:

- **Rust** — `071`'s selection ordering, reuse branches, and activity-timestamp writes;
  `072`'s "no session after sample import" assertion.
- **Frontend** — `070`'s boot cases against `mockBackend`; `073` asserting *absence*, which is
  the harder direction and the one that was missing.
- **Native** — `074`, the only test in the repo exercising render → IPC → SQLite → restart →
  render.

`072`'s update to `mockBackend.ts` is load-bearing for all of it. A test double richer than the
backend it doubles is how this defect survived a green pipeline; the fix is to make the double
honest, not to add tests around it.

## 9. Decisions requiring approval on the beta path

Two. Everything else that needed a decision has moved to §7, where it is decided on its own
terms.

**A — `startSession` reopens rather than creates (§6.2).** A behaviour change to a command
locked in Stage 7. It is what "Practice this" has to mean if `063`'s binding is to survive a
second press, but it changes an existing contract's meaning and should be an explicit decision
rather than folded in as a bug fix.

**B — `elapsedMinutes`: wire it, or remove the readout (§6.4).** Recommendation: wire it on
pause, resume and `nextProblem`. Elapsed time against an intended target is context, not a
score, so `AGENTS.md:22` permits it — and the five-dash indicator derived from `problemIndex`
is the element that genuinely reads as scoring, which `073` removes regardless. Removing the
readout entirely is the smaller change if you would rather not touch session timing now.

Decisions deferred with their tasks: Core writing `concepts` from a module capability (§7.1);
sample mastery states, the tutor panel, and the affordance purge's handoff divergences (§7.2).

## 10. What this design commits to

**Removed now (§6):** seeded sessions, their tutor exchanges and settled conclusions; seeded
workspace progress and last-activity; the `rowid` selection; the hardcoded first-launch route
and `FALLBACK_WORKSPACE_ID`; prefilled working; contradictory visualization copy; unbacked
problem counts and the five-dash indicator.

**Wired to real data now (§6):** session and workspace last-activity; active-session
selection; session reuse on "Practice this"; boot workspace and route, derived; session
elapsed time (pending decision B).

**Labelled:** nothing. Every item considered for labelling was either removable or genuinely
inert chrome. Adding a "sample" badge to make dishonest content acceptable was rejected.

**Not persisted:** the route. No `app_state` table, no `localStorage`, no stored UI position.

**Deferred to their own approval (§7):** workspace curriculum provisioning; the canned tutor
answer and the rest of the affordance purge; the sample's mastery history.

**Deferred to Stage 9:** the real tutor, the visualization engine, marketplace backend,
`knowledge.query@1` as a general capability.
