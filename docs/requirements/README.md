# MVP requirement ownership

These documents define the accepted engine MVP and production qualification
target. They do not assert that the runtime, API, libraries, processing service,
distributions or runtime tests are implemented. The current checkout supplies
documentation, licensing and checked parsed-value contract fixtures.
The [rolling plan](../execution/open-cli-mvp-v1-rcld.md) records
the implementation sequence and actual checkpoint evidence.

Each requirement has one normative owner. Links carry shared meaning across
documents; an execution plan, example or test result cannot silently amend it.
Machine-readable operation contracts will define precise fields and variants in
their owning implementation slices. Introduce them with independent positive and
negative fixtures and an explicit association to these requirements. Resolve a
contradiction through a reviewed owning-contract amendment before dependent work.

| Owner | Requirements governed |
| --- | --- |
| [Product and scope](product-and-scope.md) | The two complete workflows, public capability parity, included outcomes and exclusions |
| [Architecture](architecture.md) | Component boundaries, dependency direction and canonical versus derived responsibilities |
| [API](api.md) | Operation contracts, errors, revisions, idempotency, cancellation, exact resolution versus selection and interface compatibility |
| [Common wire contract](common-contract.md) | Exact shared identifier/revision/date encodings, command metadata, problems, operation states and readiness shapes; schemas own structural constraints |
| [Acquisition contract](acquisition-contract.md) | Complete upload and capture wire variants, immutable attachment intent, sealed-byte bindings, original/representation identities and readiness; storage owns actual sealing |
| [Observation contract](observation-contract.md) | Exact representation-bound locators, grounding and source-kind variants, with explicit semantic validation and attribution boundaries |
| [Candidate review contract](candidate-review-contract.md) | Complete candidate content and identity, discriminated review commands, immutable verdict/mapping references and exact revision/current-authority boundaries |
| [Data model](data-model.md) | Tenant-scoped identities, relationships, immutable content, controls and lifecycle meaning |
| [Financial records](financial-records.md) | Complete financial content and identity, exact money and attributable confirmation, closed commands, immutable report selections and lossless JSON/reversible CSV export profiles |
| [Context contract](context-contract.md) | Complete immutable builds and delivery, independent evaluation receipts, authenticated build approval, exact attestation/trust profiles and generation-checked channel transitions |
| [Lifecycle and operation contract](lifecycle-contract.md) | Exact operations/plans, cancellation, explicit capture analysis, subject controls, action/propagation/repair obligations, sealed financial delivery, immutable feedback and internal recognition/retirement/continuity records |
| [Processing](processing.md) | Grounded extraction, supported processing, deterministic orchestration and model gateway behavior |
| [Storage and search](storage-and-search.md) | Persistence invariants, outbox/projections, sealing and bounded retrieval mechanics |
| [Security and privacy](security-and-privacy.md) | Identity, authorization, human assurance, acquisition, egress and lifecycle protection |
| [Resource profiles](resource-profiles.md) | Resource envelopes, workload profiles, shared admission and measurement budgets |
| [Testing and coverage](testing-and-coverage.md) | Independent deterministic oracles, complete suites/process collection and strict source coverage |
| [Dependency qualification](dependency-qualification.md) | Exact toolchain/dependency targets, compatibility and reproducible preparation |
| [Deployment](deployment.md) | Supported operational topology, startup/readiness, upgrade and shutdown obligations |
| [Recovery](recovery.md) | Canonical restoration, current lifecycle restrictions, derived reconstruction and closed-serving behavior |
| [Acceptance](acceptance.md) | G0–G6, E01–E12, real workflows, independent qualification and production evidence |

The existing [release policy](../release-policy.md),
[generated artifact process](../generated-artifacts.md) and
[documentation ownership](../ownership.md) retain their authority. The
independent CLI owns command parsing, terminal presentation, installed process
behavior and its own distributions; compatible public API behavior is governed
here. Neither repository needs another checkout's directory layout or unpublished
context for ordinary contribution and use.

The [lifecycle public library](../../contracts/public/v1/lifecycle.schema.json)
and [internal continuity library](../../contracts/internal/v1/lifecycle-continuity.schema.json)
form one fixture family with explicit public and internal target namespaces.
Internal contracts may import public definitions; public contracts never import
internal ones. Deferred concrete evidence/repair producers must qualify before
their first consuming capability. The
[validation catalogue](../../contracts/validation/README.md) distinguishes these
parsed-value checks from actual services, canonical hashes, storage and restore.

Missing issuer/provider selections, authorized evaluation material, environment
capacity, operational policies or approved signing inputs remain explicit
qualification gates. They do not justify invented defaults, substituted model
responses or omitted tests. Requirements can be adopted before those inputs
exist; the affected runtime and production evidence cannot pass until they do.
