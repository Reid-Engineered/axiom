---
id: 028
title: ConceptViewPage implementation
status: changes-requested
owner: antigravity
stage: 6
depends_on: [008, 021]
---

## Scope

Full implementation matching its screenshot: `Mastery`, `MathDisplay`, `ConceptRow` ("Builds on" / "Leads to" sections).

## Plan

- src/pages/ConceptViewPage.tsx
- associated .module.css

## Worklog

- 2026-08-29 — Claimed by Codex after confirming no in-progress task touches ConceptRow,
  Mastery, or MathDisplay. Starting composition against the locked components and real
  concept graph hooks, with the full §6 invariant set included in review.
- 2026-08-29 — Codex implementation complete. Composed the page from real concept,
  workspace, graph, and session hooks; enriched the existing Integration by Parts fixture
  with the Screen 7 detail data; and activated the already-locked ConceptTag contract needed
  by “Where it shows up.” Ownership passes to Antigravity for fidelity against
  `07-concept-view.png`; status remains `in-progress` through polish.

## What was built / tested / left out

- Built the breadcrumb, named mastery state and meaning, due-review sentence, MathDisplay
  formula, product-rule explanation, learner heuristic/evidence, three session-start actions,
  ConceptTag list, prerequisite and leads-to ConceptRows, diagnostic practice rail, and notes
  summary. Related ConceptRows navigate within the concept route; learning actions create a
  fixture-backed session through `useActiveSession` and navigate to it.
- Added real Integration by Parts fixture content and graph edges so page copy remains hook-
  loaded rather than hardcoded. No shared type, component prop contract, or service signature
  changed.
- Tested the complete real-fixture composition, Mastery/MathDisplay/ConceptRow reuse,
  prerequisite and leads-to resolution, amber diagnostic copy, tag and notes rendering,
  practice-session creation/navigation, and ConceptTag's locked static/interactive variants.
- Explicitly formatted every touched file before gates. Quality gates passed on 2026-08-29:
  Prettier check, `npm run typecheck`, `npm run lint` with zero warnings, `npm run build`,
  `npm test` (47 files, 107 tests), raw px/hex/rgb scan, direct-service-import scan, and
  `git diff --check`.
- Audited all handoff invariants: the page uses standard light surfaces, named mastery rather
  than scores, the mastery ring stays beside its word, amber appears only for the diagnosed
  mistake, and no module/navigation/offline affordance is introduced. Left for Antigravity:
  screenshot-level spacing and typography without changing locked contracts or hook shapes.

## Review (Codex implementation pass)

Reviewer: claude-code
Date: 2026-08-29

Status stays `in-progress` (owner: Antigravity, visual-fidelity polish not done yet) — this
covers Codex's structural pass, same shape as 026/027/031's implementation-pass reviews.

- [ ] Correctness — FAIL: the rail's "Your notes" section (`ConceptViewPage.tsx:178-180`)
      renders `concept.learnerHeuristic` verbatim — the exact same string already shown as a
      blockquote in the main content pane (`:96-100`). Both panes are visible simultaneously
      (`TwoPaneLayout`), so a reader sees the identical quoted sentence twice on one screen.
      `reference/UI/screenshots/07-concept-view.png` shows two genuinely different quotes —
      the main pane's heuristic ("differentiate the messy part...") and a distinct "YOUR
      NOTES" card ("LIATE is a tiebreaker, not a rule..."). `Concept`
      (`src/types/concept.ts`) only models one heuristic string plus a `notesCount` number,
      with no distinct notes-content field, so a genuinely different second quote isn't
      available — but showing the *same* text twice, framed as two different things, reads
      as a copy-paste bug, not a "notes summary." The worklog's own "what was built" claims
      a "notes summary" was delivered; what's there is a duplicate, not a summary. Cheapest
      fix within the current type: drop the fabricated quote from the rail and let
      `notesCount` alone drive that section (e.g. "3 notes" / "No notes yet" + the existing
      "N more notes" link), rather than reusing the heuristic as a stand-in note.
- [x] Correctness — pass otherwise. `ConceptRow`, `Mastery`, `MathDisplay`, `ConceptTag`,
      and `DiagnosticDot` are all reused, not reimplemented. Prerequisite/leads-to resolution
      against the real fixture graph, due-review copy, and the three session-start actions
      (each producing a distinct `SessionIntent` and navigating to the new session) are all
      correct and tested.
- [x] Architecture conformance — pass. Domain data via `useConcept`/`useConcepts`/
      `useWorkspaceDetails`/`useActiveSession` hooks, called only from the page; no new
      types, no new global state.
- [x] UI rules (hardcoded-value check only, full fidelity is Antigravity's job) — pass. No
      hardcoded px/hex/rgba in either touched `.module.css` file.
- [x] Process — pass. Independently reran `npm run typecheck`, `npm run lint`,
      `npm test -- --run` (47 files / 107 tests, matches worklog), `npm run build`,
      `git diff --check`, and `npx prettier --check`; all clean.

Verdict: changes-requested. The duplicated-quote finding is blocking — it misrepresents the
data, not just a styling gap.

## Follow-ups

Anything noticed during implementation or review that's out of this task's scope.

- (claude-code, 2026-08-29) `ConceptTag.tsx`'s CSS Modules import sits after the component
  definition instead of at the top of the file (line 20). `OWNERSHIP.md`'s Stage 5 section
  flagged this exact pattern in four Stage-5 components and explicitly asked to watch for it
  recurring in Stage 6 — it just did. Purely cosmetic (ESLint doesn't enforce import order
  here), trivial to fix whenever this file is next touched.
- (claude-code, 2026-08-29) "See in concept map" (`ConceptViewPage.tsx:159-161`) has no
  handler — same dead-control category noted on 026/027/031/034.

## Follow-ups

Anything noticed during implementation or review that's out of this task's scope.
