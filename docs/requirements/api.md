# Public API requirements

Status: accepted normative requirements. The product API is not implemented.
The contract steps in the [implementation plan](../execution/open-cli-mvp-v1-rcld.md)
must freeze exact OpenAPI, JSON Schema and transport objects before dependent
implementation. A named operation here is a required interface, not an available
command or endpoint unless separately documented as implemented.

This document owns command admission, HTTP behavior, idempotency, route registration
and the distinction between exact resolution and selection. The
[selection contract](selection-contract.md) owns the finite listing, exact
companion-read and search wire profiles. [Data model](data-model.md)
owns domain values and states; [storage and search](storage-and-search.md) owns
persistence, sealing and retrieval mechanics.

## Shared operation contract

The focused first surface follows [project workflow](project-workflow.md) and
[Nostr identity](identity-contract.md). Freeze complete versioned contracts for
identity/bootstrap, workspace membership/grants, Party/project/context reads and
mutations, capture/analysis/status/cancel, typed item edit/verdict/application,
follow-up supersession, exact financial review challenge/issuance/confirmation,
project readback and independently interpretable exports before their consumers.
Each mutation names one workspace and exact domain/purpose, target/base versions
and idempotency intent. Exceptions are explicitly controlled identity/workspace
bootstrap. Same-actor access to multiple workspaces never authorizes cross-tenant
links. Define disclosure-safe typed ambiguous/dependency/stale-target outcomes.

Project schemas extend candidate/review/asset mappings through an explicit version;
financial association/review extends a separate financial composition. Unchanged
closed v1 schemas/commands are not broadened by prose. Preserve exact legacy hashes
and fixtures. Register each new command, result, operation and evidence read with
actual admission/continuity support. No generic executable mutation or financial
approval inferred from project verdicts is allowed. Capability discovery advertises
only implemented variants; disabled search/build/physical-purge/repair return an
explicit unavailable/unsupported outcome without claiming completed work.

Provide complete current/exact project, Party, item and association reads with
bounded pagination and source/context/review lineage. Agent JSON contracts require
stable typed outcomes, canonical IDs/revisions and truthful processing/partial
results. Human review must expose consequential complete content readably. Parsing,
stdout/stderr/process outcomes and terminal editing are independently owned by
the CLI; canonical transition and financial rules remain engine-owned.

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

### Focused project, identity, grant and review transport inventory

[Identity](identity-contract.md) owns authentication and review authority;
[project workflow](project-workflow.md) owns workspace/Party/project meaning;
[financial records](financial-records.md#complete-commands-and-reads) owns the
expense confirmation route replayed here. The focused first surface registers
exactly the operations below; the machine contract
[`operation_inventory`](../../contracts/public/v1/project-workflow.schema.json)
pins one registration per operation and rejects an unknown operation, a changed
method or route and any error code outside the admitted set. A named operation
without a registration is not implemented. Actual support begins only after the
owning implementation qualifies it.

| Registered operation | Method and route | Required scope and authority | Success contract |
| --- | --- | --- | --- |
| identity.bootstrap | POST `/v1/identity/bootstrap` | Bootstrap exception; operator-held expiring one-use bootstrap grant | identity_bootstrap_response |
| workspace.get | POST `/v1/workspaces/resolve` | Exactly one workspace; current membership | workspace_get_response |
| workspace.membership.get | POST `/v1/workspace-memberships/resolve` | Exactly one workspace; current membership | workspace_membership_get_response |
| project.create | POST `/v1/projects` | Exactly one workspace; scoped agent grant or authorized human review | project_response |
| project.get | POST `/v1/projects/resolve` | Exactly one workspace; current membership | project_get_response |
| project.revise | POST `/v1/projects/{project_id}/revisions` | Exactly one workspace; scoped agent grant or authorized human review | project_response |
| party.create | POST `/v1/parties` | Exactly one workspace; scoped agent grant or authorized human review | party_response |
| party.get | POST `/v1/parties/resolve` | Exactly one workspace; current membership | party_get_response |
| party.revise | POST `/v1/parties/{party_id}/revisions` | Exactly one workspace; scoped agent grant or authorized human review | party_response |
| project_change.get | POST `/v1/project-changes/resolve` | Exactly one workspace; current membership | change_item_get_response |
| project_change.edit | POST `/v1/project-changes/{item_id}/edits` | Exactly one workspace; scoped agent grant or authorized human review | change_item_edit_response |
| project_change.review | POST `/v1/project-changes/{item_id}/reviews` | Exactly one workspace; scoped agent grant or authorized human review | change_item_review_response |
| project.resolve_reference | POST `/v1/project-references/resolve` | Exactly one workspace; current membership | reference_resolution_response |
| expense_association.create | POST `/v1/expense-associations` | Exactly one workspace; scoped agent grant or authorized human review | expense_association_response |
| expense_association.get | POST `/v1/expense-associations/resolve` | Exactly one workspace; current membership | expense_association_get_response |
| grant.create | POST `/v1/agent-grants` | Exactly one workspace; scoped agent grant or authorized human review | agent_grant_response |
| grant.get | POST `/v1/agent-grants/resolve` | Exactly one workspace; scoped agent grant or authorized human review | agent_grant_get_response |
| grant.revoke | POST `/v1/agent-grants/{grant_id}/revoke` | Exactly one workspace; scoped agent grant or authorized human review | agent_grant_response |
| financial_review.challenge | POST `/v1/financial-reviews/challenges` | Exactly one workspace; authenticated review requester or scoped agent grant | financial_review_challenge_response |
| financial_review.issue | POST `/v1/financial-reviews` | Exactly one workspace; qualified trusted human reviewer | financial_review_issue_response |
| financial_review.event.get | POST `/v1/financial-reviews/events/resolve` | Exactly one workspace; qualified trusted human reviewer | financial_review_event_get_response |
| expense.confirm | POST `/v1/expenses/{expense_id}/confirm` | Exactly one workspace; current financial owner consuming one issued review event | expense_confirm_response |
| key.transition | POST `/v1/principal-key-transitions` | Exactly one workspace; qualified current recovery authority | principal_key_transition_response |

Every command uses the closed `{metadata,body}` shape and contract-version v1
scope. `identity.bootstrap` is the only workspace exception: its closed
`identity_bootstrap_metadata` names the `organization_id` and `purpose` it
creates without requiring an existing workspace or domain selection; every other
command selects exactly one organization/domain/purpose. Read-only membership,
grant, project, Party, item, association, reference-resolution and review-event
reads carry no command receipt. Every mutation names exact target/base revisions
and idempotency intent. Same-actor access to multiple workspaces never authorizes
a cross-tenant link. Each registration admits the shared `invalid_request`,
`unauthenticated`, `forbidden`, `request_too_large`, `unsupported_media_type`,
`invalid_command`, `rate_limited`, `internal_error` and `unavailable` problems,
plus `not_found` for reads and `conflict` for mutations, matching the shared HTTP
outcome table below.

Each registered grant-consuming operation carries one explicit least-privilege
operation-to-action/purpose/resource mapping in the machine contract: the exact
`grant_action`, bounded purpose and same-workspace grant resource kinds its scoped
agent grant must include, resolving the differing operation and action spellings
(for example `expense_association.create` to `association.create`). Ordinary
project, Party, item and association writes accept either that explicit scoped
agent grant or the ordinary `authorized_human_review` authority under current
domain policy; workspace membership alone is never action authority.
`authorized_human_review` is an attributable human project reviewer, distinct from
the trusted financial human reviewer, the authenticated challenge requester and the
separately permitted executor, and it cannot issue financial assurance.
The financial-review challenge is requested by a distinct authenticated review
requester (or an explicit `financial_review.request` grant action), which is not the
trusted human reviewer that issues the assurance, and not the separately permitted
executor that consumes the issued event.

### Signed-request and reference-resolution admission

The [identity profile](identity-contract.md#signed-http-profile) owns the exact
NIP-01/NIP-98 selections. The machine contract freezes the NIP-98
`Authorization` transport separately from its normalized event fields: an
`authorization_scheme` of `Nostr` with a bounded admitted base64 event, the exact
received `origin` and `request_target`, the signed method, the required SHA-256
of every nonempty admitted body and the admitted body byte length. The server
decodes and verifies those admitted bytes; it never reconstructs the URL from
forwarding headers or verifies a parsed/re-serialized body. Finite
`max_authorization_header_bytes`, `max_decoded_event_bytes`, `max_tag_cardinality`,
`max_body_bytes` and `max_url_bytes` limits reject before any signing work, and
the profile admits only the selected exact standard revision with status
`admitted`. Admission additionally requires NIP-98 (never NIP-01) and binds the
body digest to the body length: a nonempty admitted body must carry its SHA-256,
while a bodyless request must carry no digest. Duplicate/contradictory tags,
unsupported encodings, changed method/URL/body, expired/future/replayed events
and unknown or revoked authority are rejected. Authentication implementation and
its fixed-vector qualification remain a later owning identity consumer gate,
not provided here; this registration does not admit an unqualified profile.

Reference resolution is a disclosure-safe successful read that returns a typed
`resolution_outcome` (`resolved`, `ambiguous`, `dependency_unresolved`,
`stale_target`, `not_found`). `resolved` carries exactly one exact same-workspace
record or item revision; every other outcome carries no resolved value and only a
bounded clarification. An inaccessible or foreign resource never becomes
`not_found` plus another tenant's identity, and an ambiguous or stale target is
never silently resolved. These outcomes do not replace the HTTP problem contract
for malformed, unauthenticated or unauthorized requests.

The project change-item and expense-association digests bind closed canonical
preimages (`project_change_preimage`, `expense_association_preimage`); the
trusted financial review preimage additionally binds executor scope and expiry.
Record id, version and recording-time identity stay outside these envelopes.

## HTTP outcomes and health

| Status | Meaning |
| --- | --- |
| `202 Accepted` | Work was durably admitted; return an operation resource, not a success claim. `Location` identifies that operation. Supply `Retry-After` where polling/backoff guidance applies. |
| `400` | Malformed input or unsupported request shape, including the selected GET request-target byte and query-pair limits. |
| `401` | Authentication is absent or invalid; include the applicable challenge. |
| `403` | The action is forbidden and policy permits revealing the resource. |
| `404` | Unknown route or inaccessible resource identity; do not reveal another tenant's ownership. |
| `409` | Conflicting revision, generation, competing transition or idempotency payload. |
| `413` | Request body exceeds its wire-byte limit; the selected bodyless GET target/pair limits use 400. |
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

The selection contract registers the following eighteen operations and exact
response associations. Each GET returns common HTTP 200 success without a command
receipt. These are required interfaces; support requires actual route, identity,
key/time, persistence and compatible consumer qualification.

| Operation | Method and route | Named response |
| --- | --- | --- |
| capture.list | GET /v1/captures | listing_response_capture_heads |
| capture.revisions.list | GET /v1/captures/{capture_id}/revisions | listing_response_capture_revisions |
| candidate.list | GET /v1/candidates | listing_response_candidate_heads |
| candidate.revisions.list | GET /v1/candidates/{candidate_id}/revisions | listing_response_candidate_revisions |
| asset.list | GET /v1/assets | listing_response_asset_heads |
| asset.versions.list | GET /v1/assets/{asset_id}/versions | listing_response_asset_versions |
| expense.list | GET /v1/expenses | listing_response_expense_heads |
| expense.revisions.list | GET /v1/expenses/{expense_id}/revisions | listing_response_expense_revisions |
| channel.list | GET /v1/domains/{domain_id}/channels | listing_response_channel_heads |
| channel.generations.list | GET /v1/domains/{domain_id}/channels/{name}/generations | listing_response_channel_generations |
| candidate.read | GET /v1/candidates/{candidate_id} | candidate_current_get_response |
| candidate.revision.read | GET /v1/candidates/{candidate_id}/revisions/{version} | candidate_revision_get_response |
| observation.read | GET /v1/observations/{observation_id} | observation_get_response |
| asset.read | GET /v1/assets/{asset_id} | asset_current_get_response |
| asset.version.read | GET /v1/assets/{asset_id}/versions/{version_id} | asset_version_get_response |
| asset.supersession.read | GET /v1/assets/{asset_id}/supersessions/{supersession_id} | asset_supersession_get_response |
| channel.generation.read | GET /v1/domains/{domain_id}/channels/{name}/generations/{generation} | channel_generation_get_response |
| search.select.v1 | POST /v1/context/search | search_response |

All seventeen GETs are bodyless and reject a nonempty body. Require exactly one
organization_id, domain_id and purpose; route scope must match. Only listing
families accept their declared page_size, cursor and family filters. Exact
companions accept no listing selector. Typed request observations are decoded
route/query values, never JSON GET bodies. Decode strict UTF-8/percent encoding
once; reject unknown keys, empty values and duplicate singular keys. Repeatable
keys accept only distinct values, rejecting decoded aliases of the same value.
The raw origin-relative
target is at most 8192 bytes and has at most 32 query pairs, checked before
allocation; exceeding either uses invalid_request/400. Canonical paths and
primitive encodings retain their owners.

Use a bounded, typed filter grammar. Distinct filters combine with AND;
explicitly repeatable values combine with OR. Reject unknown filters. Do not
accept SQL, Cypher, JSONPath or model-produced expressions as filter syntax.

Large histories use the selection contract's exact family keysets: immutable
aggregate creation time/ID, content revision, asset-local version order, channel
name or generation. The first page fixes its upper watermark from a disclosed
row. Authenticate cursors and bind the exact executor/delegation, context,
normalized filters, route, view, page size, ordering and returned keys. The fixed
900-second expiry never renews. A cursor carries no secrets and never substitutes
for current authorization.

Apply current eligibility and permissions on every page, preferably in the
bounded SQL selection. Do not use raw OFFSET for large histories or issue one
policy query per result. Do not expose inaccessible rows through totals,
counts, exclusions or cursor diagnostics.

Current-state queues are moving eligible sets: state changes may remove rows
between pages, even with a fixed upper watermark. New visibility below the
remaining boundary can appear; an already passed range can be missed. Only a
successful bounded authorized selection/lookahead establishes the next cursor
or final page. Whole-page admission fences current authority and row/control
observations; changed observations conflict without an automatic refill loop.
Malformed raw selectors use 400. Invalid supplied cursor syntax, key, MAC,
time or binding uses the uniform invalid_command/422 cursor_invalid problem.
Missing required key/time/policy infrastructure uses unavailable/503. Authorized
missing required retained predicate, summary or exact-resource details use
conflict/409 with resource_details_unavailable; never substitute null, another
revision or successful exhaustion. Unknown or undisclosable identity remains 404.
Actual impossible complete response construction uses 503. The selection owner
fixes safe title/detail and response compositions; no hidden field/count enters
an error. Exact selected versions never fall back to current heads.

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

Search is the separate synchronous POST above, with the selection owner's closed
complete request, explicit profiles/filters/field view/output limits and no query
parameters, command metadata, idempotency receipt or public cursor. It returns one
finite response. [Storage and search](storage-and-search.md#bounded-search) owns
the bounded retrieval and canonical final fences.
Its receipt names the actual selected versions, source build if relevant,
retrieval profile, generation, applied limits, truncation and incompleteness.
Every returned candidate retains complete consequential content; optional direct
observations are the complete exact union. The unsigned receipt binds actual
output and gateway exposure, records admission and does not prove delivery.
It cannot claim that a full-build approval or signature covers different
selected bytes. Conversion of ad hoc selections into an approved release
requires a separate accepted and tested contract.

Once search dispatch might have occurred, clients must not automatically replay
the HTTP request after timeout, connection loss, ambiguous write, 5xx or a lost
response. Only a proved zero-byte pre-dispatch failure can retry the unsent
request. Retry-After guides a deliberately new invocation, with new selection
identity and current admission, not resume or page fill. Internal work keeps its
original deadline and shared reservations; reconciliation cannot start new work.
search.select.v1 remains unavailable until both search execution and complete
receipt production qualify, including compatible consumer behavior.

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
every field, precondition and result. All lifecycle exact GETs require organization_id,
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

## MVP CLI transport completeness

The 23-entry `operation_inventory` above is the focused project/identity/grant/
review inventory, not the inventory of every MVP API. Acquisition, lifecycle and
financial owners retain their own registered contracts. In particular current
capture GET and operation GET are defined by the lifecycle contract and must not
be classified as unsupported merely because they are absent from that table.
Actual unavailable capability remains an honest runtime result; absence of a
qualified contract is unfinished implementation, not removal of the requirement.

Complete the following additive contracts before CLI contract consumers, without
changing the accepted 23 registrations or closed financial/acquisition v1 bodies:

- `principal.revoke`: POST `/v1/principals/{principal_id}/revoke`, authenticated
  self-principal only, selected authorized context, exact current control revision
  and explicit reason. A qualified closed body names the principal ID,
  `expected_control_revision` and reason; success returns the complete revoked
  principal identity and terminal command receipt. It fences that server's
  principal/key authority without creating a replacement principal or claiming
  remote deletion. Key transition remains a different recovery-authority operation.
- GET `/v1/captures/{capture_id}/project-changes`: an authorized, revision-bound
  complete typed change grouping with exact capture/project identity, item content,
  review/control observations and deterministic bounded continuation. Plan and
  filtered item-list presentations use this canonical grouping, not the lifecycle
  execution-plan record. Empty, pending, failed and permission-denied cases remain
  distinguishable; no unauthorized head or truncated set may appear complete.
- GET `/v1/projects/{project_id}/readback`: qualified current accepted project
  revisions, exact accepted mappings/source/context provenance, qualified current
  expense associations and separately identified pending items when requested.
  Preserve current disclosure and complete-or-error semantics.
- Project-scoped report creation and export use the separately versioned financial
  composition required by the financial owner: exact project/currency selection,
  date policy/period/include-undated inputs, canonical server totals, exact
  expense/association revisions and availability control. Define their complete
  requests/responses and register exact methods/routes before CLI consumption.
  Do not alias away a project selector into the older workspace report command.
- GET `/v1/project-reports/{report_id}` returns the complete separately versioned
  project financial snapshot and current report control under current disclosure.
  It supplies the exact immutable reference for export; resolving an ID is not
  permission to substitute a workspace report. The normalized read selector uses
  `project_report_get_request`, the result uses `project_report_get_response`, and
  the HTTP request has no JSON body or mutation idempotency key.
- POST `/v1/projects/{project_id}/exports`: exact project revision/control and
  requested JSON/CSV profile; durable admission and qualified sealed project
  artifact/delivery containing lossless project records, provenance and financial
  composition references under current disclosure. Define independent JSON and
  reversible CSV profiles; a CLI-created file or a financial-only artifact is
  not a sealed project export.
- Project capture association is a separately versioned binding between selected
  project and exact capture intent/revision. Define its complete transport and
  independently validated source/context identity before capture composition;
  do not add a project ID to closed acquisition v1 bodies or hide it in prose.

These are contract completions for the existing MVP surfaces, not runtime support
claims. Each needs real public engine schemas, exact compatible routes, positive/
negative and semantic vectors, bounded profile qualification and independent
contract review before its consumer. CLI presentation schemas stay CLI-owned;
canonical requests, read envelopes and sealed outputs stay engine-owned. Preserve
receipt distinctions, current authority, idempotency and resource budgets. Contract
additions do not require implementation of later storage/service/model consumers.

The focused project artifact kind uses the existing exact artifact/content/
delivery routes through the separately registered
[`project_export_transport_inventory`](../../contracts/public/v1/project-workflow.schema.json).
The [project owner](project-workflow.md#focused-artifact-and-delivery-transport)
defines its complete selectors, wrappers, binary observation, subject-selected
minimum grant mapping and finite delivery profile. Preserve bodyless exact GETs,
read-only content POST, current complete disclosure and retained receipt
distinctions. These registrations do not change the original 23-entry or
eight-entry additive MVP inventories or the closed financial/lifecycle types.
Qualification of these contracts does not enable the missing asynchronous
project export operation or actual streaming service.

### Completeness of the additive MVP contracts

The additive registrations bind each exact operation to its method, route,
request symbol, response symbol, scope and admitted errors. Swapping symbols
between operations must fail. Structural `{}` negatives and empty positives do
not qualify consequential semantic content or canonical cross-record joins.

Self-principal revocation returns the complete `principal_identity` in revoked
state, including its current control observation; a two-field principal reference
is insufficient. Project-change grouping retains workspace, exact project and
capture identities, complete typed item content and review/control observations,
plus the bounded continuation's snapshot identity. Project readback carries
accepted records/mappings, source/context provenance, qualified expense
associations and separately identified pending changes; its closed shape must
permit all of them. Existing canonical limits and disclosure remain in force.

Project financial composition freezes project/currency selectors, exact selected
expense/association revisions, exclusions, server-calculated totals and observed
availability control. Reusing a workspace report without these bindings does not
produce a project report. Project export carries exact revision/control inputs,
lossless independently usable JSON/reversible CSV payload profiles, complete
sealed artifact/delivery identity and current availability restrictions. An ID,
profile and hash alone cannot qualify that export. Project capture binding retains
its required source/context identity in both admission and returned canonical
binding; source context is not an optional discarded annotation.

Require nonempty complete positive values proving these fields and targeted
negative/semantic cases for mismatched identities, revisions, hashes, scope,
control, disclosure and substituted contracts. Keep independently expected values
separate from schema or implementation generation. Complete and independently
review these contracts before admitting their CLI consumers.
