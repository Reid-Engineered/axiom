---
id: 007
title: layouts/* stub contracts
status: done
owner: claude-code
stage: 2
depends_on: [005]
---

## Scope

Full TypeScript prop interface + TSDoc for `SessionShell`, `FullVisualizationShell`, `TwoPaneLayout`, `CenteredColumnLayout` — stub bodies only. Also reconfirms `AppShell`'s contract (already has an empty implementation from Stage 0) against the rest of the layout inventory in `ARCHITECTURE.md` §2, without changing Stage 0's behavior.

## Plan

- src/layouts/SessionShell.tsx (stub)
- src/layouts/FullVisualizationShell.tsx (stub)
- src/layouts/TwoPaneLayout.tsx (stub)
- src/layouts/CenteredColumnLayout.tsx (stub)
- src/layouts/AppShell.tsx (contract review only, no behavior change)

## Worklog

- 2026-08-27 — started, claimed by claude-code. Branch `agent/claude-code/007-layouts-contracts`.
- 2026-08-27 — wrote 4 layout stubs (slot-based props, `return null` bodies) and reviewed
  `AppShell`'s existing contract against them. All gates pass. Moved to `review`.
- 2026-08-27 — Codex: `changes-requested` — `CenteredColumnLayout.maxWidth: 520 | 560` bakes
  raw mockup pixels into the public API instead of tracing to a token. Confirmed: `tokens.css`
  had no width metric at all for this layout (Stage 0 gap, surfaced here). Added
  `--layout-column-width: 520px` / `--layout-column-width-wide: 560px` to `tokens.css`'s
  metrics section, and changed the prop to a semantic `width?: 'default' | 'wide'` variant —
  matching how `Button`/`Chip` already expose `size`, not raw pixel choices. All gates
  re-run — pass. Moved back to `review`.
- 2026-08-27 — Codex re-review: correctness/architecture/UI-rules all pass now, but flagged
  Process — this branch was cut before 013 merged to `master`, so merging as-is would look
  like it reverts 013's `src/components/feedback/*` and un-archives its task. `git rebase
  master` — clean, no conflicts. All gates re-run against the rebased tree — pass. Not
  merging myself: I authored this task, and `.ai/merge-strategy.md` requires a different
  agent (or the human) to record the pass. Moved back to `review`.

## What was built / tested / left out

- **Built**: `SessionShell.tsx`, `FullVisualizationShell.tsx`, `TwoPaneLayout.tsx`,
  `CenteredColumnLayout.tsx` — each takes `ReactNode` slot props (toolbar/panes, header/
  content, content/rail, content) rather than domain data, since layouts are structural,
  not data-fetching (`ARCHITECTURE.md` §5 rule 1 — only pages call data hooks). Grounded in
  screens 1, 5, 6 and the page→layout table in `ARCHITECTURE.md` §3.
- **Tested**: `npm run typecheck` (0 errors), `npm run lint` (0 errors), `npm run build`
  (succeeded), grep for stray hex/`rgba(` (0 hits). No render tests yet — bodies are
  `return null`, nothing to assert; real tests land with Stage 3's implementation (015).
- **Left out / notable decisions**:
  - `AppShell.tsx` untouched — reviewed its existing `{ children?: ReactNode }` contract
    from Stage 0 against the other four layouts' shape; consistent, no change needed.
    Sidebar prop will be added in Stage 3 (015/017), not here.
  - `CenteredColumnLayout`'s `width` is now `'default' | 'wide'` (520px / 560px via new
    `tokens.css` metrics), not raw numbers — 'default' for screen 1, 'wide' available for a
    future consumer (not yet clear which page uses it; screen 11's 560px sheet is a `Sheet`
    overlay, not this layout, per `ARCHITECTURE.md` §3).
  - Added two tokens to `src/styles/tokens.css` (a Stage 0 file) — flagging per
    `.ai/quality-gates.md`'s "highest blast radius" note for shared-file edits, same as
    006's revision of `workspace.ts`.

## Review

Reviewer: codex
Date: 2026-08-27
- [ ] Correctness — FAIL: `CenteredColumnLayout.maxWidth` exposes mockup pixel values
  (`520 | 560`) as component API. Width choice should be semantic (or internal) so the
  implementation can source the actual dimensions from `tokens.css`, as required by the
  design-token rule.
- [x] Architecture conformance — pass: layouts are structural, slot-driven stubs and do not
  fetch domain data.
- [ ] UI rules — FAIL: the numeric max-width contract bypasses design tokens.
- [x] Process — pass: typecheck, lint, build, hardcoded color/rgba grep, and diff check pass.

Verdict: changes-requested

Filled in by the reviewing agent. See `.ai/review-checklist.md`.

### Re-review — 2026-08-27

- [x] Correctness — pass: the prior task-specific contract finding is resolved.
- [x] Architecture conformance — pass: the revised contract remains within Stage 2 scope.
- [x] UI rules — pass: the revised API preserves the applicable product invariants.
- [ ] Process — FAIL: this branch is based before merged task 013. Compared with current
  `master`, it deletes `src/components/feedback/*`, removes the archived 013 task, and
  restores 013 to the active queue. Per `.ai/merge-strategy.md`, the owner must rebase onto
  current `master` and rerun all gates before re-requesting review.

Verdict: changes-requested

### Exact-HEAD re-review — 2026-08-27

Reviewed commit: `172b9a7`

- [x] Correctness — pass: the original contract finding is fully resolved.
- [x] Architecture conformance — pass: the contract remains props-driven and within the
  locked Stage 2 boundary.
- [x] UI rules — pass: applicable product invariants and token rules are preserved.
- [x] Process — pass: this exact HEAD is rebased on `170061a`; its diff contains only the
  task's intended files. Independent typecheck, lint, build, hardcoded-value grep, and
  diff check all pass.

Verdict: pass

## Follow-ups

Anything noticed during implementation or review that's out of this task's scope.
