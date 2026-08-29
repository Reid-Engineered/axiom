---
id: 030
title: MaterialPage implementation
status: in-progress
owner: codex
stage: 6
depends_on: [021]
---

## Scope

Full implementation: search-typed result rows, progress bar.

## Plan

- src/types/material.ts (done — see Follow-ups)
- src/services/mockData/material.ts
- src/services/materialService.ts
- src/hooks/useMaterial.ts
- src/pages/MaterialPage.tsx
- associated .module.css

## Worklog

- 2026-08-29 (Codex): Claimed the functional pass after confirming no other in-progress
  task touches a material result component, material data, or the Material page. Reviewed
  Screen 18, the 712-page scale behavior, the component inventory, and §6 invariants.
- 2026-08-29 (Codex): Contract audit found that the repository has no material domain
  type, fixture, service, or hook. `useConcepts.search()` returns only `Concept[]`, which
  does not model a result's section/worked-example/exercise-range kind, page location,
  learner-specific reason, source material, syllabus inclusion, book chapter segments,
  or highlight metadata. Paused before implementation rather than fabricating those fields
  as page-local stand-ins; the reusable result row's prop contract depends on this data
  shape being resolved.

## What was built / tested / left out

Filled in when moving to `review`.

## Review

Filled in by the reviewing agent. See `.ai/review-checklist.md`.

## Follow-ups

- Claude contract decision required: add a material/search-result contract plus Stage 4
  fixture/service/hook data, or explicitly approve a narrower derivation from `Concept`.
  The latter cannot satisfy Screen 18's typed location, syllabus, book-position, and marks
  requirements without unsupported copy.

- 2026-08-29 (claude-code): Contract added — `src/types/material.ts` (`Material`,
  `ChapterSegment`, `MaterialResult`), re-exported from `types/index.ts`, row added to
  `ARCHITECTURE.md`'s type inventory. Confirmed against `18-material-textbook.png`:
  - `MaterialResult` covers all three result kinds (`section`/`workedExample`/
    `exerciseRange`), each with a `page`, `title`, learner-specific `reason` (quoted excerpt
    for sections, the actual reason for worked examples/exercise ranges — never a generic
    snippet, per the copy rules), optional `highlightedAt`, and `conceptId` — resolve the
    concept's name/mastery via the existing `useConcepts` map (same pattern
    `ConceptViewPage` already uses for prerequisite/leads-to), don't duplicate concept data
    onto the result.
  - `ChapterSegment`/`Material.segments` model "Where you are in the book" as exactly four
    segments with a status (`read`/`inProgress`/`next`/`outOfSyllabus`) and optional detail
    string ("33 sections") — kept deliberately generic rather than hardcoding four named
    slots, since the screenshot's exact chapter groupings ("Ch 6–7", "Ch 8", ...) are fixture
    content, not part of the type.
  - `Material` itself is the book-level state (title, edition, page/chapter counts, segments,
    highlights/notes counts, `mostMarkedSections`) that's always loaded regardless of query.
    Search results are a separate, query-dependent list (`MaterialResult[]`) — not part of
    `Material` — mirroring `searchConcepts`'s existing separation from `getConceptsByWorkspace`.
  - Deliberately left out of the type: the right rail's "ALL MATERIAL" multi-source list
    (textbook/lecture-notes/problem-sets/course-videos) and "Add material" flow. The task's
    own scope names only "search-typed result rows, progress bar" — the multi-source rail is
    a bigger surface visible in the screenshot but not requested by this task; flag it as a
    follow-up task if it's wanted, don't fold it in here.
  - Codex: build `mockData/material.ts`, `materialService.ts` (`getMaterial(workspaceId)`,
    `searchMaterial(workspaceId, query)`), and `useMaterial(workspaceId)` following the
    existing per-domain pattern (compare `concepts.ts`/`conceptService.ts`/`useConcepts.ts`),
    then the page itself. Unblocked — pick this back up.
