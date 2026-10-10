---
id: 080
title: Review macOS / Reid Engineered UI proposal
status: done
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

Reviewer: Codex (independent of the Claude design author)
Date: 2026-10-10
Scope of approval: retain an explicitly unadopted proposal. This does not approve its
visual language, mechanics, navigation, or implementation.

- [x] Correctness — pass for proposal scope: the nine added files match the brief; no
  application source, configuration, package, authoritative handoff, or runtime changes.
- [x] Architecture conformance — pass for proposal scope: assumptions are documented
  below, not introduced as engine behavior or locked contracts.
- [x] UI rules — pass for proposal isolation only: the README explicitly preserves the
  authoritative handoff; the mockups themselves do not conform and must not be implemented
  as supplied.
- [x] Process — independent review recorded; the user's request to review and merge if
  suitable authorizes retaining these proposal/task documents. Merge remains conditional
  on every required CI check passing on the final PR head.

### Conflicts table review

The listed palette, chrome, math typography, mastery, dashboard, and navigation conflicts
are substantially correct. Extend the table's interpretation with these findings:

- Feedback's "streak to clear it" and Profile's "5 clean attempts in a row" conflict with
  invariant 3 independently of numeric mastery. Do not expose a streak as progress.
- Main/Session/Summary's fixed mastery costs for misses and hints make evidence into a
  score and penalize seeking help. Cut this mechanism from the core UI.
- Session's template id, generator parameters, checker implementation claims, and Bloom
  promotion thresholds expose internals rather than learning reasons (§6.1–2).
- The always-open inspectors have nonfunctional dismiss buttons in these static mockups.
  Any adopted design must make inspectors secondary and dismissible (§2).
- Main replaces remembered Continue context with a new plan, and makes the graph prominent.
  Preserve exact resume/context recovery (§4 screen 3, §5 returning after time away);
  the graph remains optional rather than the default (§5 concepts at scale).
- A standalone exam card is not inherently forbidden if it reflects the Guiding goal;
  it must not establish a second independent deadline or recommendation source.
- "Correct." is explicitly allowed by §2 Copy rules. Feedback does name the missing
  constant, so its problem is the red/score/streak treatment, not an absence of diagnosis.

These are adoption findings, not requests to make an intentionally divergent exploration
look like the current app before storing it.

### Four ideas

| Idea | Ruling |
|---|---|
| First-class error profile | Later: retain recurring misconceptions and dated evidence as a workspace-scoped optional detail surface, not a permanent sidebar destination or dashboard. Current concept diagnostics already cover individual observations; aggregate pattern status is new. |
| Why this question | Adopt in the handoff vocabulary: extends existing ReasonedRecommendation / Why this? and diagnostic practice. Explain actual selection evidence in learner terms; omit template ids and algorithm thresholds. |
| Visible consequence of a miss | Adopt explanatory feedback in the current Study Session integration; automatic question replacement is later, after a versioned selection/resume contract exists. Never claim a replacement happened unless persisted state actually changed. |
| Session with a stated shape | Already covered in principle by the session-intent Home alternate and reasoned plan rows. Keep an editable, goal-related plan; new automatic warm-up/stretch/combined scheduling is later. |

"Adopt" above means a recommendation for a separately scoped presentation/contract task,
not a handoff amendment made by this PR.

### Undecided mechanics

| Mechanic | Ruling / stage |
|---|---|
| Five consecutive clean attempts resolve a pattern | Cut the visible streak; later (Stage 9+) design opportunity-based, spaced diagnostic evidence and resolution policy with the mastery capability. Five consecutive answers alone do not establish durable mastery. |
| Swap the next question after a miss | Later (Stage 9+ adaptive selection capability); lock persistence, resume, user agency, and duplicate/retry behavior first. Stage 8 currently proves one production family and bound attempts. |
| Steady / Push / Exam with hint budgets and miss costs | Cut mastery penalties and help costs. Later (Stage 9+) optional exam conditions can be separately designed; family difficulty metadata is not a learner difficulty-profile contract. |
| Per-template misconception attribution | Later (Stage 9+ Knowledge Package / Practice diagnostic contract). Existing objective/concept links and optional concept misconceptions do not specify per-family trigger opportunities or establish that a learner used a particular method. |
| Checker detail | v1 (remaining Stage 8 UI integration): show only truthful supported correctness/status and requested hints. Later (Stage 9+ versioned verification diagnostics): term-specific evidence. Cut sample counts, domains, and implementation detail from the default learner UI. |

### Data and capability boundary

- Existing concept DTOs supply masteryState, wasMasteryState, decayedAt, lastActivityAt,
  review timing, and recentDiagnostics. lastActivityAt is general activity, not a reliable
  last-practised timestamp; a dedicated practice-derived value needs a contract.
- Existing Session supplies intent, elapsed time, problem position/count, resume context,
  and currentAttemptId. It has no multi-concept phased plan, per-phase reason, adaptive
  queue, or learner difficulty profile.
- Practice supplies attempt prompt/response type, hint counts, hydration, and evaluation
  correctness/status/submission count. math.verify returns is_correct and an optional
  error; it does not return symbolic proof, sampled-domain diagnostics, missing-constant
  attribution, or triangle-construction evidence. The mockups' checker claims are imagined,
  not currently supported behavior.
- Aggregate occurrences/opportunities, first/last seen, clean-run status, durable mastery
  updates, and adaptive scheduling require new versioned capability outputs plus
  persistence/IPC and frontend hook contracts. Existing diagnostics are not that engine.
- Concept/objective relationships, family difficulty ranges, and package misconception
  content provide ingredients, not the proposed policies or opportunity model.
- Keep subject-specific diagnosis, prompt selection, hints, and resolution policy inside
  capability providers. Core routes opaque serializable workspace-scoped data; it must
  never hardcode "+ C", secant triangles, or subject-specific scheduling rules (CORE §1–2).
  Preserve services → hooks → pages → props-driven components.

### Validation and limits

Reviewed all nine additions at fa2ea912d815c6d49b76d1424f838c59c2698936,
the authoritative handoff, Architecture, Core, roadmap, and current PR-head DTO/provider
contracts. git diff --check passed. The five screens' local stylesheet/script targets
exist at their resolved repository paths. No new application tests are appropriate for
this isolated reference proposal. CI must still pass its frontend typecheck/lint/build/
Vitest/design-token checks on all three platforms, Rust check/test/clippy/fmt on all three
platforms, and Linux native E2E before merge.

No browser rendering or interaction validation was performed; this review does not certify
visual fidelity or functional mockup controls. Any redraw must render the screens and fix
illustrative inconsistencies: Summary's washer example does not require trig substitution,
and the "+ C" clean-opportunity count needs a consistent question-by-question history.

Verdict: pass

## Follow-ups

- Redraw selected ideas using the existing handoff vocabulary, exact Continue behavior,
  optional dismissible inspectors, and workspace navigation. Render the redraw before
  presentation approval.
- Separately scope/version diagnostic evidence and adaptive planning capabilities for
  Stage 9+; do not infer an implementation mandate or invariant amendment from this PR.


### Completion and later adoption

- 2026-10-10 — PR #25 squash-merged as dd886d2e67a1db72a2995fdf965209844a378a60
  after all seven required CI checks passed; task branch deleted.
- 2026-10-10 — The human subsequently approved full visual/product adoption. Task 081
  supersedes this review's cut/defer recommendations where they relied on the prior
  handoff. The original review above is retained as historical evidence.
