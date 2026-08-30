---
id: 040
title: Playwright/Tauri-driver E2E flows
status: done
owner: codex
stage: 7
depends_on: [039]
---

## Scope

A small number of E2E flows for the highest-value paths (first launch → create workspace → home), not full-screen coverage, per `AGENTS.md` §Testing.

## Plan

- e2e/first-launch-to-home.spec.ts (or equivalent)

## Worklog

- 2026-08-29 (codex): Claimed after reading `AGENTS.md`, `ARCHITECTURE.md`, the task,
  lifecycle, quality gates, and Stage 7 acceptance criteria. No other task in `.ai/tasks/`
  is in progress, so there is no overlap with the new E2E harness or Tauri configuration.
  Beginning with a local environment probe before adding dependencies or specifications.
- 2026-08-29 (codex): Verified the native stack before writing the flow. This environment
  is WSL2 on Ubuntu 26.04 with WSLg, WebKitGTK 2.52.3, and Xvfb already present, but it lacked
  both driver binaries. Installed exact `tauri-driver` 2.0.6 with Cargo. Ubuntu 26.04 names
  the native package `webkitgtk-webdriver`; passwordless sudo is unavailable here, so I
  downloaded its exact 2.52.3 `.deb` and extracted `WebKitWebDriver` to a temporary unprivileged
  path for verification. A release build launched successfully through `tauri-driver` under
  Xvfb, negotiated a real `wry` session, and exposed the first-launch DOM. CI must install
  `webkitgtk-webdriver` plus `xvfb` (older Ubuntu uses `webkit2gtk-driver`) and exact
  `tauri-driver` 2.0.6.
- 2026-08-29 (codex): Added the native harness and one bounded flow. The exact-pinned
  `selenium-webdriver` client talks directly to `tauri-driver`; Playwright is not used because
  it cannot attach to Tauri's native W3C WebDriver transport. The runner builds the release
  binary, allocates isolated driver ports and a temporary `XDG_DATA_HOME`, creates a workspace
  through the real UI/IPC/SQLite path, asserts Home and the new workspace, then tears down the
  session, drivers, and data. `npm run test:e2e:linux` passed under Xvfb; the native flow took
  about 1.2 seconds after the incremental release build.
- 2026-08-29 (codex): The first full unit run revealed that Vitest's broad discovery glob
  also picked up the Node-native E2E file. Added an explicit `e2e/**` exclusion so the native
  flow only runs through its prerequisite-aware script. Final gates passed: frontend
  typecheck, lint, build, all 57 Vitest files / 135 tests, Cargo formatting and check, all 14
  Rust tests, and the one native E2E flow. Scoped Prettier and `git diff --check` passed; no UI
  styling or hardcoded design values were introduced. Moved to review.
- 2026-08-29 (codex): Resumed after review. The blocking assertion correctly identified a
  false signal: the first-launch subject is not passed into the create-workspace route, so
  typing the fixture-default “Calculus II” there did not determine the inserted workspace
  name. Updating the flow to replace the Create Workspace form's bound Subject value with a
  distinctive E2E name and assert that exact name after SQLite round-trip. Restart persistence
  coverage and the stale Claude-owned policy docs remain recorded review follow-ups, outside
  this correction's scope.
- 2026-08-29 (codex): Corrected and independently reran the native flow after re-extracting
  Ubuntu's WebKit driver. It now replaces the bound Create Workspace subject with “Axiom E2E
  Subject,” verifies the input value, submits through real IPC, and asserts that distinctive
  name on Home; the flow passed in 0.9 seconds. Re-ran frontend typecheck, lint, build, all 57
  Vitest files / 135 tests, Cargo formatting/check/Clippy, and all 14 Rust tests successfully.
  Scoped Prettier and `git diff --check` are clean. Returned to review.

## What was built / tested / left out

Built:

- Added `e2e/first-launch-to-home.test.mjs`, a self-contained native WebDriver runner that
  manages dynamic ports, driver lifecycle, isolated application data, real UI interactions,
  and cleanup.
- Added exact `selenium-webdriver` 4.48.0 dependency and npm scripts for the unbundled release
  build, direct E2E run, and Linux/Xvfb run.
- Added Linux/WSL and CI prerequisite documentation, including the Ubuntu 26.04 driver package
  rename and WebKitGTK 2.52 typing workaround.
- Updated `ARCHITECTURE.md` for the new top-level E2E testing surface and kept Vitest's unit
  discovery isolated from it.

Tested:

- Manually negotiated a raw `tauri-driver` session against the release Axiom binary before
  implementing the spec.
- `npm run test:e2e:linux` built and passed the complete first launch → create workspace →
  Home flow under Xvfb using real Tauri IPC and SQLite; the corrected rerun proved the inserted
  subject with a distinctive value from the form that actually owns it.
- Re-ran the prepared native test after harness isolation changes, plus all frontend and
  backend gates named in the final Worklog entry.

Left out:

- No additional screens or error permutations were automated; Stage 7 explicitly calls for a
  small number of highest-value flows rather than full-screen coverage.
- No CI workflow was added. The repository has no existing CI directory or provider contract;
  `e2e/README.md` records the exact packages and command a future Linux job needs.
- No Rust WebDriver plugin or application-only test hooks were added; the flow uses the
  external native driver boundary and production command surface unchanged.

## Review

### Round 1 — changes-requested

Reviewer: claude-code
Date: 2026-08-29

Reran independently and clean: `npm run typecheck`, `npm run lint`, `npm run build`,
`npm test -- --run` (57 files / 135 tests — confirms the `e2e/**` Vitest exclusion works and
didn't drop unit coverage), `cargo test --locked` (14 tests), `cargo fmt --check`,
`cargo clippy --all-targets --locked -- -D warnings`, and `git diff --check`.

**I could not rerun the native E2E flow itself, and want that on the record rather than
implied.** `WebKitWebDriver` is not installed on this machine (`apt-cache policy
webkitgtk-webdriver` → `Installed: (none)`) and the copy Codex extracted from a `.deb` to a
temporary path is gone. I attempted the run twice: first `spawn tauri-driver ENOENT` (cargo's
bin dir isn't on a non-interactive `PATH`), then with `TAURI_DRIVER_BIN` and `PATH` fixed,
`tauri-driver` itself failed with `can not find binary WebKitWebDriver in the PATH`. This is
environmental, not a defect in the harness — the human can satisfy it with the README's
`sudo apt-get install webkitgtk-webdriver` (Codex had no passwordless sudo). But it means
this review verifies the harness by reading it, not by watching it pass.

- [ ] **Correctness — FAIL. One blocking finding.**

      **The flow's headline assertion cannot fail for the reason it appears to test.**
      `e2e/first-launch-to-home.test.mjs:100` types `'Calculus II'` into the first-launch
      subject field, and `:109-112` asserts a workspace containing `'Calculus II'` appears on
      Home. But that subject is never propagated: `FirstLaunchPage.tsx:23-25` stores the
      trimmed subject in its own local `useState` and then calls `navigate({ type:
      'createWorkspace' })`, and the `createWorkspace` route variant carries no payload
      (`hooks/navigationContext.ts:7`). `CreateWorkspacePage.tsx:26` independently hardcodes
      `useState('Calculus II')`. So the workspace is named `'Calculus II'` because of that
      literal default, not because of anything the test typed — the assertion would pass
      unchanged if line 100 were deleted outright, or if the first-launch input were removed
      from the page entirely. The test drives a step it does not verify.

      To be fair about what the flow *does* prove: the SQLite round trip is genuine. A failed
      insert would throw in `CreateWorkspacePage.tsx:41-46`, render the error alert, and never
      navigate, so the `[data-route="home"]` wait at `:106` would time out — and Home's list
      is read back through `getWorkspaces()` → `invoke` → SQLite. That core value is real and
      is why this is a fix, not a rewrite.

      Fix: type a distinctive subject into the **Create Workspace** form's Subject field
      (`CreateWorkspacePage.tsx:63` — the field that actually feeds `createWorkspace`), e.g.
      `'Axiom E2E Subject'`, and assert on that string. Typing it at first launch instead will
      not work, for the reason above. Leaving as a finding rather than fixing it here, per
      `CLAUDE.md`.

- [x] Architecture conformance — pass. `e2e/` is a sibling top-level directory that touches no
      `src/` layer, and `ARCHITECTURE.md` §2 and §6 were updated in the same task as
      `AGENTS.md` §Docs requires. The `configDefaults.exclude` + `'e2e/**'` addition
      (`vite.config.ts:35`) correctly keeps Node-native tests out of Vitest discovery without
      clobbering Vitest's own defaults. No production code was modified to make the app
      testable — no test hooks, no WebDriver plugin in `src-tauri/` — so the flow crosses the
      real external driver boundary against the unchanged production command surface.
- [x] UI rules — pass, n/a. No component, style, or markup changed.
- [x] Process — pass. Worklog is dated per entry and genuinely useful: the environment probe
      (WebKitGTK 2.52.3, the Ubuntu 26.04 `webkitgtk-webdriver` rename, the missing
      passwordless sudo) is exactly the kind of finding that saves the next person an hour, and
      `e2e/README.md` carries it forward. Scope stayed at one flow as the task specified.
      Deviating from the task title's "Playwright" is **correct, not a violation** —
      Playwright cannot attach to Tauri's native W3C WebDriver transport; `selenium-webdriver`
      is one of Tauri's two documented clients. That deviation does need a doc patch, tracked
      as a follow-up below rather than held against this task.

Verdict: **changes-requested** — one blocking finding under Correctness. Everything else
passes. Once the assertion discriminates, this is ready to approve.

### Round 2 — approved

Reviewer: claude-code
Date: 2026-08-29

Reran independently and clean: `npm run typecheck`, `npm run lint`, `npm run build`,
`npm test -- --run` (57 files / 135 tests), `cargo test --locked` (14 tests),
`cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`, scoped
`npx prettier --check` on the changed files, and `git diff --check`.

**The native E2E flow is still the one gate I cannot rerun.** Codex re-extracted the WebKit
driver to verify its own fix, but that copy is gone again — `apt-cache policy
webkitgtk-webdriver` still reports `Installed: (none)`, and no `WebKitWebDriver` binary
exists on this machine. So round 2 verifies the correction the same way round 1 did: by
reading it and by proving the assertion's discriminating power statically. Anyone wanting the
flow itself green needs `sudo apt-get install webkitgtk-webdriver` first.

- [x] **Correctness — now pass. Round 1's blocking finding is genuinely resolved**, not
      papered over:
      1. The first-launch subject typing is gone entirely (`first-launch-to-home.test.mjs:97`
         now just clicks Continue), which is the honest shape — `FirstLaunchPage` still gets
         traversed and its navigation still proven, but the test no longer implies it
         determines anything downstream.
      2. The subject is now set on the form that actually owns it (`:100-111`), located
         through its label rather than a brittle positional selector, with `Ctrl+A` clearing
         the bound `'Calculus II'` default before typing.
      3. `'Axiom E2E Subject'` appears nowhere in the repository except this test — grepped
         the whole tree to confirm no default, fixture, or seed can produce it. So the Home
         assertion at `:118-121` can now only pass if that exact string survived the round
         trip through `createWorkspace` → IPC → SQLite `INSERT` → `getWorkspaces()` → render.
         Deleting the typing step would now fail the test, which is precisely what round 1
         said it should do.
      4. Nice touch beyond what the finding asked for: the intermediate assertion at `:112`
         checks the input's value *before* submitting. That self-guards the `Ctrl+A` step — if
         select-all silently failed under WebKitGTK, the value would read
         `'Calculus IIAxiom E2E Subject'` and fail there with an obvious message, rather than
         failing confusingly later at the Home assertion.
- [x] Architecture conformance — pass, unchanged from round 1. The correction touched only the
      test body; no production code, config, or contract moved.
- [x] UI rules — pass, n/a.
- [x] Process — pass. The Worklog entry for the fix states plainly *why* the original
      assertion was a false signal rather than just recording that it changed, and correctly
      keeps restart-persistence and the stale-docs items as follow-ups instead of quietly
      widening scope.

Verdict: **approved.** No remaining blocking findings. The follow-ups below are unaffected by
this round and still stand — in particular, Stage 7 should not be declared complete until the
restart-persistence gap has its own task.

### Addendum — the E2E flow has now actually been run

Reviewer: claude-code
Date: 2026-08-29

Correcting the record: both rounds above state I could not run the native flow and verified
it by reading. That caveat is now obsolete, and the claims it hedged are confirmed by
execution rather than inspection.

A whole-filesystem search turned up Codex's extracted driver still present at
`/tmp/tmp.LU82hezeYF/root/usr/bin/WebKitWebDriver` — earlier searches missed it because
`mktemp`'s random directory names match none of the obvious `*webkit*` / `*driver*` globs.
Running with that directory prepended to `PATH`:

1. **The flow passes.** `ok 1 — first launch creates a workspace through SQLite and reaches
   home`, 1.14s, matching Codex's reported timing.
2. **The corrected assertion genuinely discriminates — proved by mutation, not by reading.**
   Changed the typed subject to `'Zzz Mutant Name'` and updated the pre-submit value
   assertion to match it, leaving *only* the Home assertion still expecting
   `'Axiom E2E Subject'`. The test failed with an `AssertionError` at that Home assertion.
   So the Home check does depend on the real persisted name surviving `createWorkspace` →
   IPC → SQLite `INSERT` → `getWorkspaces()` → render. This is the exact property round 1
   found missing, now demonstrated rather than argued. Test file restored via
   `git checkout` afterwards; working tree verified clean.
3. **`XDG_DATA_HOME` isolation genuinely works.** After three separate driver runs, the
   developer's real database at `~/.local/share/com.axiom.app/axiom.sqlite3` still contains
   zero workspace rows and its mtime predates every run. The harness never touches real
   application data — worth recording because an isolation failure here would be silent
   (the flow would still pass while quietly polluting the developer's database).

The prerequisite finding stands unchanged and still matters for 042 and 043: this only ran
because a temp-extracted driver happened to survive. `webkitgtk-webdriver` is still not
installed system-wide, and `/tmp` extractions do not survive reboots.

## Follow-ups

- **Stage 7 acceptance criterion "Data survives an app restart" is not covered anywhere.**
  Verified rather than assumed: every Rust test opens `Database::open_in_memory()`
  (`commands/tests.rs:10`, `db/tests.rs:53,85,112`), so no test exercises a file-backed
  database at all, and this E2E flow uses a single session. 040 is the last Stage 7 task, so
  this criterion would otherwise go unmet as the stage closes. It is out of 040's stated scope
  ("first launch → create workspace → home"), so per `.ai/lifecycle.md` it belongs in its own
  task rather than being folded in — but the harness now makes it cheap: quit the session,
  start a second one against the same `XDG_DATA_HOME`, assert the workspace is still listed.
  Needs a new `proposed` task before Stage 7 can honestly be called done.
- **Two Claude-owned docs are now stale against this task's outcome** (both need human
  sign-off per `.ai/merge-strategy.md`, so flagging rather than editing):
  1. `AGENTS.md:79` still says "**No E2E yet.**" and `:87` still names Playwright as the tool.
     Both are now wrong — E2E exists, and Playwright was correctly rejected.
  2. `.ai/quality-gates.md:39-42` still lists end-to-end tests under "Explicitly not a gate
     (yet) — not required until Stage 7 introduces a real IPC boundary." Stage 7 has
     introduced it, and this task activates the suite, so that entry should move into a real
     gate with its Linux prerequisites named.
- **Product gap, pre-existing, not introduced here:** the subject a learner types on first
  launch is discarded on navigation (`FirstLaunchPage.tsx:23-25` +
  `navigationContext.ts:7`), and `CreateWorkspacePage.tsx:26` opens pre-filled with a
  hardcoded `'Calculus II'` regardless. A learner who types "Topology" lands on a form that
  says "Calculus II". `AXIOM-HANDOFF.md` Screen 2 doesn't explicitly require carry-over, so
  this is a question to settle rather than a clear spec violation — but the hardcoded default
  reads as mock-era scaffolding that outlived Stage 7. Worth its own task.
- **Diff churn in a Claude-owned doc, non-blocking:** Prettier reformatted all of
  `ARCHITECTURE.md` (`*emphasis*` → `_emphasis_`, full re-padding of the §3 component table) —
  roughly 30 lines unrelated to this task. `ARCHITECTURE.md` was not Prettier-clean before
  (verified against `HEAD`), so this is a first-time normalization rather than a regression,
  and it's harmless. Noting only because it makes the next diff on that table misleading, and
  because a `.prettierignore` decision for `reference/` and Claude-owned Markdown may be worth
  making deliberately.
