# CI Pipeline — design

Sub-project 1 of 3 in the "professional CI/CD structure" initiative Marcus scoped on
2026-08-31. The other two — **agent orchestration** (eliminating manual copy-paste of
resumption prompts between Claude/Codex/Antigravity) and **CD** (automated Tauri release
artifacts) — are explicitly out of scope here and get their own brainstorm-and-spec pass.
This project exists first because orchestration needs a mechanical source of truth for "did
this pass," which doesn't exist yet, and CD needs green gates before it packages anything.

Source material: `.ai/quality-gates.md`, `.ai/merge-strategy.md`, `.ai/lifecycle.md`,
`AGENTS.md` (Testing section, Git workflow section), `e2e/README.md`, `package.json`.

---

## 1. Scope

Stand up GitHub Actions CI that mechanically enforces the gates `.ai/quality-gates.md`
currently only asks agents to self-report, and adopt the PR-based branch workflow
`.ai/merge-strategy.md` already documents but the repo doesn't yet practice (current
practice: agents commit directly to `master`; no PRs, no `gh` CLI installed, no branch
protection).

Explicitly out of scope:
- Agent orchestration (auto-dispatching Claude/Codex/Antigravity) — separate spec.
- CD / release artifact builds — separate spec, revisit once Stage 7 ships something.
- Extending e2e coverage to macOS/Windows — no driver setup for either exists anywhere in
  this repo today (`e2e/README.md` only documents Linux); this is real infrastructure work
  in its own right, tracked as a follow-up task once this spec's Linux e2e job is proven.
  This is a documentation/tooling gap, not a cost one, so it applies regardless of repo
  visibility.
- Mechanizing the `ARCHITECTURE.md`-updated-on-structural-change gate and the
  visual/pixel-diff gate — both stay human judgment calls, per `.ai/quality-gates.md`'s own
  explicit exclusion of the latter.

## 2. Decisions carried in from brainstorming

- **Adopt the documented PR/branch flow, not the current direct-to-master practice.** Agents
  push to `agent/<tool>/<task-id>-<slug>` branches and open PRs; CI gates the merge. This
  changes agent behavior (this task installs and starts using `gh` CLI), not just adds a
  workflow file — call this out explicitly to whichever agent implements it.
- **e2e is in scope for this pass, Linux-only.** `frontend-checks` and `backend-checks` run
  on a Linux/macOS/Windows matrix; `e2e` runs on `ubuntu-latest` only, per the driver-support
  gap above.
- **GitHub Actions, hosted runners** (not self-hosted). Simplest, standard, no infrastructure
  to maintain. `Reid-Engineered/axiom` is now a **public** repo (as of 2026-08-31), and
  GitHub Actions on standard hosted runners is free and unmetered for public repos across
  Linux, macOS, and Windows alike — the OS-weighted minute cost that would apply on a private
  repo (macOS 10x Linux) is a non-issue here, which is why the full 3-OS matrix is back in
  scope.

## 3. Trigger model & branch protection

`.github/workflows/ci.yml` triggers on:
- `pull_request` targeting `master` (any branch)
- `push` to `master`

`master` gets a branch protection rule requiring every job in §4 to pass before merge. This
is the actual enforcement mechanism the current honor-system gate process lacks — nothing
today stops a broken build from landing.

`push` to `master` staying in the trigger list (not just `pull_request`) preserves
`.ai/merge-strategy.md`'s existing "main is always buildable" contract: if a broken commit
ever lands anyway (e.g. an admin bypasses protection), the same workflow catches it on
`master` immediately, and the next action is the revert that doc already prescribes —
unchanged by this spec.

## 4. Jobs

One workflow, parallel jobs:

### `frontend-checks`
Matrix: `[ubuntu-latest, macos-latest, windows-latest]`.
1. `actions/checkout`, `actions/setup-node` (cached).
2. `npm ci`
3. `npm run typecheck`
4. `npm run lint`
5. `npm run build`
6. `npm run test`
7. Design-token grep: fail if `grep -rE '#[0-9a-fA-F]{3,6}|rgba\(' src/` finds a hit outside
   `src/styles/tokens.css`. This mechanizes the currently-manual gate in
   `.ai/quality-gates.md` ("Grep for stray hex codes... as part of self-check").

### `backend-checks`
Matrix: `[ubuntu-latest, macos-latest, windows-latest]`. Working directory `src-tauri/`.
1. `actions/checkout`, `dtolnay/rust-toolchain` (stable), cache `~/.cargo` and
   `src-tauri/target` keyed on `Cargo.lock` + OS.
2. `cargo check`
3. `cargo test`
4. `cargo clippy --all-targets --locked -- -D warnings`
5. `cargo fmt --all --check`

### `e2e`
`ubuntu-latest` only.
1. `actions/checkout`, Node + Rust setup as above, cache as above.
2. `cargo install tauri-driver --version 2.0.6 --locked`
3. `sudo apt-get install -y webkitgtk-webdriver xvfb` (package name per `e2e/README.md`;
   confirm against whatever Ubuntu version the runner image ships — README notes the package
   is named `webkit2gtk-driver` on 22.04/24.04 vs `webkitgtk-webdriver` on 26.04).
4. `npm ci`
5. `npm run test:e2e:linux` — exactly the script `e2e/README.md` already specifies for CI use.

All three jobs (7 total runs: 3+3+1) are required status checks on `master`.

## 5. Gate mapping & task-file changes

| `.ai/quality-gates.md` gate | Where it's enforced now |
|---|---|
| `npm run typecheck` / `lint` / `build` | `frontend-checks`, mechanical |
| No hardcoded design values | `frontend-checks` grep step, mechanical |
| Component/hook render tests exist | `npm run test` (Vitest) catches missing coverage only insofar as tests are written; presence-of-test-for-new-component stays a human review-checklist item, not something CI can detect |
| `cargo check` / `cargo test` | `backend-checks`, mechanical |
| `npm run test:e2e:linux` for `src-tauri/`/`src/services/` changes | `e2e` job runs unconditionally on every PR now (simpler than path-filtering it to "only when those dirs changed"); this widens the gate from `quality-gates.md`'s scoped version to always-on, matching that doc's own stated intent once CI exists |
| `ARCHITECTURE.md` updated on structural change | **Stays manual** — CI can't judge this |
| Visual/pixel-diff vs. mockups | **Stays manual** — `quality-gates.md` already excludes this explicitly |

`.ai/quality-gates.md` gets a short edit: a note at the top that CI now runs the mechanical
gates automatically on every PR, with a pointer to the workflow file, and the e2e section's
"no CI provider wired up yet" caveat removed since it's no longer true.

`.ai/tasks/TEMPLATE.md`'s "What was built / tested / left out" section changes from
"state which gates were run" to "link the PR and its CI run" — CI's pass/fail record
replaces hand-typed gate status as the source of truth.

## 6. Failure handling

A failing required check blocks merge via branch protection — no custom bot/comment layer,
GitHub's native PR check UI is sufficient. No new tooling for this.

## 7. Validating the pipeline

After the workflow file and branch protection are live: open one real, small PR (a trivial
doc fix or converting an already-archived task into a test case) and confirm all 7 job runs
report correctly. Then deliberately introduce one failure (e.g. a lint violation) on that
same PR to confirm a red check actually blocks merge, before trusting the pipeline for real
task PRs.

## 8. Follow-ups (out of scope here, tracked for later)

- Extend e2e to macOS and Windows — needs their native WebDriver setup researched and
  documented first; `e2e/README.md` has no Mac/Windows section today.
- Agent orchestration spec (separate brainstorm) — depends on this pipeline existing as the
  source of truth for gate status.
- CD spec (separate brainstorm) — depends on this pipeline; revisit once Stage 7 ships.
- `gh` CLI needs installing wherever agents (including Claude Code sessions) need to open/
  manage PRs — not installed in this environment as of this spec.
