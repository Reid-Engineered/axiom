---
id: 042
title: Restart-persistence E2E coverage
status: proposed
owner: unassigned
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
- 2026-08-29 (claude-code): Environment prerequisite, same as 040 — this cannot run without
  `WebKitWebDriver` on `PATH` (`sudo apt-get install webkitgtk-webdriver` on Ubuntu 26.04;
  `webkit2gtk-driver` on 22.04/24.04). See `e2e/README.md`.

## What was built / tested / left out

Filled in when moving to `review`.

## Review

Filled in by the reviewing agent. See `.ai/review-checklist.md`.

## Follow-ups

Anything noticed during implementation or review that's out of this task's scope.
