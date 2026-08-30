# Axiom — Architecture

Why the system is shaped the way it is. This document is authoritative for structure and
data flow; `reference/UI/AXIOM-HANDOFF.md` is authoritative for visual design and product
behavior. When the two could be read as disagreeing, the handoff wins on _look and behavior_,
this document wins on _how code is organized_.

The frontend implementation is complete through Stage 6. Stage 7 now has its SQLite schema,
migration runner, domain-specific IPC commands, and frontend service integration.

---

## 1. Runtime shape

Tauri v2 desktop app, three platforms, one codebase. Two processes:

- **Frontend** — React 18 + TypeScript, built with Vite, running in Tauri's webview.
- **Backend** — Rust, owns the SQLite database and any filesystem access. Exposed to the
  frontend only through `#[tauri::command]` functions (IPC), never through a raw DB
  connection the frontend can query.

**Local-first, offline-first**: SQLite is the source of truth on disk. There is no server.
Any future network calls (marketplace fetch, module updates) are additive enhancements the
app must function without — this is already a product invariant (`AXIOM-HANDOFF.md` §6.12),
not just a technical preference.

**Current phase**: the Rust backend owns an internal, mutex-protected `rusqlite` connection,
with versioned migrations under `src-tauri/src/db/`. The frontend calls only registered
domain commands through `src/services/*`; it has no raw database access. Mock fixtures now
back the test IPC adapter and remain available for the sample-data import. See §5.

---

## 2. Folder structure

```
axiom/
  src/                      # React frontend
    components/
      primitives/            Button, Chip, Toggle, ProgressBar, SegmentedControl,
                              EyebrowLabel, Placeholder
      mastery/                Mastery, ChapterStateProfile
      badges/                 TrustBadge, OfflineChip, DiagnosticDot
      feedback/                ReasonedRecommendation, SuggestionPanel
      concept/                  ConceptRow, ConceptTag
      workspace/                WorkspaceCard, WorkspaceTree
      session/                   SessionToolbar, WorkspaceToolbar, WorkingArea
      overlays/                   Sheet, Popover, Inspector, CommandPalette
      math/                        MathInline, MathDisplay
    layouts/
      AppShell.tsx              drag strip + sidebar + content slot
      SessionShell.tsx           session toolbar + resizable pane grid
      FullVisualizationShell.tsx  full-bleed, no sidebar, back header
      TwoPaneLayout.tsx           content + 250px right rail
      CenteredColumnLayout.tsx    max-520/560px column
    pages/                    one per screen, see §3
    hooks/                    useNavigation, useWorkspace, useCommandPalette,
                              useResizablePanes, useKeyboardShortcut
    services/                 data-access seam, see §5
      mockData/
    types/                    domain types, see §4
    styles/
      tokens.css              design tokens as CSS custom properties
      global.css               reset + base typography
    App.tsx
    main.tsx
  src-tauri/                # Rust backend (Stage 7+)
    src/
      commands/               domain IPC handlers + shared serialized DTOs
      db/                      connection setup, schema, migrations, queries
        migrations/            ordered SQL migrations embedded in the Rust binary
      main.rs
    Cargo.toml
    tauri.conf.json
  e2e/                     # native Tauri WebDriver flow + Linux setup notes
  reference/UI/              design source of truth (mockups, handoff doc) — do not edit
  .ai/                      multi-agent handoff system, see .ai/README.md
  AGENTS.md
  CLAUDE.md
  ARCHITECTURE.md
  ROADMAP.md
```

Component-level styles are colocated (`Mastery.module.css` next to `Mastery.tsx`), not
gathered into `styles/`. "Centralized" describes the _tokens_ — one file every component
reads from — not a single stylesheet. See `AGENTS.md` §UI Rules.

---

## 3. Component inventory

One page per screen in `AXIOM-HANDOFF.md` §4–5. Pages are the only layer allowed to call
hooks that fetch data; everything below them is props-driven.

| Page                                                     | Layout used               | Key composed components                                       |
| -------------------------------------------------------- | ------------------------- | ------------------------------------------------------------- |
| `FirstLaunchPage`                                        | `CenteredColumnLayout`    | `Placeholder` (logo), text rows                               |
| `CreateWorkspacePage`                                    | `CenteredColumnLayout`    | `Chip`, inferred-structure panel                              |
| `HomePage` (variant: default / session-intent / library) | `AppShell`                | Continue card, `WorkspaceCard`                                |
| `WorkspaceOverviewPage`                                  | `TwoPaneLayout`           | `ConceptRow`, `ReasonedRecommendation`, `SuggestionPanel`     |
| `StudySessionPage`                                       | `SessionShell`            | working area, tutor panel, visualization pane                 |
| `FullVisualizationPage`                                  | `FullVisualizationShell`  | `Inspector`, floating toolbar                                 |
| `ConceptViewPage`                                        | `TwoPaneLayout`           | `Mastery`, `MathDisplay`, `ConceptRow` (Builds on / Leads to) |
| `ConceptsListPage`                                       | `AppShell`                | `ChapterStateProfile`, `ConceptRow`, filter chips             |
| `MaterialPage`                                           | `AppShell`                | search-typed result rows, progress bar                        |
| `WorkspaceToolsPage`                                     | `AppShell`                | module rows, `Toggle`, `SuggestionPanel`                      |
| `MarketplacePage`                                        | `AppShell`                | `SegmentedControl`, `TrustBadge`, `OfflineChip`               |
| `ModuleDetailPage`                                       | `TwoPaneLayout`           | `TrustBadge`, capability-sentence rail                        |
| `GoalEditingSheet`                                       | `Sheet` overlay           | `Chip`, consequence-preview panel                             |
| `CommandPalette`                                         | `overlays/CommandPalette` | `ConceptRow`, grouped results, key legend                     |

Every row-shaped repeat (concept rows, workspace cards, module rows) is one component reused
across pages — see `AGENTS.md` §UI Rules, "no duplicated markup" is enforced here, not just
requested.

---

## 4. Shared types

`src/types/` holds the product model from `AXIOM-HANDOFF.md` §1, nothing else:

```
common.ts       MasteryState, OfflineStatus, TrustLevel, GoalState — the fixed enums
workspace.ts    Workspace, WorkspaceActivityEvent
goal.ts         Goal
concept.ts      Concept (prerequisite / related / leads-to edges live here)
module.ts       Module, WorkspaceTemplate
session.ts      Session, SessionIntent
visualization.ts VisualizationScene and its verified rendering primitives
material.ts     Material (a workspace's textbook), ChapterSegment, MaterialResult
note.ts         Note — a learner's own note content, linked to a concept
index.ts        barrel re-export — the only import path components use
```

Components, hooks, and services all import from `types/index.ts`, never from a sibling
file directly. This keeps the barrel the single place a type's public shape is declared,
and makes a type rename a one-file diff for every consumer's import statement.

Types are the first thing to lock (Stage 2 of the roadmap) because everything else —
service function signatures, component props, mock fixtures — is written against them.

---

## 5. Data flow

```
SQLite → #[tauri::command] → services/*Service.ts → hooks/use*.ts → pages/*Page.tsx → components/*
                                  invoke()             (Stage 4)       (Stage 5-6)     (Stage 1-3)

mockData/*.ts → test/mockBackend.ts → mocked Tauri IPC
 retained fixtures    test-only adapter     same service boundary
```

Rules, in order of how often they'll be checked in review:

1. **Only hooks call services.** A component or page never imports from `services/`
   directly — it calls a hook. Services own the Tauri `invoke()` boundary; hooks and every
   layer to their right remain unaware of IPC.
2. **Services preserve the locked async signatures.** For example,
   `getConceptsByWorkspace(id): Promise<Concept[]>` has the same caller-facing shape it had
   against mock data. Tests exercise those services through Tauri's mocked IPC, not by
   swapping service implementations.
3. **State ownership**: domain data (workspaces, goals, concepts, modules, sessions) is
   owned by the hook that fetched it, held in that hook's local `useState`, never lifted into
   a global store. Cross-cutting state — current route, active overlay, active workspace id —
   lives in two small React Contexts: `NavigationContext` (route + overlay) and
   `WorkspaceContext` (active workspace id, read by any hook that needs to scope a query).
   There is no Redux/Zustand/Jotai; the app has no state shape that justifies one, and
   local-first + mock-data-first makes a global store premature structure. Revisit only if
   Stage 7+ introduces cross-page optimistic-update requirements a hook-per-page can't cover.
4. **Ephemeral UI state** (hover, focus, form input value, "is this chip's tooltip open")
   lives in the component that renders it and never leaves it.
5. **Routing is not a library.** `NavigationContext` holds a discriminated-union `Route`
   (one variant per page) and a separate optional `Overlay` (Sheet / CommandPalette). No
   `react-router`: this is a single native window with no address bar, no forward/back, and a
   sidebar model (§3 of the handoff) that maps directly onto a small state machine instead of
   URL matching.

---

## 6. Testing surface (structural implications)

Every hook in `hooks/` is designed to be testable without rendering a component: it takes
plain arguments and returns plain data, so `renderHook` (Vitest + React Testing Library) is
enough. `src/test/setup.ts` installs Tauri's official IPC mock, and `test/mockBackend.ts`
serves the retained fixtures behind the same command names and payloads used in production.
See `AGENTS.md` §Testing for the policy this shape exists to support.

Stage 7's native smoke flow lives in `e2e/`. It launches the release Tauri binary through
`tauri-driver`, exercises real IPC and SQLite from first launch through workspace creation,
and isolates its application data with a temporary `XDG_DATA_HOME`. It is deliberately one
high-value flow rather than full-screen browser coverage.

---

## 7. Explicitly out of scope here

Per `AXIOM-HANDOFF.md` §7 ("Not yet designed"): empty states, notifications, the full
concept-graph view, tutor voice mode, in-app reading mode, settings, module authoring,
import onboarding, and responsive behavior below ~1100px. This document does not invent
structure for any of them — they get their own design pass, and their own architecture
addendum, when they're designed.
