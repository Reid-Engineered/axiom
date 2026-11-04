---
id: 076
title: Real elapsed-time tracking for sessions
status: done
owner: antigravity
stage: 9
depends_on: [073]
---

## Scope

Deferred follow-up from §7.1 of
`docs/superpowers/specs/2026-09-09-mock-and-dead-end-containment-design.md`.

`sessions.elapsed_minutes` is written once as `0` on insert and never updated, so the session
toolbar read `0′ of 20′` permanently. `073` removes the readout; nothing replaces it.

If session timing is still wanted, it needs `elapsed_minutes` genuinely maintained across
pause, resume and `nextProblem`, plus a decision about whether an elapsed-against-target
figure reads as context or as a score under `AGENTS.md:22` ("no percentages … never a number
presented as a score").

**Not required for `064` criteria 1, 2 or 6.** Filed so the removal in `073` is a recorded
deferral rather than a silent loss.

## Plan

Implement active foreground session timing per `083` §7 and policy `P7` (5-second checkpoint cadence, flush before session transitions, restart paused at acknowledged total, no auto-end at target).

## Worklog

- 2026-09-09 — Filed by claude alongside `073`, which removes the readout this would restore.
- 2026-11-04 — Claimed by antigravity alongside `085` as part of Track 1.
- 2026-11-04 — Implemented `session_timing_checkpoints` table and `SessionTiming` persistence in `src-tauri/src/practice/engine.rs`, foreground monotonic elapsed timer and 5-second checkpoint cadence in `src/hooks/useLearningEvidence.ts` (`useAdaptiveSession`), and live `MM:SS` / target readout in `src/pages/StudySessionPage.tsx`.

## What was built / tested / left out

- Built:
  - `session_timing_checkpoints` in `src-tauri/src/db/migrations/0005_learning_evidence.sql`
  - `SessionTiming` checkpointing and `sessions.elapsed_minutes` floor synchronization in `src-tauri/src/practice/engine.rs`
  - Foreground timer (`document.visibilityState === 'visible'` & `window.document.hasFocus()`) and 5-second checkpoint flush in `src/hooks/useLearningEvidence.ts`
  - Live `MM:SS` timer pill in `src/pages/StudySessionPage.tsx`
- Tested:
  - `src-tauri/src/practice/engine.rs` (`adaptive_session_records_evidence_patterns_and_history_idempotently`)
  - `src-tauri/src/commands/tests.rs` (`stage_9_learning_evidence_commands_run_end_to_end`)
  - `src/pages/StudySessionPage.test.tsx`

## Review

Ready for review.

## Follow-ups

### Adoption clarification — 2026-10-10

Task 081 adopts elapsed/target timing and numeric learning evidence. The old score-ban
question in this task's original scope is resolved by the human's full Reid adoption.
Coordinate the persisted timer contract with 083 and implementation with 085; this
clarification does not claim the timer already exists.
