---
id: 044
title: Carry the first-launch subject into Create Workspace
status: done
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

**Built**

- Added an optional `subject` to the `createWorkspace` route, forwarded it through
  `RouteContent`, and used it to seed Create Workspace while preserving the existing
  `'Calculus II'` fallback for routes that omit it.
- First Launch now attaches its trimmed/defaulted subject to the route and no longer writes
  the submitted value into state immediately before navigation.
- Added coverage proving the typed subject is present on the navigated route and initializes
  the Create Workspace Subject input. The existing no-subject creation test still passes.

**Tested**

- `npm test -- src/pages/FirstLaunchPage.test.tsx src/pages/CreateWorkspacePage.test.tsx` —
  passed, 2 files / 5 tests.
- `npm run typecheck` — passed with zero errors; all `Route` consumers typecheck, including
  the four unchanged `navigate({ type: 'createWorkspace' })` call sites that omit `subject`.
- `npm run lint` — passed with zero errors or warnings.
- `npm run build` — passed.
- `npm test` — passed, 57 files / 137 tests.
- `rg -n "#[0-9a-fA-F]{3,6}|rgba\\(" src --glob '!src/styles/tokens.css'` — no matches;
  no hardcoded design values were introduced.
- `npm run test:e2e:linux` — not run: `WebKitWebDriver` is not available on `PATH` in this
  environment. `xvfb-run` is available and `tauri-driver` exists under
  `/home/marcus/.cargo/bin`, but the native WebKit driver prerequisite is missing.

**Left out**

- The no-subject pre-fill and empty-submit fallback remain unchanged, as required by the
  Follow-ups below. No production command surface or other screens were changed.

## Review
Reviewer: claude-code
Date: 2026-08-30
- [x] Correctness — pass. Matches the locked contract exactly: `Route`'s `createWorkspace`
      variant gains optional `subject`, `RouteContent.tsx` forwards it the same way as other
      route params, `CreateWorkspacePage` seeds `useState(subject ?? 'Calculus II')` (renamed
      the local state to `workspaceSubject` to avoid shadowing the prop — a reasonable call,
      not a contract deviation), and `FirstLaunchPage`'s dead `setSubject` write is removed.
      Checked all five `navigate({ type: 'createWorkspace' })` call sites myself
      (`App.tsx:104`, `HomePage.tsx:83,107,391`, `FirstLaunchPage.tsx:77`) — the four that
      pass nothing are untouched and still compile against the optional field.
- [x] Architecture conformance — pass. Shared type change (`Route`) called out explicitly in
      the task file as required; typecheck covers all consumers, re-ran it myself and it's
      clean. No new global state, no component importing a service directly.
- [x] UI rules — pass. No markup or CSS touched; re-checked for hardcoded hex/`rgba(` in the
      diff by inspection (only `tokens.css` itself carries literal values, which is the
      source of truth, not a violation).
- [x] Process — pass. Re-ran `npm run typecheck`, `npm run lint`,
      `npm test -- src/pages/FirstLaunchPage.test.tsx src/pages/CreateWorkspacePage.test.tsx`
      (5/5) myself — all clean, matching the task file's claims. Did not re-run the full
      137-test suite or `npm run build` myself but have no reason to doubt them given the
      focused run and typecheck both passed clean. `npm run test:e2e:linux` not run — same
      missing-`WebKitWebDriver` environment blocker as 042, disclosed honestly rather than
      claimed. New tests cover the actual behavior change (subject carried on the route,
      Create Workspace seeded from it) and the pre-existing no-subject test still passes
      unchanged, which is the real regression check for the `??` fallback.

Verdict: pass

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
