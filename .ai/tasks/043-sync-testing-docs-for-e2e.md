---
id: 043
title: Sync AGENTS.md and quality-gates.md for the activated E2E suite
status: proposed
owner: claude
stage: 7
depends_on: [040]
---

## Scope

Docs-only. Two Claude-owned policy documents still describe a pre-Stage-7 world and are now
actively wrong about how this repo tests. Both changes need the human's sign-off per
`.ai/merge-strategy.md`, which is why 040's review flagged them rather than editing them
inline.

Docs-only tasks are Claude alone per `AGENTS.md` §Roles.

## Plan

- `AGENTS.md` §Testing
- `.ai/quality-gates.md`

## Worklog

- 2026-08-29 (claude-code, from 040's review): The two staleness points, with the decision
  each one needs:
  1. **`AGENTS.md:79`** still opens with "**No E2E yet.** Nothing end-to-end is worth
     automating until there's a real IPC boundary to cross (Stage 7+)." That boundary now
     exists and is crossed by `e2e/first-launch-to-home.test.mjs`.
  2. **`AGENTS.md:87`** names the tool as "Playwright/Tauri-driver". 040 established — with
     sound reasoning, accepted in review — that Playwright *cannot* attach to Tauri's native
     W3C WebDriver transport, and shipped `selenium-webdriver` against `tauri-driver`
     instead. The doc should name what the repo actually uses, so the next agent doesn't
     re-litigate it or try to "fix" the divergence.
  3. **`.ai/quality-gates.md:39-42`** still lists end-to-end tests under "Explicitly not a
     gate (yet) — not required until Stage 7 introduces a real IPC boundary." This entry
     should move out of that section and into a real gate.
- 2026-08-29 (claude-code): The open question this task must settle rather than assume — **is
  the E2E flow a blocking gate, and for whom?** It needs `WebKitWebDriver` plus `xvfb`
  installed, which is a real barrier: it could not be run during either round of 040's review
  on this machine, and there is no CI provider wired up yet to run it automatically. Making it
  an unconditional gate would mean tasks routinely cannot pass their own gates. Options worth
  putting to the human: gate it only on tasks that touch `src-tauri/` or `src/services/`; gate
  it only once CI exists; or keep it advisory with its prerequisites documented. Do not pick
  one unilaterally — this is a process rule, and process rules bind the other two agents too.

## What was built / tested / left out

Filled in when moving to `review`.

## Review

Filled in by the reviewing agent. See `.ai/review-checklist.md`.

## Follow-ups

Anything noticed during implementation or review that's out of this task's scope.
