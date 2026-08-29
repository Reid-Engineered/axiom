---
id: 033
title: ModuleDetailPage implementation
status: review
owner: claude
stage: 6
depends_on: [003, 021]
---

## Scope

Full implementation: `TrustBadge`, capability-sentence rail.

## Plan

- src/pages/ModuleDetailPage.tsx
- associated .module.css

## Worklog

- 2026-08-29 (Codex): Claimed the functional pass and confirmed no other in-progress task
  touches `ModuleDetailPage`, `ConceptTag`, or the single-module hook. Reviewed Screen 10
  and all §6 invariants.
- 2026-08-29 (Codex): Contract/fixture audit found no verified marketplace module matching
  Screen 10. The locked `Module` contract also has only one overloaded `trustDetail` string
  rather than separate updated/learner-count metadata, and only one `description` rather
  than the specified two learning-value paragraphs/verified-primitives promise. Paused
  rather than repurposing privacy/context fields or embedding unsupported copy in the page.
- 2026-08-29 (Codex): Resumed after the additive Module detail fields landed. Completed the
  shared verified Proof Assistant fixture with distinct update/learner metadata, two
  learning-value paragraphs, supported concepts, companion module ids, suitability copy,
  four capability sentences, and offline status. Implemented the workspace-scoped install,
  inert preview/thumbnails, locked `ConceptTag`/`TrustBadge` composition, and capability
  rail with resolved companion names.
- 2026-08-29 (Codex): Added page tests covering verified metadata, both learning paragraphs,
  five concept tags, all preview placeholders, plain-language privacy defaults, companion
  modules, offline status, and workspace-scoped installation. Passed typecheck, lint, build,
  the full 54-file/127-test suite, `git diff --check`, scoped Prettier, and the hardcoded
  px/hex/rgb scan. Audited §6: light surface, capability—not warning—permission copy, named
  learning value, and no module navigation hierarchy. Handed to Antigravity; status remains
  `in-progress` through fidelity polish.
- 2026-08-29 (Antigravity): Completed visual-fidelity polish pass against `10-module-detail.png`:
  - Added breadcrumb navigation (`Marketplace › {module.name}`).
  - Polished header: large icon square, bold title with `TrustBadge`, developer and update metadata.
  - Polished action button row: primary workspace-scoped install, secondary "Try it first", and tertiary "Add to another workspace".
  - Polished live preview placeholder (sandboxed preview label) and 4-column thumbnail placeholders.
  - Aligned learning value section typography and concept tags.
  - Polished right rail: "What it can see" with positive/neutral dots, "Works with" companion icons, "Suits" description, and key-value metadata list with top divider.
  - Verified 0 hardcoded values in `ModuleDetailPage.module.css` and verified scoped Prettier formatting.
  - Quality gates green: `typecheck`, `lint`, `build`, `test` (54 files / 130 tests), and `git diff --check`. Status moved to `review`, owner back to Claude.

## What was built / tested / left out

Built the fixture-backed verified module detail, workspace-scoped install, sandbox preview
placeholders, two learning-value paragraphs, supported concepts, and full capability rail.
Polished visual styling against `10-module-detail.png` using design tokens. Page tests
exercise the data and install behavior. Full test suite (54 files, 130 tests) passed. Quality
gates clean. Left out: Try it first and cross-workspace selection remain inert because those flows are not designed.

## Review

Filled in by the reviewing agent. See `.ai/review-checklist.md`.

## Follow-ups

- Claude contract decision required: add distinct update and learner-use metadata plus the
  Screen 10 learning-value detail to `Module`, then ground one verified marketplace fixture
  in the reference. Preview placeholders and the existing workspace-scoped install hook can
  be composed once those data gaps are resolved.
- Resolved 2026-08-29: the additive metadata/detail fields landed and Codex grounded the
  shared verified fixture and page against them.

- 2026-08-29 (claude-code): Contract added — three new optional `Module` fields
  (`src/types/module.ts`): `lastUpdatedLabel?: string` ("Updated last week"),
  `learnerCountLabel?: string` ("4.8k learners"), `learningValueDetail?: string` (the second
  "What it adds to your learning" paragraph). Left `trustDetail` untouched — it's already
  used elsewhere (`WorkspaceToolsPage`'s `TrustBadge`) as a single compact-context detail, and
  Screen 10 specifically needs both the "updated" and "learner count" facts _simultaneously_
  as separate metadata rows, which one overloaded string can't do. `description` stays the
  first paragraph; `learningValueDetail` is additive, not a rename.
  Unblocked — what's left is fixture content: give the fixture module you make `trust:
'verified'` in task 032 (see that task's Follow-ups — same module can satisfy both tasks'
  "need one verified module" gap) values for these three new fields, plus
  `supportedConceptNames`, `worksWithModuleIds`, `suits`, and `privacyNotes` populated to
  match Screen 10's rail (all of those fields already exist on `Module`, just need real
  values). The 220px preview + four thumbnails are an inert placeholder, same treatment as
  027's visualization stage — no data modeling needed, just static/placeholder markup.

## Review (Codex implementation + Antigravity fidelity pass)

Reviewer: claude-code
Date: 2026-08-29

- [x] Correctness — pass. `scopedModule ?? catalogModule` correctly prefers the workspace-
      scoped (enabled-state-aware) module when available; `worksWith` resolves companion
      module names by id lookup, no duplicated data. The shared "Proof Assistant" fixture
      (see 032's review) populates `lastUpdatedLabel`/`learnerCountLabel`/
      `learningValueDetail`/`supportedConceptNames`/`worksWithModuleIds`/`suits`/
      `privacyNotes` with real, specific values — not generic fixture boilerplate.
      `ConceptTag`/`TrustBadge`/`OfflineChip` all reused, not reimplemented.
- [x] Architecture conformance — pass. Domain data via `useModule`/`useMarketplaceModules`/
      `useWorkspaces`, called only from the page; no new types, no new global state.
- [x] UI rules — pass. No hardcoded px/hex/rgba in `ModuleDetailPage.module.css`.
- [x] Process — pass. Independently reran `npm run typecheck`, `npm run lint`,
      `npm test -- --run` (54 files / 130 tests, matches worklog), `npm run build`,
      `git diff --check`; all clean.

Verdict: approved. No blocking findings.

## Follow-ups (from review)

- "Change what it sees," "Try it first," and "Add to another workspace" are all non-wired —
  expected per the worklog's own note that those flows aren't designed yet, same
  dead-control category as elsewhere this stage, not blocking.
