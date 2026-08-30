# Axiom — Implementation roadmap

Staged so `main` is buildable at the end of every stage — see `.ai/merge-strategy.md`,
"Main is always buildable." Each stage lists deliverables, the acceptance criteria that
gate moving on, and the risk most likely to blow up the schedule or the architecture if
ignored. Stages 0–6 use mock data only, no backend — that constraint is the point, not a
temporary shortcut (see `AGENTS.md` §Engineering principles).

Feature work within a stage is broken into tasks in `.ai/tasks/`; this document sets the
stage boundaries and their acceptance criteria, not the individual tasks.

---

## Stage 0 — Foundation

**Deliverables**
- `npm create tauri-app` scaffold: Vite + React + TypeScript + Tauri v2, targeting the
  three desktop platforms.
- `src/styles/tokens.css` and `global.css` populated from `ARCHITECTURE.md` / the handoff's
  design-system section — real values, not placeholders.
- Empty `AppShell` layout rendering the 38px drag strip and a blank content area. No
  sidebar content yet.
- Repo tooling: ESLint, Prettier or equivalent, Vitest wired, `npm run typecheck` / `lint` /
  `build` scripts exist and pass on an empty app.

**Acceptance criteria**
- `npm run build` succeeds; `cargo check` succeeds inside `src-tauri/`.
- App launches on at least one platform showing the drag strip and empty content area.
- All four npm scripts above exist and exit 0.

**Risk**: Tauri v2's plugin APIs (especially anything SQLite-adjacent, even unused this
early) are newer and less stable than v1's — pin exact versions in `Cargo.toml` and
`package.json` now, don't float on `latest`, or Stage 7 inherits a moving target.

---

## Stage 1 — Design system primitives

**Deliverables**
- `components/primitives/*`, `components/mastery/*`, `components/badges/*` fully
  implemented (not stubbed): `Button`, `Chip`, `Toggle`, `ProgressBar`, `SegmentedControl`,
  `EyebrowLabel`, `Placeholder`, `Mastery`, `ChapterStateProfile`, `TrustBadge`,
  `OfflineChip`, `DiagnosticDot`.
- A temporary `/dev/gallery` route (removed before Stage 8, or gated behind a dev-only
  build flag) rendering every primitive in every variant, for visual comparison against
  `reference/UI/screenshots/00-foundations.png` and `15-system-refinements.png`.

**Acceptance criteria**
- Every primitive has a render test covering its documented variant props.
- Visual side-by-side against `00-foundations.png` / `15-system-refinements.png` — by eye,
  not automated (see `.ai/quality-gates.md`, "Explicitly not a gate (yet)").
- Zero hardcoded design values — grep passes per `.ai/quality-gates.md`.

**Risk**: primitives are the highest-leverage, highest-reuse code in the app — a wrong
call here (wrong prop shape for `Mastery`, missing a size variant `ConceptRow` will need
in Stage 5) is expensive to fix later because every page ends up depending on it. Worth an
explicit review pass focused only on prop API shape before moving to Stage 2, not just
visual correctness.

---

## Stage 2 — Component contracts

Locks the typed surface every later stage and every parallel agent builds against, before
any page-level implementation starts. This is the stage that makes "three agents, one
repo" actually safe.

**Deliverables**
- Every component in `ARCHITECTURE.md` §3's inventory (beyond Stage 1's primitives) gets a
  file with its full TypeScript prop interface and TSDoc — `layouts/`, `components/concept/`,
  `components/workspace/`, `components/session/`, `components/overlays/`, `components/math/`,
  every `pages/*Page.tsx`. Bodies are stubs (`return null` or a `Placeholder`), not real
  implementations.
- `src/types/*` fully populated per `ARCHITECTURE.md` §4, matching `AXIOM-HANDOFF.md` §1
  exactly (all five mastery states, all goal states, offline statuses, trust levels).
- `src/services/*` function signatures written (return types real, bodies `throw new
  Error('not implemented')` or similarly explicit) so hooks in Stage 4 have a locked
  contract to call.

**Acceptance criteria**
- `tsc --noEmit` passes across the entire stub tree.
- Every component/page/service file has a prop or function signature reviewable in
  isolation — a reviewer can approve `ConceptRow`'s contract without `ConceptRow`'s
  implementation existing.
- No implementation logic beyond a stub return — a task adding real behavior here is
  out of scope for this stage and gets split (`.ai/lifecycle.md`).

**Risk**: the temptation to "just implement it while I'm in the file" is highest here,
especially for anyone who's already thought through a component while writing its
contract. Resist it — the value of this stage is that Codex, Antigravity, and Claude can
each pick up different Stage 5/6 pages in parallel afterward without inventing
incompatible prop shapes for the same component.

---

## Stage 3 — Layouts and navigation

**Deliverables**
- `layouts/*` fully implemented (real implementation, not stubs from Stage 2).
- `NavigationContext` (route + overlay state) and `WorkspaceContext` (active workspace id)
  implemented per `ARCHITECTURE.md` §5 rule 5.
- `Sidebar` / `WorkspaceTree` implemented against mock workspace names only (no real data
  wiring yet — that's Stage 4); two-level expand rule enforced.
- Every `pages/*` stub (from Stage 2) reachable via sidebar navigation, rendering as an
  empty page with its layout.

**Acceptance criteria**
- Can navigate between every page stub via the sidebar.
- `⌘K` opens a stub command palette overlay (empty results acceptable — real results are
  Stage 6).
- Sidebar tree never exceeds two levels; only the "open" workspace expands (enforced in
  code, not just by the mock data happening to look right).

**Risk**: it's easy to let `NavigationContext` grow into a general-purpose store once it's
the one piece of global state in the app. Keep it to route + overlay, per
`ARCHITECTURE.md` §5 rule 3 — if a task wants to put domain data in it, that's a finding,
not a feature.

---

## Stage 4 — Mock data, services, hooks

**Deliverables**
- `services/mockData/*` — realistic fixture data for workspaces, goals, concepts (with
  prerequisite/related/leads-to edges), modules, sessions. Enough volume to exercise scale
  behaviors from `AXIOM-HANDOFF.md` §5 (e.g. enough concepts to test the "never opens flat"
  chapter-collapse behavior in Stage 6).
- `services/*Service.ts` real implementations against the mock data, matching Stage 2's
  locked signatures exactly.
- `hooks/use*.ts` real implementations.

**Acceptance criteria**
- Every hook has a `renderHook` test against the real fixtures (not synthetic test-only
  data), per `AGENTS.md` §Testing.
- Fixture data is enough to populate every page's non-empty state at least once — a
  reviewer should be able to point a page at a specific mock workspace and see a
  fully-populated screen, not placeholders.
- No component or page calls a service directly — only hooks do (gate already defined in
  `.ai/quality-gates.md`, enforced starting now that services have real bodies).

**Risk**: thin fixture data quietly hides bugs that only show up at scale (§5 of the
handoff exists specifically because the design was pressure-tested against a semester of
real content). Under-investing in fixture volume here makes Stage 6's scale-behavior
screens (concepts-at-scale, long-session tutor, offline/modules) look done when they
aren't.

---

## Stage 5 — First vertical slice

**Deliverables**
- `FirstLaunchPage → CreateWorkspacePage → HomePage → WorkspaceOverviewPage` fully
  implemented and data-driven, matching `screenshots/01` through `04`.

**Acceptance criteria**
- Visual match against `screenshots/01-first-launch.png` through
  `04-workspace-overview.png`, by eye.
- Full click-through: launch → create a workspace → land on home → open the workspace →
  see overview populated from Stage 4's mock data, no dead ends, no backend calls.
- All UI rules from `AGENTS.md` pass review (tokens, no duplicated markup, copy rules).

**Risk**: this is the first stage where "looks right" and "is structurally right" can
diverge — a page built by copying mockup pixel values instead of composing Stage 1/2's
components will visually pass while violating the whole point of Stages 1–2. Review
checklist's "no markup duplicated from an existing component" item is the specific defense
against this.

---

## Stage 6 — Remaining pages

**Deliverables**
- Every remaining page from `ARCHITECTURE.md` §3: `StudySessionPage`,
  `FullVisualizationPage` (static/inert — no real 3D engine yet, see Stage 8),
  `ConceptViewPage`, `ConceptsListPage`, `MaterialPage`, `WorkspaceToolsPage`,
  `MarketplacePage`, `ModuleDetailPage`, `GoalEditingSheet`, `CommandPalette` (real
  results now).
- Scale behaviors from `AXIOM-HANDOFF.md` §5 implemented, not just the base screens:
  chapter-collapse, "returning after time away" context recovery, tutor exchange
  collapsing, offline/modules-at-scale sheet.

**Acceptance criteria**
- All 12 screens reachable and data-driven, matching their respective screenshots.
- Every scale behavior in §5 of the handoff demonstrable against Stage 4's fixture data.
- Full app is click-through navigable with zero dead ends and zero backend calls.

**Risk**: `FullVisualizationPage` is the one screen this roadmap deliberately under-specs
— it needs an inert placeholder that still satisfies the "composed from verified
primitives, not generated as images" philosophy (`AXIOM-HANDOFF.md` §4, Screen 6) well
enough that Stage 8's real visualization engine slots in without a page-level rewrite.
Treat the placeholder's data shape (primitives: coordinate system, function, region, axis,
revolution, shell, annotation) as real API surface now, even though nothing renders it in
3D yet.

---

## Stage 7 — Real persistence

**Deliverables**
- `src-tauri/src/db/` — SQLite schema and migrations for Workspace, Goal, Concept, Module,
  Session, matching `src/types/*` exactly.
- `src-tauri/src/commands/*` — one `#[tauri::command]` per current mock-service function.
- `src/services/*` swapped from `mockData/` reads to `invoke()` calls — signatures
  unchanged from Stage 2/4 (this is the swap `ARCHITECTURE.md` §5 was written to make
  cheap).
- Mock fixtures repurposed as seed data for first-launch / sample-workspace flows, not
  deleted.

**Acceptance criteria**
- `cargo test` covers command handlers and queries.
- Data survives an app restart.
- No page, hook, or component outside `services/*` changed — if one had to change, that's
  a finding against Stage 2's contract-locking, worth a retro before continuing.
- `AGENTS.md` §Testing's E2E section activates: a small number of Playwright/Tauri-driver
  flows for the highest-value paths.

**Risk**: this is the stage most likely to reveal that a Stage 2 contract was wrong (a
service signature that was easy against an in-memory array but awkward against real async
IPC/SQLite latency). If that happens, fix the contract deliberately and update
`ARCHITECTURE.md`, rather than routing around it with a special case in one hook.

---

## Stage 8 — Capability runtime + Practice Core Utility

By the end of Stage 8, Axiom discovers and loads an official Practice module from a
versioned `module.toml` manifest, parses and validates that manifest into a typed runtime
representation, registers its capabilities through a generic capability runtime, and
invokes Practice without any Practice-specific logic in Core. Practice consumes a
subject-independent Knowledge Package, generates a valid deterministic calculus problem,
delegates mathematical verification through a replaceable capability, evaluates a learner
attempt, and returns structured diagnostic evidence — offline, surviving the appropriate
persistence boundaries, with no layer containing an undocumented first-party shortcut.
Stage 7 proved Axiom's application architecture; Stage 8 proves its modular
learning-platform architecture.

Stage 8 is large enough that it's designed and scheduled incrementally, one sub-project at a
time, each through its own brainstorming pass — the same discipline the previous version of
this section applied to Stage 8 as a whole. Only the first sub-project is locked below; the
rest get their own **Deliverables**/**Acceptance criteria** appended here once designed,
not pre-decided now.

### Sub-project 1 — Module & Capability runtime (locked)

Design: `docs/superpowers/specs/2026-08-30-module-capability-runtime-design.md`.
Tasks: `.ai/tasks/045-048`.

**Deliverables**
- `src-tauri/src/modules/` — the `module.toml` schema, its Rust parser/validator producing
  a typed `ModuleManifest` (raw TOML never propagates past this boundary), and the
  `ModuleRegistry`/capability resolution runtime, entirely first-party and in-process.
- `CORE.md` rewritten from a forward-looking, code-inert draft into Stage 8's active
  contract, with its own §5 (provider selection) and §7 (`Module` vs. `ModuleManifest`)
  open questions resolved.
- A conformance/regression test suite strong enough that a future third-party module (and
  the bundled Practice module, once it exists) can be checked against it directly.

**Acceptance criteria**
- `cargo test` covers manifest parsing/validation, registration, resolution, invocation,
  and serialization — see the design doc §9 for the full test-class breakdown.
- A manifest with an unsupported version, a missing required field, an invalid capability
  identifier, or a duplicate capability fails with a structured error; one broken manifest
  never blocks the rest of the bundle from registering.
- Two modules providing the same capability resolve deterministically by a workspace's
  enabled-module order (CORE.md §5), not by registration order or an arbitrary tiebreak.
- No UI and no real subject module are required to satisfy this sub-project — it is
  provable entirely through `cargo test` against fixture modules.

**Risk**: this is the piece every later Stage 8 sub-project depends on. A contract mistake
found here is cheap; the same mistake found after Practice (sub-project 4+) depends on it is
not. That's why it gets Claude's direct architectural review (task 045) before any
capability-consuming code is built against it.

### Remaining Stage 8 scope (not yet designed)

Each of the following becomes its own brainstorm → spec → plan cycle, built against
sub-project 1's locked contract, roughly in this order: Knowledge Package v1 schema,
canonical Problem schema, the `math.verify` verification capability (deterministic +
Symbolica-CAS providers), a tiny reference Calculus II knowledge package, deterministic
seeded problem generation, the Practice Core Utility itself
(`practice.generate`/`practice.evaluate`/`practice.hint`), Practice's own heavy testing bar
(property/generative tests, a permanent regression corpus), minimal Study Session UI
integration (Antigravity, presentation only — no engine/contract changes), and an explicit
network-disabled offline acceptance test end to end. None of these get deliverables or
acceptance criteria here until they're designed.

---

## Stage 9 and beyond — explicitly not planned here

Real 3D visualization engine, tutor/AI integration, the mastery engine, the event bus,
marketplace backend and module downloads, module sandboxing and signing, offline packaging
and sync, and everything listed in `AXIOM-HANDOFF.md` §7 ("Not yet designed": empty states,
notifications, concept-graph view, tutor voice mode, in-app reading mode, settings, module
authoring, import onboarding, responsive behavior below ~1100px). Each gets its own design
pass through the brainstorming process and its own architecture addendum before it gets a
stage here — this roadmap does not pre-decide their shape.
