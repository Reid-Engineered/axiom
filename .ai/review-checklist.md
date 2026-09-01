# Review checklist

Used by whichever agent (or the human) reviews a task in state `review`. Work through every
item that applies; record each as pass/fail with a one-line note in the task's `## Review`
section. A single unresolved fail keeps the task in `changes-requested` — there's no
"approve with minor comments" state here, because a task file with an open finding and a
`done` status is exactly the kind of silent drift this system exists to prevent.

## Correctness

- [ ] Does the code do what the task's own "what was built" section claims?
- [ ] Do the stated tests actually cover the stated behavior, not just exercise the happy path?
- [ ] Any edge case obvious from the component's prop types (e.g. `Mastery`'s `new` state,
      an empty list from a hook) handled, not just the common case?

## Architecture conformance (`ARCHITECTURE.md`)

- [ ] Domain data fetched only by hooks, only called from pages (§5 rule 1).
- [ ] New types added to the correct file in `src/types/` and re-exported from `index.ts`.
- [ ] Service functions are `async`/`Promise`-returning even against mock data (§5 rule 2).
- [ ] No new global state introduced outside `NavigationContext` / `WorkspaceContext`
      without a documented reason (§5 rule 3).

## UI rules (`AGENTS.md`)

- [ ] No hardcoded color/radius/shadow/spacing — everything traces to `tokens.css`.
- [ ] No markup duplicated from an existing component — reused, not recreated.
- [ ] Matches `reference/UI/AXIOM-HANDOFF.md` / current screenshots for the screen(s)
      touched (check `15-system-refinements.png` supersedes where noted).
- [ ] Mock copy follows the handoff's copy rules (no exclamation marks, no emoji, states the
      misconception rather than "incorrect").

## Process

- [ ] Quality gates: the task's linked PR shows every CI check green (`.ai/quality-gates.md`
      lists which gates are mechanical vs. manual); the manual ones are checked by eye.
- [ ] Task file's worklog reflects what happened, in enough detail that someone reading only
      the task file (not the diff) understands the shape of the change.
- [ ] Scope matches what the task was created for; anything extra split into a follow-up
      task rather than folded in silently.
- [ ] `ARCHITECTURE.md` updated if structure changed.

## Recording the review

```markdown
## Review
Reviewer: <agent>
Date: <date>
- [x] Correctness — pass
- [x] Architecture conformance — pass
- [ ] UI rules — FAIL: ConceptRow.module.css:14 hardcodes `#E9E7E2` instead of
      `var(--color-recessed)`
- [x] Process — pass

Verdict: changes-requested
```
