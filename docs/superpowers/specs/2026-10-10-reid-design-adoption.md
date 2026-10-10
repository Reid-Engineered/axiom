# Reid Engineered design adoption

Status: accepted product direction; implementation pending.
Date: 2026-10-10.
Decision maker: Marcus, in the Codex chat following review and merge of PR #25.

## Decision

Adopt the complete visual and product direction in the five Reid Engineered mockups.
The user's clarification was: "Adopt the full design and product direction."

[The UI handoff](../../../reference/UI/AXIOM-HANDOFF.md) is the normative specification.
It replaces the old bans on dark chrome, glass, red feedback, numeric mastery, charts,
clean-attempt counts, and app-area navigation. Task 080's recommendations to cut or defer
parts of the proposal were a review of the former contract, not a veto of this later
human decision.

## Preserved boundaries

Workspace identity/storage, goals, capability isolation, services → hooks → pages →
props-driven components, token centralization, explicit sample exploration, offline
operation, and truthful learner evidence remain required.

Domains is a presentation label for existing workspaces. A stable Practice product area
does not bind navigation to a named module implementation. Native controls must actually
operate on their platform. Numbers in the drawings are example values, not DB migrations.

## Delivery

Task 081 records the adoption and aligns authoritative docs. Tasks 082–086 cover visual
foundations, capability contracts, navigation/Today, adaptive Practice/feedback, and
Error profile/History. Task 076's timing scope is still relevant; 067/068's regression and
offline proof remain required. Task 078's old visual-polish target is superseded by 082,
without silently rewriting its historical record.

This decision is not evidence that the runtime already contains a mastery engine,
diagnostic classifier, scheduler, or the new pages. Each implementation PR locks its
interfaces, supplies real data, passes quality gates, and receives independent review.

## Validation

Documentation consistency and reference checks apply to this adoption. Browser rendering,
platform fidelity, and behavioral/native tests belong to implementation acceptance and
have not been claimed by this decision.
