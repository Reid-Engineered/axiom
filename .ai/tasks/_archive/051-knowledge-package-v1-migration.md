---
id: 051
title: Knowledge Package v1 — migrate knowledge-package/ (Calc II) content
status: done
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

- 2026-08-30 (claude-code, plan Task 17): Implemented this one myself rather than issuing
  it to Codex — per `CLAUDE.md`, "anything touching `ARCHITECTURE.md`" is Claude's specific
  responsibility in the agent rotation, and Task 17 modifies `ARCHITECTURE.md` directly.
  **Step 1**: replaced the `knowledge-package/` entry's stale "ad hoc until Knowledge
  Package v1" comment with a two-line comment pointing at the v1 spec, preserving the
  block's column-29 comment-alignment convention (verified with `index($0,"#")` across the
  edited and neighboring lines, not eyeballed).
  **Step 2**: rewrote `synthesis-report.md` per all five prescribed bullets — Concepts/
  Objectives/Problem-families ID references updated to Task 15's actual grammar-compliant
  IDs; added a "Provenance Collapse to a Single Source" entry under "Structural inferences"
  documenting the 11→1 Source collapse and what `direct` vs `derived` now means, spot-
  verified against every migrated Example's actual `provenance_refs[].kind` (not asserted
  from memory); reworded the three other "Structural inferences" entries that described
  now-dropped generation/templating machinery as explicitly historical rather than current;
  moved `pf-method-select-integral-count` out of "Problem families" into "Rejected
  candidates" with its exclusion reason and a pointer to `.ai/tasks/_archive/
  049-knowledge-package-shell-fixes.md`; left "Human review priorities" untouched (no item
  resolved by this migration). Per the plan's own review note for this step, independently
  verified every backtick-quoted Concept/Objective/Example ID in the rewritten report
  against `id = "..."` in the actual migrated files — exact match, no missing/stray IDs.
  This is prose; no code gate applies, and no `src-tauri/` file was touched.
  **This closes the entire 17-task Knowledge Package v1 plan.**
- 2026-08-30 (claude-code, plan Task 16 review): Reviewed the Task 16 implementation
  (commit `faedc45`) against plan Task 16 Steps 1–4. Diffed `migration.rs` against the
  plan's prescribed test module — matches exactly (only rustfmt line-wrapping differs).
  `mod migration;` is registered in `tests/mod.rs`. Re-ran the gates: `cargo test --locked
  knowledge::tests::migration -- --nocapture` passes 2/2, `cargo test --locked` (full
  workspace) passes 145/145, `cargo clippy --all-targets --locked -- -D warnings` clean,
  `cargo fmt --all --check` clean. **Task 16 is accepted.** Only plan Task 17 (docs
  cleanup: `ARCHITECTURE.md`, `knowledge-package/synthesis-report.md`) remains — this
  closes the entire 17-task Knowledge Package v1 plan once accepted.
- 2026-08-30 (codex, plan Task 16): Added the permanent migration regression module with
  the prescribed structural-completeness and deprecated-artifact tests, and registered it
  alongside the canonical/conformance suites. Focused migration tests passed 2/2 with 143
  filtered out. The final workspace gate `cargo test --locked` passed 145/145 with 40
  filtered out (including the two migration tests), `cargo clippy --all-targets --locked
  -- -D warnings` passed cleanly, and `cargo fmt --all --check` passed. No runtime files or
  Task 17 documentation were changed; only the permanent tests and this Worklog were added.
- 2026-08-30 (claude-code, plan Task 15 review): Reviewed the Task 15 implementation
  (commit `17d8aa3`) against plan Task 15 Steps 1–7. Independently reproduced the sanity
  check with a throwaway integration test calling `load_knowledge_package` on the migrated
  `../knowledge-package` — got the identical counts Codex reported
  (`concepts=3 objectives=6 examples=6 sources=1`), then deleted the test. Diffed
  `package.toml`/`sources.toml` and spot-checked `shell.method_vertical_axis.md` and
  `shell.example_y_poly.md` against the plan's Step 1/2/4 content byte-for-byte — matches
  exactly (except that plan's own Markdown source line-wraps some hint bullets across two
  physical lines for readability; the frozen Task 7 grammar requires one physical line per
  hint, so Codex correctly un-wrapped rather than transcribing the wrap literally — every
  hint line in every migrated Example now starts `- ` with no continuation). Confirmed the
  pre-v1 removal list (Step 5) was applied in full — no `package.json`, `provenance.json`,
  old-naming JSON files, or `problem-families/` remain on disk. Re-ran the gates: `cargo
  test --locked knowledge::` 103/103 (no new tests committed yet — Task 16 adds the
  permanent version), `cargo clippy --all-targets --locked -- -D warnings` clean, `cargo
  fmt --all --check` clean, worktree clean. **Task 15 is accepted.** Plan Task 16
  (migration validation) is authorized next; pre-checked it — its prescribed assertions
  (id, schema_version, 1 source, 3/6/6 entity counts, no `pf-` prefixed example ids) match
  what's actually on disk, and its Step 2 registers `mod migration;` before Step 2's own
  test run (no repeat of the Task 13/14-class ordering defect).
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

**Built:** the real `knowledge-package/` (Calc II cylindrical shells) migrated in full to
Knowledge Package v1 — `package.toml`, `sources.toml`, 3 Concepts, 6 Objectives, 6
Examples, all pre-v1 JSON/`problem-families/` removed; a permanent committed regression
test (`migration.rs`) proving the real package loads and is structurally complete; and
`ARCHITECTURE.md`/`synthesis-report.md` updated to match.

**Tested:** the migrated package loads through `load_knowledge_package` with 0 validation
failures (verified twice independently — once via a throwaway integration test during
Task 15 review, once as the committed `migration.rs` in Task 16); full workspace suite
145/145 passing; clippy and fmt clean throughout. Every ID in the rewritten
`synthesis-report.md` was checked against the actual on-disk `id = "..."` values.

**Left out, deliberately:** `pf-method-select-integral-count` did not migrate (spec §19,
unresolved source contradiction — see "Rejected candidates" in `synthesis-report.md`).
Right-shifted vertical axes remain unrepresented (noted in "Unresolved gaps"). The
disk/washer cross-package prerequisite gap is now structurally unrepresentable in v1's
schema, not merely undeclared — also noted there, with no fix attempted (would need a
disk/washer package or a future cross-package reference mechanism, neither in scope).

## Review

Filled in by the reviewing agent — must be a different agent than whoever implements this
task's plan Tasks, per `CLAUDE.md`'s coordination rules and this task's own Scope section.
One exception, noted rather than silently taken: plan Task 17 modifies `ARCHITECTURE.md`
directly, which `CLAUDE.md` assigns specifically to Claude in the agent rotation regardless
of who implements the surrounding task — so Task 17 was implemented and self-verified by
claude-code (the same agent reviewing Tasks 15–16 here). Mitigations: Task 17 is prose-only
(no `src-tauri/` file touched, no code gate applies per the plan itself), and the plan's own
review instruction for this step — confirm every ID in the rewritten report exists on disk
— was independently run as a `comm`-based set-diff, not eyeballed, with an empty result.
A different reviewer (or the human) should still sanity-read `synthesis-report.md`'s prose
before merge, since prose accuracy beyond ID existence isn't something a diff can verify.

### Plan Tasks 1–14 (050)

Reviewed and accepted in full under
[050](050-knowledge-package-v1-implementation.md) — see that task's own `## Review`
section. Not repeated here.

### Plan Task 15 — Migrate `knowledge-package/` to v1 (commit `17d8aa3`)

See the Task 15 review worklog entry above (2026-08-30, claude-code). **Verdict: accepted.**

### Plan Task 16 — Migration validation (commit `faedc45`)

See the Task 16 review worklog entry above (2026-08-30, claude-code). **Verdict:
accepted.**

### Plan Task 17 — Documentation cleanup (uncommitted at review time)

Implemented and self-verified by claude-code, per the self-review note above. **Verdict:
accepted, pending a second reader's prose sanity-check before merge** (see caveat above).

---

**Task-level verdict: 051 — and with it, the entire 17-task Knowledge Package v1 plan — is
complete.** Status set to `review`, pending human merge decision.

## Follow-ups

Anything noticed during implementation or review that's out of this task's scope.
