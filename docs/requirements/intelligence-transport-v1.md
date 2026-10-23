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
This version names that fixed UTF-8/JCS record codec `intelligence-jcs-v1` in
the selected processing profile. An unknown codec name remains unsupported;
its presence or a valid profile digest cannot enable a fallback.
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
Use these declared transport limits. The separate source-JSON selection
profile's narrower node quotas do not apply to this record codec.

Enforce the selected profile's message/record/collection limits and complete
combined headroom. Check whole UTF-8/JCS values, Unicode, duplicate keys, numbers
and collections; exact input bytes must equal checked canonical bytes.
Validate each fixed target and all semantic joins. Preserve public closed-core
and bounded-additive rules. Bulk bytes use immutable authorized blob references;
credentials, original media and reusable signed URLs have no transport slot.
Unsupported codec, resource or billing envelopes remain explicitly unsupported.

## Independent observations

Complete-message conformance compares supplied records with independent
Rust-admitted state. The Rust side of the verifier supplies and retains the exact
admitted scope, source/profile/plan identities, execution/control snapshot,
allocations, accepted dependencies, worker attribution and fence/cancellation
observations. Dispatch and response checking have separately retained clock
observations. Message echoes and profile-mode or receipt-kind labels do not
authenticate any of these facts.

At the response-check checkpoint, Rust independently obtains and retains the
canonical observations for returned representations and observations. Each binds
the exact allocated identity/reference, complete immutable JCS bytes and server
recorded_at to the admitted context and bound allocation. The processor response
cannot supply its own trusted observation. Allocation grants no creation,
timestamp or reuse authority; any reuse needs its owning Rust admission evidence.
The public server/canonical meanings of recorded_at remain unchanged.

Returned records must equal those observed complete bytes. Missing or conflicting
observations fail this conformance comparison. Do not substitute a processor
clock, restamp a record, or reconstruct different record bytes after result
hashing. Preserve the fixed targets, public preimages, result-core digest and
exact retained-result/protobuf output agreement.

Literal observations used for conformance are explicitly synthetic inputs and
prove agreement only with the supplied observations. They qualify no actual
Rust issuance, clock, authorization or persistence. Before the first real
consumer, qualify the fenced Rust issuance boundary, authoritative clock,
ownership/cancellation races and producer ordering that supply final immutable
records before their result bytes and digest are frozen. This comparison neither
defines an additional service nor grants a canonical writer permission.

### Complete comparison facts for observation provenance

Comparison-input objects and collections must have their declared JSON shapes.
Integer facts remain integers: booleans and floating-point values do not stand
for ordinals, fences, indices, counts or dimensions. Structured comparisons
preserve scalar types as well as values; canonical JCS spelling alone cannot
repair a mistyped independent fact. Malformed comparison inputs refuse through
the static message result, while missing required fixture/tool files remain
fatal outside that invalid-value comparison.

A current model inference uses the selected creation profile. Its ordered basis
resolves exact complete immutable observations from returned outputs, accepted
dependencies or Rust's additional basis_observation_records. Each additional row
supplies scope, reference and canonical_jcs. Only a retained model_inference row
also requires creation_profile_digest; that field is absent for other kinds,
including user assertions without a processing-profile association. canonical_jcs
is an opaque supplied-material key, not a path the comparator opens. Rust obtains
those exact bytes through the separately qualified current-authorized read.

The selected max_context_records separately bounds both the number of additional
basis rows and the distinct reachable basis-target union across current, prior
and additional observations. The source context's existing count bound remains
separate. Count an immutable identity once in the reachable union even when
present in multiple sources; conflicting complete bytes refuse. Current inference
roots count only if another reachable basis names them. All reachable basis
records must be supplied, and the graph must be acyclic; self-edges refuse.
Each complete record obeys max_record_bytes/max_json_depth. The comparison-input
encoded footprint, observations_json plus supplied material bytes counted once
per opaque material key, must fit max_memory_bytes. This byte preflight does not
qualify actual process RSS, decoded media memory or worker containment.
Missing, foreign or substituted facts refuse.
A prior or externally read inference retains its own admitted creation profile;
it does not acquire the current stage's profile. The empty basis remains explicit.

User assertions require an independently supplied assertion_attributions row
with scope, audit_event_id, capture, source, principal, role, recorded_at and
supplied_sha256. source is the exact assertion-source value; principal uses the
existing context principal_reference. role is user_supplied_assertion. The row
binds the exact immutable capture revision and the genuine source-supplier audit
association, not necessarily the later capture submitter. Its canonical audit
time cannot follow the resulting observation's canonical recorded_at.
For an assertion about the exact capture already carried by SourceInput, use
that complete capture only after matching the full immutable capture reference.
An attribution for another same-workspace capture additionally supplies
capture_snapshot: the complete acquisition.capture_resource JSON object,
not a new capture_view target or a current-head substitution. Its revision must
match the assertion's exact capture reference, and its workspace must match
the independent scope. When a supplied snapshot names the current capture,
its immutable revision must also equal the admitted SourceInput revision.
Each snapshot obeys max_record_bytes/max_json_depth; its encoded bytes are
included in the existing observation/material footprint bound. Relevant
historical representation records/material and geometry/alignment remain
independently supplied; do not force them into the current capture's blob set.
Missing or substituted historical source facts refuse. This snapshot grants
no actual read, audit, supplier or human authority.

For capture_context, supplied_sha256 identifies the exact captured user_context
UTF-8 bytes, and grounding stays context_only. For supplied_representation, it
identifies the exact supplied representation bytes; that representation's
original must resolve through a ready sealed user_note attachment in that capture.
Its locators must use that same supplied representation. An evidence attachment
or a claimed role/audit ID alone establishes no supplier assertion.

These rows are independent recorded comparison facts, not new public audit
responses or authentication proofs. Their absence supplies no implicit authority.
Synthetic fixtures qualify agreement only. The actual Rust read, creation-profile
association, audit/role/principal attribution and producer ordering remain
mandatory real-consumer gates, without substituting human or live evidence.

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
max_output_records bounds returned typed records; unused prospective identities
retain their separate allocation-array schema bounds.
Accepted dependency results cannot be overwritten through another output.

CandidateProposalRecord carries a complete candidate_reference and unchanged
candidate_content. ExpenseProposalRecord carries a complete expense_reference
and unchanged financial_content. These are proposed initial references: content
revision kind content, value 1. Their content_sha256 retains the complete owning
public preimage. Candidate identity uses the exact context and normalized content
inside the four-field candidate_content envelope defined by the
[candidate contract](candidate-review-contract.md#canonical-candidate-identity).
Expense identity uses the exact organization/domain and normalized content inside
the arboresce.financial-content.v1 envelope, including its 131,072-byte bound,
defined by the [financial contract](financial-records.md#canonical-financial-identity).
Normalize only the declared evidence, binding and basis sets after rejecting
duplicate identities. Retain all other values and array order. Bare content
hashes do not identify either proposal family.
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

A declared pending-item DAG may name exact other returned items, exact available
prior-result items or supplied committed-record snapshots. An accepted stage
result grants no proposal review acceptance or accepted mapping. Actual review
eligibility, current authorization and final dependency/version fences remain
with the project workflow's canonical transaction owner.

## Plans, attempts and deadlines

The complete OperationExecution snapshot joins the immutable plan, original
operation/stage timing, declared stages, actual attempts and worker attribution.
Binding/deadlines echo the Rust-admitted operation, plan, stage, attempt,
input/profile, result ID and worker fence. Compare exact kind, ordinal,
dependencies and output kind. Attempts do not replace accepted-result uniqueness
by organization/operation/stage/input/profile. No retry, response or poll renews
a fence, counter or fixed deadline. Check UTC arithmetic and remaining monotonic
bounds. Actual clocks, leases and authenticated workers retain their runtime gates.

remaining_attempt_ms is sampled by Rust at the dispatch checkpoint, using an
independently retained paired UTC/monotonic observation. It is the whole
millisecond floor of the smaller of the nonnegative monotonic attempt remainder
and the time to the fixed UTC attempt_deadline. Never round up. Unknown,
unrepresentable or expired observations deny new dispatch; a zero floor supplies
no new work authority. The message must equal that retained dispatch value.
Response checking uses its own current observations against the original fixed
deadlines; echoing or resampling a larger value cannot renew the budget. Qualify
the actual clock adapter and its observation pairing at the real consumer.

Attempt ordinals use the operation-wide scope defined by the
[lifecycle owner](lifecycle-contract.md#3-shared-operation-plan-clocks-and-evidence).
The selected attempt's ordinal is distinct from a stage's matching-attempt count.
Declared output kind/context select the output family; this contract supplies
no additional universal stage-kind/output-kind or failure-code/class matrix.

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

The plan-selected processing profile's maximum_reserved_usd is the fixed
whole-operation accounting allowance for this comparison. It is not renewed by
another stage, attempt, invocation or provider profile. Join it to the exact
admitted plan/profile; a processor cannot select a replacement allowance.
Across the complete operation, count each exposure identity once. A verified
settled known receipt contributes its exact amount_usd; otherwise retain the
original reservation or the greater authenticated retained liability. Neither
completed work nor a known-but-unsettled receipt releases that reservation.
Check both the complete prior snapshot and its union with the current attempt.
Keep original requests, receipts and contradictory evidence unchanged.
A verified settled actual amount may exceed its original estimate; compare that
actual amount with the operation allowance without discarding truthful evidence.

This operation comparison does not establish shared per-capture admission.
The actual Rust/gateway owner must reserve and reconcile all work sharing that
capture, including other operations, under the existing USD 1 qualification
ceiling and shared policy/slot limits. An operation or profile change cannot
reset that ceiling. Qualification of those authoritative joins and atomic claims
remains mandatory before any real invocation.

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
