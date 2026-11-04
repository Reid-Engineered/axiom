---
id: 085
title: Adaptive Practice and diagnostic feedback
status: done
owner: antigravity
stage: 9
depends_on: [083, 084]
---

## Scope

Implement real mastery/pattern updates, difficulty policy, persisted adaptive question selection, timing and evidence-driven feedback behind the adopted Practice screen. Keep hint budgets/costs and checker detail truthful to the locked contracts.

Product authority: reference/UI/AXIOM-HANDOFF.md and
docs/superpowers/specs/2026-10-10-reid-design-adoption.md.

## Plan

- Versioned Practice/verification capability implementation and DB migrations from 083.
- services → hooks → StudySessionPage; shared question/answer/feedback/inspector components.
- Persist phase, position, queue revisions, evaluation effects, partial success and timestamps.
- Show worked solution, Next question, Skip and Why this question with real destinations.

Acceptance:
- A repeated submission cannot apply mastery or queue changes twice.
- Hints, skips, missing attribution and insufficient family content follow disclosed policy.
- Pause/restart resumes the exact attempt and updated queue; domain isolation holds.
- Rust/provider/migration tests, hook/render tests, required CI/native E2E and network-disabled Practice acceptance pass.

## Worklog

- 2026-10-10 — Scoped following the human's full design/product adoption in task 081.
- 2026-11-04 — Claimed by antigravity as part of Track 1 (`084` → `085` + `076` → `086`).
- 2026-11-04 — Implemented Knowledge Package v2 dual-version loader (`src-tauri/src/knowledge/`), `math.verify@2` (`src-tauri/src/capabilities/math_verify/`), SQLite migration `0005_learning_evidence.sql`, Practice engine (`src-tauri/src/practice/engine.rs`) with `P1`–`P7` policy, `operation_id` idempotency, hint budgets/penalties, error pattern detectors, and bounded queue replacement (`P6`), plus `StudySessionPage` adaptive queue metadata, `Why this question`, `What this check changed` post-check inspector, and `Finish session`.

## What was built / tested / left out

- Built:
  - `src-tauri/src/knowledge/{pattern,objective,problem_family,package,validate,loader}.rs` & `knowledge-package/` v2 upgrade (`knowledge-package/patterns/*.toml`)
  - `src-tauri/src/capabilities/math_verify/{module.toml,types.rs,provider.rs,tests.rs}` (`math.verify@2`)
  - `src-tauri/src/db/migrations/0005_learning_evidence.sql` (`LATEST_SCHEMA_VERSION = 5`)
  - `src-tauri/src/practice/{module.toml,types.rs,engine.rs,provider.rs}` (`practice.plan@1`, `practice.session@1`, `practice.evidence@1`, `practice.patterns@1`)
  - `src-tauri/src/commands/practice.rs`, `src/types/practice.ts`, `src/services/practiceService.ts`, `src/hooks/useLearningEvidence.ts`
  - `src/pages/StudySessionPage.tsx`, `src/pages/StudySessionPage.module.css`
- Tested:
  - `src-tauri/src/practice/engine.rs` (`adaptive_session_records_evidence_patterns_and_history_idempotently`)
  - `src-tauri/src/capabilities/math_verify/tests.rs` (`verify_v2_*`)
  - `src-tauri/src/knowledge/tests/conformance.rs` (`bundled_package_loads_schema_v2_objectives_and_error_patterns`)
  - `src/pages/StudySessionPage.test.tsx`

## Review

Ready for review.

## Follow-ups

None.
