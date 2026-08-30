# AGENTS.md — Axiom

Conventions for any agent (human or AI) working in this repo. This document covers *what
the rules are*; `.ai/` covers *how agents hand work to each other*; `CLAUDE.md` covers
*Claude-specific behavior*. If you're an AI agent picking up a task, read this file and
`ARCHITECTURE.md` before writing anything.

---

## Vision

Axiom is a desktop learning environment (Tauri, offline-first) built around one idea: a
learner has a **goal**, works inside a **workspace**, and everything the app shows —
recommendations, tutoring, practice — is in service of that goal, expressed in the learner's
own terms, never as scores, streaks, or dashboards. The full product model and every
screen's behavior is specified in `reference/UI/AXIOM-HANDOFF.md` — that document is not
background reading, it is the spec. Read it before implementing any page.

The twelve invariants in that document's §6 are non-negotiable design constraints, not
suggestions. The two most likely to be violated by an agent moving fast:

- **No dashboards, no percentages, no streaks/XP/badges.** Progress is a named mastery
  state and a sentence, never a number presented as a score.
- **Modules are never navigation destinations.** If you're adding a sidebar row for a
  module, stop — that's the one thing the sidebar is specified to never do.

## Engineering principles

- **YAGNI over the mock-data phase especially.** Nothing in Stages 0–6 talks to a backend.
  Do not add a state management library, a caching layer, or an API client "for later" —
  `ARCHITECTURE.md` §5 explains why the current shape already isolates the Stage 7 swap.
- **Match existing patterns before introducing new ones.** If a similar component,
  hook, or service already exists, extend or reuse it. A second, slightly different way of
  doing the same thing is a bug, not a style choice.
- **Small, single-purpose files.** A component file that's grown past doing one visible
  thing is a signal to split it, not to add a `// section` comment and keep going.
- **No comments explaining what code does.** Comment only a non-obvious *why* — a
  constraint from the handoff doc, a workaround, an invariant a reader could otherwise miss.

## UI rules

- **Design tokens only.** Every color, radius, shadow, spacing value, and font stack comes
  from `src/styles/tokens.css`. A hardcoded hex code, `rgba()`, or `px` value copied from a
  mockup instead of referencing a token is a review-blocking finding, not a style nit.
- **No duplicated markup.** If the same visual row/card/badge shape appears on two pages
  (concept rows, workspace cards, trust badges — see `ARCHITECTURE.md` §3), it is one
  component in `src/components/`, imported twice, not implemented twice.
- **Components are data-driven.** A component in `src/components/` takes props and renders
  them; it does not import a service, call a hook that fetches, or know which page it's on.
  Only `pages/*` call data-fetching hooks. This is what `ARCHITECTURE.md` §5 rule 1 exists to
  protect — check it in review.
- **Preserve the mockups' visual language exactly**: two accents only, no gradients beyond
  the one sanctioned Continue-card treatment, no red, STIX Two Text for math, system font
  stack for UI. `AXIOM-HANDOFF.md` §2 is the exact spec; when a mockup screenshot and this
  section disagree, the screenshot is probably older — check `15-system-refinements.png`,
  which is explicitly authoritative over the four screens it supersedes.
- **Copy rules apply to placeholder/mock text too.** No exclamation marks, no emoji, no
  "Amazing!" — mock data should read the way real product copy would, per
  `AXIOM-HANDOFF.md` §2 "Copy rules". Sloppy mock copy sets a bad pattern for whoever
  writes the real copy later.

## State ownership

See `ARCHITECTURE.md` §5, rule 3, for the full rule. Summary for review purposes: domain
data lives in the hook that fetched it; cross-cutting state (route, overlay, active
workspace) lives in exactly two contexts; nothing else is global. A new `useState` at the
top of a page component that isn't ephemeral UI state (form value, hover, open/closed) is
worth a second look — it's either misplaced domain data or missing from a hook.

## Testing

Current phase (real backend, Stage 7+):

- **Hooks**: `renderHook` (Vitest + React Testing Library) against real mock fixtures, not
  hand-rolled test doubles — the fixtures in `services/mockData/` are the test data.
- **Components**: render tests for anything with conditional rendering or prop-driven
  variants (`Mastery`'s five states, `TrustBadge`'s three levels). Pure layout components
  (`CenteredColumnLayout`) don't need a dedicated test.
- **Backend**: `cargo test` covers Rust command handlers and DB queries.
- **E2E**: `e2e/*.test.mjs` — a small number of native flows for the highest-value paths
  (first launch → create workspace → home; restart persistence), not full-screen coverage.
  Driven against the release Tauri binary through `tauri-driver`'s W3C WebDriver transport,
  with `selenium-webdriver` as the client — Playwright cannot attach to Tauri's native
  WebDriver transport (see `e2e/README.md` for why, and for setup). Gated per
  `.ai/quality-gates.md`: required for any task touching `src-tauri/` or `src/services/`,
  advisory elsewhere — it needs `WebKitWebDriver` plus `xvfb` on `PATH`, which isn't
  guaranteed in every agent's environment, and there's no CI provider wired up yet to run it
  automatically for every task.
- Every task in `.ai/tasks/` states what got tested and how in its handoff doc — "tests
  pass" without naming which ones is not sufficient for the quality gate in
  `.ai/quality-gates.md`.

## Docs

- `ARCHITECTURE.md` is kept in sync with structure — if a task changes the folder layout,
  adds a top-level directory, or changes a data-flow rule, the same task updates
  `ARCHITECTURE.md`. A structural change without a doc update fails review.
- Every exported component and hook gets a one-line TSDoc if its purpose isn't obvious from
  its name and prop types alone. Not a paragraph — a sentence, only when needed.
- No separate changelog file during the mock-data phase; git history and `.ai/tasks/`
  handoff docs are the record. Revisit once Stage 7 ships something users actually run.

## Agent responsibilities and handoff workflow

Three tools work this repo, each with a fixed lane. A task's owner
(`.ai/lifecycle.md` frontmatter) is drawn from whichever lane the task's actual work falls
in — a task doesn't get reassigned to "whoever's free."

### Roles

- **Claude — architect and reviewer.** Owns `ARCHITECTURE.md`, this file, `CLAUDE.md`, and
  `.ai/` (changes to any of them still need human sign-off per `.ai/merge-strategy.md`).
  Writes and locks component/service contracts (`ROADMAP.md` Stage 2). Reviews every task
  before it merges, using `.ai/review-checklist.md` — Claude is never the sole reviewer of
  its own task. Makes the call on refactors and scope splits per `.ai/lifecycle.md`.
- **Codex — implementation and tests.** Builds the functional layer against a locked
  contract: hooks, services, state wiring, business logic, and the tests that go with them
  (see §Testing above). Works from `pages/*` inward — a Codex pass on a page is correct and
  navigable, not necessarily visually finished.
- **Antigravity — UI and styling.** Builds the visual layer: CSS Modules against
  `tokens.css`, primitive component variants (`ROADMAP.md` Stage 1), and page-level visual
  fidelity against `reference/UI/screenshots/`. Antigravity does not change a component's
  prop contract or a hook's return shape — if the visual work reveals the contract is wrong,
  that's a finding routed back to Claude, not a silent workaround.

Exceptions: a docs-only task is Claude alone. A pure primitive (Stage 1) is Antigravity
implement + Codex test, with no separate Codex "implement" pass needed. Not every task uses
all three roles — the loop below describes the default shape for page-level work, where it
does.

### Artifacts per role

| Role | Produces | Where |
|---|---|---|
| Claude | task scope + contract, review verdict | `.ai/tasks/<id>.md` Plan and Review sections; `types/`, component/service prop interfaces |
| Codex | hooks/services/logic + tests | component/hook/service implementation files; task Worklog |
| Antigravity | styled components and pages | `*.module.css`, markup adjustments within the locked contract, visual-fidelity note in the task file |

### Plan → implement → polish → review

1. **Plan** (Claude) — task scoped in `.ai/tasks/`, contract confirmed or updated against
   `ARCHITECTURE.md`, acceptance criteria pulled from the task's `ROADMAP.md` stage, owner
   set for the implement step.
2. **Implement** (Codex) — logic layer built against the locked contract: data wiring,
   state, behavior, tests. Markup exists and is correct; styling may still be
   primitive-only.
3. **Polish** (Antigravity) — visual layer applied on top of Codex's implementation:
   tokens, spacing, fidelity to the mockup screenshot for that screen. Contract untouched.
4. **Review** (Claude) — `.ai/review-checklist.md` run in full, verdict recorded, merged
   per `.ai/merge-strategy.md` only on a pass.

A task moves through `.ai/lifecycle.md`'s states in step with this loop: `proposed` →
`in-progress` (steps 2–3 — owner changes from Codex to Antigravity mid-task without a
status change, both are `in-progress`) → `review` (step 4) → `done`.

## Git workflow

- **Trunk-based.** `main` is always buildable (see per-stage acceptance criteria in
  `ROADMAP.md`). Work happens on short-lived branches, one per task:
  `agent/<tool>/<task-id>-<slug>` (e.g. `agent/claude-code/012-concept-row`).
- **Conventional commits**: `type(scope): summary` — `feat(components): add ConceptRow`,
  `fix(hooks): correct useWorkspaceConcepts filter`, `docs(architecture): note stage 7 swap`.
- **Squash-merge to `main`.** Multiple agents produce many small commits per task; the
  branch history doesn't need to survive, the task's handoff doc in `.ai/tasks/` is the
  durable record of what happened and why.
- **No force-push to `main`, no `--no-verify`.** If a hook or check fails, fix the cause.
- Full merge mechanics (who reviews, what gates block a merge) live in
  `.ai/merge-strategy.md` and `.ai/quality-gates.md` — this section covers naming and
  commit hygiene only.
