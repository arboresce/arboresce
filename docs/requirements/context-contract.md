# Context builds, evaluations and attestations v1 contract

**Authority:** normative exact context identity, complete delivery, evaluation,
authenticated build approval, signature/trust and channel-transition contracts.
**Status:** specified contract; product runtime, current identity/trust services,
canonicalization, transactions and complete-transfer qualification remain required
at their owning implementation checkpoints. Parsed-value fixtures and bounded
fixed signing vectors do not qualify those implementations.

The [context schema](../../contracts/public/v1/context.schema.json) exposes named
v1 definitions and rejects every instance at its library root. The
[standalone validation lane](../../contracts/validation/README.md) checks independent
parsed-value cases and the separately frozen signing-vector structure. The
[common contract](common-contract.md) owns primitive encodings, raw admission,
command/idempotency and response behavior; [candidate review](candidate-review-contract.md)
owns accepted asset mappings/content; [observations](observation-contract.md) own
source kinds and grounding. Those exact definitions are referenced without changing
their meaning or treating additive transport extensions as canonical content.

[Data model](data-model.md), [API](api.md), [security](security-and-privacy.md),
[resource profiles](resource-profiles.md), [dependency qualification](dependency-qualification.md)
and [testing and coverage](testing-and-coverage.md) retain their authority. This
contract freezes their context representations. [Acceptance](acceptance.md) and
the [rolling plan](../execution/open-cli-mvp-v1-rcld.md) retain all later runtime,
service, native and operational requirements. Financial confirmation is independent
of knowledge publication and has no dependency on context approval.

## Exact reusable identities

All shapes below are closed unless expressly described as outer response readers. IDs, hashes, timestamps, revisions and raw unsigned-integer admission reuse the common owner. No financial-family import is needed.

- `build_reference = {organization_id:identifier, domain_id:identifier, build_digest:sha256}`. The build is already content-addressed; no additional mutable build UUID is needed. Tenant/domain/path equality and current access are semantic checks.
- Input `reference` is exactly E005 `accepted_asset_reference`: organization, asset ID, version ID, candidate reference and review ID. Its nested candidate pins intended-use context, candidate ID, content revision and content SHA-256. Never weaken this to an asset/head alias.
- `asset_snapshot = {reference:accepted_asset_reference, content:candidate_content, observations:observation_core[0..192]}`. `observation_core` is a closed projection of E004's seven required fields: observation ID, organization ID, recorded time, kind, statement, source and grounding. It retains the existing three kind/source/grounding variants and their constraints; it does not hash arbitrary additive transport extensions.
- `compiled_identity = {sha256, byte_length:integer 1..262144, media_type:'application/json', encoding:'utf-8'}`. This is the identity of the exact compiled UTF-8 document, not the outer HTTP response or an approval.

The canonical accepted-review mapping must exist and match all supplied references. Recompute the candidate fingerprint from its complete normalized content before binding the snapshot. Retain the exact mapping ID in content; exclude review receipt bodies, human approval events and build evaluation/approval/signature receipts. The already immutable source observation's `recorded_at` is retained provenance, distinct from the excluded build execution/attempt timestamp.

The observations array is exactly the union of supporting, refuting and qualifying references in this asset's candidate content. It contains each direct observation once even if cited in more than one role. Every reference must resolve to that exact immutable core; missing, contradictory, unauthorized or unavailable required evidence fails the whole build. The same observation repeated across different asset snapshots must have equal bytes. Its repeated appearance is meaningful per-asset provenance, not grounds to erase a role or snapshot.

Inference basis, source originals and representations retain their exact existing references. Do not recursively inline observation ancestry or original media. Current source-use restrictions still traverse the canonical dependency closure; omission from an inline representation cannot remove an underlying restriction. Unsupported dependency closure, missing policy or a required unavailable source denies admission. This fixed projection is not an evidence summarizer.

## Selection and normalization

`inputs` has 1–100 exact accepted-asset references. Reject duplicate `(organization_id, asset_id, version_id)` identities, including contradictory references to the same version. Multiple exact versions of one asset are permitted when each is independently currently eligible. Historical versions require explicit current eligibility; there is no implicit newest-version selection.

The schema rejects identical duplicate values in set-valued inputs, assets,
observation unions, suite cases, evaluation results, evaluation-reference sets
and usage asset references. Different values claiming the same identity still
require semantic rejection, as do incomplete unions and inconsistent joins.
This rule does not make compiled chunks a set or change the inherited rules
for meaningful prose, evidence-role and locator ordering.

Every asset and observation must belong to the target organization. Input candidate domains may differ from the target build domain only under explicit current cross-domain source-use policy for the requested purpose; the mere presence of references is not that grant. Preserve each candidate's original context and intended purpose. Build admission purpose and later consumer purpose are separate current checks; changing consumer purpose does not rewrite the build.

Sort assets by ASCII `(organization_id, asset_id, version_id)` after duplicate rejection. Normalize each candidate evidence-role set by the E005 `(organization_id, observation_id)` rule. Sort each asset's observation union by that same ASCII pair. Preserve scope/prose list order, statement text, applicability, assumptions, qualifications, invalidation conditions, review due, locator order and all other semantically ordered arrays. No trimming, Unicode normalization, heading rewrites, inference, evidence-role reassignment or hidden defaults occur. A different normalized consequential value changes identity; a permitted set permutation does not.

Availability remains mutable state outside the snapshot. The compiler cannot freeze a boolean that declares a source forever eligible. Deprecated versions require explicit current trusted policy permitting that exact version and purpose; there is no caller allow-deprecated flag. Revocation/purge and missing required source authority always deny. Non-head or active status alone cannot establish eligibility; no silent fallback or newer-version substitution is permitted.

## Fixed compiler and canonical bytes

The first compiler has one complete configuration and no arbitrary template, executable command, provider, prompt or model settings:

```text
compiler_profile = {
  name: 'context_jcs_v1',
  version: '1',
  configuration: {
    input_order: 'asset_version_ascii_v1',
    observation_order: 'organization_observation_ascii_v1',
    content: 'complete_candidate_and_direct_observations_v1'
  }
}
compiler_descriptor = {profile: compiler_profile, profile_sha256: sha256}
```

Every shown configuration field is required and constant. This intentionally leaves no hidden default or approximate general number in the first configuration. Define `profile_sha256 = SHA256(JCS({kind:'arboresce.context-compiler.v1', profile:compiler_profile}))`. The digest field is not in that profile preimage. A later compiler/configuration change requires a distinct qualified profile, not reuse of this name/version with changed behavior.

The compiler creates exactly this closed document after source identity and eligibility checks:

```text
context_document = {
  kind: 'arboresce.context-document.v1',
  contract_version: 'v1',
  organization_id, domain_id,
  compiler: compiler_descriptor,
  assets: normalized asset_snapshot[1..100]
}
compiled_bytes = JCS(context_document)
```

JCS output is UTF-8 without BOM or final newline. Its field escaping and UTF-16 object-key order are those of RFC 8785. This JSON text is the complete compiled representation supplied to applications; it retains all candidate fields, direct evidence statements, source kinds and grounding. It is not a generated narrative, token-based excerpt or Markdown rewrite. Renderers may present a view, but a modified view cannot claim the exact compiled digest.

Count actual encoded bytes while compiling. At 262144 bytes the full output may be admitted; at 262145 reject the whole build without publishing a result. Every selected asset still counts toward 100 even if content happens to match. Per-field maxima do not guarantee a selection fits the aggregate bound. Stop bounded work on a decisive size failure; do not publish the already produced prefix or remove evidence to fit.

The immutable manifest is flattened to avoid unnecessary control-container depth:

```text
build_manifest = {
  kind: 'arboresce.context-build.v1',
  contract_version: 'v1',
  organization_id, domain_id,
  compiler: compiler_descriptor,
  assets: normalized asset_snapshot[1..100],
  compiled: compiled_identity
}
build_digest = SHA256(JCS(build_manifest))
```

Compiled identity must match `compiled_bytes`; manifest organization/domain, profile and assets are exactly the document values. The manifest contains neither its own build digest nor execution timestamps, attempts, evaluations, approvals, signatures, current policy/grants or availability. The compiler profile, complete inputs and bytes determine identity. Build/compiled/profile digests have different preimages; none may substitute for another.

Apply common raw JSON admission, individual scalar/container bounds and depth rules. Every inline observation/candidate scalar already has an appropriate source-contract bound. The manifest's canonical bytes get an explicit 524288-byte cap in addition to the compiled-text cap: its complete content is almost identical to the document, with only a different fixed kind and small compiled-identity object, so this does not narrow a valid 262144-byte compiled document. GET response composition must remain within common depth 16 and 1 MiB; operation completion returns a build reference rather than adding another deeply nested full manifest.

Fetch current canonical references in bounded batches and copy/compile/seal outside database transactions. Seal compiled bytes and immutable input edges before binding the canonical manifest/result. Final binding uses short current-authority and input-control checks with fenced reservations; a changed or revoked source requires revalidation/failure, not stale publication. Record orphan cleanup ownership if a later transaction fails. The canonical uniqueness key includes organization and build digest. A matching foreign digest cannot reveal existence; identical bytes do not merge tenant authority.

## Complete delivery within the existing control envelope

`compiled_content = {encoding:'utf8_chunks_v1', chunks:string[1..5]}`. Each string is nonempty and at most 65536 Unicode scalars. The semantic encoding additionally caps each decoded chunk at 65536 UTF-8 bytes. Chunk greedily from left to right at the largest complete Unicode-scalar boundary that fits that byte cap. Every nonfinal chunk is therefore at least 65533 bytes; five chunks suffice for a full 262144-byte document, including the case where four boundaries leave a small fifth chunk. Empty intermediate chunks, different cut points, invalid UTF-8 and byte/count mismatches fail. This is a canonical transport segmentation, not independent source text pieces.

Concatenating the decoded strings without delimiters produces exactly `compiled_bytes`. Require total byte length and SHA-256 equality before the consumer presents the result as a verified exact build or submits it downstream. Outer JSON member order/whitespace are not compiled identity. Reordering chunks, adding a newline, reserializing a parsed document differently or silently taking only the first chunk changes the bytes and fails.

The complete resolve response does not duplicate the whole manifest. It carries
the build reference, compiled identity, complete chunks and usage receipt
containing exact selected references. The producer's outer JSON serializer must
emit each compiled chunk's non-control Unicode scalars as literal UTF-8, except
for mandatory quotation-mark and backslash escapes. It must not introduce HTML
escaping, optional solidus escaping or ASCII-only Unicode escapes. Canonical
JSON source text has no raw control bytes; under this fixed outer encoding,
embedding adds at most one byte for each quote/backslash. Content expansion is
therefore at most twice the compiled byte count plus chunk framing. Exact
references are ASCII and bounded. The selected representation supports the full
compiled envelope under 1 MiB; still enforce the actual complete encoded response
cap, including bounded producer fields. Qualification must exercise near-limit
literal HTML punctuation, multibyte text and quote/backslash-heavy content using
the actual response serializer. No compressed expansion, presigned object URL,
redirect, browser token, query selection, truncation flag or fallback partial
body is introduced.

Materialize and verify the complete bounded response before its final admission/exposure check. Recheck current principal, purpose, source restrictions, build availability and required eligibility receipts. Each admitted response is one bounded proxy-authorized transfer under the existing ten-second request deadline; no deadline renewal on retry. `transfer_expires_at` is no later than admission plus ten seconds, the remaining request deadline or the applicable authorization expiry. A server deadline/cancellation may produce an incomplete transport, but the client must not promote partial bytes as successful exact content. Already delivered bytes are not recallable; later admissions and new exposures recheck current restrictions.

## Token admission and usage identity

Use a required trusted `tokenizer_reference`, a named alias of the context-local organization/domain-scoped `profile_reference` defined below. It identifies an immutable registered tokenizer profile whose complete implementation/artifact/configuration bytes and deterministic content-only behavior are qualified separately. The reference is not a caller-selected model name, URL or instruction. Missing, unknown, unsupported or unqualified profiles deny token admission; no byte-based token estimate or ambient model fallback is permitted.

`resolve_limits = {max_compiled_bytes:integer 1..262144, tokenizer:tokenizer_reference, max_tokens:integer 1..1048576}`. The token ceiling is selected wire capacity, not a model's context-window promise. Count the exact entire compiled text with the declared profile, including its declared special-token policy; freeze that policy in the profile, not caller options. `token_admission = {limits:resolve_limits, actual_tokens:integer 1..1048576}`. Actual bytes/count must be within both caller limits and the qualified profile's own bounds. Prompt framing, other messages and model-output reservations remain separate model-call admission; a successful resolution is not proof that a larger downstream prompt fits. Changing tokenizer or caller limits changes usage evidence, not immutable build content.

The usage receipt uses the context-local `principal_reference` and `policy_reference` rather than importing financial types:

```text
usage_receipt = {
  usage_id: identifier,
  build: build_reference,
  compiled: compiled_identity,
  asset_versions: accepted_asset_reference[1..100],
  purpose: common.slug,
  executor: principal_reference,
  delegation_id: identifier | null,
  policy: policy_reference,
  authority_audit_event_id: identifier,
  token_admission: token_admission,
  admitted_at: timestamp,
  transfer_expires_at: timestamp,
  delivery: 'admitted'
}
```

The exact normalized selected references equal the build's input references; usage cannot rename a different selection under a whole-build signature. Policy identity binds immutable policy content, and the authority audit event binds the actual current membership/delegation/source decision. Neither field alone is a current authorization grant. The actual executor is server-derived, never a client authority claim. This receipt records admission for the complete identity; it does not claim downstream receipt, use, factual correctness or model success. The integration decisions define a nullable exact channel observation for channel resolution; it never changes the build preimage.

## Build and resolution operations

- Build creation: closed `{metadata:command_metadata(command='context.build'), body:{inputs:accepted_asset_reference[1..100], compiler:compiler_descriptor}}`, `POST /v1/context-builds`. Metadata organization/domain select the target; all supplied compiler fields must equal the one supported profile. Durable admission returns the existing common202 operation form; the operation eventually carries the exact build reference with its terminal receipt on success. Seal/bind failure never returns completed content. Completion polling should link to the full resource rather than nesting its full manifest under an operation result.
- Exact build resource: authorized `GET /v1/context-builds/{build_digest}` with the owning context, returning a bounded common success result with reference, manifest and separately observed availability/control. Current controls and receipt collections do not enter manifest identity. The final resource composition is defined in the integration decisions below.
- Exact resolution: read-only `POST /v1/context/resolve` with the closed `{context:command_context, selection:resolve_selection, limits:resolve_limits}` variants defined in the integration decisions. It has no query/search field, client actor, idempotency key or approval flag. Each actual read exposure records a new usage receipt after current admission; it does not mutate build content or channels. Return common completed success with `result:{build, compiled, content:compiled_content, usage:usage_receipt}` and no mutating-command idempotency receipt.

This operation avoids conflating a short transfer authorization with a seven-day mutating-command replay. Retrying resolution reauthorizes and creates a distinct observation of admission, while exact content remains unchanged. Context/reference/channel-selection equality, current permission, token arithmetic and delivered-byte identity are semantic checks.

## Minimal independent fixture targets and semantic oracles

Parsed targets: `build_reference`, `compiler_profile`, `compiler_descriptor`, `observation_core` (all three variants), `asset_snapshot`, `compiled_identity`, `context_document`, `build_manifest`, `compiled_content`, `tokenizer_reference`, `resolve_limits`, `token_admission`, `usage_receipt`, complete build/resolve request variants and each selected resource/response composition. Reuse existing field constraints; retain bounded additive outer response readers while closing every canonical payload/request. Reject reserved approval/authority/outcome aliases.

Independent semantic vectors must include: accepted mapping mismatch; changed candidate content with old digest; missing/refuting/qualifying observation or extra/duplicate union member; cross-role same-source success; mismatched same-ID observation cores; source-kind preservation; foreign/currently revoked source; explicit cross-domain policy versus absent policy; multiple eligible versions of one asset success versus duplicate exact version rejection; normalized set permutations versus meaningful prose/locator order; scope-only fingerprint change; immutable source time versus ignored build execution time; exact compiler/profile/document/manifest preimages with no self-hash; boundary 100/101 inputs and 262144/262145 compiled bytes; four full ASCII chunks versus valid five-chunk multibyte boundary; reordered/noncanonical chunks and incomplete delivery; exact tokenizer-profile/count mismatch; unknown profile and raw integer aliases; current authority/revocation race; lost response and fresh read admission; no partial result when the last required source fails.

Parsed schemas cannot compute hashes, validate canonical sorting/equality joins, enforce byte/depth/raw-token limits, establish current trust or qualify tokenization/stream delivery. Those remain explicit pending E011/domain/service/qualification cases. No fixture success may be reported as their runtime evidence.


## Shared types and immutable object rules

All objects described below are closed canonical values. Ordinary response
envelopes retain common bounded additive-reader behavior; extensions never enter
a content projection, fingerprint, authority decision or signed preimage.
Common identifiers, SHA-256 values, UTC millisecond timestamps, positive tagged
content revisions and command context keep their existing definitions. Every new
ID is server-issued. A digest or principal-kind label is not an authorization
capability.

Use these definitions locally in the E007 context family:

```text
build_reference = {organization_id, domain_id, build_digest}
principal_reference = {principal_id, kind: human | agent | service}
policy_reference = {
  organization_id, domain_id, policy_id,
  version: content_revision, content_sha256
}
execution_attribution = {
  executor: principal_reference,
  delegation_id: identifier | null,
  authority_audit_event_id: identifier,
  command_id: identifier
}
profile_reference = {
  organization_id, domain_id, profile_id,
  version: content_revision, content_sha256
}
```

These are authenticated server assertions, not fields a request can submit to
assert authority. Principal ID resolves the canonical issuer/subject identity;
do not expose access tokens, issuer credentials or arbitrary claims. The audit
event names immutable actual admission evidence, including current membership,
delegation-chain and policy revisions. Audit IDs are allocated by the server and
become authoritative only with their committed audit record. The complete chain
is checked canonically; a single delegation ID is its reference, not permission
to omit intermediate issuers. A direct authenticated executor has null delegation.

The build's organization/domain must equal command context and route scope.
Approval and evaluation purpose is the exact common context purpose. It remains
outside build identity. Policy/profile IDs bind their complete registered immutable
bytes and version; a hash or a recognizable profile name does not make a profile
trusted, supported or currently applicable. Unknown or unqualified profiles deny
the consuming operation. The same policy encoding may be used by channel and
read-only usage receipts. A read-only usage receipt has no invented command ID.

For each record below, `record_sha256 = SHA256(JCS(record_core))`. Each closed core
has the explicitly named versioned `kind`; its own hash and outer transport fields
are excluded. A reference binds its organization/domain, ID and exact record hash.
IDs, content hashes and mutable availability controls are different identities.
All JCS/raw-token, Unicode and numeric requirements come from the common owner.
Build inputs and compiled identity remain entirely independent of these records.

## Evaluation suite and oracle authority

Freeze one registered immutable suite per evaluation, with 1–256 declared cases.
An approval policy requires 1–16 exact suites. These are selected v1 receipt and
selection capacities, not reductions of acceptance E10's separate 10000/1000/250
qualification workloads. No suite can be truncated to fit; a larger product
evaluation requires separately registered complete suites and an explicit policy
that requires every constituent suite. It cannot be split silently after failure.

```text
suite_reference = {
  organization_id, domain_id, suite_id,
  version: content_revision, manifest_sha256
}
dataset_reference = {
  organization_id, domain_id, dataset_id,
  version: content_revision, manifest_sha256
}
case_declaration = {
  case_id: slug,
  input_sha256,
  oracle_sha256,
  required: boolean
}
suite_manifest = {
  kind: 'arboresce.context-evaluation-suite.v1',
  organization_id, domain_id, suite_id, version: content_revision,
  class: deterministic | live_quality,
  evaluator: profile_reference,
  dataset: dataset_reference,
  cases: case_declaration[1..256]
}
```

The suite manifest hash excludes its own hash. The immutable dataset manifest
binds the exact input bytes and allowed organization/domain use, including an
explicit empty dataset if no external cases are needed. No ambient dataset is
inferred from null or an unversioned filename. Each case is one independently
declared assertion. Its oracle digest binds the complete expected assertion,
comparison/scoring rule, expected value or threshold, and interpretation version
under the exact evaluator profile. Compound assertions are declared as separate
cases. Actual suite/oracle material must exist and be independently reviewed before
E102/E103 admit it; a digest without those bytes is not a usable oracle.

Sort the declared case set by ASCII case ID after rejecting duplicate IDs. At
least one case is required. The frozen `required` flags and evaluator/dataset
references cannot be overridden by an evaluation request or result. An output
cannot register its own expected value or replace an oracle to become passing.
The suite's class is explicit; deterministic evidence cannot acquire a live
fallback. A live-quality suite needs its separately qualified evaluator, holdout,
provider exposure and bounded cost profile before execution or release use.
E103's first implemented evaluator may support deterministic suites only; an
unsupported live class fails explicitly and is not silently relabeled.

The selected immutable evaluator profile includes finite case/total deadlines,
attempt limits, input/result bytes, memory/CPU and any live concurrency/cost
reservation. These are required consuming-profile inputs; absent numbers mean
unsupported work. Common shared admission, two-second transaction and ten-second
control-request limits still apply. Do not introduce a second scheduler, identity
service or quota store. Oracle/input/result objects are authorized canonical
objects addressed by exact identity; no request-provided download URL or code is
executed. Complete suite and receipt controls each have an actual 1 MiB byte cap
and common depth/container limits. Reject a profile whose complete bounded receipt
cannot fit before admitting it; per-field maxima are not an aggregate guarantee.

## Complete immutable evaluation receipt

```text
case_result = {
  case_id: slug, input_sha256, oracle_sha256,
  outcome: passed | failed | error | not_run,
  observed_result_sha256: sha256 | null,
  diagnostic: null | case_error | case_timeout | result_invalid |
                    suite_deadline | dependency_unavailable
}
evaluation_core = {
  kind: 'arboresce.context-evaluation.v1',
  evaluation_id,
  build: build_reference,
  context: command_context,
  policy: policy_reference,
  suite: suite_reference,
  class: deterministic | live_quality,
  evaluator: profile_reference,
  dataset: dataset_reference,
  expected_case_count: integer 1..256,
  results: case_result[1..256],
  counts: {passed, failed, error, not_run},
  validity: valid | invalid,
  outcome: passed | failed | invalid,
  started_at: timestamp,
  finished_at: timestamp,
  execution: execution_attribution
}
evaluation_record = {core: evaluation_core, evaluation_sha256}
evaluation_reference = {
  organization_id, domain_id, evaluation_id, evaluation_sha256
}
```

Every count is an unsigned integer 0–256 under the raw common token rule. At
least one of the four counts is positive because every suite has at least one
case. The schema rejects the all-zero count object; exact sums still require
semantic verification. The
record binds the exact build, requested purpose and actual evaluation executor;
the suite/evaluator/dataset/class are exactly those admitted. Every declaration
appears once in `results`, in declared order, with equal case/input/oracle IDs.
There is no missing-case omission or duplicate substitution. The case count is
both the manifest count and results length, and the four counts equal actual
outcomes and sum to it. Start is no later than finish; use authoritative server
time, never a caller timestamp. Concurrent/retried attempts cannot mutate a
committed receipt. The idempotent logical evaluation has one committed result.

Passed/failed results have a nonnull digest of the exact retained evaluator
result and null diagnostic. Error/not_run have null result digest and a nonnull
finite safe diagnostic. `error` uses case_error, case_timeout, result_invalid or
dependency_unavailable. `not_run` uses suite_deadline or dependency_unavailable.
Result bytes have the qualified profile's closed interpretation and limits;
an opaque hash is neither a score nor proof of the evaluator's correctness.
Unknown/malformed runtime output becomes explicit error/result_invalid; it cannot
be counted as a pass. Optional failures remain visible.

`valid` requires exact reference/count correspondence and zero error/not_run.
`invalid` requires at least one error/not_run, and its aggregate outcome is
invalid. A structurally contradictory record is rejected rather than accepted by
setting validity to invalid. A valid receipt is passed exactly when every
required case passed; otherwise it is failed. Therefore an optional failed case
can coexist with a passed suite, but an invalid optional result cannot make an
incomplete suite valid. Current release policy may require stricter suites; it
cannot reinterpret this receipt or ignore a required case.

Unknown suites or unavailable oracle inputs fail admission before evaluator
work. A subsequently interrupted bounded evaluation can commit an invalid
diagnostic receipt with every unexecuted case represented. Cancellation and
committed-effect reporting follow the operation owner; cancellation cannot
rewrite a previously committed result or erase consumed external work.

## Human event and separate approval receipt

Only the finite attestation kind `human_approval` is introduced. An evaluation
receipt is machine evidence, not a human attestation. Candidate acceptance is
not a build approval. Other attestation kinds require an explicitly owned future
profile and evidence semantics; an arbitrary string or boolean is insufficient.

```text
build_approval_event = {
  kind: 'arboresce.context-build-approval-event.v1',
  approval_event_id,
  build: build_reference,
  context: command_context,
  policy: policy_reference,
  reviewer: principal_reference(kind=human),
  assurance: profile_reference,
  executor: principal_reference,
  delegation_id: identifier | null,
  evaluations: evaluation_reference[1..16],
  decision: 'approve_build',
  reason: string 1..2048 scalars,
  authenticated_at: timestamp,
  expires_at: timestamp,
  authentication_audit_event_id: identifier
}
approval_event_reference = {
  organization_id, domain_id, approval_event_id, event_sha256
}
approval_core = {
  kind: 'arboresce.context-build-approval.v1',
  approval_id,
  build: build_reference,
  context: command_context,
  policy: policy_reference,
  event: approval_event_reference,
  reviewer: principal_reference(kind=human),
  assurance: profile_reference,
  evaluations: evaluation_reference[1..16],
  execution: execution_attribution
}
approval_record = {core: approval_core, approval_sha256}
approval_reference = {
  organization_id, domain_id, approval_id, approval_sha256
}
```

The event is issued by the accepted authenticated human-assurance mechanism and
stored canonically. Its closed wire representation is server evidence, never a
request that can create its own human identity. A request supplies only the
opaque event ID, not a reviewer, human flag, assurance claims, event body or
caller-selected public key. Actual issuer/provider/UI/delegation implementation
and exact assurance profile remain security-owner qualification inputs. No new
identity issuer endpoint or authentication service is introduced here.

The actual human authenticates an exact decision with the build, requested purpose,
policy and receipt set in view. The executor is either that authenticated human
with null delegation, or the distinct named principal operating under the
explicit valid scoped delegation. The event itself is not a delegation grant.
Null delegation therefore requires a human executor in both the event and its
approval receipt. Exact equality with the authenticated reviewer is a semantic
check. This condition does not restrict unrelated build or evaluation executors.
Its delegation must confer the action/purpose/exact scope/lifetime and preserve
every upstream assurance limit. Never label the agent executor as the reviewer.
Prior valid scoped authority can support noninteractive execution; a new prompt
per API call is not required.

The event expires exclusively: authenticated_at < expires_at, and a fresh
consumption requires authenticated_at <= now < expires_at under the configured
assurance profile's finite event lifetime. Current reviewer and executor
membership, action, delegation, purpose, evidence eligibility and event revocation
are checked again before the approval effect. The approval duplicates selected
event fields deliberately for direct inspection; every duplicate must be equal.
Sort evaluation references by ASCII (organization_id, domain_id, evaluation_id)
after rejecting duplicates. They must all name this exact build and purpose,
and match the event's exact normalized set. One required receipt per exact suite
is selected; two competing attempts for the same suite cannot conceal a failure
through accidental array order. The domain's explicit required-suite policy
decides which complete passing receipt is acceptable, including age and class.

Approval policy binds the exact required suite/version/evaluator/dataset sets,
allowed assurance profiles, purposes, issuer-key scope, finite receipt ages and
attestation lifetime. The v1 approval receipt has 1–16 suites and at least one
required deterministic suite. A required live-quality suite also has to pass
and be qualified; deterministic evidence does not replace it. Required receipts
must be valid/passed, correctly scoped, currently permitted and within their
declared age interval. Unrelated optional receipts do not substitute for a missing
required suite. All selected receipt identities remain immutable and readable
under current evidence policy; required unavailable receipt/oracle material denies.

An event can authorize one successful approval receipt only. Reserving it for a
durable command is not consumption; the reservation is mutable control outside
the event hash. An identical authorized idempotent replay returns the original
operation/receipt. Another command cannot consume an active reservation or an
already consumed event. A terminal failed command may release its reservation;
a later newly authorized command may use the still-unexpired, unrevoked event.
There is no implicit renewal or retargeting. Event expiry after a successful
consumption does not itself undo the historical approval; attestation expiry and
current trust/source/approval policy govern later use.

## Acyclic subject profile and exact signature

```text
subject_profile = {
  kind: 'arboresce.context-human-approval-profile.v1',
  build: build_reference,
  context: command_context,
  policy: policy_reference,
  approval: approval_reference,
  evaluations: evaluation_reference[1..16]
}
subject_profile_digest = SHA256(JCS(subject_profile))

signed_payload = {
  organization_id,
  domain_id,
  build_digest,
  attestation_kind: 'human_approval',
  subject_profile_digest,
  issuer_key_id,
  issued_at: timestamp,
  expires_at: timestamp
}
signature_input = UTF8('arboresce.context-attestation.v1') || 0x00 ||
                  JCS(signed_payload)

attestation_core = {
  kind: 'arboresce.context-attestation-record.v1',
  attestation_id,
  payload: signed_payload,
  subject_profile,
  algorithm: 'ed25519',
  signature: lowercase hexadecimal, exactly 128 characters / 64 bytes
}
attestation_record = {core: attestation_core, attestation_sha256}
attestation_reference = {
  organization_id, domain_id, attestation_id, attestation_sha256
}
```

Exactly eight fields occur in signed_payload. There is one actual zero byte
between the ASCII prefix and UTF-8 JCS object; no BOM, printable backslash-zero,
terminal newline, alternate container or hash-only replacement. Signature and
algorithm do not enter the eight-field object. The only supported envelope fixes
Pure Ed25519 over the complete signature_input bytes, not Ed25519ph/ctx and not
Pure Ed25519 over a substituted SHA-256 digest. The outer algorithm is a constant
consistency assertion, never caller-controlled algorithm negotiation.

The graph is suite/oracle material -> evaluation -> authenticated event ->
approval receipt -> subject profile -> signed payload -> signature record.
No earlier object contains a later object's hash. Approval is prepared with
server-issued IDs, exact immutable event/evaluation references and the actual
execution/audit identities; its hash is calculated without needing a signature.
The signature binds reviewer, assurance, actual executor/delegation, exact purpose,
policy and evaluation authority transitively through that exact approval hash.
Subject/event/approval duplicate fields must match, including evaluation order.
Signatures do not inherit meaning from only a profile label or an unverified hash.
The verifier needs the exact referenced bytes and trusted records to establish
the complete attested proposition. Missing bytes allow no positive authority
claim. Public-key signature verification alone is a narrower result.

Approval and signature are separate append-only records committed together. The
attestation ID itself need not be added to the mandated signing payload: the
trusted canonical attestation reference hashes the complete record, while the
signature's meaning is the eight-field proposition and its exact profile. A
copied signature with an invented record ID cannot become a canonical issued
record or authorize a channel. There is one canonical approval/signature result
per successful approving command and event consumption.

## Trust, time and revocation

```text
trusted_key_core = {
  kind: 'arboresce.context-attestation-key.v1',
  issuer_key_id,
  organization_id, domain_id,
  attestation_kind: 'human_approval',
  algorithm: 'ed25519',
  public_key: lowercase hexadecimal, exactly 64 characters / 32 bytes,
  not_before: timestamp,
  not_after: timestamp
}
key_control = {
  issuer_key_id,
  control_revision: common.control_revision,
  state: active | revoked,
  revoked_at: timestamp | null
}
```

Key core is immutable trusted configuration, not an accepted embedded key.
Control is authoritative current state outside signed/build content. Active has
null revoked_at; revoked has nonnull revoked_at and cannot become active again.
Initial registration has control revision 1 and active state; a revoked record
must have control revision at least 2. Every accepted control mutation advances
the counter without overflow or reuse; time-based expiry does not rewrite it.
Unknown/missing core/control, unsupported algorithm, wrong organization/domain/
kind, or a caller-provided key URL denies. Same issuer ID can never be rebound
to different bytes or scope. The selected policy fixes which registered keys
may issue for its purposes; no wildcard purpose is inferred from the key core.

Rotation uses distinct IDs and new public bytes with explicit configured overlap.
For this organization/domain, reject aliases of existing public bytes under a
new ID, including retired keys, so reimport cannot bypass revocation. Preserve
historical public bytes and revocation history without retaining an operational
private key. Separate key IDs may remain valid during an approved overlap, but
an expired or revoked member does not become eligible through the other member.
Operational key creation, storage, rotation and recovery remain explicit later
inputs/actions, not effects of accepting this contract.

Require not_before < not_after and issued_at < expires_at. At issue, require
not_before <= issued_at < not_after. At every positive current-use decision,
require current key state active, not_before <= now < not_after, and
issued_at <= now < expires_at. The initial profile has zero positive clock-skew
allowance: future-issued or exactly-expired records deny; unknown/unqualified
authoritative time denies. Host/client clocks are not authoritative evidence.

The immutable applicable policy and assurance/evaluator profiles must declare
finite event lifetime, attestation lifetime and maximum age for each required
evaluation. These are durations selected and qualified before use, not universal
values invented by this contract. Encode each selected duration as an unsigned integer
number of milliseconds within 1..9007199254740991, enforce the common raw token
rule and reject checked timestamp arithmetic outside the common year range.
Their finite values and complete interpretation are part of the policy/profile
hash; absent/zero/overflow/unsupported values deny. No unbounded null expiry exists.

Choose expires_at no later than all of: issued_at plus the configured attestation
lifetime, the selected key not_after, and each selected required evaluation's
finished_at plus its applicable maximum age. At final issue all these intervals
must still be open. Event expiry constrains fresh consumption, not an already
committed attestation's whole lifetime. A request cannot choose an arbitrary
longer expiry or silently extend a replayed record.

At channel/resolve use, verify the exact signed profile and recompute current
eligibility under the current domain policy. The historical policy reference is
not proof that an obsolete policy remains active. Changed requirements either
explicitly accept the exact historical evidence/assurance under current rules or
deny; there is no implicit backwards-compatible trust decision. Newly required
evaluation cannot be appended to an old signed subject profile. Obtain a new
approval/attestation when the current policy requires evidence not covered there.
Current revocation/expiry of a signing key denies even a signature issued before
revocation. Distinguish mathematical historical validity from a current trusted
issuer/eligible build; never label the first as the second. A source revocation,
purge, purpose denial or missing required evidence independently denies the build
despite intact signature bytes. Channel rollback cannot bypass those checks.

## Strict vetted cryptographic acceptance profile

Hex decoding uses the exact lowercase alphabet and full length with an absolute
end assertion; whitespace, uppercase, separators, odd length and alternate encodings
are rejected. Decoded public keys are exactly 32 bytes and signatures 64 bytes.
These lexical checks do not establish curve membership or valid signatures.

Adopt the narrower project Pure Ed25519 profile defined here:
canonical point decoding for both public key A and signature point R, canonical
scalar 0 <= S < L, rejection of identity/small-order/mixed-order A and R, and
the uncofactored verification equation. Require A and R to be nonidentity points
of the prime-order subgroup. This is an explicit project acceptance choice, not
a claim that RFC 8032 mandates all these extra rejections. A vetted pinned library
must actually enforce the selected profile in every supported language; a method
named verify_strict is not sufficient evidence. Never implement a custom curve
fallback to cover a library mismatch.

Key registration and every verification fail safely on noncanonical/invalid
encodings and unsupported group cases. The signer uses the same vetted Pure
Ed25519 profile and must not issue a signature rejected by the project verifier.
No secret is added to any receipt, public example, process argument or diagnostic.
The source-free parsed fixture checks can accept a well-shaped hexadecimal value
whose curve/signature validity remains a pending semantic test; label that scope
honestly. Primary references are [RFC 8785](https://www.rfc-editor.org/rfc/rfc8785),
[RFC 8032](https://www.rfc-editor.org/rfc/rfc8032) and
[RFC 8410](https://www.rfc-editor.org/rfc/rfc8410). Actual vector results are
recorded separately from these requirements.

## Evaluation and approval commands

Closed commands have exactly
`metadata` (common command metadata) and the following closed `body`:

| Command | Route | Body | Immediate result |
| --- | --- | --- | --- |
| context.evaluate | POST /v1/context-builds/{build_digest}/evaluations | {build: build_reference, suite: suite_reference} | 202 common durable operation |
| context.approve | POST /v1/context-builds/{build_digest}/approvals | {build: build_reference, approval_event_id} | 202 common durable operation |

Path digest, body build and context organization/domain must agree. Method, route,
v1, full metadata purpose/body and raw semantic array ordering enter the normal
idempotency fingerprint. There is no mutable build content revision to compare;
the exact digest is the content precondition. Event identity supplies the separate
single-consumption authority precondition. Current source and receipt controls
are compared canonically, never inferred from a caller's human flag.

Current syntax/capability/auth/authorization precede disclosure-safe existence and
idempotency handling. Authorized identical replay returns the original admitted
operation before fresh event-reservation or expiry checks; it does not perform a
new approval, extend time or assert the old result is currently eligible. New
conflicting event use/idempotency gives 409; incompatible exact references,
invalid receipt sets or expired event give 422 if current policy permits revealing
them; unavailable qualified dependencies give 503. Ordinary 400/401/403/404/413/
429 and current resource disclosure rules remain the common owner.

`context.evaluate` operation completion means the exact immutable evaluation record
was durably committed. Its typed terminal result is {command:'context.evaluate',
evaluation:evaluation_reference}. Validity/outcome in the required referenced
receipt determines evaluation success for release. A complete diagnostic receipt
may have validity invalid or outcome failed; operation completion alone never
means its tests passed. Unknown suite admission, failure to persist any receipt
or an unrecoverable orchestration failure uses the common failed operation. No
success result can substitute for missing cases or conceal an invalid receipt.

`context.approve` terminal result is {command:'context.approve', build,
approval:approval_reference, attestation:attestation_reference}. Only the jointly
committed records justify success. A reserved event, prepared bytes, outbound key
request or available signature alone stays pending. Authorized exact GET reads
of evaluations, approvals and attestations by their server IDs return the complete
closed immutable record inside the bounded additive GET result/envelope. The
record wrapper itself permits no additional properties. Their request context supplies
organization/domain/purpose explicitly; the ID alone cannot infer tenant scope.
No client-facing mutation route to register a suite, fabricate a human event or
import a signing key is introduced by these commands.

Building/evaluating requires current input and action eligibility, not an existing
build approval or signature. Approval requires the required completed evaluations.
Release/channel/resolve adds the current approval/signature gate. These distinct
stages cannot form a circular approval prerequisite. Current build availability
and control live in the separate resource wrapper and never in build,
evaluation or signed content identity.

Durable admission reserves the event/logical command and shared work capacity in
a short transaction. Resolve/copy exact profiles and evidence, check hashes and
prepare canonical receipt/payload bytes outside database locks. Allocate stable
logical result/audit IDs and choose issue/expiry timestamps before calling the
injected key service. Persist a fenced candidate binding so retries sign the
same chosen bytes rather than renewing lifetime invisibly. Signing occurs outside
database locks under finite qualified attempts/deadlines; it cannot run while a
client request holds the admission transaction open.

Before exposing the signature, use the same canonical authority-lock/fence
discipline as permission writers. Recheck actual human and executor membership,
delegation chain, purpose, active policy/assurance, event reservation/revocation/
expiry, exact build/source availability and receipt identities/age, issuer trust,
signature bytes and time. Earlier byte fetch/signing does not waive these current
checks. If any consequential dependency changed, abandon/revalidate the candidate
under the existing workflow; do not publish stale approval. An expired fixed
signing candidate fails rather than extending its signed time on replay.

One short final transaction consumes the event and appends the approval receipt,
signature record, actual authority audit, operation result and outbox effects.
The approval's preallocated audit ID is populated with the actual final admission
facts in this same commit. Until then neither candidate record is canonical.
Key-service output from a losing/crashed/fenced attempt is not externally exposed
and never counts as an issued attestation. Concurrent contenders can produce at
most one canonical approval/signature pair for the event. A committed pair cannot
later be split by a retry or cancellation; current revocation remains separate.
No channel promotion or asset lifecycle transition occurs implicitly.

## E007 fixed signing-vector gate

E007 must freeze concrete project signing vectors as well as field encoding.
Prepare a finite independently reviewed vector document with an exact literal
eight-field ASCII payload, manually frozen JCS bytes, prefix/NUL bytes and the
complete signing message identity. Use only the published RFC test seed/key
fixture and two independent vetted Pure Ed25519 implementations to calculate and
cross-check the exact signature and public key. Record their pinned identities,
exact commands and actual outcomes; never use an operational key or pretend a
test issuer is a trusted production issuer. The fixture may contain the clearly
labeled public test seed required to reproduce its published bytes.

Include an empty-message published primitive control, the exact project message,
and finite tampered message/field/prefix/signature rejection vectors. Verify both
implementations actually support the selected profile or explicitly limit the
current vector evidence to valid Pure Ed25519 agreement; selected malformed
point/subgroup rejection that is not exercised stays pending. Do not call the
manually frozen ASCII payload a qualified general JCS implementation. This narrow
deterministic vector step is a contract prerequisite, not all E10 qualification,
runtime issuer trust, authorization, key-service integration or custom crypto.
The frozen vector data and actual qualification evidence are separate from this
normative signing-input definition; neither replaces the later runtime gates.

## Independent pending oracles and actual inputs

The following are semantic/runtime oracles for the later owners, not outcomes of
parsed-value schema validation and not a claim of E10 completion:

1. Exact suite/case/oracle/evaluator/dataset joins, duplicate/order/count failures,
   one missing required case, malformed result and every error/not_run condition.
2. Optional failed case remains visible; valid/passed requires every required case;
   any error/not_run makes the complete receipt invalid and release-ineligible.
3. Independent oracle mutation cannot turn a known failed output into passing;
   unknown suite/version/class/profile never falls back to another evaluator.
4. Exact case/total budgets, cancellation, provider uncertainty, replay and fenced
   duplicate attempts retain one immutable complete receipt and honest effects.
5. Same content/profile with altered evaluation/approval/time preserves build
   digest; changing any bound receipt/actor/purpose/policy changes its own identity.
6. Normalize only declared sets; preserve meaningful order and full common JCS
   Unicode/numeric/raw-token rules across independent implementations.
7. Human event cannot be forged by kind/flag/body; human reviewer and delegated
   executor remain distinct, current and scoped at admission and final effect.
8. Event reservation/expiry/revocation, repeated same key, different key and crash
   races yield at most one canonical approval/signature pair without renewal.
9. Alter each of the eight fields, prefix, actual NUL, canonical container or
   signature profile; exact expected signing bytes and verification must disagree.
10. Public RFC Pure Ed25519 vectors plus actual project-prefix vectors; reject
    Ed25519ph/ctx/hash-only substitution and all selected noncanonical scalar,
    point, identity, small-order and mixed-order cases in every qualified runtime.
11. Trust substitution, same-ID rebind, public-byte alias/reimport, wrong scope,
    future issue, exclusive expiry, current revocation and explicit overlap.
12. Changed current policy/source controls between fetch, signing, final issue and
    channel/resolve admission deny stale eligibility; orphan signatures never grant
    canonical issued status and rollback never restores revoked evidence.
13. Cross-language complete receipt hashes and subject-profile/payload joins, with
    mutable transport extensions excluded and no self-hash/signature cycles.
14. Exact 1 MiB/depth/collection/case/receipt-set limits including aggregate byte
    arithmetic; no partial receipt or hidden dropped suite can make work fit.

E10's independent 10000 canonicalization cases, 1000 malformed keys/signatures and
250 financial cases remain required at their established owners, as do source,
package, repository and process coverage. Real required suite/oracle/dataset bytes,
vetted crypto/canonical libraries, actual human-assurance mechanism, domain policy,
finite work/time profiles, signer/trust configuration, authoritative clock and
transaction/recovery evidence are still implementation qualification inputs.
Those operational inputs require actual qualification. Specified contract values
and synthetic parsed cases do not activate a runtime.


## Identity and observed resources

`channel_key` is closed `{organization_id, domain_id, name}`. IDs reuse common
UUIDv4 identifiers; `name` reuses common slug without case folding or aliases.
There is exactly one retained generation history per key. Key existence, history,
targets and counts require current authorized organization/domain/purpose access.

`channel_core` is closed `{channel, generation, state, target, last_event_id}`.
The channel is `channel_key`; generation is common tagged generation constrained
to nonzero for an existing resource. `state` is `empty`, `active` or `suspended`.
Empty requires null target. Active and suspended require the exact owning build
reference with equal organization/domain. A suspended target is historical
retained identity, not serving permission. `last_event_id` is a server-issued
identifier for the immutable event at the same key and generation. Response
resources carry this closed core inside the existing bounded additive outward
response convention, rejecting contradictory state/outcome aliases.
`channel_resource` has exactly the required `core: channel_core` field and
permits bounded additive transport fields; the nested core remains closed.
Under the defined commands, empty resources have generation1, active resources
at least2 and suspended resources at least3. These reachable minima are static
schema constraints, not merely comments. No later command in this contract
returns a channel to empty.

Build availability, channel state and current release eligibility stay separate.
An active channel can refer to an ineligible build after a source/key/policy
revocation; exact resolution must deny it. Reads do not silently change generation
or rewrite history. Lifecycle control may explicitly suspend it through the same
transition authority; read-time denial needs no successful background suspension.

## Commands and transition matrix

All requests are closed common `{metadata, body}` commands. Metadata context must
match route organization/domain and requested channel key, with current purpose.
The actor is derived from authenticated authority, never a supplied actor field.
For these atomic commands, success is HTTP 200 and common succeeded response with
terminal idempotency receipt. The additive result requires `command` equal to
the channel command name, `channel: channel_resource` and `event: channel_event`.
The event action and resulting core must match that command's defined transition.
No command claims 202 pending work as an already committed generation change.

| Command | Required closed body | Fresh transition |
| --- | --- | --- |
| `channel.create` | `name`, `expected_generation` exactly tagged zero | Only absent retained key; create empty/null target at generation 1. |
| `channel.promote` | `name`, nonzero `expected_generation`, exact `build`, `reason` | Empty/active/suspended to active exact currently eligible build. Active-to-identical-target is a no-op error; suspended-to-same-target is explicit resumed promotion. |
| `channel.suspend` | `name`, nonzero `expected_generation`, `reason` | Active to suspended, preserving exact target. Empty or already suspended rejects. Current target eligibility is unnecessary to suspend; actor authority remains mandatory. |
| `channel.rollback` | `name`, nonzero `expected_generation`, nonzero `target_generation`, exact `build`, `reason` | Existing channel to active historical build. `target_generation` must precede the current generation and identify an active event in this same channel; its exact target must equal `build`. Current eligibility is fully rechecked. Active-to-identical-target rejects. |

`reason` is nonempty text at most 2048 Unicode characters; no markup interpretation.
Creation needs no arbitrary reason field. Route family is POST
`/v1/domains/{domain_id}/channels` for creation;
POST suffixes `/{name}/promote`, `/{name}/suspend`, `/{name}/rollback` for other
commands. GET `/{name}` returns a common success result containing current channel
resource without idempotency receipt. Listing/history pagination belongs to E009;
this contract does not invent a history enumeration endpoint or delete/clear API.

Every accepted fresh mutation increments generation exactly once. Reject maximum
generation before any effect; counters never wrap, reset or recycle after purge,
restore, failed attempts or name reuse. Generation zero is only the pre-creation
condition. An empty resource already has generation 1; empty never means zero.
There is no generic caller-selected generation assignment or partial state patch.

Authorized exact idempotent replay returns the original command event/resource
and original generation, even if the current channel has since advanced. Recheck
current disclosure authority before replay; do not reapply the fresh generation
precondition or increment again. Same key/different intent follows common conflict
semantics. A GET retrieves current state independently. Atomic failures leave no
new channel generation, successful receipt or publishable event.

## Immutable transition event

`channel_event` is closed `{event_id, channel, generation, previous_generation,
action, previous_state, previous_target, state, target, rollback_generation,
reason, executor, delegation_id, authority_audit_event_id, recorded_at}`.
The `action` is exactly `create`, `promote`, `suspend` or `rollback`.
Executor is the context-local server-asserted `principal_reference`
`{principal_id, kind: human|agent|service}`. Audit identity is a server-issued
common identifier; delegation is required nullable identifier. Time is common
UTC timestamp, not ordering.
The authority audit retains actual authenticated actor, bounded grant chain,
purpose and policy; an event field alone does not confer those rights.

Creation requires generation 1, previous_generation 0, previous_state/target null,
empty state/null target, null rollback_generation and reason. Promotion from empty
requires generation 2 and previous_generation 1; promotion from active requires
generation at least 3 and previous_generation at least 2; promotion from suspended
requires generation at least 4 and previous_generation at least 3. Suspension
requires active previous state, generation at least 3 and previous_generation at
least 2, with suspended result and unchanged nonnull target. Rollback requires
active or suspended previous state, generation at least 4, previous_generation
at least 3 and rollback_generation at least 2. An empty channel has no earlier
active generation to restore. Promotion and rollback end active. Every prior
state/target must match the preceding event.
Only rollback has nonnull rollback_generation; only creation has null reason.
Exact increment, key/build joins, prior event linkage and rollback target equality
are mandatory semantic checks; static schema fixtures label that distinction.
Expected-generation requests may name the maximum generation; runtime overflow
checks must then reject a fresh mutation without effects.

## Atomic current eligibility and disclosure

An eligible target requires complete sealed bytes and exact accepted input/review
bindings, current source availability and use restrictions, applicability/review
policy, required valid evaluation receipts and exact current trusted build approval.
Verify purpose, domain and source audience intersection; cross-domain build inputs
require their separately explicit policy. A signed historical receipt does not
override current source, human grant, evaluation, signer expiry or revocation.

Acquire admission against canonical membership/authority and domain context epochs.
Batch-prefetch immutable content and do expensive signature/digest checks outside
the two-second database transaction. Inside the final bounded transaction lock
the canonical channel row/key and relevant authority/context fences in the fixed
global lock order. Recheck all observed revisions/epochs, time-dependent validity
at a fresh injected clock instant, target availability, exact required receipts
and generation before updating channel, appending event, audit/outbox and terminal
idempotency result atomically. Concurrent inserts of constraints or revocations
must advance/contend on the same relevant fences; merely re-reading existing
rows does not protect absent records or newly activated/expired grants. If a
prefetch observation changed, abort with explicit retryable conflict; never sign,
do network/key I/O or hold a transaction open for a consumer transfer.

Unknown/unauthorized keys and targets use common non-disclosing errors before
existence/state details. Authorized stale generation, existing-key create,
unreachable-state mutation, unavailable/ineligible target or overflow conflicts
use common `conflict`/409. Structurally valid semantic no-op, invalid rollback
ordering or mismatched historical target uses `invalid_command`/422. Unauthorized
or inaccessible references use `forbidden`/403 only when current policy allows
existence disclosure, otherwise `not_found`/404; missing authentication uses
`unauthenticated`/401. Missing required backing dependencies uses `unavailable`/503,
structural/raw input failure `invalid_request`/400, byte-limit `request_too_large`/413,
media `unsupported_media_type`/415, shared admission `rate_limited`/429, unexpected
failure `internal_error`/500. All failure paths preserve uncertain actual effects
under the common contract rather than fabricating absence.

## Independent qualification inventory

Static cases cover all request/result/event variants, zero/nonzero/minimum/maximum
tagged generations, nullability, closed requests and additive outward responses,
extra actor/approval fields, wrong family/context shape and boundary reasons/names.
Pending runtime vectors require all cross-record joins and current authority,
50 same-generation contenders with one winner, create-versus-create, no-op and
replay, rollback to revoked/expired/incomplete content, suspended same-target
promotion, maximum overflow, new-grant/revocation phantoms, exact expiry boundaries,
crash/restore non-reuse, publication/receipt atomicity and cancellation races.
Parsed JSON Schema cannot execute these state, transaction or cryptographic checks.

## Complete response composition and admission

The capacities and profile choices above are selected v1 contract values:
1–100 exact eligible asset versions, 0–192 distinct direct observations per asset,
262144 compiled UTF8 bytes, 524288 manifest canonical bytes, 1–256 cases per
complete evaluation suite and 1–16 selected required suites including deterministic
evidence. They do not reduce any acceptance experiment count. Multiple eligible
versions of one asset are allowed; duplicate exact versions are rejected. All
actual aggregate control messages retain 1MiB/depth16/object128/array256/string65536
limits. Inputs that cannot fit fail whole, without truncation or hidden splitting.

`build_resource` is a bounded additive response object with required `reference`
(build_reference), positive `control_revision`, `availability` and `manifest`.
Availability is available/unavailable/purged. Available requires the exact complete
build_manifest; unavailable/purged return null manifest. An initial successfully
sealed/bound build starts available/control1. Unavailable/purged control is at
least2. Current read authorization must allow every disclosed field; null never
stands for a secret partial manifest. Restored availability and other control
transitions remain E008/later application owners; historical immutable identities
cannot change. Availability does not claim approval or current release eligibility.

All core objects remain closed. Canonical observation cores compose the existing
complete observation definition with exactly its seven named properties and
additionalProperties:false, preserving every source-kind constraint without
copying or weakening the source definition. Only explicit transport resource and
result/envelope objects tolerate bounded additive fields. Their reserved alias
set rejects state/status/code/problem/error/failure/failure_code/success/accepted/
approved/human_approved/authorized/confidence/result/operation where not a defined
required field. A documented nested evaluation core's own validity/outcome fields
remain legitimate domain results, not aliases for common operation success.

`context.build`, `context.evaluate` and `context.approve` always durably admit
through the common202 form with pending receipt and operation, even if workers
complete quickly. Their pollable terminal succeeded operations carry these required
result fields: `{command:context.build, build}`, `{command:context.evaluate,
evaluation}`, and `{command:context.approve, build, approval, attestation}`.
These outward result objects retain common bounded additive-reader behavior and
reserved-alias rejection, including at most 32 total members in each result
object. This same limit applies to GET, resolve, channel and operation-read
results. A completed
evaluation operation establishes receipt persistence, not a passing evaluation.
No full manifest is nested beneath an operation result. Replays return the
original operation/terminal result according to common reauthorization rules;
they never allocate new receipt IDs or renew event/attestation expiry.

`build_operation`, `evaluate_operation` and `approve_operation` specialize the
common operation with a required `command` discriminator (`context.build`,
`context.evaluate`, `context.approve`) and that command's succeeded result above.
`context_operation` selects exactly one of them. Queued/running/succeeded/failed/
cancelled state rules remain common. The operation has no nested idempotency
receipt. Initial 202 responses carry the selected queued/running operation and
their required top-level pending receipt, as specified by the common owner.

`context_operation_read_response` is a common succeeded response with no
top-level idempotency receipt. Its additive result requires
`{operation: context_operation, idempotency_receipt}`. When that operation is
queued/running, both receipt expiry fields are null; for any terminal state,
both are nonnull common timestamps with their required retention ordering.
Receipt/operation identity and context must match the recorded command.
GET `/v1/operations/{operation_id}` returns this HTTP 200 resource-read form for
these context commands. Authorized POST replay also returns this form with the
current recorded operation state, even when pending. It does not repeat effects,
renew expiry, invent a new receipt or force a terminal resource back into 202.
Retrieving a failed/cancelled operation successfully is not command success.
Generic progress, committed-effect details, cancellation and their races remain
the operation/lifecycle owner's responsibility; listing remains the listing owner.

The exact authorized reads are GET `/v1/context-builds/{build_digest}`,
`/v1/context-evaluations/{evaluation_id}`,
`/v1/context-approvals/{approval_id}` and
`/v1/context-attestations/{attestation_id}`. Their common succeeded response
results require respectively `{build:build_resource}`, `{evaluation:evaluation_record}`,
`{approval:approval_record}` and `{attestation:attestation_record}`, without an
idempotency receipt. These outward results retain bounded additive-reader behavior.
No listing, arbitrary suite/key registration or evidence-creation route is added.
GET channel is `/v1/domains/{domain_id}/channels/{name}` with result `{channel}`.
Every GET supplies the three explicitly required query fields organization_id,
domain_id and purpose using the common context types; reject duplicate or unknown
query keys, percent-decode once, and compare any route domain with query domain.
These selectors confer no authority. POST contexts remain in the typed bodies or
command metadata already defined; there are no competing context-header defaults.

The exact resolution route is read-only POST `/v1/context/resolve`. Closed body is
`{context:command_context, selection:resolve_selection, limits:resolve_limits}`.
Selection is exactly one closed variant:

- `{kind:build, build:build_reference}`.
- `{kind:channel, channel:channel_key, expected_generation:positive generation}`.

For the channel variant, require currently active state and equal current generation
at final admission; otherwise use disclosure-safe conflict. Resolve the exact
target and perform all the same release/source/byte/token checks as explicit
build resolution. No follow-latest alias, query selection or fallback exists.
The usage receipt adds required `channel`: null for direct build selection or
closed `{channel:channel_key, generation:positive generation}` for channel selection.
Its returned build and channel/generation must match the admitted selection.
Common200 succeeded result requires `{build, compiled, content, usage}` and has
no command idempotency receipt. Partial transmission is never a complete result.

Usage/policy/tokenizer references must all have equal organization/domain context;
policy reference resolves the exact current registered immutable policy bytes and
canonical policy controls. Profile/dataset/policy hashes identify exact immutable
registered bytes, interpreted only by their owning qualified profile, without
implicitly granting authority or introducing an unreviewed generic policy engine.
Unknown bytes, interpretation, policy or tokenizer qualification deny consuming
work. Future first-use implementation slices must pin and qualify those concrete
profiles before accepting them; a placeholder identifier cannot activate support.

Preassemble and verify complete content/identities/token counts and bound all
response fields before final read admission. Final admission atomically binds the
new usage identity, actual authority audit, current channel target when selected,
current source/receipt/key/policy fences and injected time. Final bounded receipt
serialization/encoded response size checks occur before committing admission;
expensive content work or network transfers never run under that transaction.
Require admitted_at < transfer_expires_at, no later than the remaining ten-second
request deadline or any applicable authority/attestation/key/receipt-age expiry.
Enforce the remaining duration with qualified monotonic time as well as retained
UTC metadata. A later source revocation denies new admissions; an already admitted
bounded transfer has only that grant. Repeating the read admits afresh, not from
an old seven-day command result. The receipt says delivery:admitted and does not
claim complete client receipt, downstream consumption or recallability.

Separate eligibility stages prevent cycles: construction and evaluation require
current exact input/action eligibility; approval additionally requires its exact
complete current required evaluation set and human event; promotion/rollback and
resolution additionally require a complete currently valid trusted attestation.
GET inspection is authorized separately and does not require a build to approve
itself before its content or evaluation may be reviewed.

The common numeric profile applies without broadening: general admitted binary64
numbers follow common bounds, while exact IDs/digests/revisions remain strings
and unsigned protocol counters reject raw1.0/1e0/bool aliases before coercion.
The fixed compiler configuration contains only explicit constants; it neither
introduces arbitrary numeric settings nor removes later JCS numeric qualification.
No positive runtime result is inferred from a parsed schema's integer acceptance.

## Signing-vector data contract

The separate [signing fixture](../../contracts/validation/context-v1.signing-vectors.json) uses schema_version1, protocol
`context-attestation-v1`, explicit `fixture_key` identifying published RFC8032
TEST1 seed/publickey/source and a finite array of named project vectors. Its
expected payloads and exact canonical bytes are independent frozen data. Vector
fields are `id`, exact `payload`, `canonical_payload_utf8`, `signing_input_hex`,
`signing_input_sha256`, `signature`, and `expected_primitive_valid` boolean.
IDs and public seed/key are explicitly fixtures, never trusted issuer registration.
The metadata schema freezes the exact vector inventory and expected booleans,
checks fields/encodings through the root-local common/context schema graph. The
one finite attestation-kind substitution is intentionally not a valid product
signed_payload: its vector shape admits only that named negative's exact altered
kind, while preserving all eight required fields and other primitive constraints.
It must not broaden the production kind enum. Exact preimage/payload/hash consistency and signatures require the
separate actual two-implementation vector check. Deliberately changed-prefix/NUL
vectors can carry a signing input different from mandated payload construction;
the finite vector identity explains that expected negative experiment. Primitive
verification can be false even though each parsed field has a valid shape.

Start with one known valid exact project vector, then eight individual payload
field substitutions, changed prefix, removed NUL, printable backslash-zero,
message-byte change and signature-bit change using the frozen original signature.
All are signed-message mutations with expected primitive false, not claims about
current time/trust policy. A separate actual wrong-public-key probe and published
primitive controls retain their exact raw results. The signature and preimage
hash are populated only after independent actual agreement and review, then remain
literal fixtures. Schema positive/negative field-bound tests separately exercise
malformed hex/length/unknown envelope and never stand in for curve validation.
