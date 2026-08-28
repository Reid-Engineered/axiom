# Ownership and delegation

Who's building what, and which agent it's delegated to. This is a human-readable summary —
the live, authoritative record is `.ai/tasks/*.md` (one file per task, with owner/status/
dependencies in its frontmatter) and `ROADMAP.md` (the stage plan this breakdown follows).
If this file and `.ai/tasks/` ever disagree, `.ai/tasks/` is right; this is a snapshot that
can go stale, not the source of truth. See `.ai/README.md` for why the split exists — Claude,
Codex, and Antigravity don't share a context window, so `.ai/tasks/` is how they hand off work
to each other; this file is for a human skimming the same thing.

_Last updated: 2026-08-28._

## Who does what, by default

Three tools work this repo, each with a fixed lane (`AGENTS.md` §Agent responsibilities has
the full version):

| Agent | Lane |
|---|---|
| **Claude** | Architecture, review, and locked contracts. Owns `ARCHITECTURE.md`, `AGENTS.md`, `CLAUDE.md`, `.ai/`. Writes Stage 2's component/service contracts. Reviews every task before merge — never the sole reviewer of its own work. |
| **Codex** | Implementation and tests. Hooks, services, state wiring, business logic, and the tests that go with them. Works pages from the inside out — correct and navigable first, not necessarily visually finished. |
| **Antigravity** | UI and styling. CSS Modules against `tokens.css`, primitive variants (Stage 1), visual fidelity against the mockup screenshots. Never changes a prop contract or a hook's return shape — a contract problem found while styling routes back to Claude as a finding, not a silent workaround. |

Page-level work (Stages 3–6) generally follows **Plan (Claude) → Implement (Codex) → Polish
(Antigravity) → Review (Claude)**, one task, owner changing mid-task without a status change.
Stage 1's primitives were the one exception: Antigravity implemented, Codex only added tests
in the same task — no separate Codex "implement" pass.

## Stage-by-stage allocation

| Stage | Focus | Tasks | Owner | Status |
|---|---|---|---|---|
| 0 | Foundation (scaffold, tokens, empty shell) | — (predates `.ai/tasks/`) | — | **Done**, merged to `master` |
| 1 | Design system primitives | [001–004](.ai/tasks/_archive/) | Antigravity (+ Codex tests) | **Done**, merged to `master` |
| 2 | Component contracts | [005–014](.ai/tasks/_archive/) | Claude | **Done**, merged to `master` |
| 3 | Layouts and navigation | [015–018](.ai/tasks/_archive/) | Codex (+ Antigravity polish) | **Done**, merged to `master` |
| 4 | Mock data, services, hooks | [019–021](.ai/tasks/) | Codex | Proposed, unblocked — Stage 3 merged |
| 5 | First vertical slice | [022–025](.ai/tasks/) | Codex (+ Antigravity polish) | Proposed, blocked on Stage 4 |
| 6 | Remaining pages | [026–036](.ai/tasks/) | Codex (+ Antigravity polish) | Proposed, blocked on Stage 4 |
| 7 | Real persistence (Rust/SQLite) | [037–040](.ai/tasks/) | Codex | Proposed, blocked on Stage 2 types (unblocked in principle, not started) |

Full per-task detail — scope, exact files, dependency ids — lives in `.ai/tasks/<id>-<slug>.md`
(or `.ai/tasks/_archive/<id>-<slug>.md` once a task is `done`).

### Stage 3 — closed out

All four tasks merged to `master` (fast-forward, `4116c17..123320f`) on 2026-08-28. The
chain was linear — 016 → 017 → 015 → 018, each stacked on the last — so one merge landed
all four. Typecheck, lint, build, and all 63 tests pass on `master`. Task files moved to
`.ai/tasks/_archive/`, `status: done`.

- **016** (`NavigationContext`/`WorkspaceContext`) and **017** (`Sidebar`/`WorkspaceTree`) —
  reviewed pass, no findings.
- **018** (page routing + stub `CommandPalette`) — went through two rounds of
  changes-requested before merge. Round 1: `FullVisualizationPage` overflowed the viewport
  nested in `AppShell` (its shell used `height: 100vh` instead of `100%` like its siblings).
  Round 2: the fix for that (unwrapping `FullVisualizationPage` from `AppShell` entirely)
  traded the overflow bug for a worse one — it removed the window's only draggable region
  (`AppShell` is the sole source of `data-tauri-drag-region`). Final fix (`c2f2103`): keep
  the `AppShell` wrapper with no `sidebar` prop (same pattern `FirstLaunchPage`/
  `CreateWorkspacePage` use) and change the shell's height to `100%` instead — fixes both
  bugs with one change. Approved 2026-08-28.
- **015** (`layouts/*` real implementation) — coordination failure worth remembering for
  future stages: it was implemented **twice, independently**, on two branches unaware of
  each other (`agent/codex/015-layouts-impl` @ `eba8c9a`, Codex's original pass, and a
  from-scratch redo bundled into `018`'s branch by Antigravity). Resolved: the version
  bundled in `018` (`124578f`) was declared canonical; the standalone branch was marked
  superseded on both branches (not just the winning one) and was not merged separately.

Full findings history: `.ai/tasks/_archive/018-page-nav-wiring.md` and
`.ai/tasks/_archive/015-layouts-impl.md` (`## Review`).

## Where to look for more

- **`ROADMAP.md`** — the stage plan itself: deliverables, acceptance criteria, and the risk
  most likely to blow up each stage.
- **`.ai/tasks/*.md`** (and `.ai/tasks/_archive/*.md` for `done` tasks) — one file per task:
  scope, plan, worklog, review, follow-ups.
- **`AGENTS.md`** §Agent responsibilities — the full role/lane definitions this table summarizes.
- **`.ai/lifecycle.md`** — how a task moves `proposed → in-progress → review → done`.
