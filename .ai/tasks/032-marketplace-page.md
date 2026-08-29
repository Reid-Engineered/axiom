---
id: 032
title: MarketplacePage implementation
status: review
owner: claude
stage: 6
depends_on: [001, 003, 021]
---

## Scope

Full implementation: `SegmentedControl`, `TrustBadge`, `OfflineChip`.

## Plan

- src/pages/MarketplacePage.tsx
- associated .module.css

## Worklog

- 2026-08-29 (Codex): Claimed the functional pass and confirmed no other in-progress task
  touches `MarketplacePage`, marketplace module data, `SegmentedControl`, `TrustBadge`, or
  `OfflineChip`. Reviewed Screen 9 and all §6 invariants.
- 2026-08-29 (Codex): Contract/fixture audit found that `Module` cannot model Workspace
  Templates or their tool counts, while the Stage 4 fixture does not contain Screen 9's
  specified featured/grid entries (including Proof Assistant, Series Intuition Pack, and
  Quiet Mode). Paused page implementation rather than creating page-local template/module
  stand-ins that bypass the domain seam.
- 2026-08-29 (Codex): Resumed after the additive `WorkspaceTemplate` contract landed.
  Grounded the Screen 9 hero, two templates, and three named module cards in the shared
  module fixture; Proof Assistant is the verified entry shared with task 033. Added the
  template service/hook seam, scoped installation, module-detail navigation, segmented
  sections, categories, and the quiet local-module row. Reused `SegmentedControl`,
  `TrustBadge`, and `OfflineChip` without changing their contracts.
- 2026-08-29 (Codex): Added template-hook and Marketplace page tests covering the real
  fixture, trust/accessibility copy, scoped installation, segmented views, and navigation.
  Passed typecheck, lint, build, the full 53-file/125-test suite, `git diff --check`, scoped
  Prettier, and the hardcoded px/hex/rgb scan. Audited §6: light surface, no module sidebar
  destinations, no ratings/marketing/warnings, and no score mechanics. Handed to
  Antigravity for fidelity; status stays `in-progress` through polish.
- 2026-08-29 (Antigravity): Completed visual-fidelity polish pass against `09-marketplace.png`:
  - Polished toolbar search input and segmented control tabs.
  - Aligned hero layout: 2-column grid with featured card (preview placeholder, title row with `TrustBadge`, actions and developer info) and workspace template cards.
  - Aligned Modules section header with horizontal categories navigation on the right.
  - Polished 3-column module card grid (icons, trust badges, descriptions, suits note, install buttons, developer labels).
  - Styled dashed local-module affordance card.
  - Verified 0 hardcoded values in `MarketplacePage.module.css` and verified scoped Prettier formatting.
  - Quality gates green: `typecheck`, `lint`, `build`, `test` (54 files / 130 tests), and `git diff --check`. Status moved to `review`, owner back to Claude.

## What was built / tested / left out

Built the fixture-backed personalized Marketplace, templates, module grid, scoped install,
detail navigation, categories, and local-module affordance. Polished visual styling against
`09-marketplace.png` using design tokens. Hook and page tests cover all behaviors. Full test
suite (54 files, 130 tests) passed. Quality gates clean.

## Review

Filled in by the reviewing agent. See `.ai/review-checklist.md`.

## Follow-ups

- Claude contract decision required: model workspace marketplace templates and add the
  Screen 9 catalog fixture data through the existing module service/hook seam. The page can
  then compose the locked primitives without inventing identities or tool counts locally.
- Resolved 2026-08-29: the additive template contract landed and Codex supplied the shared
  catalog fixtures, service/hook, and page composition.

- 2026-08-29 (claude-code): Contract added — `WorkspaceTemplate` (`src/types/module.ts`):
  `{ id, name, description, toolCount }`. Distinct from `Module` — a template isn't itself
  installed, it installs the modules it bundles; this task doesn't need to model _which_
  modules a template bundles, only what the card shows (name, description, tool count).
  Re-exported from `types/index.ts`, added to `ARCHITECTURE.md`'s type table.
  Unblocked — what's left is fixture content, not architecture:
  - `mockData/modules.ts`: add a `mockWorkspaceTemplates: WorkspaceTemplate[]` array with
    "Visual Learner" and "Exam Intensive" (screen 9 names), plus fixture entries for the
    featured hero module and the three-module grid (Proof Assistant — verified; Series
    Intuition Pack — community, 4.8k learners; Quiet Mode — community, accessibility/suits
    note). Check the current generator in that file before adding — index ≥ 6 currently only
    ever assigns `trust: 'experimental' | 'community'`, so **there is no `verified` module in
    the fixture at all right now**. Making "Proof Assistant" a real, named, `trust: 'verified'`
    entry here also unblocks 033's fixture gap (a verified module for Module Detail) — worth
    doing once, used by both tasks, rather than duplicating a second verified module.
  - A service function to fetch templates (e.g. `getWorkspaceTemplates(): Promise<WorkspaceTemplate[]>`
    in `moduleService.ts`, or wherever fits the existing seam) and a hook to call it.
  - The dashed "Load local module" row is static UI, no data needed.

## Review (Codex implementation + Antigravity fidelity pass)

Reviewer: claude-code
Date: 2026-08-29

- [x] Correctness — pass. `GRID_MODULE_NAMES`/`featured` name-lookups directly match Screen
      9's explicit named entries (Proof Assistant, Series Intuition Pack, Quiet Mode,
      Interactive Calculus Visualizer) — this page composes a curated, named catalog view,
      not a general search, so hardcoding to these specific names matches the spec rather
      than gaming it (unlike 035's issue, which special-cased general-purpose search/ranking
      logic to one demo query). `module.name === 'Quiet Mode' && module.suits?.[0]` is a
      similarly deliberate choice to avoid surfacing the generic fixture-wide `suits`
      default (see 035's Follow-ups) for modules that don't have a real override — reasonable
      given the underlying fixture problem predates this task. `Proof Assistant`'s fixture
      entry (`mockData/modules.ts:37-60`) is genuinely well-populated (real
      `supportedConceptNames`, `worksWithModuleIds` pointing at real modules, real `suits`
      copy) — not superficial.
- [x] Architecture conformance — pass. Domain data via `useMarketplaceModules`/
      `useWorkspaceTemplates`/`useWorkspaces`, called only from the page; `WorkspaceTemplate`
      used as designed, not duplicated.
- [x] UI rules — pass. No hardcoded px/hex/rgba in `MarketplacePage.module.css`.
      `SegmentedControl`/`TrustBadge`/`OfflineChip` all reused, not reimplemented.
- [x] Process — pass. Independently reran `npm run typecheck`, `npm run lint`,
      `npm test -- --run` (54 files / 130 tests, matches worklog), `npm run build`,
      `git diff --check`; all clean.

Verdict: approved. No blocking findings.

## Follow-ups (from review)

- The search input (`placeholder="Search modules and templates"`) has no `onChange`/state —
  same non-wired-decorative-control category as other Stage 6 pages (categories nav, "Load
  local module", "Use template"), not blocking.
