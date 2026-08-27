# Task lifecycle

Every task file in `.ai/tasks/` carries a frontmatter block:

```yaml
---
id: 012
title: ConceptRow component
status: in-progress
owner: claude-code
stage: 5           # ROADMAP.md stage this task belongs to
depends_on: [004, 007]
---
```

## States

```
proposed → in-progress → review → done
                ↑            │
                └─ changes-requested
```

- **`proposed`** — task exists, scoped, not started. Anyone can create one; it names what's
  in scope, what stage it belongs to, and what it depends on. An agent picking up work reads
  `proposed` tasks first when nothing is assigned to it explicitly.
- **`in-progress`** — an agent has claimed it (set `owner`) and is actively working. Only one
  owner at a time. If you want to help on someone else's `in-progress` task, coordinate via a
  note in the file, don't just start editing the same files.
- **`review`** — implementation complete per the task's own definition of done, quality
  gates run and recorded, waiting on a review pass by a different agent or the human.
- **`changes-requested`** — reviewer found blocking issues, listed in the task's `## Review`
  section. Owner stays the same; task returns to `in-progress` once the owner starts
  addressing findings.
- **`done`** — review passed, merged to `main`. File moves to `tasks/_archive/`.

## Creating a task

Copy `tasks/TEMPLATE.md` to `tasks/<next-id>-<slug>.md`. Task ids increment from the
highest id in `tasks/` + `tasks/_archive/` combined — check both before assigning one.

A task should be small enough that its "what changed" section, written honestly, is a short
list. If a task's scope grows mid-implementation past what it was created for, don't
silently expand it — split the new part into a fresh `proposed` task and note the split in
both files.

## Stale tasks

A task sitting `in-progress` with no worklog update in a while is either actually stalled or
the owner forgot to update it — either way, flag it rather than starting duplicate work.
There's no automated timeout; this is a "notice and ask" problem, not a cron job, given the
repo doesn't yet have any automation wired up to enforce it.
