# Intelligence processing and durable work

Status: accepted normative requirements; the processing services and workflows
are not implemented. Exact internal messages are frozen by the intelligence
transport contract step in the [implementation plan](../execution/open-cli-mvp-v1-rcld.md).

This document owns pipeline behavior, stage/result identity, durable execution
and controlled model invocation. [Architecture](architecture.md) owns package
boundaries. [Dependency qualification](dependency-qualification.md) owns exact
toolchain, SDK, library and service pins; [resource profiles](resource-profiles.md)
owns byte, time, memory, concurrency, queue and cost limits.

## One processing boundary

The [lifecycle analysis contract](lifecycle-contract.md#5-acquisition-and-analysis)
defines capture.analyze as a separately authorized command over an exact current
submitted revision/control. capture.submit remains atomic and never starts work
on admission, replay or later readiness. Analysis freezes the qualified profile,
ordered input eligibility and immutable submission attribution retained beyond
detailed receipt expiry. Later drafts or optional readiness cannot retarget it.
Its complete success record accounts for every submitted part and created output;
empty outputs may be explicit abstention, while required work must complete.

Begin with one coarse-grained Python gRPC service whose modules separate
normalization, extraction, transcription, field validation and claim synthesis.
A module does not imply a separately deployed service. Rust owns canonical
commands, policy and accepted persistence. Python receives no canonical database
credentials and cannot write an approval or asset through a processing response.

The pipeline must:

1. Consume an explicit submitted capture revision and its exact ordered
   attachment set after required readiness checks.
2. Use quarantined, verified sealed originals; preserve original bytes
   separately from every derived representation.
3. Normalize/decode supported media under independent encoded, decoded,
   CPU, memory, duration, scratch and wall-time limits.
4. Apply the qualified extraction/transcription profile to exact input digests.
5. Validate typed fields, evidence locators, contradictions, uncertainty and
   output bounds; mark absent grounding explicitly.
6. Propose structured financial records or scoped candidate knowledge with
   support, refutation and qualifications. Neither output is an approval.
7. Return a typed stage result for Rust revalidation and fenced, idempotent
   canonical acceptance after current policy checks.
8. Record processing/model/prompt/parser versions, input/output digests, safe
   request identifiers and bounded exposure/cost receipts.

Required attachment failure prevents dependent processing. Optional attachment
failure stays visible alongside permitted partial results. Do not silently
remove an attachment, substitute another source revision or declare the whole
capture successful because one stage finished. [Data model](data-model.md) owns
capture readiness and source-kind distinctions.

## Supported inputs and representations

MVP input classes are plain text, Markdown, JSON, JPEG/PNG, WAV and qualified
non-DRM M4A. A concrete profile must select actual codecs, channels, languages,
OCR/ASR libraries and provider behavior before claiming support. No generic
container extension establishes codec support, and no universal document,
language, audio or video capability is implied.

Text/JSON normalization rejects invalid encoding and duplicate keys. Preserve
the original digest, produce a new immutable representation for changed bytes,
and validate UTF-8 offsets and JSON Pointer semantics against that output.
Do not quote an original-byte offset as though it referred to normalized text.

Image decoding validates encoded bytes and decoded dimensions independently.
Reject bombs, truncation and unsupported animation, preserve orientation
mapping for regions, minimize unnecessary metadata, and contain parser failures.
Audio normalization validates selected codecs, channels and decoded duration;
preserve exact channel/time mappings into normalized audio and transcript spans.
Subprocess invocation uses explicit argument boundaries, no shell interpolation,
and bounded cancellation/termination and scratch cleanup.

Use the limits from [resource profiles](resource-profiles.md) across the API,
Activities and Python process. Library defaults do not replace these limits.
Malformed media, slow streams and resource exhaustion must fail before
unbounded allocation or uncontrolled external exposure. Cancellation and timeout
must reclaim owned resources and retain an honest operation outcome.

## Grounded proposals and disagreement

The [observation contract](observation-contract.md) defines precise grounding,
locator and source-kind representations. Processing must preserve those
distinctions and revalidate the underlying source and reference semantics.

Validate provider output as untrusted data. Reject incompatible shapes, invalid
locators, overlong output and out-of-profile values before acceptance; Rust
revalidates the resulting typed message rather than trusting Python's success
flag. Model responses cannot select an arbitrary internal operation.
Raw evidence, retrieved passages and their embedded instructions have no
privileged control role. Sensitive fields and permissions require deterministic
validation; model judgment alone cannot admit them.

Field extraction preserves the difference between an observed document value,
an author's statement and an inferred value. Receipt totals, voice assertions
and competing interpretations can disagree. Keep that disagreement and route
unresolved fields for review. Never invent a date, currency, evidence location
or confidence value to complete a proposal. Exact monetary interpretation and
confirmation belong to [financial records](financial-records.md).

Transcription can abstain on noise or unsupported content. Spans refer to the
exact audio representation, output is bounded, and unknown speakers remain
unknown. Diarization does not verify identity. Target lexical and consequential
semantic scoring is separately qualified, including negation, amounts, dates
and attribution; a plausible transcript alone is not a quality result.

Candidate synthesis retains scope, assumptions, refuting/qualifying evidence
and invalidation conditions. It must not turn a purpose assertion into an
independently proved outcome, discard confounders or fabricate sources.
Cross-document work uses bounded selection rather than global all-pairs
comparison. Output cardinality and content bounds are explicit.

## Typed internal transport

Version internal Protobuf under the engine's internal contract boundary.
The request/response contract identifies organization and authorized scope,
operation, stage, exact source revision, input digest, processing-profile
digest, supported contract version, deadlines and permitted output kinds.
Use immutable authorized blob references for bulk bytes; control messages stay
within the [resource profile](resource-profiles.md).

Return typed outputs with exact representation/locator identities, uncertainty,
warnings, safe provider request references and exposure/cost evidence. Reject
oversized, malformed or incompatible messages explicitly. Credentials, complete
source bytes and reusable signed URLs must not enter workflow histories or
general-purpose RPC metadata.
gRPC supplies transport, not a durable queue or an exactly-once guarantee.
Durable acceptance and retry reconciliation remain the operation/outbox contract.

Generate Rust/Python bindings using pinned local tools and verify reproducibility.
Do not hand-maintain competing canonical schema copies. Ship ordinary prompts
as versioned package resources with input/output schemas and evaluation links.
No production notebooks, model weights, credentials or automatic runtime tool
downloads belong in the repository/package contract. External weights and
providers retain their own rights and qualification requirements.

## Durable operations and fencing

The [shared operation contract](lifecycle-contract.md#3-shared-operation-plan-clocks-and-evidence)
owns exact immutable plans, required stage states, distinct attempts and accepted
results, injected UTC/monotonic deadlines, sampled progress, actual worker
attribution and complete append-only effect/exposure ledgers. Select all finite
profile limits and combined serialized headroom before admission. Replays,
takeover, retry, polls and receipt delivery never reset deadlines or counters.
Cancellation closes new business work under its final fence; it preserves all
effects, source restriction, cleanup responsibility and conservative liability.
[Late confirmation](lifecycle-contract.md#9-late-confirmation-and-cancellation-races)
uses only already admitted actions and qualified current evidence authority;
it cannot change terminal outcomes, issue new actions or renew expired probes.

Use Temporal for long-running analysis, builds, repair, lifecycle propagation,
exports and notification work. Simple reads, ordinary transactions and current
authorization do not need a workflow merely to access the application boundary.

Admitted command state, audit and outbox are committed together under the
[API contract](api.md#idempotent-command-admission). Dispatch starts a stable
operation-derived workflow and records its delivery independently. A crash
after workflow start but before delivery acknowledgement must converge on
the existing operation rather than start a second business effect.

Keep command idempotency, operation identity, workflow start identity, workflow
run ID, stage identity and attempt ID distinct. Accepted stage-result uniqueness
uses organization, operation, stage, exact input digest and processing-profile
digest. Attempt number is never the business deduplication key.

Lease acquisition/recovery carries a fencing value that is checked when
accepting a stage result or acknowledging delivery. A worker whose ownership
expired cannot overwrite the winner's accepted result, even if its remote
call finishes late. Retry reuses a recorded valid result when the exact
identity matches; changed input/profile requires a distinct result. Persist
actual source revision and output identity with the accepted record.

Workflow code is deterministic and may use the supported workflow SDK APIs.
External I/O, parsing, database/provider calls, system wall clocks and
uncontrolled randomness belong outside replayed workflow code. Effects run
through Activities and focused application ports. Domain/application code does
not import Temporal or infrastructure types. Architecture verification must
test permitted SDK dependencies as well as prohibited effect paths.

Classify transient, permanent and uncertain failures by stage. Bound maximum
attempts and total lifetime. HTTP request deadline, operation lifetime,
Activity deadline, gRPC deadline and provider timeout are separate clocks;
propagate the appropriate remaining bound rather than resetting it at each
hop. Long Activities heartbeat and observe cancellation as their contract
requires. Do not hold a database transaction across model, storage or RPC work.

Workflow IDs, task queues and searchable execution metadata must not contain
customer names, document text or credentials. Preserve safe opaque correlation
identity and bounded diagnostic state.

A timeout can occur after a provider performed work or accepted a billable
request. Retain that uncertain outcome and reconcile it; do not report zero
cost or claim exactly-once inference. An explicit cancellation may lose to
completion and can retain completed immutable stages. Detaching a client wait
does not cancel the operation. [API operation behavior](api.md#operations-lifecycle-and-exports)
owns the externally visible result.

Graceful shutdown stops new work, bounds draining and leaves recoverable
operation/lease state. Restart must not lose accepted commands, report unfinished
work as successful or treat a maximum outbox ID as completion proof.
[Storage and search](storage-and-search.md#atomic-audit-and-fenced-outbox)
owns delivery persistence and [recovery](recovery.md) owns restored serving.

## Model gateway and shared reservations

Before any exposure, Rust authorizes the provider/model/feature, organization,
purpose, classification, region and current policy/terms validity. Reserve
shared job slots and bounded cost before invocation, atomically across service
instances. Apply admission with the first live adapter; a later consolidation
step cannot excuse unbounded early effects.

All model, embedding and reranking traffic passes the controlled gateway.
Internal invocation is authenticated and bound to job, stage and purpose;
wrong scope fails before a provider call. Do not place an unrestricted vendor
key in Python as a replacement for gateway enforcement. Any Python-owned
invocation design requires equivalent verified scoped authorization and egress
control. Unknown, unapproved or expired policy fails closed for restricted data.

Query text has the same exposure obligations as source content. Retrieval
candidates must pass canonical authorization before their text reaches an
external reranker, as specified in [storage and search](storage-and-search.md#bounded-search).
In-flight stages recheck current policy before new exposure and effect
boundaries under [security and privacy](security-and-privacy.md).

Settle reservations against actual/uncertain provider usage with safe receipts
and bounded retries. Leases and reconciliation recover interrupted reservations
without allowing concurrent instances to exceed the declared quota. Keep
customer-specific prompts, datasets and keys restricted; no cross-customer
training occurs by default. Generic pipeline source and ordinary packaged
prompts remain independently usable under the project's public rights.
Versioned provider/model replacement must preserve existing assets and evaluation
receipts; changing a processing route creates new qualified profile identity,
not a rewrite of accepted knowledge. Exposure receipts distinguish attempted,
completed and uncertain requests and retain authorized asset references and
classification without copying content into unrestricted telemetry.

At least one real processing route must be qualified. A user-selected external
provider may require credentials/payment under [product scope](product-and-scope.md);
an Arboresce entitlement is not a prerequisite. Concrete data-exposure and
spend permission remains necessary for the selected live run.

## Verification and qualification

E010 qualifies complete stage/input/output/result and exposure-byte profiles.
E063 supplies the earliest actual shared operation GET/cancel and upload
specialization using E046 persistence, E068's qualified server and E071's
Temporal dispatcher.
E072/E073 extend that orchestration with deterministic explicitly test-only
Activities; E074 uses synthetic histories and actual registered Temporal service
tests. They do not enable real capture.analyze. E086 must qualify and wire the
selected real processing adapters, gateway and profiles before advertising that
capability. No second operation store/scheduler or reverse dependency on later
real processing is introduced by the earlier source gates.

Deterministic recorded processing resolves a response only by the exact
canonical request fingerprint and profile digest, with explicit fixture
selection. A missing recording or profile mismatch fails; it never falls back
to a live provider. This lane supplies repeatable test behavior and cannot
qualify the shipped model's accuracy.

Maintain replay histories for successful work, retries, cancellation,
failure boundaries and worker restart. Replay every relevant workflow or
dependency change using supported pinned APIs; do not reach into unsupported
SDK internals or automatically download a test server. Real integration tests
must cover crash-after-start, lease takeover, stale completion, current-policy
revocation, invalid output, uncertain cost and actual cross-process execution.

[Testing and coverage](testing-and-coverage.md) owns fixture independence,
collection and process coverage. [Acceptance](acceptance.md) owns held-out live
OCR/ASR scoring, corpus approvals and performance thresholds. Missing target
languages, codecs, rights, labels or numerical ASR criteria blocks the dependent
qualification; it is not permission to invent a value or replace live evidence
with recorded responses.
Report schema validity, factual grounding, calibration, abstention and observed
outcome quality separately. Strong results in one dimension cannot substitute
for missing evidence in another. Keep final holdouts separate from prompt-tuning
examples under the testing and acceptance owners.
