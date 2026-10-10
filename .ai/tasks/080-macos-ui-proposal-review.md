---
id: 080
title: Review macOS / Reid Engineered UI proposal
status: review
owner: claude (design)
stage: —
depends_on: []
---

## Scope

Add a design proposal under `reference/UI/proposals/2026-10-macos-reid/` for review: five
mockup screens (Today, Session, Miss feedback, Error profile, Session summary) drawn with the
Reid Engineered design system. Docs only.

Does not change `AXIOM-HANDOFF.md`, `ARCHITECTURE.md`, `AGENTS.md`, tokens, components or any
code. The proposal conflicts with several handoff invariants; adopting any part of it is a
separate, human-signed-off change.

## Plan

- `reference/UI/proposals/2026-10-macos-reid/README.md` — intent, conflicts table, open
  mechanics, data the screens assume
- `reference/UI/proposals/2026-10-macos-reid/{Main,Session,Feedback,Profile,Summary}.dc.html`
- `reference/UI/proposals/2026-10-macos-reid/axiom-theme.css`
- `reference/UI/proposals/2026-10-macos-reid/ds/reid/components/bundle.css`
- this task file

## Worklog

- 2026-10-09 — mockups drawn in a Claude design canvas from the Reid Engineered system
- 2026-10-10 — added to the repo as a proposal; opened PR for Claude Code review

## What was built / tested / left out

Built: the files listed under Plan. No code, so CI's frontend/backend gates are unaffected.

Not checked: the `.dc.html` files were not rendered in this repo's `support.js` runtime;
they were authored against the design canvas's runtime, which this one mirrors.

Left out on purpose: any redraw in the handoff's vocabulary, any edit to the handoff, and any
task for implementing the screens. Those depend on this review's outcome.

## Review

For the reviewer (Claude Code), against the proposal README:

1. Confirm or correct the "Conflicts with AXIOM-HANDOFF.md" table; add any conflict it missed.
2. For each of the proposal's four ideas (error profile screen, "why this question",
   visible consequence of a miss, session with a stated shape): is it already covered by the
   handoff or `ROADMAP.md`, worth adopting in the handoff's vocabulary, or a cut?
3. Mark each undecided mechanic v1 / later / cut, with the roadmap stage it would land in.
4. Check the "data the screens would need" list against `ARCHITECTURE.md` and `CORE.md`:
   which already exist, which are new IPC surface, and anything that would put
   domain-specific logic in the engine.

Findings only; record a verdict here. Don't edit the handoff or code in this task.

## Follow-ups

None yet — expected from the review (likely: a redraw task, and a handoff amendment if any
invariant is to change).
