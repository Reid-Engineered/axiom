---
id: 059
title: First production ProblemFamily in the bundled knowledge package
status: done
owner: claude
stage: 8
depends_on: [054, 055, 056, 057, 058]
---

## Scope

Add the first real `ProblemFamily` to the bundled `knowledge-package/` —
`problems/problem.shell_y_poly.md` — so that `practice.generate` can produce a problem from
production content rather than only from the canonical test fixture. This closes task 058's
stated follow-up ("the real knowledge package currently has zero canonical ProblemFamily
entries") and unblocks Study Session integration.

Does **not** build: the other five problem families implied by the synthesis report's
historical family list. `crate::generation::generate_problem_instance` dispatches only on
`gen.shell_y_poly`; every other family would need a new generator, which is a separate
design + content task, not a content edit. Also not in scope: any Study Session UI, any
change to the Problem schema, the generator, `math.verify`, or the Practice capability
contract — this task adds content and the tests that prove the content works through the
already-built machinery.

## Plan

Files to be created or touched:

- `knowledge-package/problems/problem.shell_y_poly.md` (new — the family itself)
- `knowledge-package/package.toml` (version bump `0.2.0` → `0.3.0`)
- `knowledge-package/synthesis-report.md` (record the family; correct the now-stale
  "v1 has no parametric generation schema" claim, which tasks 054/056 superseded)
- `src-tauri/src/knowledge/tests/migration.rs` (bundled package exposes the family, and
  all of its references resolve)
- `src-tauri/src/generation/tests/mod.rs` (determinism + validity across many seeds against
  the *bundled* family, not the fixture)
- `src-tauri/src/commands/practice.rs` (tests: real bundled package through
  `generate_attempt_handler`, and a canonical answer accepted by `math.verify` through
  `evaluate_attempt_handler`)

No production Rust changes are expected. If any turn out to be needed, that is a signal the
content does not fit the built schema and belongs in a follow-up task, not here.

## Content verification (done before authoring, recorded here as the durable record)

The family generalizes OpenStax *Calculus Volume 2* §2.3 Example 2.13. Region `R` is
bounded above by `f(x) = c·x − x²` and below by the x-axis over `[0, b]`, revolved about
the y-axis.

1. **Geometric premise.** `f(x) = x(c − x) ≥ 0` exactly on `[0, c]`. The declared bound
   `b.max = { parameter = "coeff" }` forces `b ≤ c`, so `[0, b] ⊆ [0, c]` and `f ≥ 0`
   throughout the interval — "bounded above by `f`, below by the x-axis" is true for every
   sampled instance. `b.min = 1` keeps the interval non-degenerate.
2. **Formula.** Rule 2.6 gives `V = ∫ₐᵇ 2π x f(x) dx` for a region revolved about the
   y-axis with `a ≥ 0`; `a = 0` is fixed, so the rule applies.
   `V = ∫₀ᵇ 2πx(cx − x²) dx = 2π∫₀ᵇ (c x² − x³) dx = 2π[c x³/3 − x⁴/4]₀ᵇ`
   `= 2π(c b³/3 − b⁴/4)` — exactly `canonical_solution.expression`,
   `"2*pi*(coeff*b^3/3 - b^4/4)"`.
3. **Positivity.** `V = 2π b³ (c/3 − b/4)`; with `b ≤ c` and `b ≥ 1`,
   `c/3 − b/4 ≥ b/3 − b/4 = b/12 > 0`, so every instance has a strictly positive volume.
4. **Parameter space.** `coeff ∈ {2..6}`, `a = 0`, `b ∈ {1..coeff}` — 20 distinct
   `(coeff, b)` pairs, all valid by (1)–(3). Small enough that the whole space is covered
   many times over by the 10 000-seed property test.
5. **Textbook cross-check.** Two published OpenStax answers fall inside the family's own
   parameter space and are reproduced by the formula:
   - `(c, b) = (2, 2)` — Example 2.13, `f(x) = 2x − x²` on `[0, 2]`:
     `2π(8·2/3 − 16/4) = 2π(16/3 − 4) = 8π/3`. Matches the text's answer.
   - `(c, b) = (3, 3)` — Checkpoint 2.13, `f(x) = 3x − x²` on `[0, 3]`:
     `2π(27 − 81/4) = 2π(27/4) = 27π/2`. Matches the checkpoint's answer.
6. **Hints.** Four strictly-increasing levels: (1) name radius and height, (2) give
   `r(x) = x`, `h(x) = {coeff}x − x²`, (3) give the assembled definite integral, (4) give
   the antiderivative to evaluate. Level 4 still stops short of stating the final value, so
   the hint ladder never hands over the answer that `practice.evaluate` is checking.
7. **Provenance.** `direct` → §2.3 Rule 2.6 (the volume formula is transcribed from the
   rule); `derived` → §2.3 Example 2.13 (the parametric family generalizes that example's
   single instance). This matches the `direct`/`derived` semantics the synthesis report
   already defines ("`direct` transcribes an OpenStax example's own numbers, `derived` is a
   freshly authored instance within that example's method"), and is consistent with the
   existing citations on `shell.method_vertical_axis` and
   `shell.compute_volume_y_axis_single_curve`.

**Limit on (5) and (7):** the OpenStax PDF is not in this repo (the synthesis report records
this — "the package's source text is unavailable"). The section/rule/example labels are
verified against the package's own recorded source facts (the synthesis report plus the
citations already carried by the concept and objective this family attaches to), and the
*mathematics* of both cited instances is verified independently from the formula above. A
human with the PDF should still eyeball the two labels; that is exactly the kind of check
`.ai/quality-gates.md` calls out as non-mechanical.

## Worklog

- 2026-09-05 — started, claimed by claude. Read `.ai/tasks/_archive/054-058`, the Problem
  schema (`knowledge/problem_family.rs`, `problem_family_body.rs`, `validate.rs`), the
  generator (`generation/`), `math.verify`, and the Practice command layer before authoring
  anything. No other task is `in-progress` or `review`, so no file-overlap coordination was
  needed.
- 2026-09-05 — content verification above completed before writing the family file.
- 2026-09-05 — authored `knowledge-package/problems/problem.shell_y_poly.md` and bumped
  `package.toml` to `0.3.0`. Confirmed the package still loads
  (`cargo test --lib knowledge::tests::migration`) before writing any further tests.
- 2026-09-05 — added the five tests. Confirmed **red**: with `knowledge-package/problems/`
  temporarily moved aside, exactly the five new tests fail and all 279 pre-existing tests
  still pass; restored, all 284 pass.
- 2026-09-05 — two mutation checks, to prove the tests assert the content and not just its
  presence. (a) Canonical solution changed to `... - b^4/3`: the two math-checking tests and
  the migration test fail, the two structural tests still pass. (b) `b.max` widened to
  `{ parameter = "coeff", offset = 2 }` (breaking `b <= c`, so the region's height goes
  negative): the 10 000-seed validity test and the migration test fail. Content restored
  after each.
- 2026-09-05 — no production Rust change was needed, as the plan predicted. The one
  non-test source edit is `ARCHITECTURE.md`: its Practice paragraph asserted "the real
  package currently contains worked examples but no canonical ProblemFamily entries", which
  this task makes false.
- 2026-09-05 — gates run. Backend: `cargo fmt --check`, `cargo check`, `cargo clippy
  --all-targets -- -D warnings`, `cargo test` (284 passed). Frontend: `npm run typecheck`,
  `lint`, `build`, `test` (59 files / 149 tests) — unchanged from task 058's baseline, as
  expected for a task that touches no `src/` file. Design-token grep clean.
  `cargo check` initially failed on a stale `/var/tmp/axiom-058-target` path baked into the
  cached Tauri build script (a leftover of task 058's mid-task cache move, not a code
  problem); `cargo clean -p tauri -p axiom` cleared it.
- 2026-09-05 — native gates. `tauri build --no-bundle` succeeded and `diff -qr` confirms
  `target/release/knowledge-package` matches the source package byte for byte, including the
  new `problems/` directory. Both `e2e/*.test.mjs` flows pass against that binary — which by
  itself proves the bundle loads, since `lib.rs` `.expect()`s `load_knowledge_package` during
  setup, so a broken bundle panics before the window appears.
- 2026-09-05 — added a temporary native IPC smoke (kept in the session scratchpad, not
  committed) that drives the release binary through `createWorkspace` → `generateAttempt` →
  `requestHint` → `evaluateAttempt` for `problem.shell_y_poly`, recovering the sampled
  parameters from the rendered prompt and computing the answer in the test rather than
  reusing the generator's canonical string. Five runs, five distinct instances, all pass:
  `(c,b)` = (4,4), (5,3), (2,2), (3,2), (6,5). The (2,2) run returned V = 8.37758… = 8π/3,
  i.e. the shipping binary reproduced OpenStax Example 2.13's published answer.
- 2026-09-05 — status moved `in-progress` → `review` after all gates passed, then reviewed
  and archived at the user's request (see `## Review` for the independence caveat).

## What was built / tested / left out

**Built.** `knowledge-package/problems/problem.shell_y_poly.md` — the bundled package's
first canonical `ProblemFamily`, adapted from the canonical test fixture
(`src-tauri/src/knowledge/tests/fixtures/canonical/problems/problem.shell_y_poly.md`) onto
the real concept `shell.method_vertical_axis` and the real objectives
`shell.setup_radius_height_y_axis` + `shell.compute_volume_y_axis_single_curve` (the fixture
cited a fixture-only objective id, `shell.setup_radius_height`). Parameters, generator ref,
response type, canonical solution and the four hint levels carry over from the fixture
unchanged in substance. Three deliberate deviations from the fixture, all noted here rather
than made silently:

1. **Two objectives instead of one.** The problem exercises both the setup subskill (hint
   levels 1–2) and the evaluation subskill (levels 3–4), and both objectives hang off the
   family's own concept, which is what `validate.rs` requires.
2. **A second `derived` provenance ref** to Example 2.13, alongside the fixture's single
   `direct` ref to Rule 2.6 — a parametric family generalizing an example is exactly the
   `derived` case the synthesis report defines, and the fixture's single-`direct` citation
   would have overclaimed.
3. **The `## Solution` section is written without `{...}` placeholders.** The fixture's
   solution text contains a `{coeff}` token, but `generate_problem_instance` substitutes
   only the prompt and the hints — `solution_structure` is never templated, so that token
   could never be filled in. Nothing renders `solution_structure` today, so this is a latent
   bug fixed at the source rather than an observed one.

Also: `package.toml` `0.2.0` → `0.3.0`; `synthesis-report.md` gains a "Canonical problem
families (`problems/`)" section recording the family, its derivation, its OpenStax basis and
its verification, marks the old "Problem families" section as partly superseded (it still
claimed v1 has no parametric-generation schema, which task 054 changed), and closes out the
report's "Tolerance Policy for Exact Expressions" review item now that `math.verify@1`'s
behavior is known; `ARCHITECTURE.md`'s stale "no canonical ProblemFamily entries" sentence
corrected.

**Tested.** Five new Rust tests, 284 total (279 before):

- `knowledge::tests::migration::bundled_package_exposes_the_verified_shell_y_poly_problem_family`
  — the bundled package loads (which is itself the full §12/§13 validation pass: references
  resolve, objectives belong to the family's concept, provenance sources exist) and the
  family's declared shape is pinned field by field, including *which* production entities it
  attached to and the direct/derived provenance split.
- `generation::tests::bundled_shell_y_poly_is_deterministic_for_every_seed` — 2 000 seeds,
  each generated twice and compared. This is the property the attempt store depends on: an
  attempt persists only its seed and is replayed through the generator on every load.
- `generation::tests::bundled_shell_y_poly_instances_are_valid_across_ten_thousand_seeds` —
  per instance: parameters inside their declared bounds, `a = 0`, `b <= coeff`, shell height
  non-negative at 51 points across `[0, b]`, four hints, no surviving `{...}` placeholder in
  prompt or hints, and a canonical solution that `mathcore` evaluates to the hand-derived
  closed form and that is finite and strictly positive. The closing assertion is that all
  20 reachable `(coeff, b)` pairs were actually produced — so this is exhaustive over the
  family's content, not a spot check.
- `commands::practice::tests::generate_attempt_succeeds_against_the_real_bundled_knowledge_package`
  — the real `generateAttempt` command handler, built against the real bundled package
  rather than the canonical fixture, returns a substituted prompt, `symbolic-expression`,
  four hints, and a servable first hint.
- `commands::practice::tests::bundled_shell_y_poly_answers_are_accepted_by_math_verify_through_the_command_layer`
  — over five seeds: a wrong answer (off by 1) is rejected and leaves the attempt `open`; the
  independently computed volume is accepted and solves it; and an algebraically different
  exact form, `pi*(2*c*b^3/3 - b^4/2)`, is accepted on a second attempt at the same instance,
  showing verification is by value rather than by string match. All of it through the real
  command handlers and the real `math.verify` provider resolved from the registry.

Gates run (this task touches `knowledge-package/`, `src-tauri/` and docs; no `src/` file):
`cargo fmt --check`, `cargo check`, `cargo clippy --all-targets -- -D warnings`,
`cargo test` (284 passed) — plus the frontend set (`typecheck`/`lint`/`build`/`test`,
59 files / 149 tests) run anyway to confirm nothing moved, and `npm run test:e2e:linux`'s
two native flows. The design-token grep is clean (no CSS touched). Non-mechanical gates:
`ARCHITECTURE.md` was updated for the factual claim this task invalidated; there is no
visual surface to check against the mockups.

**Left out.** The other five problem families from the synthesis report's historical list.
`crate::generation::generate_problem_instance` dispatches on `gen.shell_y_poly` only; each
remaining family needs a generator designed, implemented and tested before its content file
can do anything, which is a design task rather than a content edit. Also left out, and
unchanged from task 058's deferrals: Study Session UI consumption, a `seed` parameter on the
`generateAttempt` command, and per-workspace module enablement. No production Rust, no
dependency, no schema, and no capability contract changed.

**One honest limit.** The OpenStax PDF is not in this repo. The `Rule 2.6` / `Example 2.13`
*labels* are carried from the package's own earlier source review (the synthesis report and
the citations already on the concept and objective this family attaches to), not re-read from
the text. What is verified independently here is the mathematics: the closed form is derived
from first principles above, and both cited instances' published answers — 8π/3 for
Example 2.13 and 27π/2 for Checkpoint 2.13 — fall out of it at `(c,b) = (2,2)` and `(3,3)`,
which would be a surprising coincidence if the labels were wrong. A human with the PDF should
still confirm the two labels; this is recorded as review item 4 in the synthesis report.

## Review

Reviewer: claude — **author self-review, not an independent one.** `.ai/lifecycle.md` asks
for a different agent or the human; the user asked for this task to be reviewed and archived
in the same pass, so this is recorded as what it is. The findings below are worth no more
than a self-check, and the one item most in need of another pair of eyes is the provenance
labels (see the limit above).

Date: 2026-09-05

- [x] **Correctness — pass.** The code matches "what was built": the diff is one new content
      file, a version bump, two docs updates, and three test files. The tests were confirmed
      red before the content existed and green after, and two mutation checks (a wrong
      canonical solution; a widened `b` bound that breaks the region's geometry) each fail
      the tests that should catch them — so they assert the content, not merely its
      presence. Edge cases beyond the happy path are covered: a wrong answer rejected, an
      algebraically-different-but-equivalent form accepted, the endpoints `b = 1` and
      `b = coeff` reached (the exhaustive-pair assertion forces both), and the degenerate
      risk `b > coeff` excluded by construction rather than by rejection sampling.
- [x] **Architecture conformance — pass.** No `src/` file touched, so the hooks/services/
      pages rules aren't exercised. No new shared type, no new global state, no new
      directory in `src/` or `src-tauri/src/`. `knowledge-package/problems/` is not a new
      structural pattern — the loader has scanned for it since task 054 and the canonical
      fixture already used it; this is the first *production* file in it.
      `ARCHITECTURE.md` was updated, though only to correct a claim this task falsified.
- [x] **UI rules — N/A.** No markup, no CSS, no design tokens, no copy that reaches a
      screen through a component. The prompt and hint copy do follow the handoff's rules
      (no exclamation marks, no emoji).
- [x] **Process — pass.** Every applicable gate in `.ai/quality-gates.md` was run and is
      listed above, including the native E2E gate, which for this task is not ceremony: the
      family only helps if it survives Tauri resource bundling, and `lib.rs` panics at
      startup if it doesn't. Scope matches what the task was created for; the three
      deviations from the reference fixture are named rather than folded in silently. The
      one thing a reviewer should not take on trust from this section is the section itself
      — it was written by the author.

Verdict: approved (self-review; independent review not performed)

## Follow-ups

- **Generators for the remaining five families.** Content files are cheap; the generators
  are not. Each of `shell.example_y_reciprocal`, `shell.example_y_between_curves`,
  `shell.example_shifted_vertical_axis`, `shell.example_x_axis` and
  `shell.example_setup_integrand` would need its own generator in
  `crate::generation` before a `problems/` file for it does anything. Worth designing as one
  task that asks whether `generate_generic` can already serve most of them — it is
  generator-agnostic apart from the dispatch `match`, so several of these may need only a
  new arm and new content, not new sampling code.
- **Nothing renders `ProblemFamily::solution_structure`.** It is parsed, validated and
  stored, then unused; `generate_problem_instance` doesn't even template it. Either a
  consumer (Study Session's worked-solution view) or an explicit note that it is
  authoring-time documentation only.
- **`difficulty` is inert.** Declared per family, consumed by nothing. The synthesis
  report's "Difficulty Scale Alignment" review item stays open until Practice actually
  selects on it.
- **Confirm the OpenStax labels against the PDF** — recorded as review item 4 in
  `knowledge-package/synthesis-report.md`.
