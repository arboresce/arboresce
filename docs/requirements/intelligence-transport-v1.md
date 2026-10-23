# Intelligence stage transport v1

Status: implementation candidate. Independent input, validator, generation and
source qualification remain required. The [processing owner](processing.md)
retains pipeline behavior; this contract enables no service or canonical writer.

## Canonical sources and fixed codecs

[processing.proto](../../contracts/internal/proto/arboresce/intelligence/v1/processing.proto)
owns framing. The committed descriptor supports dynamic Python conformance
decoding. Descriptor reproduction selects protoc 36.2 and the pure Python
protobuf 7.36.2 distribution under CPython 3.14.7, with their BSD 3-Clause
licenses. The [generation contract](../../contracts/internal/proto/intelligence-generation-v1.json)
pins the complete descriptor recipe, compiler, libraries and runtime.
The separate [framing conformance project](../../contracts/intelligence-validation/README.md)
owns that Python dependency and its combined source qualification profile.
The [generation guidance](../../contracts/internal/proto/README.md) separates
this descriptor promise from the still-required reproducible Rust/Python service
bindings at their first consumers. Neither descriptor generation nor dynamic
conformance decoding qualifies a running gRPC service.

The [internal library](../../contracts/internal/v1/intelligence.schema.json)
owns complete record codecs. Existing public schemas remain canonical. Each
wrapper carries complete JCS bytes for exactly one fixed target:

| Wrapper | Complete target |
| --- | --- |
| SourceInput | intelligence.source_input |
| ProcessingProfile | intelligence.processing_profile_record |
| OperationPlan | lifecycle.execution_plan_record |
| PriorStageResult | intelligence.stage_result_record |
| OperationExecution | lifecycle.operation_execution |
| CostQuoteRecord | intelligence.cost_quote |
| PriorExposureLedger | intelligence.prior_exposure_ledger |
| RepresentationRecord | acquisition.representation |
| ObservationRecord | observation.observation |
| ProjectChangeItemRecord | project-workflow.project_change_item |
| CandidateProposalRecord | intelligence.candidate_proposal |
| ExpenseProposalRecord | intelligence.expense_proposal |
| SuccessfulStage.complete_result_jcs | intelligence.stage_result_record |
| FailedStage.complete_failure_jcs | intelligence.stage_failure |

No caller-selected schema URI, generic JSON or Any fallback exists. A fixed
wrapper does not permit partial record decoding or an alternate public preimage.
Expense content supplies no confirmation/voiding authority. Project output cannot
manufacture an accepted resource, mapping or second approval ledger.

## Frames, depth and limits

The version is exactly arboresce.intelligence.v1. Reject unknown fields/enums,
unspecified sentinels, absent consequential values, duplicate singular/oneof
members, overlong integers, malformed lengths and unsupported wire types.
Stage ordinal zero requires explicit presence; stage ordinals are 0..63,
attempt ordinals 1..256 and worker fences positive safe integers. Field ordering
alone does not change protobuf meaning; protobuf serialization is not a canonical
content/core preimage.

Reject a complete frame above 1 MiB before decoding. Protobuf message nesting
has a hard limit of sixteen messages, with the outer message at depth one.
Before decoding a JCS record, enforce the hard sixty-four-container JSON depth
ceiling. Objects and arrays each add one; a root scalar has depth zero.
The mandatory profile field max_json_depth selects a narrower limit from 1..64,
and every complete record, including the profile itself, must fit that selection.
Missing limits or a profile too small to hold its own required records fail;
there is no library-default fallback.

Enforce the selected profile's message/record/collection limits and complete
combined headroom. Check whole UTF-8/JCS values, Unicode, duplicate keys, numbers
and collections; exact input bytes must equal checked canonical bytes.
Validate each fixed target and all semantic joins. Preserve public closed-core
and bounded-additive rules. Bulk bytes use immutable authorized blob references;
credentials, original media and reusable signed URLs have no transport slot.
Unsupported codec, resource or billing envelopes remain explicitly unsupported.

## Source and context selection

Source input retains the submitted capture's exact ordered attachments, roles,
requiredness, analysis input, immutable submission cause, representations, blob
references, earlier dependency declarations and stage-specific output allocations.
Its context has one explicit branch:

- workspace supplies bounded complete authorized asset snapshots and invents no
  project, project origin or event date.
- project supplies the exact project reference and bounded complete project
  records. Its proposal_origin is explicitly null until a consequential project
  proposal is requested; normalization, extraction, transcription and abstention
  do not manufacture proposal temporal metadata.

Project-change outputs require a nonnull exact proposal_origin and retain all
existing temporal-basis, source/context/default/inference, uncertainty and lineage
rules. Workspace input preserves retained non-project knowledge and financial
processing. Selecting a branch grants no membership, resource or model authority.

Exact source JCS bytes define input_sha256. Dependency declarations are known
at admission; supplied complete accepted prior results must match the plan's
dependencies and the execution snapshot. The source preimage contains no future
result or its own plan digest. Plan SHA256(JCS(plan.core)), profile
SHA256(JCS(profile.core)) and result SHA256(JCS(result.core)) remain distinct.

## Output identities and families

output_identities.stage_id must equal the selected binding stage. Rust freezes
the actual allocation for that stage before dispatch. All allocated IDs and
project item/candidate pairs are distinct in that allocation; an output may use
an allocated identity once, without positional assignment. Unused allocations
remain unused and do not establish canonical creation or a reuse permission.
Accepted dependency results cannot be overwritten through another output.

CandidateProposalRecord carries a complete candidate_reference and unchanged
candidate_content. ExpenseProposalRecord carries a complete expense_reference
and unchanged financial_content. These are proposed initial references: content
revision kind content, value 1, and content_sha256 equals SHA256(JCS(content)).
Scope and the explicit own ID must match the source and the corresponding
allocated candidate/expense ID. Neither envelope supplies recorded_at, audit,
acceptance, confirmation or other server-owned fields.

Project-change output instead joins the allocated item_id/candidate_id pair.
Its candidate reference and complete change preimage retain their existing
project-workflow revision, digest, target, source and lineage semantics.
A project target's absence predicate remains different from an exact revision.

| Declared output kind | Permitted typed records |
| --- | --- |
| representation | representation only |
| observation_set | observation only |
| candidate_set with workspace context | candidate proposal only |
| candidate_set with project context | project change only |
| expense_proposal_set | expense proposal only |

Validate exact representations/locators, source/profile/target identity,
same-workspace acyclic dependencies, unknowns, disagreement and all four lineage
kinds. Unrelated project changes do not invalidate every item. Complete retained
result outputs and protobuf outputs must agree in exact order and content.
Empty outputs require explicit admissible abstention, never hidden failure.

## Plans, attempts and deadlines

The complete OperationExecution snapshot joins the immutable plan, original
operation/stage timing, declared stages, actual attempts and worker attribution.
Binding/deadlines echo the Rust-admitted operation, plan, stage, attempt,
input/profile, result ID and worker fence. Compare exact kind, ordinal,
dependencies and output kind. Attempts do not replace accepted-result uniqueness
by organization/operation/stage/input/profile. No retry, response or poll renews
a fence, counter or fixed deadline. Check UTC arithmetic and remaining monotonic
bounds. Actual clocks, leases and authenticated workers retain their runtime gates.

## Admitted quotes and receipts

StageRequest carries the exact finite set of CostQuoteRecord values admitted
by Rust for new dispatch from this stage attempt. IDs are unique. A
provider_request includes its complete selected quote before dispatch; it must
equal an admitted quote, not merely share its profile/policy. Its request digest
is SHA256(JCS(the complete provider_request)), including that quote. Quote
identity is its quote_id together with SHA256(JCS(the complete cost_quote)).
This does not create a self-referential profile preimage: quotes refer to the
already-frozen profile and are outside profile.core and source input.

known_cost.quote must equal that request's selected quote. Match scope,
profile/policy, request identity, input digest and validity at actual admission.
USD amounts are reduced nonnegative rational fractions with canonical integer
strings and positive denominators; zero has denominator 1. Known cost equals
the exact quoted input/output prices times actual units, plus request price.
The current quote selects token units only; it implies no page, duration or
other OCR/ASR price profile. Unknown outcomes retain reserved liability.
Timeout, cancellation, silence and expiry prove neither zero cost nor
not-performed. USD 1 remains a qualification ceiling, without spending authority.

Every provider_receipt has its own receipt_id, separate from observation_id.
The lifecycle entry's retained receipt_id must match it, and receipt_sha256 is
SHA256(JCS(the complete provider_receipt)), including its evidence and ID.
No receipt digest is embedded in its own preimage. Join issuer, exposure, scope,
operation/stage/attempt, policy/profile, request ID, request digest and outcome
to the complete request and observation. A receipt retains its complete gateway
request and performed/not-performed/unknown observation.

The real service boundary authenticates its issuer independently of message
claims. Synthetic conformance is distinct from a genuine gateway observation
and cannot qualify one. A caller's profile or receipt flag grants no trust.

## Complete exposure scope and retained history

StageRequest.prior_exposures is Rust's complete retained operation exposure
snapshot at its stated control revision, with complete request/receipt bodies
when present. It includes failed and uncertain attempts and other stages;
accepted prior-stage results are not a substitute. Its operation/scope, stage
and attempt identities must join the plan/execution snapshot. Historical quote
validity is checked at its original admission, not at a later retry.

A result or failure's exposure_records.kind is attempt_complete: exactly every
admitted external invocation for the bound stage and attempt, including earlier
uncertain work within that attempt. Scope, operation, stage_id and attempt_id
are explicit. This is not a declaration that the operation has no earlier
exposure. All history entries for that current attempt remain accounted for;
an empty result cannot erase them.

Rust retains the prior snapshot independently and reconciles returned records
with the actual authenticated append-only gateway/operation ledger before
acceptance. A processor cannot erase another attempt, rewrite an existing
identity's bytes or assert operation completeness from its local list.
Immutable replacements based on new authenticated evidence retain earlier
contradictory evidence under the lifecycle reconciliation contract; this stage
response supplies no authority to replace it. Keep uncertain liability and the
fixed reconciliation deadline, with no return to attempted or settled-to-reserved
transition. Complete operation projections still require the entire actual ledger,
including entries absent from the current processor's work.

## Remaining consuming gates

Independent complete examples precede semantic consumption. Reproduce descriptor
generation in independent fresh directories and compare every promised output.
Reproducible Rust/Python service bindings, authentication, maintained helper and
actual process coverage, strict required metrics and adapter resource containment
remain mandatory at their first consumers.

Conformance does not qualify actual service authority, current authorization,
live models, canonical storage, recovery or effects. A qualified-processing
declaration requires independently qualified resources and authority at its
real adapter. All later operational and production requirements remain required.
