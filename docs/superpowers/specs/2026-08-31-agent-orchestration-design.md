# Agent Orchestration — design

Sub-project 2 of 3 in the "professional CI/CD structure" initiative Marcus scoped on
2026-08-31 (sub-project 1, CI, is `.ai/tasks/_archive/052-ci-pipeline.md`, merged). This is
the piece that actually kills the original pain point: Marcus manually copy-pasting
resumption/handoff prompts between Claude, Codex, and Antigravity sessions.

Source material: `.ai/README.md`, `.ai/lifecycle.md`, `.ai/tasks/TEMPLATE.md`,
`.ai/merge-strategy.md`, `AGENTS.md` (Agent responsibilities and handoff workflow section),
`CLAUDE.md` (Coordinating with Codex and Antigravity CLI section), and this session's own
direct investigation of Codex's and Antigravity's headless invocation.

---

## 1. Scope

A script Marcus (or Claude, on his request) runs to trigger a **dispatch round**: scan
`.ai/tasks/` for work that's ready to hand to an agent, and for each ready task, actually
invoke that agent non-interactively instead of Marcus composing and pasting a prompt by
hand. Everything about *how* an agent works once dispatched — reading the task file,
following `AGENTS.md`/`CLAUDE.md`, updating its own status, opening a PR, going through
CI/review — is already built (sub-project 1) and unchanged by this. This project is purely
about the trigger: turning "Marcus notices a task is ready and pastes a prompt" into "Marcus
runs one command."

Explicitly out of scope:
- CD / release artifact builds — sub-project 3, separate spec.
- A fully autonomous background daemon — Marcus chose on-demand dispatch (he or Claude
  triggers a round) over a live watcher; revisit only if on-demand proves too slow in
  practice.
- Parallel dispatch within a round — sequential for v1; see §5.
- Inferring which agent an unowned task should go to — the dispatcher only acts on tasks
  that already have a specific `owner` set (not `unassigned`). Assigning an owner to a new
  task stays a human/Claude planning decision, same as it is today.

## 2. Decisions carried in from brainstorming

- **All three tools are headless-capable, confirmed directly, not assumed:**
  - Codex: `codex exec "<prompt>"` — proven working non-interactively against this exact
    repo earlier this session.
  - Antigravity: `agy -p "<prompt>"` (Google Antigravity CLI's documented print/headless
    mode: https://antigravity.google/docs/cli/headless/). **Not yet installed on this
    machine** — only the Antigravity IDE desktop app is present; installing `agy` is a setup
    step for the implementation plan, the same shape as installing `gh` was for sub-project 1.
  - Claude: no external CLI needed — a task owned by Claude becomes an `Agent` tool dispatch
    in the current session, the same mechanism sub-project 1 used throughout.
- **On-demand trigger**, not a daemon or a schedule — Marcus runs a dispatch round when he
  wants one.
- **Dispatcher is a Node/TypeScript script** (`npm run dispatch`), matching the repo's
  existing toolchain rather than introducing Python as a second scripting language.
- **Readiness is checked, not assumed**: dependencies (existing `depends_on`), file
  conflicts (needs a new structured `files:` frontmatter field — see §3), and `master`'s CI
  status.
- **Sequential dispatch within a round** — one machine, multiple potentially-heavy agentic
  CLI processes; avoid resource contention rather than parallelize prematurely.
- **No automatic retry** on a failed/timed-out dispatch — report it, leave the branch as the
  agent left it, let a human or a later round decide.

## 3. Task file changes required

`.ai/tasks/TEMPLATE.md`'s frontmatter gains one field:

```yaml
files: []   # e.g. [src/components/ConceptRow.tsx, src/components/ConceptRow.test.tsx]
```

Populated when a task is scoped (by whichever agent — usually Claude, per its planning
role), alongside the existing free-prose `## Plan` section, which stays as-is for human
readability. This is the only new data the system needs — it's what lets the dispatcher
check file-conflicts mechanically instead of a human eyeballing two tasks' prose Plan
sections.

## 4. A dispatch round, step by step

1. **Read `.ai/tasks/`**: every file whose `status` is `proposed`, or `in-progress` with no
   worklog entry in the last 3 days (stale — the exact condition `.ai/lifecycle.md` already
   names as a "notice and ask" problem it has no automated answer for; 3 days is a starting
   default, easy to change once this has been used for a while and Marcus has a feel for
   what "stale" actually means in practice).
2. **Filter to ready**: for each candidate —
   - Every id in `depends_on` has `status: done` in its (possibly archived) task file.
   - No other `in-progress` task's `files:` list overlaps this one's.
   - `master`'s latest `push`-triggered CI run (checked via `gh api`) is green. If it isn't,
     the round stops entirely and reports that — dispatching more work onto a broken base
     compounds the problem.
   - `owner` is set to a specific agent, not `unassigned`.
3. **For each ready task, in order**:
   - Create a worktree on branch `agent/<owner>/<task-id>-<slug>` (existing naming
     convention).
   - Commit the claim: `status: in-progress`, `owner` confirmed, a worklog line — on that
     branch, in that worktree.
   - Invoke the owning agent:
     - `codex` → `codex exec "Read .ai/tasks/<file> in full, then AGENTS.md and CLAUDE.md,
       and pick up this task following the workflow they describe."`, run with the worktree
       as the working directory.
     - `antigravity` → the equivalent prompt via `agy -p "..." --output-format json
       --dangerously-skip-permissions --print-timeout 30m`, same working directory.
     - `claude-code` → an `Agent` tool dispatch with the same pointer prompt.
   - Capture exit status and output for the round summary. Do not parse or act on what the
     agent did beyond that — its commits and task-file updates are the record.
4. **Report**: after every ready task has been attempted, print a summary — task, agent,
   outcome (done / still in-progress / blocked / errored), and the worktree path or PR link
   if one exists.

## 5. Error handling

A non-zero exit or a timed-out CLI invocation is not retried automatically. It's recorded in
the round's summary, and the task/branch is left exactly as the agent left it. This isn't a
gap — it's a deliberate choice matching "on-demand, Marcus sets the pace": an unattended
retry loop on a possibly-half-broken change has more downside than a human looking at it
before deciding what happens next.

Blast radius is already contained by sub-project 1 and existing repo convention: every
dispatched task works in its own worktree/branch, nothing reaches `master` without a PR and
CI passing, and anything touching `.ai/`, `AGENTS.md`, or `ARCHITECTURE.md` additionally
needs Marcus's explicit sign-off before merge (`.ai/merge-strategy.md`). A bad dispatch
produces a bad branch or PR, not bad `master`.

## 6. Validating the dispatcher

Once built: create one small, deliberately trivial `proposed` task (similar in spirit to how
sub-project 1's CI was validated with a real PR, not just a read-through), run a dispatch
round, and confirm end-to-end: the right worktree/branch gets created, the right CLI actually
runs non-interactively and does something, the task file's claim commit lands correctly, and
the round's summary accurately reflects what happened. Then a second validation: two tasks
with overlapping `files:` lists, confirming the dispatcher correctly skips the second rather
than racing it against the first.

## 7. Follow-ups (out of scope here, tracked for later)

- CD spec (sub-project 3) — separate brainstorm, after this.
- Parallel dispatch within a round, if sequential proves too slow once this is in real use.
- A scheduled or daemon trigger mode, if on-demand proves too easy to forget to run.
- Owner-inference for `unassigned` tasks, if that turns out to be a real bottleneck rather
  than a rare case.
- `agy` needs installing and authenticating on this machine before any of this can dispatch
  to Antigravity — not done as part of this spec, tracked for the implementation plan.
