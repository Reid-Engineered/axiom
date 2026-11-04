---
id: 083
title: Lock mastery diagnostic and adaptive-session contracts
status: review
owner: codex
stage: 9
depends_on: [081]
---

## Scope

Design/version provider-owned mastery, Bloom/readiness, misconception opportunities, difficulty costs/hint budgets, resolution rules, persisted plans/queues, timing and verification evidence. Resolve actual policy values before implementation. No subject-specific policy in Core.

Product authority: reference/UI/AXIOM-HANDOFF.md and
docs/superpowers/specs/2026-10-10-reid-design-adoption.md.

## Plan

- CORE.md and ARCHITECTURE.md addenda; capability and Knowledge Package specs under docs/superpowers/specs/.
- Inventory existing src/types/*, src-tauri/src/{practice,knowledge,capabilities,commands,db}.
- Lock payloads, error/unavailable states, migrations, session resume and idempotency boundaries.
- Coordinate existing task 076 timing scope; retain Stage 8 regression/offline work.

Design delivery plan (after the brainstorm/spec steps recorded below):
1. Add the inventory, wire/persistence contracts, policy proposals and conformance plan in
   `docs/superpowers/specs/2026-11-03-learning-evidence-contracts.md`.
2. Add the versioned package metadata/validation contract in
   `docs/superpowers/specs/2026-11-03-knowledge-evidence-extension.md`.
3. Add scoped boundary/data-flow addenda to `CORE.md` and `ARCHITECTURE.md`.
4. Validate documentation references, scope and whitespace; record open decisions and
   docs-only gate applicability here; commit/push and open a PR for independent review.
   No production types, migrations, package content or task 082 styling are edited.

Acceptance:
- Every adopted field has a real owner, provenance, version and persistence rule.
- No invented numeric conversion of legacy mastery; define absent/historical evidence behavior.
- Opportunity attribution differs from observed misconception; policies disclose costs.
- Conformance/test plan covers repeated submissions, skips, domain isolation, restart, insufficient content and numerical verification limits.

## Worklog

- 2026-10-10 — Scoped following the human's full design/product adoption in task 081.
- 2026-11-03 — Human reassigned the design lane to Codex. Claimed on
  `agent/codex/083-learning-evidence-contracts`; read the required authorities and
  checked active task records. 081 is merged although its record remains `review`;
  082 is in a separate worktree per human instruction. No styling files will change.
- 2026-11-03 — Brainstorm: compared UI-derived scores, an event-bus redesign, and
  provider-owned transactional evidence. Chose the last: it preserves Core isolation,
  exact resume and offline operation without adding an event bus. Rejected converting
  named mastery or treating final-answer agreement as method diagnosis. The
  brainstorming skill referenced by CLAUDE.md was not installed in the searched skill
  roots/plugin cache; this records the alternatives without claiming a skill run.
- 2026-11-03 — Spec step: inventory and proposed contracts in
  `docs/superpowers/specs/2026-11-03-learning-evidence-contracts.md` and
  `docs/superpowers/specs/2026-11-03-knowledge-evidence-extension.md`.
  Structural boundaries are reviewable; uncalibrated policy values remain explicit
  human decisions, not approved implementation defaults.
- 2026-11-03 — Delivery plan executed: added both specs and scoped CORE/ARCHITECTURE
  addenda, without touching runtime or 082 files. Reviewed field ownership/provenance,
  legacy behavior, opportunity semantics and retry/resume boundaries against the handoff.
  Moved to review after documentation validation; open policies are deliberately not
  locked as approved defaults. Independent reviewer/human must resolve them before
  policy-dependent implementation and sign off shared-document changes before merge.

## What was built / tested / left out

Two proposed specs and CORE/ARCHITECTURE addenda define provider ownership, provenance,
wire/policy/package/storage versions, immutable snapshots, atomic action receipts,
opportunity vs diagnosis, exact attempt/draft/feedback resume, queue changes, timing,
verification limits and legacy/unavailable states. Existing code inventory and future
migration/conformance cases are included. No runtime implementation or fabricated evidence.

Policy-dependent implementation remains blocked on the human decisions below. Task 076
retains timer implementation scope coordinated with 085; 067/068 remain required outstanding
Stage 8 regression/offline proof. Independent review and human sign-off are still required
before merge under `.ai/merge-strategy.md`; this owner does not self-approve.

Validation on 2026-11-03:
- Passed local Markdown target validation: all 14 relative links in the five delivery
  documents resolve. Passed `git diff --check`.
- Manual scope/authority check: only this task record, CORE.md, ARCHITECTURE.md and the
  two specs changed; no tokens, primitives, styling, runtime types or schema edits.
  Architecture addendum records the proposed transactional data-flow rule in this task.
- Manual acceptance coverage: owners/provenance/version/persistence inherit explicit
  stamps; legacy numeric conversion forbidden; opportunities distinct from diagnoses;
  costs disclosed; future conformance includes retries, skips, domain isolation, restart,
  insufficient content and numeric-checker limitations. No visual-fidelity claim.
- Local frontend typecheck/lint/build/tests/token grep and Rust check/test/clippy/fmt:
  not run, not applicable to a docs-only diff under `.ai/quality-gates.md`. No shared
  runtime type consumers changed. Native Linux E2E not run locally on this Windows host.
  All seven required CI jobs, including unconditional E2E, must pass on the PR before merge;
  the PR is the mechanical-gate authority. No runtime conformance tests claimed as run.
- GitHub CLI is absent; use the connected GitHub tool for PR creation after git push.
  PR: [#27](https://github.com/Reid-Engineered/axiom/pull/27). No repository-specific Codex co-author trailer
  convention was found in the required authorities; no invented trailer is added.

## Open policy decisions for the human

All values are **proposals, not approved defaults**. The full formulas, reset semantics and
rationale are in the evidence spec §8; approve or replace each before implementation:

- P1: weighted observed success ratio, partial=0.5 where supported; thresholds 0.4/0.7/0.9;
  five trials per scoped objective before Mastered. No prior or legacy score conversion.
- P2: 30-day exponential half-life, daily persisted projection; prerequisite readiness 0.7.
- P3: Bloom needs three unassisted first-check correct attempts across two sessions on
  authored level-tagged objectives; no inferred untested levels or automatic Bloom decay.
- P4: five clean opportunities to resolve, editable 1–20; unknown interrupts streak;
  observed recurrence resets; rule edits start a new revision/streak without rewriting history.
- P5: Steady/Push/Exam budgets 3/1/0 bounded by available hints; per-hint credit discounts
  0/0.05/0; no extra negative miss/skip debit. Solution reveal before assessment has no
  credit; hints prevent clean/Bloom qualification. This proposes the meaning of costs.
- P6: editable 20-minute/10-question default, limits 1–180 minutes/1–50 questions,
  phase shares 30/40/remainder; deterministic evidence ordering and at most one pending
  replacement per observed miss, with explicit limited-content/difficulty fallback.
- P7: active foreground time including feedback/tools, five-second checkpoint cadence,
  restart paused at acknowledged total, no auto-end or speed scoring.

## Review

Pending independent review and human policy/architecture sign-off. No pass verdict claimed.

## Follow-ups

Split implementation further if a locked contract or file boundary requires it.
The specs name the implementation order for 084–086, 076, and package/detector support;
they do not claim that schema support alone supplies new verified content.
