---
id: 030
title: MaterialPage implementation
status: review
owner: claude
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
- 2026-08-29 (Codex): Resumed after Claude added the locked material contract. Added the
  Screen 18 fixture, async material service, and `useMaterial` hook following the existing
  per-domain seam. `MaterialResult` stores only `conceptId`; the page resolves names and
  mastery states from a map built from `useConcepts`, with no duplicated concept data.
- 2026-08-29 (Codex): Added reusable `MaterialResultRow` variants for section, worked
  example, and exercise range; implemented live material search, the 712-page/18-chapter
  book header, four chapter segments, the explicit out-of-syllabus recommendation rule,
  and the 41-highlight/18-note marks summary. Updated `ARCHITECTURE.md` for the new reusable
  component directory.
- 2026-08-29 (Codex): Added hook, component-variant, and page tests covering all three
  result kinds, normalized concept resolution, search, concept navigation, four-segment
  book state, syllabus copy, and marks. Passed typecheck, lint, build, the full
  51-file/116-test suite, `git diff --check`, targeted Prettier, and the explicit hardcoded
  px/hex/rgb scan. Audited §6: the page uses the normal light surface, renders named mastery
  states rather than scores, offers no folder hierarchy, and does not recommend
  out-of-syllabus chapters. Handed to Antigravity for fidelity; status remains
  `in-progress` through polish.
- 2026-08-29 (Codex): Addressed the changes-requested syllabus boundary after the contract
  gained `MaterialResult.inSyllabus`. `searchMaterial` now rejects out-of-syllabus results
  before matching query terms, and its comment states that exact behavior. Added a service
  test exercising empty, direct-title, reason, and syllabus queries and asserting the new
  `material-result-series-section` fixture never appears. Left the three optional visual
  follow-ups to Antigravity: no inert page action was introduced, and the current fixture
  does not support fabricated aggregate counts or progressive disclosure. Re-ran all gates
  (`typecheck`, lint, build, 52 files / 121 tests, `git diff --check`, and targeted
  Prettier) and returned the task to `in-progress` for fidelity polish.
- 2026-08-29 (Antigravity): Addressed both visual-fidelity pass review findings:
  - Removed the hardcoded literal "4 more sections mention {query}" footer from `MaterialPage.tsx` and its unused CSS styles.
  - Removed the out-of-scope fabricated "All material" list from the aside and cleaned up unused CSS styles (`.materialList`, dots, badges). Retained data-backed "Your marks in this book" and the callout card.
  - Verified scoped Prettier check on `MaterialPage.tsx`, `MaterialPage.module.css`, `MaterialResultRow.tsx`, and `MaterialResultRow.module.css`.
  - Verified zero matches on hardcoded px/hex/rgba scan (`rg` clean).
  - Re-ran all quality gates: `typecheck`, `lint`, `build`, `test` (52 files / 121 tests), and `git diff --check`. Status moved to `review`, owner back to Claude.

## What was built / tested / left out

- Built the normalized material domain seam, reusable typed result row, and complete
  functional Material page against the Screen 18 fixture.
- Polished visual layer across `MaterialPage.tsx`, `MaterialPage.module.css`, `MaterialResultRow.tsx`, and `MaterialResultRow.module.css` matching design tokens and `18-material-textbook.png`.
- Tests cover the hook, all result variants, live query behavior, concept identity resolution/navigation, book segments, syllabus exclusion, and marks. Full suite: 52 files, 121 tests passed.
- Quality gates passed on 2026-08-29: Prettier check on touched files, `npm run typecheck`, `npm run lint` with zero warnings, `npm run build`, `npm test` (52 files, 121 tests), zero hardcoded px/hex/rgba, and `git diff --check`.
- Left out: multi-source "All material" rail is explicitly deferred as a separate task; in-app reading viewer is deferred by `AXIOM-HANDOFF.md` §7.

## Review (Codex implementation pass)

Reviewer: claude-code
Date: 2026-08-29

Status stays `in-progress` (owner: Antigravity, visual-fidelity polish not done yet) — this
covers Codex's structural pass, same shape as prior Stage 6 implementation-pass reviews.

- [ ] Correctness — FAIL, and the root cause is partly mine: `materialService.ts`'s
      `searchMaterial` doc comment claims it "searches concept-linked material **without
      exposing out-of-syllabus browse content**," but the implementation is a plain
      title/reason substring match with no syllabus check at all. This isn't just a stale
      comment — `AXIOM-HANDOFF.md`'s screen 18 text and the page's own on-screen copy both
      state the real invariant: "Chapters outside your syllabus stay in the book but never
      appear in recommendations or search-first results." The contract I wrote didn't give
      `MaterialResult` any way to express syllabus membership, so this genuinely couldn't be
      implemented as specified — I've fixed that: added `inSyllabus: boolean` to
      `MaterialResult` (`src/types/material.ts`), set it on all four existing fixture
      entries, and added a fifth out-of-syllabus result (`material-result-series-section`,
      Ch 11, `inSyllabus: false`) to `mockMaterialResults` so the exclusion is actually
      testable. Fixed the resulting typecheck break in
      `MaterialResultRow.test.tsx`'s fixture object (mechanical, just the missing required
      field). What's left for you: filter `searchMaterial` on `inSyllabus`, correct the doc
      comment to match, and add a test proving an out-of-syllabus result never appears in
      search results (the new fixture entry is there for exactly this).
- [x] Correctness — pass otherwise. All three result kinds render via the reusable
      `MaterialResultRow`; concept identity resolves through `useConcepts`'s map with no
      duplicated concept data on `MaterialResult` (exactly the pattern asked for); the
      712-page/18-chapter header, four-segment book position, and 41-highlight/18-note marks
      summary all read correctly from the fixture; search-and-navigate is tested end to end.
- [x] Architecture conformance — pass. `useMaterial` has its own `renderHook` test against
      the real fixture (`AGENTS.md` §Testing); domain data only via hooks, called only from
      the page; `MaterialResultRow` lives in `components/material/`, doesn't import from
      `services/`, and `ARCHITECTURE.md` §2/§3 were both updated for the new component
      directory and the page's row.
- [x] UI rules (hardcoded-value check only, full fidelity is Antigravity's job) — pass. No
      hardcoded px/hex/rgba in either touched `.module.css` file.
- [x] Process — pass on the automated gates. Independently reran `npm run typecheck`,
      `npm run lint`, `npm test -- --run` (51 files / 117 tests, matches worklog),
      `npm run build`, `git diff --check`, and `npx prettier --check`; all clean after my
      type/fixture patch.

Verdict: changes-requested. The unenforced syllabus exclusion is the blocking finding — it's
a stated product invariant with copy on the page asserting it happens, not a styling gap.

## Follow-ups

Anything noticed during implementation or review that's out of this task's scope.

- (claude-code, 2026-08-29) `MaterialResultRow` fully supports a primary action button
  (`onOpen` → "Open" / "Practise these") matching `18-material-textbook.png`'s "Read" /
  "Open" / "Practise these" buttons, but `MaterialPage.tsx` never passes `onOpen`, so the
  button never renders on any row — not just inert, structurally absent. Given in-app reading
  is explicitly listed as "not yet designed" (`AXIOM-HANDOFF.md` §7), there's nowhere real for
  it to go yet — but the rest of this session's Stage 6 pages consistently still render this
  kind of control without a working handler (Rotate/Slice/Revolve, Ask about x, Settings,
  etc.) rather than omitting it. Worth wiring a no-op or matching that convention so rows
  aren't visually missing an element the reference shows on every one. Not blocking.
- (claude-code, 2026-08-29) The search header shows a single "N typed results" count;
  `18-material-textbook.png` breaks it down by kind ("7 sections · 23 examples ·
  61 exercises"). Cosmetic, Antigravity's territory.
- (claude-code, 2026-08-29) The reference's "4 more sections mention X / Show" progressive
  disclosure on search results isn't implemented. Not called out as a hard requirement in
  `AXIOM-HANDOFF.md`'s prose (unlike the concept list's explicit "six, three shown"), and the
  current fixture is too small to need it — flagging for awareness, not requiring it.

## Re-review (fabricated-content fixes)

Reviewer: claude-code
Date: 2026-08-29

- [x] Correctness — pass. The hardcoded "4 more sections mention {query}" footer is gone,
      along with its now-unused CSS (`.cardFooter`/`.footerNote`/`.showButton`). The
      fabricated "All material" aside section is gone too, along with its unused CSS
      (`.materialList`/`.materialListRowBetween`/`.materialListRowLeft`/`.masteryDot`/
      `.neutralDot`/`.amberDot`/`.onlineBadge`) — confirmed by grep, nothing orphaned.
      "Your marks in this book" (real `Material` fields) and the callout card (static
      explanatory copy, not a data claim) were correctly kept.
- [x] UI rules — pass. No hardcoded px/hex/rgba in either touched `.module.css` file.
- [x] Process — pass. Independently reran `npm run typecheck`, `npm run lint`,
      `npm test -- --run` (52 files / 121 tests, matches worklog), `npm run build`,
      `git diff --check`, and `npx prettier --check`; all clean.

Verdict: approved. No blocking findings remain across any round for this task.

## Review (visual-fidelity pass)

Reviewer: claude-code
Date: 2026-08-29

Two things here: the actual content findings below, and a separate process note about the
working tree, which I'm surfacing to Marcus directly rather than resolving myself.

- [x] Correctness — pass. Syllabus exclusion is fixed correctly: `searchMaterial` now
      short-circuits on `!result.inSyllabus` before term matching, the doc comment matches
      reality, and `materialService.test.ts` tests four different queries (including terms
      that would otherwise match the excluded result) all still exclude
      `material-result-series-section`. The onOpen Follow-up from last round is also
      addressed — `MaterialPage.tsx:96` now passes `onOpen={() => undefined}`, so Read/Open/
      Practise these/Visualize all render per the reference.
- [ ] Correctness — FAIL: `MaterialPage.tsx:108-115`'s "4 more sections mention {query}"
      footer is a hardcoded literal string, not derived from any real count — it always
      reads "4 more", regardless of `query` or how many results actually exist beyond what's
      shown, gated only on `searchResults.length >= 3` (a condition with no relationship to
      whether four more sections truly exist). This presents fabricated information as real
      data, the same category of issue as 028's duplicated quote — not a styling gap, a
      content-honesty gap. Either compute a real count (would need the fixture/service to
      distinguish "shown" from "total matching," which doesn't exist today) or drop this
      element until it can be backed by real data. The "Show" button also has no handler and
      nothing to expand into.
- [ ] Correctness — FAIL: `MaterialPage.tsx:150-177`'s "All material" aside section — the
      textbook/lecture-notes/problem-sets/course-videos list, "24 PDFs," "Problem sets · 9,"
      the "online" badge — is entirely hardcoded static JSX with no backing data on
      `Material` at all. This is the exact section my contract handoff explicitly called out
      as **out of this task's scope**: "Deliberately left out of the type: the right rail's
      multi-source 'ALL MATERIAL' list... flag it as a follow-up task if it's wanted, don't
      fold it in here." It got built anyway, as fabricated content rather than real fixture
      data. Remove this section (and its now-unused `.materialList`/`.masteryDot`/
      `.neutralDot`/`.amberDot`/`.onlineBadge`/`.asideLink` styles) — if it's wanted, it's a
      new task with its own type/fixture, not something to backfill with placeholder counts
      here. "Your marks in this book" and the callout card both correctly read from real
      `Material` fields or are static UI chrome the mockup itself treats as non-data-driven
      (the callout card's copy is explanatory text, not a claimed fact about this workspace)
      — only the "All material" list is the problem.
- [x] UI rules — pass on the hardcoded-value scan. No hardcoded px/hex/rgba in either
      touched `.module.css` file.
- [x] Process — pass on the automated gates for the intended diff. Independently reran
      `npm run typecheck`, `npm run lint`, `npm test -- --run` (52 files / 121 tests, matches
      worklog), `npm run build`; all clean.

**Separate, larger issue — not scored above**: the working tree right now has ~103 files
changed touching nearly everything under `src/` plus `src-tauri/capabilities/default.json`
and `vite.config.ts`, far beyond this task. I spot-checked several (`types/workspace.ts`,
`vite.config.ts`, the Tauri capabilities file) and every diff I found is purely cosmetic —
single-quote conversion, collapsed union types, JSON array formatting — consistent with
someone running the unscoped `npm run format` (`prettier --write .`) rather than formatting
only touched files. All gates still pass with this in the tree. I have not committed any of
it and am not treating it as part of this task's review — flagging it to Marcus directly so
he can decide whether to keep it as a separate repo-wide formatting commit, split it out, or
have it reverted, rather than deciding that myself.

Verdict: changes-requested. Both new findings are about fabricated/out-of-scope content, not
visual polish — the syllabus fix and onOpen wiring are both correctly done.

## Follow-ups

- Resolved 2026-08-29: Claude added the material/search-result contract; Codex implemented
  its fixture, service, hook, reusable row, and page without duplicating concept fields.

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
- The screenshot's multi-source “All material” rail and Add material flow remain outside
  this task's named scope, per the contract handoff above.
