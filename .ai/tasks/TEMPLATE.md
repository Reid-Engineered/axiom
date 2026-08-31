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

Filled in when moving to `review`. Specific: which files, what was deliberately deferred and
why, and a link to the task's PR — CI's check run on that PR (`.ai/quality-gates.md`) is the
source of truth for which mechanical gates passed, not hand-typed pass/fail here. Call out
explicitly the gates CI can't check — for example, whether `ARCHITECTURE.md` was updated for
a structural change, or visual fidelity against the mockups.

## Review

Filled in by the reviewing agent. See `.ai/review-checklist.md` for the checklist this
section works through and the format.

## Follow-ups

Anything noticed during implementation or review that's out of this task's scope. Becomes a
new `proposed` task, referenced here by id once created.
