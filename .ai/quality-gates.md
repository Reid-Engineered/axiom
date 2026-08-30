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

## End-to-end (Stage 7+)

- Any task touching `src-tauri/` or `src/services/`: `npm run test:e2e:linux` passes. This
  is where an IPC or persistence regression would actually surface, so it's a hard gate for
  this surface specifically, not the whole repo.
- Any other task: advisory. Run it if `e2e/README.md`'s prerequisites
  (`WebKitWebDriver` + `xvfb`) are available in the environment; if not, state that plainly
  in the task file as an environment blocker rather than claiming a pass — see 040's, 042's,
  and 044's `## Review` sections for the pattern.
- No CI provider runs this automatically yet. Once one exists, widen the first bullet to
  every task touching `src/` — the scoping here is a concession to agents not reliably
  having the native WebKit driver installed, not a statement that other surfaces are exempt
  from regressions an E2E flow could catch.

## Explicitly not a gate

- Visual/pixel-diff checks against the mockup screenshots — done by eye against
  `reference/UI/screenshots/` per stage, not automated. Revisit if visual regressions start
  slipping through.
