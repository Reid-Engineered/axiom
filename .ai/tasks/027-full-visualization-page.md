---
id: 027
title: FullVisualizationPage inert placeholder
status: changes-requested
owner: antigravity
stage: 6
depends_on: [021]
---

## Scope

Static/inert placeholder — no real 3D engine (Stage 8). Must still satisfy "composed from verified primitives, not generated as images" (`AXIOM-HANDOFF.md` §4, Screen 6). Treat the placeholder's data shape (coordinate system, function, region, axis, revolution, shell, annotation) as real API surface now so Stage 8's engine slots in without a page-level rewrite.

## Plan

- src/pages/FullVisualizationPage.tsx
- associated .module.css
- placeholder primitive data-shape types (coordinate system, function, region, axis, revolution, shell, annotation)

## Worklog

- 2026-08-29 — Claimed by Codex after confirming no other in-progress task touches the
  page, FullVisualizationShell, Inspector, or visualization types. Starting the typed scene
  model and inert interaction layer against the existing session/navigation contracts.
- 2026-08-29 — Codex implementation complete. Added the public verified-primitive scene
  model, a typed shell-method scene, the full page composition, and the first implementation
  of the locked Inspector contract. Ownership passes to Antigravity for visual fidelity
  against `06-full-visualization.png`; status remains `in-progress` through polish.

## What was built / tested / left out

- Built `VisualizationScene` with explicit coordinate-system, function, region, axis,
  revolution, shell, and annotation primitives. The placeholder reads the typed scene graph;
  no bitmap or loose-prop stand-in was introduced. Updated `ARCHITECTURE.md`'s shared-type
  inventory for the new public API surface.
- Built the full-bleed, no-sidebar page with session return, scene actions, Bounds controls,
  shell toggle, floating visualization and zoom controls, and a selected-shell inspector that
  is selection-dependent, dismissible, and restorable.
- Tested all seven required primitive kinds, fixture-backed session/concept loading, bounds,
  selection dismissal/restoration, shell visibility, selected-shell math and interpretation,
  session-return navigation, and Inspector's open/closed variants.
- Quality gates passed on 2026-08-29: Prettier check, `npm run typecheck`, `npm run lint`
  with zero warnings, `npm run build`, `npm test` (45 files, 104 tests), raw px/hex/rgb scan,
  direct-service-import scan, and `git diff --check`.
- Left out by design: real rendering, camera controls, geometry mutation, saving/sharing,
  tutor/note mutations, and advanced bounds behavior belong to Stage 8 or later. Antigravity
  owns the remaining screenshot-fidelity pass without changing the typed scene contract.

## Review (Codex implementation pass)

Reviewer: claude-code
Date: 2026-08-29

Status stays `in-progress` (owner: Antigravity, visual-fidelity polish not done yet) — this
covers Codex's structural pass ahead of the usual full pass once it reaches `review`, same
shape as 026/031's implementation-pass reviews.

- [ ] Correctness — FAIL: `FullVisualizationPage.module.css:1-7,17-22` renders the stage and
      loading state with `background: var(--color-ink); color: var(--color-content);` — a
      dark, inverted palette. `reference/UI/screenshots/06-full-visualization.png` shows this
      screen with the app's normal light background throughout, and
      `AXIOM-HANDOFF.md:215` (Invariant 10) is explicit: "the dark learning canvas is the
      single sanctioned exception, and only for the mathematical object itself" — the
      "learning canvas" is Screen 20 (`20-learning-canvas.png`), a different screen, not this
      one. `AXIOM-HANDOFF.md:193` also separately calls out that screen 20 is "the one place
      Axiom inverts its own palette." The sibling `VisualizationPane` in `StudySessionPage.tsx`
      (task 026, already approved) correctly uses a light `--color-chrome` background for the
      same kind of stage — this task inverted it instead. This isn't a nitpick: it's the
      page's entire visual identity, contradicts a numbered invariant, and disagrees with an
      already-approved sibling implementation of the same concept.
- [x] Correctness — pass otherwise. All seven verified-primitive kinds
      (`src/types/visualization.ts`) are modeled as real typed fields, not loose props, and
      `shellMethodScene` populates all of them; the placeholder reads the typed graph
      (`FullVisualizationPage.tsx:93-112`) rather than hardcoding a bitmap or ad hoc shape.
      Inspector's open/dismiss/restore cycle, shell-visibility toggle, and session-return
      navigation are all correctly wired and tested.
- [x] Architecture conformance — pass. `visualization.ts` added to `src/types/` and
      re-exported from `index.ts`; `ARCHITECTURE.md`'s type inventory updated in the same
      task, per `.ai/quality-gates.md`'s structural-change rule. Domain data still only via
      `useSession`/`useConcept` hooks, called only from the page.
- [x] UI rules (hardcoded-value check only, full fidelity is Antigravity's job) — pass. No
      hardcoded px/hex/rgba in either touched `.module.css` file.
- [x] Process — pass. Independently reran `npm run typecheck`, `npm run lint`,
      `npm test -- --run` (45 files / 104 tests, matches worklog), `npm run build`, and
      `git diff --check`; all clean.

Verdict: changes-requested. The inverted palette is the blocking finding. It's a CSS/token
choice, so it's fair game for Antigravity to fix as the first step of the fidelity pass
already assigned — flagging it as changes-requested rather than leaving it silently bundled
into "polish," since it's a named-invariant violation, not a taste call.

## Follow-ups

Anything noticed during implementation or review that's out of this task's scope.

- (claude-code, 2026-08-29) The Bounds panel's two range inputs
  (`FullVisualizationPage.tsx:168,173`) use `defaultValue` (uncontrolled) while their adjacent
  `<output>` reads directly from the static `lower`/`upper` props — dragging either slider
  moves the thumb but the displayed number never updates. This doesn't require any Stage 8
  engine work to fix, just local `useState` + `value`/`onChange`, so it's not covered by "left
  out by design: ... advanced bounds behavior." Not blocking since it's untested either way
  and cosmetic, but worth fixing while someone's already in this file.
- (claude-code, 2026-08-29) Zoom controls (+/−/⌾) and the "Advanced…" button have no
  handlers — same dead-control category noted on 026/031/034, likely blocked on the Stage 8
  engine.

## Follow-ups

Anything noticed during implementation or review that's out of this task's scope.
