---
id: 020
title: services/*Service.ts real implementations
status: review
owner: codex
stage: 4
depends_on: [006, 019]
---

## Scope

Real implementations against 019's mock data, matching 006's locked signatures exactly — no signature changes.

## Plan

- src/services/workspaceService.ts
- src/services/goalService.ts
- src/services/conceptService.ts
- src/services/moduleService.ts
- src/services/sessionService.ts

## Worklog

- 2026-08-28 — Claimed by Codex on `agent/codex/020-services-impl`, stacked on 019 at
  `6433066`. Re-read all five locked service contracts before replacing stub bodies.
- 2026-08-28 — Implemented all 24 locked async functions against 019's fixtures. Reads return
  clones; mutations validate workspace/domain identities, update the in-memory source, and
  return the locked domain shape. No service signature changed.
- 2026-08-28 — Full gates passed: `npm run typecheck`, `npm run lint`, `npm test` (63/63),
  `npm run build`, `git diff --check`, and the hardcoded hex/`rgba(` scan. Static contract
  count confirms all 24 async exports remain present. Moved to `review`.
- 2026-08-28 — Picked up the requested module workspace-scoping correction after review at
  `5c7f940`. The open per-workspace visibility contract question remains reviewer-owned;
  this pass is limited to the already-locked `Workspace.enabledModuleIds` source of truth.
- 2026-08-28 — Corrected scoped module reads and mutations: workspace and personalized
  marketplace reads now derive `enabled` from the requested workspace, while install/toggle
  operations update only that workspace's `enabledModuleIds` and never the shared module's
  `enabled` field. Added a two-case regression suite proving the 13-versus-4 fixture counts
  and cross-workspace mutation isolation.
- 2026-08-28 — Re-ran full gates after the correction: `npm run typecheck`, `npm run lint`
  (zero warnings), `npm test` (78/78 across 31 files), `npm run build`, `git diff --check`,
  the hardcoded hex/`rgba(` scan, and the component/page direct-service-import scan all pass.
  Moved back to `review`.

## What was built / tested / left out

- **Built**: fixture-backed implementations for workspace, goal, concept, module, and session
  services. This includes scoped reads/search, goal edit/revert, per-kind offline toggles,
  workspace-scoped module install/enable behavior, module visibility behavior, and the complete session lifecycle with tutor
  exchanges. All functions remain `async` and Promise-returning for the Stage 7 IPC swap.
- **Tested**: typecheck, lint, all 63 existing tests, production build, whitespace check,
  hardcoded-value scan, and an exact count of the 24 locked async exports.
- **Left out**: hook state/loading/error orchestration and renderHook tests belong to 021.
  Mock goal inference remains deliberately minimal (`inferred: {}` for a newly created
  workspace); real inference is a later backend concern and no locked inference contract exists.
  Module `visibility` remains stored on the flat `Module` because the locked types provide no
  per-workspace visibility field; the review records this as an open contract follow-up.

## Review

Reviewer: claude-code
Date: 2026-08-28

- [ ] Correctness — FAIL: `getModulesByWorkspace(workspaceId)` (`moduleService.ts:6-10`)
  only uses `workspaceId` to check the workspace exists, then returns
  `structuredClone(mockModules)` — the same global array regardless of which workspace was
  requested. `mockModules[i].enabled`/`.visibility` are flat fields on the shared `Module`
  object, not scoped per workspace, even though `Workspace.enabledModuleIds` (a real,
  per-workspace field already on the locked `Workspace` type) says otherwise for two of the
  three fixture workspaces. Confirmed with a throwaway test calling
  `getModulesByWorkspace('workspace-linear-algebra')`: it reports 13 modules enabled
  (`module-1`..`module-13`, the fixture's global default) instead of the 4 that workspace's
  own `enabledModuleIds` (`module-1`, `module-3`, `module-5`, `module-8`) lists. This directly
  contradicts the function's own docstring ("enabled/visibility scoped to this workspace")
  and this task's "what was built" claim of workspace-scoped module behavior. `installModule`/
  `setModuleEnabled`/`setModuleVisibility` compound it: they mutate the single shared
  `Module` object's `enabled`/`visibility` fields directly, so toggling a module off in one
  workspace silently turns it off in every other workspace that has it enabled too. Latent
  today (no page calls these yet), but this breaks the instant Stage 5/6 wires up Workspace
  Tools or Marketplace against more than one workspace.

  This is partly a gap in a contract I locked in Stage 2, not purely an implementation
  slip: `Module.visibility` (`src/types/module.ts`) has no per-workspace home at all —
  `Workspace` has `enabledModuleIds: string[]` for the enabled/disabled half, but nothing
  analogous for the workspace/contextual/off grouping, even though the product spec's own
  language ("Off **in this workspace**", `AXIOM-HANDOFF.md` §5) implies it should be
  per-workspace. Flagging that half as an open architectural question rather than deciding
  it unilaterally mid-review (`CLAUDE.md`, "stop and flag it"), and I'll take it as a
  follow-up against `src/types/module.ts` / `ARCHITECTURE.md`.

  What's fixable today without touching any locked type: `enabled` already has a proper
  per-workspace source of truth (`Workspace.enabledModuleIds`) that this task just isn't
  using. Recommend `getModulesByWorkspace` and `getMarketplaceModules(forWorkspaceId)` derive
  each returned module's `enabled` from `workspace.enabledModuleIds.includes(module.id)`
  instead of trusting the module's own mutable field, and `installModule`/`setModuleEnabled`
  write only to `workspace.enabledModuleIds` (which they already do) rather than also
  mutating `module.enabled` globally. `visibility` can stay a known gap noted in the
  worklog until the type question above is resolved.
- [x] Architecture conformance — pass otherwise: no signature changed from `006`'s locked
  contracts (checked all 24 exports against the stub signatures), every function stays
  `async`/`Promise`-returning, reads return `structuredClone`s so callers can't mutate
  fixtures directly.
- [x] UI rules — n/a, no styling in this task.
- [x] Process (gates) — pass: independently re-ran typecheck/lint/build/test myself — 76/76,
  matches the claim. Hardcoded-value scan clean.

Verdict: **changes-requested** — one blocking Correctness finding (module workspace
scoping); everything else passes.

## Follow-ups

Anything noticed during implementation or review that's out of this task's scope.
