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
