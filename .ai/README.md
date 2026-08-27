# `.ai/` — multi-agent handoff system

Axiom is built by more than one AI tool against the same repo: Claude Code, Codex, and
Antigravity CLI, plus a human. None of these share a context window with each other. This
directory is how work hands off between them without relying on chat history any one of
them can see.

The rule this directory exists to enforce: **anchor every agent to the same source of
truth.** `ARCHITECTURE.md` and `AGENTS.md` are that truth for *what* to build and *how*.
This directory is the protocol for *who is doing what right now* and *whether it's actually
done*.

## Files

- `lifecycle.md` — the states a task moves through and what triggers each transition.
- `quality-gates.md` — the checks a task must pass before it can move to `review` or `done`.
- `review-checklist.md` — what a reviewing agent checks, and how findings get recorded.
- `merge-strategy.md` — branch naming, merge method, who can merge what.
- `tasks/TEMPLATE.md` — the shape every task handoff doc follows.
- `tasks/*.md` — one file per task, named `<task-id>-<slug>.md` (e.g. `012-concept-row.md`).
  Task ids are sequential, assigned when a task is created, never reused.
- `tasks/_archive/` — tasks in state `done`, moved here to keep `tasks/` scannable for
  "what's active." Moving a file is the last edit made to it.

## How an agent uses this directory

1. **Starting work**: list `tasks/`, find the task (or create one if picking up
   unplanned work — see `lifecycle.md` for how a task gets created), read it fully,
   check for overlapping `in-progress` tasks touching the same files.
2. **During work**: update the task's status and worklog as state actually changes, not in
   a batch at the end.
3. **Finishing**: run the checks in `quality-gates.md`, fill in the task's "What was built /
   tested / left out" section, move status to `review`.
4. **Reviewing**: a different agent (or the human) works through
   `review-checklist.md`, records findings in the task file, moves status to `done` or
   `changes-requested`.
5. **Done**: task file moves to `tasks/_archive/`.

## What does not belong here

Product/UI decisions belong in `reference/UI/AXIOM-HANDOFF.md`. Structural decisions belong
in `ARCHITECTURE.md`. Conventions belong in `AGENTS.md`. This directory is process only — if
a task file starts accumulating design rationale, that rationale belongs in one of the other
three docs, with a pointer left behind.
