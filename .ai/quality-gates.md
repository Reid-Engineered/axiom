# Quality gates

A task cannot move from `in-progress` to `review` until every gate that applies to it
passes. "Applies to it" matters — a docs-only task doesn't need `cargo check` run against
Rust code it didn't touch. State in the task file which gates were run.

## Always (any task touching `src/`)

- `npm run typecheck` (`tsc --noEmit`) — zero errors.
- `npm run lint` — zero errors. Warnings noted, not necessarily fixed, unless the task
  introduced them.
- `npm run build` — succeeds.
- No hardcoded design values introduced — every color/radius/shadow/spacing traces to
  `src/styles/tokens.css`. Grep for stray hex codes (`#[0-9a-fA-F]{3,6}`) and raw `rgba(`
  outside `tokens.css` as part of self-check before marking a task `review`.

## Components and hooks

- Any new component with conditional rendering or variant props has a render test.
- Any new hook has a `renderHook` test against real fixtures from `services/mockData/`.
- No component in `src/components/` imports from `src/services/` directly (see
  `ARCHITECTURE.md` §5 rule 1) — only hooks call services, only pages call
  data-fetching hooks.

## Structural changes

- If the task changed folder structure, added a top-level directory, or changed a data-flow
  rule: `ARCHITECTURE.md` is updated in the same task, not a follow-up.
- If the task changed a shared type in `src/types/`: every consumer still typechecks (covered
  by the typecheck gate above, but call it out explicitly in the task file since a type
  change is the highest-blast-radius kind of edit in this codebase).

## Backend (Stage 7+, once `src-tauri/` exists)

- `cargo check` and `cargo test` both pass.
- Any new `#[tauri::command]` has a corresponding frontend service function already wired,
  not left as a dangling backend-only addition.

## Explicitly not a gate (yet)

- End-to-end tests — not required until Stage 7 introduces a real IPC boundary. See
  `AGENTS.md` §Testing.
- Visual/pixel-diff checks against the mockup screenshots — done by eye against
  `reference/UI/screenshots/` per stage, not automated. Revisit if visual regressions start
  slipping through.
