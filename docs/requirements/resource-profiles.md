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
| Listing page | 1–100 entries, default 20; each complete summary item at most 4096 UTF-8 bytes. |
| Listing/exact companion GET target | At most 8192 raw origin-relative bytes and 32 query pairs, checked before allocation; invalid_request/400 on overflow. |
| Listing cursor | At most 4096 ASCII token bytes and 2048 decoded payload bytes; fixed nonrenewing lifetime 900000 ms, at most four accepted key selectors and one issuer. Defensive caps do not imply a canonical value can reach them. |
| Search query/request | Query 1–4096 Unicode scalars and at most 16384 decoded UTF-8 bytes; complete POST request at most 65536 UTF-8 bytes before parsing. |
| Search output | Complete canonical selection document at most 262144 bytes, also bounded by explicit caller byte/token limits and an exact qualified tokenizer; requested token limit 1–1048576. |
| Listing/search response | Complete encoded response at most 524288 UTF-8 bytes, retaining common depth 16 and collection/additive-reader bounds. |
| Complete content GET | Complete current/exact response at most 1048576 UTF-8 bytes; no omitted consequential content. |
| Exact asset supersession GET | Complete record response at most 65536 UTF-8 bytes, with bounded endpoint references and no inline endpoint content/history chain. |
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

## Bounded source JSON processing

`source-json-processing-v1` is the separate pure parsing and selection profile
for the [source JSON semantics](observation-contract.md#json-pointer-and-selected-value-identity).
It does not inherit control-object key, string, minimum-length or depth limits.
Admission checks the entire original, including values outside the selected path.
An unsupported selection does not authorize deletion or alteration of an original
retained by acquisition.

| Dimension | Inclusive ceiling |
| --- | --- |
| Original UTF-8 input | 10485760 bytes, checked before decoding |
| Whole value depth, root counted as one | 128 |
| Whole value nodes, excluding object keys | 262144 |
| Total object members | 131072 |
| Members in one object | 65536 |
| Elements in one array | 131072 |
| Original numeric token | 4096 ASCII bytes, with exact decimal admission |
| Selected value depth, root counted as one | 64 |
| Selected value nodes, excluding keys | 65536 |
| Complete selected canonical output | 10485760 bytes |

The requested-payload/stack estimate has a 1073741824-byte ceiling. The logical
byte/fixed-node work estimate has a 68719476736-unit ceiling. With I,N,M the whole
input bytes, nodes and members, C,S the selected canonical bytes and nodes, D the
whole depth and Ds the selected depth, their reservation models are:

```text
estimated bytes = 8I + 512N + 256M + 32C + 1024S + 8192D + 1048576
logical work   = 64(I+C) + 8192N + 256(I+M)*18 + 16Ds*C + 4096S + 1048576
```

The structural ceilings imply conservative maxima of 656408576 estimated bytes
and 63418925056 work units. These are reservation estimates, not allocator or
RSS measurements, machine instructions, latency or pure host-OOM recovery.
They do not replace actual supported-platform native maxima, wall/CPU/memory
measurements, process limits and failure supervision. Own fallible reservation
errors reject admission. String decoding, scalar formatting and the selected
canonicalizer also use infallible allocations; unavailable host memory may abort
the consuming process.

The source proof must bind actual compiler layouts (Node at most 128 and Member
at most 32 bytes), explicit capped minimum/doubling requests, child-index storage,
single-token decoder scratch and nonrecursive owned cleanup. For N>=2, Node
old/new overlap is at most384N bytes and array child slots at most 32(N-1), jointly
below 512N. At N=1 the initial four slots use512 bytes and no child edges exist.
Member storage at most 128M plus two exact-size index buffers at 16M fits 256M.
Decoded text, one-token scratch/result and relocation overlap must fit 8I.

Selected exact escaped output length is checked before opaque JCS object work.
Completed sibling properties and active prefixes partition selected content;
popping an object retains properties while copying into its parent or output.
The 32C allowance must cover this overlap, requested vector growth, final output,
key re-decode and UTF-16 storage; 1024S separately covers fixed container overhead.
Control escaping can expand sixfold. A final writer cap alone is insufficient.
Actual parser/serializer frames must establish 8192D stack dominance. Own decoded
key merge/equality checks take at most 17 passes; 18 is the surrounding-work factor,
not the pinned BTree insertion comparison bound. The selected Rust1.99 BTree has
at most six levels at 65536 keys, hence at most 66 comparisons per insertion.
Formatter callbacks allocate boxed writer references; charge their repeated
escape activity and complete output/hash passes to I+C. Nested copy/growth work
must fit 16Ds*C. Independently review these source and compiler obligations and
exercise actual maximum and otherwise-valid one-over inputs before acceptance.

### Python source JSON reference

`source-json-python-reference-v1` retains every original and selected structural,
numeric, pointer and byte ceiling above. It selects the exact prepared CPython
and independent JCS subset in
[dependency qualification](dependency-qualification.md#independent-python-source-json-reference).
Its separate requested-payload/interpreter-frame reservation model is:

```text
estimated bytes = 32I + 512N + 256M + 40C + 1024S + 8192D + 1048576
logical work   = 64(I+C) + 8192N + 256(I+M)*18 + 16Ds*C + 4096S + 1048576
```

The structural maximum is 991952896 estimated bytes and 63418925056 work units,
within the unchanged 1073741824-byte and 68719476736-unit caps. The Rust model
and its accepted evidence remain distinct. This estimate does not bound physical
RSS, arbitrary host/runtime configuration, OS/library stack use or host-OOM survival.
Actual native/process/platform measurements and independent proof remain required
before the reference profile is accepted.

The input term covers original bytes, whole wide Unicode text, one quoted slice,
decoder old/new/promotion/shrink buffers and substring scratch, retained decoded
values, UTF-8 surrogate checking, cached UTF-16 keys and bounded numeric copies.
Use actual pinned object layouts. The node/member terms include immutable records,
metric/index/float/Unicode headers, list growth and tuple overlap, active ancestor
collections, merge scratch and flat rejection cleanup. Members and array edges
partition the admitted tree; keys do not count as value nodes.

Exact selected escaped size/depth/count precede private list/dict/key construction.
The selected fixed-node term covers key/ordinal/container/item/sort objects and
shallow-copy table/index storage. UTF-16-sorted private keys use distinct ordinal
hashes in fresh append-only general-key dictionaries; copies preserve hash/order.
The independent writer's UTF-16 sort sees one ascending run. Prove these exact
paths without a randomized hash-seed assumption or ordinary-string key mixing.

The output term also covers regex escaping's nonmatching string chunks and their
headers/reference lists, wide Unicode joins, UTF-8 encoding, key/sort payloads and
BytesIO growth/final ownership. A byte-buffer-only estimate is insufficient.
Account simultaneous lifetimes and relocation. The private sink checks growth
before append and exact final length before complete output/digest publication.

Source/code-frame evidence must bind the installed interpreter ABI, recursive
parser/tree/serializer and rejection/deallocation paths, dependency callbacks and
ordinary native runtime assumptions. A successful small-stack probe alone is not
a universal stack proof. Retain exact archive/build/backend/allocator provenance,
actual maxima, otherwise-valid one-over, wide escaping, long-prefix and dictionary
resize discriminators, complete strict source/helper/process collection and
independent Rust/Python agreement. Service and operating qualification remain at
their actual consuming gates.

## Listing and selection profile qualification

The [selection contract](selection-contract.md) owns exact wire compositions and
separate listing/search semantics for the bounds above. These ceilings select
protocol capacity, not operational defaults or measured throughput. Actual
profiles must supply narrower finite byte/token/model input/output, timeout,
retry, concurrency, scratch, cost, storage and retention limits before admission.
No provider/key/tokenizer selection is implied. Search reserves the original
two-round/200-slot budget across sources and shares organization/global admission.
An uncertain dispatch cannot return charged capacity or reset its deadline.

Preflight the actual combined serializer, including escaped Unicode, required
content/review/attribution, full receipt and emitted extensions. Candidate/asset
producers reserve maximum fixed future review/successor/head/control and immediate
relationship-reference overhead before content writes. The supersession producer
also preflights the complete record and each affected required asset-read
composition without reconstructing unavailable old content or walking history.
No individual field or preimage limit proves that these compositions fit.

All preparation, bounded SQL, output-prefix tokenization and delivery share the
original ten-second request deadline; each ordinary transaction remains at most
two seconds. Do variable-size preparation outside locks, finalize only qualified
bounded framing/time/hash fields and fence admission inside the short transaction,
and stream after commit. Search evaluates at most 21 complete ranked prefixes
without assuming monotonic token counts; no field truncation or hidden refill.
Read-run audit/query/receipt and gateway records need explicit finite quotas and
retention policy, without borrowing durable-command replay/recognition lifetime.

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
| D0-project focused | Versioned synthetic receipt/approximately 30-second audio/context and review scenarios under [project fixtures](project-fixtures.md); two workspaces including an actor admitted to both, parties/projects, ambiguity/unknowns, per-item dependencies/manual/follow-up revisions, financial review separation, races/failure and old-backup restriction restore. Exact inventory/count/seed/sizes/hashes are frozen before consumers; no personal data. |
| D0 functional | Two synthetic organizations and 100 captures/scenarios, including text, receipt-image/voice-note pairs, conflicting values, Unicode, timezones, duplicate submissions, refunds and malicious files. No real personal data. |
| D1 pilot | Ten organizations; 100,000 captures; 200,000 artifact versions; 1,000,000 observations; 100,000 asset versions; 500,000 vectors with 1,024 float32 components each; 30 GiB original and derived bytes. |
| D1 authorization selectivity | Test 0%, 0.1%, 1%, 10% and 100% eligible evidence under declared policies. Ground truth includes the authorized set and source identities. |
| D2 growth | Ten times D1 cardinalities and byte volume: 100 organizations, 1,000,000 captures, 2,000,000 artifact versions, 10,000,000 observations, 1,000,000 asset versions, 5,000,000 vectors and 300 GiB original/derived bytes. Vector dimensionality remains 1,024. This is post-MVP qualification, not supported launch scale. |

Version generators, scenario identities, seed, sizes, digests and ground-truth
selection. The D0 scenario inventory and real holdout are different assets with
different evidence purposes; [testing and coverage](testing-and-coverage.md)
governs deterministic fixtures, and [acceptance](acceptance.md) governs live scoring.

D0-project is the first focused capability profile, not a reduction of retained
full D0/D1 or a population-quality dataset. Declare its actual service allocations,
database connections, codecs, finite context/item/dependency/lineage sizes and
combined serialization/model/worker/cost budgets before admission. Existing initial
resource ceilings remain applicable; missing limits cannot mean unlimited. Full
Qdrant/search/build/physical-purge/repair envelopes are not required for disabled
capabilities; actual expanded propagation and restore precede later enablement.
Existing shared admission, global slots and the per-capture qualification ceiling
remain enforced from the first expensive adapter. H0 resources are measured for
the exact enabled local service set, including separate Temporal persistence.

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

The first synthetic migration harness uses its own protected Unix socket and
fresh task-owned PostgreSQL 17.11 cluster/database, with no TCP listener or
operational credentials. Its separate synthetic migration/writer roles are not
product principals. The writer is neither owner nor superuser and has no
createdb/createrole/replication/BYPASSRLS privilege. This local qualification input
does not claim full H0, managed capacity or native Linux service support.

The cluster has at most 16 connections: at most two writer connections and three
direct migration/control connections, with eleven remaining for other explicit
administration and safety. No per-request or per-tenant pool is introduced.
Ordinary statements and transactions retain 2,000 ms limits; ordinary lock waits
are at most 500 ms and connection establishment at most 1,000 ms. Controlled
migration jobs separately allow 10,000 ms statements, 15,000 ms transactions and
5,000 ms lock waits. These do not change ordinary application budgets.

Each scenario uses a freshly recreated owned database and at most 32 typed rows;
the actual retained cluster must remain below 256 MiB. Record actual storage,
server allocations/configuration and failure behavior. Readiness checks bind the
selected server and roles, not an ambient default database. Cleanup/shutdown has
a 10,000 ms deadline and may address only the exact owned cluster/PID; retain
its evidence. Existing services, data, sessions and host configuration are not
cleanup targets.

The actual uncertainty relay has at most one client/server pair, two forwarding
threads, 1 MiB per frame, 8 MiB total bytes per direction, 4,096 server frames,
2,000 ms I/O waits and a 10,000 ms total deadline. Migration-overlap barriers
require observed real lock waits within 2,000 ms before explicit release. Limits
do not permit sleeps to replace observed ordering. Profile/readiness identities
must be explicit, bounded and hash-bound before native qualification; required
service tests fail unavailable rather than skip when preparation is absent.

The separately selected native service input is at most 2 MiB and binds at most
64 regular configuration/native files, at most 2 GiB each and 8 GiB in total.
This accommodates the actual macOS dyld cache/subcaches without changing the
existing compiler-resource, source or raw-profile budgets. Its exact paths,
tool/file hashes, protected directories and finite lease are checked before and
after qualification. The [service and relay literals](../../tests/integration/postgres/fixtures/migration-service-v1/README.md)
fix structural boundaries and failure cases. A complete server protocol frame,
including its one-byte type and four-byte length, fits the existing 1 MiB cap.
Every I/O timeout also fits the remaining total relay deadline.

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
