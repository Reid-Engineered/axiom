---
id: 002
title: components/mastery/* implementation
status: proposed
owner: antigravity
stage: 1
depends_on: [001]
---

## Scope

Implement `Mastery` (five states) and `ChapterStateProfile` — full implementation, per `ARCHITECTURE.md` §2 and `00-foundations.png`. Likely composes primitives from 001 (e.g. `ProgressBar`), hence the dependency. Antigravity implements; Codex adds render tests covering all five mastery states in the same task, same owner-handoff pattern as 001.

## Plan

- src/components/mastery/Mastery.tsx (+ .module.css, + test)
- src/components/mastery/ChapterStateProfile.tsx (+ .module.css, + test)

## Worklog

## What was built / tested / left out

Filled in when moving to `review`.

## Review

Filled in by the reviewing agent. See `.ai/review-checklist.md`.

## Follow-ups

Anything noticed during implementation or review that's out of this task's scope.
