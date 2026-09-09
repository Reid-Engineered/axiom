---
id: 068
title: Network-disabled native acceptance test for the Practice loop
status: proposed
owner: unassigned
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

- 2026-09-09 — Filed by claude. Named as "not yet filed" in task `064`'s follow-ups; this
  closes that gap. Scope drawn from `ROADMAP.md:297`, task `064` criterion 3, and the existing
  `e2e/` harness rather than newly invented.

## What was built / tested / left out

## Review

## Follow-ups
