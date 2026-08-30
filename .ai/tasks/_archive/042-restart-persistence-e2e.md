---
id: 042
title: Restart-persistence E2E coverage
status: done
owner: codex
stage: 7
depends_on: [040]
---

## Scope

Prove `ROADMAP.md` Stage 7's acceptance criterion **"Data survives an app restart"** — the
one Stage 7 criterion no test currently covers. Extends 040's native WebDriver harness with a
second session against the same application data, rather than adding a new harness or a new
kind of test.

Explicitly not in scope: broader E2E coverage of other screens (040's scope note and
`AGENTS.md` §Testing both cap this at a small number of high-value flows), and any change to
the production command surface.

## Plan

- `e2e/` — either a second `test()` in `first-launch-to-home.test.mjs` or a sibling
  `restart-persistence.test.mjs`, whichever keeps each file single-purpose.

## Worklog

- 2026-08-29 (claude-code, from 040's review): Why this is a real gap and not a theoretical
  one — verified rather than assumed:
  1. Every Rust test opens an in-memory database (`commands/tests.rs:10`,
     `db/tests.rs:53,85,112`). Grepped for any file-backed `Database::open` in a test: there
     is none. So nothing in `cargo test` exercises a database that outlives its process.
  2. 040's E2E flow uses a single WebDriver session, so it proves a write-then-read round
     trip within one app lifetime, not across one.
  3. 040 was the last Stage 7 task, so without this the stage would close with a stated
     acceptance criterion unproven.
- 2026-08-29 (claude-code): Implementation note that should save the next agent the
  discovery — 040's harness already does the hard part. `first-launch-to-home.test.mjs:66`
  creates a temporary `XDG_DATA_HOME` and passes it to the `tauri-driver` process env
  (`:73`), which is what isolates the app's SQLite file. The shape here is: create a
  workspace in session one with a distinctive name, `driver.quit()`, build a second driver
  against the **same** `isolatedDataHome`, and assert the workspace is still listed on Home
  without recreating it. Keep the temp-directory cleanup in a single outer `finally` so both
  sessions share one lifecycle.
- 2026-08-29 (claude-code): Correction to the note above, found by checking the code rather
  than trusting the plan — **"assert the workspace is still listed on Home" is not reachable
  on boot.** `App.tsx:175` hardcodes `initialRoute={{ type: 'firstLaunch' }}`, so the app
  always starts on First Launch no matter what SQLite already holds, and
  `FirstLaunchPage.tsx:71-85` offers no path to Home that doesn't first create or import
  something. So session two's shape is: Continue → Create Workspace → create a **second**
  distinctly-named workspace → land on Home → assert **both** names are listed. The
  session-one name appearing next to a workspace this session never created is the proof
  that data crossed the process boundary. Do not add a boot-routing change to make the
  simpler assertion work — see Follow-ups.

- 2026-08-29 (claude-code): Environment prerequisite, same as 040 — this cannot run without
  `WebKitWebDriver` on `PATH` (`sudo apt-get install webkitgtk-webdriver` on Ubuntu 26.04;
  `webkit2gtk-driver` on 22.04/24.04). See `e2e/README.md`.

## What was built / tested / left out

- Built `e2e/restart-persistence.test.mjs`, a native two-session WebDriver flow that creates
  one workspace, fully quits its WebDriver session and `tauri-driver`, then starts a new
  session against the same temporary `XDG_DATA_HOME`, creates a second workspace, and
  asserts Home lists both names. One outer `try`/`finally` owns both sessions, both child
  processes, and the single temporary-directory cleanup.
- `npm run typecheck` — passed with zero errors.
- `npm run lint` — passed with zero errors or warnings.
- `npm run build` — passed (`tsc` and Vite production build).
- `node --check e2e/restart-persistence.test.mjs`, `npx prettier --check
  e2e/restart-persistence.test.mjs`, and `git diff --check` — passed.
- `npm run test:e2e:linux` — not run: no `WebKitWebDriver` (or equivalent WebKitGTK driver)
  executable is available on `PATH`. `tauri-driver` is installed at
  `/home/marcus/.cargo/bin/tauri-driver`, but without the native WebKit driver it cannot
  launch the application session. This is an environment blocker, not an E2E pass.
- Left out the returning-learner boot-route behavior and all other screens, as scoped.

## Review
Reviewer: claude-code
Date: 2026-08-30
- [x] Correctness — pass. Two-session flow matches the corrected plan note: session one
      creates a workspace and fully exits (`driver.quit()` + `stopProcess`) before session
      two starts against the same `XDG_DATA_HOME`; session two creates a second, distinct
      workspace and Home is asserted to list both names, which is the only way to prove
      persistence given `App.tsx:175`'s unconditional first-launch boot. Cleanup is a single
      `finally` covering both driver processes and the temp dir, matching 040's pattern.
- [x] Architecture conformance — pass (test-only change, no app code touched).
- [x] UI rules — pass (no app UI touched; no hardcoded design values).
- [x] Process — pass. Re-ran `npm run typecheck`, `npm run lint`, `npm run build`,
      `node --check`, and `npx prettier --check` myself — all clean, matching the task
      file's claims. `npm run test:e2e:linux` genuinely cannot run here: confirmed
      `tauri-driver` is on `PATH` (`~/.cargo/bin`) but no `WebKitWebDriver` binary is
      installed on this machine (`dpkg -l | grep webkit` shows the WebKitGTK libraries, not
      the driver package) — same blocker 040 hit, disclosed the same honest way rather than
      claimed as a pass. Worklog and follow-ups are detailed and accurate.

This leaves ROADMAP.md Stage 7's "data survives an app restart" criterion covered by a test
that is correct by inspection but not yet executed, same position 040 was merged from. Task
043 (still open, needs a human decision) is what turns this from "written" into "enforced" —
not a blocker on this task, since it mirrors approved precedent, but flagging so Stage 7
isn't considered fully closed until either this runs green once or 043 settles the gating
question.

Verdict: pass

## Follow-ups

- **Should a returning learner ever see First Launch?** `App.tsx:175` boots to
  `firstLaunch` unconditionally, so someone with existing workspaces is asked "What are you
  learning?" every launch and has no way to Home except creating or importing something.
  That is a product decision about first-run detection, not a test concern, so 042 works
  around it rather than fixing it. Worth its own task.
