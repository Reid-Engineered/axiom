---
id: 068
title: Network-disabled native acceptance test for the Practice loop
status: review
owner: codex
stage: 8
depends_on: [063, 065, 066]
---

## Scope

Build the "explicit network-disabled offline acceptance test end to end" that `ROADMAP.md:297`
names as the last item of remaining Stage 8 scope, and that task `064`'s beta gate lists as
criterion 3. It is the test that turns Stage 8's headline claim — Practice generates,
evaluates, and hints **offline** — from an architectural assertion into something CI proves.

The test drives the real release binary through the full learner loop with networking
unavailable: open a Study Session on the mapped Shell method concept, receive a generated
`shell_y_poly` problem, submit a wrong answer and see it rejected with a hint, submit the
correct answer and see it accepted, advance to the next problem. All with no network.

Does **not** build: the regression corpus (task `067`), any new capability or command, any UI
change, offline *sync* (explicitly Stage 9 per `ROADMAP.md`'s "Stage 9 and beyond"), or
network-isolation for anything outside this one flow.

## Prior art to follow

`e2e/` already holds two native acceptance tests driving the release binary through the W3C
WebDriver boundary — `first-launch-to-home.test.mjs` (task `040`) and
`restart-persistence.test.mjs` (task `042`). This is a third file in that directory following
the same shape, not new infrastructure. `e2e/README.md` documents the harness: `tauri-driver`,
pinned `selenium-webdriver`, isolated temporary `XDG_DATA_HOME` per run, `xvfb-run` on Linux.

`npm run test:e2e:linux` is already a required CI check on every PR (`.ai/quality-gates.md`),
so a test added here is enforced from the moment it lands — no CI wiring needed.

## Plan

Files expected to be created or touched:

- `e2e/offline-practice-loop.test.mjs` (new) — the flow above, written in the existing tests'
  style and using their existing helpers rather than a parallel harness.
- `e2e/README.md` — document how the test disables networking and how to run it locally,
  including what to expect on WSLg where the isolation mechanism may differ from CI.
- Possibly `package.json` — only if the chosen isolation mechanism needs its own script; prefer
  making the test self-isolating so `npm run test:e2e:linux` keeps covering everything.

## The open design question: how to actually disable the network

This is the substance of the task and should be settled deliberately, in the handoff doc,
before the test is written. The mechanism has to be **credible** — a test that merely doesn't
happen to make a request proves nothing, because it would pass identically against a build
that phones home. Candidates, roughly in order of strength:

1. **Network namespace isolation** (`unshare -rn`, or a CI container with networking off) —
   the app genuinely cannot reach anything. Strongest evidence; needs care to keep the
   WebDriver/`tauri-driver` loopback connection working, since the harness itself talks to the
   binary over a local socket.
2. **Deny-by-default outbound firewall for the app's user/process** during the run — strong,
   but more fragile across environments and harder to reproduce locally.
3. **Asserting no outbound sockets were opened** (strace/eBPF/`ss` sampling) — observational
   rather than preventative; weaker, since it proves absence in one run rather than
   impossibility.

Whichever is chosen, the test must **fail loudly if the isolation itself silently stops
working** — otherwise it decays into a green test that proves nothing, which is worse than no
test because it carries false authority. State how that failure mode is guarded in the
handoff doc.

Note the harness constraint up front: `tauri-driver` and the WebKit driver communicate with
the app over local sockets, so "no network" must mean *no external network*, with loopback
intact. Getting that distinction right is most of the work.

## Acceptance criteria

- The loop above passes with external networking genuinely unavailable, in CI, on Linux.
- The test fails — visibly, with a clear message — if isolation is not actually in effect.
- Deliberately breaking offline behavior (e.g. a stubbed outbound call inserted temporarily
  in the practice path) makes it fail. Verify this once by hand and record the result in the
  handoff doc; an offline test never observed failing is not yet evidence of anything.

## Worklog

- 2026-11-04 — CI at be1961f passed all three native flow bodies, including offline
  Practice, but never exited and hit the 30-minute job timeout. Driver cleanup now
  kills surviving descendants even when tauri-driver has already exited and closes
  inherited output pipes. Non-isolated drivers get their own process group; isolated
  drivers stay in the wrapper's group so its outer timeout still reaches them. Added
  a fixture reproducing an exited driver with a live descendant holding its pipes.
  Six Linux harness checks and all three native flows passed with normal Node exit;
  syntax, Prettier and whitespace checks passed. Required CI must confirm the fix.

- 2026-11-04 — Codex addressed Claude review of d99effe: the real wrapper now
  requires a child completion marker, has abort/process-group cleanup, and gives the
  outer test 150 seconds versus the child's 120. Added empty-child rejection and
  abort regressions. Replaced the ambiguous CORS-mode negative proof with the same
  no-cors-mutated binary passing online and failing in isolation. Recorded exact
  artifact provenance below and updated ARCHITECTURE.md's native-test paragraph.
  CI's initial native failure was ElementNotInteractableError on the Answer click;
  explicitly scroll the field into view in its scrollable pane before interaction.

- 2026-11-04 — Isolation decision before implementation: re-execute this one native
  test inside `unshare --user --map-root-user --net`; enable only `lo`, and run the
  harness, tauri-driver, WebKit driver and app as descendants in that namespace.
  Build outside isolation. Before launching the app, require a different network
  namespace from the parent, only the loopback interface, no external IPv4/IPv6
  routes, immediate ENETUNREACH for literal external IPv4/IPv6 addresses, and a
  successful loopback socket exchange. Missing permissions/tools or an ineffective
  namespace fail the test, never skip it. CI may need its AppArmor user-namespace
  restriction disabled explicitly; no host firewall or interface changes.
  Baseline is origin/master 0b479f3 (includes 082 and 083); unrelated 067 excluded.
- 2026-11-04 — Implemented the native loop and extracted the existing restart test's
  driver/workspace helpers for reuse. The actual provider concept name is
  `The Method of Cylindrical Shells (Vertical Axis of Revolution)`, not the old sample
  `Shell method` label. 082 renders the problem counter uppercase through CSS, so the
  assertion reads its DOM text content. 083 is contracts only; no production changes
  were needed. Re-fetched master before final validation; baseline remains 0b479f3.
- 2026-11-04 — Initial re-execution exposed Node's inherited `NODE_TEST_CONTEXT`
  suppressing nested test discovery. That apparent green was discarded. The wrapper
  now removes the worker marker, and a focused regression test requires the nested
  test body to verify isolation and write a completion file. Subsequent native runs
  visibly executed the child test (one test, zero skipped).

- 2026-09-09 — Filed by claude. Named as "not yet filed" in task `064`'s follow-ups; this
  closes that gap. Scope drawn from `ROADMAP.md:297`, task `064` criterion 3, and the existing
  `e2e/` harness rather than newly invented.

## What was built / tested / left out

Implementation PR: https://github.com/Reid-Engineered/axiom/pull/31

Built:

- Self-isolating native acceptance through real release assets, WebDriver, IPC and
  SQLite: create workspace, open mapped vertical-shell concept, start Practice,
  reject `0`, reveal automatic and explicit hints, accept an independently calculated
  rational multiple of pi from the visible generated prompt, advance to Problem 2,
  and verify cleared answer/hints/feedback. No fixture import or canonical-answer read.
- Kernel-state and IPv4/IPv6 socket guards before and after Practice. Failure is
  mandatory on unsupported platforms, missing tools, denied namespaces, extra links,
  external routes, unchanged namespace, unexpected socket result or broken loopback.
- Narrow CI prerequisites: iproute2/util-linux, disposable-runner AppArmor setting,
  focused harness checks. Existing required `e2e` job/script still runs every native
  test. README covers commands, WSLg, isolation boundaries and the negative control.

Validation actually executed on 2026-11-04:

- Windows worktree: `npm ci`; `npm run typecheck`; `npm run lint`; `npm run build`;
  `npm run test` — 62 files / 209 tests passed. `node --check` on all five new MJS
  files; Prettier check on every changed e2e MJS/README; `git diff --check` passed.
  Windows focused checks: two passed, Linux namespace regression explicitly skipped.
- Linux native environment: Ubuntu 26.04 in WSL2, kernel 6.18.33.2, Node 22.23.1,
  tauri-driver 2.0.6 (Cargo install metadata), WebKitGTK/driver 2.52.6, Xvfb. Matching
  driver package extracted under the user's cache without system installation.
  Built a snapshot of this worktree under `/home/marcus/.cache/axiom-068/repo`, using
  the existing Linux node_modules (JSON-equivalent package lock) and Cargo target cache.
  No other checkout's source was edited. `npm run build:e2e` passed, then
  `AXIOM_E2E_APP=/home/marcus/axiom/src-tauri/target/release/axiom xvfb-run -a node --test
  --test-concurrency=1 e2e/*.test.mjs` passed all three native flows. The isolated child
  executed the full Practice loop, not just its wrapper. This is the build/run
  equivalent of `npm run test:e2e:linux`, with an explicit cached target path.
- Linux `node --test e2e/harness-checks.mjs`: three passed, zero skipped, including
  rejection of the real parent namespace marker and execution of the nested Node test.
- Negative Practice proof: only in that disposable snapshot, inserted
  `await fetch('https://1.1.1.1/');` immediately before the `describeAttempt` IPC call,
  rebuilt the release app, and ran the same isolated native test. It failed with exit 1
  waiting 10 seconds for the Answer input, after the isolation guard and session entry.
  Restored the source from this unchanged worktree, rebuilt clean assets (original
  frontend hash restored), and reran all three native flows successfully. No mutation
  is committed; the negative log remains in the WSL cache as `negative.log`.

Review remediation validation (supersedes the original CORS-mode negative proof):

- Five Linux `node --test e2e/harness-checks.mjs` checks passed, zero skipped, including
  zero-completion rejection and abortion. Windows syntax/Prettier/whitespace checks
  passed. Production source in this worktree remains unchanged.
- The snapshot is based on PR head `d99effe311deb2ee8d39d24069916d709a49c12b`, plus this
  commit's e2e-only remediation. Build command: `npm run build:e2e` with explicit
  `CARGO_TARGET_DIR=/home/marcus/axiom/src-tauri/target`. That shared Cargo cache's
  release artifacts were overwritten during the disposable mutation and restored by
  the clean rebuild; no other checkout's source was edited. Each tested executable
  was copied immediately to a private `artifacts/<control|clean>/bin/axiom`, with its
  matching knowledge-package under `../lib/Axiom/`, so test runs do not depend on
  later changes to the cache's release binary.
- Mutated `describeAttempt` awaited
  `fetch('https://1.1.1.1/', { mode: 'no-cors', signal: AbortSignal.timeout(5000) })`.
  Exact binary `/home/marcus/.cache/axiom-068/artifacts/control/bin/axiom`, SHA-256
  `2730f171fbb0e6aee7c44e2cd72ce1b29c3df0a6f58a7314c524f12323253e84`:
  the learner assertions passed online with only isolation wrapper/guards/marker
  removed from a disposable test copy, then the real isolated test failed with exit 1
  at the Answer wait after isolation and session entry. Logs: `online-control.log`
  and `offline-control.log` under `/home/marcus/.cache/axiom-068/`.
- Restored `practiceService.ts` directly from this unchanged worktree (both SHA-256
  `98398589966cc63778ef76fae22a4d2b489a0d37c4f258a10f87d6388cbbe202`), rebuilt, then
  all three native flows passed using
  `AXIOM_E2E_APP=/home/marcus/.cache/axiom-068/artifacts/clean/bin/axiom xvfb-run -a node
  --test --test-concurrency=1 e2e/*.test.mjs`. Clean binary SHA-256
  `3c3deb5187fcc77f5d08e3f54b04b97f0b191cf9c192e9b92033bae7f1869711`;
  log `clean-native.log`. Neither the mutation nor online-only control is committed.
  That log includes the explicit Answer scroll added after the initial CI failure.
  The WSL window did not reproduce clipping, so local success does not prove the CI
  window issue resolved; the updated required native CI result remains authoritative.

Limits / gates remaining:

- Required GitHub CI remains a separate merge gate; independent code review passes
  at c5bc294 as recorded below. Local WSL evidence does not replace CI. Native Windows/macOS runs were
  not attempted; the offline acceptance deliberately requires Linux namespaces.
- No Rust or production frontend/service code changed. Separate Cargo test/check/
  clippy/fmt gates were not run locally; the native release build did compile Rust.
  CI retains all existing frontend/backend gates. ARCHITECTURE.md now describes the
  actual native flows, shared helpers and namespace/completion boundary.
- Isolation proves this learner flow works without external networking; it does not
  assert absence of best-effort outbound attempts or test offline sync. No app UI,
  Practice production logic, task 067 files or other task records are included.

## Review

Reviewer: Claude (independent final review, 2026-11-04)
Reviewed implementation head: `c5bc2943a91745967c61ec3e7fb34397ae697f77`

- Correctness: pass. Driver cleanup stops descendants by process group even after
  driver exit, closes inherited pipes, and clears the timer. Isolated drivers remain
  in the wrapper group so outer abort still reaches them. Completion-marker and
  isolation guards are sound; the new fixture reproduces the post-test hang mode.
- Tests: pass by inspection; descendant, aborted-grandchild and empty-child regressions
  assert actual outcomes. Offline answer oracle and unchanged restart assertions hold.
- Architecture: pass; no production behavior changed. README and architecture match
  the process-group rules, cleanup, isolation boundaries and timeout limitations.
- UI rules: not applicable.
- Scope/worklog: pass. Remaining first-launch helper duplication is a named follow-up.

Verdict: pass (independent code review), conditional on all seven required CI checks.

Merge requires green CI on the final head, including native E2E completing with normal
exit within the job limit; any required failure or timeout keeps this task unmergeable.
This review-record commit changes no implementation. Human sign-off is still required
for ARCHITECTURE.md and .ai/ changes. No CI gate or local limitation is waived.

Claude re-reviewed `c821fb8` on 2026-11-04: all five original findings resolved;
correctness/architecture pass apart from the conditional CI clipping risk. Required
CI was still pending. Additional notes: scroll all button targets, prove a grandchild
stops on abort, keep cleanup errors from replacing the original failure, and document
timeout limits and whether the clean log included scrolling.

Codex addressed those notes: the shared click helper scrolls every button target;
the abort fixture launches `sleep 60` and verifies it is absent or killed/zombie after
abortion (host init may reap it later); process cleanup errors are logged without
replacing the primary failure. README documents abandoned temporary data and session
escape limitations. Five Linux harness checks and the native isolated Practice flow
passed again with these changes; syntax, Prettier and whitespace checks passed. The
updated required CI result remains the gate for the runner-specific clipping issue.

Claude reviewed d99effe on 2026-11-04: changes-requested. Isolation guards, loopback,
native Practice assertions, answer oracle and helper extraction passed inspection.
Blocking proof gaps were the ambiguous CORS-mode negative control and wrapper acceptance
of a zero-completion child. Remediation above addresses both; fresh independent review
and CI remain required. Initial CI native run failed on a clipped Answer click; fresh
CI must validate the explicit scroll. No gate is waived.

Human sign-off is required before merging the
task record under `.ai/merge-strategy.md`; implementation agent has not merged or watched.

## Follow-ups

- Consolidate the remaining first-launch test helper duplication in a separately
  scoped maintenance task; restart persistence and offline Practice share helpers now.
- Network namespaces leave filesystem Unix-domain IPC available. This proves the
  learner flow works with external IP networking unavailable, not a general sandbox.
