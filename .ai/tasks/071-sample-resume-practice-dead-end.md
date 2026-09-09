---
id: 071
title: Remove the sample Resume session practice dead end
status: proposed
owner: unassigned
stage: 8
depends_on: [041, 063, 065, 066]
---

## Scope

Ensure the seeded sample workspace's prominent Home continuation opens a usable practice
state, or replace that continuation with an honest action that cannot promise a resumable
problem it cannot supply.

Does not add practice content to arbitrary learner-created workspaces or change the
generated Shell-method loop that already works when entered through Concept > Practice
this.

## Reproduction and acceptance

Observed in the Windows release build from `0503f3e` while walking task `064`:

1. On first launch, choose "Explore a sample workspace."
2. On Home, choose the prominent Calculus II — Shell method "Resume session" action.
3. The Study Session opens at "Problem 6 of 12," but its problem pane says only "There is
   no practice content for this concept yet." There is no practice action or recovery in
   that pane; the learner must abandon the advertised continuation through the sidebar.

Acceptance: the Home continuation cannot lead to that no-content state. Its destination
must either be bound to the seeded Shell-method knowledge concept and expose a real prompt,
or its copy/action must accurately route the learner to a usable next step. Add a native or
rendered regression that starts from the first-launch sample import and exercises this
specific continuation.

## Plan

Inspect the seeded continuation/session records and the Home resume routing before naming
the exact files. Keep the fix scoped to the sample continuation; do not generalize sample
data into automatic content for real workspaces.

## Worklog

- 2026-09-09 — Filed from the task 064 Windows release-build walk as the blocking dead end
  for beta criterion 6.

## What was built / tested / left out

## Review

## Follow-ups
