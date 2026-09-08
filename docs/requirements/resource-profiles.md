# Resource and workload profiles

**Authority:** normative requirements. **Status:** accepted qualification targets;
no capacity or service-level result is implied.

This document owns resource envelopes, corpus/hardware definitions and shared
admission budgets. [Storage and search](storage-and-search.md) owns retrieval
operation limits; [API](api.md) owns operation and idempotency semantics.
[Acceptance](acceptance.md) owns experiment workloads and pass/fail thresholds.
An implementation must publish a versioned processing/deployment profile that
enforces these limits and identifies any narrower supported envelope.

## Initial resource envelope

MiB and GiB are binary byte units. Wire, decoded and compiled sizes are separate
limits; increasing one global request size does not qualify multimodal ingestion.

| Resource | Required initial bound and rejection behavior |
| --- | --- |
| JSON command | At most 1 MiB, rejected before parsing; also impose explicit nesting and collection bounds. |
| Text, Markdown or JSON artifact | At most 10 MiB of source; analysis chunks and derived output have separate bounds. |
| JPEG or PNG | At most 25 MiB encoded and 40 million decoded pixels; reject unsupported animation and format variants. |
| Audio | Qualified WAV or non-DRM M4A only, at most 20 MiB and five minutes decoded duration; bound transcript length separately. |
| Capture | At most eight attachments and 100 MiB combined input; roles and the submitted attachment set are explicit and atomic. |
| Batch import | At most 100 manifest entries, streamed; default two simultaneous transfers per CLI process. |
| Remote acquisition | HTTPS only, zero redirects initially, five-second connect deadline and 120-second total acquisition deadline. Enforce the stream byte cap even without Content-Length. |
| API control request | Ten-second request deadline; a database transaction budget of two seconds, apart from explicitly controlled administrative jobs. |
| Internal gRPC control message | At most 1 MiB by default; bulk content uses authorized blob references. Each stage declares its own deadline. |
| Context build | At most 100 asset versions and 256 KiB compiled text. |
| Search | Apply the single retrieval budget defined by [storage and search](storage-and-search.md); fan-out and retries cannot create new budgets. |
| Active model work | Initially two active jobs per organization, also subject to explicit global worker slots and cost reservations. |
| Admitted backlog | At most 100 pending jobs per organization. Reject additional work with retry guidance instead of accumulating an unbounded queue. |
| Initial live-processing reservation | USD 1 per capture is the qualification cost ceiling. It is neither a price promise nor spending authority. |
| Parser scratch and execution | Every job has explicit byte, inode, CPU, wall-time and memory limits and bounded derived output; use separate scratch and crash cleanup. |
| Downloads | Explicit stream duration and buffer bounds; use resumability where supported. Do not hold a transaction for a slow client. |
| Restricted-data download mode | Proxy-authorized initially. A separately qualified direct-download grant may last no more than 60 seconds and must disclose residual validity. |

Counts or deadlines not assigned a number above are required profile fields to
select and qualify before the consuming adapter accepts work. Missing limits do
not mean unlimited. Freeze JSON depth, collection/chunk/transcript counts, scratch,
retry, operation lifetime and stream budgets in their owning machine contracts;
do not invent universal values or accept unsupported profiles silently.

Owners may reduce a supported envelope explicitly. Raising an accepted cap or
changing a qualification commitment requires measured evidence and a reviewed
profile amendment that preserves the earlier criterion and its result. Local
inference has its own hardware/context envelope; hosted-inference memory does not
qualify a local model. Unsupported format or scale is rejected and documented.

## Lifecycle and operation profile qualification

The [shared operation contract](lifecycle-contract.md#3-shared-operation-plan-clocks-and-evidence)
owns finite wire capacities, execution_limits and clock arithmetic. A selected
profile must preflight the entire permitted combined execution: stage/attempt
counts, all effects/exposures/material actions, sampled progress, complete encoded
responses and reserved late-confirmation/control headroom. Individual maxima are
not operational defaults and may not fit together. Every operation/stage/attempt,
reconciliation and cancellation-observation lifetime is finite; retries, lease
takeover and reads never reset its fixed deadline. Bound actual adapter memory,
scratch, concurrency, costs and all temporary copies as well as wire records.

The [continuity contract](lifecycle-contract.md#11-retention-recognition-and-independent-continuity)
requires a positive finite intent_prepare_lifetime_ms in the qualified integrity
profile, actual retained recognition/storage quotas, bounded complete evidence
records/batches, and selected terminal-relative replay/diagnostic durations.
All UTC arithmetic must remain representable. Expired preparation is not abort
proof; unknown outcomes cannot release consumed capacity. Passive authenticated
receipt ingestion needs its own finite request/transaction/authority and concrete
producer/verifier before use; it grants no fresh outbound business action.

The [financial delivery profile](lifecycle-contract.md#10-sealed-financial-export-and-bounded-delivery)
requires every generation, retry, staging, artifact lifetime, transfer total/idle,
buffer, organization/global slot and temporary-byte field before support. Qualify
actual complete 64 MiB artifacts and throughput within those limits. This exact
financial download has no Range/resume or presigned variant; its fixed bulk
deadline is separate from the ten-second control-request deadline and preserves
the ordinary two-second transaction budget. A new delivery never extends the
artifact's fixed expiry. Later complete customer archives need their own profile.

The [repair owner](lifecycle-contract.md#required-bounded-repair-and-complete-coverage-at-e124)
requires finite grants and one bounded complete contiguous range per operation,
including byte-sensitive range sizing, existing/late confirmation headroom and
all retained failed-attempt history. An inventory larger than one operation is
handled by separately admitted bounded batches; no arbitrary whole-organization
or whole-obligation cardinality cap, truncation or hidden continuation is allowed.
These profile duties preserve the existing shared model/backlog/global-slot and
USD 1-per-capture qualification ceiling without providing spending authority.

## Shared admission from the first adapter

Essential admission arrives with the first model, transfer and parser adapters,
before any real-data use. Later consolidation strengthens the same controls; it
cannot be the first enforcement of quotas. Per-process counters are insufficient
when several API instances, workers, tokens or CLI processes share an organization.
Use canonical reservations/leases or a single controlled dispatch authority with
testable atomic claims. Do not add a second quota system merely for a later lane.

Admission combines organization fairness, pending and active jobs, global worker
slots, storage classes, memory classes and cost. Reserve before performing expensive
or external work. Define ownership, expiry, renewal, fencing and recovery of each
claim. Cancellation, timeout and a dead worker must not release resources still
consumed by an unresolved side effect. Reconcile realized cost and uncertain
provider outcomes without allowing another attempt to bypass the reservation.
Define bounded attempts and a total deadline for every stage, not merely a timeout
per retry. Preserve accepted durable work under overload and report denial/backlog
honestly. No new Redis dependency is required merely to implement shared admission.

Decoded images, audio normalization and models need distinct resource classes and
a combined memory budget. Account for original bytes, normalized data, transcripts,
chunks, vectors, evidence edges, audit, exports and backups rather than charging
only the source upload. Bound temporary storage during retries, rebuilds and
overlapping generations. Every failure path must release or record responsibility
for controlled resources; cleanup cannot delete another job's data.

## Corpus definitions

| Profile | Exact qualification content |
| --- | --- |
| D0 functional | Two synthetic organizations and 100 captures/scenarios, including text, receipt-image/voice-note pairs, conflicting values, Unicode, timezones, duplicate submissions, refunds and malicious files. No real personal data. |
| D1 pilot | Ten organizations; 100,000 captures; 200,000 artifact versions; 1,000,000 observations; 100,000 asset versions; 500,000 vectors with 1,024 float32 components each; 30 GiB original and derived bytes. |
| D1 authorization selectivity | Test 0%, 0.1%, 1%, 10% and 100% eligible evidence under declared policies. Ground truth includes the authorized set and source identities. |
| D2 growth | Ten times D1 cardinalities and byte volume: 100 organizations, 1,000,000 captures, 2,000,000 artifact versions, 10,000,000 observations, 1,000,000 asset versions, 5,000,000 vectors and 300 GiB original/derived bytes. Vector dimensionality remains 1,024. This is post-MVP qualification, not supported launch scale. |

Version generators, scenario identities, seed, sizes, digests and ground-truth
selection. The D0 scenario inventory and real holdout are different assets with
different evidence purposes; [testing and coverage](testing-and-coverage.md)
governs deterministic fixtures, and [acceptance](acceptance.md) governs live scoring.

## Hardware definitions

H0 is a native macOS host with at least 16 GiB total memory, Docker limited to
four vCPU and eight GiB memory, and 50 GiB free disk. Hosted-inference memory is
outside that local budget. Neo4j is disabled. H0 establishes a bounded local
feasibility profile, not universal laptop or local-model support.

H1 has the following component allocations. These are qualification resources,
not claims that any environment has been provisioned.

| Component | Instance count | vCPU per instance | GiB memory per instance |
| --- | --- | --- | --- |
| API | 2 | 2 | 2 |
| Python worker | 1 | 4 | 4 |
| Application PostgreSQL | 1 | 4 | 8 |
| Qdrant | 1 | 4 | 8 |
| Temporal service | 1 | 2 | 2 |
| Separate Temporal PostgreSQL | 1 | 2 | 4 |

The near-database microbenchmark records and requires round-trip latency no greater
than ten milliseconds. Deployment qualification also records its actual region
and latency; a result on a nearby database does not qualify a remote deployment.
Native macOS/Linux CLI tests and Linux AMD64 service qualification use the supported
profiles in [dependency qualification](dependency-qualification.md) and
[deployment](deployment.md). Emulation is not native capacity evidence.

## Database and storage capacity

Maintain an explicit allocation satisfying:

```text
sum(maximum instances for role * maximum connections per instance)
  + direct administration allowance
  + migration and repair allowance
  + database safety reserve
  <= qualified database connection budget
```

The initial application-database qualification allocation totals at most 80
connections: two APIs with 16 each, two Activity-worker pools with eight each,
eight total for dispatcher/projector work, eight shared by administration/migration/
repair, and 16 reserved for safety. Distinguish process pool allocations from the
hardware role layout. Temporal has a separately qualified database budget.
Connection-per-request and pool-per-tenant designs are prohibited. Do not raise
pools past the allocation to conceal queueing or an unsupported isolation model.

Raw vector storage is only `count * dimensions * bytes per component`. D1's vectors
alone require 2,048,000,000 bytes, about 1.91 GiB, before graph indexes, payloads,
optimizer duplication, filesystem cache and replicas. Qualification includes
peak memory/disk during optimization, cold reads and simultaneous old/new rebuild
generations. Warm-cache latency alone cannot establish capacity.

## Bounded implementation and measurements

Hash acquisition streams rather than loading an attachment set into memory. Reuse
accepted derivatives keyed by exact input and processing-profile digest; avoid
re-extracting unchanged input on every retry. Rebuild projections from retained,
authorized embeddings instead of automatically re-embedding the corpus. Candidate
selection and contradiction checking must be bounded; a global pairwise comparison
of all claims is not an MVP algorithm. Use bounded canonical batch authorization,
not one database request per search candidate.

Measure admission and rejection, queue age, throughput, retry reason, idempotent
replay, acquired/sealed/decoded bytes, parser RSS/CPU, reserved versus realized cost,
pool occupancy/wait, transaction duration, authorization latency, candidate/result
ratio, exhausted search budgets, projection lag, oldest outbox age, release/resolve
latency, revocation admission latency, orphan bytes and restore/rebuild time.
Operational metric labels remain bounded and omit content and customer identifiers;
controlled audit and exposure records have their own access/retention policy.

Each result records exact versions, source/profile/configuration, dependency/image
identity, hardware limits, dataset, workload generator, clock source, region/RTT,
warmup, cache/optimizer state and authorized spend. Report cold, nominal, overload
and recovery behavior separately, including rejection and partial-result behavior.
Do not convert these engineering targets into customer service promises without
the actual [acceptance evidence](acceptance.md).
