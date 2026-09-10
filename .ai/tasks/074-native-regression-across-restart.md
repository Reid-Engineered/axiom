---
id: 074
title: Native regression — practice loop across two restarts
status: proposed
owner: codex
stage: 8
depends_on: [070, 071, 072, 073, 075]
---

## Scope

Implement §6.6 of `docs/superpowers/specs/2026-09-09-mock-and-dead-end-containment-design.md`.

Nothing in the suite crosses render → IPC → SQLite → render. Frontend tests run against
`src/test/mockBackend.ts`; Rust tests never render a page; `069`'s guard catches unregistered
commands but not a registered command returning the wrong row. That gap is why a green
pipeline coexisted with a Home button leading to a dead end.

This is the test that settles `064` criteria 1, 2 and 6 together. Until it passes on a real
build, all three stay unmet.

Does **not** build: `067`'s regression corpus (a backend corpus, no rendering) or `068`'s
network-disabled acceptance test. Neither renders a page; this does not replace either.

## Plan

- `e2e/practice-loop-restart.test.mjs` — new, following the structure of
  `e2e/restart-persistence.test.mjs` (isolated temporary `XDG_DATA_HOME`, `tauri-driver`,
  pinned `selenium-webdriver`, W3C Actions for typing).
- `e2e/README.md` — document the new flow alongside the existing two.

**Main flow — two restarts, deliberately.** The first proves open-attempt restoration; the
second proves submission and hint-state restoration.

1. Create a workspace through First Launch.
2. Navigate to its Shell method concept.
3. Choose "Practice this".
4. Assert a generated prompt with no unsubstituted placeholders.
5. Record the exact prompt text.
6. Terminate and relaunch against the same application-data directory.
7. Assert the app opens Home, not First Launch.
8. Use Continue; assert the exact same prompt and an open attempt.
9. Submit a deliberately wrong answer.
10. Assert "That does not match yet." and the authored hint.
11. Terminate and relaunch again.
12. Continue; assert the same prompt, the wrong-submission state, and the revealed hint all
    remain.
13. Submit the correct answer.
14. Assert "Correct."
15. Choose "Next problem".
16. Assert the prompt changes and the answer and hints reset.

Steps 1–3 exercise `075`: this reaches a real generated problem in a **learner-created**
workspace, which is what criterion 1 requires and what no existing test covers.

**Retained negative assertion.** Separately: immediately after importing the sample workspace,
Home must not show a fabricated Continue card.

**Acceptance.** Both flows green on Linux CI's existing `e2e` job. The dev environment lacks
`WebKitWebDriver`, so CI is the source of truth here — do not claim a local pass.

## Worklog

- 2026-09-09 — Filed by claude from the approved design, `proposed` for codex.

## What was built / tested / left out

Not started.

## Review

## Follow-ups
