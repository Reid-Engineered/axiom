---
id: 061
title: practice::store test fails on Windows CI — open SQLite handle across remove_dir_all
status: review
owner: claude
stage: 8
depends_on: []
---

## Scope

Fix `practice::store::tests::attempt_surviving_a_fresh_connection_to_the_same_file_reads_back_identically`
so `backend-checks (windows-latest)` passes. The test holds an open SQLite connection while
deleting the directory that contains the database file; Windows refuses the delete, so the
test fails deterministically on Windows and passes everywhere else.

Does **not** build: any change to `PracticeStore`'s production behaviour, to `crate::db::open`,
or to the other three `std::env::temp_dir()` test helpers — those hold no open file handle
across their cleanup, so they are not affected by this bug. Introducing a `tempfile`
dev-dependency to make cleanup RAII-based across all four is a reasonable option to consider
while here, but adding a dependency needs its own justification in the worklog, not a silent
slide.

## Evidence

`src-tauri/src/practice/store.rs:321-347`. The first `PracticeStore` is correctly scoped in a
block so its connection drops, but `reopened` is still alive when the cleanup runs:

```rust
let reopened = PracticeStore::new(crate::db::open(&db_path).unwrap());
let row = reopened.load_attempt("attempt-1", "ws-1").unwrap();
assert_eq!(row.instance, sample_instance());

std::fs::remove_dir_all(&dir).unwrap();   // store.rs:345 — panics on Windows
```

CI (run `33991157438`, job `101373518900`, PR #4):

```
thread '...attempt_surviving_a_fresh_connection_to_the_same_file_reads_back_identically'
panicked at src\practice\store.rs:345:39:
called `Result::unwrap()` on an `Err` value: Os { code: 32, kind: Uncategorized,
message: "The process cannot access the file because it is being used by another process." }
test result: FAILED. 283 passed; 1 failed
```

The same job also failed on the master run `33989904129`, so this is pre-existing from task
057 and not introduced by PR #4. It is the reason task 059's and PR #4's "284 Rust tests
passed" is true on Linux and macOS only.

## Plan

Files to be touched:

- `src-tauri/src/practice/store.rs` — scope `reopened` in its own block (or `drop(reopened);`)
  before `remove_dir_all`. While here, consider making the cleanup panic-safe: as written, a
  failed assertion leaks the temp directory, because the removal never runs.

## Worklog

- 2026-09-05 — created as a follow-up from task 060's review of PR #4.
- 2026-09-07 — claimed by claude; `proposed` → `in-progress`.
- 2026-09-07 — fix applied, gates run, `in-progress` → `review`.

## What was built / tested / left out

`src-tauri/src/practice/store.rs` — `reopened` is now scoped so its `Connection` drops
before `remove_dir_all`, and the assertion moved after cleanup so a failing assert no longer
leaks the temp directory. `crate::db::open` sets `PRAGMA journal_mode = WAL`
(`src-tauri/src/db/mod.rs:30`), so the open handle held `axiom.sqlite3-wal` and
`axiom.sqlite3-shm` open alongside the database file — dropping the connection closes all
three. The test now binds only the `ProblemInstance` out of the scope, which is all the
assertion needs.

Gates run locally on Linux: `cargo test` 284 passed / 0 failed, `cargo fmt --check` clean,
`cargo clippy --all-targets -- -D warnings` clean.

**Not verified locally, and this is the point of the task:** the bug only reproduces on
Windows, and this machine is Linux. A green Linux run says the fix is not a regression, not
that it works. `backend-checks (windows-latest)` on a PR is the only real proof, and this
task should not move past `review` until that check is green.

Left out: the three other `std::env::temp_dir()` test helpers
(`knowledge/loader.rs:47`, `knowledge/discover.rs:127`, `knowledge/tests/mod.rs:19`). Each was
checked — none holds an open file handle across its cleanup, so none has this bug. A
`tempfile` dev-dependency for RAII cleanup across all four was considered and not taken: it
would add a dependency to fix three call sites that are not broken.

## Review

## Follow-ups

- None of the four temp-dir test helpers is panic-safe if an `unwrap()` before the cleanup
  line fails; each leaks a directory under the system temp dir. Not worth a dependency on its
  own, but worth folding into any future task that touches this test scaffolding.
