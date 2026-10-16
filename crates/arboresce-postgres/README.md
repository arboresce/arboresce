# Isolated PostgreSQL migration foundation

This package implements the synthetic typed record port and two ordered forward
migrations. It is under qualification. It does not admit product or bootstrap
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
