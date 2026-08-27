# CLAUDE.md — Axiom

How Claude specifically operates in this repo. `AGENTS.md` is the ruleset every agent
follows; this file is about *behavior* — what Claude does before, during, and after a task,
and how it coordinates with Codex and Antigravity CLI when they're working the same repo.
Read `ARCHITECTURE.md` and `AGENTS.md` first — this file assumes both.

---

## Before starting any task

1. Read `.ai/tasks/` for anything `in-progress` or `review` that touches the same files or
   components this task will touch. Two agents editing `ConceptRow` in parallel is a merge
   conflict waiting to happen — check first, not after.
2. Read the task's handoff doc in full (use `.ai/tasks/TEMPLATE.md` shape). If the task
   references a stage in `ROADMAP.md`, read that stage's acceptance criteria before writing
   code — they're the definition of done, not a suggestion.
3. If the task is architectural (changes folder structure, adds a new cross-cutting
   pattern, introduces a dependency) and isn't already covered by `ARCHITECTURE.md`, stop and
   flag it rather than deciding unilaterally — update the task file with the open question
   and either resolve it against existing docs or surface it to the human.

## Planning

- Plan in the task's handoff doc, not in a separate scratch file — the plan is part of the
  handoff record other agents (and the human) read later.
- A plan names the specific files it will create or touch. If that list grows past what the
  task's stated scope implies, the task is bigger than it looked — split it into a follow-up
  task in `.ai/tasks/` rather than quietly expanding scope.
- For anything genuinely architectural (not covered by this repo's existing docs), use the
  brainstorming skill before planning — this repo's own docs came from that process, and
  new architectural surface should too.

## Reviewing another agent's work

- Review against `.ai/review-checklist.md`, not vibes. Every item on that checklist gets an
  explicit pass/fail in the task file, not a general "looks good."
- Findings go in the task file under a `## Review` heading with file:line references, same
  format whether the author is Claude, Codex, or Antigravity — no different bar for another
  agent's code than for your own.
- Don't silently fix another agent's findings while reviewing. Leave them as findings; the
  original author (or a follow-up task) applies the fix, so the handoff doc stays an
  accurate record of who did what.
- If a review finds the task violates an `AGENTS.md` rule (hardcoded color, duplicated
  markup, a hook called from a component instead of a page), that's a blocking finding, not
  a style suggestion.

## Refactoring

- Refactor only what the current task touches. A task fixing `ConceptRow`'s prop types is
  not the place to also rename `Mastery`'s `size` prop — that's a separate task, however
  small.
- If a refactor is clearly warranted but out of scope, note it in the task's handoff doc
  under "Follow-ups" rather than doing it anyway.

## Coordinating with Codex and Antigravity CLI

- All three tools read and write the same `.ai/tasks/` files — that directory, not chat
  history, is the shared source of truth between agents that don't share a context window.
- Update a task's status frontmatter (`.ai/lifecycle.md` states) the moment work actually
  changes state — don't batch status updates. Another agent may be deciding what to pick up
  next based on what's marked `in-progress`.
- Never edit another agent's active task file except to append (a review section, a
  follow-up note). If something in an active task looks wrong, say so in a new entry, don't
  rewrite their plan or worklog.
- If Codex or Antigravity's output doesn't match `ARCHITECTURE.md` or `AGENTS.md`, the fix
  is a review finding against the code, not a private workaround in your own task. The docs
  are the shared contract; patch the docs (with the human's sign-off) if they're genuinely
  wrong, don't quietly diverge from them.
- Claude's specific responsibility in the rotation: architectural review and anything
  touching `ARCHITECTURE.md`, `AGENTS.md`, or `.ai/` itself. Feature implementation against
  an already-locked component contract (`ROADMAP.md` Stage 2+) is fair game for any agent.

## What "done" means before handing back

A task is not done because the code compiles. It's done when: the acceptance criteria in
`ROADMAP.md` for its stage are met, `.ai/quality-gates.md` passes, `ARCHITECTURE.md` is
updated if structure changed, and the task's handoff doc states what was built, what was
tested, and what — if anything — was deliberately left out of scope.
