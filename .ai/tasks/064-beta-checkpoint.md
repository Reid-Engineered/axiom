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
- **070–075** (`proposed`, owner codex) — mock and dead-end containment, from the 2026-09-09
  walk. `071` active-session selection and reuse; `072` sample seed carries curriculum, not
  activity; `075` workspace curriculum provisioning; `070` startup restoration from domain
  state; `073` Study Session contradictory-state removal; `074` the native regression that
  verifies criteria 1, 2 and 6 together. Claude reviews all six.
- **076**, **077**, **078** (`proposed`) — deferred follow-ups from the same design, held
  outside the beta path by explicit decision: real elapsed-time tracking, the application-wide
  affordance purge, and a conditional visual pass after `073`.

This task's own file gets updated (not the sub-tasks') as each dependency lands, and moves
`proposed → in-progress` once the session-selection design task is filed and claimed.

## Blocker cleared (2026-09-09)

**Resolved.** `065` merged as `23a6dad`, registering `describeAttempt` and `nextProblem` in
`lib.rs`. `master` no longer ships a frontend calling commands the backend does not expose.

The gap that allowed it — frontend tests running against `mockBackend.ts` while Rust tests
never cross IPC, so neither suite looked at the seam — is now covered by `069`
(`src/test/commandRegistration.test.ts`, merged as `e054b7c`). It cross-references every
`invoke(...)` in `src/services/` against the `#[tauri::command]` declarations and the
`generate_handler!` list, and it runs in the frontend suite on every PR. It was verified in
both directions: red while the commands were missing, green once they landed.

## Criterion status

Updated 2026-09-09 after `065` and `069` merged. **"Unblocked" is not "met"** — three criteria
now have nothing standing in their way but have still never been observed working in a real
build, and this table says so rather than inferring success from a green pipeline. That
distinction is the whole reason this checkpoint exists.

| # | Criterion | Status |
|---|---|---|
| 1 | End-to-end practice loop | **not met** — walked 2026-09-09; works in the imported sample only. A learner-created workspace has no concepts, so Practice is unreachable in one. `075` |
| 2 | Survives restart | **not met** — walked 2026-09-09; the app reopens on First Launch and restores nothing. Storage layer still sound. `070`, `071` |
| 3 | Fully offline | **not started** — `068` builds it |
| 4 | CI green on required checks | **met** — all 7 green on `master` |
| 5 | Builds + launchable installer | **met** — three Linux installers produced |
| 6 | No dead ends | **not met** — walked 2026-09-09; Home's advertised "Resume session" opens a session with no practice content. `071`, `072`, `073` |

**What is genuinely proven.** Criterion 5: `npm run build`, `cargo check --release` and
`cargo build --release` all pass, and `npm run tauri build` produces `Axiom_0.1.0_amd64.deb`
(5.8M), `Axiom-0.1.0-1.x86_64.rpm` (5.8M) and `Axiom_0.1.0_amd64.AppImage` (81M).
Launchability rests on CI's `e2e` job, which builds the release binary and drives it through
WebDriver. Criterion 2's storage half: `a_bound_practice_attempt_survives_reopening_the_database_file`
opens a real file-backed database, binds an attempt, drops both connections, reopens, and
asserts both the same `current_attempt_id` *and* the identical prompt — the learner resumes
the same problem, not merely some problem. Criterion 4: green across all seven checks, with
`061` and `062` fixed rather than tolerated as known flakes.

**What is not proven, and cannot be from here.** Nobody has opened the built app, started a
Study Session on the Shell method concept, been served a generated problem, submitted a wrong
answer, seen a hint, submitted the right answer, and advanced to the next problem. Every
individual piece is tested; the composition of them is not. The dev environment lacks
`WebKitWebDriver`, so even the existing e2e harness cannot be driven locally, and the e2e
suite covers first-launch and workspace persistence only — it never opens a Study Session.

**The walk happened on 2026-09-09**, on a Windows release build, and is recorded in the
worklog below. It settled criteria 1, 2 and 6 in the negative: the Practice loop itself works
end to end, and the application around it presents mock fixtures as live domain state.
Containment is designed in
`docs/superpowers/specs/2026-09-09-mock-and-dead-end-containment-design.md` and split across
`070`–`075`. **Criteria 1, 2 and 6 stay unmet until all six land and `074`'s native regression
passes on CI** — no subset settles a criterion, and none is claimed on a green unit suite.

**Build-time note for whoever cuts it:** AppImage bundling downloads `linuxdeploy`, `AppRun`
and plugins from GitHub, so the first build on a clean machine needs network. That is a
build-time requirement and does not bear on criterion 3, which is about the running app.

## Worklog

- 2026-09-09 — **Walked the loop on a Windows release build.** Criterion 1's mechanism passed:
  a real `shell_y_poly` problem generated with every placeholder substituted, a wrong answer
  stayed open and revealed a relevant authored hint, `8*pi` produced restrained "Correct."
  copy, and Next problem regenerated and reset cleanly. Criteria 2 and 6 failed. Restart
  reopened on First Launch despite persisted workspaces in the sidebar, and returning to Shell
  method produced a fresh problem rather than the one left open. Home's seeded "Problem 6 of
  12 · Resume session" opened a Study Session reading "There is no practice content for this
  concept yet." Root-caused by claude to one defect — the app has no representation of the
  session the learner is in — plus a sample import that seeds activity records. Design and six
  tasks filed; criteria 1, 2 and 6 moved from "unblocked, unverified" to **not met**.

- 2026-09-09 — `065` and `069` merged, clearing the blocker recorded above. Criteria 1, 2 and
  6 are now unblocked but remain **unverified**: the practice loop has never been walked in a
  real build. Updated the status table to distinguish unblocked from met, and named the single
  manual pass that would settle all three. Also archived `065` and `069` as `done` — both had
  merged while their records still said `review`.

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
