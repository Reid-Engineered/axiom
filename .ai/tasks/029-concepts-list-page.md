---
id: 029
title: ConceptsListPage + chapter-collapse behavior
status: changes-requested
owner: antigravity
stage: 6
depends_on: [008, 021]
---

## Scope

Full implementation: `ChapterStateProfile`, `ConceptRow`, filter chips. Includes the chapter-collapse scale behavior ("never opens flat") from `AXIOM-HANDOFF.md` §5, demonstrable against Stage 4's fixture volume.

## Plan

- src/pages/ConceptsListPage.tsx
- associated .module.css

## Worklog

- 2026-08-29 (Codex): Claimed the functional implementation. Confirmed no other
  in-progress task touches `ChapterStateProfile`, `ConceptRow`, or the concept hooks;
  reviewed Screen 17's scale requirements and the §6 invariants before implementation.
- 2026-08-29 (Codex): Implemented the fixture-backed 87-concept list. Needs work derives
  six candidates from real state and blocking edges, orders them by what they block, and
  shows three initially. All other concepts start collapsed in nine textbook chapters
  composed from `ChapterStateProfile` and `ConceptRow`. Added live actionable review/exam
  counts, search, and an opt-in graph placeholder.
- 2026-08-29 (Codex): Added page tests covering the 87-item scale, three-of-six disclosure,
  closed chapter defaults, filter and graph interaction, and ConceptRow navigation. Passed
  typecheck, lint, build, the full 48-file/110-test suite, `git diff --check`, targeted
  Prettier, and the explicit hardcoded-value scan. Audited §6: the page uses the normal
  light surface, named mastery states, and no dashboard scores, streaks, or module routes.
  Handed to Antigravity for fidelity; status remains `in-progress` through polish.

## What was built / tested / left out

Built the functional list against all 87 Calculus II fixtures, including the six/three
needs-work hierarchy, closed chapter groups, actionable filters, search, graph toggle, and
concept navigation. Page tests exercise those behaviors. Screenshot fidelity remains for
Antigravity; the graph is deliberately inert pending the later visualization stage.

## Review (Codex implementation pass)

Reviewer: claude-code
Date: 2026-08-29

Status stays `in-progress` (owner: Antigravity, visual-fidelity polish not done yet) — this
covers Codex's structural pass, same shape as prior Stage 6 implementation-pass reviews.

- [ ] Correctness — FAIL: `ConceptsListPage.tsx:71-72` excludes needs-work concepts from
      the by-chapter grouping whenever `filter` is `'needs-work'` or `'all'`
      (`if ((filter === 'needs-work' || filter === 'all') && needsWorkIds.has(concept.id))
      return false`). `reference/UI/screenshots/17-concepts-at-scale.png` shows the opposite:
      with "Needs work" selected, "Shell Method" still appears in "7 · Applications of
      Integration" below (visible in the 3 shown rows), and that chapter's own header count
      reads "9 concepts" — consistent with including, not excluding, its needs-work member.
      A concept belongs to both its "Needs work" spot and its chapter group simultaneously;
      this implementation makes it vanish from its chapter whenever it's also a needs-work
      pick, which both undercounts chapters and contradicts the reference. Fix: drop the
      `needsWorkIds` exclusion from `chapterConcepts` entirely.
- [ ] Correctness — FAIL (lower severity, same root cause): the filter row only has 4 of
      the reference's 6 actionable-count filters — missing "In progress" and "Not started".
      `MasteryState` (`src/types/common.ts`) has 5 named states, so both are derivable (likely
      "Not started" → `'New'`, "In progress" → some subset of `'Developing'`/`'Familiar'`/
      `'Strong'` — exact mapping is a judgment call, not specified anywhere, so whoever
      implements it should pick one and state the reasoning in the worklog). The task's own
      scope says "filter chips" without limiting the count, and `AXIOM-HANDOFF.md`'s "Filters
      are actionable counts... not taxonomy" describes the pattern generally, not a specific
      subset.
- [x] Correctness — pass otherwise. All 87 fixture concepts render; needs-work ordering by
      `blocksConceptIds.length` then name matches "ordered by what it blocks"; chapters
      default closed (native `<details>`, no `open` attribute) and stay closed until
      expanded, satisfying "never opens flat"; search, the three-of-six needs-work
      disclosure, and the inert opt-in graph toggle are all correct and tested.
- [x] Architecture conformance — pass. Domain data via `useConcepts`, called only from the
      page; no new types, no new global state; `AppShell` (no selection rail) matches
      `ARCHITECTURE.md` §3's locked layout choice for this page — the reference screenshot's
      right-hand "Selected/Chain/Attached" rail is real but out of this task's contracted
      scope, not a gap.
- [x] UI rules (hardcoded-value check only, full fidelity is Antigravity's job) — pass. No
      hardcoded px/hex/rgba in the touched `.module.css` file.
- [x] Process — pass. Independently reran `npm run typecheck`, `npm run lint`,
      `npm test -- --run` (48 files / 110 tests, matches worklog), `npm run build`,
      `git diff --check`, and `npx prettier --check`; all clean.

Verdict: changes-requested. The chapter-exclusion bug is blocking — it's a real, demonstrable
behavior difference from the reference, not a styling gap. The missing two filters is a
second, real gap worth fixing in the same pass.

## Follow-ups

Anything noticed during implementation or review that's out of this task's scope.

## Follow-ups

Anything noticed during implementation or review that's out of this task's scope.
