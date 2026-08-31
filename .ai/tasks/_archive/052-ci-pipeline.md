---
id: 052
title: CI pipeline (GitHub Actions)
status: done
owner: claude-code
stage: N/A — tooling/infrastructure, not a ROADMAP.md stage
depends_on: []
---

## Scope

Stand up GitHub Actions CI (`.github/workflows/ci.yml`) enforcing `.ai/quality-gates.md`'s
gates mechanically, and adopt the PR-based branch workflow `.ai/merge-strategy.md` already
documents. Full design: `docs/superpowers/specs/2026-08-31-ci-pipeline-design.md`. Full
task breakdown: `docs/superpowers/plans/2026-08-31-ci-pipeline.md`.

Does not build: agent orchestration, CD/release builds, or macOS/Windows e2e — all tracked
as follow-ups in the spec's §8.

## Plan

Files to be created or touched:
- Create: `.github/workflows/ci.yml`
- Modify: `.ai/quality-gates.md`
- Modify: `.ai/tasks/TEMPLATE.md`

## Worklog

- 2026-08-31 — started, claimed by claude-code
- 2026-08-31 — executed via subagent-driven-development in an isolated worktree; 7
  planned tasks completed (task file, workflow YAML, quality-gates.md/TEMPLATE.md
  edits, `gh` CLI install + Marcus's device-code auth, PR opened, branch protection
  configured, deliberate-failure validation) — see PR #1 for the full commit history.
- 2026-08-31 — merged (squash), commit 3c6be53. Archiving.

## What was built / tested / left out

**Built:** `.github/workflows/ci.yml` (3 jobs — `frontend-checks`/`backend-checks` on
Linux+macOS+Windows, `e2e` on Linux, matching
`docs/superpowers/specs/2026-08-31-ci-pipeline-design.md` exactly), branch protection
on `master` requiring all 7 job/matrix-leg checks, and the `.ai/quality-gates.md` /
`.ai/tasks/TEMPLATE.md` edits pointing at CI as the source of truth for gate results.

**Tested:** every task got an implementer + independent task-reviewer subagent pass
(all approved, no Critical/Important findings) — full detail in
`docs/superpowers/plans/2026-08-31-ci-pipeline.md`'s companion ledger (now deleted
with the worktree; this file and PR #1 are the durable record). Beyond task review,
the pipeline was validated against the real system, not just read: PR #1
(https://github.com/Reid-Engineered/axiom/pull/1) ran all 7 checks green, a
deliberate lint violation was pushed and confirmed `mergeStateStatus: BLOCKED` via
`gh pr view`, then reverted and confirmed `CLEAN`.

**Left out (deliberately, per spec §1):** agent orchestration, CD/release builds,
extending e2e to macOS/Windows.

**Deviations from the plan, ruled on live (see PR #1 commit history for the fix):**
- Neither the spec nor Task 2's brief anticipated Tauri's native Linux build
  dependencies — GitHub's bare `ubuntu-latest` runner has none of the GTK/WebKit dev
  libraries Tauri needs, so `backend-checks (ubuntu-latest)` and `e2e` both failed on
  the first real PR run (`pkg-config` couldn't find `glib-2.0`/`gobject-2.0`). Fixed
  by adding the standard Tauri CI apt step (`libwebkit2gtk-4.1-dev`,
  `libappindicator3-dev`, `librsvg2-dev`, `patchelf`) to both jobs; re-run confirmed
  green. Scoped review of the fix: approved, no blocking findings.
- `gh` CLI was installed via the standalone binary release into `~/.local/bin`
  instead of the plan's apt-based method — this sandbox has no passwordless `sudo`.
  Same outcome, no functional difference.
- Tasks 4-6 (gh install/auth, push+open PR, branch protection) produced no git diff
  to review, so they were executed directly in the controller session rather than
  via implementer+reviewer subagent dispatch — ledgered as rulings at the time.

## Review

Reviewed via the subagent-driven-development process: every task (1-3) got an
independent task-reviewer subagent pass before being marked complete, all approved
with no Critical/Important findings. The Tauri-deps fix (discovered live during
Task 5) got its own scoped review, also approved. Final human sign-off from Marcus
before merge (required — this task touches `.ai/quality-gates.md` and
`.ai/tasks/TEMPLATE.md`, both under `.ai/merge-strategy.md`'s human-sign-off rule).

**Verdict: pass.**

## Follow-ups

- Extend e2e to macOS and Windows — needs their native WebDriver setup researched and
  documented first; `e2e/README.md` has no Mac/Windows section today. (spec §8)
- Agent orchestration spec (separate brainstorm) — depends on this pipeline existing
  as the source of truth for gate status. (spec §8)
- CD spec (separate brainstorm) — depends on this pipeline; revisit once Stage 7
  ships. (spec §8)
- `GoalEditingSheet.test.tsx`'s "allows inferred facets to be corrected without
  changing the goal text" test failed once, flakily, in CI during Task 7's
  validation — passed clean on an immediate retry with no code change. Unrelated to
  this task's scope (pre-existing test, untouched by this work), but worth its own
  investigation task since this is the first time it's been caught mechanically.
- Consider apt package caching (e.g. `awalsh128/cache-apt-pkgs-action`) for the new
  Tauri Linux dependency install step — both `backend-checks (ubuntu-latest)` and
  `e2e` currently pay the full `apt-get update && install` cost every run. Flagged as
  a minor by the Tauri-deps fix's review; deferred as out of scope for that fix.
