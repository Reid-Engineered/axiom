---
id: 045
title: Stage 8 architecture lock — rewrite CORE.md into the active Rust contract
status: review
owner: claude
stage: 8
depends_on: []
---

## Scope

Rewrite `CORE.md` from a forward-looking, code-inert draft ("nothing in this document
changes code today") into Stage 8's active, locked contract, per
`docs/superpowers/specs/2026-08-30-module-capability-runtime-design.md` (the approved
design for this sub-project). This is a docs-only task — Claude alone, per `AGENTS.md`
§Roles — that produces the reference tasks 046–048 implement against. It does not write any
Rust code itself.

Explicitly not in scope: the event bus, Core's storage abstraction, and permissions —
`CORE.md`'s existing framing already names these as deliberately undesigned, and nothing in
the design spec changes that. Leave those three sections as-is.

## Plan

- `CORE.md` — rewritten in place.

## Worklog

- 2026-08-30 (claude-code): Concrete rewrite guidance, so the next read of `CORE.md` doesn't
  have to reconstruct it from the design spec:
  1. **Header** — drop "nothing in this document changes code today." Add that this is
     Stage 8 sub-project 1's active contract, implemented by tasks 046–048, and link the
     design spec.
  2. **§3 (capability descriptor and module manifest)** — replace the TypeScript sketch
     with the Rust types from the design spec §6: `ModuleId`, `CapabilityId`,
     `ModuleManifest`, `CapabilityDescriptor`, `CapabilityRequirement`. Keep the surrounding
     prose about `input`/`output` being capability-specific — it's still true, just now
     expressed as `serde_json::Value` at the `CapabilityProvider` boundary (spec §6) rather
     than TypeScript `unknown`.
  3. **§4 (the call)** — replace `CapabilityCall`/`ModuleRegistry` with the spec §6 Rust
     versions (`CapabilityCall<Input>`, `CallEnvelope`, `CapabilityHandle`), and add the
     `CapabilityProvider` trait (spec §6) as the answer to "how does an invocation reach
     real code" — the original draft didn't address this because no real capability existed
     yet to force the question.
  4. **New: manifest format and errors.** Add a section (or extend §3) covering the
     `module.toml` schema (spec §4), the identifier grammar (spec §5), the `ManifestError`/
     `RegistryError` split (spec §7), and the lifecycle (spec §8). This is new content the
     original draft didn't have, since it predates the manifest format existing at all.
  5. **§5 (resolving multiple providers)** — this is already correct and matches the design
     spec's `ModuleInstallation` ordering exactly (spec §3, §6). Carry it forward essentially
     unchanged; add a note that `ModuleInstallation` (spec §3) is the concrete type that
     carries `enabledModuleIds` for Stage 8, staying in-memory/test-fixture data until a
     later sub-project wires it to SQLite.
  6. **§7 (relationship to `src/types/module.ts`)** — this section's open question is now
     answered: keep `ModuleMetadata` (the renamed `src/types/module.ts::Module`, catalog
     layer, untouched) and `ModuleManifest` (Rust, runtime layer) separate, with
     `ModuleInstallation` as the third, per-workspace layer (spec §3). State the resolution
     plainly rather than leaving it flagged as open. Note: renaming `Module` →
     `ModuleMetadata` in `src/types/module.ts` itself is **not** part of this task — that's
     a TypeScript change with its own blast radius (every import site) and belongs to
     whichever later task actually needs the rename to avoid confusion in practice, not a
     docs-only architecture task. Flag it as a Follow-up here instead of doing it now.
  7. **§8 (what "done" looks like)** — update: this document is no longer "not implemented
     by any code change" for the parts tasks 046–048 build. Split it: the manifest/runtime
     contract sections are implemented once 046–048 land; sections not yet touched (event
     bus, storage abstraction, permissions) keep the original framing.
  8. **New: exit gate note.** Record the design spec §10 reasoning (why the
     `CapabilityProvider` boundary already supports a future out-of-process module) directly
     in `CORE.md` §2 ("Real isolation, designed for now") — that section already makes the
     in-process-today/out-of-process-later argument in the abstract; now there's a concrete
     mechanism to point to.
- 2026-08-30 (claude-code): Sequencing note — this task has no code dependency on 046–048,
  but they depend on this one being `done` first, since it's their reference contract.
  Claim and finish this before 046 starts, even though nothing enforces that mechanically.

## What was built / tested / left out

**Built**

- Rewrote `CORE.md` per the Worklog's eight points: header no longer claims code-inertness;
  §3 now covers the full `module.toml` schema, identifier grammar, Rust types
  (`ModuleManifest`, `CapabilityDescriptor`, `CapabilityRequirement`, `OfflineCapability`),
  `ManifestError`, `ManifestSource`/`EmbeddedManifestSource`, and the lifecycle; §4 replaced
  the TS `CapabilityCall`/`ModuleRegistry` sketch with the Rust versions plus the
  `CapabilityProvider` trait and `RegistryError`/`InvocationError`; §5 gained the concrete
  `ModuleInstallation` type and its in-memory-for-now scoping note; §6 points at task 048's
  conformance harness as the enforcement mechanism; §7's open question is resolved to the
  three-layer split (`ModuleMetadata`/`ModuleManifest`/`ModuleInstallation`); §8 splits
  "done" between the now-implemented §§1–5 and the still-aspirational three subsystems.
- Also noted in §3: `CapabilityDescriptor` dropped the original draft's placeholder
  `input`/`output: unknown` fields (a struct field can't hold a type) — explained why the
  Rust design defers the same way through `CapabilityCall<Input>`'s generic and each
  provider's typed deserialization instead, so a future reader doesn't mistake the removal
  for an oversight.

**Tested**

- Read the full rewritten document twice for internal consistency (type names/signatures
  match between §3 and §4's uses of them) and for stray TypeScript-isms
  (`grep -n "typescript\|interface "` — no matches).
- No `src/`, `src-tauri/`, or `.ai/` code changed — a docs-only task, per
  `.ai/quality-gates.md`'s own opening line the typecheck/lint/build/`cargo` gates don't
  apply.

**Left out**

- Renaming `Module` → `ModuleMetadata` in `src/types/module.ts` — filed as a Follow-up
  below, not this task's job (a TypeScript change with its own blast radius).
  `ROADMAP.md`'s stale "Playwright/Tauri-driver" line, noticed in passing — also a
  Follow-up, unrelated to Stage 8.
- The event bus, storage abstraction, and permissions sections — explicitly out of scope,
  unchanged.

## Review
Reviewer: Marcus (human sign-off — `CORE.md` is the shared cross-agent module contract;
not literally named in `.ai/merge-strategy.md`'s sign-off list, given the same treatment as
`ARCHITECTURE.md`/`AGENTS.md` for that reason. Docs-only tasks have no second AI reviewer
per `AGENTS.md` §Roles).
Date: 2026-08-30
- [x] Process — pass. Full diff reviewed and approved before commit.

Verdict: pass

## Follow-ups

- Rename `Module` → `ModuleMetadata` in `src/types/module.ts` (and every import site) once a
  later task actually needs the two concepts distinguished in code, not before.
- `ROADMAP.md`'s Stage 7 acceptance criteria still says "Playwright/Tauri-driver flows" —
  task 043 fixed this in `AGENTS.md` and `.ai/quality-gates.md` but missed this one instance.
  Small, unrelated to Stage 8; worth a one-line fix whenever someone's next in that file.
