# Lifecycle, operations and attributed feedback v1 contract

**Authority:** normative lifecycle and shared-operation semantics, public and
internal wire definitions. **Status:** accepted contract; schema qualification
and every runtime capability require their owning measured evidence. This
document does not advertise an implemented service, storage adapter or repair
endpoint.

The [public lifecycle library](../../contracts/public/v1/lifecycle.schema.json)
owns named public operation, acquisition, lifecycle, export and feedback shapes.
The [internal continuity library](../../contracts/internal/v1/lifecycle-continuity.schema.json)
owns internal upload-session journals, recognition, retirement and continuity
batches. Both are Draft 2020-12 definition libraries whose roots reject every
instance. Select an exact named definition through reviewed local relative
references. Internal definitions may import public definitions; public contracts
never import internal ones. Neither library has a remote, dynamic or recursive
reference or an identifier that changes the base URI.

The [common contract](common-contract.md) owns primitive and raw-JSON encoding;
[acquisition](acquisition-contract.md), [observations](observation-contract.md),
[candidate review](candidate-review-contract.md), [financial records](financial-records.md)
and [context](context-contract.md) retain their immutable domain definitions.
[API](api.md) owns general admission and transport, [security](security-and-privacy.md)
owns current authorization, [storage](storage-and-search.md) owns persistence,
and [recovery](recovery.md) owns reopening restored service. This document owns
the precise lifecycle additions and shared record relationships; the linked
owners do not duplicate them. [Resource profiles](resource-profiles.md) and
[testing and coverage](testing-and-coverage.md) retain their wider qualification
requirements.

[Standalone checks](../../contracts/validation/README.md) validate independently
authored parsed values in one lifecycle family with public and internal target
namespaces. They do not run authority checks, canonicalization, storage,
Temporal, repair, delivery or restoration. The
[rolling plan](../execution/open-cli-mvp-v1-rcld.md) assigns those implementations
and records actual evidence separately.

## 1. Notation and inherited limits

Every displayed object is CLOSED, including canonical cores, record wrappers, references, controls and command bodies, unless explicitly marked additive32. Additive32 means at most32 total members with the existing common bounded-JSON extension values, key syntax and reserved-alias rules. Required legitimate fields are permitted in their declared position; top-level or nested aliases for result, receipt, authority, outcome or operation are forbidden elsewhere. A record returned by GET remains closed inside its additive result.

The following notation is exact type shorthand, not optional fields: unannotated *_id and identifier are common UUIDv4 identifiers; *_sha256 and digest are common lowercase SHA-256; time/timestamp, admitted_at/recorded_at/received_at/created_at/placed_at/authorized_at/sealed_at/expires_at and other *_at/*_deadline timestamps use common.timestamp; version/content_revision use common.content_revision; revision/control_revision use common.control_revision; reason is string1..2048. context means common.command_context, command_metadata means the existing closed common metadata. Numeric integer bounds are inclusive; arrays T[a..b] have those inclusive lengths. Literal words in unions are string constants. unsigned means integer0..9007199254740991 and positive means integer1..9007199254740991. All fields shown are required; null is accepted only when explicitly shown. Definition aliases resolve existing leaves rather than copied approximations.

Prefixes common, acquisition, observation, candidate_review, financial and context name the six existing public application schema families (financial is financial-records). Unqualified principal_reference, profile_reference and execution_attribution use E007 context definitions; accepted_asset_reference is candidate-review.schema.json#/$defs/accepted_asset_reference; build/evaluation/approval/attestation references use their E007 definitions. capture_reference is observation.capture_reference; observation_reference is observation.observation_reference; calendar_date is common.calendar_date. The lifecycle scoped_policy_reference is the new discriminated organization/domain policy union below, distinct from E007 policy_reference. Historical command names use common.command_name plus actual registered capability semantics; operation command fields use the finite eight-kind operation enum.

All control JSON obeys the existing raw1MiB, depth16, object128, array256 and string65536 admission bounds and actual complete serialization check. The context contract's literal-UTF-8 producer encoding rules remain unchanged. Complete canonical JCS cores exclude their own digest and mutable transport observations. Exact byte/hash, canonical identity, authority and cross-record comparisons are semantic qualification, not consequences of schema validation.

Selected wire capacities are stages1..64, total attempts0..256, effects0..256, exposures0..256, material actions0..256 and sampled progress updates0..256. Real profiles must preflight the combined complete permitted execution, late-confirmation headroom, message size, counters and finite resource budgets. Wire maxima are not operational defaults and may not fit together. No truncation, hidden entries, pagination or invented empty set may make a profile pass. Existing shared resource limits remain in force, including two active model jobs and100 pending jobs per organization, global slots, canonical reservations, and the existing USD1-per-capture qualification ceiling without spending authority. HTTP control remains10seconds and ordinary canonical transactions2seconds; bulk delivery has its separately selected finite limits.

## 2. Capability, origin and transport matrix

The eight operation kinds defined by this v1 contract are exactly:

| Operation command | Origin | Successful result |
| --- | --- | --- |
| upload.finalize | command | upload_finalize_operation_result |
| capture.analyze | command | capture_analysis_result |
| context.build | command | existing context.build_result |
| context.evaluate | command | existing context.evaluate_result |
| context.approve | command | existing context.approve_result |
| expense_report.export | command | financial_export_result |
| lifecycle.purge | command | lifecycle_purge_result |
| lifecycle.propagate | lifecycle_event | propagation_operation_result |

The existing21 public request commands retain their exact requests: upload.allocate, upload.finalize, capture.create, capture.revise, capture.submit; candidate.revise, candidate.accept, candidate.reject, candidate.contest; expense.revise, expense.confirm, expense.void, expense_report.create, expense_report.export; context.build, context.evaluate, context.approve; channel.create, channel.promote, channel.suspend, channel.rollback. This lifecycle family adds exactly these public commands: upload.abort, capture.analyze, operation.cancel, lifecycle.revoke, lifecycle.hold.place, lifecycle.hold.release, lifecycle.purge and feedback.record. lifecycle.propagate has no public request or idempotency claim. Common command-name syntax does not register a capability. First-owner provisioning later qualifies its actual registered operator command and immutable attribution at E054; E008 invents no bootstrap endpoint or name.

E124 must add a separately admitted propagation-repair command, operation kind, plan/stage/result contracts and their registered transport atomically before enabling repair. Section 8 defines its exact reference, obligation, material-action and completion hooks now; they do not enable a ninth operation or unspecified command at E008. A successful partial repair batch must have its own exact result, never the whole-obligation lifecycle.propagate result. E124 reuses the existing operation store and dispatcher.

All mutation POSTs use closed {metadata,body}. Ordinary new asynchronous commands return common202 with queued/running operation and top-level originating pending receipt. E007 command-origin operation GET and authorized POST replay return common200 succeeded with additive result {operation,idempotency_receipt}, no top-level receipt. The receipt never appears inside the operation. Initial/replayed upload.finalize uses its explicit exceptions in section 5. Atomic commands return their own common200 terminal receipt and original result on eligible replay. Read-only POSTs have no command metadata or receipt.

Internal lifecycle.propagate GET returns common200 with additive result {operation,origin_event:lifecycle_event_reference}; both top-level and nested command receipts are forbidden. Its source revoke already has its own terminal atomic receipt. Polling a failed/cancelled target successfully never labels its work succeeded.

The exact shared operation read is GET /v1/operations/{operation_id}. Its
origin selects the command receipt or lifecycle-event result above. This read
returns every required retained ledger and record at one observed control or
fails under the current-authority and retention rules; it never substitutes a
different operation, hides entries or restarts work.

Every exact GET uses required organization_id, domain_id, purpose query selectors, rejects extra/duplicate selectors, decodes once and matches route/reference organization/domain. POST selectors and command context also match. Current authority/disclosure checks occur before revealing claim, state, identity or count. The complete requested record/ledger is returned or the read fails: no redacted ledger, hidden-domain count or fabricated empty set. Unknown/undisclosable resources use404; applicable authentication/action failures use401/403. Known authorized missing required retained details use409; unavailable required infrastructure, policy or continuity uses503. No lifecycle reference grants access.

## 3. Shared operation, plan, clocks and evidence

```text
operation_reference = {organization_id, domain_id, operation_id}
execution_plan_reference = {
  organization_id, domain_id, plan_id, plan_sha256
}
execution_plan_core = {
  kind:'arboresce.operation-plan.v1', plan_id, operation:operation_reference,
  command:operation_kind, context:command_context, profile:profile_reference,
  limits:execution_limits, stages:stage_declaration[1..64]
}
stage_declaration = {
  stage_id, ordinal:0..63, kind:stage_kind, required:boolean,
  depends_on:integer[0..63], input_sha256, profile:profile_reference,
  output_kind:stage_output_kind
}
stage_result_reference = {
  organization_id, domain_id, operation_id, stage_id, stage_result_id,
  input_sha256, profile_sha256, result_sha256, kind:stage_output_kind
}
```

execution_plan_core is a closed discriminated union. The seven command-origin variants have exactly the fields above and their exact command constant. The internal propagation variant has those fields plus required origin:operation_origin and obligation:propagation_obligation_reference; it requires command=lifecycle.propagate and lifecycle_event origin. Define execution_plan_record={core:execution_plan_core,plan_sha256:sha256}. GET /v1/operation-plans/{plan_id} returns common200 result {plan:execution_plan_record}, additive32, current context/disclosure and no receipt.

Plan hash is SHA256(JCS(core)). Plans are immutable and selected at admission from exact input/profile; caller stages, workflow IDs and retries cannot change them. Stage IDs and contiguous unique ordinals remain stable; dependency ordinals are unique, strictly earlier and0..63. Each accepted result is unique by organization/operation/stage/input/profile digest. Complete stage-result bytes and any promised exact read surface must be defined and qualified at E010 or the first consuming owner before support.

stage_kind is seal_input, normalize, extract, transcribe, validate_output, synthesize, compile_context, evaluate_suite, prepare_approval, sign_attestation, bind_result, compile_export, propagate_restriction, purge_material or reconcile_exposure. stage_output_kind is sealed_original, representation, observation_set, candidate_set, expense_proposal_set, build, evaluation, approval_attestation, export_artifact, restriction_propagation, purge_completion, exposure_reconciliation, capture_analysis, propagation_manifest or propagation_proof. These finite identities do not publish arbitrary JSON result bodies.
```text
execution_limits = {
  max_stage_count:1..64, max_total_attempts:1..256,
  max_canonical_effects:0..256, max_exposures:0..256,
  max_progress_updates:0..256,
  operation_lifetime_ms, stage_lifetime_ms, attempt_lifetime_ms,
  reconciliation_lifetime_ms, cancellation_observation_ms
}
operation_timing = {
  admitted_at, work_deadline, reconciliation_deadline,
  started_at:timestamp|null, terminal_at:timestamp|null
}
progress = {kind:'unknown'} |
  {kind:'counted', unit:'items'|'bytes'|'cases', completed:unsigned, total:positive}
stage_snapshot = {
  stage_id, ordinal, kind:stage_kind, required:boolean,
  state:'not_started'|'running'|'retry_wait'|'succeeded'|'failed'|'skipped'|'cancelled',
  started_at:timestamp|null, deadline:timestamp|null, finished_at:timestamp|null,
  progress, attempts_started:0..256, last_attempt_id:identifier|null,
  accepted_result:stage_result_reference|null,
  diagnostic:stage_diagnostic|null
}
attempt_record = {
  attempt_id, ordinal:1..256,
  subject:{kind:'stage',stage_id}|{kind:'exposure_reconciliation',exposure_id},
  state:'running'|'succeeded'|'failed'|'cancelled'|'uncertain',
  started_at, deadline, finished_at:timestamp|null,
  failure:attempt_failure|null,
  execution:worker_execution_attribution
}
attempt_failure = {
  class:'transient'|'permanent'|'uncertain', code:stage_diagnostic
}
```

Every execution_limits duration is positive_duration_ms. Real registered profiles select all durations plus narrower per-stage/adapter, memory, scratch, concurrency and cost bounds before admission. stage_diagnostic is exactly source_unavailable, policy_changed, capacity_denied, dependency_unavailable, invalid_output, deadline_exceeded, worker_lost, attempts_exhausted, dependency_failed, cancel_requested or outcome_unknown; raw provider exceptions are excluded.

Use injected qualified UTC and monotonic remaining durations. work_deadline=admitted_at+operation_lifetime_ms and reconciliation_deadline=work_deadline+reconciliation_lifetime_ms, fixed at admission. First stage start fixes min(stage lifetime, remaining work) and each attempt stays inside it. Retry, lease takeover, polling, replay and receipt delivery never reset a deadline. Cancellation closes work within min(cancellation observation budget, remaining work). Checked arithmetic must fit common UTC years. Counted progress has completed<=total and profile-defined immutable unit/total; processed failed/skipped cases may count only under that exact profile. Unknown is explicit.

| Stage state | Required branch |
| --- | --- |
| not_started | All clocks null;0 attempts; last attempt/result/diagnostic null; unknown progress |
| running | start/deadline nonnull, finish null; positive attempts; result/diagnostic null |
| retry_wait | start/deadline nonnull, finish null; positive finished failed attempt; result null, diagnostic nonnull |
| succeeded | All clocks nonnull; positive attempts; exact result nonnull, diagnostic null |
| failed | finish nonnull; start/deadline both null iff0 attempts; result null, non-cancel diagnostic |
| skipped | finish nonnull; start/deadline null;0 attempts; result null; dependency_failed or policy_changed |
| cancelled | finish nonnull; start/deadline both null iff0 attempts; result null; cancel_requested |

Zero attempts requires null last_attempt_id; positive requires the exact last matching attempt. Stage start<=finish<=deadline, except a qualified late timeout observation gives no additional work authority. All required stages must succeed before operation success. A successful evaluation operation may still return an E007 invalid evaluation receipt; the nonempty case outcomes remain exact.

Attempt ordinals are contiguous public counters and IDs distinct, independent of Temporal run/attempt IDs. running has null finish/failure; succeeded has finish and null failure; failed has finish and transient/permanent failure; uncertain has finish and uncertain failure; cancelled has finish and null failure. Every engine attempt records its actual service/audit attribution.

```text
effect_entry = {
  effect_id, stage_id:identifier|null, committed_at,
  authority_audit_event_id, value:canonical_effect
}
canonical_effect =
  {kind:'original_sealed', original:acquisition.original_reference} |
  {kind:'representation_created', representation:observation.representation_reference} |
  {kind:'observation_created', observation:observation.observation_reference} |
  {kind:'candidate_created', candidate:candidate_review.candidate_reference} |
  {kind:'expense_created', expense:financial.expense_reference} |
  {kind:'build_bound', build:build_reference} |
  {kind:'evaluation_committed', evaluation:evaluation_reference} |
  {kind:'approval_attestation_committed', approval:approval_reference,
     attestation:attestation_reference} |
  {kind:'export_artifact_bound', artifact:export_reference} |
  {kind:'lifecycle_event_committed', event:lifecycle_event_reference} |
  {kind:'capture_analysis_committed', analysis:capture_analysis_reference} |
  {kind:'lifecycle_material_outcome_committed', confirmation:lifecycle_material_confirmation_reference} |
  {kind:'restriction_propagation_committed', propagation:propagation_record_reference}

exposure_reference = {organization_id, domain_id, exposure_id}
exposure_entry = {
  exposure_id, stage_id, attempt_id,
  profile:profile_reference, policy:exposure_policy_reference, input_sha256,
  authority_audit_event_id, admitted_at,
  state:'attempted'|'completed'|'uncertain'|'confirmed_not_performed',
  finished_at:timestamp|null,
  receipt:{kind:'pending'}|{kind:'retained',receipt_id,receipt_sha256},
  settlement:'reserved'|'settled'|'unsettled',
  reconciliation:exposure_reconciliation
}
exposure_reconciliation = {
  state:'not_required'|'pending'|'resolved'|'unresolved',
  attempts_started:0..256, last_checked_at:timestamp|null,
  next_attempt_at:timestamp|null, deadline:timestamp|null,
  diagnostic:stage_diagnostic|null
}
effects_snapshot = {kind:'complete', entries:effect_entry[0..256]}
exposures_snapshot = {kind:'complete', entries:exposure_entry[0..256]}
```

exposure_policy_reference is an origin-selected exact union: existing context.policy_reference for ordinary processing/export/context exposures, scoped_policy_reference for lifecycle storage deletion exposures. Lifecycle action/exposure bindings use that same scoped encoding; no widening or duplicate interpretation of the E007 closed leaf. Exposure input/receipt bodies are exact profile-qualified retained bytes; E010/first gateway or E050/storage-adapter owner must define complete typed producer/verifier schemas and any promised read before use.

Effects are all actually committed consequential records, including admission-time purge request and inherited source revocation in its internal operation. Audit/outbox/reservations/leases are required bookkeeping rather than extra user business effects. One approval/attestation pair is one effect. Entries are append-only, exact duplicate set members rejected, unique identities with differing bytes rejected semantically. effects.kind=complete describes the entire ledger at the observed operation control. It does not claim all external outcomes are known.

| Exposure state | Exact branch |
| --- | --- |
| attempted | finish null; pending receipt; reserved; reconciliation not_required |
| completed | finish and retained receipt; settled with not_required/resolved, or unsettled with pending/unresolved |
| uncertain | finish (end of local observation); pending/retained receipt; reserved/unsettled; pending/unresolved |
| confirmed_not_performed | finish and affirmative retained receipt; settled; resolved |

Timeout, cancellation, silence and lease expiry never prove not_performed or zero cost. Preserve uncertain liability. No return to attempted or settled→reserved. New affirmative evidence may settle uncertainty; bounded immutable evidence replacements preserve prior contradictory evidence and exact input/attempt binding. Reconciliation not_required has0 attempts and all nullable fields null; pending has next/deadline and null diagnostic, last_checked null iff0 attempts; resolved has last/deadline, next/diagnostic null; unresolved has last/deadline, next null, nonnull diagnostic. resolved/unresolved can have0 attempts when authoritative observation needs no further outbound attempt. All nonnull reconciliation deadlines equal the fixed operation deadline. Summary priority is pending, unresolved, resolved if any was required, otherwise not_required.

```text
operation_control = {
  control_revision:control_revision,
  work_admission:'open'|'closed',
  cancellation:{kind:'none'}|
    {kind:'requested',cancellation_id,requested_at,reason},
  reconciliation:'not_required'|'pending'|'resolved'|'unresolved'
}
operation_execution = {
  plan:execution_plan_reference,
  timing:operation_timing,
  stages:stage_snapshot[1..64], attempts:attempt_record[0..256]
}
operation_resource = common.operation + required {
  command:operation_kind, context:command_context, origin:operation_origin,
  causal_command:historical_command_cause,
  execution_attribution:origin-selected attribution,
  control:operation_control, execution:operation_execution,
  effects:effects_snapshot, exposures:exposures_snapshot
}
```
operation_resource is additive32 and selects its exact result by command. upload.finalize and capture.analyze additionally require their closed input. lifecycle.purge requires purge_progress and material_actions; lifecycle.propagate requires obligation and material_actions. Other kinds forbid those lifecycle-only fields as reserved extensions. Common queued/running/failed/cancelled/succeeded branches remain exact. queued has null start/terminal and no business attempts; running has start and no terminal; terminal has terminal_at and closed work admission. succeeded alone has result; failed alone has problem; cancelled has neither. All five retain complete ledgers and can have committed admission effects.

Control begins1 and increments exactly once per canonical observable combined state/attempt/progress/cancellation/ledger mutation, never on identical updates/polls. Publish one consistent revision, pre-reserve worst-case revision headroom, prohibit oscillation/unbounded progress. Terminal state/time/result/problem never changes; only bounded already-admitted reconciliation and the narrow lifecycle confirmation in section 9 may advance afterward.

```text
worker_execution_attribution = {
  executor:context.principal_reference with kind='service',
  delegation_id:null,
  authority_audit_event_id:id
}
historical_command_cause = {
  kind:'command',
  command:common.command_name,
  context:common.command_context,
  execution:context.execution_attribution,
  admitted_at:timestamp
}
operation_origin =
  {kind:'command',command_id:id} |
  {kind:'lifecycle_event',event:lifecycle_event_reference}
historical_operation_cause = {
  kind:'operation', operation:operation_reference,
  command:operation_kind,
  context:common.command_context,
  origin:operation_origin,
  execution:worker_execution_attribution,
  causal_command:historical_command_cause,
  admitted_at:timestamp
}
historical_cause_snapshot =
  historical_command_cause | historical_operation_cause
lifecycle_action_execution =
  {kind:'command',cause:historical_command_cause} |
  {kind:'worker',execution:worker_execution_attribution,
   cause:historical_cause_snapshot}
event_action_attribution =
  {kind:'command',command:common.command_name,
   context:common.command_context,execution:context.execution_attribution} |
  {kind:'worker',execution:worker_execution_attribution,
   cause:historical_cause_snapshot}
```

For command origins, origin.command_id=causal_command.execution.command_id, command/context/execution_attribution equal the historical admitted command. For propagation, the operation's actual admitting service attribution is separate from the exact historical lifecycle.revoke cause; later attempts/actions record their own actual service. Worker branches require service principals and null delegation, never forge human command IDs. historical_operation_cause is nonrecursive and describes the actual current action's worker with its operation's original admission time. worker action execution equals its operation-cause execution where that branch is used. Automatic upload expiry can have the exact previously admitted allocation command cause plus actual worker authority; it does not invent an expiry command.

Historical snapshots survive detailed receipt GC under policy and continuity, establish provenance only, and never authorize current work. Prepared origins are allocated correlations, not admitted historical causes. Common command grammar remains distinct from the eight operation-kind enum; lifecycle.propagate is forbidden as a public command or historical_command_cause.command.

## 4. Cancellation

operation.cancel uses POST /v1/operations/{operation_id}/cancel. The route ID
must equal body.operation.operation_id, and route/reference organization/domain
must match the authenticated command context. Current action/disclosure checks
precede the target-control and disposition decisions below.

```text
operation.cancel request = {
  metadata:command_metadata(command='operation.cancel'),
  body:{operation:operation_reference,
        expected_control_revision:control_revision, reason:string1..2048}
}
cancellation_core = {
  kind:'arboresce.operation-cancellation.v1', cancellation_id,
  operation:operation_reference, context:command_context,
  expected_control_revision:control_revision,
  observed_control_revision:control_revision,
  disposition:'requested'|'already_requested'|'already_terminal',
  observed_state:'queued'|'running'|'succeeded'|'failed'|'cancelled',
  reason:string1..2048, recorded_at,
  execution:execution_attribution
}
```

Define cancellation_record={core:cancellation_core,cancellation_sha256:sha256}; hash the complete core. The atomic200 additive result is {command:'operation.cancel',cancellation:cancellation_record,operation:operation_resource}, with its own terminal receipt. Eligible replay returns the original record and original observed target snapshot; GET separately reads current state.

After fresh current authority/context checks, compare expected target control first: mismatch409 even if the target just became terminal. With equal control, terminal takes priority and returns already_terminal unchanged; a prior request returns already_requested unchanged; otherwise queued/running becomes requested and closes new business admission with exactly one target increment. The own attributable cancellation record/receipt/audit commits in all three dispositions. observed_* describe the resulting snapshot; requested observed_control=expected+1, observational dispositions equal expected. Immediate queued cancellation can share that one increment; otherwise bounded shutdown finishes later.

Success and cancellation use the same final fence. If success wins, stale cancel409 then fresh already_terminal. If cancel wins, no new business stage/action dispatch/success bind; previously admitted irreversible actions may later be confirmed exactly as section9. Client disconnect/poll timeout never cancels implicitly. Cancellation preserves source revocation, purge responsibility, completed stages, sealed results and conservative liabilities; no hidden restart or key replacement.

## 5. Acquisition and analysis

Existing upload/capture requests are preserved. Define E008 upload_resource as acquisition.upload_resource plus required control_revision:common.control_revision and the state-specific reachable-control constraints below, still additive32. All E008 upload reads/accepted/completed/abort resources use this specialization; existing E003 consumer definitions remain intact. New E008 consumers require the observed upload_resource.control_revision; it begins1 at allocation. Each actual upload state transition increments once, while reads, progress and duplicate delivery do not. created→uploading or uploaded, uploading→uploaded, uploaded→sealing, sealing→sealed; unsealed sessions may become expired/rejected under their qualified exact rules. upload.abort handles created/uploading/uploaded; cancellation of sealing uses its finalizer operation. All terminal session states are irreversible. Minimum controls are2 for uploading/uploaded/pre-sealing terminal,3 for sealing,4 for sealed. Upload generation remains its distinct existing tagged type.

```text
POST /v1/uploads/{upload_id}/finalize
{
  metadata:command_metadata(command='upload.finalize'),
  body:{upload_id:id,expected_generation:acquisition.upload_generation,
        expected_source:acquisition.declared_original}
}
```
```text
upload_finalize_input = {
  upload_id:id, organization_id:id,
  generation:acquisition.upload_generation,
  source:acquisition.declared_original
}
upload_finalize_operation_result = {
  command:'upload.finalize',
  upload:sealed_upload_snapshot
}
```
```text
finalization =
  {kind:'operation',operation:operation_reference} |
  {kind:'existing_binding'}
completed_finalize_result = {
  upload:sealed_upload_snapshot,
  finalization:finalization
}
```

sealed_upload_snapshot is the existing acquisition.upload_resource constrained sealed with required E008 control_revision. All allocation/generation/declared-source/verified-byte/original-binding identities match. Admission atomically sets sealing and stores its one operation/plan/recognition/audit/outbox. Final original binding, original_sealed effect, material initialization, sealed state and operation success share the final current-authority fence.

upload_finalize_operation_result and completed_finalize_result are additive32 transport results. Harmless bounded extension fields are accepted through32 total members; a33rd member and reserved aliases are rejected. Their required upload/finalization fields retain the exact declared types; this reader compatibility does not open any closed canonical core or reference.

| upload.finalize observation | Response |
| --- | --- |
| Initial nonterminal admission | Existing202 top-level operation, pending receipt, sealing upload; Location exact operation |
| Same-key nonterminal POST replay | Same202 with current nonterminal operation, its pending originating receipt and sealing upload |
| Original command direct completion or successful POST replay | Existing200 top-level terminal receipt; result:completed_finalize_result, finalization.kind=operation |
| Same-key failed/cancelled POST replay |200 result {operation,idempotency_receipt}; no top-level receipt |
| GET operation, any state |200 result {operation,idempotency_receipt}; originating receipt belongs to origin.command_id |
| Fresh new key against already sealed exact binding | Existing permitted atomic200 lookup; own new terminal receipt; finalization.kind=existing_binding |

The existing-binding lookup creates no operation, original, material control or business effect, and cannot alias the prior operation's receipt. Exact bytes/generation must match; competing live finalize under another key conflicts. Expired replay remains409 rather than a fresh lookup. Successful direct replay preserves the recorded sealed snapshot and original finalization reference after a lost202. No model analysis is part of sealing.

```text
POST /v1/uploads/{upload_id}/abort
{
  metadata:command_metadata(command='upload.abort'),
  body:{upload_id:id,expected_generation:acquisition.upload_generation,
        expected_control_revision:control_revision,reason:string1..2048}
}
upload_abort_result = {
  command:'upload.abort', upload:aborted_upload_snapshot
}
```

upload.abort returns own atomic200 result {command:'upload.abort',upload:aborted_upload_snapshot}; aborted_upload_snapshot is acquisition.upload_resource state=aborted with its E008 control. All route/org/generation/control joins apply. sealing conflicts409 and requires operation.cancel; sealed/expired/rejected/aborted reject fresh state mutation. Replay returns the original snapshot. Abort may leave residual staging grants/copies under finite cleanup responsibility, but they cannot bind an original afterward. No raw staging URL or returned grant is treated as recalled.

upload_abort_result is additive32 with the common bounded extension-value and reserved-alias rules; its required command and upload fields remain exact.

GET /v1/uploads/{upload_id} returns {upload:upload_resource}; GET /v1/captures/{capture_id} returns {capture:acquisition.capture_resource}; GET /v1/captures/{capture_id}/revisions/{version} returns {capture:acquisition.capture_revision}. These additive32 common200 results have no command receipt and no implicit listing/head substitution for the exact revision. The version route decodes the existing exact revision value with common selector rules.

```text
POST /v1/captures/{capture_id}/analyses
{
  metadata:command_metadata(command='capture.analyze'),
  body:{capture:capture_reference,
        expected_control_revision:control_revision}
}
capture_analysis_input = {
  capture:capture_reference,
  submission_command_id:id,
  submitted_control_revision:control_revision,
  profile:context.profile_reference
}
capture_analysis_reference = {
  organization_id:id,domain_id:id,analysis_id:id,
  analysis_sha256:digest
}
capture_analysis_result = {
  command:'capture.analyze', capture:capture_reference,
  analysis:capture_analysis_reference
}
```

capture.submit stays atomic200 and creates no processing operation on admission, replay, or later attachment readiness. capture.analyze is the explicit separately authorized trigger. Fresh admission requires the exact current submitted capture revision/control, all required parts ready, one qualified selected processing profile and a fixed ordered input-eligibility snapshot; otherwise409. A submitted incomplete capture remains valid, but cannot be analyzed yet. At most one live analysis of an exact capture; a different key conflicts while live. Admission does not increment the capture's control; it starts the separate operation control.

capture_analysis_result is additive32 with the common bounded extension-value and reserved-alias rules. Its command, capture and analysis fields retain their exact types; capture_analysis_core and its record/reference remain closed.

submission_command_id and submitted_control_revision come from the canonical immutable submission command/audit association, retained independently of detailed idempotency receipts. No live receipt lookup is required. Missing attribution or unknown permitted retention denies dependent work. A later capture draft does not mutate an admitted operation's old submitted input; current authority still fences work. Optional omitted parts remain explicit and later readiness never appends them to this plan. A deliberately new authorized intent after terminal work can use current eligible state; no automatic fresh key or operation.

capture.analyze uses ordinary initial202 and subsequent command-origin200 replay/GET wrappers. Its terminal record is:
```text
capture_analysis_core = {
  kind:'arboresce.capture-analysis.v1', analysis_id:id,
  operation:operation_reference,
  input:capture_analysis_input,
  recorded_at:time,
  disposition:'required_work_completed',
  parts:analysis_part_outcome[1..8],
  outputs:analysis_output_reference[0..256]
}
analysis_part_outcome = {
  index:integer0..7, intent:acquisition.attachment_intent,
  disposition:'completed'|'omitted_optional',
  reason:null|'not_ready'|'processing_failed'|'policy_changed'
}
analysis_output_reference =
  {kind:'representation',reference:observation.representation_reference} |
  {kind:'observation',reference:observation.observation_reference} |
  {kind:'candidate',reference:candidate_review.candidate_reference} |
  {kind:'expense_proposal',reference:financial.expense_reference}
capture_analysis_record = {
  core:capture_analysis_core,analysis_sha256:digest
}
```

parts has every submitted attachment exactly once in original index order,1..8. Required parts must be completed with null reason; omitted_optional requires a nonnull finite reason and an optional attachment intent. No invented successful required part. outputs lists every created representation/observation/candidate/expense proposal with exact existing owner references, no accepted asset or approval. Empty outputs can be valid explicit abstention. Combined effect capacity reserves the analysis record effect and each output; a syntactic256-output array does not promise that many fit one runtime operation. All required work and record bind atomically with capture_analysis_committed and success; failed partial output effects remain in the operation without a success manifest.

GET /v1/capture-analyses/{analysis_id} returns {analysis:capture_analysis_record}, additive32 result/no receipt. SHA256(JCS(core)) defines analysis_sha256. E010 defines/qualifies complete concrete stage input/output/result profiles; E086 qualifies actual processing adapters/gateway integration. E072/E073 deterministic test Activities and E074 synthetic histories qualify source/orchestration only, never advertise runnable real capture.analyze before E086. This preserves the forward dependency graph.

## 6. Lifecycle subjects, controls and commands

lifecycle_subject is exactly this closed union; all listed fields are required:

| kind | Other fields | Scope |
| --- | --- | --- |
| original_version | organization_id:id, artifact_id:id, version_id:id | organization |
| representation | organization_id:id, representation_id:id | organization |
| observation | organization_id:id, observation_id:id | organization |
| capture_revision | organization_id:id,domain_id:id,capture_id:id,version:content_revision | domain |
| candidate_revision | reference:candidate_review.candidate_reference | reference context domain |
| accepted_asset_version | organization_id:id,domain_id:id,asset_id:id,version_id:id | canonical accepted domain |
| expense_revision | reference:financial.expense_reference | reference domain |
| expense_report | reference:financial.report_reference | reference domain |
| context_build | reference:context.build_reference | reference domain |
| export_artifact | organization_id:id,domain_id:id,export_id:id | domain |

Exact canonical lookup resolves the complete existing owner identity. No mutable head, missing version, generic blob selector, organization deletion or membership mutation is introduced. Organization-scoped subjects retain the authenticated invoking domain without becoming domain-owned; mutation requires authority over the whole exact shared subject/global restriction. One consuming-domain membership grants no such authority and results cannot leak other domain identities/counts.

```text
organization_policy_reference = {
  policy_scope:'organization', organization_id:id, policy_id:id,
  version:version, content_sha256:digest
}
domain_policy_reference = {
  policy_scope:'domain', organization_id:id, domain_id:id, policy_id:id,
  version:version, content_sha256:digest
}
scoped_policy_reference = organization_policy_reference | domain_policy_reference
integrity_profile_reference = {
  organization_id:id, profile_id:id, version:version, content_sha256:digest
}
```
```text
lifecycle_control = {
  subject: lifecycle_subject,
  control_revision: revision,
  restriction: unrestricted | revoked,
  bytes_state: available | unavailable | purged,
  hold_state: none | held,
  purge_state: none | requested | running | blocked | partial | complete,
  last_event: lifecycle_event_reference
}
lifecycle_event_reference = {
  organization_id:id, event_id:id, event_sha256:digest
}
lifecycle_event_core = {
  kind:'arboresce.lifecycle-event.v1', event_id:id,
  subject:lifecycle_subject,
  previous_control_revision:revision | null,
  control_revision:revision,
  action: initialize | revoke | hold_place | hold_release | purge_request |
          purge_start | purge_block | purge_progress | purge_complete |
          bytes_unavailable | bytes_restored,
  restriction:unrestricted | revoked,
  bytes_state:available | unavailable | purged,
  hold_state:none | held,
  purge_state:none | requested | running | blocked | partial | complete,
  hold_id:id | null,
  purge_id:id | null,
  policy:scoped_policy_reference,
  execution:event_action_attribution,
  reason:reason,
  recorded_at:time,
  journal_intent_id:id
}
```
```text
event_admission_core = {
  kind:'arboresce.event-admission.v1',
  event:lifecycle_event_reference,
  admitted_at:timestamp,
  execution:lifecycle_action_execution
}
event_admission_record = {
  core:event_admission_core, admission_sha256:sha256
}
lifecycle_event_record = {
  core:lifecycle_event_core, event_sha256:sha256,
  admission:event_admission_record
}
```

event_sha256 hashes its complete closed core; admission_sha256 hashes its separate complete admission core. All repeated state/subject/revision values equal the committed control. initialize alone has null previous revision and control1. Noninitial events increment once and retain their exact previous control. Event IDs/digests alone prove no sequence or grant.

Builds and expense reports reuse the existing availability control row/counter, so E007 build control and E006 export/report preconditions see the same counter. Exact candidate/expense revision lifecycle controls are distinct from their aggregate review/content controls; all relevant writers still share final authority/material fences. Other exact material subjects initialize one lifecycle control at registration. Historical/voided financial mutation prohibitions govern financial content/review transitions. Independent restriction/hold/purge remains permitted without rewriting the immutable revision.

unrestricted means no direct lifecycle revocation only. Upstream restrictions, expiry, applicability, trust and current grants still apply. No un-revoke or purged-byte restoration exists. Direct revoked assets retain the existing revoked mapping; deprecation remains distinct. Revoked builds are unavailable even if their bytes exist. bytes_state observes this subject's own material, not dependency eligibility.

| Action | Prior condition | Result |
| --- | --- | --- |
| initialize | New exact material | unrestricted/available/none/none, control1 |
| revoke | unrestricted, not purged | restriction revoked; other states unchanged |
| hold_place | not purged; no unresolved destructive handoff; permitted hold | held; an interrupted admitted purge becomes blocked atomically |
| hold_release | exact active hold; both expected controls | held iff another active hold remains; other states unchanged |
| purge_request | revoked/not purged; no active hold/reference/lease/policy blocker; no competing live purge | none hold/requested purge |
| purge_start | requested/blocked/partial under same live operation; blockers absent | none hold/running purge |
| purge_block | requested/running; new blocker; no unchecked destructive outcome | blocked |
| purge_progress | running; known copy outcome with incomplete obligations | partial |
| purge_complete | running; all fixed obligations/continuity satisfied | revoked/complete; bytes purged iff own material no longer reconstructable; none hold |
| bytes_unavailable | available; qualified observed loss/corruption | unavailable |
| bytes_restored | unavailable, never purged; qualified exact old identity | available |

No other transition is admitted. Unchanged columns preserve exact values. Hold and requested/running purge cannot coexist. hold_place identifies an interrupted purge_id in that same event; release never restarts it. Terminal failed/cancelled partial/blocked purge permits a deliberately fresh purge intent/new ID with current controls and retained receipts, not repeated known deletion. The narrow late-confirmation branch in section 9 may update an already-admitted purge's truthful partial/complete observation under fresh shared fences, including blocked/partial state; it never rewrites a newer purge/control or terminal operation result. purge_complete still requires all current hold/reference/policy/continuity conditions.

Reachable minima: revoke/hold_place>=2; hold_release>=3; purge_request>=3; purge_start>=4; ordinary purge_complete>=5. hold actions require hold_id; purge actions require purge_id; all others null except hold_place interrupting an existing purge. Encode those finite action/state/minimum/null branches structurally; actual increments and remaining-hold/exact-copy joins are semantic.

| Command | POST route | Closed body |
| --- | --- | --- |
| lifecycle.revoke | /v1/lifecycle/revocations | {subject:lifecycle_subject,expected_control_revision:control_revision,reason:string1..2048} |
| lifecycle.hold.place | /v1/lifecycle/holds | same three fields |
| lifecycle.hold.release | /v1/lifecycle/holds/{hold_id}/release | {subject,hold_id:id,expected_control_revision,expected_hold_revision:control_revision,reason} |
| lifecycle.purge | /v1/lifecycle/purges | same three fields as revoke |

The first three are atomic200, with own terminal receipt and additive result {command,control:lifecycle_control,event:lifecycle_event_reference,hold:hold_resource|null,propagation:operation_reference|null,obligation:propagation_obligation_reference|null}. Revoke requires hold=null and nonnull propagation/obligation. Hold commands require hold and null propagation/obligation. Revoke commits immediate canonical denial, exact snapshot obligation, one initial operation, association/audit/outbox/continuity; it does not wait for cleanup enumeration or completion. Unknown/oversized later cleanup never undoes or prevents canonical revocation.

Fresh already-revoked revoke is422; stale control, incompatible release or purged mutation409. Unknown required policy/continuity denies503. Eligible atomic replay returns original event/control/hold/operation/obligation observations and receipt without new work or fresh revision tests. lifecycle.purge returns202 with its durable purge_request effect and follows normal command-origin operation replay.

```text
hold_core = {
  kind:'arboresce.lifecycle-hold.v1', hold_id:id,
  subject:lifecycle_subject, policy:scoped_policy_reference,
  placed_at:time, execution:execution_attribution, reason:reason
}
hold_resource = {
  core:hold_core, hold_sha256:digest,
  control_revision:revision, state:active | released,
  released_event:lifecycle_event_reference | null
}
purge_reference = {organization_id:id, domain_id:id, purge_id:id}
purge_manifest_reference = {
  organization_id:id, purge_id:id, manifest_sha256:digest
}
purge_progress = {
  purge:purge_reference, subject:lifecycle_subject,
  state:requested | running | blocked | partial | complete,
  block: null | active_hold | surviving_reference | active_lease |
         policy_unavailable | continuity_unavailable | external_outcome_unknown,
  receipt_manifest:purge_manifest_reference | null
}
lifecycle_purge_result = {
  command:'lifecycle.purge', purge:purge_reference,
  subject:lifecycle_subject, receipt_manifest:purge_manifest_reference,
  control:lifecycle_control
}
```

hold_resource is closed: active requires control1 and null released_event; released requires control2 and exact release event. No expiry, third transition or hold-as-read-grant. Multiple holds require a selected finite active-hold capacity and no exposed unauthorized count.

lifecycle_purge_result is additive32 with the common bounded extension-value and reserved-alias rules. Its command, purge, subject, receipt_manifest and control fields retain their exact types; their closed canonical records/references do not gain extension fields.

purge_progress is required on its operation in every state. Its exact finite branch preserves requested/running/blocked/partial/complete; blocked requires a nonnull block, requested/running/complete require null block, and partial carries null or its actual current blocker. receipt_manifest is null until actual exact retained copy receipts exist, nonnull for complete. Succeeded alone has lifecycle_purge_result with complete obligations; failure/cancellation retains request, partial effects/exposures and progress.

Purge acts only on exact registered organization/material/storage-class/opaque-object-version/digest bindings, never hash-only deletion. Freeze the complete required-copy obligation before dispatch; the qualified E050/storage profile defines its finite complete manifest/proof bytes and cannot drop required items. Each irreversible handoff rechecks current holds, surviving references, leases, policy, continuity and exact conditional version at a short fence; slow I/O stays outside DB locks. Hold/reference writers cannot claim protected success while a prior destructive handoff is unresolved; conflict or bounded wait outside locks preserves that race. Lease expiry is not abort proof, replacement bytes never inherit an old delete, and unknown external outcomes remain conservative.

Existing financial expense/report GET remains complete-or-error: no null/truncated immutable source snapshots to hide retention loss. Read-only POST /v1/lifecycle/resolve has closed {context:command_context,subject:lifecycle_subject} and common200 result {control:lifecycle_control}. GET /v1/lifecycle/events/{event_id} returns {event:lifecycle_event_record,obligation:propagation_obligation_reference|null}: revoke nonnull, other actions null. GET /v1/lifecycle/holds/{hold_id} returns {hold:hold_resource}. These exact additive32 results have no command receipt and do not infer access from retained metadata.

## 7. Prepared event and committed admission ordering

Prepared event_action_attribution contains exact command/context/E007 executor/audit correlation without unknown admitted_at, or actual worker attribution with an already committed historical cause. Event recorded_at is the immutable pre-ack construction clock observation and equals intent.prepared_at; it is not durability time or admission proof. Include journal_intent_id before JCS hashing. Independently acknowledge exact intent bytes outside canonical DB locks. Changing prepared bytes requires a new intent.

Every lifecycle-family intent requires prepare_expires_at=prepared_at+the qualified integrity profile's positive finite intent_prepare_lifetime_ms, checked for representability. The final fenced decision must occur strictly before expiry. Expiry does not prove abort. The short final transaction samples actual admitted_at, creates the small closed admission record and atomically binds event/command-or-worker admission/effects/audit/outbox/relations. This timestamp is the successfully committed transaction's final admission-decision observation, not a predicted physical durability instant. Large cores, snapshots and proof bytes are prepared outside locks; only the bounded admission core is hashed inside.

For command events, admission.execution.kind=command and its historical cause projection exactly equals the prepared command/context/execution; cause.admitted_at=admission.admitted_at. For worker events, admission.execution equals the prepared worker branch, with actual current authority/audit; historical cause time remains its old admission. Always recorded_at<=admission.admitted_at<prepare_expires_at. The committed independent resolution binds both exact event hash and full admission record plus complete qualified canonical association. A missing resolution leaves continuity unresolved; an old backup's missing row or an intent clock cannot replace it.

## 8. Revocation obligation, bounded cleanup and exact material actions

```text
controlled_material_reference = {
  organization_id:id, material_id:id, version_id:id, content_sha256:sha256
}
propagation_inventory_snapshot_reference = {
  organization_id:id, inventory_id:id, snapshot_id:id, snapshot_sha256:sha256
}
propagation_obligation_item = {
  item_id:id,
  subject:lifecycle_subject,
  target:controlled_material_reference,
  required_action:'invalidate_projection'|'delete_cache_copy'
}
propagation_obligation_core = {
  kind:'arboresce.lifecycle-propagation-obligation.v1', obligation_id:id,
  context:command_context,
  source_event:lifecycle_event_reference,
  subject:lifecycle_subject,
  source_control_revision:control_revision,
  source_command:event_action_attribution with kind='command',
  policy:scoped_policy_reference,
  profile:context.profile_reference,
  created_at:timestamp,
  scope:{kind:'dependency_snapshot',
    snapshot:propagation_inventory_snapshot_reference}
}
propagation_obligation_reference = {
  organization_id:id, domain_id:id, obligation_id:id,
  obligation_sha256:sha256
}
propagation_obligation_record = {
  core:propagation_obligation_core, obligation_sha256:sha256
}
propagation_repair_reference = {
  organization_id:id, domain_id:id, repair_id:id, repair_sha256:sha256
}
propagation_repair_batch_reference = {
  organization_id:id, domain_id:id, batch_id:id, batch_sha256:sha256
}
propagation_coverage_reference = {
  organization_id:id, domain_id:id, coverage_id:id, coverage_sha256:sha256
}
propagation_manifest_core = {
  kind:'arboresce.lifecycle-propagation-manifest.v1', manifest_id:id,
  obligation:propagation_obligation_reference,
  snapshot:propagation_inventory_snapshot_reference,
  operation:operation_reference,
  profile:context.profile_reference,
  items:propagation_obligation_item[0..256],
  enumerated_at:timestamp,
  execution:worker_execution_attribution
}
propagation_manifest_reference = {
  organization_id:id, domain_id:id, manifest_id:id, manifest_sha256:sha256
}
propagation_manifest_record = {
  core:propagation_manifest_core, manifest_sha256:sha256
}
```

Revoke freezes an immutable server-issued complete material/dependency inventory snapshot outside locks, then compares its exact canonical head/version at the final restriction/writer fence and binds obligation plus one operation. A mismatch requires fresh preparation. The retained snapshot identity/bytes-root/profile interpretation defines the complete dependency predicate; it is not a live head, arbitrary caller list, first256, count estimate or cursor. All serving and materializing writers enforce immediate restriction independently of cleanup.

The initial operation later enumerates that exact scope. Its complete manifest has0..256 unique items, sorted by ASCII item_id; identities and target/action pairs are unique. Unknown inventory, absent interpretation, over256 items, insufficient combined effect/action headroom or oversized bytes fails enumeration before any material-action admission, retaining canonical denial and a directly readable repairable incomplete obligation. Empty means positively proved complete empty inventory. One source event plus item confirmations plus completion already restricts such a256-effect operation to at most254 items before other effects. This is a per-operation cap, never an obligation-cardinality limit. E124 must finish larger inventories through explicitly admitted finite repair batches over this same immutable scope. No arbitrary organization/domain cardinality cap, truncated initial manifest, hidden continuation or repeated revoke is introduced.

The controlled-material reference resolves a server registration binding organization, owning subject, storage class, exact opaque object/version and full byte digest. Actual adapter mappings, conditional actions and inventory completeness are qualified by E050/storage before use; no raw locator or generic JSON is exposed. invalidate_projection only changes a derived projection-use barrier; delete_cache_copy removes a registered disposable cache, never a canonical source reclassified by caller input. Canonical source deletion requires explicit purge.

Obligation created_at equals source_event.recorded_at and source_command equals the prepared event command attribution. Its core has no operation/plan ID or unknown future admitted_at. Freeze event, obligation, then plan/operation association without a hash cycle. Actual causal admission is separately retained in the event admission and operation. manifest_sha256 and obligation_sha256 hash their complete named cores.

```text
lifecycle_material_action_scope =
  {kind:'propagation',obligation:propagation_obligation_reference,
   manifest:propagation_manifest_reference,item_id:id} |
  {kind:'propagation_repair',obligation:propagation_obligation_reference,
   repair:propagation_repair_reference,
   batch:propagation_repair_batch_reference,item_id:id} |
  {kind:'purge',purge:purge_reference,item_id:id}
lifecycle_material_action_core = {
  kind:'arboresce.lifecycle-material-action.v1', action_id:id,
  operation:operation_reference, stage_id:id, attempt_id:id,
  exposure_id:id|null,
  scope:lifecycle_material_action_scope,
  target:controlled_material_reference,
  action:'invalidate_projection'|'delete_cache_copy'|'delete_controlled_copy',
  policy:scoped_policy_reference, profile:context.profile_reference,
  authorized_at:timestamp, execute_expires_at:timestamp,
  execution:worker_execution_attribution
}
lifecycle_material_action_reference = {
  organization_id:id, domain_id:id, action_id:id, action_sha256:sha256
}
lifecycle_material_action_record = {
  core:lifecycle_material_action_core, action_sha256:sha256
}
material_actions_snapshot = {
  kind:'complete', entries:lifecycle_material_action_reference[0..256]
}
qualified_action_evidence_reference = {
  organization_id:id, evidence_id:id,
  profile:context.profile_reference,
  content_sha256:sha256, byte_length:integer1..65536
}
lifecycle_material_confirmation_core = {
  kind:'arboresce.lifecycle-material-confirmation.v1', confirmation_id:id,
  action:lifecycle_material_action_reference,
  outcome:'performed'|'not_performed',
  evidence:qualified_action_evidence_reference,
  asserted_performed_at:timestamp|null,
  recorded_at:timestamp,
  execution:worker_execution_attribution
}
lifecycle_material_confirmation_reference = {
  organization_id:id, domain_id:id, confirmation_id:id,
  confirmation_sha256:sha256
}
lifecycle_material_confirmation_record = {
  core:lifecycle_material_confirmation_core, confirmation_sha256:sha256
}
lifecycle_material_action_resource = {
  record:lifecycle_material_action_record,
  control_revision:control_revision,
  state:'admitted'|'uncertain'|'performed'|'not_performed',
  confirmation:lifecycle_material_confirmation_reference|null,
  observed_at:timestamp
}
```

Only lifecycle_material_action_resource is additive32; all other shown records/references/snapshots are closed. Every purge/propagate operation, and the later qualified E124 repair operation, requires material_actions, initially complete empty, sorted by action_id with unique exact refs; append atomically before dispatch and never remove uncertainty/failure/not_performed. The complete action GET makes local actions observable without fake provider exposure.

Initial propagation action scope exactly matches obligation/manifest/item/target/action. The reserved propagation_repair branch exactly matches obligation/repair/batch/item/target/action and has no producer until E124 qualifies its complete contracts. Both allow only invalidate_projection or delete_cache_copy. Purge scope matches its fixed item and only delete_controlled_copy. Each action names its actual declared stage/attempt. invalidate_projection requires exposure_id=null; deletion requires nonnull pre-reserved real storage exposure. For deletion input_sha256=action_sha256 and operation/stage/attempt/profile/scoped-policy/audit joins match. Local invalidation creates no provider call, cost or exposure.

authorized_at is the actually observed preparatory authority check; execute_expires_at is fixed inside remaining work/stage/attempt/authority lifetime. Final dispatch rechecks cancellation/fence. Unhanded-off actions cannot start after terminal/cancel fencing; already irreversible handoffs retain their exact uncertainty without a substitute action or renewed deadline.

Action control1/admitted/null confirmation may become control2/uncertain/null, or control2/performed/not_performed with confirmation; uncertain may become control3/performed/not_performed. Those are the only transitions. Each action has at most one immutable affirmative confirmation; identical redelivery is a no-op and contradictory evidence is an integrity fault. not_performed requires affirmative proof and null asserted_performed_at; performed permits null when downstream time is unknown. Nonnull asserted time is evidence's assertion, not inferred server truth. recorded_at observes verified evidence before final canonical binding.

qualified_action_evidence_reference binds exact retained closed profile-defined authentic evidence bytes,1..65536 encoded bytes. The concrete producer/verifier and actual service identity are mandatory before use; hash/length alone proves no outcome. For deletion receipts receipt_id=confirmation_id and receipt_sha256=confirmation_sha256; uncertainty remains conservative. Engine invalidation uses its qualified canonical barrier/commit proof and confirmation without an external exposure.

```text
propagation_proof_entry = {
  item_id:id,
  action:lifecycle_material_action_reference,
  confirmation:lifecycle_material_confirmation_reference
}
propagation_proof_core = {
  kind:'arboresce.lifecycle-propagation-proof.v1', proof_id:id,
  obligation:propagation_obligation_reference,
  manifest:propagation_manifest_reference,
  operation:operation_reference,
  profile:context.profile_reference,
  entries:propagation_proof_entry[0..256],
  verified_at:timestamp,
  execution:worker_execution_attribution
}
propagation_proof_reference = {
  organization_id:id, domain_id:id, proof_id:id, proof_sha256:sha256
}
propagation_proof_record = {core:propagation_proof_core,proof_sha256:sha256}
propagation_record_core = {
  kind:'arboresce.lifecycle-propagation.v1', propagation_id:id,
  operation:operation_reference,
  source_event:lifecycle_event_reference,
  obligation:propagation_obligation_reference,
  repair:propagation_repair_reference|null,
  proof:propagation_proof_reference|propagation_coverage_reference,
  policy:scoped_policy_reference,
  profile:context.profile_reference,
  completed_at:timestamp,
  basis:'work'|'late_confirmation',
  disposition:'canonical_obligations_satisfied',
  execution:worker_execution_attribution
}
propagation_record_reference = {
  organization_id:id, domain_id:id, propagation_id:id,
  propagation_sha256:sha256
}
propagation_record = {core:propagation_record_core,propagation_sha256:sha256}
propagation_operation_result = {
  command:'lifecycle.propagate', event:lifecycle_event_reference,
  propagation:propagation_record_reference
}
```

All records/cores/refs above are closed; propagation_operation_result is additive32. Their named hashes cover the whole corresponding JCS core. propagation_record_core is a closed union: repair=null requires proof:propagation_proof_reference and the immutable initial operation; nonnull repair requires proof:propagation_coverage_reference and that repair's actual final operation. Mixed proof/repair branches are invalid. Initial proof entries exactly equal the complete initial manifest item set, sorted, one matching performed confirmation per item, no extras and no unresolved competing action/exposure capable of invalidating completeness. All admitted actions remain in their ledger even if only the satisfying action appears in proof. Empty initial proof requires the genuinely complete empty initial manifest.

Exactly one immutable completion binds an obligation across both paths. basis=work binds with successful required stages and the actual completing operation's result. basis=late_confirmation applies only to that already failed/cancelled operation under section 9, creates no success result or accepted cancelled-stage result, and does not recall uncontrolled copies or renew grants. An initial completion may bind only before any repair admission. A repair completion requires repair equal the obligation's current repair head, its final range already admitted and complete authentic global coverage. Older operation confirmations stay on their original ledgers and become repair inputs; they cannot independently satisfy an obligation after a later repair is admitted. Initial completion uses the original policy/profile; repair completion uses its actual qualified grant's policy/profile with an explicit verified compatibility link to the original frozen snapshot interpretation.

The initial internal operation requires command=lifecycle.propagate, origin={kind:'lifecycle_event',event}, context, admitting worker execution_attribution, exact historical lifecycle.revoke causal_command, obligation and material_actions. Its effects always include the admission source revoke with original committed_at/audit; it is not falsely described as a new worker revocation. All operation/reference/plan/manifest/proof/completion and later repair/batch/coverage domains equal the source invoking context; org-scoped source does not become domain-owned. Source event must be revoke, its subject/control/ref and admission cause exactly match obligation and origin, and all source event references in read/result/completion agree. Repair retains the historical source association without claiming a new revoke effect; its actions and command/operation attribution record actual current execution.

The initial internal plan's required origin and obligation equal the operation. Each stage input is closed {kind:'arboresce.propagation-stage-input.v1',operation:operation_reference,obligation:propagation_obligation_reference,stage_id:id}, hashed as JCS. It contains no future manifest hash/time or array. Exactly three required stages with server-fixed IDs and consecutive dependency ordinals: enumerate/validate manifest (kind propagate_restriction, output propagation_manifest); apply exact actions and verify proof (same kind, output propagation_proof); bind completion (kind bind_result, output restriction_propagation). The later stages consume the exact predecessor accepted result under the fixed profile/input, never a mutable head. Even an empty scope executes complete inventory verification and required result binding. E124 owns its distinct finite range plan and complete stage inputs/results before repair activation.

```text
propagation_obligation_control = {
  control_revision:control_revision,
  state:'pending'|'enumerating'|'applying'|'incomplete'|'satisfied',
  operation:operation_reference,
  manifest:propagation_manifest_reference|null,
  repair:propagation_repair_reference|null,
  initial_failure_reason:null|'operation_failed'|'operation_cancelled',
  satisfaction:propagation_record_reference|null
}
propagation_obligation_resource = {
  record:propagation_obligation_record,
  control:propagation_obligation_control,
  observed_at:timestamp
}
```

Only the resource is additive32. operation always names the one immutable initial operation; manifest is its original manifest, null iff none was published. repair is null until first repair admission, then the latest admitted repair. Each immutable repair record retains its exact predecessor (null only for first), obligation and actual operation association. This linked history preserves every failed/cancelled attempt without an unbounded inline array or discarded ledger. E124 must define those complete records and reads before returning any repair reference.

Initial transitions are pending→enumerating→applying→satisfied or pending/enumerating/applying→incomplete. pending=control1 with queued original operation and null manifest; enumerating=control2 with running original and null manifest; applying=control3 with running original and nonnull manifest. All three require null repair/initial_failure_reason/satisfaction. Publishing initial manifest, accepted first-stage result and applying is atomic. Initial failure/cancellation sets initial_failure_reason once to match the terminal original operation and preserves it through every repair and later satisfaction. No repair resets the original operation, manifest, failure reason, denial or deadline.

Before repair admission, incomplete has null repair/satisfaction, failed/cancelled original and nonnull matching reason; null manifest permits controls2/3, nonnull manifest requires4. Initial ordinary satisfaction has null repair/reason, nonnull manifest/satisfaction, succeeded original and control4. Initial late satisfaction has null repair, nonnull manifest/satisfaction/reason, failed/cancelled original and control5. It is allowed only while repair remains null. There is no skipped enumeration success.

Fresh repair admission requires incomplete, terminal original, no satisfaction and no live competing repair batch. It CASes the exact expected control and previous repair head, publishes the next linked repair/operation, and increments control once. The obligation stays incomplete during active or failed repair; repair progress belongs to its actual operation. Each new operation has current actual authority, qualified profile, continuity, resource reservations and fixed finite deadlines. At most one live batch per obligation. No retry, lease, replay, GET or receipt renewal creates a repair. Explicit authorized automation may invoke repairs within its finite invocation/time/attempt/cost/storage budgets without a human prompt for every invocation.

Repair satisfaction atomically binds the unique completion, closes repair admission and changes incomplete→satisfied. It requires nonnull repair equal completion.repair, nonnull matching original failure reason and failed/cancelled original. Its initial manifest may remain null. Each actual admission/head advance or satisfaction increments once; reads and duplicate delivery never increment. Repair-incomplete reachable minima are3 with null initial manifest and5 with nonnull; repair-satisfied minima are4 and6 respectively. Those are minima, not fixed1..5 equations: later separately admitted attempts consume exactly one revision each. Reserve finite control/evidence headroom before each admission and deny exhaustion without wraparound or unrecorded work.

### Required bounded repair and complete coverage at E124

The original obligation/snapshot/policy/profile and initial operation remain immutable. Each repair binds a newly qualified profile and actual current grant, including verified compatibility with the original snapshot interpretation. It cannot silently replace old inputs or inherit expired authority. Each grant fixes finite batch/operation/time/attempt/cost/storage budgets and closes on termination. A later separately authorized grant may resume the retained verified prefix without renewing the earlier grant or its deadlines.

E124 must define the full concrete repair, batch, range-proof and coverage producer/verifier schemas before any repair mutation. Hashes in the three typed references bind exact retained canonical cores, never access grants or generic metadata. Construction must be acyclic: repair admission binds known operation IDs and predecessor, batch core binds fixed inventory range before any action, and later coverage binds completed proof without altering repair/batch hashes. Unknown future commit times or outcomes cannot enter prepared cores. Record bytes, current-authorized reads, registered command/result/operation kinds, stage contracts, all finite profile dimensions and continuation history are mandatory activation criteria, not optional later cleanup.

The retained original inventory has a qualified immutable ordering and membership proof over exact item identity, target and required action. A repair operation handles one contiguous range starting immediately after the authentic covered prefix, never a mutable head or caller-selected item list. Its complete range manifest, attempts/effects/exposures/actions/proof and late-confirmation reserve must fit combined wire and actual serialization budgets; select smaller ranges when bytes demand. There is no whole-obligation256-item ceiling. Initial over-cap/unknown enumeration remains repairable when authentic inventory and required service availability are restored.

E124 coverage is a closed union with a positively verified empty-full-inventory branch (no range or actions) and nonempty contiguous-range nodes. Empty does not mean unknown inventory. Nonempty nodes bind obligation, original snapshot identity/root, exact inclusive ordinal range, complete range-manifest hash, complete one-to-one item/performed-confirmation proof, actual repair/operation and predecessor coverage reference. First node begins at authenticated inventory start; every successor starts immediately after the proved predecessor end. No gaps, overlaps, extras, duplicate identities or unresolved competing effects are acceptable. A maximum observed ordinal/count alone is not coverage proof. Failed attempts remain in repair history even when they publish no coverage node.

Before each dispatch, inspect prior exact obligation/item/physical-version/action under a shared fence. Reuse a prior authentic performed confirmation as proof, never a newly dispatched action or newly committed effect in a later operation. An unresolved irreversible handoff blocks duplicate consequential action. Only affirmative not_performed permits a freshly authorized replacement with its own identity and actual execution. Reconciliation uses the original still-live finite budget or qualified passive receipt ingestion; a new repair grant cannot restart expired old-exposure probes. Deadline expiry, worker loss, lease expiry and a new grant never prove absence. Current holds, references, leases, version, policy and continuity are rechecked at each consequential boundary; replacement generations never inherit an old deletion.

Verify authentic predecessor and complete new range outside locks, then CAS the exact predecessor/current repair head and current material/authority fences. Non-final successful batches may commit their node and terminal batch result while the obligation remains incomplete; that distinct result reports the covered range and absence of global completion. The final node, unique global completion and successful repair terminal result bind atomically with obligation satisfaction. Thus no published full prefix is stranded without completion or uncovered work. Final late-confirmation binding performs the corresponding atomic head/completion/satisfaction update while preserving failed/cancelled terminal state/result/receipt. It may use only the current repair's already admitted final range and preallocated capacity; no post-terminal enumeration, new action or accepted cancelled-stage result is allowed. Confirmations of older repairs remain on their actual ledgers and may be reused by the current repair.

Final coverage binds authentic inventory end/count/root and the complete inductively verified contiguous chain back to start, or exact empty proof. Final canonical binding checks the verified head and fixed end in the short transaction; it does not rescan full inventory under DB locks. Each node, range manifest and proof has a complete bounded read; this explicit evidence graph does not paginate or truncate an operation ledger. Recovery must verify the entire authentic graph in bounded phases before opening the affected scope. Missing bytes/relations, altered prior proof, unknown cutoff or unverified predecessor keep it closed. A new range may adopt exact performed actions from an earlier failed partial range and prove all its own required items.

Exact current-authorized common200 reads, additive results/no receipts:
- /v1/lifecycle-propagation-obligations/{obligation_id}: {obligation:propagation_obligation_resource}
- /v1/lifecycle-propagation-manifests/{manifest_id}: {manifest:propagation_manifest_record}
- /v1/lifecycle-propagation-proofs/{proof_id}: {proof:propagation_proof_record}
- /v1/lifecycle-propagations/{propagation_id}: {propagation:propagation_record}
- /v1/lifecycle-actions/{action_id}: {action:lifecycle_material_action_resource}
- /v1/lifecycle-confirmations/{confirmation_id}: {confirmation:lifecycle_material_confirmation_record}

E124 must implement the following exact current-authorized common200 reads with additive32 results and no receipts before it returns any corresponding reference: /v1/lifecycle-propagation-repairs/{repair_id} returns {repair:<complete E124 repair record>}; /v1/lifecycle-propagation-batches/{batch_id} returns {batch:<complete E124 batch record>}; /v1/lifecycle-propagation-coverages/{coverage_id} returns {coverage:<complete E124 coverage record>}. E124 must replace these named producer obligations with its complete closed record contracts; These reserved references do not import a nonexistent schema or advertise enabled routes. Complete records or errors only, including predecessor/history and exact range-proof retrieval.

Event→obligation and all operation/plan/manifest/proof/action/confirmation relationships are exact retained associations with full current disclosure; deny an entire cross-domain read if complete fields are unavailable to the caller. The obligation stays directly observable after command receipt expiry and operation cancellation, subject to its actual qualified retention. Detailed receipt expiry never discards outstanding lifecycle responsibility.

## 9. Late confirmation and cancellation races

Only an exact material action/operation/stage/attempt/physical version and any real exposure admitted before the final handoff fence may be confirmed later. The confirmer requires current qualified evidence-recording authority, not inherited historical permission. Original finite reconciliation may obtain affirmative evidence, never issue another delete or reset time/budget. At its fixed deadline automatic probes stop. Later passive authenticated receipt ingestion is supported only once its own finite request/transaction/authority producer-verifier is qualified; it performs no remote business action.

One fenced transaction binds immutable confirmation, truthful copy/invalidation observation, real exposure settlement/reconciliation if applicable, lifecycle_material_outcome_committed effect, operation control increment, audit/outbox and independent continuity association. Preallocate full representation/revision capacity before action admission. Duplicate receipts consume nothing. Unverifiable evidence retains uncertainty/reservations and closed serving; it never becomes not_performed or zero cost.

Late purge evidence can advance exact-copy receipts and truthful subject purge_progress/purge_complete only under all current shared hold/reference/policy/continuity and old-obligation/newer-control fences. It cannot overwrite a newer purge's state or make an old failed/cancelled operation succeeded. Late complete initial propagation proof may bind basis=late_confirmation and restriction_propagation_committed only before any repair admission. A later qualified repair may bind late satisfaction only for its current head and already admitted final range under section 8's atomic coverage/completion rule. Other old confirmations remain on their original ledgers and become repair inputs. Terminal operation state/time/result/problem, cancelled stage results and idempotency expiry remain immutable. No new operation, hidden restart, fresh grant or extended reconciliation deadline is inferred from confirmation; separately admitted E124 repair is an explicit new authorized intent.

## 10. Sealed financial export and bounded delivery

Preserve the complete existing E006 expense_report.export request and report/payload/profile/format semantics. Its exact operation result and artifact are:
```text
export_artifact_core = {
  kind:'arboresce.financial-export-artifact.v1',
  export_id:id, organization_id:id, domain_id:id,
  report:financial.report_reference,
  payload_sha256:digest,
  format:financial-json-v1 | financial-csv-v1,
  media_type:application/json | text/csv,
  encoding:'utf-8',
  emitted:{sha256:digest, byte_length:integer 1..67108864},
  profile:profile_reference,
  sealed_at:time, expires_at:time
}
export_reference = {
  organization_id:id, domain_id:id, export_id:id,
  artifact_record_sha256:digest
}
financial_export_result = {
  command:'expense_report.export',
  report:financial.report_reference,
  artifact:export_reference
}
export_artifact_resource = {
  reference:export_reference,
  core:export_artifact_core,
  control:lifecycle_control,
  observed_at:time,
  availability:available | unavailable | expired | purged
}
```

export_artifact_resource and financial_export_result are additive32; core and reference are closed. artifact_record_sha256 hashes JCS(export_artifact_core), distinct from payload and emitted-byte hashes. financial-json-v1 requires application/json and emitted.sha256=payload_sha256; financial-csv-v1 requires text/csv and its actual reversible CSV byte digest. encoding is literal utf-8. Maximum sealed complete bytes is67108864 (64MiB), not a JSON control envelope allowance.

Generation writes a private complete staged artifact, verifies exact bytes/hash/length and current report/control/profile, then atomically binds immutable artifact/control1/operation effect/result/audit under the final current-authority fence. No unsealed partial stream is published. sealed_at observes completed seal; expires_at is fixed checked sealed_at+artifact_lifetime_ms, exclusive. Replays, later GET, delivery or new grants never regenerate/extend that artifact. A new deliberately keyed export is distinct authorized work.

GET /v1/export-artifacts/{export_id} returns common200 {artifact:export_artifact_resource}, no receipt. availability precedence is purged, then now>=expires_at expired, then unavailable for current denial/missing bytes, otherwise available; this observation grants no content access. Control.subject and all report/artifact/org/domain/hash/format/profile joins are exact.

```text
financial_export_delivery_profile = {
  kind:'arboresce.financial-export-delivery-profile.v1',
  artifact_lifetime_ms:positive_duration_ms,
  staging_lifetime_ms:positive_duration_ms,
  generation_total_ms:positive_duration_ms,
  generation_attempt_limit:integer 1..256,
  transfer_total_ms:positive_duration_ms,
  transfer_idle_ms:positive_duration_ms,
  transfer_buffer_bytes:integer 1..1048576,
  transfer_attempt_limit:integer 1..256,
  max_active_transfers_per_organization:integer 1..256,
  max_active_transfers_global:integer 1..65536,
  max_staged_bytes_per_operation:integer 67108864..9007199254740991,
  lifecycle_policy:scoped_policy_reference
}
positive_duration_ms = integer 1..9007199254740991
```

All listed fields are mandatory selected qualification inputs, not new defaults. idle<=total and organization slots<=global, representable clock arithmetic, generation/retry/staging capacity, actual full64MiB throughput and cleanup are qualified before support. Account for all temporary copies; uncertainty cannot free consumed reservations. A maximum duration representation does not promise it fits a deployed profile or current UTC date.

Content is a read-only POST /v1/export-artifacts/{export_id}/content with closed {context:command_context,artifact:export_reference,expected_control_revision:control_revision}. It has no idempotency metadata and returns the complete authenticated binary artifact, no Range/resume, redirect, compression, presigned URL or multipart variant. A stale control conflicts before body. Recheck current authenticated authority, subject/source restrictions, artifact/report/profile/expiry and shared slots; reserve delivery/audit before any bytes. Proxy admission is separate from the10second control deadline and uses the qualified finite bulk profile without a long DB transaction.

```text
export_delivery_receipt = {
  delivery_id:id, artifact:export_reference,
  emitted:{sha256:digest, byte_length:integer 1..67108864},
  context:context,
  executor:principal_reference, delegation_id:id | null,
  policy:scoped_policy_reference, profile:profile_reference,
  authority_audit_event_id:id,
  admitted_at:time, transfer_expires_at:time,
  delivery:'admitted'
}
```

This closed receipt records admitted delivery only. transfer_expires_at=min(fixed artifact expiry, known current authority expiry, admitted_at+transfer_total_ms, qualified server deadline), with monotonic duration enforcement. Idle activity never renews total; new grant/retry requires fresh current admission and delivery ID, cannot extend artifact expiry. Already admitted bounded streams follow the existing security rule for later revocation: they may finish within their fixed grant while every new admission is denied. Unknown outcomes retain ownership/capacity responsibility.

Exact response headers are Content-Type, Content-Length, Arboresce-Export-Id, Arboresce-Delivery-Id, Arboresce-Artifact-Sha256 and Arboresce-Transfer-Expires-At. Their values equal the exact format/length/IDs/emitted digest/receipt timestamp. Before body, errors use common problems; after bytes begin, abort transport and never append JSON or claim complete success. A client verifies full length and digest before committing a protected destination, never silently overwrites partial/existing content. GET /v1/export-deliveries/{delivery_id} returns common200 {delivery:export_delivery_receipt}, additive result/no command receipt; it is not a reusable capability.


## 11. Retention, recognition and independent continuity

```text
command_retention_limits = {
  replay_retention_ms: integer 604800000..9007199254740990,
  diagnostic_retention_ms: integer 2592000000..9007199254740991
}
```

The selected policy requires diagnostic_retention_ms>replay_retention_ms; both are terminal-relative durations, not absolute admission-relative expiration. The smaller replay maximum is intrinsic to that strict ordering beneath diagnostic max9007199254740991. Exact pair comparison, selected clock arithmetic and common-year representability are semantic qualification.7/30 days are permissible synthetic/minimum examples only; a longer replay policy requires a strictly later diagnostic deadline, not a fixed30-day maximum. No representation cap chooses actual legal/operational retention.

Pending operations retain null replay/diagnostic expiry pair and their necessary claim details throughout finite work. Terminalization fixes both once from terminal_at. now<replay expiry permits eligible POST replay; at/after it POST returns idempotency_expired409, even if a record still exists. GET is a fresh authorized read: after either detailed deadline it can return the complete retained operation and original retained receipt with replayed=false, without renewing eligibility. POST eligible replay marks replayed=true. If known authorized required details/receipt were actually removed, GET returns operation_details_unavailable409. No reconstructed receipt or missing effect count is fabricated. Transient/continuity unknown503 and unknown/undisclosable404 precede detailed disclosure.

```text
operation_details_unavailable_problem = common.problem with
  type:'urn:arboresce:problem:v1:conflict', code:'conflict', status:409,
  title:'Operation details unavailable',
  detail:'Required retained operation details are unavailable.',
  reason:'operation_details_unavailable'
idempotency_expired_problem = common.problem with
  type:'urn:arboresce:problem:v1:conflict', code:'conflict', status:409,
  title:'Idempotency key expired',
  detail:'The prior command cannot be replayed with this key.',
  reason:'idempotency_expired'
```
```text
recognition_scope = {organization_id, principal_id}
recognition_identity = {
  organization_id, principal_id, command, key_digest
}
recognition_reference = {scope:recognition_scope, recognition_sha256}
continuity_intent_reference = {organization_id, intent_id}
recognition_entry = {
  recognition:recognition_reference, admitted_at,
  intent:continuity_intent_reference
}
```
```text
origin_candidate =
  {kind:'command', command_id} |
  {kind:'operation', operation:operation_reference}
command_origin_candidate = {kind:'command', command_id}
command_admission_reference = historical_command_cause
continuity_abort_reference = {
  organization_id, intent_id, abort_id,
  proof_reference:{sha256, byte_length:integer 1..65536}
}
recognition_intent = {
  intent_id, organization_id, prepared_at,
  origin:command_origin_candidate,
  recognition:recognition_reference,
  policy:organization_policy_reference
}
recognition_resolution = {
  intent_id, organization_id, resolved_at,
  outcome:
    {kind:'committed', recognition:recognition_reference,
       admission:command_admission_reference} |
    {kind:'aborted', abort:continuity_abort_reference}
}
```

recognition_identity.command uses actual admitted registered common.command_name; all its unannotated IDs/digests use the notation in section 1. key_digest=SHA256(exact admitted ASCII opaque key). recognition_sha256=SHA256(UTF8('arboresce.command-recognition.v1') || one actual NUL byte || JCS(recognition_identity)); no normalization, newline, domain or purpose is added. The preimage is transient internal data. Retain recognition reference/scope, actual admission time and intent relation, with no raw key/body/fingerprint/result/receipt in the minimal index.

Domain/purpose stay in the existing complete request fingerprint, not recognition scope. Revocation/regrant, credentials, signing-key rotation and domain changes never create a new organization/principal claim scope. A collision conservatively denies new effect. Recognition/live claim/accepted operation or atomic effect/reservations/audit/outbox commit atomically. While that scope can admit commands recognition cannot be GC'd; finite actual storage/quota exhaustion denies before new effects instead of dropping identities.

Origin candidates are preallocated command/operation correlations, not historical admissions. Recognition accepts only command candidate; its committed resolution retains actual historical_command_cause, not guessed actor/time. An aborted intent has only qualified abort evidence, no invented command/event/admission placeholders. The abort proof must bind exact intent/origin and authoritative fence proving its transaction did not and can no longer commit. Missing HTTP response, timeout/lease expiry, allocated gap or absence from an old backup is insufficient. Exact duplicate resolution is idempotent; differing or committed+aborted resolution is corruption and closes affected admission.

```text
retirement_scope =
  {kind:'organization', organization_id} |
  {kind:'principal', organization_id, principal_id}
scope_retirement_reference = {
  organization_id, retirement_id, retirement_sha256
}
scope_retirement_core = {
  kind:'arboresce.command-scope-retirement.v1', retirement_id,
  scope:retirement_scope, retired_at,
  policy:organization_policy_reference,
  cause:historical_cause_snapshot,
  intent:continuity_intent_reference
}
scope_retirement_record = {core:scope_retirement_core, retirement_sha256}
scope_retirement_intent = {
  intent_id, organization_id, prepared_at,
  origin:origin_candidate, retirement_id, scope:retirement_scope,
  policy:organization_policy_reference
}
scope_retirement_resolution = {
  intent_id, organization_id, resolved_at,
  outcome:
    {kind:'committed', retirement:scope_retirement_reference} |
    {kind:'aborted', abort:continuity_abort_reference}
}
```

Retirement is irreversible closure of an organization ID or exact organization/principal pair under actual policy, not suspension/revocation/regrant. It shares the recognition admission fence: admitted-before-retirement claims remain accounted for; retirement winning prevents later effects. No public retirement command is invented. Later identity/retention owner qualifies the concrete registered command/operation trigger and actual cause before enabling this internal variant.

Per-key recognition may be removed only after permanent retirement is committed, independently durable under a complete qualified checkpoint, IDs are proved nonreusable and all retained claims/exposures/holds/policy obligations permit removal. The smaller retirement barrier/non-reuse evidence survives GC/restore/provisioning. Unknown actual policy or proof denies removal. scope_retirement_core.retired_at is the actual canonical retirement decision observation; its final exact record/hash is retained by the committed resolution profile, while its prepared intent contains no unknown future time.

Internal continuity uses one separate application library. Exact upload terminal subject/event:
```text
upload_session_journal_subject = {
  kind:'upload_session',organization_id:id,upload_id:id,
  generation:acquisition.upload_generation
}
upload_session_event_core = {
  kind:'arboresce.upload-session-terminal.v1',event_id:id,
  subject:upload_session_journal_subject,
  previous_control_revision:control_revision,
  control_revision:control_revision,
  action:'abort'|'expire'|'reject',
  state:'aborted'|'expired'|'rejected',
  problem:common.problem|null,
  policy:scoped_policy_reference,
  execution:event_action_attribution,
  recorded_at:time, journal_intent_id:id
}
upload_session_event_record = {
  core:upload_session_event_core,event_sha256:sha256,
  admission:event_admission_record
}
```
abort/expire/reject pair respectively with aborted/expired/rejected; reject requires common.problem, the others null. Controls increment and retain exact generation/session identity; expiry uses the actual qualified worker and previously admitted allocation cause, or actual finalizer operation cause where applicable. Upload-session continuity does not extend public lifecycle_subject or add a public event route.

The lifecycle intent/resolution pair covers a finite union of exact prepared event/action/evidence associations. It is not a seventh journal family or arbitrary JSON extension. This integration retains relations an older canonical backup may lack:
```text
continuity_binding_reference = {
  organization_id:id,binding_id:id,
  profile:integrity_profile_reference,
  content_sha256:sha256,byte_length:integer1..65536
}
initial_propagation_binding = {
  obligation:propagation_obligation_record,
  inventory:propagation_inventory_snapshot_reference,
  operation:operation_reference,
  plan:execution_plan_record
}
journal_prepared_payload =
  {kind:'material_event',core:lifecycle_event_core,
   propagation:initial_propagation_binding|null} |
  {kind:'upload_session_event',core:upload_session_event_core} |
  {kind:'material_action',record:lifecycle_material_action_record} |
  {kind:'material_confirmation',record:lifecycle_material_confirmation_record} |
  {kind:'propagation_manifest',record:propagation_manifest_record} |
  {kind:'propagation_completion',proof:propagation_proof_record,
   record:propagation_record with core.repair=null} |
  {kind:'propagation_repair_completion',coverage:propagation_coverage_reference,
   record:propagation_record with nonnull core.repair}
journal_committed_payload =
  {kind:'material_event',event:lifecycle_event_reference,
   admission:event_admission_record} |
  {kind:'upload_session_event',event:lifecycle_event_reference,
   admission:event_admission_record} |
  {kind:'material_action',action:lifecycle_material_action_reference} |
  {kind:'material_confirmation',
   confirmation:lifecycle_material_confirmation_reference} |
  {kind:'propagation_manifest',manifest:propagation_manifest_reference} |
  {kind:'propagation_completion',proof:propagation_proof_reference,
   propagation:propagation_record_reference} |
  {kind:'propagation_repair_completion',repair:propagation_repair_reference,
   coverage:propagation_coverage_reference,
   propagation:propagation_record_reference}
journal_intent = {
  intent_id:id,organization_id:id,origin:origin_candidate,
  prepared_at:timestamp,prepare_expires_at:timestamp,
  policy:scoped_policy_reference,
  payloads:journal_prepared_payload[1..256],
  prepared_binding:continuity_binding_reference
}
journal_resolution = {
  intent_id:id,organization_id:id,resolved_at:timestamp,
  outcome:
    {kind:'committed',admitted_at:timestamp,
     payloads:journal_committed_payload[1..256],
     committed_binding:continuity_binding_reference} |
    {kind:'aborted',abort:continuity_abort_reference}
}
```
All these objects and arrays are closed/complete; exact duplicate payloads are rejected and identity uniqueness is checked semantically. Multiple finite payloads permit one actual atomic mutation to bind a confirmation, its truthful lifecycle event and possible completion together. Their order and exact type/identity/digest projections must match between intent and committed resolution. Every event carries this intent ID and prepared_at; all payloads share the intended one fenced transaction and organizational scope. material_event requires nonnull initial propagation only for revoke; all other event actions require null. Inventory/obligation/source/plan/operation joins are exact and retained in the prepared association.

The initial propagation_completion payload is restricted to repair=null and its exact initial proof. The reserved propagation_repair_completion payload requires the completion's nonnull repair and coverage proof to equal the corresponding committed refs. Its coverage reference binds the exact independently retained complete E124 coverage node/graph, not a claim that a hash alone proves cleanup. E124 must supply the full typed producer/verifier bytes, complete retention and finite prepared/committed binding schemas before this branch can be produced. Its full coverage/predecessor/range and original/later action associations must be recoverable from those bindings.

E124 must also add closed finite lifecycle-pair payload variants for repair admission and range/coverage progress before their first mutations, using the already defined propagation_repair_reference, propagation_repair_batch_reference and propagation_coverage_reference. E008 explicitly reserves these typed associations and forbids encoding them as arbitrary metadata or borrowing an unrelated initial payload. Actual variants retain every repair predecessor, its operation/plan, batch snapshot/range, prior performed or uncertain action history, new effects/confirmations, current head/control and final completion. E050's existing operation/lifecycle store and six-family stream own this history; there is no seventh family, second scheduler or discarded failed-attempt ledger. Prepared correlations contain no future actual commit times, and committed bindings retain the exact actual authority/cause/control/effect/audit/outbox relationships.

The cancellation record and observed operation snapshot are final race observations, so a recognition intent never pre-hashes their still-unknown disposition/state/time. A hold's immutable placed_at is its prepared construction observation, not unknown commit time; intended event/state bytes are frozen before acknowledgement and final fences must verify them or abort/reprepare. Final records/result associations belong in committed evidence. No prepared binding includes a still-unknown cancellation/hold/result observation.

The qualified E050 integrity profile MUST define closed complete producer and verifier schemas for both binding-byte stages, actual retention/storage identity, authentication and continuity interpretation before any consuming mutation. A prepared binding names the exact immutable input/expected-control/authority correlation, record hashes, operation/plan/obligation/inventory and intended action/confirmation/effect associations, with no future canonical timestamps. A committed binding names the exact successfully committed records/relations, actual final admission observation, actual historical action cause, operation/control/effect/exposure/material state and audit/outbox association needed to replay that mutation idempotently. It binds the prepared intent digest and exact committed payload set. Source inventory scope includes independently retained complete inventory bytes/root and interpretation, not an unexplained snapshot hash.

The bounded binding evidence can reference separately retained, exactly typed complete records under that same qualified profile; its declared byte length is actual evidence bytes. That reference cannot hide an unspecified data format, missing inventory, unknown canonical relation or incomplete restore input. Full producing/verifying schemas and retained bytes must exist at E050/first writer. No generic metadata-only endpoint is provided. Encoding large prepared bytes happens outside canonical locks; committed evidence is constructed from the exact transaction/outbox result after commit and independently acknowledged. No prepared preimage requires unknown final time, and no unacknowledged post-commit association is treated as recovered continuity.

Recognition and retirement committed-resolution profiles similarly bind their exact recognition/retirement record, actual canonical cause/observation and admission barrier. Their small wire references alone are not continuity proof or permission to discard the independently retained exact record. Current known material initialization has command/operation origin; no registration-ID origin or speculative bootstrap action is added.

```text
continuity_record =
  {kind:'lifecycle_intent', value:journal_intent} |
  {kind:'lifecycle_resolution', value:journal_resolution} |
  {kind:'recognition_intent', value:recognition_intent} |
  {kind:'recognition_resolution', value:recognition_resolution} |
  {kind:'scope_retirement_intent', value:scope_retirement_intent} |
  {kind:'scope_retirement_resolution', value:scope_retirement_resolution}
continuity_entry = {
  ordinal:common.positive_revision_value, record:continuity_record
}
```
```text
journal_checkpoint = {
  organization_id:id, stream_id:id, checkpoint_id:id,
  first_ordinal:positive_revision_value,
  last_ordinal:positive_revision_value,
  previous_checkpoint_sha256:digest | null,
  records_sha256:digest, integrity_profile:integrity_profile_reference,
  proof_reference:{sha256:digest, byte_length:integer 1..65536}
}
```

continuity_batch is a closed {entries:continuity_entry[1..256],checkpoint:journal_checkpoint}; complete encoded bytes<=1MiB, common depth/collections also apply. Each entry ordinal is common.positive_revision_value, checkpoint first/last match the exact contiguous batch, records_sha256 hashes the profile-defined complete ordered record bytes, prior checkpoint linkage is exact/null only at an authenticated beginning. Rollover creates a new complete batch; it never splits/truncates an entry. Ordinary event/control IDs and maximum allocated sequence are not committed completeness proofs.

Before canonical mutation, independently durably acknowledge the exact prepared intent outside DB locks; then repeat authority/claim/control/inventory fences inside the short transaction and atomically bind effects/recognition/audit/outbox; afterward independently retain authenticated committed resolution. Losing/expired intents need qualified abort proof, not guesses. Crash/recovery must authenticate every six-family entry and complete cutoff, reapply all later restrictions, recognition and retirement barriers, and restore complete action/obligation associations before serving, worker admission or outbound delivery. Unknown tail/cutoff, unresolved intent, absent binding evidence or inconsistent resolution keeps affected serving closed. Other future authorization writers must receive their own typed qualified interpretation before a deployment claims complete recovery.

E050 owns independent storage, exact integrity/binding/abort proof grammar, ordering/cutoff, fences, actual keys and complete restore qualification before its first consumer. E112 later owns retention/export/purge policy interpretation and stronger end-to-end lifecycle qualification; it cannot retroactively justify earlier missing continuity. No real provider/key/policy is selected here.

Bootstrap precedes tenant rows under actual operator authority: prepare stable candidate organization/principal IDs, one-use expiring enrollment/actor binding, exact prospective registered command, immutable organization policy bytes/interpreted retention, integrity profile and independent stream namespace. These are operator-prepared candidates, not an API call requiring membership in a tenant that does not yet exist. The first short transaction binds policy/initial owner/recognition/claim/effect/audit/outbox/intents under uniqueness and admission fences. Unknown real namespace/policy/operator/capability inputs deny actual provisioning. E054 and E058 therefore require E050; component persistence E042 remains earlier without a reverse dependency.

## 12. Immutable attributed feedback

feedback.record POST /v1/feedback has closed {metadata,body}, body={subject:feedback_subject,event_time:event_time,content:feedback_content,evidence:observation.observation_reference[0..16]}. It returns own atomic200 terminal receipt with additive result {command:'feedback.record',feedback:feedback_record}. GET /v1/feedback/{feedback_id} returns additive result {feedback:feedback_record}, no receipt/listing. Current context and complete subject/evidence disclosure apply.
```text
feedback_subject =
  {kind:'asset_version',asset:accepted_asset_reference} |
  {kind:'context_build',build:build_reference}
event_time = {kind:'unknown'} |
  {kind:'instant',value:timestamp} | {kind:'date',value:calendar_date}
feedback_reference = {organization_id,domain_id,feedback_id,feedback_sha256}
reported_actor = {kind:'unknown'} |
  {kind:'principal',principal:principal_reference} |
  {kind:'unverified_label',label:string1..256}
text_fact = {kind:'unknown',reason:'not_observed'|'not_recorded'|'not_applicable'} |
  {kind:'known',text:string1..2048}
text_set = {kind:'unknown',reason:finite unknown reason} |
  {kind:'known',items:string1..2048[0..16]}
feedback_relation = {kind:'unknown'} | {kind:'none'} |
  {kind:'exact',reference:feedback_reference}
used_context = {kind:'unknown'} | {kind:'none'} |
  {kind:'exact',asset_versions:accepted_asset_reference[1..100],
   build:build_reference|null,usage:usage_reference|null}
usage_reference = {organization_id,domain_id,usage_id}
```

finite unknown reason is not_observed|not_recorded|not_applicable. Known collections are closed {kind:'known',items:T[min..max]}; unknown collections are closed {kind:'unknown',reason:finite unknown reason}. feedback_content is the closed union:

| kind | Other required fields |
| --- | --- |
| note | statement:string1..8192,qualifications:text_set |
| decision | question:text_fact,alternatives:unknown or known alternative[1..16],responsible_actor:reported_actor,assumptions:text_set |
| experiment | hypothesis:text_fact,intervention:text_fact,population:text_fact,metrics:unknown or known metric[1..16],assumptions:text_set,confounders:text_set |
| action | decision:feedback_relation,description:text_fact,performed_by:reported_actor,used:used_context |
| outcome | decision:feedback_relation,experiment:feedback_relation,action:feedback_relation,result:text_fact,assessment:positive|negative|mixed|no_change|unknown,metrics:unknown or known metric[0..16],alternative_explanations:text_set,used:used_context |

alternative={option_id:common.slug,description:string1..2048,disposition:chosen|rejected|deferred|unknown,reason:text_fact}; metric={name:common.slug,definition:text_fact,value:text_fact,unit:text_fact,baseline:text_fact}. Option IDs and metric names are unique; at most one chosen alternative (schema maxContains1/minContains0), no manufactured winner. All set-valued evidence, exact used assets, alternatives, metrics and text sets reject exact duplicate members; same identity/different bytes is rejected semantically. Relations resolve already committed expected decision/experiment/action kind under current context, without self/cycles.

```text
feedback_core = {
  kind:'arboresce.feedback.v1', feedback_id, context:command_context,
  subject:feedback_subject, event_time:event_time, content:feedback_content,
  evidence:observation.observation_reference[0..16],
  source:feedback_source,
  author:principal_reference, executor:principal_reference,
  delegation_id:identifier|null, authority_audit_event_id,
  received_at:timestamp
}
feedback_record = {core:feedback_core,feedback_sha256}
feedback_source = {kind:'author_report'} |
  {kind:'qualified_observation',profile:profile_reference,
   observation:observation_reference,input_sha256,output_sha256}
```

feedback_sha256 hashes complete JCS(core). Public record always produces source=author_report and accepts no source/author/executor/delegation/audit/server time/ID/hash fields. Direct execution has author=executor and null delegation; delegated author requires the actual explicit current grant for that action/context. reported_actor is a claim and grants no identity or authority. Receipt time is actual server observation; event date/instant is the author's assertion with its stated precision, can remain future/contradictory unless actual policy forbids, and never overrides authoritative receipt time.

qualified_observation is a server-only variant whose concrete producer must independently qualify procedure, independence, units, exact observation/input/output and profile before capability support; a source label/hash/service alone establishes none of these. No client endpoint creates it. Until that qualification only reported feedback is supported. used_context links exact retained E007 usage admission and build/selection/compiled identities; admitted delivery does not prove consumption. Exact used subsets are reported subsets, never a claim of complete signed-build coverage. No feedback correction/supersession, financial editing or implicit human approval command is added.

## 13. Consuming qualification map and dependencies

The E008 source gate is limited to selected finite contracts and independently checked parsed values. Runtime capability remains disabled until its actual owner qualifies complete bytes, admission/race rules and resources. Deferred evidence references are exact typed identities defined here; their producer/verifier and promised consumer read transport cannot remain unspecified when that capability first qualifies.

| Owner slice | Mandatory consuming obligation |
| --- | --- |
| E010 intelligence transport | Complete closed stage input/output/result/set schemas, exact input/profile/part/output bindings, cardinality, source classification and exposure input/receipt encodings; any promised stage/exposure exact evidence read; independent semantic/malformed cases |
| E034 subject model | Exact feedback/lifecycle identities and controls with immutable content meaning; no public runtime claim from model-only code |
| E046 operation/result persistence | Immutable complete plans, result IDs, stage/attempt/control snapshots and full effect/exposure/material-action associations; no second operation store |
| E050 lifecycle/journal persistence | Complete independent inventory snapshot, binding and abort proof bytes; six-family authenticated intent/resolution/checkpoint/cutoff; exact action/confirmation/obligation/effect relation recovery and retained repair/batch/coverage associations when E124 adds their concrete variants; current policy interpreter and service/integrity authority for first consuming mutation; missing tail closes serving |
| E054 first owner | Operator-prepared candidate namespace/policy/stream and real registered bootstrap command/attribution, first-owner/recognition atomic bind; E050 required before actual first admission |
| E058 command admission | Actual complete fingerprint, retained replay, minimized recognition/retirement barrier, current authorization and bounded canonical claims; E050 required |
| E063 sealing/shared operations | Earliest real generic operation GET/cancel and upload specialization, exact direct finalize replay, plan/result persistence and material initialization/continuity; uses E046 store, E068 server and E071 Temporal dispatcher |
| E072/E073/E074 | Orchestration/capture operation extension with deterministic explicitly test-only Activities and synthetic histories; no actual provider processing support |
| First gateway/storage action adapter | Actual complete closed exposure input/usage/receipt/evidence profiles, authenticity, conditional material versions, irreversible handoff and affirmative not_performed; no provider default or fabricated exposure |
| E086 Activities integration | Qualified selected real processing adapters/gateway/profiles wired end-to-end; required before runnable real capture.analyze capability |
| E096 financial exports | Existing financial JSON/CSV bytes plus exact sealed64MiB artifact, fixed expiry, current-authorized complete download, finite profile/slots, safe client full-byte verification and loss/replay races |
| E109 feedback | Exact author/delegation/evidence/source semantics, reported outcomes remain assertions, current disclosure and immutable retry; independent measurement producer must separately qualify before that source branch is enabled |
| E111 revocation | Immediate restrictive read/writer fence and exact immutable inventory scope, one bounded initial operation; complete initial manifest/proof, large/unknown cleanup and transient enumeration failure preserve durable repairable obligation; original failure/cancellation stays observable, no fake successful cleanup; E124 must qualify the repair capability |
| E112-retention-contract / E112 | Actual parameterized holds, references, purge policy and exact physical-copy manifest/receipt interpretation, current races, late confirmation and passive-ingress finite producer/verifier; E050 continuity is an earlier prerequisite, not retroactive work |
| E113/E114 and later exit/restore | Complete customer archive format/streaming operation has a distinct future reviewed capability/result contract; this financial-export format does not claim complete customer exit or invent that operation now |
| E124 derived invalidation/deletion | Mandatory current-authorized finite propagation repair over the original immutable obligation, complete command/operation/plan/batch/coverage/continuity producer-verifier contracts and exact reads; prior performed reuse/unknown-handoff fencing, authentic contiguous and empty coverage, all failed-attempt history, atomic final completion; actual RUST and SERVICE/Qdrant multi-batch, failure/cancellation, races and interrupted-repair qualification |
| E134-restore | Actual old-backup restoration of initial and later repair operations, all ranges/actions/confirmations and authentic coverage/completion associations before D0 or independent-user qualification; missing graph or unknown cutoff keeps serving closed |

Feedback corrections remain new evidence leading to explicit reviewed lesson/candidate supersession under the existing review owners; immutable feedback is never silently edited. The later exact supersession workflow/CLI obligations remain mandatory and are not removed by this bounded feedback.record command.

## 14. Required semantic and runtime qualification

The following are required independent oracles for the consuming implementations.
They are not executed by the parsed-value fixture manifest and must not be
reported as passing from schema positives.

Consuming semantic/runtime oracles cover canonical replay after expiry/GC and membership revoke-regrant/retirement races; exact hashes/format bytes and escaped near-limit serializers; complete stage/result joins and analysis attribution after receipt expiry; no implicit submit processing; cancellation versus bind/handoff, no hidden restart; actual service/human attribution; before/after independent intent/ack/canonical/outbox/resolution crashes, abort proof and old-backup restore; over-cap/unknown cleanup with canonical revoke preserved, complete empty inventory, cross-domain disclosure, late materialization and replacement versions; holds/references/leases versus irreversible deletion; finite uncertain exposure/liability/probes/passive confirmation; late successful obligation with failed/cancelled operation unchanged; real export throughput/full hashes/current authority/fixed expiry/midstream failures; and authentic feedback source/independence. Do not claim those run from literal schema positives.

E111/E124/E134-restore's mandatory repair oracles include0/254/255/256/257+ inventory and escaped near-byte limits; over-cap or transient failed initial enumeration with null manifest followed by multiple actually authorized bounded ranges; empty snapshot repair; failed/cancelled partial work and worker loss at every boundary; reuse of prior performed evidence without a new effect, affirmative not_performed replacement and unknown-handoff duplicate blocking; new holds/references/versions/revoked authority; missing/reordered/overlapping/gapped/duplicate/wrong-root or max-ordinal-only coverage; competing admissions/final binding/older passive receipts; final-head/completion atomicity; original and older repair terminal results unchanged; lost canonical backup restored from complete independently retained repair/action/coverage graph and receipt-GC preservation. Actual SERVICE and RUST/Qdrant multi-batch tests and old-backup restore must run under those consuming owners; E008 reference fixtures alone do not qualify them.


Every actually supported capability must define and qualify its complete concrete
producer/verifier bytes, exact current-authorized read, fixed finite profile and
retention before first use. Missing actual policy, provider, service identity,
key, inventory interpretation or restore proof denies that capability; no
placeholder hash or future slice can supply an earlier missing guarantee.
