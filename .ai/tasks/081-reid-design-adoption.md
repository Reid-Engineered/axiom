---
id: 081
title: Adopt Reid Engineered visual and product direction
status: review
owner: codex
stage: 9
depends_on: [080]
---

## Scope

Record the human-approved full adoption of PR #25's design as the new product authority.
Align the UI handoff, agent rules, architecture migration boundary and roadmap. This is
the design-contract amendment, not runtime implementation of five new screens/engines.

The human explicitly approved both the visual design and product direction on 2026-10-10.
This supersedes task 080's advice based on the former UI rules. The normal role lanes remain
in effect for follow-up implementation; Codex prepared this amendment at the human's request.

## Plan

- reference/UI/AXIOM-HANDOFF.md and an archived copy of its former version.
- reference/UI/proposals/2026-10-macos-reid/README.md status/precedence.
- AGENTS.md, ARCHITECTURE.md, ROADMAP.md and review-checklist alignment.
- Adoption clarifications appended to proposed timing/polish tasks 076/078.
- docs/superpowers/specs/2026-10-10-reid-design-adoption.md.
- Proposed tasks 082–086; archive completed task 080 with its original review preserved.

## Worklog

- 2026-10-10 — Human requested adoption and selected "Adopt the full design and product direction."
- 2026-10-10 — Prepared authoritative amendment and bounded implementation tasks.
  No production source, dependency, token or runtime contract changed in this task.

## What was built / tested / left out

Built: the adoption documents and tasks listed above. Retained the earlier handoff as
historical material; the new handoff defines current product direction.

Validation: git diff --check, Markdown local-reference checks, task-id/dependency checks,
and a consistency search for obsolete prohibitions in active authority docs. Mechanical
CI runs on the linked PR; no runtime test result or browser-fidelity result is claimed by
this docs-only amendment.

PR: pending creation.

Left out: runtime redesign, calibrated numeric policies, new capability contracts and
migrations, native chrome implementation, and browser/platform fidelity verification.
These are scoped in 082–086 rather than represented as already implemented.

## Review

Human sign-off on the direction is recorded above. Independent review of this authored
amendment and required CI are still pending; the author does not self-approve the PR.

## Follow-ups

082–086 implement the adopted direction. Existing 067/068 regression/offline acceptance
work and 076 timing work remain relevant. 078's historical visual target is superseded
by 082; do not implement a return to the old palette.
