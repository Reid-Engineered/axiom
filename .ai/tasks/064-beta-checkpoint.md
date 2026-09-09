---
id: 064
title: Beta checkpoint — first installable build for external testing
status: proposed
owner: unassigned
stage: 8
depends_on: [062, 063]
---

## Scope

Declares the stopping point for Axiom's first beta: the point at which the human can hand
a built installer to a real learner outside the dev team and have them run a genuinely
useful loop, not just click through empty screens. This task does not itself implement
that loop — it records the goal-line and acceptance criteria, and gates the actual cut.
Beta is cut once every criterion below is met; the specific implementation work happens in
the sub-tasks it names (existing where they exist, `proposed` here where they don't yet).

Does **not** build: any Stage 9+ scope (tutor/AI, real 3D visualization, marketplace
backend, module sandboxing, offline sync, additional problem families beyond
`shell_y_poly`). Those are explicitly out of scope for a first beta per `ROADMAP.md`'s
"Stage 9 and beyond" section — adding them here would be scope creep against an already
large stage, not a fix for this checkpoint.

## Why this is the line

Stages 0–7 (full app shell, all 12 screens navigable, real SQLite persistence surviving
restart) are already `done` per the archived task record — a beta tester could already
click through the app, but there is nothing for them to *do*. `ROADMAP.md`'s own "Remaining
Stage 8 scope" section already names the exact remaining work to make Study Session
functional: problem selection/resume, UI integration, a regression corpus, and an explicit
offline acceptance test. This task adopts that critical path as the beta gate rather than
inventing new scope.

## Acceptance criteria (the beta gate)

A build qualifies as beta-shippable when all of the following hold on a clean checkout:

1. **End-to-end practice loop works.** From a fresh workspace: open a Study Session, get
   served a real generated `shell_y_poly` problem, submit an attempt, see it evaluated with
   structured diagnostic feedback (not a stub or placeholder state).
2. **Session state survives restart.** Closing and relaunching the app resumes the correct
   in-progress attempt for that session, per Stage 7's persistence guarantee extended to
   Practice.
3. **Works fully offline.** The flow in (1) passes an explicit network-disabled native
   acceptance test — no capability in the path makes a network call.
4. **CI is green on required checks**, `backend-checks (windows-latest)` and
   `frontend-checks (ubuntu-latest)` included — i.e., 061 and 062 are `done`, not merely
   "known flakes."
5. **`npm run build` and `cargo check`/`cargo build --release` succeed** and produce a
   launchable installer for at least one desktop platform (Stage 0's acceptance criterion,
   re-verified against the current tree rather than assumed still true).
6. **No dead ends.** Every screen reachable from the practice loop (Study Session, hint
   states, evaluation states) matches `reference/UI/AXIOM-HANDOFF.md`, per Stage 6's
   zero-dead-ends bar.

## Plan

This task tracks the gate; the work is split across:

- **061** (`done`, archived) — Windows SQLite connection-cleanup fix; criterion 4's Windows
  half is no longer blocked.
- **062** (`done`, archived) — `GoalEditingSheet` flake fix; criterion 4's Ubuntu half is no
  longer blocked.
- **063** (`done`, archived) — Study Session ↔ Practice attempt binding, spec'd in
  `docs/superpowers/specs/2026-09-08-study-session-practice-binding-design.md`. The session
  selection/resume piece the rest depend on.
- **065** (`proposed`, owner codex) — the two Tauri commands (`describeAttempt`,
  `nextProblem`) the UI needs. **The single largest blocker: see "Current blocker" below.**
- **066** (`done`, archived) — the frontend half: `useAttempt`, `ProblemPane` rewired to real
  attempts.
- **067** (`proposed`) — Practice regression corpus. Supports criterion 1's "not a stub" bar
  with durable coverage, and criterion 4's CI-green bar going forward.
- **068** (`proposed`) — network-disabled native acceptance test. Directly implements
  criterion 3.

This task's own file gets updated (not the sub-tasks') as each dependency lands, and moves
`proposed → in-progress` once the session-selection design task is filed and claimed.

## Current blocker (as of 2026-09-09)

**`master`'s practice loop does not work in a real build.** Task `066` merged the frontend
that calls `describeAttempt` and `nextProblem`; task `065` — which registers those two
commands in `lib.rs` — has not merged. So `src/services/practiceService.ts:19` and
`sessionService.ts:51` invoke commands the backend does not expose, and the problem pane
cannot hydrate.

Every required check is nonetheless green, because frontend tests run against
`src/test/mockBackend.ts` (which implements both) and the Rust tests never cross the IPC
boundary. This is precisely the seam that the two parallel tasks were split along, and no
existing suite covers it.

The failure is at least **honest** rather than silent: `StudySessionPage.tsx:207` renders the
real error, and the "no practice content for this concept yet" empty state at `:212` only
renders when there is *no* error — so a missing command cannot masquerade as the
`knowledge_concept_id` crosswalk gap.

Merging `065` clears criteria 1, 2 (its end-to-end half) and 6 together. Nothing else should
be attempted against this gate until it lands.

## Criterion status

Verified against the tree at `a7f7b2d4` on 2026-09-09. "Verified" below means a command was
actually run or a check actually observed — not inferred from code.

| # | Criterion | Status |
|---|---|---|
| 1 | End-to-end practice loop | **blocked** — `065` |
| 2 | Survives restart | **half met** — persistence layer verified, loop blocked by `065` |
| 3 | Fully offline | **not started** — `068` builds it |
| 4 | CI green on required checks | **met** — `061`/`062` archived `done`; all 7 green on `a7f7b2d4` |
| 5 | Builds + launchable installer | **met** — see below |
| 6 | No dead ends | **blocked** — `065` |

**Criterion 5 — met.** `npm run build` passes; `cargo check --release` passes;
`cargo build --release` produces a 19MB binary; `npm run tauri build` produces three Linux
installers — `Axiom_0.1.0_amd64.deb` (5.8M), `Axiom-0.1.0-1.x86_64.rpm` (5.8M), and
`Axiom_0.1.0_amd64.AppImage` (81M). Launchability rests on CI's `e2e` job, green on
`a7f7b2d4`, which builds the release binary and drives it through WebDriver; it could not be
run locally because `WebKitWebDriver` is absent from the dev environment (`tauri-driver` is
present). Worth knowing before cutting the beta: **AppImage bundling downloads `linuxdeploy`,
`AppRun` and plugins from GitHub at build time**, so the first build on a clean machine needs
network. That is a build-time requirement and does not bear on criterion 3, which is about
the running app — but `068` should keep the two distinct.

**Criterion 2 — persistence layer verified, end-to-end blocked.** This had no coverage at all
and was untestable by construction: `database()` and `practice_connection()` in
`commands/tests.rs` both build in-memory databases that die with the connection, and
`e2e/restart-persistence.test.mjs` (task `042`) covers workspace data only — it predates
Practice and never touches sessions or attempts. A test now covers the layer that does exist:
a real file-backed database, a session that binds an attempt, both connections dropped and
reopened, asserting the session keeps the same `current_attempt_id` *and* that
`practice.describe` returns the identical prompt — i.e. the learner resumes the same problem,
not merely some problem. The remaining half needs `065`.

## Worklog

- 2026-09-09 — Verified criteria 2 and 5 against the tree at `a7f7b2d4` at the human's
  request, rather than leaving them as inherited assumptions from Stages 0 and 7. Criterion 5
  is met (three Linux installers produced). Criterion 2's persistence layer is verified by a
  new test; its end-to-end half is blocked. Recorded the `065` blocker above, found while
  verifying criterion 2 — `master` currently ships a frontend calling two unregistered
  commands. Added the "Current blocker" and "Criterion status" sections.
- 2026-09-09 — Filed the two remaining unfiled dependencies as `067` (regression corpus) and
  `068` (offline acceptance test), and updated this file's Plan and Follow-ups to point at
  them instead of "not yet filed". Every dependency this checkpoint names now exists as a
  task. Criterion status at time of writing: 4 met (`061`/`062` archived `done`); 1 and 6
  pending `065` merging (`066` already landed the frontend half); 2, 3 and 5 still unverified
  — 2 and 5 have never been checked against the current tree, and 3 is what `068` builds.
- 2026-09-07 — Filed by claude at the human's request, to formalize the beta stopping-point
  discussed in conversation rather than leave it as a verbal agreement. No code changed.
- 2026-09-09 — Renumbered `063` → `064`. The file was written while uncommitted and never
  pushed; meanwhile codex's `063-study-session-practice-binding` took the same id on
  `master`. Dependencies updated to reflect that 061 is now archived `done` and that 063 is
  the filed session-binding task this checkpoint waits on.

## What was built / tested / left out

Nothing yet — this is a gate/checkpoint task, not an implementation task. It will be
updated as the dependent tasks above are filed and completed, and moved to `done` only once
all six acceptance criteria are independently verified (not just claimed) against a real
build.

## Review

## Follow-ups

- ~~File the session-selection/resume task~~ — filed and completed as `063`.
- ~~File the Study Session UI integration task (Antigravity)~~ — filed as `065` (Rust IPC,
  codex) and `066` (frontend, antigravity); `066` is `done`, `065` is in review on PR #11.
- ~~File the Practice regression corpus task~~ — filed as `067`.
- ~~File the network-disabled native acceptance test task~~ — filed as `068`.
