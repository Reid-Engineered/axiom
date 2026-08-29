# Axiom Core — the module contract

What "Axiom Core" means, and the contract between it and everything that isn't it. This
document is forward-looking: it defines a target shape for a later, not-yet-scheduled stage,
the same way `ARCHITECTURE.md` described the whole app before Stage 0 existed and
`src/types/visualization.ts` described Stage 8's rendering engine before Stage 8 was
scheduled. **Nothing in this document changes code today.** It doesn't touch
`src/types/module.ts`, doesn't block any Stage 6 task, and doesn't change Stage 7's scope
(SQLite matching `src/types/*` as currently defined).

Scope of this document: the module contract only — what a module is, what it declares, and
how Core invokes it. Three related subsystems are named but deliberately **not** designed
here, each because it's a large enough surface to need its own pass:

- **The event bus** — how capabilities react to other capabilities' activity (a practice
  attempt triggering tutor/mastery/analytics/history updates). Needs the module contract
  settled first, since events are just another kind of message crossing the same boundary.
- **Core's storage abstraction** — how Core persists workspaces/goals/concepts/sessions
  without modules reaching into storage directly. Overlaps with Stage 7 and deserves its own
  design once Stage 7 is closer.
- **Permissions** — what a module can see or do when Core invokes it. Depends on the
  invocation model this document defines, so it comes after.

---

## 1. The boundary

**Axiom Core** owns workspaces, goals, concepts, sessions, and the module registry itself.
It has no knowledge of any subject or capability's internals — no "Calculus," no "shell
method," no "Socratic method" anywhere in Core code. Core's job is to host capabilities, not
understand them.

**A module** is a bundle that declares capabilities. **A capability** is a structured,
versioned contract, not a category label: `tutoring.socratic@1`, not "Tutor." This mirrors a
pattern already in this codebase — `VisualizationScene` (`src/types/visualization.ts`) models
a 3D scene as plain, verified primitives specifically so a real rendering engine can slot in
later without a page-level rewrite. Capabilities apply the same discipline to *any* module
output, not just visualization.

Instead of Core asking "what type of module is this?", it asks "what capabilities does this
provide, and what does it require?" A module that provides `tutoring.socratic@1` and a module
that provides `tutoring.coach@1` are both just tutoring-shaped modules to Core — it never
special-cases either by name.

## 2. Real isolation, designed for now

Modules run in-process today. This document designs as though they won't always: every value
that crosses the Core/module boundary — capability input, capability output, and (in a future
event-bus design) event payloads — must be plain, structurally-clonable data. No functions, no
class instances, no React nodes, nothing that only makes sense inside the same JS heap. A
capability that wants to render UI describes *what* to render as data (again, the
`VisualizationScene` pattern); it does not hand Core a component to mount.

This is the one property every other section in this document exists to protect.

## 3. Capability descriptor and module manifest

```typescript
interface CapabilityDescriptor {
  id: string;              // "practice.generation", "visualization.3d", "tutoring.socratic"
  version: number;         // integer, bumped on breaking input/output changes
  input: unknown;          // placeholder — each capability defines its own concrete payload type
  output: unknown;         // same
}

interface CapabilityRequirement {
  id: string;
  minVersion: number;
}

interface ModuleManifest {
  id: string;
  name: string;
  provides: CapabilityDescriptor[];
  requires: CapabilityRequirement[];
}
```

A module declares both what it offers (`provides`) and what it depends on (`requires`). This
is what makes "this tutor module needs a visualization provider" a checkable fact at
registration time instead of a runtime surprise.

`input`/`output` are typed `unknown` here on purpose — this document defines the envelope
every capability shares, not the payload shape of any specific capability. Each capability
author defines its own concrete input/output types when that capability is actually built,
the same way each service function today defines its own return type against the shared
`Promise`-returning shape from `ARCHITECTURE.md` §5.

## 4. The call

Every invocation is a shared envelope plus a capability-specific payload, and per §2, both
must be plain serializable data:

```typescript
interface CapabilityCall<Input> {
  envelope: {
    workspaceId: string;
    capabilityId: string;
    version: number;
    callingModuleId: string;
  };
  input: Input;
}
```

The envelope is what lets Core generically log, route, and version-check every call without
understanding what's inside `input` — the same separation an HTTP request draws between
headers and body.

Invocation is async request/response, deliberately mirroring `ARCHITECTURE.md` §5 rule 2's
"services are `async` now, on purpose" — a capability call today runs in-process and resolves
immediately, but nothing about its shape changes when a future call is actually
postMessage- or IPC-backed:

```typescript
interface ModuleRegistry {
  register(manifest: ModuleManifest): void;
  resolve(workspaceId: string, requirement: CapabilityRequirement): CapabilityHandle | null;
  invoke<Input, Output>(handle: CapabilityHandle, call: CapabilityCall<Input>): Promise<Output>;
}
```

## 5. Resolving multiple providers

If more than one enabled module in a workspace provides a matching capability at a
sufficient version, `resolve` returns the first match walking the workspace's
`enabledModuleIds` in order. This is a deliberate, simple default so the contract is fully
specified — not a claim that it's the right long-term UX. A real user-facing choice (a
settings surface for "which tutor module handles Socratic prompts") is future work, tracked
here so it isn't lost, not designed now.

## 6. First-party modules are third-party modules

Any module Axiom itself ships — Tutor, Visualizer, Practice, CAS, Notes, Review — is built
against exactly this contract, with no back door into Core's internals. If a first-party
module needs something this contract can't express, that's a finding against this document,
worth fixing here, not a private exception for first-party code. The discipline this buys:
Core can't quietly accumulate assumptions that only hold because "we control both sides
today" — assumptions that break the moment a real third-party module shows up.

## 7. Relationship to `src/types/module.ts`

Unchanged, for now, and deliberately unreconciled. Today's `Module` type is a UI/catalog
record — name, icon, trust badge, description, `enabled`, `visibility` — used to render
marketplace and workspace-tools rows. It has no relationship to a module *running* anything;
every "module" in the app today is a data row, not code. `ModuleManifest` and
`CapabilityDescriptor` describe a different thing entirely: the contract for a module that
actually does something.

**Open question, not resolved here**: when real module capability code gets built (likely
after Stage 7, since that's the first point a module needs to *do* something rather than
just be listed), does `Module` grow a `manifest` field, do the two types merge, or do they
stay separate (catalog metadata vs. runtime contract)? Flagged so it isn't a surprise later,
not answered now — answering it before any capability actually exists would be designing
against a guess.

## 8. What "done" looks like for this document

This document is complete when it accurately describes the module contract; it is not
"implemented" by any code change, because no module with real capability code exists yet.
The next concrete step that touches code is whenever a task actually builds a module with a
real capability — at that point, that task is reviewed against this document the way a
Stage 6 page task is reviewed against `ARCHITECTURE.md`, and any gap found here is a finding
against this document, not a silent workaround in that task.
