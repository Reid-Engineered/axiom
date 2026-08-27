---
id: 000
title: <short title>
status: proposed
owner: <unassigned | agent name>
stage: <ROADMAP.md stage number>
depends_on: []
---

## Scope

What this task builds, in one or two sentences. What it explicitly does not build (if
there's an obvious adjacent thing someone might assume is included).

## Plan

Files to be created or touched. If this list grows materially once work starts, that's a
signal the task is bigger than scoped — see `.ai/lifecycle.md` on splitting.

## Worklog

Dated, append-only. Update as state actually changes, not in a batch at the end.

- <date> — started, claimed by <owner>
- <date> — <what happened>

## What was built / tested / left out

Filled in when moving to `review`. Specific: which files, which tests, which gates were run
(`.ai/quality-gates.md`), and anything deliberately deferred with a reason.

## Review

Filled in by the reviewing agent. See `.ai/review-checklist.md` for the checklist this
section works through and the format.

## Follow-ups

Anything noticed during implementation or review that's out of this task's scope. Becomes a
new `proposed` task, referenced here by id once created.
