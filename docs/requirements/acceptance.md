# Acceptance and qualification

**Authority:** normative requirements. **Status:** accepted targets; no experiment,
runtime workflow or production release is certified by this specification.

This document owns G0–G6, experiment workloads E01–E12, pass/fail thresholds and
completion evidence. [Resource profiles](resource-profiles.md) owns D0/D1/D2,
H0/H1 and admission budgets. [Testing and coverage](testing-and-coverage.md) owns
fixture and measurement integrity. Requirements, implementation, runtime verification
and user acceptance have separate recorded states.

## Product outcome and ordering

An independent user must complete the public workflow with their own permitted
evidence: capture, inspect/correct, produce reviewed expense records and an exact
source-linked report/export; and evidence, review, asset, exact context build,
evaluation, promotion, consumer resolution, feedback and revision. The review
experience must be readable; raw JSON alone is insufficient. The same public engine
capabilities are available through the independently built CLI and documented
interfaces, without undisclosed endpoints or a paid Arboresce entitlement.

Users may explicitly select a third-party inference route requiring credentials
or payment. Disabling optional Arboresce services must preserve core capability;
it does not promise that paid third-party inference becomes free or that a
credential-free local model is qualified. Record required prerequisites and costs.

Essential shared admission and isolation must ship with their first adapters.
Packaged G2 precedes restricted or personal evidence. Actual D0 restoration,
repeated through the packaged local profile, precedes independent-user evaluation.
Approved synthetic and non-sensitive provider tests can advance earlier work but
cannot stand in for those gates. Full D1 E09 remains required for production.

## Gates

| Gate | Required evidence |
| --- | --- |
| G0 — Adopted intent | Current independent engine and CLI source identities, approved self-contained public requirements, preserved rights and notices, no contradictory active ownership, and an auditable requirement-to-owner/verification mapping. |
| G1 — Build and contracts | Exact qualified toolchains/dependencies, independent public builds, complete discriminated operation contracts, supported compatibility pairs, negative semantic fixtures, deterministic generation and every applicable source-coverage gate. |
| G2 — Secure evidence admission | Actual packaged first-owner provisioning, scoped credentials, current authorization/tenant policy, upload sealing, source/destination restrictions, bounded media, shared quotas/cost reservations and audit. These checks pass before restricted evidence is admitted; synthetic processing alone cannot close G2. |
| G3 — Functional MVP | Both full user journeys run through the actual compatible CLI and engine, including report/export and context evaluation/promotion/resolve/feedback. Accepted submission is not processing completion; failed, denied, unavailable and interrupted outcomes remain truthful. |
| G4 — Recovery and integrity | Real-service idempotency/conflict races, worker failure/replay, revocation, export, projection rebuild and packaged D0 restoration pass without duplicate canonical transitions or unauthorized access. Current independently retained post-backup restrictions are reapplied before serving; incomplete proof keeps serving closed. |
| G5 — Quality and utility | Independently scored live holdout, independent users and the required quality, capacity and soak experiments meet their accepted criteria. Recorded providers and invented users do not qualify. |
| G6 — Production v1 | All mandatory E01–E12 evidence, including full D1 E09, exact supported environment, native distributions and Linux AMD64 images, actual authorized artifact signing, independent verification, incident/recovery ownership, retention/key/residency policy and support commitments match measured behavior. |

Content approval attestations do not sign a native binary or image. Implemented
signing/verifier tooling, an unsigned candidate and actual verified signed bytes
are distinct states. Production verification must be independently operated and
separated from privileged signing. [Release policy](../release-policy.md) and
[generated-artifact guidance](../generated-artifacts.md) remain applicable.

## Experiment protocol

All twelve experiments are required; D2 is separately deferred growth qualification.
Use the exact corpus/hardware definitions in [resource profiles](resource-profiles.md),
pin dependencies and record the actual region, RTT, configuration, image, cache
state, clocks, workload generator, dataset/scorer and authorized spend. An emulated
image build is not native performance evidence. Run deterministic checks before
larger qualification, but do not report them as the experiment itself.

Correctness criteria require zero unauthorized access, session leakage, lost
accepted canonical commands or duplicate canonical transitions where specified.
Report latency distributions, not just averages. Cold starts, overload, index
optimization, rejection and recovery are reported separately from nominal warm
results. Preserve failing results and the original commitment when proposing a
reviewed revision; silently reducing workload, omitting cases or changing hardware
cannot turn a failed experiment into a pass.

### E01 — Toolchain, native builds and compatibility

Build the engine and required native macOS/Linux CLI artifacts from clean independent
public checkouts using the exact [dependency contract](dependency-qualification.md).
Exercise 1,000 contract vectors, including an unknown compatible response field,
and two adjacent supported API versions. Verify the actual independently built
client/service pairs and unsupported-pair rejection.

Every required build and test must pass. Supported pairs work; unsupported pairs
fail clearly before incompatible behavior. No unpublished source dependency or
essential undocumented runtime endpoint is permitted. Qualify the supported native
matrix explicitly; an architecture label or cross-compilation alone is insufficient.
Resolve a toolchain or essential dependency failure before dependent feature work.

### E02 — PostgreSQL pooling and tenant isolation

Run D0 first through local PgBouncer transaction mode and then the actual selected
managed PostgreSQL profile. Use a non-owner application role, 64 clients and 10,000
mixed authorize/read/write commands. Enforce the application connection allocation
of at most 80 defined in [resource profiles](resource-profiles.md). Qualify every
production direct/pooled connection mode claimed by the release.

Require zero cross-tenant reads/writes, session bleed or leaked prepared-statement/
transaction state. Under H1, database pool-wait p95 is at most 50 ms. Reconnects,
failed transactions and reused connections must not carry authority. Repair roles,
driver settings, transaction scope or the qualified profile if it fails; unsupported
required isolation blocks activation rather than being bypassed for a provider.

### E03 — Workflow durability and upgrade compatibility

Use actual dedicated workflow persistence for 1,000 jobs and 32 active Activities.
Kill/restart workers at 25% of controlled side-effect boundaries, race cancellation,
restart the server and rehearse one supported schema/worker upgrade. Include
accept-then-crash ambiguity for external effects and replay the relevant histories.

Require no lost accepted canonical command and no duplicate canonical transition.
Every non-expired job reaches a terminal state within ten minutes after required
dependencies recover; histories replay under the qualified version path. This does
not promise exactly-once inference or external delivery. An essential SDK/persistence
failure blocks that profile until fixed or a compatible supported persistence
alternative is independently qualified; do not silently replace orchestration.

### E04 — Query and compare-and-set performance

Use D1 at 100 requests per second from 64 clients for 30 measured minutes after
warmup. The mix is 70% reads, 20% ordinary commands/metadata writes and 10%
conflicting review/promote mutations. Include 50 simultaneous writers competing
on the same revision. Record sampled query plans and actual contention outcomes.

Excluding model and upload duration, p95 is at most 300 ms and p99 at most 800 ms.
Exactly one competing same-revision mutation wins; all others report the defined
conflict. Sampled plans must not contain unbounded scans for bounded operations.
Repair indexes or transaction scope, or explicitly requalify a revised supported
envelope. Increasing pools beyond the connection budget is not a remedy.

### E05 — Acquisition, sealing and parser containment

Combine the D0 malicious-input suite with 100 transfers of 20 MiB, 16 concurrent
transfers and 20% deliberate disconnections. Overwrite staging after verification;
exercise DNS changes, redirects, proxies, malformed metadata and image/audio decode
bombs. Run real acquisition, object storage, finalization and parser boundaries.

Require zero unsafe egress and zero published unverified bytes. Repeating the same
finalization never changes accepted content. Additional API transfer RSS is strictly
below 512 MiB over idle; parser RSS is strictly below 3 GiB. Every failure has an
explicit recoverable state and controlled cleanup. Reject an unsafe source/format
until fixed; do not move arbitrary acquisition into a privileged service to bypass
the failed boundary.

### E06 — Retrieval under restrictive access

Use D1 with 1,000 exact-ground-truth queries, 20 queries per second and at most 32
clients across all D1 authorization selectivities. Request the top 20 using the
shared cap of 200 cumulative pre-deduplication candidates and two total retrieval
rounds defined by [storage and search](storage-and-search.md). Include bounded
hybrid fan-out, stale projection entries and exhausted budgets.

Require zero unauthorized returned or externally exposed passages and recall@20
of at least 0.95 on permitted ground truth. Latency p95 is at most 750 ms and p99
at most two seconds; Qdrant peak RSS is strictly below seven GiB. Report results
by access selectivity, including honest incomplete results. Zero-eligible cases
must yield no unauthorized result and are reported separately from a nonzero
recall denominator. Do not count them as perfect recall. Tune filters/indexes or
qualify a narrower envelope; no unlimited refill or authority transfer is allowed.

### E07 — Projection disorder and rebuild

With D1, reorder and duplicate 100,000 events, delay older revisions, remove the
projection and rebuild while reusing valid retained embeddings. Use an explicit
scoped write pause and the canonical maintenance-barrier protocol.

Require current membership and per-record digests to reconcile with canonical
state, with zero stale authority. D1 rebuild completes within 15 minutes under H1;
the old generation stays safe until the validated switch. All participating writers
must honor the barrier. If the interruption is unacceptable, an online capture
boundary requires a separate reviewed design and qualification; a maximum event ID
is not that design.

### E08 — Authorization, credentials and shared quota races

Run D0 with 10,000 revoke/promote/download races, concurrent credentials, model
budget exhaustion and 1,000 client credential/profile/error cases. Use explicit
barriers around admission and commit, including parallel admission across instances.

Require zero new admissions after committed revocation for the affected scope;
p99 time to new denial is at most one second. Already admitted bounded transfers
follow [security and privacy](security-and-privacy.md). No prompt or confirmation
flag confers privilege. No paid Arboresce entitlement is required, and parallel
admission cannot bypass any cap. Local credential invalidation, cleanup failure and
refresh races must follow their actual contracts. A failed boundary blocks real
evidence use; eventual cache expiry is not a substitute.

### E09 — Restoration and residency

Take a D1 backup, apply 500 subsequent revocations/deletions, and restore database,
blobs, keys/trust and the current independently retained lifecycle journal into an
isolated environment. Reapply later restrictions before serving. Exercise actual
selected storage jurisdiction and restricted retrieval, and stop the projection
host to test the search-outage boundary.

Recovery-point objective is at most 15 minutes and recovery-time objective at most
60 minutes, measured through qualified serving as specified by [recovery](recovery.md).
Require no resurrected serving and safe exact-build behavior during search outage.
The selected location must satisfy the recorded obligations. Incomplete journal or
key/blob proof keeps serving closed even when that misses the time target. Actual
residency and restoration need actual retained resources, not synthetic provider
metadata. An early D0 drill does not close E09.

### E10 — Canonical identity, signatures and money

Run published canonical-JSON vectors and 10,000 generated Unicode, numeric and
key-order cases, 1,000 malformed signatures/keys, and 250 independently reviewed
monetary scenarios. Compare Rust, Python and client canonical bytes and identities.
Exercise unknown/revoked trust and forbidden cross-field/state combinations.

Require exact cross-language agreement and safe rejection of invalid signatures,
unknown keys and invalid runtime states. Every monetary result is exact; no
cross-currency total or fabricated unknown value is allowed. A failing canonicalizer
or trust dependency blocks hashes and signing until repaired or replaced with a
qualified implementation. Do not implement custom cryptography to bypass a failure.

### E11 — Live extraction, transcription and cost

Use a rights-cleared untouched holdout of 300 receipt images and 100 voice notes
lasting 30–300 seconds. Cover declared currencies, legibility, noise and accents.
Before qualification, approve languages/codecs, readability labels, normalization,
scoring/denominators and numeric ASR lexical and consequential-semantic thresholds,
including negation, amount, date and attribution. Those ASR values are required
inputs; this specification does not invent them. Separate development/tuning data
from the final holdout. Tuning on that holdout invalidates its independence.

Readable-image total-and-currency exact accuracy is at least 98%. Count a readable
case that abstains as unsuccessful in that accuracy denominator, and also report
coverage and abstention separately. No extracted result is automatically confirmed.
Image-job p95 is at most 60 seconds after admission; audio-job p95 is at most twice
its decoded duration plus 30 seconds. Enforce the initial per-capture cost ceiling
in [resource profiles](resource-profiles.md), record reservation and realized/uncertain
cost, and obtain actual data-exposure and spending authority before live calls.

Recorded provider responses cannot satisfy live OCR/ASR quality. A failure requires
an explicitly reviewed narrower format/language profile, a changed qualified
processing route, more review or deferred unsupported capability. Missing corpus
rights, labels, ASR thresholds, approved provider policy or funds blocks the affected
result; a plausible transcript and an invented score are not evidence.

### E12 — Independent utility, soak and exit

Three independent technical users each complete their own permitted-evidence
capture/review/report and context-consumer workflow without maintainer-only steps.
Target the first useful outcome within 30 minutes after prerequisites, excluding
dependency downloads. Apply a correction and demonstrate feedback-linked revision
and usable export. Disable optional paid Arboresce services while preserving the
explicitly selected third-party inference prerequisites.

Run a 72-hour H1 soak at 25% of E04's request rate, with recorded bursts, and 100
varied human/agent CLI tasks. After garbage collection and queue drain where
applicable, idle memory growth must be at most 10% above the measured starting
baseline. Record queue/resource recovery rather than concealing growth with process
restarts. Exports must be usable in a clean independent open deployment with
references, attribution and policy intact. Independent users need packaged G2 and
actual D0 recovery first. Invented users, recorded interactions or a download
without a functioning reader cannot close this experiment.

## Evidence, blockers and completion

For each gate/experiment record exact engine and CLI commits/artifact digests,
contracts, schemas, profiles, source/test/fixture identities, locks, tool versions,
configuration excluding secrets, hardware/region/RTT, actual counts, distributions,
commands/exit results, failures/warnings/skips, independent review and known limits.
Keep actual dataset/scorer and signed-byte identity distinguishable from their
intended names. A release result must identify a clean exact source revision.
Record a checkpoint's final commit hash in a subsequent or external record.

Missing actual identity/provider selections, corpus permissions/scoring, native
capacity, retention/residency/journal policy, independent users, operational owners
or approved signing inputs block only their dependent work. Name the missing item,
the affected gate and the evidence required to close it; continue other eligible
work. Tests cannot invent external authority or claim a live service from a fake.
Do not waive security, tenant isolation, corrupt-state or required-gate failures.
Known baseline failures need reproduction and explicit disposition; new failing
required checks block the next dependent checkpoint.

MVP completion does not add deferred application, integration, graph, binding,
universal-media or tax capabilities. [Product and scope](product-and-scope.md)
owns those exclusions. Completing documentation or a requirement mapping is not
completing the product, and passing a source test is not a release or deployment.
