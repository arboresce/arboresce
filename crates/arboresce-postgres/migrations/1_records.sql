CREATE SCHEMA migration_harness;

CREATE TABLE migration_harness.records (
    organization_id uuid NOT NULL,
    record_id uuid NOT NULL,
    content_revision bigint NOT NULL CHECK (
        content_revision BETWEEN 1 AND 999999999999999999
    ),
    record_date date NOT NULL CHECK (
        record_date BETWEEN DATE '0001-01-01' AND DATE '9999-12-31'
    ),
    PRIMARY KEY (organization_id, record_id),
    CHECK (
        (get_byte(uuid_send(organization_id), 6) >> 4) = 4
        AND (get_byte(uuid_send(organization_id), 8) & 192) = 128
    ),
    CHECK (
        (get_byte(uuid_send(record_id), 6) >> 4) = 4
        AND (get_byte(uuid_send(record_id), 8) & 192) = 128
    )
);

GRANT USAGE ON SCHEMA migration_harness TO harness_writer;
GRANT SELECT, INSERT ON migration_harness.records TO harness_writer;
