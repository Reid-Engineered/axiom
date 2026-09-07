---
id: 061
title: practice::store test fails on Windows CI — open SQLite handle across remove_dir_all
status: proposed
owner: unassigned
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

## What was built / tested / left out

## Review

## Follow-ups
