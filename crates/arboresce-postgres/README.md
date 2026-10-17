# Isolated PostgreSQL migration foundation

This package implements the synthetic typed record port and two ordered forward
migrations. Its isolated migration foundation is independently qualified; see the
[accepted checkpoint](../../docs/execution/open-cli-mvp-v1-rcld.md#e204--create-focused-application-ports-and-migration-harness).
It does not admit product or bootstrap
writes, tenant authorization, lifecycle continuity, pooling or packaged recovery.

The explicit target is a prepared protected Unix socket for PostgreSQL 17.11,
database `arboresce_migration_harness`, with distinct `harness_migrator` and
non-owner `harness_writer` roles. PG environment variables are rejected before
the upstream options constructor runs. No ambient DSN or password file selects
the target. Connection and transaction deadlines match the selected local
resource profile. Only positively completed COMMIT or ROLLBACK supplies a
definite transaction result; completion loss remains Unknown without retries.

Migration history is read under the real direct migration advisory lock. It must
be an exact successful checksum-matching prefix before any ledger creation,
schema change or replay. A newer database rejects an older selected migration
set. Transactional DDL failure cannot produce a successful migration receipt.
Exact independently reviewed scenarios and service input requirements belong to
the [migration expectations](../../tests/integration/postgres/fixtures/migration-harness-v1/README.md).

The tenant foundation adds an explicitly selected third forward migration while
preserving the original two migration profiles and their exact SQL. It creates
canonical organization, principal, domain and membership keys, exact identity
profile/key uniqueness, same-organization domain references, forced row security
and SELECT-only ordinary access. Privileged synthetic qualification seeds these
relations; no registration, first-owner or product writer is exposed.

`Harness::tenant_session` reuses one physical direct connection in the prepared
isolated namespace. Each bounded read transaction validates the actual role and
catalogue, rejects retained session context, installs complete transaction-local
organization/domain/principal settings and reads current active membership under
explicit READ COMMITTED, READ ONLY. `TenantContext` must come from the qualified
server boundary; its construction is no signature or action-authority proof.
Positive commit/rollback completion and lost completion remain distinct.
`TenantFailure` implements the standard Rust diagnostic error traits with its
finite variant label and no underlying cause. Typed propagation preserves the
variant through boxed errors without allocating replacement diagnostic strings.

The complete [tenant inputs](../../tests/security/fixtures/tenant-foundation-v1/README.md)
and all 25 original authority tenant cases run serially in the existing required
service harness. Qualification-only probe relations exercise actual RLS and
composite typed references. They do not implement product Project, Party, capture
or expense persistence. Complete source/native measurement includes the fixture
reader, seeds, catalogue observations, fault support and all actual process paths.
Current action/grant locks, admission/final-effect races, shared quotas, authentic
lifecycle continuity, product writers, actual pooling, HTTP/authentication and
whole milestone acceptance retain their separate consuming gates.

The explicitly selected CurrentAuthority profile adds a fourth forward
migration for bounded synthetic current-authority records and immutable sealed
members. The original three SQL files and ordinary tenant reader remain exact.
Separate nonowner admission and permission sessions validate the selected role,
forced row security, policies and column privileges. Their parent-first protocol
holds actual selected child locks through a positive terminal. A live admission
fence retains its originating request, trusted configuration and supplied clock;
marker/CAS effects repeat complete current reads and time checks after waits.
Resource CAS also requires the review action. Completion loss and cancellation
poison the owned session; only positive bounded transport close releases its
finite local slot. Poisoned transports are destroyed by an owned task without
polling driver shutdown, which could flush a queued rollback. The slot remains
charged until that task completes within the close budget. This is a direct
synthetic session set, with actual backend
census and the selected database connection limit, rather than a general pool.

The [current SQL inputs](../../tests/security/fixtures/current-sql-authority-v1/README.md)
select two complete native targets, authority_current and authority_races,
before the existing qualification target and its final restart. Each new target
owns one serial service cohort and restores the initial database/roles before
returning. SQL observations keep original fixture identities. Their additional
physical-consumer events use sql-native- plus the fixture identity under the
existing collector case= prefix; earlier pure and tenant events remain exact.
Separate sql-helper- events cover comparison representations and harmless owned
PID fixture files only. They never substitute for actual service identity,
profile, process, race or recovery observations. Existing race actors share the
same terminal helper while retaining their actual blocker, backend and clock
observations. Full physical before/after values retain controlled malformed
timestamps independently of their canonical display.
Full actual catalogue and constraint vectors are retained in bounded,
hash-bound JSON files under the Cargo-owned target/tmp evidence tree. Controlled
faults must record the exact named removal and preservation of every other
constraint. Legacy qualification separately observes the selected production
decoder under the nonowner admission role without claiming an original first-failure trace.
The additional isolated metadata inputs populate every preceding valid field
and leave only declared nullable fields empty, preserving their actual foreign
keys. A same-count column rename retains full catalogue, physical identity and
effective privilege observations, including its exact named dependent CHECK.
The trusted Clock failure is an injected dependency result after actual locked
reads. Separate sql-classifier- events exercise structured in-memory migration
errors; they establish no native SQL, transport or host failure cause.
Guard constructor refusals occur before owned filesystem effects and before any
overbound copy or fill. Shared connection routines positively close owned
sessions on operation errors and retain both errors if cleanup also fails.
Every maintained reader, seed, catalogue observer, fault, relay and race helper
remains measured. These isolated markers and controller restrictions enable no
product/bootstrap writer, command receipt, human assurance or HTTP route.
Authentic lifecycle, authentication, pooling and production/recovery qualification
retain their separate mandatory gates.

Finite sql-cleanup- events inspect original boxed values and both distinct typed errors, including their allocation identity, through the measured cleanup combiner. Separate sql-preparation- events inspect existing scoped-selector uniqueness, timeout parsing and compiled lock-query fragments with bound keys. They perform no database or filesystem operation and establish no native error, actual server setting, lock schedule or recovery outcome. Ordinary raw probe transactions share exactly one awaited rollback and owned close while retaining operation and cleanup errors. Authority sessions, fences and ambiguous completions keep their existing terminal paths and all seven original database schedules.
