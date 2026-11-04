# Knowledge Package learning-evidence extension

Date: 2026-11-03. Task [083](../../../.ai/tasks/083-learning-evidence-contracts.md).
Status: proposed schema-2 contract; independent review and human sign-off pending.
No package, loader or production content is changed by this document.

Depends on [learning evidence contracts](2026-11-03-learning-evidence-contracts.md).
This extends [Knowledge Package v1](2026-08-30-knowledge-package-v1-spec.md), not Core's
subject model. Runtime package schema is currently 1; its strict parser must not be fed
these new fields under schema_version=1. Policies P1–P7 in the companion spec remain open.

## Ownership and compatibility

Package authors own objective/Bloom demands, prerequisite edges, pattern descriptions,
opportunity eligibility, evidence requirements and source provenance. Practice owns
detector implementation/version, execution and learner attributions. A package may request
a registered detector; it cannot execute scripts, SQL, arbitrary code or network requests.
Core routes opaque content identifiers and never reads or interprets this schema.

All additions below inherit package_id, package semantic version, schema_version=2,
canonical content digest and immutable snapshot persistence defined by Stamp in the
companion spec. Every authored entity has provenance_refs using the existing SourceId,
locator, direct/derived format. References resolve within the same package version; a
source citation supports authored material, never proves learner behavior. Changing a
detector rule or tag requires a new package version/digest, not an in-place update.

Schema 1 remains readable unchanged by a future dual-version loader. Its missing evidence
metadata means unsupported, not an empty list of known-clean opportunities. Old instances,
seeds, hints and canonical solutions must reproduce identically; adding metadata does not
authorize changing generation. Historical attempts retain their schema-1 instance. Schema
2 adoption requires an explicit reviewed content migration; do not auto-infer Bloom or
patterns from names, prose, difficulty integers, or old free-text diagnostics.

## Authored shapes

Schema 2 retains all v1 fields. New objective frontmatter fields:

```toml
bloom_level = "apply"
assessment_evidence = ["final-answer"]
```

bloom_level is one of remember/understand/apply/analyze/evaluate/create and describes
question demand only. assessment_evidence is a nonempty unique list from final-answer,
structured-step, method-selection. Practice rejects an assessment claim whose required
evidence it cannot collect. Objective id/concept_id/provenance_refs and body remain v1.
No Bloom demonstration is created just because a tagged question was presented.

Add a `patterns/` entity collection. Each `patterns/<id>.md` has TOML frontmatter:

```toml
id = "pattern.example"
name = "A named misconception"
concept_ids = ["concept.example"]
objective_ids = ["objective.example"]
detector = { id = "diagnostic.example", version = 1 }
required_evidence = ["structured-step"]
resolution_policy_id = "policy.example"
[[provenance_refs]]
source_id = "src.example"
kind = "derived"
+++
```

The example shows fields only; actual files use the v1 `+++` opening and closing delimiters.
The body has exactly `## Explanation` and `## Next action`, both nonempty. It supplies
learner copy; runtime evidence provides the actual conclusion. Pattern ids use existing
dot-segmented lowercase identifier grammar, are unique in the package, and are not Core
concept ids. concept_ids/objective_ids are nonempty unique lists. detector id/version must
resolve to a provider-supported implementation before the pattern can be assessed.
required_evidence uses the objective vocabulary above. resolution_policy_id references an
approved provider policy snapshot, not an inline Core formula. Detector execution parameters
are excluded from schema 2; a new detector version carries any rule change.

ProblemFamily adds explicit objective assessment scope and opportunity entries:

```toml
assessed_objective_ids = ["objective.example"]
[[opportunities]]
id = "opportunity.example"
pattern_id = "pattern.example"
eligible_when = "always"
required_evidence = ["structured-step"]
```

assessed_objective_ids is a nonempty unique subset of objective_ids. The v1 objective_ids
can include contextual objectives; assessment scope prevents awarding all of them credit
for one final answer. opportunity id is unique package-wide. eligible_when is either
`always` or a constraint in the existing finite parameter-constraint grammar; evaluate
against resolved parameters at generation. No learner-expression or arbitrary-code
predicates. required_evidence must include all pattern requirements; collecting extra
evidence never permits weakening them. A family can have zero opportunities, explicitly
declared as `opportunities = []` when there are no array entries. This means no authored
pattern coverage, not successful learner avoidance.

Generated evidence metadata (owned by Practice, version 1) snapshots:

```typescript
type InstanceEvidence = {
  assessed_objective_ids: string[];
  bloom_targets: { objective_id: string; level: string }[];
  opportunities: { id: string; pattern_id: string; eligible: boolean;
    required_evidence: string[]; detector_id: string; detector_version: number }[];
  restrictions: string[];
  content_ref: { package_id: string; package_version: string;
    schema_version: number; digest: string };
};
```

Persist this beside the immutable ProblemInstance, not by reconstructing it from the
latest package on resume. resolved parameters and the originating authored constraint are
its provenance for eligible. Objective and detector references carry the pinned content
snapshot. restrictions are resolved human-readable parameter constraints generated from
the family's constraints; they do not imply the checker supports arbitrary-domain proofs.
Provider assessment requires both eligible metadata and collected evidence. A generated
eligible opportunity with no supported detector/evidence finalizes unknown. No method
evidence can be synthesized from a final numeric value. Schema-1 attempts have unavailable
InstanceEvidence; do not materialize these fields with empty/zero substitutes.

## Validation and unavailable behavior

The dual-version loader rejects unknown fields per schema, unsupported schema versions,
duplicate/dangling ids, empty required lists, unknown Bloom/evidence enums, malformed
provenance, mismatched concept/objective scope, repeated pattern entries per family,
unknown parameter constraints, nonfinite bounds and detector version zero. Each assessed
objective must belong to the family's concept; a pattern opportunity must intersect the
family's assessed objectives and include their necessary evidence. Preserve v1's existing
constraint, solution, hint order and provenance validation. Validate references after
parsing all entities, before advertising package assessment support.

Unknown detector/policy availability is distinguished from malformed authored content:
loading structurally valid content may succeed, but assessment support is unavailable for
that pattern/objective. Ordinary supported practice may continue with explicit limitations;
a targeted drill requiring unsupported evidence returns content-unavailable. No silent
replacement with a different pattern or provider. Packages cannot approve P1–P7 by naming a
policy id; the approved provider revision must be installed and its digest pinned.

The current bundled family cannot claim method attribution without a new evidence
collection surface and a verified detector. Schema support alone does not make the five
reference screens' sample diagnoses real. Additional authored content/detector work must
receive its own scope and tests before UI reports an occurrence or clean attempt.

## Implementation plan and conformance

After approval, extend `knowledge/{raw,types,identifier,loader,validate,problem_family,
objective}.rs`, add the pattern parser and named mutation fixtures under existing
`knowledge/tests/`, and update architecture's folder inventory for actual files added.
Keep unsupported schema-2 rejection covered in the old-loader compatibility fixture.
Provider manifests advertise new capability versions only after DTO, store, command,
service and hook implementations exist together. No production content is upgraded merely
to satisfy a nonempty UI state.

Future tests must round-trip every new field with provenance/version/digest, reject each
invalid/reference case above, pin eligible/ineligible parameter cases, and verify unsupported
detectors produce unknown rather than clean. Test method-selection evidence against final
answer alone, objective scope vs contextual objectives, unchanged v1 loading and identical
seed-to-instance generation. Exercise package updates/removal across restart with old
snapshots still readable and no retroactive diagnostic reassignment. Include these cases
in the provider conformance suite and task 067's permanent never-deleted regression corpus.
Task 068 must still prove the real generate/check/hint/advance loop with external networking
disabled; static package validation is not that proof.
