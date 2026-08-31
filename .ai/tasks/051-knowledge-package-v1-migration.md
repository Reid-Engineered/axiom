---
id: 051
title: Knowledge Package v1 — migrate knowledge-package/ (Calc II) content
status: in-progress
owner: codex
stage: 8
depends_on: [050]
---

## Scope

Migrates the existing `knowledge-package/` (Calc II cylindrical-shells reference content)
to the Knowledge Package v1 format, against the now-proven, gate-clean loader/validator
built in [050](050-knowledge-package-v1-implementation.md). Covers plan Tasks 15–17 of:

```text
docs/superpowers/plans/2026-08-30-knowledge-package-v1.md   (implementation plan, 17 tasks)
```

050's own Task 14 acceptance carries the plan's process note verbatim: this migration is a
deliberately separate workstream from the runtime implementation, tracked as its own task
file and reviewed by a different agent than whoever implements it, per this repo's usual
`.ai/` process — not a continuation of 050 under a shared task id.

Same execution discipline as 050: one plan Task at a time, reviewed and accepted before the
next is issued, findings recorded in this file (not fixed silently by the reviewer), any
plan-level defect fixed at the plan-document level with the fix and its reasoning recorded
here before re-issuing — see 050's Worklog/Review sections for the established pattern
(six plan defects were found and fixed that way across Tasks 1–14).

Explicitly out of scope, same as the plan itself: Canonical Problem, `math.verify`,
Practice, Tutor integration, UI, Docling ingestion, `knowledge.query`, and any change to
the `src-tauri/src/knowledge/` runtime module itself (that module is done and frozen as of
050 — this task only produces content that loads through its existing public API).

## Plan

Files expected to change, per the implementation plan's own Task 15–17 file lists:
- `knowledge-package/package.toml`, `knowledge-package/sources.toml` (new, v1 format)
- `knowledge-package/concepts/*.md`, `knowledge-package/objectives/*.md`,
  `knowledge-package/examples/*.md` (migrated Calc II content — see plan Task 15's full
  file list for exact filenames)
- `src-tauri/src/knowledge/tests/migration.rs` (new — permanent regression test proving the
  real migrated package loads and its content is intact, not just structurally valid)
- `src-tauri/src/knowledge/tests/mod.rs` (register the migration test module)
- `ARCHITECTURE.md`, `knowledge-package/synthesis-report.md` (plan Task 17 — documentation
  made stale by adopting v1; no unrelated rewrites)

## Worklog

- 2026-08-30 (codex, plan Task 15): Migrated the Calc II package to the v1 TOML+Markdown
  layout: `package.toml`, `sources.toml`, 3 Concepts, 6 Objectives, and 6 concrete Examples
  (17 new files), then removed the exact pre-v1 manifests/JSONs and `problem-families/`
  tree listed by the plan. The first throwaway loader check caught a content-format issue:
  five prescribed Markdown hint bullets were physically wrapped across continuation lines,
  but the frozen parser's grammar requires every hint line to begin `- `. Flattened those
  five line wraps without changing wording; no runtime code was touched. The deleted local
  throwaway test then loaded `Path::new("../knowledge-package")` successfully and printed
  `concepts=3 objectives=6 examples=6 sources=1`. Task 16+ and `src-tauri/src/knowledge/`
  remain untouched. The plan's prose says "16 new files" while its explicit list contains
  17; followed the explicit list.
- 2026-08-30 (claude-code): Created this task, splitting plan Tasks 15–17 out of
  [050](050-knowledge-package-v1-implementation.md) per that plan's own process note at the
  end of its Task 14 section, now that 050's runtime workstream (Tasks 1–14) is reviewed
  and accepted.
- 2026-08-30 (claude-code): Claimed for codex; authorized plan Task 15 (migrate
  `knowledge-package/` to v1 format) only. Confirmed the on-disk pre-v1 files match Task
  15 Step 5's `git rm` list exactly (`package.json`, `provenance.json`, three concept
  JSONs, six objective JSONs, `problem-families/`) — no drift between the plan and the
  actual repo state to flag before issuing.

## What was built / tested / left out

Filled in when moving to `review`.

## Review

Filled in by the reviewing agent — must be a different agent than whoever implements this
task's plan Tasks, per `CLAUDE.md`'s coordination rules and this task's own Scope section.

## Follow-ups

Anything noticed during implementation or review that's out of this task's scope.
