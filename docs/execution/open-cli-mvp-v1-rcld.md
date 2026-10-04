# Engine MVP project-workflow rolling plan

Plan version: 2

Status: owner-approved target; all 68 implementation slices remain
`not_started`. The previous sequence is superseded, not completed. Earlier
adoption/contract evidence keeps its historical scope; it does not qualify new
project/identity wire variants or runtime. Current available behavior remains
separately described by the repository guidance.

## Scope and milestones

The first delivery is a workspace-scoped project workflow through the public
HTTP API and independently built CLI: signup-free Nostr identity, project/Party
context, receipt plus short voice capture, contextual independently revisable
proposals, reviewed project updates, separate exact financial confirmation,
source-linked readback/export and actual packaged restoration.

Workspace uses the Organization tenant plus default Domain; a local connection
profile is not a tenant. Scoped agents use their own keys and explicit grants.
Project acceptance does not confer human financial review. Source/context/default/
inference distinctions, current authorization and exact revision identity persist
through corrections, reanalysis, conflicts, retries and recovery.

P1-P8 delivers G3-project; G2 and actual D0-project recovery precede private
evidence/users and expand before every newly enabled capability. P9 retains
full knowledge build/evaluate/promote/resolve/feedback/search and its broader
recovery. P10 retains full original experiments and production qualification,
native supported artifacts and independent signed-byte verification. Initial
context uses bounded canonical reads; search/build/purge-repair capabilities
not yet qualified are explicitly unavailable. The focused result never claims
old full G3/D0, G5/G6 or released status.

Exact resolution/delivery and usage receipts, complete customer exit archive
production/delivery and optional SMTP notifications each have an engine-owned
checkpoint below. Optional SMTP stays unavailable until its actual profile and
controlled-mail uncertainty tests qualify; it is not inferred from a project note.

## Normative owners

Use [the requirement ownership index](../requirements/README.md). Product behavior
belongs to those requirements, not to this plan. Exact versioned wire/command
variants and literal negative fixtures are frozen before their consumers; later
service/model/codec/assurance choices do not block unrelated synthetic work.

Only one implementation slice is active across a coordinated delivery. Each
slice declares its true prerequisites, owning paths, green behavior and applicable
verification. External public capability receipt/compatibility prerequisites
remain explicit; a missing issuer/profile/corpus is not silently assumed.


## Owning verification and state

Use documented commands from this independent checkout. New command/profile
contracts are introduced with their first consumers. Exact toolchain and
dependency qualification precedes that consumer, not all future capabilities.

```text
cargo fmt --all -- --check
cargo check --locked --workspace --all-targets
cargo test --locked --workspace
cargo clippy --locked --workspace --all-targets -- -D warnings
```

Strict coverage is above 90% for every applicable executable source, package,
repository and supported profile: Rust lines/regions and Python lines/branches.
Actual subprocess/native boundaries and maintained fixture support belong to the
source closure. Required collections have nonzero tests and zero skips. Exact
source, dependencies, fixtures and run/profile identities accompany evidence.
Recorded models prove deterministic integration, not live-provider quality.
Keep implemented, verified, committed/complete and independently accepted states
separate. A verified uncommitted candidate is not a committed checkpoint. No
source commit, publication, signing, deployment or new spend follows from this
plan alone. Use the configured build-output router where enabled.

### E200 — Freeze focused public project and identity contract extensions

Status: `in_progress`. Prerequisites: none. Verify lane: DOC + CONTRACT + RUST + PYTHON + SERVICE + E2E (only applicable changed sources and actual consumers; no waived required lane).

Stage: P1.

Scope: Freeze focused public project and identity contract extensions; implement only the owning behavior described by this slice and its linked requirements. Split a red or oversized increment into independently green consumer checkpoints while preserving every obligation.

Planned paths: `contracts/public/`, `docs/requirements/`

External prerequisite capabilities: Adopt the approved specifications and versioned catalogue. Require independently qualified compatible public contracts/corpus/policy or actual client/service evidence for these consumed capabilities before this slice starts. This declares capability readiness without a private checkout or tooling dependency.

Definition of green: Closed workspace/Party/project/change-item/auth/review-event variants, errors, bounds and compatibility vectors agree with owning requirements; old wire families are not silently repurposed.

Verification: Independent literal contract/fixture oracles, owning language checks/strict coverage and the actual consumer/service/native/recovery lanes stated in the definition of green. Real-input or assurance requirements remain unavailable until supplied and qualified; mocks cannot close them.

Progress: project targets/origins/lineage, association revision/digest/control/provenance, identity/grant and exact financial-review bindings/rendering/proof-reference shapes are represented. Closed registered operation/route/scope/request/result/error inventory, NIP-98 wire-to-normalized admission with finite parser limits and admitted standard revision selection, disclosure-safe resolved/ambiguous/dependency/stale/not-found reference outcomes, and the least-privilege operation-to-action/purpose/resource authority map with the authorized-human alternative and the expanded Party/association/grant action vocabulary are frozen. The current corpus has 411 parsed cases (208 positive, 203 negative); the focused family check and the 26-schema metaschema check pass. Historical results remain labelled: a complete 121-command protocol passed for the earlier 403-case/37-input set (12 direct, 9 relocated, 27 oracle mutations, 6 consumed broken references, 9 consumed missing libraries, 56 off-diagonal pairings and 2 restored guards, stable pre/post hashes and successful tool-identity capture), and a 117-command / 390-case protocol is older still. The current three project-workflow inputs changed under C301-R9, so the changed-closure checks are being re-established while unchanged-closure checks are reusable under the recorded per-check consumed-input/profile closure map; no stale packet is promoted to the current candidate. This is parsed-value evidence only; it does not qualify semantic joins, authority, canonical effects, actual human assurance or any consumer runtime. Independent review and actual owning-consumer joins remain required before consumers. Status remains in_progress.

### E201 — Bootstrap measured Rust types on the accepted toolchain

Status: `not_started`. Prerequisites: E200. Verify lane: DOC + CONTRACT + RUST + PYTHON + SERVICE + E2E (only applicable changed sources and actual consumers; no waived required lane).

Stage: P1.

Scope: Bootstrap measured Rust types on the accepted toolchain; implement only the owning behavior described by this slice and its linked requirements. Split a red or oversized increment into independently green consumer checkpoints while preserving every obligation.

Planned paths: `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `crates/arboresce-domain/`

External prerequisite capabilities: Approve the focused synthetic corpus and independent oracles. Require independently qualified compatible public contracts/corpus/policy or actual client/service evidence for these consumed capabilities before this slice starts. This declares capability readiness without a private checkout or tooling dependency.

Definition of green: Exact Rust 1.98.0 builds useful opaque identities; independent formats/roundtrips and strict source/region coverage pass; the binding proof is not treated as an API runtime.

Verification: Independent literal contract/fixture oracles, owning language checks/strict coverage and the actual consumer/service/native/recovery lanes stated in the definition of green. Real-input or assurance requirements remain unavailable until supplied and qualified; mocks cannot close them.

### E202 — Introduce repository-owned verification and dependency guards

Status: `not_started`. Prerequisites: E201. Verify lane: DOC + CONTRACT + RUST + PYTHON + SERVICE + E2E (only applicable changed sources and actual consumers; no waived required lane).

Stage: P1.

Scope: Introduce repository-owned verification and dependency guards; implement only the owning behavior described by this slice and its linked requirements. Split a red or oversized increment into independently green consumer checkpoints while preserving every obligation.

Planned paths: `tools/xtask/`, `tests/architecture/`, `docs/requirements/testing-and-coverage.md`

Definition of green: Actual formatting/check/test/Clippy/coverage commands cover all selected packages/processes; forbidden domain imports and permitted deterministic workflow boundaries are independently tested.

Verification: Independent literal contract/fixture oracles, owning language checks/strict coverage and the actual consumer/service/native/recovery lanes stated in the definition of green. Real-input or assurance requirements remain unavailable until supplied and qualified; mocks cannot close them.

### E203 — Implement independent revisions, clocks, dates and locators

Status: `not_started`. Prerequisites: E201. Verify lane: DOC + CONTRACT + RUST + PYTHON + SERVICE + E2E (only applicable changed sources and actual consumers; no waived required lane).

Stage: P1.

Scope: Implement independent revisions, clocks, dates and locators; implement only the owning behavior described by this slice and its linked requirements. Split a red or oversized increment into independently green consumer checkpoints while preserving every obligation.

Planned paths: `crates/arboresce-domain/`, `tests/contract/`

Definition of green: Content/control revisions, overflow, observed versus recorded time, timezone ambiguity and typed pixel/audio locators pass independent boundary vectors.

Verification: Independent literal contract/fixture oracles, owning language checks/strict coverage and the actual consumer/service/native/recovery lanes stated in the definition of green. Real-input or assurance requirements remain unavailable until supplied and qualified; mocks cannot close them.

### E204 — Create focused application ports and migration harness

Status: `not_started`. Prerequisites: E203. Verify lane: DOC + CONTRACT + RUST + PYTHON + SERVICE + E2E (only applicable changed sources and actual consumers; no waived required lane).

Stage: P1.

Scope: Create focused application ports and migration harness; implement only the owning behavior described by this slice and its linked requirements. Split a red or oversized increment into independently green consumer checkpoints while preserving every obligation.

Planned paths: `crates/arboresce-application/`, `crates/arboresce-postgres/`, `tests/integration/postgres/`

Admission boundary: The useful record is an isolated synthetic migration-harness write. Admitted canonical product and bootstrap writers remain disabled until independently retained lifecycle continuity is qualified. Schema preparation cannot admit user effects.

Definition of green: A real isolated PostgreSQL migration persists a useful typed record; compatible migration/retry/rollback limits and transaction outcome boundaries pass without requiring later build/search tables.

Verification: Independent literal contract/fixture oracles, owning language checks/strict coverage and the actual consumer/service/native/recovery lanes stated in the definition of green. Real-input or assurance requirements remain unavailable until supplied and qualified; mocks cannot close them.

### E205 — Enforce tenant roles, RLS and first-adapter admission

Status: `not_started`. Prerequisites: E204. Verify lane: DOC + CONTRACT + RUST + PYTHON + SERVICE + E2E (only applicable changed sources and actual consumers; no waived required lane).

Stage: P1.

Scope: Enforce tenant roles, RLS and first-adapter admission; implement only the owning behavior described by this slice and its linked requirements. Split a red or oversized increment into independently green consumer checkpoints while preserving every obligation.

Planned paths: `crates/arboresce-postgres/`, `crates/arboresce-application/src/policy/`, `tests/security/`

Definition of green: Actual non-owner roles and concurrent current-authority/admission fences prevent tenant bleed and quota bypass; bootstrap namespace and all relationship references are scoped.

Verification: Independent literal contract/fixture oracles, owning language checks/strict coverage and the actual consumer/service/native/recovery lanes stated in the definition of green. Real-input or assurance requirements remain unavailable until supplied and qualified; mocks cannot close them.

### E206 — Implement independent lifecycle continuity before first writers

Status: `not_started`. Prerequisites: E205. Verify lane: DOC + CONTRACT + RUST + PYTHON + SERVICE + E2E (only applicable changed sources and actual consumers; no waived required lane).

Stage: P1.

Scope: Implement independent lifecycle continuity before first writers; implement only the owning behavior described by this slice and its linked requirements. Split a red or oversized increment into independently green consumer checkpoints while preserving every obligation.

Planned paths: `crates/arboresce-postgres/src/lifecycle/`, `crates/arboresce-application/src/lifecycle/`, `tests/integration/recovery/`

Definition of green: Existing six-family prepared/committed/abort continuity, inventory interpretation, admission recognition and scope-retirement barriers qualify against independently retained authentic bytes; unknown tail/cutoff keeps affected serving closed.

Verification: Independent literal contract/fixture oracles, owning language checks/strict coverage and the actual consumer/service/native/recovery lanes stated in the definition of green. Real-input or assurance requirements remain unavailable until supplied and qualified; mocks cannot close them.

### E207 — Persist idempotent commands, audit and durable outbox

Status: `not_started`. Prerequisites: E206. Verify lane: DOC + CONTRACT + RUST + PYTHON + SERVICE + E2E (only applicable changed sources and actual consumers; no waived required lane).

Stage: P1.

Scope: Persist idempotent commands, audit and durable outbox; implement only the owning behavior described by this slice and its linked requirements. Split a red or oversized increment into independently green consumer checkpoints while preserving every obligation.

Planned paths: `crates/arboresce-postgres/`, `crates/arboresce-application/src/commands/`, `tests/integration/concurrency/`

Definition of green: Identical retries return one currently authorized result, changed input conflicts, durable audit/result/outbox bind atomically and real lease/crash races retain uncertainty without duplicate canonical transitions.

Verification: Independent literal contract/fixture oracles, owning language checks/strict coverage and the actual consumer/service/native/recovery lanes stated in the definition of green. Real-input or assurance requirements remain unavailable until supplied and qualified; mocks cannot close them.

### E208 — Start the public HTTP service and capability envelope

Status: `not_started`. Prerequisites: E207. Verify lane: DOC + CONTRACT + RUST + PYTHON + SERVICE + E2E (only applicable changed sources and actual consumers; no waived required lane).

Stage: P1.

Scope: Start the public HTTP service and capability envelope; implement only the owning behavior described by this slice and its linked requirements. Split a red or oversized increment into independently green consumer checkpoints while preserving every obligation.

Planned paths: `crates/arboresce-api/`, `tests/integration/api/`

Definition of green: Actual selected local API starts/stops, reports readiness/capabilities and discriminated errors, bounds requests, hides foreign identities and negotiates supported client contracts.

Verification: Independent literal contract/fixture oracles, owning language checks/strict coverage and the actual consumer/service/native/recovery lanes stated in the definition of green. Real-input or assurance requirements remain unavailable until supplied and qualified; mocks cannot close them.

### E209 — Qualify Nostr request authentication and replay profile

Status: `not_started`. Prerequisites: E208. Verify lane: DOC + CONTRACT + RUST + PYTHON + SERVICE + E2E (only applicable changed sources and actual consumers; no waived required lane).

Stage: P2.

Scope: Qualify Nostr request authentication and replay profile; implement only the owning behavior described by this slice and its linked requirements. Split a red or oversized increment into independently green consumer checkpoints while preserving every obligation.

Planned paths: `crates/arboresce-api/src/auth/`, `crates/arboresce-application/src/auth/`, `tests/contract/identity/`

Definition of green: Exact pinned NIP-98 URL/method/body/time/signature vectors, malformed events, replay and idempotency interaction pass; Nostr key control never supplies human assurance or replaces Ed25519 attestations.

Verification: Independent literal contract/fixture oracles, owning language checks/strict coverage and the actual consumer/service/native/recovery lanes stated in the definition of green. Real-input or assurance requirements remain unavailable until supplied and qualified; mocks cannot close them.

### E210 — Provision the first owner through the protected operator boundary

Status: `not_started`. Prerequisites: E209, E206. Verify lane: DOC + CONTRACT + RUST + PYTHON + SERVICE + E2E (only applicable changed sources and actual consumers; no waived required lane).

Stage: P2.

Scope: Provision the first owner through the protected operator boundary; implement only the owning behavior described by this slice and its linked requirements. Split a red or oversized increment into independently green consumer checkpoints while preserving every obligation.

Planned paths: `crates/arboresce-admin/`, `crates/arboresce-application/src/identity/`, `tests/security/bootstrap/`

Definition of green: Actual configured operator/namespace/policy/stream provisioning admits one owner under races, securely delivers credentials and cannot overwrite or resurrect recognized/retired identities.

Verification: Independent literal contract/fixture oracles, owning language checks/strict coverage and the actual consumer/service/native/recovery lanes stated in the definition of green. Real-input or assurance requirements remain unavailable until supplied and qualified; mocks cannot close them.

### E211 — Provision workspaces, default domains and memberships

Status: `not_started`. Prerequisites: E210. Verify lane: DOC + CONTRACT + RUST + PYTHON + SERVICE + E2E (only applicable changed sources and actual consumers; no waived required lane).

Stage: P2.

Scope: Provision workspaces, default domains and memberships; implement only the owning behavior described by this slice and its linked requirements. Split a red or oversized increment into independently green consumer checkpoints while preserving every obligation.

Planned paths: `crates/arboresce-application/src/workspaces/`, `crates/arboresce-postgres/src/workspaces/`, `crates/arboresce-api/src/routes/`

Definition of green: A workspace uses the Organization tenant plus default Domain; creation/read/membership/last-owner constraints work through real API transactions, with explicit bootstrap exceptions and exactly one workspace per content command.

Verification: Independent literal contract/fixture oracles, owning language checks/strict coverage and the actual consumer/service/native/recovery lanes stated in the definition of green. Real-input or assurance requirements remain unavailable until supplied and qualified; mocks cannot close them.

### E212 — Enforce scoped agent grants and current authorization

Status: `not_started`. Prerequisites: E211. Verify lane: DOC + CONTRACT + RUST + PYTHON + SERVICE + E2E (only applicable changed sources and actual consumers; no waived required lane).

Stage: P2.

Scope: Enforce scoped agent grants and current authorization; implement only the owning behavior described by this slice and its linked requirements. Split a red or oversized increment into independently green consumer checkpoints while preserving every obligation.

Planned paths: `crates/arboresce-application/src/auth/`, `crates/arboresce-api/src/auth/`, `tests/security/delegation/`

Definition of green: Own-key agent grants bind action/resource/workspace/purpose/lifetime; expiry/revocation/chain-switch/regrant races deny new effects and replay disclosure; no owner-key lending or service self-promotion.

Verification: Independent literal contract/fixture oracles, owning language checks/strict coverage and the actual consumer/service/native/recovery lanes stated in the definition of green. Real-input or assurance requirements remain unavailable until supplied and qualified; mocks cannot close them.

### E213 — Prove CLI identity and two-workspace isolation

Status: `not_started`. Prerequisites: none. Verify lane: DOC + CONTRACT + RUST + PYTHON + SERVICE + E2E (only applicable changed sources and actual consumers; no waived required lane).

Stage: P2.

Scope: Prove CLI identity and two-workspace isolation; implement only the owning behavior described by this slice and its linked requirements. Split a red or oversized increment into independently green consumer checkpoints while preserving every obligation.

Planned paths: `tests/e2e/identity/`, `tests/security/`

External prerequisite capabilities: Expose identity, workspace and delegated project actions. Require independently qualified compatible public contracts/corpus/policy or actual client/service evidence for these consumed capabilities before this slice starts. This declares capability readiness without a private checkout or tooling dependency.

Definition of green: Actual CLI/API cases cover outsiders and an actor authorized in both workspaces; forged references, replay, revocation and key changes never cross scope or broaden authority.

Verification: Independent literal contract/fixture oracles, owning language checks/strict coverage and the actual consumer/service/native/recovery lanes stated in the definition of green. Real-input or assurance requirements remain unavailable until supplied and qualified; mocks cannot close them.

### E214 — Define typed Party and project changes

Status: `not_started`. Prerequisites: E203, E200. Verify lane: DOC + CONTRACT + RUST + PYTHON + SERVICE + E2E (only applicable changed sources and actual consumers; no waived required lane).

Stage: P3.

Scope: Define typed Party and project changes; implement only the owning behavior described by this slice and its linked requirements. Split a red or oversized increment into independently green consumer checkpoints while preserving every obligation.

Planned paths: `crates/arboresce-domain/src/project/`, `contracts/public/`, `tests/contract/project/`

Definition of green: Stable external Party/alias/principal distinctions and project participants/decision/work/assignment/date/status/fact create-revise-associate variants have exact bounds, versions, provenance and literal negative vectors.

Verification: Independent literal contract/fixture oracles, owning language checks/strict coverage and the actual consumer/service/native/recovery lanes stated in the definition of green. Real-input or assurance requirements remain unavailable until supplied and qualified; mocks cannot close them.

### E215 — Persist authorized project records and revisions

Status: `not_started`. Prerequisites: E214, E212, E207. Verify lane: DOC + CONTRACT + RUST + PYTHON + SERVICE + E2E (only applicable changed sources and actual consumers; no waived required lane).

Stage: P3.

Scope: Persist authorized project records and revisions; implement only the owning behavior described by this slice and its linked requirements. Split a red or oversized increment into independently green consumer checkpoints while preserving every obligation.

Planned paths: `crates/arboresce-postgres/src/project/`, `crates/arboresce-application/src/project/`, `crates/arboresce-api/src/routes/project.rs`

Definition of green: Real CLI-addressable project/Party CRUD records immutable attributable revisions; current workspace/reference permissions and target CAS are transactional, attendance/assignment never grants authority.

Verification: Independent literal contract/fixture oracles, owning language checks/strict coverage and the actual consumer/service/native/recovery lanes stated in the definition of green. Real-input or assurance requirements remain unavailable until supplied and qualified; mocks cannot close them.

### E216 — Resolve bounded exact prior project context

Status: `not_started`. Prerequisites: E215. Verify lane: DOC + CONTRACT + RUST + PYTHON + SERVICE + E2E (only applicable changed sources and actual consumers; no waived required lane).

Stage: P3.

Scope: Resolve bounded exact prior project context; implement only the owning behavior described by this slice and its linked requirements. Split a red or oversized increment into independently green consumer checkpoints while preserving every obligation.

Planned paths: `crates/arboresce-application/src/context/`, `tests/integration/project/`

Definition of green: Explicit project and alias resolution returns only current-authorized exact versions under bounded reads; omitted established facts are attributable, ambiguous names/dates remain alternatives or unknown, and unrelated changes do not invalidate every target.

Verification: Independent literal contract/fixture oracles, owning language checks/strict coverage and the actual consumer/service/native/recovery lanes stated in the definition of green. Real-input or assurance requirements remain unavailable until supplied and qualified; mocks cannot close them.

### E217 — Qualify project and context fixtures against real persistence

Status: `not_started`. Prerequisites: none. Verify lane: DOC + CONTRACT + RUST + PYTHON + SERVICE + E2E (only applicable changed sources and actual consumers; no waived required lane).

Stage: P3.

Scope: Qualify project and context fixtures against real persistence; implement only the owning behavior described by this slice and its linked requirements. Split a red or oversized increment into independently green consumer checkpoints while preserving every obligation.

Planned paths: `tests/fixtures/project/`, `tests/integration/project/`, `tests/e2e/project/`

External prerequisite capabilities: Expose project context creation, revision and readback. Require independently qualified compatible public contracts/corpus/policy or actual client/service evidence for these consumed capabilities before this slice starts. This declares capability readiness without a private checkout or tooling dependency.

Definition of green: Independent seed/read/revision/CAS/alias/clock/negative fixtures prove contextual blanks and workspace disclosure through real CLI/API/PG, with expected answers outside runtime inputs.

Verification: Independent literal contract/fixture oracles, owning language checks/strict coverage and the actual consumer/service/native/recovery lanes stated in the definition of green. Real-input or assurance requirements remain unavailable until supplied and qualified; mocks cannot close them.

### E218 — Freeze typed intelligence and durable stage messages

Status: `not_started`. Prerequisites: E200. Verify lane: DOC + CONTRACT + RUST + PYTHON + SERVICE + E2E (only applicable changed sources and actual consumers; no waived required lane).

Stage: P4.

Scope: Freeze typed intelligence and durable stage messages; implement only the owning behavior described by this slice and its linked requirements. Split a red or oversized increment into independently green consumer checkpoints while preserving every obligation.

Planned paths: `contracts/internal/proto/`, `contracts/internal/`, `tests/contract/intelligence/`

Definition of green: Versioned source/context/change-item/profile/stage/exposure/cost messages are bounded and credential-free; target/version/dependency/locator errors fail before canonical effects.

Verification: Independent literal contract/fixture oracles, owning language checks/strict coverage and the actual consumer/service/native/recovery lanes stated in the definition of green. Real-input or assurance requirements remain unavailable until supplied and qualified; mocks cannot close them.

### E219 — Bootstrap measured Python processing and generated transport

Status: `not_started`. Prerequisites: E218, E202. Verify lane: DOC + CONTRACT + RUST + PYTHON + SERVICE + E2E (only applicable changed sources and actual consumers; no waived required lane).

Stage: P4.

Scope: Bootstrap measured Python processing and generated transport; implement only the owning behavior described by this slice and its linked requirements. Split a red or oversized increment into independently green consumer checkpoints while preserving every obligation.

Planned paths: `python/`, `contracts/internal/proto/`, `tests/contract/`

Definition of green: Independently pinned Python environment and generated DTO drift checks pass; maintained processors/helpers and actual subprocess boundaries meet strict line/branch coverage without DB write credentials.

Verification: Independent literal contract/fixture oracles, owning language checks/strict coverage and the actual consumer/service/native/recovery lanes stated in the definition of green. Real-input or assurance requirements remain unavailable until supplied and qualified; mocks cannot close them.

### E220 — Persist operations, stage ledgers, leases and cancellation

Status: `not_started`. Prerequisites: E207, E218. Verify lane: DOC + CONTRACT + RUST + PYTHON + SERVICE + E2E (only applicable changed sources and actual consumers; no waived required lane).

Stage: P4.

Scope: Persist operations, stage ledgers, leases and cancellation; implement only the owning behavior described by this slice and its linked requirements. Split a red or oversized increment into independently green consumer checkpoints while preserving every obligation.

Planned paths: `crates/arboresce-application/src/operations/`, `crates/arboresce-postgres/src/operations/`, `tests/integration/operations/`

Definition of green: Real immutable plans/attempts/results/effects/exposures bind origin and current authority; cancellation preserves committed/uncertain effects, concurrent leases cannot double-commit.

Verification: Independent literal contract/fixture oracles, owning language checks/strict coverage and the actual consumer/service/native/recovery lanes stated in the definition of green. Real-input or assurance requirements remain unavailable until supplied and qualified; mocks cannot close them.

### E221 — Implement bounded object storage and sealed identities

Status: `not_started`. Prerequisites: E220, E208. Verify lane: DOC + CONTRACT + RUST + PYTHON + SERVICE + E2E (only applicable changed sources and actual consumers; no waived required lane).

Stage: P4.

Scope: Implement bounded object storage and sealed identities; implement only the owning behavior described by this slice and its linked requirements. Split a red or oversized increment into independently green consumer checkpoints while preserving every obligation.

Planned paths: `crates/arboresce-blob-s3/`, `crates/arboresce-application/src/blob/`, `tests/integration/storage/`

Definition of green: Actual selected storage profile isolates staging/sealed roles and tenants, checks digest/length, denies overwrite substitution, bounds streams and retains uncertain storage outcomes.

Verification: Independent literal contract/fixture oracles, owning language checks/strict coverage and the actual consumer/service/native/recovery lanes stated in the definition of green. Real-input or assurance requirements remain unavailable until supplied and qualified; mocks cannot close them.

### E222 — Implement captures, attachment roles and fenced sealing

Status: `not_started`. Prerequisites: E221, E212. Verify lane: DOC + CONTRACT + RUST + PYTHON + SERVICE + E2E (only applicable changed sources and actual consumers; no waived required lane).

Stage: P4.

Scope: Implement captures, attachment roles and fenced sealing; implement only the owning behavior described by this slice and its linked requirements. Split a red or oversized increment into independently green consumer checkpoints while preserving every obligation.

Planned paths: `crates/arboresce-application/src/captures/`, `crates/arboresce-api/src/routes/captures.rs`, `tests/integration/acquisition/`

Definition of green: Required/optional ordered sources and capture revisions remain exact; staged bytes seal once under final fences, lost responses/retries/cancellation do not create duplicate sources or silently drop attachments.

Verification: Independent literal contract/fixture oracles, owning language checks/strict coverage and the actual consumer/service/native/recovery lanes stated in the definition of green. Real-input or assurance requirements remain unavailable until supplied and qualified; mocks cannot close them.

### E223 — Qualify the selected Temporal runtime and separate persistence

Status: `not_started`. Prerequisites: E201. Verify lane: DOC + CONTRACT + RUST + PYTHON + SERVICE + E2E (only applicable changed sources and actual consumers; no waived required lane).

Stage: P4.

Scope: Qualify the selected Temporal runtime and separate persistence; implement only the owning behavior described by this slice and its linked requirements. Split a red or oversized increment into independently green consumer checkpoints while preserving every obligation.

Planned paths: `docker/temporal/`, `crates/arboresce-workflows/`, `tests/integration/temporal/`

Definition of green: Exact SDK 1.0.0/Core-compatible0.9.0/server/admin/schema pins and actual dedicated PostgreSQL profile start/replay/restart correctly; managed application DB qualification and later search are not prerequisites.

Verification: Independent literal contract/fixture oracles, owning language checks/strict coverage and the actual consumer/service/native/recovery lanes stated in the definition of green. Real-input or assurance requirements remain unavailable until supplied and qualified; mocks cannot close them.

### E224 — Wire deterministic workflows, workers and outbox dispatch

Status: `not_started`. Prerequisites: E223, E220, E218. Verify lane: DOC + CONTRACT + RUST + PYTHON + SERVICE + E2E (only applicable changed sources and actual consumers; no waived required lane).

Stage: P4.

Scope: Wire deterministic workflows, workers and outbox dispatch; implement only the owning behavior described by this slice and its linked requirements. Split a red or oversized increment into independently green consumer checkpoints while preserving every obligation.

Planned paths: `crates/arboresce-workflows/`, `crates/arboresce-activities/`, `crates/arboresce-worker/`, `tests/integration/temporal/`

Definition of green: Actual dispatch/workers use stable operation/workflow IDs and deterministic histories; crash/retry/lease/cancel boundaries retain results and prevent duplicate canonical transitions.

Verification: Independent literal contract/fixture oracles, owning language checks/strict coverage and the actual consumer/service/native/recovery lanes stated in the definition of green. Real-input or assurance requirements remain unavailable until supplied and qualified; mocks cannot close them.

### E225 — Implement deterministic recorded processing providers

Status: `not_started`. Prerequisites: E219. Verify lane: DOC + CONTRACT + RUST + PYTHON + SERVICE + E2E (only applicable changed sources and actual consumers; no waived required lane).

Stage: P4.

Scope: Implement deterministic recorded processing providers; implement only the owning behavior described by this slice and its linked requirements. Split a red or oversized increment into independently green consumer checkpoints while preserving every obligation.

Planned paths: `python/src/`, `python/tests/`, `tests/fixtures/project/`

External prerequisite capabilities: Approve the focused synthetic corpus and independent oracles. Require independently qualified compatible public contracts/corpus/policy or actual client/service evidence for these consumed capabilities before this slice starts. This declares capability readiness without a private checkout or tooling dependency.

Definition of green: Pinned recording inputs/outputs and independent schema/locator/context expectations produce multiple typed changes; recording identity is explicit and never reported as live model qualification.

Verification: Independent literal contract/fixture oracles, owning language checks/strict coverage and the actual consumer/service/native/recovery lanes stated in the definition of green. Real-input or assurance requirements remain unavailable until supplied and qualified; mocks cannot close them.

### E226 — Integrate the recorded capture-to-proposal workflow

Status: `not_started`. Prerequisites: E224, E225, E222, E216. Verify lane: DOC + CONTRACT + RUST + PYTHON + SERVICE + E2E (only applicable changed sources and actual consumers; no waived required lane).

Stage: P4.

Scope: Integrate the recorded capture-to-proposal workflow; implement only the owning behavior described by this slice and its linked requirements. Split a red or oversized increment into independently green consumer checkpoints while preserving every obligation.

Planned paths: `crates/arboresce-activities/`, `python/src/`, `tests/integration/processing/`

Definition of green: Real API/PG/object/Temporal/gRPC pipeline produces bounded grounded proposals for the exact capture/context; target mismatches, partial source failures and worker interruption remain truthful.

Verification: Independent literal contract/fixture oracles, owning language checks/strict coverage and the actual consumer/service/native/recovery lanes stated in the definition of green. Real-input or assurance requirements remain unavailable until supplied and qualified; mocks cannot close them.

### E227 — Enforce first-provider and media resource admission

Status: `not_started`. Prerequisites: E226, E205. Verify lane: DOC + CONTRACT + RUST + PYTHON + SERVICE + E2E (only applicable changed sources and actual consumers; no waived required lane).

Stage: P4.

Scope: Enforce first-provider and media resource admission; implement only the owning behavior described by this slice and its linked requirements. Split a red or oversized increment into independently green consumer checkpoints while preserving every obligation.

Planned paths: `crates/arboresce-application/src/admission/`, `python/src/`, `tests/security/resources/`

Definition of green: Current workspace/provider/exposure grants and shared slot/cost/bytes/time/process limits reserve before effects; concurrent/slow/malformed inputs fail without unbounded work or private-data egress.

Verification: Independent literal contract/fixture oracles, owning language checks/strict coverage and the actual consumer/service/native/recovery lanes stated in the definition of green. Real-input or assurance requirements remain unavailable until supplied and qualified; mocks cannot close them.

### E228 — Apply current source restrictions and holds

Status: `not_started`. Prerequisites: E212, E221, E206. Verify lane: DOC + CONTRACT + RUST + PYTHON + SERVICE + E2E (only applicable changed sources and actual consumers; no waived required lane).

Stage: P4.

Scope: Apply current source restrictions and holds; implement only the owning behavior described by this slice and its linked requirements. Split a red or oversized increment into independently green consumer checkpoints while preserving every obligation.

Planned paths: `crates/arboresce-application/src/lifecycle/`, `tests/security/lifecycle/`

Definition of green: Revoke/restrict/hold effects deny new reads/processing/exports and preserve independent continuity; disabled physical purge/repair cannot execute or fabricate confirmation, held content never expands access.

Verification: Independent literal contract/fixture oracles, owning language checks/strict coverage and the actual consumer/service/native/recovery lanes stated in the definition of green. Real-input or assurance requirements remain unavailable until supplied and qualified; mocks cannot close them.

### E229 — Package the minimal local profile and real restore tooling

Status: `not_started`. Prerequisites: E228, E227, E223. Verify lane: DOC + CONTRACT + RUST + PYTHON + SERVICE + E2E (only applicable changed sources and actual consumers; no waived required lane).

Stage: P4.

Scope: Package the minimal local profile and real restore tooling; implement only the owning behavior described by this slice and its linked requirements. Split a red or oversized increment into independently green consumer checkpoints while preserving every obligation.

Planned paths: `docker/`, `crates/arboresce-admin/`, `docs/runbooks/`, `tests/integration/recovery/`

External prerequisite capabilities: Expose operation status, polling and cancellation. Require independently qualified compatible public contracts/corpus/policy or actual client/service evidence for these consumed capabilities before this slice starts. This declares capability readiness without a private checkout or tooling dependency.

Definition of green: Actual declared API/worker/Python/CLI/service profile starts securely and restores all enabled canonical/derived bytes, identities/keys/trust/operations and independent journal; missing/corrupt/unknown proof keeps serving closed.

Verification: Independent literal contract/fixture oracles, owning language checks/strict coverage and the actual consumer/service/native/recovery lanes stated in the definition of green. Real-input or assurance requirements remain unavailable until supplied and qualified; mocks cannot close them.

### E230 — Qualify bounded image decoding and orientation provenance

Status: `not_started`. Prerequisites: E219, E227. Verify lane: DOC + CONTRACT + RUST + PYTHON + SERVICE + E2E (only applicable changed sources and actual consumers; no waived required lane).

Stage: P5.

Scope: Qualify bounded image decoding and orientation provenance; implement only the owning behavior described by this slice and its linked requirements. Split a red or oversized increment into independently green consumer checkpoints while preserving every obligation.

Planned paths: `python/src/`, `python/tests/`, `tests/fixtures/media/`

Definition of green: Actual JPEG/PNG profile bounds pixel/byte/CPU/memory/process behavior and preserves normalized-to-original mappings; HEIC is unsupported unless separately qualified or explicit preserved-original derivative conversion is reviewed.

Verification: Independent literal contract/fixture oracles, owning language checks/strict coverage and the actual consumer/service/native/recovery lanes stated in the definition of green. Real-input or assurance requirements remain unavailable until supplied and qualified; mocks cannot close them.

### E231 — Qualify actual audio formats and time mappings

Status: `not_started`. Prerequisites: E219, E227. Verify lane: DOC + CONTRACT + RUST + PYTHON + SERVICE + E2E (only applicable changed sources and actual consumers; no waived required lane).

Stage: P5.

Scope: Qualify actual audio formats and time mappings; implement only the owning behavior described by this slice and its linked requirements. Split a red or oversized increment into independently green consumer checkpoints while preserving every obligation.

Planned paths: `python/src/`, `python/tests/`, `tests/fixtures/media/`

Definition of green: Actual selected iPhone WAV/M4A codec profile enforces duration/size/resource limits and sample/time provenance; unsupported/corrupt inputs and incomplete decoding are typed outcomes.

Verification: Independent literal contract/fixture oracles, owning language checks/strict coverage and the actual consumer/service/native/recovery lanes stated in the definition of green. Real-input or assurance requirements remain unavailable until supplied and qualified; mocks cannot close them.

### E232 — Implement and qualify actual receipt extraction

Status: `not_started`. Prerequisites: E230. Verify lane: DOC + CONTRACT + RUST + PYTHON + SERVICE + E2E (only applicable changed sources and actual consumers; no waived required lane).

Stage: P5.

Scope: Implement and qualify actual receipt extraction; implement only the owning behavior described by this slice and its linked requirements. Split a red or oversized increment into independently green consumer checkpoints while preserving every obligation.

Planned paths: `python/src/`, `python/tests/`, `tests/integration/media/`

External prerequisite capabilities: Freeze real media and semantic provider profiles. Require independently qualified compatible public contracts/corpus/policy or actual client/service evidence for these consumed capabilities before this slice starts. This declares capability readiness without a private checkout or tooling dependency.

Definition of green: Actual detector/recognizer/dictionary expected hashes validate; OCR readiness is independent, truncation/skips are warnings, locators are source-valid and raw confidence never establishes financial correctness.

Verification: Independent literal contract/fixture oracles, owning language checks/strict coverage and the actual consumer/service/native/recovery lanes stated in the definition of green. Real-input or assurance requirements remain unavailable until supplied and qualified; mocks cannot close them.

### E233 — Implement and qualify actual transcription

Status: `not_started`. Prerequisites: E231. Verify lane: DOC + CONTRACT + RUST + PYTHON + SERVICE + E2E (only applicable changed sources and actual consumers; no waived required lane).

Stage: P5.

Scope: Implement and qualify actual transcription; implement only the owning behavior described by this slice and its linked requirements. Split a red or oversized increment into independently green consumer checkpoints while preserving every obligation.

Planned paths: `python/src/`, `python/tests/`, `tests/integration/media/`

External prerequisite capabilities: Freeze real media and semantic provider profiles. Require independently qualified compatible public contracts/corpus/policy or actual client/service evidence for these consumed capabilities before this slice starts. This declares capability readiness without a private checkout or tooling dependency.

Definition of green: Actual ASR route retains exact audio/transcript/profile identity and time maps; supplied Whisper output is an explicit recording distinct from human truth, live word/content scoring and latency.

Verification: Independent literal contract/fixture oracles, owning language checks/strict coverage and the actual consumer/service/native/recovery lanes stated in the definition of green. Real-input or assurance requirements remain unavailable until supplied and qualified; mocks cannot close them.

### E234 — Generate contextual project interpretations

Status: `not_started`. Prerequisites: E216, E232, E233. Verify lane: DOC + CONTRACT + RUST + PYTHON + SERVICE + E2E (only applicable changed sources and actual consumers; no waived required lane).

Stage: P5.

Scope: Generate contextual project interpretations; implement only the owning behavior described by this slice and its linked requirements. Split a red or oversized increment into independently green consumer checkpoints while preserving every obligation.

Planned paths: `python/src/`, `python/tests/`, `tests/fixtures/project/`

External prerequisite capabilities: Freeze real media and semantic provider profiles. Require independently qualified compatible public contracts/corpus/policy or actual client/service evidence for these consumed capabilities before this slice starts. This declares capability readiness without a private checkout or tooling dependency.

Definition of green: Actual separately versioned semantic route uses authorized prior context, labels source/context/default/inference, preserves alternatives/unknowns and abstains from invented monetary facts, assent, attendance or completion.

Verification: Independent literal contract/fixture oracles, owning language checks/strict coverage and the actual consumer/service/native/recovery lanes stated in the definition of green. Real-input or assurance requirements remain unavailable until supplied and qualified; mocks cannot close them.

### E235 — Validate multimodal disagreement and typed change dependencies

Status: `not_started`. Prerequisites: E234, E214. Verify lane: DOC + CONTRACT + RUST + PYTHON + SERVICE + E2E (only applicable changed sources and actual consumers; no waived required lane).

Stage: P5.

Scope: Validate multimodal disagreement and typed change dependencies; implement only the owning behavior described by this slice and its linked requirements. Split a red or oversized increment into independently green consumer checkpoints while preserving every obligation.

Planned paths: `python/src/`, `crates/arboresce-application/src/project/`, `tests/contract/project/`

Definition of green: Receipt/note/context disagreements and relative dates yield bounded valid typed items or explicit uncertainty; missing financial fields remain incomplete and all target/dependency/reference joins validate.

Verification: Independent literal contract/fixture oracles, owning language checks/strict coverage and the actual consumer/service/native/recovery lanes stated in the definition of green. Real-input or assurance requirements remain unavailable until supplied and qualified; mocks cannot close them.

### E236 — Integrate qualified live processors into Activities

Status: `not_started`. Prerequisites: E235, E226. Verify lane: DOC + CONTRACT + RUST + PYTHON + SERVICE + E2E (only applicable changed sources and actual consumers; no waived required lane).

Stage: P5.

Scope: Integrate qualified live processors into Activities; implement only the owning behavior described by this slice and its linked requirements. Split a red or oversized increment into independently green consumer checkpoints while preserving every obligation.

Planned paths: `crates/arboresce-activities/`, `python/src/`, `tests/integration/processing/`

Definition of green: Actual approved OCR/ASR/synthesis route executes through authorized gateways and durable stages; retries/cost/exposure/partial failure retain uncertainty without approval or duplicate canonical updates.

Verification: Independent literal contract/fixture oracles, owning language checks/strict coverage and the actual consumer/service/native/recovery lanes stated in the definition of green. Real-input or assurance requirements remain unavailable until supplied and qualified; mocks cannot close them.

### E237 — Persist change plans and exact review lineage

Status: `not_started`. Prerequisites: E226, E214. Verify lane: DOC + CONTRACT + RUST + PYTHON + SERVICE + E2E (only applicable changed sources and actual consumers; no waived required lane).

Stage: P6.

Scope: Persist change plans and exact review lineage; implement only the owning behavior described by this slice and its linked requirements. Split a red or oversized increment into independently green consumer checkpoints while preserving every obligation.

Planned paths: `crates/arboresce-postgres/src/project/`, `crates/arboresce-application/src/review/`, `tests/integration/review/`

Definition of green: Each typed item names capture/evidence/context/target/base revision/dependencies and uses existing candidate/review/asset identity; presentation grouping does not create indivisible verdicts or competing approvals.

Verification: Independent literal contract/fixture oracles, owning language checks/strict coverage and the actual consumer/service/native/recovery lanes stated in the definition of green. Real-input or assurance requirements remain unavailable until supplied and qualified; mocks cannot close them.

### E238 — Apply item edits, verdicts and partial dependencies atomically

Status: `not_started`. Prerequisites: E237, E215, E228. Verify lane: DOC + CONTRACT + RUST + PYTHON + SERVICE + E2E (only applicable changed sources and actual consumers; no waived required lane).

Stage: P6.

Scope: Apply item edits, verdicts and partial dependencies atomically; implement only the owning behavior described by this slice and its linked requirements. Split a red or oversized increment into independently green consumer checkpoints while preserving every obligation.

Planned paths: `crates/arboresce-application/src/review/`, `crates/arboresce-api/src/routes/review.rs`, `tests/integration/concurrency/`

Definition of green: Accept/edit/reject/pending preserves exact content attribution, new pending revisions invalidate old approvals, dependent holds and current CAS prevent invalid effects while independent items remain actionable.

Verification: Independent literal contract/fixture oracles, owning language checks/strict coverage and the actual consumer/service/native/recovery lanes stated in the definition of green. Real-input or assurance requirements remain unavailable until supplied and qualified; mocks cannot close them.

### E239 — Preserve corrections across reanalysis and follow-up revisions

Status: `not_started`. Prerequisites: E238. Verify lane: DOC + CONTRACT + RUST + PYTHON + SERVICE + E2E (only applicable changed sources and actual consumers; no waived required lane).

Stage: P6.

Scope: Preserve corrections across reanalysis and follow-up revisions; implement only the owning behavior described by this slice and its linked requirements. Split a red or oversized increment into independently green consumer checkpoints while preserving every obligation.

Planned paths: `crates/arboresce-application/src/project/`, `python/src/`, `tests/integration/reanalysis/`

Definition of green: Reanalysis matches prior targets, preserves manual/accepted/rejected decisions and offers attributable new suggestions; later corrections link superseded exact versions without duplicate actions or silent overwrites.

Verification: Independent literal contract/fixture oracles, owning language checks/strict coverage and the actual consumer/service/native/recovery lanes stated in the definition of green. Real-input or assurance requirements remain unavailable until supplied and qualified; mocks cannot close them.

### E240 — Qualify review races and correction persistence

Status: `not_started`. Prerequisites: E239. Verify lane: DOC + CONTRACT + RUST + PYTHON + SERVICE + E2E (only applicable changed sources and actual consumers; no waived required lane).

Stage: P6.

Scope: Qualify review races and correction persistence; implement only the owning behavior described by this slice and its linked requirements. Split a red or oversized increment into independently green consumer checkpoints while preserving every obligation.

Planned paths: `tests/integration/concurrency/`, `tests/e2e/review/`, `tests/fixtures/project/`

External prerequisite capabilities: Expose readable and composable machine review commands. Require independently qualified compatible public contracts/corpus/policy or actual client/service evidence for these consumed capabilities before this slice starts. This declares capability readiness without a private checkout or tooling dependency.

Definition of green: Real contention has one canonical winner, revocation denies new effects, retries preserve results and restart/follow-up/manual override fixtures retain exact history; recorded-model review can proceed before live provider readiness.

Verification: Independent literal contract/fixture oracles, owning language checks/strict coverage and the actual consumer/service/native/recovery lanes stated in the definition of green. Real-input or assurance requirements remain unavailable until supplied and qualified; mocks cannot close them.

### E241 — Expand packaged recovery for accepted project changes

Status: `not_started`. Prerequisites: E240, E229. Verify lane: DOC + CONTRACT + RUST + PYTHON + SERVICE + E2E (only applicable changed sources and actual consumers; no waived required lane).

Stage: P6.

Scope: Expand packaged recovery for accepted project changes; implement only the owning behavior described by this slice and its linked requirements. Split a red or oversized increment into independently green consumer checkpoints while preserving every obligation.

Planned paths: `crates/arboresce-admin/`, `tests/integration/recovery/`

Definition of green: Actual packaged old-backup restore preserves every newly enabled project/review/supersession state and current restrictions before private project-review capability is enabled.

Verification: Independent literal contract/fixture oracles, owning language checks/strict coverage and the actual consumer/service/native/recovery lanes stated in the definition of green. Real-input or assurance requirements remain unavailable until supplied and qualified; mocks cannot close them.

### E242 — Implement exact financial types and reviewed project association

Status: `not_started`. Prerequisites: E203, E200, E215. Verify lane: DOC + CONTRACT + RUST + PYTHON + SERVICE + E2E (only applicable changed sources and actual consumers; no waived required lane).

Stage: P7.

Scope: Implement exact financial types and reviewed project association; implement only the owning behavior described by this slice and its linked requirements. Split a red or oversized increment into independently green consumer checkpoints while preserving every obligation.

Planned paths: `crates/arboresce-domain/src/financial/`, `contracts/public/`, `tests/contract/financial/`

Definition of green: Existing exact-money/currency/date/reconciliation rules and typed same-workspace project association remain revision/hash-bound; unknown required values cannot be confirmable or enter confirmed reports.

Verification: Independent literal contract/fixture oracles, owning language checks/strict coverage and the actual consumer/service/native/recovery lanes stated in the definition of green. Real-input or assurance requirements remain unavailable until supplied and qualified; mocks cannot close them.

### E243 — Persist grounded expense proposals and corrections

Status: `not_started`. Prerequisites: E242, E226. Verify lane: DOC + CONTRACT + RUST + PYTHON + SERVICE + E2E (only applicable changed sources and actual consumers; no waived required lane).

Stage: P7.

Scope: Persist grounded expense proposals and corrections; implement only the owning behavior described by this slice and its linked requirements. Split a red or oversized increment into independently green consumer checkpoints while preserving every obligation.

Planned paths: `crates/arboresce-postgres/src/financial/`, `crates/arboresce-application/src/financial/`, `tests/integration/financial/`

Definition of green: Capture-linked proposal/revise history, components and reviewed project linkage persist as separate financial records; project verdicts/model assertions do not create financial review.

Verification: Independent literal contract/fixture oracles, owning language checks/strict coverage and the actual consumer/service/native/recovery lanes stated in the definition of green. Real-input or assurance requirements remain unavailable until supplied and qualified; mocks cannot close them.

### E244 — Issue and verify trusted exact-content financial review events

Status: `not_started`. Prerequisites: E243, E212. Verify lane: DOC + CONTRACT + RUST + PYTHON + SERVICE + E2E (only applicable changed sources and actual consumers; no waived required lane).

Stage: P7.

Scope: Issue and verify trusted exact-content financial review events; implement only the owning behavior described by this slice and its linked requirements. Split a red or oversized increment into independently green consumer checkpoints while preserving every obligation.

Planned paths: `crates/arboresce-application/src/financial/`, `crates/arboresce-api/src/routes/financial.rs`, `tests/security/financial-review/`

External prerequisite capabilities: Select and qualify the actual human review assurance mechanism. Require independently qualified compatible public contracts/corpus/policy or actual client/service evidence for these consumed capabilities before this slice starts. This declares capability readiness without a private checkout or tooling dependency.

Definition of green: Actual trusted flow binds qualified reviewer/assurance/expense/version/digest/tenant/purpose/expiry/executor; forged, revoked, stale or mismatched events and general grants cannot manufacture human decisions.

Verification: Independent literal contract/fixture oracles, owning language checks/strict coverage and the actual consumer/service/native/recovery lanes stated in the definition of green. Real-input or assurance requirements remain unavailable until supplied and qualified; mocks cannot close them.

### E245 — Confirm, void and revise financial records transactionally

Status: `not_started`. Prerequisites: E244, E207. Verify lane: DOC + CONTRACT + RUST + PYTHON + SERVICE + E2E (only applicable changed sources and actual consumers; no waived required lane).

Stage: P7.

Scope: Confirm, void and revise financial records transactionally; implement only the owning behavior described by this slice and its linked requirements. Split a red or oversized increment into independently green consumer checkpoints while preserving every obligation.

Planned paths: `crates/arboresce-application/src/financial/`, `crates/arboresce-postgres/src/financial/`, `tests/integration/concurrency/`

Definition of green: Exact confirmation, one-use review consumption/result/audit/outbox bind atomically; delegated executor has own current scope, idempotent replay is safe and void/refund/correction preserve original lineage.

Verification: Independent literal contract/fixture oracles, owning language checks/strict coverage and the actual consumer/service/native/recovery lanes stated in the definition of green. Real-input or assurance requirements remain unavailable until supplied and qualified; mocks cannot close them.

### E246 — Compile source-linked financial reports and exports

Status: `not_started`. Prerequisites: E245, E228. Verify lane: DOC + CONTRACT + RUST + PYTHON + SERVICE + E2E (only applicable changed sources and actual consumers; no waived required lane).

Stage: P7.

Scope: Compile source-linked financial reports and exports; implement only the owning behavior described by this slice and its linked requirements. Split a red or oversized increment into independently green consumer checkpoints while preserving every obligation.

Planned paths: `crates/arboresce-application/src/financial/`, `tests/integration/export/`

Definition of green: Immutable per-currency report selections and closed JSON/CSV exports independently reconstruct exact arithmetic, project links and source revisions; incomplete/denied records and current lifecycle changes remain disclosure-safe.

Verification: Independent literal contract/fixture oracles, owning language checks/strict coverage and the actual consumer/service/native/recovery lanes stated in the definition of green. Real-input or assurance requirements remain unavailable until supplied and qualified; mocks cannot close them.

### E247 — Qualify financial one-use, race and independent export oracles

Status: `not_started`. Prerequisites: none. Verify lane: DOC + CONTRACT + RUST + PYTHON + SERVICE + E2E (only applicable changed sources and actual consumers; no waived required lane).

Stage: P7.

Scope: Qualify financial one-use, race and independent export oracles; implement only the owning behavior described by this slice and its linked requirements. Split a red or oversized increment into independently green consumer checkpoints while preserving every obligation.

Planned paths: `tests/integration/financial/`, `tests/e2e/financial/`, `tests/fixtures/financial/`

External prerequisite capabilities: Expose expense correction, exact review and verified export. Require independently qualified compatible public contracts/corpus/policy or actual client/service evidence for these consumed capabilities before this slice starts. This declares capability readiness without a private checkout or tooling dependency.

Definition of green: Actual races allow one exact review consumption, stale approvals/current permission loss fail, missing fields remain proposed and separate literal oracles reconstruct exported records.

Verification: Independent literal contract/fixture oracles, owning language checks/strict coverage and the actual consumer/service/native/recovery lanes stated in the definition of green. Real-input or assurance requirements remain unavailable until supplied and qualified; mocks cannot close them.

### E248 — Expand packaged recovery for financial and export state

Status: `not_started`. Prerequisites: E247, E241. Verify lane: DOC + CONTRACT + RUST + PYTHON + SERVICE + E2E (only applicable changed sources and actual consumers; no waived required lane).

Stage: P7.

Scope: Expand packaged recovery for financial and export state; implement only the owning behavior described by this slice and its linked requirements. Split a red or oversized increment into independently green consumer checkpoints while preserving every obligation.

Planned paths: `crates/arboresce-admin/`, `tests/integration/recovery/`

Definition of green: Actual packaged restore conserves confirmed/proposed/void/superseded history, review consumption, immutable reports and current restrictions; missing bytes/keys/journal proofs keep serving closed before private finance enablement.

Verification: Independent literal contract/fixture oracles, owning language checks/strict coverage and the actual consumer/service/native/recovery lanes stated in the definition of green. Real-input or assurance requirements remain unavailable until supplied and qualified; mocks cannot close them.

### E249 — Run the complete deterministic real-service CLI scenario

Status: `not_started`. Prerequisites: E247, E240, E217. Verify lane: DOC + CONTRACT + RUST + PYTHON + SERVICE + E2E (only applicable changed sources and actual consumers; no waived required lane).

Stage: P8.

Scope: Run the complete deterministic real-service CLI scenario; implement only the owning behavior described by this slice and its linked requirements. Split a red or oversized increment into independently green consumer checkpoints while preserving every obligation.

Planned paths: `tests/e2e/project-workflow/`, `tests/fixtures/project/`

External prerequisite capabilities: Expose operation status, polling and cancellation. Require independently qualified compatible public contracts/corpus/policy or actual client/service evidence for these consumed capabilities before this slice starts. This declares capability readiness without a private checkout or tooling dependency.

Definition of green: Actual independent CLI/API/PG/object/Temporal/Python with recorded models proves seed/capture/multiple items/manual edits/partial verdicts/confirmed expense/project readback/JSON-CSV export and every required negative without answer leakage.

Verification: Independent literal contract/fixture oracles, owning language checks/strict coverage and the actual consumer/service/native/recovery lanes stated in the definition of green. Real-input or assurance requirements remain unavailable until supplied and qualified; mocks cannot close them.

### E250 — Prove whole-workflow replay, restart and scoped readback

Status: `not_started`. Prerequisites: E249, E248. Verify lane: DOC + CONTRACT + RUST + PYTHON + SERVICE + E2E (only applicable changed sources and actual consumers; no waived required lane).

Stage: P8.

Scope: Prove whole-workflow replay, restart and scoped readback; implement only the owning behavior described by this slice and its linked requirements. Split a red or oversized increment into independently green consumer checkpoints while preserving every obligation.

Planned paths: `tests/e2e/project-workflow/`, `tests/integration/recovery/`

Definition of green: Accepted changes and financial history survive actual restarts/retries, cancellation retains uncertain effects, later corrections compose and actor-in-both-workspaces tests never leak or move content.

Verification: Independent literal contract/fixture oracles, owning language checks/strict coverage and the actual consumer/service/native/recovery lanes stated in the definition of green. Real-input or assurance requirements remain unavailable until supplied and qualified; mocks cannot close them.

### E251 — Harden the complete focused packaged profile

Status: `not_started`. Prerequisites: E250, E227, E228. Verify lane: DOC + CONTRACT + RUST + PYTHON + SERVICE + E2E (only applicable changed sources and actual consumers; no waived required lane).

Stage: P8.

Scope: Harden the complete focused packaged profile; implement only the owning behavior described by this slice and its linked requirements. Split a red or oversized increment into independently green consumer checkpoints while preserving every obligation.

Planned paths: `docker/`, `docs/runbooks/`, `tests/security/`, `tests/e2e/`

Definition of green: All enabled profile components have actual current-auth/resource/codec/cost/startup/shutdown/secret/failure behavior, human-readable and machine flows; disabled future capabilities fail explicitly.

Verification: Independent literal contract/fixture oracles, owning language checks/strict coverage and the actual consumer/service/native/recovery lanes stated in the definition of green. Real-input or assurance requirements remain unavailable until supplied and qualified; mocks cannot close them.

### E252 — Qualify full bounded selection and listing consumers

Status: `not_started`. Prerequisites: none. Verify lane: DOC + CONTRACT + RUST + PYTHON + SERVICE + E2E (only applicable changed sources and actual consumers; no waived required lane).

Stage: P9.

Scope: Qualify full bounded selection and listing consumers; implement only the owning behavior described by this slice and its linked requirements. Split a red or oversized increment into independently green consumer checkpoints while preserving every obligation.

Planned paths: `contracts/public/`, `tests/contract/selection/`, `tests/integration/selection/`, `docs/requirements/selection-contract.md`

External prerequisite capabilities: Record focused MVP completion and the remaining milestones. Require independently qualified compatible public contracts/corpus/policy or actual client/service evidence for these consumed capabilities before this slice starts. This declares capability readiness without a private checkout or tooling dependency.

Definition of green: Retained selection/listing obligations and useful unfinished prior qualification work are integrated against actual consumers; all cursor/disclosure/serialization/reservation/transport vectors pass without lowering coverage or accepting old incomplete evidence.

Verification: Independent literal contract/fixture oracles, owning language checks/strict coverage and the actual consumer/service/native/recovery lanes stated in the definition of green. Real-input or assurance requirements remain unavailable until supplied and qualified; mocks cannot close them.

### E253 — Complete full reviewed asset versions and supersession

Status: `not_started`. Prerequisites: E252, E238. Verify lane: DOC + CONTRACT + RUST + PYTHON + SERVICE + E2E (only applicable changed sources and actual consumers; no waived required lane).

Stage: P9.

Scope: Complete full reviewed asset versions and supersession; implement only the owning behavior described by this slice and its linked requirements. Split a red or oversized increment into independently green consumer checkpoints while preserving every obligation.

Planned paths: `crates/arboresce-application/src/assets/`, `tests/integration/assets/`

Definition of green: Exact asset/review/version/control/supersession lineage and current eligibility agree across full historical reads; focused project materialization does not substitute for broader knowledge semantics.

Verification: Independent literal contract/fixture oracles, owning language checks/strict coverage and the actual consumer/service/native/recovery lanes stated in the definition of green. Real-input or assurance requirements remain unavailable until supplied and qualified; mocks cannot close them.

### E254 — Implement exact snapshot compilation and persistence

Status: `not_started`. Prerequisites: E253. Verify lane: DOC + CONTRACT + RUST + PYTHON + SERVICE + E2E (only applicable changed sources and actual consumers; no waived required lane).

Stage: P9.

Scope: Implement exact snapshot compilation and persistence; implement only the owning behavior described by this slice and its linked requirements. Split a red or oversized increment into independently green consumer checkpoints while preserving every obligation.

Planned paths: `crates/arboresce-application/src/builds/`, `crates/arboresce-workflows/`, `tests/integration/builds/`

Definition of green: Exact authorized asset versions compile canonical bytes/digests and sealed references through durable workflows, preserving qualifications and refusing partial or revoked-source snapshots.

Verification: Independent literal contract/fixture oracles, owning language checks/strict coverage and the actual consumer/service/native/recovery lanes stated in the definition of green. Real-input or assurance requirements remain unavailable until supplied and qualified; mocks cannot close them.

### E255 — Implement evaluation, attestations and channel promotion

Status: `not_started`. Prerequisites: E254. Verify lane: DOC + CONTRACT + RUST + PYTHON + SERVICE + E2E (only applicable changed sources and actual consumers; no waived required lane).

Stage: P9.

Scope: Implement evaluation, attestations and channel promotion; implement only the owning behavior described by this slice and its linked requirements. Split a red or oversized increment into independently green consumer checkpoints while preserving every obligation.

Planned paths: `crates/arboresce-application/src/builds/`, `crates/arboresce-application/src/channels/`, `tests/integration/channels/`

Definition of green: Independent evaluation suites, exact approval/Ed25519 trust receipts and CAS promotion/suspend/rollback qualify with current source eligibility; signing never bypasses policy.

Verification: Independent literal contract/fixture oracles, owning language checks/strict coverage and the actual consumer/service/native/recovery lanes stated in the definition of green. Real-input or assurance requirements remain unavailable until supplied and qualified; mocks cannot close them.

### E256 — Implement version-linked feedback and full lifecycle outcomes

Status: `not_started`. Prerequisites: none. Verify lane: DOC + CONTRACT + RUST + PYTHON + SERVICE + E2E (only applicable changed sources and actual consumers; no waived required lane).

Stage: P9.

Scope: Implement version-linked feedback and full lifecycle outcomes; implement only the owning behavior described by this slice and its linked requirements. Split a red or oversized increment into independently green consumer checkpoints while preserving every obligation.

Planned paths: `crates/arboresce-application/src/feedback/`, `crates/arboresce-application/src/lifecycle/`, `tests/integration/feedback/`

External prerequisite capabilities: Expose exact knowledge resolution and consumption. Require independently qualified compatible public contracts/corpus/policy or actual client/service evidence for these consumed capabilities before this slice starts. This declares capability readiness without a private checkout or tooling dependency.

Definition of green: Feedback/outcomes bind exact versions/author/source/event time; reviewed supersession and canonical dependency restrictions preserve prior assertions and current authority without automatic truth claims.

Verification: Independent literal contract/fixture oracles, owning language checks/strict coverage and the actual consumer/service/native/recovery lanes stated in the definition of green. Real-input or assurance requirements remain unavailable until supplied and qualified; mocks cannot close them.

### E257 — Implement qualified Qdrant and embedding projections

Status: `not_started`. Prerequisites: E252, E253. Verify lane: DOC + CONTRACT + RUST + PYTHON + SERVICE + E2E (only applicable changed sources and actual consumers; no waived required lane).

Stage: P9.

Scope: Implement qualified Qdrant and embedding projections; implement only the owning behavior described by this slice and its linked requirements. Split a red or oversized increment into independently green consumer checkpoints while preserving every obligation.

Planned paths: `crates/arboresce-projector/`, `python/src/`, `docker/qdrant/`, `tests/integration/qdrant/`

Definition of green: Actual pinned embedding dimensions/models and Qdrant generation membership/filtering/budgets qualify as rebuildable derived state; cached ACLs never authorize or redefine canonical assets.

Verification: Independent literal contract/fixture oracles, owning language checks/strict coverage and the actual consumer/service/native/recovery lanes stated in the definition of green. Real-input or assurance requirements remain unavailable until supplied and qualified; mocks cannot close them.

### E258 — Implement bounded authorized search and selection receipts

Status: `not_started`. Prerequisites: E257, E255. Verify lane: DOC + CONTRACT + RUST + PYTHON + SERVICE + E2E (only applicable changed sources and actual consumers; no waived required lane).

Stage: P9.

Scope: Implement bounded authorized search and selection receipts; implement only the owning behavior described by this slice and its linked requirements. Split a red or oversized increment into independently green consumer checkpoints while preserving every obligation.

Planned paths: `crates/arboresce-application/src/search/`, `crates/arboresce-api/src/routes/search.rs`, `tests/integration/search/`

Definition of green: Two total rounds and200 cumulative candidates include every branch before dedupe, authorize before rerank and report exact budgets/receipts/uncertainty with no hidden refills or post-revoke exposure.

Verification: Independent literal contract/fixture oracles, owning language checks/strict coverage and the actual consumer/service/native/recovery lanes stated in the definition of green. Real-input or assurance requirements remain unavailable until supplied and qualified; mocks cannot close them.

### E259 — Qualify actual propagation repair and multi-batch recovery

Status: `not_started`. Prerequisites: E256, E228. Verify lane: DOC + CONTRACT + RUST + PYTHON + SERVICE + E2E (only applicable changed sources and actual consumers; no waived required lane).

Stage: P9.

Scope: Qualify actual propagation repair and multi-batch recovery; implement only the owning behavior described by this slice and its linked requirements. Split a red or oversized increment into independently green consumer checkpoints while preserving every obligation.

Planned paths: `crates/arboresce-application/src/lifecycle/`, `crates/arboresce-projector/`, `tests/integration/recovery/`, `tests/integration/qdrant/`

External prerequisite capabilities: Complete search, broad command matrices and transport negatives. Require independently qualified compatible public contracts/corpus/policy or actual client/service evidence for these consumed capabilities before this slice starts. This declares capability readiness without a private checkout or tooling dependency.

Definition of green: Original real E124 SERVICE/RUST/Qdrant inventory/binding/repair/action/coverage negatives and actual multi-batch old-backup continuity pass; unknown effects never repeat and complete current restriction proof precedes activation.

Verification: Independent literal contract/fixture oracles, owning language checks/strict coverage and the actual consumer/service/native/recovery lanes stated in the definition of green. Real-input or assurance requirements remain unavailable until supplied and qualified; mocks cannot close them.

### E260 — Repeat broader D0 restore before knowledge/search activation

Status: `not_started`. Prerequisites: E259, E254, E248. Verify lane: DOC + CONTRACT + RUST + PYTHON + SERVICE + E2E (only applicable changed sources and actual consumers; no waived required lane).

Stage: P9.

Scope: Repeat broader D0 restore before knowledge/search activation; implement only the owning behavior described by this slice and its linked requirements. Split a red or oversized increment into independently green consumer checkpoints while preserving every obligation.

Planned paths: `crates/arboresce-admin/`, `tests/integration/recovery/`, `docs/runbooks/recovery.md`

Definition of green: Actual packaged recovery restores full builds/projections/feedback/repair state and current restrictions, corrupt derived stores rebuild canonically, and initial focused proof is not relabeled as full old D0.

Verification: Independent literal contract/fixture oracles, owning language checks/strict coverage and the actual consumer/service/native/recovery lanes stated in the definition of green. Real-input or assurance requirements remain unavailable until supplied and qualified; mocks cannot close them.

### E261 — Qualify real managed PostgreSQL and pooling E02

Status: `not_started`. Prerequisites: none. Verify lane: DOC + CONTRACT + RUST + PYTHON + SERVICE + E2E (only applicable changed sources and actual consumers; no waived required lane).

Stage: P10.

Scope: Qualify real managed PostgreSQL and pooling E02; implement only the owning behavior described by this slice and its linked requirements. Split a red or oversized increment into independently green consumer checkpoints while preserving every obligation.

Planned paths: `tests/experiments/postgres/`, `docs/runbooks/`

External prerequisite capabilities: Accept the retained broader G3 knowledge journey. Require independently qualified compatible public contracts/corpus/policy or actual client/service evidence for these consumed capabilities before this slice starts. This declares capability readiness without a private checkout or tooling dependency.

Definition of green: Actual selected permitted provider/region/non-owner roles/pool/session/failure/capacity matrix meets original isolation and latency allocations; local tests are not relabeled cloud evidence.

Verification: Independent literal contract/fixture oracles, owning language checks/strict coverage and the actual consumer/service/native/recovery lanes stated in the definition of green. Real-input or assurance requirements remain unavailable until supplied and qualified; mocks cannot close them.

### E262 — Qualify full workflow durability and upgrade E03

Status: `not_started`. Prerequisites: none. Verify lane: DOC + CONTRACT + RUST + PYTHON + SERVICE + E2E (only applicable changed sources and actual consumers; no waived required lane).

Stage: P10.

Scope: Qualify full workflow durability and upgrade E03; implement only the owning behavior described by this slice and its linked requirements. Split a red or oversized increment into independently green consumer checkpoints while preserving every obligation.

Planned paths: `tests/experiments/temporal/`, `docs/runbooks/`

External prerequisite capabilities: Accept the retained broader G3 knowledge journey. Require independently qualified compatible public contracts/corpus/policy or actual client/service evidence for these consumed capabilities before this slice starts. This declares capability readiness without a private checkout or tooling dependency.

Definition of green: Actual dedicated persistence/worker-loss/cancellation/retry/replay/upgrade workload meets original counts/time bounds and retains uncertain external effects without duplicate canonical transitions.

Verification: Independent literal contract/fixture oracles, owning language checks/strict coverage and the actual consumer/service/native/recovery lanes stated in the definition of green. Real-input or assurance requirements remain unavailable until supplied and qualified; mocks cannot close them.

### E263 — Qualify full D1 restoration and operating integrity E09

Status: `not_started`. Prerequisites: none. Verify lane: DOC + CONTRACT + RUST + PYTHON + SERVICE + E2E (only applicable changed sources and actual consumers; no waived required lane).

Stage: P10.

Scope: Qualify full D1 restoration and operating integrity E09; implement only the owning behavior described by this slice and its linked requirements. Split a red or oversized increment into independently green consumer checkpoints while preserving every obligation.

Planned paths: `tests/experiments/recovery/`, `docs/runbooks/recovery.md`

External prerequisite capabilities: Qualify retrieval, context and resource experiments E06-E08. Require independently qualified compatible public contracts/corpus/policy or actual client/service evidence for these consumed capabilities before this slice starts. This declares capability readiness without a private checkout or tooling dependency.

Definition of green: Actual original D1 later-restriction/repair/workflow/object/key/journal restoration meets selected RPO/RTO/residency commitments and fails closed on absent/corrupt/incomplete evidence.

Verification: Independent literal contract/fixture oracles, owning language checks/strict coverage and the actual consumer/service/native/recovery lanes stated in the definition of green. Real-input or assurance requirements remain unavailable until supplied and qualified; mocks cannot close them.

### E264 — Qualify production native service distributions

Status: `not_started`. Prerequisites: none. Verify lane: DOC + CONTRACT + RUST + PYTHON + SERVICE + E2E (only applicable changed sources and actual consumers; no waived required lane).

Stage: P10.

Scope: Qualify production native service distributions; implement only the owning behavior described by this slice and its linked requirements. Split a red or oversized increment into independently green consumer checkpoints while preserving every obligation.

Planned paths: `docker/`, `docs/runbooks/`, `tests/experiments/distribution/`

External prerequisite capabilities: Accept the retained broader G3 knowledge journey. Require independently qualified compatible public contracts/corpus/policy or actual client/service evidence for these consumed capabilities before this slice starts. This declares capability readiness without a private checkout or tooling dependency.

Definition of green: Actual supported native Linux AMD64 image/runtime/resource/startup/shutdown/provenance profiles and clean independent builds meet original E01/G6; cross-compilation or architecture labels alone do not qualify.

Verification: Independent literal contract/fixture oracles, owning language checks/strict coverage and the actual consumer/service/native/recovery lanes stated in the definition of green. Real-input or assurance requirements remain unavailable until supplied and qualified; mocks cannot close them.

### E265 — Implement and qualify optional SMTP notifications

Status: `not_started`. Prerequisites: E224, E228. Verify lane: DOC + CONTRACT + RUST + PYTHON + SERVICE + E2E (applicable sources and actual consumers; no waived required lane).

Stage: P10.

Scope: Implement and qualify optional SMTP notifications under its owning normative requirements; independently green code, contract and real-service evidence qualify this capability before use.

Planned paths: `crates/arboresce-application/src/notifications/`, `crates/arboresce-infrastructure/src/smtp/`, `crates/arboresce-worker/src/activities/notifications.rs`, `tests/integration/mail/`

Definition of green: The bounded engine notification port, SMTP adapter and Temporal Activities enforce exact authorized intents, current restrictions, qualified destinations/TLS/credentials, resource reservations and durable uncertain outcomes. Independent literal injection/retry/cancellation/delivery vectors and actual controlled Mailpit acceptance/failure/uncertainty tests pass. No email is sent from an inferred project update; an optional operational route stays disabled until its actual profile qualifies.

Verification: Independent literal fixture/oracle truth, exact owning format/static/build/test and strict coverage, actual process/service integration and the existing qualification limits. Optional unqualified routes remain disabled.

### E266 — Produce and deliver complete customer exit archives

Status: `not_started`. Prerequisites: E256. Verify lane: DOC + CONTRACT + RUST + PYTHON + SERVICE + E2E (applicable sources and actual consumers; no waived required lane).

Stage: P9.

Scope: Produce and deliver complete customer exit archives under its owning normative requirements; independently green code, contract and real-service evidence qualify this capability before use.

Planned paths: `crates/arboresce-application/src/export/`, `crates/arboresce-infrastructure/src/export/`, `crates/arboresce-api/src/routes/export.rs`, `tests/integration/export/`

Definition of green: The engine selects the complete chosen authorized customer scope, durably produces a bounded versioned archive of required canonical records, source bytes, relationships and revision/receipt/profile history, and fences publication and every download with current authority/restrictions. Independent parsers verify actual archive bytes, manifests/digests and completeness; missing required bytes, fake completeness, cross-scope data, stale authority, retry/restart and interrupted download fail truthfully. Secrets are excluded.

Verification: Independent literal fixture/oracle truth, exact owning format/static/build/test and strict coverage, actual process/service integration and the existing qualification limits. Optional unqualified routes remain disabled.

### E267 — Resolve exact context snapshots and complete authorized delivery

Status: `not_started`. Prerequisites: E255. Verify lane: DOC + CONTRACT + RUST + PYTHON + SERVICE + E2E (applicable sources and actual consumers; no waived required lane).

Stage: P9.

Scope: Resolve exact context snapshots and complete authorized delivery under its owning normative requirements; independently green code, contract and real-service evidence qualify this capability before use.

Planned paths: `crates/arboresce-application/src/resolve/`, `crates/arboresce-api/src/routes/context.rs`, `tests/integration/context/resolve/`

Definition of green: Actual engine resolution returns the exact qualified snapshot/channel generation and attributable usage receipt; current authorization and every required source/control/profile/trust reference remain valid at resolution and complete bounded delivery. Independent literal identity/receipt/preimage/byte expectations and real-service races prove stale generation, revoked or unknown authority, partial delivery and mismatched source identity cannot masquerade as a usable complete context.

Verification: Independent literal fixture/oracle truth, exact owning format/static/build/test and strict coverage, actual process/service integration and the existing qualification limits. Optional unqualified routes remain disabled.
