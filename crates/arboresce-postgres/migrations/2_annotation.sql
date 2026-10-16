ALTER TABLE migration_harness.records
    ADD COLUMN annotation text NOT NULL DEFAULT ''
    CHECK (octet_length(annotation) <= 256);
