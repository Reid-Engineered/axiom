---
id: 082
title: Reid visual foundations and native shell
status: proposed
owner: antigravity
stage: 9
depends_on: [081]
---

## Scope

Port the adopted theme to production tokens and shared primitives; restyle the current shell and existing screens. Keep existing domain/IPC contracts until their replacements are locked. This task does not fabricate numeric mastery or add an adaptive engine.

Product authority: reference/UI/AXIOM-HANDOFF.md and
docs/superpowers/specs/2026-10-10-reid-design-adoption.md.

## Plan

- src/styles/tokens.css and global.css; existing primitive/badge/mastery CSS Modules.
- AppShell / Sidebar / shared inspectors and relevant layout styles.
- Render the five adopted references and compare live app surfaces.
- Validate macOS native controls and Windows/Linux chrome; add tokens for opaque glass fallback.

Acceptance:
- Centralized tokens, readable focus/contrast, responsive panels and working controls.
- Existing first-launch/create/resume/check/hint flows still work.
- Conditional variants have render tests; frontend gates pass and native smoke flow remains green.

## Worklog

- 2026-10-10 — Scoped following the human's full design/product adoption in task 081.

## What was built / tested / left out

Not started. Implementation must name its checks and link its PR.

## Review

Pending implementation and independent review.

## Follow-ups

Split implementation further if a locked contract or file boundary requires it.
