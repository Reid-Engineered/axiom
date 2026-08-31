# Quality gates

**As of task 052, CI (`.github/workflows/ci.yml`) enforces the mechanical gates below
automatically on every PR and push to `master`** — `npm run typecheck`/`lint`/`build`/`test`,
the design-token grep, `cargo check`/`test`/`clippy`/`fmt`, and `npm run test:e2e:linux`, all
as required status checks. A task's handoff doc links its PR instead of restating pass/fail
by hand (see `.ai/tasks/TEMPLATE.md`). Everything not listed above stays manual — component/
hook test presence, the no-cross-import rule, `#[tauri::command]` frontend wiring, the
"Structural changes" section below, and "Explicitly not a gate" below; CI can't judge any of
these mechanically.

**Flaky checks:** if a required check fails once and then passes clean on an immediate
re-run with no code change, treat it as a pass and file a follow-up task for the flake —
don't block or re-litigate the PR over it. (Precedent: task 052's own validation hit exactly
this with `GoalEditingSheet.test.tsx`, confirmed as a flake by an immediate green re-run.)

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

CI runs `npm run test:e2e:linux` on every pull request and push to `master`, unconditionally
— not scoped to tasks touching `src-tauri/` or `src/services/` (the `e2e` job in
`.github/workflows/ci.yml` isn't path-filtered). This is a required status check; a task
can't merge without it passing, regardless of which files it touched.

## Explicitly not a gate

- Visual/pixel-diff checks against the mockup screenshots — done by eye against
  `reference/UI/screenshots/` per stage, not automated. Revisit if visual regressions start
  slipping through.
