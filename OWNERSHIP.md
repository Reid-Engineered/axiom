# Ownership and delegation

Who's building what, and which agent it's delegated to. This is a human-readable summary —
the live, authoritative record is `.ai/tasks/*.md` (one file per task, with owner/status/
dependencies in its frontmatter) and `ROADMAP.md` (the stage plan this breakdown follows).
If this file and `.ai/tasks/` ever disagree, `.ai/tasks/` is right; this is a snapshot that
can go stale, not the source of truth. See `.ai/README.md` for why the split exists — Claude,
Codex, and Antigravity don't share a context window, so `.ai/tasks/` is how they hand off work
to each other; this file is for a human skimming the same thing.

_Last updated: 2026-08-28 (Stage 5 merge)._

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
| 4 | Mock data, services, hooks | [019–021](.ai/tasks/_archive/) | Codex | **Done**, merged to `master` |
| 5 | First vertical slice | [022–025](.ai/tasks/_archive/) | Codex (+ Antigravity polish) | **Done**, merged to `master` |
| 6 | Remaining pages | [026–036](.ai/tasks/) | Codex (+ Antigravity polish) | Proposed, unblocked — Stage 5 merged |
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

### Stage 4 — closed out

All three tasks merged to `master` (fast-forward, `b83202b..7d17c32`) on 2026-08-28. Linear
stack — 019 → 020 → 021 — same pattern as Stage 3. Typecheck, lint, build, and all 78 tests
pass on `master`. Task files archived, `status: done`.

- **019** (mock data fixtures) and **021** (hooks) — reviewed pass on the first round, no
  blocking findings.
- **020** (service implementations) — one round of changes-requested: `getModulesByWorkspace`
  ignored `workspaceId` beyond an existence check and returned global module state, so every
  workspace saw the same `enabled`/`visibility` values regardless of its own
  `enabledModuleIds` — confirmed with a throwaway test against the Linear Algebra fixture
  (expected 4 enabled, got 13). Fixed by deriving `enabled` from the requested workspace's
  `enabledModuleIds` instead of the shared `Module` object, with a new regression test pair
  (`moduleService.test.ts`) proving both the per-workspace counts and mutation isolation.
  Approved 2026-08-28.
- **Open follow-up, not yet a task**: `Module.visibility` (`src/types/module.ts`) has no
  per-workspace home in the locked type — unlike `enabled`, which now correctly derives from
  `Workspace.enabledModuleIds`, `visibility` ('workspace' / 'contextual' / 'off') is still a
  flat field on the shared `Module`, even though the product spec's own language ("Off **in
  this workspace**") implies it should be per-workspace. Consequence already visible:
  `setModuleEnabled` and `setModuleVisibility` are now asymmetric — enabling a module that's
  `visibility: 'off'` no longer promotes it, so a module can end up `enabled: true` /
  `visibility: 'off'` in one workspace. Mine (Claude) to resolve against `ARCHITECTURE.md`/
  `src/types/module.ts` before Stage 5/6 builds a page that renders module visibility
  groupings (Workspace Tools, screen 8; Marketplace, screen 9).

Full findings history: `.ai/tasks/_archive/020-services-impl.md` (`## Review`).

### Stage 5 — closed out

All four tasks merged to `master` (fast-forward, `fc2c126..52be46b`) on 2026-08-28. Linear
stack — 022 → 023 → 024 → 025 — same pattern as Stages 3–4. The full click-through works:
launch → create a workspace → land on Home → open a workspace → see the overview populated
from Stage 4's fixtures, no dead ends, no backend calls. Typecheck, lint, build, and all 92
tests pass on `master`. Task files archived, `status: done`.

- **023**, **024**, **025** — reviewed pass, no blocking findings. **025**
  (`WorkspaceOverviewPage`) was the strongest of the four — matched
  `04-workspace-overview.png` almost line-for-line, including status-label heuristics
  clearly reverse-engineered from the mockup rather than guessed.
- **022** (`FirstLaunchPage`) — one round of changes-requested: the subject field rendered
  "Calculus II" as a real, dark, controlled value (`useState('Calculus II')`,
  `color: var(--text-primary)`) instead of the ghost placeholder
  `AXIOM-HANDOFF.md` names explicitly for screen 1 ("pre-filled with a **ghost** 'Calculus
  II'") and the screenshot clearly shows in muted gray — a spec-named detail the self-
  certified "visual check completed" worklog entry missed. Fixed with an empty controlled
  value, a real `placeholder` attribute styled via the existing `--text-metadata` token, and
  a submit-time fallback to "Calculus II" when left untouched, plus regression assertions
  for both the empty value and the visible placeholder. Approved 2026-08-28.
- **Process note**: none of the four tasks show an Antigravity polish pass in their
  worklogs — Codex self-certified visual fidelity for all four rather than the usual
  Plan → Implement (Codex) → Polish (Antigravity) → Review (Claude) loop. 022's finding is
  a data point that self-certification missed a spec-named detail; worth reinstating the
  Antigravity pass for Stage 6's page tasks rather than treating it as optional by default.
- **Open follow-ups, not yet tasks**:
  - `App.tsx`'s sidebar workspace list (`WORKSPACES`) is still a hardcoded 3-entry array,
    not sourced from `useWorkspaces()`. A workspace created via 023 shows up correctly on
    Home (which uses the real hook) but never appears in the persistent sidebar tree for the
    rest of the session. Not a dead end today, but should be closed before Stage 6 leans on
    the sidebar reflecting live workspace state.
  - Four of the five newly-implemented locked component bodies (`WorkspaceCard`,
    `MathDisplay`, `ReasonedRecommendation`, `SuggestionPanel`) originally placed their
    `import`s after the function body instead of at the top; fixed during 022's re-review
    pass. Purely cosmetic, already resolved — noted here only in case the pattern recurs in
    Stage 6's new components.

Full findings history: `.ai/tasks/_archive/022-first-launch-page.md`,
`.ai/tasks/_archive/023-create-workspace-page.md`, `.ai/tasks/_archive/024-home-page.md`,
`.ai/tasks/_archive/025-workspace-overview-page.md` (`## Review`).

## Where to look for more

- **`ROADMAP.md`** — the stage plan itself: deliverables, acceptance criteria, and the risk
  most likely to blow up each stage.
- **`.ai/tasks/*.md`** (and `.ai/tasks/_archive/*.md` for `done` tasks) — one file per task:
  scope, plan, worklog, review, follow-ups.
- **`AGENTS.md`** §Agent responsibilities — the full role/lane definitions this table summarizes.
- **`.ai/lifecycle.md`** — how a task moves `proposed → in-progress → review → done`.
