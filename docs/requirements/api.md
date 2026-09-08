# Public API requirements

Status: accepted normative requirements. The product API is not implemented.
The contract steps in the [implementation plan](../execution/open-cli-mvp-v1-rcld.md)
must freeze exact OpenAPI, JSON Schema and transport objects before dependent
implementation. A named operation here is a required interface, not an available
command or endpoint unless separately documented as implemented.

This document owns command admission, HTTP behavior, idempotency, listing and
the distinction between exact resolution and selection. [Data model](data-model.md)
owns domain values and states; [storage and search](storage-and-search.md) owns
persistence, sealing and retrieval mechanics.

## Shared operation contract

Provide a bounded, versioned v1 HTTP surface with stable public operation
identifiers and versioned request/response contracts. Long-running work exposes
the public operation resource; workflow or run identifiers are not the client
interface for controlling it.

The CLI and other clients use the same public operations, authorization and
canonical semantics. Clients must not bypass the API through SQL, internal
service messages or server persistence types. The [product scope](product-and-scope.md)
owns capability parity, and [architecture](architecture.md) owns dependency
boundaries. Version and capability discovery must identify only implemented,
supported behavior without disclosing confidential deployment configuration.

The [common wire contract](common-contract.md) now freezes shared encodings,
command metadata, problem/state consistency and readiness shapes. Each operation
must add its own closed body and semantic admission rules. Passing the common
metadata schema alone cannot validate or authorize a complete domain command.

The [observation contract](observation-contract.md) owns grounding and source-kind
values carried by the relevant operations. It adds no mutation route by itself;
each operation still supplies its complete admission and response contract.

The [financial contract](financial-records.md#complete-commands-and-reads) owns
the complete expense revise/confirm/void and report create/export variants,
their current/exact reads and report availability-control precondition.
Its financial review event is distinct from candidate acceptance. The common
admission and reauthorization rules still apply to every financial effect.

Every operation must define:

- Required and optional fields, complete discriminated variants, field bounds
  and cross-field validation, including independent revision types.
- Required organization, domain, actor, purpose and authorized resource scope.
- Success status, domain state, operation/result identity and typed errors.
- Version preconditions, idempotency, expiration and retry behavior.
- Ordering, filters, pagination, request/response limits and admission quotas
  where applicable.
- Positive and negative compatibility cases and its entry in the public
  capability-to-operation inventory.

Reject unsupported command fields and unknown variants instead of silently
discarding intent. Reject malformed, duplicate-key, oversized or overly nested
input before expensive work. Response readers tolerate documented additive
fields and unknown RFC 9457 problem extensions while still requiring the
specified fields and consistent state. A success result cannot contain a
failure code. Shape validation alone cannot prove authorization, references or
arithmetic; the server must apply typed semantic validation.

Use body-based `expected_version` or `expected_generation`, with the exact
content/control revision type specified by the operation. Stale expectations
return a conflict. MVP does not simultaneously require `If-Match`; introducing
conditional headers requires a later explicit precedence and status contract.

## HTTP outcomes and health

| Status | Meaning |
| --- | --- |
| `202 Accepted` | Work was durably admitted; return an operation resource, not a success claim. `Location` identifies that operation. Supply `Retry-After` where polling/backoff guidance applies. |
| `400` | Malformed input or unsupported command shape. |
| `401` | Authentication is absent or invalid; include the applicable challenge. |
| `403` | The action is forbidden and policy permits revealing the resource. |
| `404` | Unknown route or inaccessible resource identity; do not reveal another tenant's ownership. |
| `409` | Conflicting revision, generation, competing transition or idempotency payload. |
| `413` | Request exceeds its size limit. |
| `415` | Unsupported content or media type. |
| `422` | Syntax is valid but the domain command is invalid. |
| `429` | Shared admission or quota limit prevents acceptance; provide bounded retry guidance. |
| `500` | Unexpected server failure; use the common contract's safe `internal_error` problem without exposing internal details or assuming the effect was absent. |
| `503` | A dependency required for the requested work is unavailable. |

The problem contract must keep its HTTP status, machine code and domain state
consistent. Validation and referential failures must not expose foreign IDs,
SQL, addresses, credentials or internal stack traces. All request, response
and waiting bounds come from [resource profiles](resource-profiles.md).

`/health/live` checks process liveness only. `/health/ready` returns `200` only
when that process role can perform its required work, otherwise `503`.
Readiness must reflect role dependencies: an optional search outage can degrade
search while exact-build serving remains ready. Health output is bounded and
contains no private operational details.

## Idempotent command admission

Scope a claim by organization, principal, command kind and idempotency-key
digest. Bind it to a fingerprint of method, canonical resource, contract
version, purpose and complete command body. Exact canonical encoding and
key constraints are frozen in the machine contract. Never reuse a key for a
different intent or let a new attempt number become a new business command.

The admission sequence is:

1. Validate syntax, capability, sizes and authentication; compute the command
   fingerprint without performing the requested effect.
2. Begin a short PostgreSQL transaction and establish transaction-local tenant
   context. Lock the relevant membership, policy and aggregate rows in the
   agreed deterministic order.
3. Check current action and evidence permissions. Claim the scoped key.
4. For an existing claim, reject a different fingerprint. Return the recorded
   operation or result for an identical command only after current authorization.
5. For a new claim, compare the expected revision and apply the allowed
   transition or record the accepted operation. Atomically reserve applicable
   shared capacity; a rejected reservation is not accepted work.
6. Write the domain effect, command result, audit record and outbox event in
   the same transaction, then commit before returning a result or operation ID.

Permission writers must honor the same locking discipline. Cached roles or
multi-table RLS predicates alone do not serialize concurrent revocation. The
[storage contract](storage-and-search.md) specifies locking, leases and atomic
persistence. No provider request, parsing or byte stream runs while these
database locks are held.

The [lifecycle retention contract](lifecycle-contract.md#11-retention-recognition-and-independent-continuity)
owns selected terminal-relative replay and diagnostic durations, their exact
bounds, minimized recognition and independently durable scope retirement. Replay
retention is at least seven days; diagnostic retention is at least thirty days
and strictly longer than replay. These minima do not select actual deployment
policy or promise legal retention. Pending operations retain null deadlines;
terminalization fixes both once. A lost-response retry keeps its original key.
Expired POST replay returns idempotency_expired/409 and cannot repeat an effect.
GET uses fresh current authorization and can return genuinely retained complete
details with the original receipt after expiry; removed required details return
operation_details_unavailable/409. Unknown required infrastructure or continuity
uses 503, and inaccessible identity remains 404.

Before a new claim can commit, independently acknowledge its exact prepared
recognition intent outside canonical locks. The short transaction atomically
binds recognition, claim, operation or effect, reservations, audit and outbox;
the independently retained committed resolution binds actual admission and
relations afterward. The lifecycle owner defines finite preparation, abort proof,
retired-scope races and closed-serving recovery. No old-backup absence or expired
receipt makes an admitted identity new. Business duplicate detection remains a
separate review suggestion.

## Acquisition and review operations

The [acquisition contract](acquisition-contract.md) owns precise upload and
capture routes, complete command variants, responses and semantic admission
rules. Its parsed-value fixtures do not replace current authorization, expiry,
cross-reference or sealed-storage verification.

Upload allocation grants only a scoped staging destination, permitted headers,
media and byte bounds. Finalization names the upload and expected byte length
and digest. The service returns acceptance only for the binding required by
[sealed storage](storage-and-search.md#sealed-objects-and-finalization).
Capture submission fixes an ordered required/optional attachment set and remains
atomic 200. It starts no processing operation, including on replay or later
attachment readiness. The separately authorized
[capture.analyze command](lifecycle-contract.md#5-acquisition-and-analysis)
binds the exact submitted revision/control and immutable submission attribution.
Upload completion alone does not imply capture readiness or successful processing.

Review commands are distinct variants. `revise` requires a complete replacement
of candidate content, including statement, scope, assumptions, all evidence
roles and invalidation conditions; it creates a new pending revision. The
[candidate review contract](candidate-review-contract.md) owns their exact v1
routes, closed command bodies and response variants. `accept`
pins the exact reviewed revision and digest and rejects replacement fields.
Reject, contest and subsequent revision preserve attributable history.
Evidence references require current permission to read and use those exact
versions. A service actor cannot obtain human-review authority from a flag.

The candidate lock, policy check, review, unique accepted asset mapping and
audit/outbox append are one short transaction. Replaying the same accepted
command returns its original mapping after reauthorization; a competing
conflicting command fails. There is at most one accepted mapping per candidate
revision, including under simultaneous review.

## Listing and filters

Use a bounded, typed filter grammar. Distinct filters combine with AND;
explicitly repeatable values combine with OR. Reject unknown filters. Do not
accept SQL, Cypher, JSONPath or model-produced expressions as filter syntax.

Large histories use keyset pagination on immutable ordering fields, such as
descending creation time with an opaque ID tie-breaker. The first page fixes
an upper watermark. Authenticate cursors and bind them to organization,
principal, domain, purpose, filter fingerprint, ordering, upper watermark and
expiry. A changed filter or purpose invalidates the cursor. A cursor carries
no secrets and never substitutes for current authorization.

Apply current eligibility and permissions on every page, preferably in the
bounded SQL selection. Do not use raw OFFSET for large histories or issue one
policy query per result. Do not expose inaccessible rows through totals,
counts, exclusions or cursor diagnostics.

Current-state queues are moving eligible sets: state changes may remove rows
between pages, even with a fixed upper watermark. Document this behavior.
A financial report uses the separate immutable selection operation specified
by [financial records](financial-records.md), not a traversal of that queue.
The [financial compilation fences](financial-records.md#compilation-and-permission-fences)
cover inserted or newly visible records and current authority before the
complete manifest commits. Financial export keeps the exact frozen report;
current report access and delivery authority remain separate from its identity.

## Exact snapshot resolution and search selection

The [context contract](context-contract.md) freezes complete build/evaluation/
approval commands and operation reads, exact build and channel resolution,
current authority, bounded content chunks and usage receipts. Its immutable
build content is separate from evaluation, human approval, signatures and
current availability. Channel changes compare generation and never restore
eligibility merely by naming an older signed build.

Exact resolution serves the complete selected content of one immutable Context
Build after current authorization, source eligibility and byte/token bounds.
Any unavailable or revoked required component makes the build unavailable.
Do not silently omit it, truncate the build or substitute a newer version.
Snapshot mode rejects query-driven selection.

Search is a separate operation governed by [storage and search](storage-and-search.md#bounded-search).
Its receipt names the actual selected versions, source build if relevant,
retrieval profile, generation, applied limits, truncation and incompleteness.
It cannot claim that a full-build approval or signature covers different
selected bytes. Conversion of ad hoc selections into an approved release
requires a separate accepted and tested contract.

Resolution and selection receipts retain exact served identity, purpose and
policy context. Neither receipt proves that a downstream decision was correct.
Current access restrictions apply even to content with a valid signature.

## Operations, lifecycle and exports

The [lifecycle contract](lifecycle-contract.md#3-shared-operation-plan-clocks-and-evidence)
owns complete plans, operation controls, stage/attempt clocks, actual effects,
exposures and material-action ledgers. Operation identity, workflow/run identity,
stage result and attempt are distinct; [processing](processing.md) owns durable
orchestration. Every exact record or ledger read is complete or fails, with no
hidden entries, fabricated empty set or pagination of one operation ledger.

The [operation.cancel contract](lifecycle-contract.md#4-cancellation) compares the
expected control before terminal state and atomically returns requested,
already_requested or already_terminal with its own immutable record/receipt.
Replay retains its original observed target snapshot. Success and cancellation
share the final fence; stale cancellation conflicts even if completion just won.
Client waiting detaches by default. Cancellation preserves irreversible work,
source restriction, purge responsibility and uncertainty; bounded authenticated
[late confirmation](lifecycle-contract.md#9-late-confirmation-and-cancellation-races)
never changes a terminal result or starts hidden replacement work.

### Exact lifecycle transport inventory

Every mutation below uses closed {metadata,body}; the lifecycle owner defines
every field, precondition and result. All exact GETs require organization_id,
domain_id and purpose query selectors, reject extra/duplicate selectors, decode
once and match route/reference scope. Read-only POSTs use their exact closed
body and carry no command receipt. These are specified capabilities; actual
support begins only after the owning implementation qualifies them.

| Command | POST route | Initial success |
| --- | --- | --- |
| upload.abort | /v1/uploads/{upload_id}/abort | Atomic 200, aborted upload and own terminal receipt. |
| capture.analyze | /v1/captures/{capture_id}/analyses | 202, operation and originating pending receipt. |
| operation.cancel | /v1/operations/{operation_id}/cancel | Atomic 200, cancellation and observed target, own receipt. |
| lifecycle.revoke | /v1/lifecycle/revocations | Atomic 200, immediate restriction, event, initial propagation and obligation. |
| lifecycle.hold.place | /v1/lifecycle/holds | Atomic 200, control/event/hold and own receipt. |
| lifecycle.hold.release | /v1/lifecycle/holds/{hold_id}/release | Atomic 200, exact release/control and own receipt. |
| lifecycle.purge | /v1/lifecycle/purges | 202 with durable purge-request effect. |
| feedback.record | /v1/feedback | Atomic 200, immutable attributed record and own receipt. |

The [origin/transport matrix](lifecycle-contract.md#2-capability-origin-and-transport-matrix)
preserves all twenty-one earlier public requests. Command-origin operation GET
and eligible ordinary POST replay use common 200 result {operation,
idempotency_receipt} with no top-level receipt. Internal lifecycle.propagate GET
uses {operation,origin_event}, with no command receipt at either level and no
public propagation command. Failed/cancelled resource reads retain their actual
work outcome. The [upload.finalize exceptions](lifecycle-contract.md#5-acquisition-and-analysis)
retain 202 nonterminal replay with sealing upload, direct 200 successful result
with original finalization reference and own terminal receipt, and the fresh-key
sealed-binding lookup without a new operation. Failed/cancelled replay and every
finalizer operation GET use the command-origin 200 operation wrapper.

| Exact read | Required result or content |
| --- | --- |
| GET /v1/operations/{operation_id} | Origin-selected operation wrapper above. |
| GET /v1/operation-plans/{plan_id} | {plan}, complete immutable plan. |
| GET /v1/uploads/{upload_id} | {upload}, required lifecycle control specialization. |
| GET /v1/captures/{capture_id} | {capture}, current resource. |
| GET /v1/captures/{capture_id}/revisions/{version} | {capture}, exact immutable revision. |
| GET /v1/capture-analyses/{analysis_id} | {analysis}, complete success record. |
| POST /v1/lifecycle/resolve | {control}; body {context,subject}. |
| GET /v1/lifecycle/events/{event_id} | {event,obligation}; revoke has obligation, other events null. |
| GET /v1/lifecycle/holds/{hold_id} | {hold}. |
| GET /v1/lifecycle-propagation-obligations/{obligation_id} | {obligation}, including original operation and retained repair head. |
| GET /v1/lifecycle-propagation-manifests/{manifest_id} | {manifest}. |
| GET /v1/lifecycle-propagation-proofs/{proof_id} | {proof}. |
| GET /v1/lifecycle-propagations/{propagation_id} | {propagation}. |
| GET /v1/lifecycle-actions/{action_id} | {action}, complete action record and current control. |
| GET /v1/lifecycle-confirmations/{confirmation_id} | {confirmation}. |
| GET /v1/export-artifacts/{export_id} | {artifact}, exact sealed identity and observed availability. |
| POST /v1/export-artifacts/{export_id}/content | Complete bounded binary content; body {context,artifact,expected_control_revision}. |
| GET /v1/export-deliveries/{delivery_id} | {delivery}, admission receipt without a reusable grant. |
| GET /v1/feedback/{feedback_id} | {feedback}. |

Except the origin-selected operation wrapper, these JSON reads use additive
common 200 results without a command receipt; nested canonical records remain
closed. The [repair contract obligations](lifecycle-contract.md#required-bounded-repair-and-complete-coverage-at-e124)
reserve exact repair/batch/coverage reads for E124, which must define complete
records and registered mutation/result/operation transport before activation.
E008 does not enable a ninth operation or an unspecified repair command.

The public surface must cover identity and membership; upload/capture and
authorized download; processing and cancellation; candidates/reviews/assets;
expenses, reports and exports; builds, evaluations, attestations and channels;
exact resolution and search; exact-version feedback and supersession; and
revocation, holds, purge and complete customer export. Operator-only operations
are explicitly identified in the capability inventory.

Lifecycle commands compare the appropriate control revision and preserve
immutable history. Export reports missing or purged material honestly and
never leaks inaccessible entries or counts. General export and revocation
policy is owned by [security and privacy](security-and-privacy.md); financial
export field selection and arithmetic are owned by [financial records](financial-records.md).

## Required verification

Contract and process suites must cover malformed and contradictory variants,
unknown response extensions, hidden foreign resources, same-key replay after
revocation, different-body conflict, expired identity, competing reviews,
cursor substitution, unavailable readiness, cancellation races and complete
snapshot denial. Exercise actual API behavior once implemented under
[testing and coverage](testing-and-coverage.md). The [acceptance criteria](acceptance.md)
own experiment thresholds and capability evidence; this document claims no
passing runtime results.
