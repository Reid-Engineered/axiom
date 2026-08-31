---
id: 052
title: CI pipeline (GitHub Actions)
status: in-progress
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

## What was built / tested / left out

(filled in when moving to review)

## Review

(filled in by reviewer)

## Follow-ups

(filled in when moving to review — see spec §8 for known ones)
