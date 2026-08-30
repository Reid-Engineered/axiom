---
id: 044
title: Carry the first-launch subject into Create Workspace
status: proposed
owner: codex
stage: 7
depends_on: []
---

## Scope

Wire the subject a learner types on first launch through to the Create Workspace form, so the
form no longer discards it and silently substitutes a hardcoded `'Calculus II'`.

The contract change is locked below by Claude (`AGENTS.md` §Roles — the `Route` union is a
contract); implementation and tests are Codex's.

Explicitly not in scope: what Create Workspace should pre-fill when it is reached *without* a
subject (four of the five entry points), and whether an empty first-launch submit should still
fall back to `'Calculus II'` at all. Both are live product questions — see Follow-ups. This
task preserves today's behavior exactly on every path that does not carry a subject.

## Plan

- `src/hooks/navigationContext.ts` — add the optional field to the `createWorkspace` variant
- `src/layouts/RouteContent.tsx` — pass it through as a prop
- `src/pages/CreateWorkspacePage.tsx` — accept the prop, seed initial state from it
- `src/pages/FirstLaunchPage.tsx` — send it, and drop the dead `setSubject` call
- `src/pages/FirstLaunchPage.test.tsx` / `src/pages/CreateWorkspacePage.test.tsx` — cover it

## Locked contract

```ts
// navigationContext.ts — subject is OPTIONAL, so the four call sites that pass none keep
// compiling and keep their current behavior unchanged.
| { type: 'createWorkspace'; subject?: string }
```

`RouteContent.tsx` forwards it the same way every other route param is already forwarded
(`workspaceId={route.workspaceId}`, `variant={route.variant}`) — pages take props, they do not
read the route from context:

```tsx
case 'createWorkspace':
  return (
    <AppShell>
      <CreateWorkspacePage subject={route.subject} />
    </AppShell>
  );
```

`CreateWorkspacePage`'s props type changes from `Record<string, never>` to
`{ subject?: string }`, and its initial state becomes `useState(subject ?? 'Calculus II')` —
the `??` fallback is what preserves current behavior for the entry points that pass nothing.

## Worklog

- 2026-08-29 (claude-code): Scoped from 040's review follow-up, after the human chose to wire
  the carry-over rather than blank the field or keep the pre-fill as-is.

  **The symptom:** type "Organic Chemistry" on first launch, press Continue, and the Create
  Workspace Subject field reads "Calculus II". Submit without noticing and the workspace is
  named "Calculus II".

  **The cause:** `FirstLaunchPage.tsx:17` holds the typed subject in that page's own
  `useState`; `navigate({ type: 'createWorkspace' })` carries nothing because the route variant
  has no payload (`navigationContext.ts:7`); and `CreateWorkspacePage.tsx:26` independently
  starts at the literal `useState('Calculus II')`.

  **Evidence this was intended and simply left unfinished** — `FirstLaunchPage.tsx:23-24`:

  ```ts
  const submittedSubject = subject.trim() || 'Calculus II';
  setSubject(submittedSubject);
  ```

  That `setSubject` writes to state on a page that unmounts on the very next line, so nothing
  can ever read it. Someone wrote the carry-over and had nowhere to send the value. Delete it
  as part of this fix — it is not load-bearing, and leaving it invites someone to "restore" a
  behavior that never worked.
- 2026-08-29 (claude-code): Checked every caller before locking the contract, so the optional
  field is a deliberate choice rather than a convenience. Five sites navigate to this route:
  `App.tsx:104`, `HomePage.tsx:83`, `:107`, `:391`, and `FirstLaunchPage.tsx:77` all mean "new
  workspace, nothing typed yet" and must keep passing nothing; only `FirstLaunchPage.tsx:25`
  carries a subject. Making the field required would force four unrelated call sites to invent
  a value.
- 2026-08-29 (claude-code): No E2E impact to worry about. `e2e/first-launch-to-home.test.mjs`
  clicks Continue with the field empty, so it takes the `|| 'Calculus II'` fallback and lands
  on an identically pre-filled form, then replaces the value itself. It should keep passing
  untouched — but re-run it (`npm run test:e2e:linux`) rather than assuming, since this task
  changes the exact screen transition it drives. Note it needs `webkitgtk-webdriver` installed;
  see `e2e/README.md` and task 043.

## What was built / tested / left out

Filled in when moving to `review`.

## Review

Filled in by the reviewing agent. See `.ai/review-checklist.md`.

## Follow-ups

- **What should Create Workspace pre-fill when reached with no subject?** Four of the five
  entry points ("New workspace" from Home, "Import a syllabus" from first launch) will still
  land on a form pre-filled `'Calculus II'` after this task, which is the same misleading
  default from a different door. Blanking it is not free: `CreateWorkspacePage.tsx:102`
  disables submit while `!subject.trim()`, so an empty default changes the button's initial
  state. Needs a product decision, not a code decision.
- **Should an empty first-launch submit still default to `'Calculus II'`?**
  `AXIOM-HANDOFF.md` Screen 1 describes that field as "pre-filled with a ghost `'Calculus II'`"
  — *ghost* meaning placeholder, so the real value starts empty — but says nothing about what
  pressing Continue on an empty field should do. Today it silently adopts the ghost as a real
  value.
