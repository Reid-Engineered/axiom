---
id: 060
title: Stage 8 roadmap sync and task 059 independent review
status: review
owner: codex
stage: 8
depends_on: [059]
---

## Scope

Documentation-only sync of Stage 8's recorded state: bring `ROADMAP.md` up to date through
task 059, correct two factually wrong claims carried in task 059's archived handoff and in
`knowledge-package/synthesis-report.md`, and record the independent review of task 059 that
its self-review explicitly said had not been performed.

Does **not** build: any production Rust, any frontend, any schema, dependency, or capability
contract change. No test was added or modified.

PR: https://github.com/Reid-Engineered/axiom/pull/4
Branch: `agent/codex/060-stage8-docs-sync`

## Plan

Files touched:

- `ROADMAP.md` — replace the "Remaining Stage 8 scope (not yet designed)" section with a
  completed-sub-projects list (045–059) and a restated remaining critical path.
- `.ai/tasks/_archive/059-first-production-problem-family.md` — correct the Checkpoint 2.13
  cross-check, correct the determinism-test rationale, replace the "One honest limit"
  section with an independent source check, and append an independent review section.
- `knowledge-package/synthesis-report.md` — same Checkpoint 2.13 correction, plus close
  open review item 4 (the OpenStax label check).

## Worklog

- 2026-09-05 — Work performed by codex on `agent/codex/060-stage8-docs-sync`; PR #4 opened.
- 2026-09-05 — This task file did not exist when the PR was opened. Reconstructed from the
  PR diff by claude during review so the branch's `060` id has a handoff record, per
  `.ai/lifecycle.md`. The Scope/Plan sections above describe what the diff does, not a plan
  codex wrote in advance — noted here so the record is not read as more than it is.
- 2026-09-05 — Reviewed by claude. Verdict `changes-requested` (findings below).
- 2026-09-06 — Codex resumed as owner; `changes-requested` → `in-progress`. Fetched
  `origin/master` and rebased before editing; Git reported the branch already up to date
  at base `9b43842`. The task record was still absent, so followed PR #4's fallback
  comment 5563248909 and imported the existing record from `claude/pr-review-af36e5`
  (file blob `e7eee36b4ce2a5d4876a656187c54b5b6209f80d`). Preserved its Scope, Plan,
  dated worklog, and Review; this is the reviewer's record, not a competing reconstruction.
  The user's revision scope authorizes Codex to update this documentation and task record.
  Revision plan: edit only `ROADMAP.md` and this file for findings 4–7 and the handoff;
  leave archived task 059, production code, and follow-up tasks 061/062 untouched.
- 2026-09-06 — Applied findings 4–7 in `ROADMAP.md` and completed this handoff. Chose
  finding 4's task-spec indexing resolution; removed finding 5's duplicate runtime entry
  and orphaned label; restored finding 6's Antigravity/presentation-only/no-engine-change
  boundary; and replaced finding 7's dangling reference with
  `reference/UI/AXIOM-HANDOFF.md`. Skipped optional finding 8 because the revision was
  scoped to the required process findings and archived task 059 was explicitly frozen.
  `git diff --check` and focused prose checks passed. Re-ran the task 062 Ubuntu frontend
  failure unchanged; it passed. CI is green except `backend-checks (windows-latest)`, which
  remains red for task 061's pre-existing SQLite cleanup failure. No source or test code
  changed. Work complete; `in-progress` → `review` for independent re-review.

## What was built / tested / left out

Three documentation corrections, all verified independently during review:

1. **Checkpoint 2.13 interval and answer.** The archived task 059 handoff and the synthesis
   report both claimed Checkpoint 2.13 was `f(x) = 3x − x²` on `[0, 3]` giving `27π/2`,
   i.e. `(c, b) = (3, 3)`. OpenStax §2.3 states the checkpoint on `[0, 2]`; the Chapter 2
   answer key gives `8π`. Corrected to `(c, b) = (3, 2)` → `8π` in both files.
2. **Determinism-test rationale.** The handoff claimed an attempt "persists only its seed and
   is replayed through the generator on every load". `PracticeStore` in fact serializes the
   complete `ProblemInstance` to `instance_json` (`src-tauri/src/practice/store.rs:42-56`)
   and deserializes it on load (`store.rs:138`). Corrected.
3. **Open review item 4 closed.** The synthesis report's "OpenStax label check" item is
   marked answered against the published Section 2.3 page.

Gates: documentation-only, so `npm run *` and `cargo *` do not apply to changed files.
CI on PR #4 is nevertheless red — see finding 1 below. `git diff --check` is clean.
`ARCHITECTURE.md` not updated; no structural change was made, so none was needed.

No acceptance criteria were copied into `ROADMAP.md`: finding 4 was resolved by making the
roadmap an explicit index to the specifications and task records that contain them.

Revision for findings 4–7:

4. Chose the indexing resolution: `ROADMAP.md` now states that the additional completed
   sub-projects' acceptance criteria live in their specifications and task records under
   `.ai/tasks/_archive/`, while the roadmap indexes them. The full criteria were not
   duplicated into the roadmap.
5. Removed the duplicate module/capability-runtime list entry and the orphaned `(locked)`
   label. Its existing sub-project section remains the single description with full
   deliverables and acceptance criteria.
6. Restored the Study Session UI owner and boundary verbatim: "Antigravity, presentation
   only — no engine/contract changes."
7. Replaced the unnamed "authoritative session design" reference with the repository's
   authoritative screen specification, `reference/UI/AXIOM-HANDOFF.md`.

Tested: `git diff --check`; focused text searches for the removed duplicate/orphaned wording,
the restored Study Session constraint, and the explicit handoff path; and inspection of the
PR's changed-file list to confirm the revision remains documentation-only. No frontend or
Rust test was run locally because no production or test code changed. CI is green except
`backend-checks (windows-latest)`, which remains red for task 061's pre-existing open-SQLite-
connection cleanup failure. The earlier `frontend-checks (ubuntu-latest)` task 062 flake was
re-run without a code change as `.ai/quality-gates.md` directs and passed.

Deliberately left out: findings 1 and 2's production test fixes remain tasks 061 and 062;
finding 3 is recorded and requires no code change; optional finding 8 was skipped to honor
the explicit instruction not to edit archived task 059 further and to keep this revision to
the required process findings. No production Rust, frontend, schema, capability contract,
or archived task 059 content changed.

## Review

Reviewer: claude
Date: 2026-09-05

Source and code claims were re-verified independently rather than taken from the PR body:
the OpenStax Section 2.3 page and Chapter 2 answer key were fetched directly (Rule 2.6,
Example 2.13 at `f(x)=2x−x²` on `[0,2]` → `8π/3`, Checkpoint 2.13 at `f(x)=3x−x²` on
`[0,2]` → `8π`); `2π(3·8/3 − 16/4) = 8π` was rederived; `store.rs` was read to confirm the
persistence correction; both cited test names were confirmed present
(`src-tauri/src/generation/tests/mod.rs:57` and `:80`); tasks 045–059 were confirmed
`status: done`; and `cargo test` was run locally (284 passed).

- [x] **Correctness — pass.** Every factual correction in the diff is right, and both
      corrected claims were genuinely wrong before. Dropping "Symbolica-CAS providers" from
      `ROADMAP.md` is also correct: task 055's worklog records Symbolica being ruled out
      with the human over licensing/offline-activation risk, so that line was stale.
- [x] **Architecture conformance — pass.** `PracticeStore` behaves as the corrected text now
      describes. No structural change; `ARCHITECTURE.md` correctly untouched.
- [x] **UI rules — N/A.** No files under `src/` changed.
- [ ] **Process — FAIL.** Findings 1, 2 and 3 below.

### Findings

1. **BLOCKING — required checks are red on PR #4.** Two distinct failures, neither caused by
   this PR:
   - `backend-checks (windows-latest)` fails deterministically at
     `src-tauri/src/practice/store.rs:345`: `std::fs::remove_dir_all(&dir)` runs while
     `reopened` still holds an open SQLite connection, giving
     `Os { code: 32, ... "being used by another process" }`. Windows sees 283 passed /
     1 failed, so this PR's "284 Rust tests passed" is true on Linux and macOS only.
     Split out as task 061.
   - `frontend-checks (ubuntu-latest)` fails on
     `src/pages/GoalEditingSheet.test.tsx:42`. This is the flake `.ai/quality-gates.md`
     already names by file as precedent from task 052, so per that policy it is a re-run,
     not a blocker — but the follow-up task that policy calls for was never filed, and it is
     still flaking. Split out as task 062.
2. **Missing handoff doc.** The branch carries the `060` id but shipped no
   `.ai/tasks/060-*.md`. `.ai/lifecycle.md` requires one, and CLAUDE.md makes `.ai/tasks/`
   the shared source of truth between agents that do not share a context window. This file
   is that record, written after the fact.
3. **Reviewer applied the fixes instead of recording them.** CLAUDE.md: "Don't silently fix
   another agent's findings while reviewing. Leave them as findings; the original author (or
   a follow-up task) applies the fix, so the handoff doc stays an accurate record of who did
   what." Task 059's owner is `claude`; codex edited its body. Task 055's own review section
   states this convention explicitly. Mitigating, and the reason this is recorded rather
   than reverted: 059 is archived and `done`, the corrections are factual, and the appended
   review section discloses exactly what was changed and why. Noted as a pattern to avoid
   next time, not as a change request against this diff.
4. **`ROADMAP.md` asserts completion against criteria it never records.** The diff removes
   "the rest get their own **Deliverables**/**Acceptance criteria** appended here once
   designed, not pre-decided now", then marks eight sub-projects complete without appending
   any. CLAUDE.md treats a stage's `ROADMAP.md` acceptance criteria as "the definition of
   done, not a suggestion". Either append criteria for 049–059, or state plainly that
   per-sub-project criteria live in each task's spec and the roadmap only indexes them.
5. **`ROADMAP.md` duplication and an orphaned heading.** `### Sub-project 1 — Module &
   Capability runtime (locked)` (which carries full Deliverables/Acceptance criteria) and the
   new `### Completed Stage 8 sub-projects` bullet "Module and capability runtime
   (`045`–`048`)" describe the same sub-project. "(locked)" is now unexplained, because the
   sentence defining the locking convention was the one removed.
6. **A scope constraint was dropped.** The removed text scoped Study Session UI integration
   as "(Antigravity, presentation only — no engine/contract changes)". The replacement drops
   both the owner and the no-engine-changes guarantee. Both are worth preserving verbatim.
7. **Dangling reference.** "polish the learner-facing problem, evaluation, and hint states
   against the authoritative session design" names no document that exists — nothing in the
   repo matches. It should point at `reference/UI/AXIOM-HANDOFF.md`, which
   `.ai/review-checklist.md` already treats as authoritative for screens.
8. **Nit — citation precision.** `knowledge-package/synthesis-report.md:195` cites the
   Section 2.3 page for "producing $8\pi$", but `8π` is published in the Chapter 2 answer
   key, not on that page. For an item whose entire purpose is pinning down sourcing, cite
   both. Relatedly, task 059's corrected checkpoint line reads `2π(8 − 4) = 8π` while the
   Example line above it shows the full substitution; write `2π(3·8/3 − 16/4)` so a reader
   can check the `(c, b)` substitution the same way in both.

Findings 4–7 are prose edits to `ROADMAP.md` and belong in this PR. Finding 8 is optional.
Findings 1–3 are process, and 1 is split into its own tasks rather than absorbed here.

Verdict: changes-requested

## Follow-ups

- **061** — `practice::store` test leaves a SQLite connection open across `remove_dir_all`,
  failing deterministically on Windows CI.
- **062** — `GoalEditingSheet.test.tsx` flake, still unfixed since task 052 flagged it.
