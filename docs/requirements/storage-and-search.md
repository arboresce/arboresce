# Canonical storage and derived search

Status: accepted normative requirements; database adapters, storage services
and search are not implemented. Physical DDL, exact query contracts and
backend behavior require their owning implementation and qualification steps.
Conceptual keys below are requirements, not a claim that named tables exist.

This document owns relational integrity, atomic audit/outbox, sealing,
projection identity/rebuild and bounded retrieval. [Data model](data-model.md)
owns domain values; [API](api.md) owns transport/admission behavior;
[financial records](financial-records.md) owns monetary semantics.

## Canonical and derived responsibilities

PostgreSQL is the canonical authority for organization, membership, policy,
capture/evidence, review, expense, asset, build, channel, operation and outbox
state. Object storage retains exact original and derived bytes referenced by
that state. Temporal retains workflow history, not a second approval store.
Qdrant supplies derived retrieval candidates and cannot authorize them.
Atomic event/audit records do not require full event sourcing of every product
aggregate. Canonical relational state remains independently queryable.

The system must remain correct when projection delivery is delayed or search
is unavailable. Exact eligible build resolution does not require search.
The [architecture](architecture.md) owns service boundaries and deferred graph
features. [Dependency qualification](dependency-qualification.md) owns selected
versions; [deployment](deployment.md) owns concrete operating profiles without
changing these invariants.

## Tenant-safe relational enforcement

Use `organization_id` in canonical tenant keys and consequential relationships.
An artifact-version reference, capture attachment, evidence dependency, report
selection or build edge must have a same-tenant foreign key or equally explicit
validated constraint. IDs and content hashes are not access grants.

| Record family | Required persistence boundaries |
| --- | --- |
| Principals, memberships, domains | Unique canonical issuer/subject mapping independent of email; organization/principal membership uniqueness, active state, policy epoch and membership revision; serialized initial-owner creation. |
| Uploads and captures | Separate session/generation, sealed object binding, immutable submitted capture revision and ordered required/optional attachment association. |
| Artifacts and objects | Distinguish user-facing artifact identity, immutable versions and server-owned object identity; account for every authorized reference and hold. |
| Processing | Separate operation, attempt and accepted result, with source revision, exact input/profile identity and current fencing token. |
| Candidates, reviews, assets | Immutable content revisions, mutable current heads/availability, exact review links, one accepted mapping per candidate revision and reverse evidence dependencies. |
| Expenses and reports | Exact reviewed component types, immutable expense revisions and report selections, original refund/credit links and current eligibility; no requirement to publish knowledge. |
| Builds, receipts, channels | Separate payload bytes/input edges, evaluations/approvals/signatures, domain association and mutable monotonic channel generation. |
| Audit and deliveries | Stable canonical event identity and separate per-consumer delivery state, bounded metadata, leases and fencing. |
| Lifecycle | Separate logical restrictions, policy/hold/reference accounting, physical purge obligations, minimal tombstones and recovery journal completeness. |

Avoid arbitrary polymorphic strings where a real foreign key is possible.
Generalized feedback uses a constrained subject variant with explicit
referential and deletion behavior. JSONB is appropriate for bounded extensible
metadata and schema-shaped documents. Ownership, lifecycle state, dates, money,
ordering/filter keys and important references need typed relational enforcement;
they cannot exist only in unvalidated JSON.

Use non-owner application roles without `BYPASSRLS`, forced RLS on tenant
relations, and transaction-local tenant context. Missing context fails closed.
Transaction-pool reuse must not retain tenant/session state. Migrations and
administration have separate direct-connection configuration and controlled
roles. Parameterize queries and keep sensitive SQL bind values out of logs.
Actual pooled/direct compatibility and total connection allocation require
their [resource](resource-profiles.md) and dependency qualification evidence.
Transaction-pooled application work cannot rely on `LISTEN/NOTIFY`, session
advisory locks, persistent temporary tables or session affinity. Qualify actual
SQLx prepared-statement behavior through the selected pooler. Keep migration
credentials out of ordinary application containers; upgrade compatibility and
expand–backfill–contract sequencing follow [deployment](deployment.md).

## Current authorization and short transactions

The [API admission sequence](api.md#idempotent-command-admission) binds current
policy, idempotency and aggregate revision to one short transaction. Lock policy
and membership rows in a deterministic order shared by permission writers;
for example, the chosen shared admission lock must conflict with the writers'
update lock. Also lock/CAS the affected aggregate and enforce database
uniqueness. RLS alone does not resolve a concurrent multi-table policy change.

Disabled membership, purpose restrictions and source-audience intersections
must be resolved canonically as required by [security and privacy](security-and-privacy.md).
Do not combine audiences with a union or trust a stale cache as admission.
Same-revision concurrent transitions have one winner; bounded deadlock retry
preserves the same command identity.

Hold no database locks across provider calls, byte streaming, media parsing or
long-running work. Perform the bounded effect outside the transaction, then
recheck applicable authority, lease/fence and revision before binding its
accepted result in another short transaction.

## Atomic audit and fenced outbox

Commit command result, domain transition, audit record and outbox event
atomically. Rollback leaves none of them accepted. A replay must not duplicate
the transition or event. Keep event payloads bounded and reference immutable
content rather than copying full documents or secrets into events/histories.

Each consumer has its own delivery state. Lease a bounded due batch with a
defined owner, expiry and fencing value. Reclaim expired work through a bounded
query. Concurrent dispatchers cannot share a live lease, and an old owner cannot
acknowledge or finalize after takeover. Attempts, backoff, total retry budgets
and dead-letter handling are explicit and observable.

External execution can complete before its acknowledgement is stored. Reconcile
the stable operation/result identity on retry. Do not claim exactly-once
delivery or use attempt number as business identity. Accepted stage results
are unique by organization, operation, stage, input digest and profile digest,
and stale completions cannot replace them. [Processing](processing.md) owns
workflow and provider uncertainty behavior.

An allocated event ID, sequence maximum or largest observed delivery is not
commit completeness. Concurrent transactions can allocate IDs and commit in a
different order, and rolled-back allocations can leave gaps. Track actual
event/consumer outcomes and explicit capture boundaries; never advance a
completion watermark merely to the largest ID.

## Sealed objects and finalization

Maintain separate staging and server-only sealed storage permissions. A client
can upload only to its scoped staging grant; it cannot write a sealed key.
Reusable storage authorization is not proof of a single accepted finalization.
Validate allocation against shared quotas, allowed media, byte bounds and
organization ownership before granting access.

Finalization must:

1. Validate upload identity/generation, expected length/digest, expiry and
   current authority. Claim a finalization lease with fencing, then end the
   database transaction.
2. Read the staging object once as a bounded stream into a new server-owned
   sealed object. Incrementally hash the exact bytes written and enforce
   encoded/decode and quarantine requirements.
3. Compare actual length and SHA-256 with the declared expectations. An ETag
   is not a SHA-256 shortcut. Backend versioning or conditional-copy guarantees
   may be used only after backend-specific qualification.
4. In a short fenced transaction, bind the verified sealed reference once to
   the upload/artifact/capture and append the required audit/outbox effect.
   An expired lease or stale generation cannot replace the accepted binding.
5. Reconcile unreferenced staging or sealed objects after a documented safety
   delay, checking ownership, active finalizers, references and holds.

A still-valid staging URL or concurrent staging overwrite cannot alter the
accepted original. A crash can leave an orphan, but cannot make unverified or
client-mutable bytes canonical. Retries use recorded finalization identity;
expired/aborted sessions cannot resume into success.

Reconciliation provides bounded dry-run reporting and requires the deletion
authority applicable to the operation. Interrupted cleanup is recoverable;
another tenant's paths, live finalizers and shared/held objects remain protected.
New canonical references cannot appear merely because an orphan scanner finds
plausible bytes.

Originals, normalized representations and compiled output have separate exact
identities. Deduplication is scoped and reference-aware; a hash match cannot
broaden access, become a cross-tenant existence oracle or authorize deletion
of bytes another reference still needs.

Authorized downloads bind exact digest/length and current control state, use
bounded transfer deadlines, and release resources for slow or cancelled clients.
[Security and privacy](security-and-privacy.md) owns admission and direct/proxy
grant policy; search state cannot grant or deny canonical object ownership.

## Query-driven indexes

Choose physical names and indexes from required access patterns and measured
plans, not from an assumed production schema. Required uniqueness and candidate
query shapes include:

| Access pattern | Key or index shape |
| --- | --- |
| Canonical principal/membership | Unique `(issuer, subject)` and `(organization_id, principal_id)`. |
| Recent domain captures | Tenant/domain followed by immutable recorded time and ID in the declared descending order. |
| Pending review queue | Tenant/domain, creation time and ID, with an appropriate pending-state partial predicate or state key selected from the workload. |
| Immutable revisions | Unique `(organization_id, aggregate_id, revision)`. |
| Evidence impact/revocation | Tenant, artifact version and dependent kind/identity, with reverse traversal support. |
| Current eligible expenses | Tenant, expense date and expense identity on a bounded current-head relation; additional eligibility/currency fields require plan evidence. |
| Channel CAS | Unique `(organization_id, domain_id, channel_name)`. |
| Build dependencies | Tenant/build/asset-version association and its asset-to-build reverse index. |
| Idempotency | Unique organization/principal/command-kind/key-digest claim. |
| Due outbox work | Partial index on due time/event identity for pending deliveries, with a separate bounded expired-lease query. |
| Accepted stage reuse | Unique organization/operation/stage/input/profile identity. |
| Subject audit | Tenant, subject kind/identity, recording time and stable event identity. |

Index foreign-key referencing columns when required joins/deletes need them;
declaring a foreign key does not supply those indexes. Representative high-volume
queries require actual `EXPLAIN (ANALYZE, BUFFERS)` evidence with realistic
selectivities and maintenance costs. [API](api.md#listing-and-filters) owns
keyset cursor semantics.

Do not add blanket JSON GIN indexes, standalone low-selectivity Boolean indexes,
redundant composite-prefix indexes, trigram indexes on all text, unneeded
geospatial indexes or premature partitioning. There is no general analytics or
geospatial MVP requirement. A PostgreSQL full-text fallback, if selected, needs
a bounded purpose-filtered representation, explicit text-search configuration
and measured maintenance/authorization behavior; it is not an implicit second
search product.

## Projection identity and delivery

Qdrant collections contain compatible model/dimension profiles and indexed
tenant/filter fields. Each point identifies organization, immutable content or
representation revision, chunk and embedding-profile identity within a projection
generation. A point ID is not the public asset ID or its current head. Reject
dimension/profile mismatches, absent tenant constraints, oversized queries and
unqualified/unindexed filter paths.
Derived records retain the applicable classification/filter metadata alongside
canonical identity, version and generation, while current authorization still
comes from PostgreSQL. Any later graph projection has the same derived-authority
boundary and remains deferred until its own workflow and qualification exist.

Embedding work is keyed by exact text, profile and permitted scope. Reuse stored
embeddings only for unchanged content/profile with current permitted use.
Changed model dimensions require a new profile. Both content and query
embeddings use the [processing gateway](processing.md#model-gateway-and-shared-reservations)
and its shared exposure/cost controls.

Projection dispatch consumes fenced per-consumer outbox work with bounded
retry and backpressure. Duplicate or reordered events must converge without
letting stale updates redefine current content or acknowledge another owner's
delivery. Delayed additions cannot resurrect authority after revocation.
Prioritize invalidation/deletion delivery, but canonical admission already
enforces the restriction when the projector is paused.

## Bounded search

MVP search requests `k` from **1 through 20**. One operation has at most
**two total retrieval rounds** and **200 cumulative candidates before
deduplication across all retrieval sources**. The initial retrieval consumes
one round; at most one refill remains. Hybrid fan-out, duplicates and refills
share these counters, rather than receiving independent per-source budgets.
Allocate and enforce the remaining budget before dispatching each branch.

The search pipeline is:

1. Authorize the request and validate typed filters, purpose, requested result
   count and the shared retrieval budget.
2. Retrieve bounded candidates from the active permitted generation with
   mandatory tenant/filter constraints.
3. Fetch candidate canonical metadata and authorization dependencies in one
   bounded SQL batch. Drop inaccessible, superseded or unavailable candidates
   before their content reaches a reranker.
4. Rerank only permitted content, then deduplicate exact asset-version/chunk
   identities with stable ties. Query text sent externally also requires
   gateway authorization.
5. If short, refill only within the remaining shared round/candidate allowance.
   Return permitted results and explicit safe freshness/incompleteness or
   budget-exhaustion indicators.

Never loop until a result page fills under restrictive permissions. Avoid N+1
authorization, count candidates before deduplication, and do not expose denied
candidate counts through totals, diagnostics or receipts. Permission filtering
must precede external reranking, not merely final response formatting.

A selection receipt binds actual selected version/chunk identities, output
identity, source build when applicable, retrieval/profile version, generation,
purpose/policy context, limits and truncation/incompleteness. Preserve relevant
qualifications and make any omission explicit within authorized disclosure.
It is not an approval over an exact snapshot and cannot reuse a full-build
attestation for different bytes. [API](api.md#exact-snapshot-resolution-and-search-selection)
owns the separate snapshot/selection operation boundary.

These retrieval limits are accepted constraints requiring benchmark evidence,
not measured performance promises. [Acceptance](acceptance.md) owns quality,
latency and workload thresholds; [resource profiles](resource-profiles.md) owns
the other service, parser and capacity budgets.

## Projection rebuild and caches

MVP rebuild uses an explicit scoped mutation barrier:

1. Acquire the barrier for the selected projection scope and continue permitted
   reads from the old generation.
2. Pause or explicitly queue all writes that can affect it, and drain/record
   prior accepted projection work under a documented completeness boundary.
3. Read a consistent canonical snapshot and construct a new generation,
   reusing unchanged content/profile embeddings without a transaction spanning
   a model call.
4. Reconcile membership, counts and per-record digests. Reject mixed model,
   dimension, content or generation results.
5. Atomically switch the active generation in PostgreSQL only after success.
6. Resume affected mutations and retain the old generation for a bounded
   rollback window, with explicit transition and rollback receipts.

Every affected writer must honor the barrier. A failure retains the old active
generation; neither a partial build nor a maximum-event-ID shortcut proves
completeness. If the approved maintenance window is insufficient, a separately
designed/tested online algorithm must define a real capture boundary before use.
Generation rollback never bypasses current lifecycle restrictions.

Caches are bounded and justified by measured repeated work. Keys include
organization, principal/delegation, purpose, policy epoch, exact selected
content, processing/retrieval profile and projection generation. Query text
alone is insufficient. Cache hits never replace current canonical admission;
revocation remains effective despite stale caches, old generations or delayed
invalidation. Verify bounded eviction and no cross-tenant hash oracle.

## Retention, purge and recovery

Account for write amplification across originals, representations, transcripts,
chunks, vectors, dependency edges, traces, exports, events and backups in the
resource/retention model. Charging or bounding only original bytes is incomplete.
Do not copy full documents into every event or workflow payload.

Logical revocation, retention/hold and physical purge remain separate. Purge
rechecks policy, active holds, leases and all surviving references before
deleting controlled copies; retries cannot delete shared content or pretend
partial deletion succeeded. Minimal tombstones and deletion evidence have a
documented purpose and minimization policy. The generic lifecycle contract can
use clearly synthetic policy values in tests; unknown actual deployment policy
cannot authorize real-data purge.

[Security and privacy](security-and-privacy.md) owns access and external-copy
limits. [Recovery](recovery.md) owns independently retained lifecycle journal
completeness and restoration before serving resumes. Restoring an old backup
or rebuilding a projection must not resurrect access revoked after that backup.

## Required verification

Real-service suites must exercise empty/upgrade migrations, tenant/session reuse,
foreign references, policy/revision races, outbox rollback, lease takeover,
crash-after-effect, concurrent staging overwrite, stale finalization, orphan
cleanup, shared/held bytes, reordering, query-plan bounds, restrictive search,
shared budget exhaustion, failed rebuild and cache/revocation races.

Use synchronized barriers and independent expected outcomes under
[testing and coverage](testing-and-coverage.md). Register actual services and
process profiles when introduced; skipped required dependencies cannot qualify
the slice. [Acceptance](acceptance.md) owns experiment-scale counts and targets.
