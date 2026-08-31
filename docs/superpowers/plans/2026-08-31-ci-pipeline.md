# CI Pipeline Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Stand up GitHub Actions CI that mechanically enforces `.ai/quality-gates.md`'s
checks on every PR and push to `master`, and adopt the PR-based branch workflow
`.ai/merge-strategy.md` already documents but the repo doesn't yet practice.

**Architecture:** One workflow file (`.github/workflows/ci.yml`) with three parallel jobs —
`frontend-checks` and `backend-checks` on a Linux/macOS/Windows matrix, `e2e` on Linux only.
Two `.ai/` docs get updated to point at CI instead of the honor-system self-check. There's no
traditional unit test for a CI config, so "TDD" here means: author the artifact, then prove
it behaves correctly against the real system (a real PR shows all checks run; a deliberately
broken commit on that PR proves a required check actually blocks merge) before trusting it.

**Tech Stack:** GitHub Actions, `gh` CLI, existing npm/cargo scripts (`package.json`,
`src-tauri/Cargo.toml`).

**Spec:** `docs/superpowers/specs/2026-08-31-ci-pipeline-design.md`

## Global Constraints

- Repo is `Reid-Engineered/axiom`, public, default branch `master`.
- Workflow triggers: `pull_request` targeting `master`, and `push` to `master` (spec §3).
- Node version: no `.nvmrc`/`engines` field exists in this repo; use Node 20 (current LTS,
  satisfies Vite 7 / React 19's minimums) — pin this exact version in every job that needs
  Node.
- Rust toolchain: `stable`, with `clippy` and `rustfmt` components.
- `dtolnay/rust-toolchain@stable` and `Swatinem/rust-cache@v2` are the actions used for Rust
  setup/caching — do not substitute a hand-rolled `actions/cache` config for Rust.
- Per `.ai/merge-strategy.md`: changes to anything in `.ai/` require explicit human sign-off
  before merge, regardless of which agent authored or reviewed them. This plan modifies
  `.ai/quality-gates.md` and `.ai/tasks/TEMPLATE.md` — **Marcus's explicit approval is
  required before Task 7's merge**, no exceptions.
- Task id for this work in `.ai/tasks/` is **052** (next available; highest existing is 051
  in `.ai/tasks/_archive/`).

---

### Task 1: Claim the task file and branch

**Files:**
- Create: `.ai/tasks/052-ci-pipeline.md`

**Interfaces:**
- Produces: task id `052`, branch `agent/claude-code/052-ci-pipeline` — every later task in
  this plan commits to this branch.

- [ ] **Step 1: Create the branch**

```bash
cd /home/marcus/axiom
git checkout master
git pull origin master
git checkout -b agent/claude-code/052-ci-pipeline
```

- [ ] **Step 2: Write the task file**

Create `.ai/tasks/052-ci-pipeline.md`:

```markdown
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
```

- [ ] **Step 3: Verify and commit**

```bash
cat .ai/tasks/052-ci-pipeline.md   # confirm it matches the content above exactly
git add .ai/tasks/052-ci-pipeline.md
git commit -m "chore(052): claim for claude-code, ci pipeline"
```

Expected: commit succeeds; `git log -1 --stat` shows one file added.

---

### Task 2: Author the CI workflow file

**Files:**
- Create: `.github/workflows/ci.yml`

**Interfaces:**
- Consumes: `package.json` scripts (`typecheck`, `lint`, `build`, `test`), `src-tauri/`
  cargo commands, `npm run test:e2e:linux` (from `e2e/README.md`).
- Produces: three job names GitHub will report as status checks — `frontend-checks`,
  `backend-checks` (each fanned out per matrix OS as `<job> (<os>)`), and `e2e`. Task 5
  (branch protection) and Task 6 (validation) depend on these exact names.

- [ ] **Step 1: Write the workflow file**

Create `.github/workflows/ci.yml`:

```yaml
name: CI

on:
  pull_request:
    branches: [master]
  push:
    branches: [master]

jobs:
  frontend-checks:
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4

      - uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'

      - run: npm ci
      - run: npm run typecheck
      - run: npm run lint
      - run: npm run build
      - run: npm run test

      - name: Check for hardcoded design values
        if: matrix.os == 'ubuntu-latest'
        shell: bash
        run: |
          set -euo pipefail
          matches=$(grep -rEn '#[0-9a-fA-F]{3,6}|rgba\(' src/ \
            --include='*.ts' --include='*.tsx' --include='*.css' \
            --exclude='tokens.css' || true)
          if [ -n "$matches" ]; then
            echo "Hardcoded design values found outside src/styles/tokens.css:"
            echo "$matches"
            exit 1
          fi
          echo "No hardcoded design values found."

  backend-checks:
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    runs-on: ${{ matrix.os }}
    defaults:
      run:
        working-directory: src-tauri
    steps:
      - uses: actions/checkout@v4

      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy, rustfmt

      - uses: Swatinem/rust-cache@v2
        with:
          workspaces: src-tauri

      - run: cargo check
      - run: cargo test
      - run: cargo clippy --all-targets --locked -- -D warnings
      - run: cargo fmt --all --check

  e2e:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'

      - uses: dtolnay/rust-toolchain@stable

      - uses: Swatinem/rust-cache@v2
        with:
          workspaces: src-tauri

      - name: Install tauri-driver
        run: cargo install tauri-driver --version 2.0.6 --locked

      - name: Install WebKit driver + xvfb
        run: |
          sudo apt-get update
          sudo apt-get install -y webkit2gtk-driver xvfb \
            || sudo apt-get install -y webkitgtk-webdriver xvfb

      - run: npm ci
      - run: npm run test:e2e:linux
```

Note on the "hardcoded design values" step: it only runs once (`ubuntu-latest` leg) rather
than 3x — it's a static grep over `src/`, identical on every OS, so running it on all three
would be pure redundancy. This doesn't change what the spec's gate table (§5) checks, only
where it runs.

Note on the WebKit driver install: `e2e/README.md` says the package is named
`webkit2gtk-driver` on Ubuntu 22.04/24.04 and `webkitgtk-webdriver` on 26.04. The `||`
fallback handles either without needing to know in advance which Ubuntu version
`ubuntu-latest` resolves to.

- [ ] **Step 2: Validate YAML syntax**

```bash
python3 -c "import yaml; yaml.safe_load(open('.github/workflows/ci.yml'))" && echo "valid YAML"
```

Expected: prints `valid YAML` with no exception. If `pyyaml` isn't installed, run
`pip install --user pyyaml` first.

- [ ] **Step 3: Commit**

```bash
git add .github/workflows/ci.yml
git commit -m "feat(052): add CI workflow (frontend/backend checks + e2e)"
```

---

### Task 3: Update quality-gates.md and TEMPLATE.md

**Files:**
- Modify: `.ai/quality-gates.md`
- Modify: `.ai/tasks/TEMPLATE.md`

**Interfaces:**
- Consumes: nothing from earlier tasks (pure doc edits).
- Produces: nothing later tasks depend on programmatically — this is spec §5's documented
  requirement, included for spec coverage.

- [ ] **Step 1: Edit `.ai/quality-gates.md`'s opening**

Find this text near the top of the file:

```markdown
# Quality gates

A task cannot move from `in-progress` to `review` until every gate that applies to it
passes. "Applies to it" matters — a docs-only task doesn't need `cargo check` run against
Rust code it didn't touch. State in the task file which gates were run.
```

Replace it with:

```markdown
# Quality gates

**As of task 052, CI (`.github/workflows/ci.yml`) enforces the mechanical gates below
automatically on every PR and push to `master`** — `npm run typecheck`/`lint`/`build`/`test`,
the design-token grep, `cargo check`/`test`/`clippy`/`fmt`, and `npm run test:e2e:linux`, all
as required status checks. A task's handoff doc links its PR instead of restating pass/fail
by hand (see `.ai/tasks/TEMPLATE.md`). Two gates stay manual — see "Structural changes" and
"Explicitly not a gate" below; CI can't judge either mechanically.

A task cannot move from `in-progress` to `review` until every gate that applies to it
passes. "Applies to it" matters — a docs-only task doesn't need `cargo check` run against
Rust code it didn't touch. State in the task file which gates were run.
```

- [ ] **Step 2: Edit `.ai/quality-gates.md`'s "End-to-end" section**

Find this text:

```markdown
## End-to-end (Stage 7+)

- Any task touching `src-tauri/` or `src/services/`: `npm run test:e2e:linux` passes. This
  is where an IPC or persistence regression would actually surface, so it's a hard gate for
  this surface specifically, not the whole repo.
- Any other task: advisory. Run it if `e2e/README.md`'s prerequisites
  (`WebKitWebDriver` + `xvfb`) are available in the environment; if not, state that plainly
  in the task file as an environment blocker rather than claiming a pass — see 040's, 042's,
  and 044's `## Review` sections for the pattern.
- No CI provider runs this automatically yet. Once one exists, widen the first bullet to
  every task touching `src/` — the scoping here is a concession to agents not reliably
  having the native WebKit driver installed, not a statement that other surfaces are exempt
  from regressions an E2E flow could catch.
```

Replace it with:

```markdown
## End-to-end (Stage 7+)

CI runs `npm run test:e2e:linux` on every pull request and push to `master`, unconditionally
— not scoped to tasks touching `src-tauri/` or `src/services/` (the `e2e` job in
`.github/workflows/ci.yml` isn't path-filtered). This is a required status check; a task
can't merge without it passing, regardless of which files it touched.
```

- [ ] **Step 3: Edit `.ai/tasks/TEMPLATE.md`'s "What was built / tested / left out" section**

Find this text:

```markdown
## What was built / tested / left out

Filled in when moving to `review`. Specific: which files, which tests, which gates were run
(`.ai/quality-gates.md`), and anything deliberately deferred with a reason.
```

Replace it with:

```markdown
## What was built / tested / left out

Filled in when moving to `review`. Specific: which files, what was deliberately deferred and
why, and a link to the task's PR — CI's check run on that PR (`.ai/quality-gates.md`) is the
source of truth for which mechanical gates passed, not hand-typed pass/fail here. Call out
explicitly only the two gates CI can't check: whether `ARCHITECTURE.md` was updated for a
structural change, and visual fidelity against the mockups.
```

- [ ] **Step 4: Verify and commit**

```bash
git diff .ai/quality-gates.md .ai/tasks/TEMPLATE.md   # confirm both edits match exactly
git add .ai/quality-gates.md .ai/tasks/TEMPLATE.md
git commit -m "docs(052): point quality-gates.md and TEMPLATE.md at CI"
```

---

### Task 4: Install and authenticate `gh` CLI

**Files:** none (environment setup only).

**Interfaces:**
- Produces: a working, authenticated `gh` CLI — Task 5 and Task 7 both require it.

- [ ] **Step 1: Install `gh`**

```bash
type gh >/dev/null 2>&1 && echo "gh already installed" || (
  curl -fsSL https://cli.github.com/packages/githubcli-archive-keyring.gpg \
    | sudo dd of=/usr/share/keyrings/githubcli-archive-keyring.gpg
  sudo chmod go+r /usr/share/keyrings/githubcli-archive-keyring.gpg
  echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" \
    | sudo tee /etc/apt/sources.list.d/github-cli.list > /dev/null
  sudo apt update
  sudo apt install gh -y
)
gh --version
```

Expected: prints a `gh` version string.

- [ ] **Step 2: Authenticate (requires the human)**

`gh auth login` is an interactive device-code flow — it cannot be completed
non-interactively. Hand this step to Marcus:

```bash
gh auth login --hostname github.com --git-protocol ssh --web
```

He completes the browser confirmation; once done, verify:

```bash
gh auth status
```

Expected: shows `Logged in to github.com account Reid-Engineered` (or the account that owns
the repo) with token scopes including `repo`.

---

### Task 5: Push the branch and open the PR

**Files:** none (git/GitHub operations only).

**Interfaces:**
- Consumes: branch `agent/claude-code/052-ci-pipeline` (Task 1), authenticated `gh` (Task 4).
- Produces: an open PR number, and — once CI runs on it — the exact GitHub status-check
  context strings Task 6 needs (`frontend-checks (ubuntu-latest)`, etc.).

- [ ] **Step 1: Push and open the PR**

```bash
cd /home/marcus/axiom
git push -u origin agent/claude-code/052-ci-pipeline
gh pr create \
  --title "feat: CI pipeline (GitHub Actions)" \
  --body "Implements docs/superpowers/specs/2026-08-31-ci-pipeline-design.md. See .ai/tasks/052-ci-pipeline.md for scope." \
  --base master
```

Expected: prints the new PR's URL. Note the PR number from it (referred to as `<PR#>` below).

- [ ] **Step 2: Wait for checks to run once, then list their exact names**

```bash
gh pr checks <PR#> --watch
```

This blocks until all jobs report (pass or fail — either is fine here, this step only
exists to make the check names exist in GitHub's system). When it returns, run:

```bash
gh pr checks <PR#>
```

Expected: seven rows, one per job/matrix-leg. Copy the exact name strings from the first
column verbatim — Task 6 needs them exactly as printed here, not the guessed names below.

---

### Task 6: Configure branch protection on `master`

**Files:** none (GitHub repo settings via API).

**Interfaces:**
- Consumes: the exact check-name strings observed in Task 5, Step 2.
- Produces: an active branch protection rule — Task 7 depends on this being live to prove
  it blocks a bad merge.

- [ ] **Step 1: Write the protection rule request body**

Create `/tmp/branch-protection.json`, substituting the real names copied from Task 5's
`gh pr checks` output if they differ from the ones below (they're expected to match this
exactly, since GitHub renders a matrixed job's context as `<job name> (<matrix.os value>)`):

```bash
cat > /tmp/branch-protection.json <<'EOF'
{
  "required_status_checks": {
    "strict": true,
    "contexts": [
      "frontend-checks (ubuntu-latest)",
      "frontend-checks (macos-latest)",
      "frontend-checks (windows-latest)",
      "backend-checks (ubuntu-latest)",
      "backend-checks (macos-latest)",
      "backend-checks (windows-latest)",
      "e2e"
    ]
  },
  "enforce_admins": false,
  "required_pull_request_reviews": null,
  "restrictions": null
}
EOF
```

`enforce_admins: false` deliberately mirrors the spec's own §3, which assumes an admin can
still bypass in an emergency (and the same `push`-triggered workflow catches it after the
fact). `required_pull_request_reviews: null` is deliberate too — spec §6 names CI's status
checks as the enforcement mechanism, not GitHub's native review-approval feature; adding a
review requirement isn't in scope here.

- [ ] **Step 2: Apply it**

```bash
gh api --method PUT repos/Reid-Engineered/axiom/branches/master/protection \
  --input /tmp/branch-protection.json
```

Expected: prints the resulting protection JSON back, no error.

- [ ] **Step 3: Verify it persisted**

```bash
gh api repos/Reid-Engineered/axiom/branches/master/protection \
  --jq '.required_status_checks.contexts'
```

Expected: prints the same 7 context strings from Step 1.

---

### Task 7: Prove the gate blocks a bad merge, then fix and merge

**Files:**
- Modify (temporarily): any `src/` file, to introduce a deliberate lint violation.

**Interfaces:** none — this task consumes the live PR and branch protection from Tasks 5–6.

- [ ] **Step 1: Introduce a deliberate lint violation**

```bash
echo "const unusedDeliberateTestVar = 1;" >> src/main.tsx
git add src/main.tsx
git commit -m "test(052): deliberately break lint to validate the required check"
git push
```

- [ ] **Step 2: Confirm the check goes red and blocks merge**

```bash
gh pr checks <PR#> --watch
```

Expected: `frontend-checks` fails on all three OS legs (unused-variable lint error).

```bash
gh pr view <PR#> --json mergeable,mergeStateStatus
```

Expected: `mergeStateStatus` is `BLOCKED` (not `CLEAN`) — proof the branch protection rule
from Task 6 is actually enforced, not just configured.

- [ ] **Step 3: Revert the deliberate breakage**

```bash
git revert --no-edit HEAD
git push
gh pr checks <PR#> --watch
```

Expected: all 7 checks pass; `gh pr view <PR#> --json mergeStateStatus` now reports `CLEAN`.

- [ ] **Step 4: Get Marcus's sign-off, then merge**

Per `.ai/merge-strategy.md`, this PR touches `.ai/quality-gates.md` and
`.ai/tasks/TEMPLATE.md` — both require explicit human sign-off before merge, regardless of
who authored or reviewed the task. Do not merge until Marcus has reviewed the PR and said so
explicitly.

Once approved:

```bash
gh pr merge <PR#> --squash --delete-branch
```

- [ ] **Step 5: Close out the task file**

On `master`, after the merge:

```bash
git checkout master
git pull origin master
```

Edit `.ai/tasks/052-ci-pipeline.md` (now on `master`):
- Set `status: done` in the frontmatter.
- Fill in `## Worklog` with a line for today's date noting the merge commit.
- Fill in `## What was built / tested / left out`: the workflow file, the doc edits, and
  the deliberate-failure validation from Steps 1–3 above (link the PR).
- Fill in `## Follow-ups`: macOS/Windows e2e extension, agent orchestration spec, CD spec
  (all from spec §8).

Then move it to the archive:

```bash
mkdir -p .ai/tasks/_archive
git mv .ai/tasks/052-ci-pipeline.md .ai/tasks/_archive/052-ci-pipeline.md
git add .ai/tasks/_archive/052-ci-pipeline.md
git commit -m "chore(052): close out and archive CI pipeline task"
git push origin master
```

Expected: `git log --oneline -1` shows the archive commit on `master`; CI runs again on this
`push` and passes (confirms the `push`-to-`master` trigger from spec §3 works, not just
`pull_request`).

---

## Self-Review Notes

**Spec coverage:** §1 scope (Task 2's job set) ✓. §2 decisions (PR flow — Tasks 1/5/7;
3-OS matrix — Task 2; hosted runners — implicit, no self-hosted config anywhere) ✓. §3
trigger model + branch protection (Task 2's `on:` block; Task 6) ✓. §4 jobs (Task 2, verbatim)
✓. §5 gate mapping + doc changes (Task 3) ✓. §6 failure handling (Task 7, Steps 1–3, native
GitHub UI only, no bot) ✓. §7 validating the pipeline (Task 7 in full) ✓. §8 follow-ups
(recorded in the task file's Follow-ups section, Task 7 Step 5, not built here) ✓.

**Placeholder scan:** no TBD/TODO; every step has literal commands or file content.

**Type/name consistency:** job names `frontend-checks`, `backend-checks`, `e2e` used
identically in Task 2 (defines them), Task 5 (observes them), Task 6 (references them in the
protection rule), Task 7 (checks their status) — no drift.
