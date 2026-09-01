# Merge strategy

## Branching

One branch per task: `agent/<tool>/<task-id>-<slug>`, e.g. `agent/codex/018-trust-badge`.
Branch names identify which tool produced the work — useful when a review finding needs to
reference "how Codex tends to structure this" versus a Claude Code or Antigravity habit, and
useful for the human skimming `git branch` to see what's in flight without opening
`.ai/tasks/`.

Branches are short-lived: created when a task moves to `in-progress`, deleted after merge.
Do not keep long-running per-agent branches — the shared state is `.ai/tasks/` and `master`,
not a personal branch that drifts.

## Merging

- **Squash-merge only.** The task's handoff doc in `.ai/tasks/_archive/` is the durable
  record of what happened; the branch's commit-by-commit history doesn't need to survive.
- A task merges only from state `review` with a recorded `Verdict: <pass>` in its
  `## Review` section — not from `in-progress`, and not on the owning agent's own say-so.
  The reviewer is always a different agent (or the human) than the task's owner.
- Merge commit message is the task's title and id: `feat: ConceptRow component (#012)`.
- After merge: task file moves to `tasks/_archive/`, branch deleted.

## Who can merge what

- Any agent can merge a task it reviewed (not authored) once quality gates and the review
  checklist both pass.
- Changes to `ARCHITECTURE.md`, `AGENTS.md`, `CLAUDE.md`, or anything in `.ai/` itself
  require a human sign-off before merge, regardless of which agent authored or reviewed
  them — these are the shared contract every agent operates against, so a change to them
  changes what "correct" means for everyone, not just for the task that prompted it.

## Conflicts

Two tasks touching the same file is a coordination failure that should have been caught at
"before starting any task" (see `CLAUDE.md`), not a merge-time surprise. If it happens
anyway: the task that reached `review` first merges first; the second task's owner rebases
onto the new `master` and re-runs quality gates before re-requesting review — a rebase after a
dependency changed is not exempt from re-review.

## Main is always buildable

`master` never fails its stage's acceptance criteria in `ROADMAP.md`. If a merge breaks the
build, the very next action — by whichever agent notices, not necessarily the one who broke
it — is a revert, then a new task to redo the work correctly. A broken `master` is not a state
any task is allowed to leave for "someone will fix it later."
