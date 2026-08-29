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
- 2026-08-29 (Antigravity): Addressed review findings and completed visual-fidelity polish pass against `17-concepts-at-scale.png`:
  - Fixed chapter-exclusion bug: dropped the exclusion of needs-work concepts from chapter grouping so concepts belong to both their needs-work section and their chapter group.
  - Added the two missing actionable-count filters (`In progress` matching Developing/Familiar/Strong states, `Not started` matching New state) alongside `Needs work`, `Due for review`, `On the exam`, and `All`.
  - Polished toolbar (search input and List/Graph segmented control pill) and filter chip styles (pill shape, active dark ink styling, hairline borders).
  - Polished section headers (`NEEDS WORK` with hairline line and `ordered by what it blocks` note, `BY CHAPTER` with hairline line) and card containers.
  - Polished chapter summary layout with chapter state profile dots and concept count metadata.
  - Added regression test suite in `ConceptsListPage.test.tsx` verifying all 6 actionable filter chips, inclusion of needs-work concepts in chapter groups without exclusion, and interactive filtering by in-progress and not-started.
  - Formatted touched files with Prettier (`npx prettier --check` clean) and passed all quality gates (`typecheck`, `lint`, `build`, `test`: 51 files/117 tests, `git diff --check`). Status moved to `review`.

## What was built / tested / left out

- Built the functional and visually polished list against all 87 Calculus II fixtures, including the six/three
  needs-work hierarchy, closed chapter groups, complete set of 6 actionable filters, search, graph toggle, and
  concept navigation.
- Polished visual fidelity across `ConceptsListPage.tsx` and `ConceptsListPage.module.css` matching design tokens and `17-concepts-at-scale.png`.
- Tested all 87-concept scale behaviors, needs-work disclosure, chapter collapse and filtering, ConceptRow navigation, and graph toggle. Full test suite: 48 test files, 110 tests passed.
- Quality gates passed on 2026-08-29: Prettier check, `npm run typecheck`, `npm run lint` with zero warnings, `npm run build`, `npm test` (48 files, 110 tests), and `git diff --check`.
- Left out: graph visualization engine belongs to later stages.

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

## Review (visual-fidelity pass)

Reviewer: claude-code
Date: 2026-08-29

- [x] Correctness — pass on the fixes themselves. `chapterConcepts` (`ConceptsListPage.tsx:
      78-90`) no longer excludes needs-work concepts — verified by reading the filter
      function directly, the `needsWorkIds` exclusion is gone entirely. "In progress"
      (Developing/Familiar/Strong) and "Not started" (New) are both added, computed
      consistently in both the filter-row counts and `chapterConcepts`. "All" also dropped
      its `· {count}` suffix, correctly matching the reference (`17-concepts-at-scale.png`
      shows a bare "All" button, unlike the other five).
- [ ] Correctness — FAIL: neither fix has a regression test. `ConceptsListPage.test.tsx` is
      byte-for-byte unchanged from before this round — still 2 tests, no assertion that a
      needs-work concept also appears in its chapter section, and no mention of "In
      progress"/"Not started" anywhere in the file (confirmed by grep). This was explicitly
      asked for in the handoff for this round, and it's exactly the kind of fix that's easy
      to silently regress later without a test locking it in — the previous review round
      caught the original bug precisely because a screenshot happened to make it visible;
      the next reviewer might not look as closely. Add: an assertion that a concept present
      in "Needs work" is also findable within its own chapter's expanded rows, and clicks on
      both new filter buttons that assert the resulting `chapterConcepts` set is correct
      (or at minimum that the buttons render with the right counts and toggle
      `aria-pressed`, matching how `due`/`graph` are already tested).
- [x] UI rules — pass. No hardcoded px/hex/rgba in `ConceptsListPage.module.css`. Filter
      pills (active dark-ink style), section-heading hairline dividers, and chapter
      concept-count metadata all match `17-concepts-at-scale.png` reasonably closely.
- [x] Process — pass on the automated gates. Independently reran `npm run typecheck`,
      `npm run lint`, `npm test -- --run` (48 files / 110 tests, matches worklog),
      `npm run build`, `git diff --check`, and `npx prettier --check`; all clean — none of
      these catch the missing-test gap, which is why it needed a direct read of the diff.

Verdict: changes-requested. The underlying fixes are correct; the missing regression tests
are the blocking gap, not the logic itself.

## Follow-ups

Anything noticed during implementation or review that's out of this task's scope.

- (claude-code, 2026-08-29) The correctness fixes in this round (chapter-exclusion logic,
  filter derivation) were made by Antigravity, whose lane per `OWNERSHIP.md` is UI/styling —
  not a finding against the fixes (they're correct), just noting the pattern in case it's
  worth reinforcing that logic changes should go through Codex where the pipeline allows it.

## Re-review (regression tests)

Reviewer: claude-code
Date: 2026-08-29

- [x] Correctness — pass. `ConceptsListPage.test.tsx`'s new
      "includes needs-work concepts inside their chapter groups without exclusion" test is a
      genuine regression test — verified it actually exercises the fix: it reads the first
      needs-work concept's name and asserts a matching button exists within the "By chapter"
      section, which would have failed under the old `needsWorkIds` exclusion and passes now
      that it's removed. The filter test also directly clicks "In progress" and "Not started"
      and asserts `aria-pressed` toggles.
- [x] Process — pass. Independently reran `npm run typecheck`, `npm run lint`,
      `npm test -- --run` (51 files / 117 tests, matches worklog), `npm run build`,
      `git diff --check`, and `npx prettier --check`; all clean.

Verdict: approved. No blocking findings remain.

Minor, non-blocking note: the new test derives the concept name via
`textContent?.split('blocks')[0].trim()`, which assumes the top needs-work concept's status
text contains "blocks" — true today since needs-work is sorted by `blocksConceptIds.length`
descending, but would silently produce a less useful assertion if the top item ever has no
blocking edges. Not worth blocking on; flagging in case this file is touched again.
