# Practice regression corpus

A permanent, case-named, never-deleted record of what Practice actually serves a learner
(task 067). Each `<family>/seed-<n>.json` is the full generated problem for one seed: resolved
parameters, prompt text, every hint in order, response type and canonical solution.

The property tests in `../mod.rs` prove the generator is deterministic and in-bounds. They
can't see a change to *what* it generates, as long as the new output is also deterministic
and in-bounds. This corpus catches that kind of change.

Runner: `../corpus.rs`. End-to-end check through Practice and `math.verify`:
`regression_corpus_cases_round_trip_through_practice_and_math_verify` in
`src/practice/tests/mod.rs`.

## Rules

1. **Never delete a case**, even after the bug it caught is fixed, and even if it looks
   redundant. The seed list (`SHELL_Y_POLY_SEEDS`) is append-only, and a test fails if the
   files and the list disagree.
2. **A red corpus test means generated content changed.** Don't regenerate to make it green.
   First decide whether the change was intended.
3. **Regenerating is its own reviewable commit.** If the change was deliberate (new prompt
   wording, a reworded hint, a generator fix), regenerate in a commit that contains only the
   corpus diff and says why the content changed. Never fold a regeneration into an unrelated
   commit.

## Regenerating

From `src-tauri/`:

```bash
AXIOM_REGENERATE_CORPUS=1 cargo test regenerate_shell_y_poly_corpus -- --ignored
```

PowerShell:

```powershell
$env:AXIOM_REGENERATE_CORPUS = "1"; cargo test regenerate_shell_y_poly_corpus '--' --ignored
```

The test is `#[ignore]`d and also refuses to run without the environment variable, so
`cargo test -- --ignored` can't overwrite the corpus by accident. Review the diff before you
commit it.

## Adding a case

Append the seed to `SHELL_Y_POLY_SEEDS` in `../corpus.rs`, say why in its doc comment,
regenerate, and check that the only *new* file is the new case and no existing file changed.

## Why these seeds

- `0, 1, 3, 4, 6, 7, 10, 11, 12, 15, 17, 20, 22, 26, 27, 37, 39, 46, 49, 100`: the lowest seed
  reaching each of `problem.shell_y_poly`'s 20 reachable `(coeff, b)` pairs (`coeff` in 2..=6,
  `b` in 1..=coeff). A test enforces that every pair stays covered.
- `42`: the seed the command-layer and registry tests already use.
- `18446744073709551615` (`u64::MAX`): the far edge of the seed space.

## Shape

Committed JSON files, one per seed, not Rust mutation functions like the Knowledge Package
conformance corpus. A `ProblemInstance` is one serializable value, and a JSON diff shows
exactly which field of a problem changed, which is what a reviewer needs to judge a
regeneration.
