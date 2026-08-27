# Axiom — Architecture

Why the system is shaped the way it is. This document is authoritative for structure and
data flow; `reference/UI/AXIOM-HANDOFF.md` is authoritative for visual design and product
behavior. When the two could be read as disagreeing, the handoff wins on *look and behavior*,
this document wins on *how code is organized*.

No implementation exists yet. This is the target shape for the code that gets written
starting at Stage 0 of `ROADMAP.md`.

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

**Current phase**: no Rust/SQLite code yet. The frontend is built first against mock data
(Stages 0–6 of the roadmap) with the service layer shaped so the Stage 7 swap to real IPC
touches only `src/services/*`, never components, hooks, or pages. See §5.

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
      commands/               one module per domain (workspace, goal, concept, module, session)
      db/                      schema, migrations, queries
      main.rs
    Cargo.toml
    tauri.conf.json
  reference/UI/              design source of truth (mockups, handoff doc) — do not edit
  .ai/                      multi-agent handoff system, see .ai/README.md
  AGENTS.md
  CLAUDE.md
  ARCHITECTURE.md
  ROADMAP.md
```

Component-level styles are colocated (`Mastery.module.css` next to `Mastery.tsx`), not
gathered into `styles/`. "Centralized" describes the *tokens* — one file every component
reads from — not a single stylesheet. See `AGENTS.md` §UI Rules.

---

## 3. Component inventory

One page per screen in `AXIOM-HANDOFF.md` §4–5. Pages are the only layer allowed to call
hooks that fetch data; everything below them is props-driven.

| Page | Layout used | Key composed components |
|---|---|---|
| `FirstLaunchPage` | `CenteredColumnLayout` | `Placeholder` (logo), text rows |
| `CreateWorkspacePage` | `CenteredColumnLayout` | `Chip`, inferred-structure panel |
| `HomePage` (variant: default / session-intent / library) | `AppShell` | Continue card, `WorkspaceCard` |
| `WorkspaceOverviewPage` | `TwoPaneLayout` | `ConceptRow`, `ReasonedRecommendation`, `SuggestionPanel` |
| `StudySessionPage` | `SessionShell` | working area, tutor panel, visualization pane |
| `FullVisualizationPage` | `FullVisualizationShell` | `Inspector`, floating toolbar |
| `ConceptViewPage` | `TwoPaneLayout` | `Mastery`, `MathDisplay`, `ConceptRow` (Builds on / Leads to) |
| `ConceptsListPage` | `AppShell` | `ChapterStateProfile`, `ConceptRow`, filter chips |
| `MaterialPage` | `AppShell` | search-typed result rows, progress bar |
| `WorkspaceToolsPage` | `AppShell` | module rows, `Toggle`, `SuggestionPanel` |
| `MarketplacePage` | `AppShell` | `SegmentedControl`, `TrustBadge`, `OfflineChip` |
| `ModuleDetailPage` | `TwoPaneLayout` | `TrustBadge`, capability-sentence rail |
| `GoalEditingSheet` | `Sheet` overlay | `Chip`, consequence-preview panel |
| `CommandPalette` | `overlays/CommandPalette` | `ConceptRow`, grouped results, key legend |

Every row-shaped repeat (concept rows, workspace cards, module rows) is one component reused
across pages — see `AGENTS.md` §UI Rules, "no duplicated markup" is enforced here, not just
requested.

---

## 4. Shared types

`src/types/` holds the product model from `AXIOM-HANDOFF.md` §1, nothing else:

```
common.ts       MasteryState, OfflineStatus, TrustLevel, GoalState — the fixed enums
workspace.ts    Workspace
goal.ts         Goal
concept.ts      Concept (prerequisite / related / leads-to edges live here)
module.ts       Module
session.ts      Session, SessionIntent
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
mockData/*.ts  →  services/*Service.ts  →  hooks/use*.ts  →  pages/*Page.tsx  →  components/*
   (Stage 4)          (Stage 4)               (Stage 4)          (Stage 5-6)      (Stage 1-3)

  [Stage 7 swap: mockData + services/* replaced by Tauri commands.
   Everything to the right of services/* is unchanged.]
```

Rules, in order of how often they'll be checked in review:

1. **Only hooks call services.** A component or page never imports from `services/`
   directly — it calls a hook. This is what makes the Stage 7 backend swap safe: hooks are
   the only thing that needs to know a service function is now `invoke()`-backed instead of
   an in-memory filter.
2. **Services are `async` now, on purpose**, even though `mockData/` is a synchronous array.
   `getConceptsByWorkspace(id): Promise<Concept[]>` today, real IPC tomorrow — no caller
   changes shape.
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
enough — no need for a mounted page just to verify `useWorkspaceConcepts` filters correctly.
See `AGENTS.md` §Testing for the policy this shape exists to support.

---

## 7. Explicitly out of scope here

Per `AXIOM-HANDOFF.md` §7 ("Not yet designed"): empty states, notifications, the full
concept-graph view, tutor voice mode, in-app reading mode, settings, module authoring,
import onboarding, and responsive behavior below ~1100px. This document does not invent
structure for any of them — they get their own design pass, and their own architecture
addendum, when they're designed.
