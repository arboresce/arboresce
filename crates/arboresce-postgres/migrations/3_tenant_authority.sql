CREATE SCHEMA arboresce_authority;
REVOKE ALL ON SCHEMA arboresce_authority FROM PUBLIC;

CREATE DOMAIN arboresce_authority.identifier AS uuid
  CHECK ((get_byte(uuid_send(VALUE), 6) >> 4) = 4
     AND (get_byte(uuid_send(VALUE), 8) >> 6) = 2);

CREATE TABLE arboresce_authority.organizations (
  organization_id arboresce_authority.identifier PRIMARY KEY,
  state text NOT NULL CHECK (state IN ('active', 'revoked'))
);

CREATE TABLE arboresce_authority.principals (
  principal_id arboresce_authority.identifier PRIMARY KEY,
  identity_profile text NOT NULL CHECK (identity_profile = 'arboresce.nostr-identity.v1'),
  identity_version text NOT NULL CHECK (identity_version = 'nip01-v1'),
  public_key bytea NOT NULL CHECK (octet_length(public_key) = 32),
  state text NOT NULL CHECK (state IN ('active', 'revoked')),
  UNIQUE (identity_profile, identity_version, public_key)
);

CREATE TABLE arboresce_authority.domains (
  organization_id arboresce_authority.identifier NOT NULL
    REFERENCES arboresce_authority.organizations,
  domain_id arboresce_authority.identifier NOT NULL,
  state text NOT NULL CHECK (state IN ('active', 'revoked')),
  PRIMARY KEY (organization_id, domain_id)
);

CREATE TABLE arboresce_authority.memberships (
  organization_id arboresce_authority.identifier NOT NULL,
  principal_id arboresce_authority.identifier NOT NULL
    REFERENCES arboresce_authority.principals,
  domain_id arboresce_authority.identifier NOT NULL,
  membership_id arboresce_authority.identifier NOT NULL,
  state text NOT NULL CHECK (state IN ('active', 'revoked')),
  PRIMARY KEY (organization_id, principal_id),
  UNIQUE (organization_id, membership_id),
  FOREIGN KEY (organization_id, domain_id)
    REFERENCES arboresce_authority.domains (organization_id, domain_id)
);

ALTER TABLE arboresce_authority.organizations ENABLE ROW LEVEL SECURITY;
ALTER TABLE arboresce_authority.organizations FORCE ROW LEVEL SECURITY;
ALTER TABLE arboresce_authority.principals ENABLE ROW LEVEL SECURITY;
ALTER TABLE arboresce_authority.principals FORCE ROW LEVEL SECURITY;
ALTER TABLE arboresce_authority.domains ENABLE ROW LEVEL SECURITY;
ALTER TABLE arboresce_authority.domains FORCE ROW LEVEL SECURITY;
ALTER TABLE arboresce_authority.memberships ENABLE ROW LEVEL SECURITY;
ALTER TABLE arboresce_authority.memberships FORCE ROW LEVEL SECURITY;

CREATE POLICY principal_context ON arboresce_authority.principals FOR SELECT
  USING (principal_id::text = nullif(current_setting('arboresce.principal_id', true), ''));

CREATE POLICY membership_context ON arboresce_authority.memberships FOR SELECT
  USING (organization_id::text = nullif(current_setting('arboresce.organization_id', true), '')
     AND principal_id::text = nullif(current_setting('arboresce.principal_id', true), ''));

CREATE POLICY organization_context ON arboresce_authority.organizations FOR SELECT
  USING (organization_id::text = nullif(current_setting('arboresce.organization_id', true), '')
     AND EXISTS (
       SELECT 1 FROM arboresce_authority.memberships m
       JOIN arboresce_authority.principals p USING (principal_id)
       WHERE m.organization_id = organizations.organization_id
         AND m.state = 'active' AND p.state = 'active'
     ));

CREATE POLICY domain_context ON arboresce_authority.domains FOR SELECT
  USING (organization_id::text = nullif(current_setting('arboresce.organization_id', true), '')
     AND domain_id::text = nullif(current_setting('arboresce.domain_id', true), '')
     AND EXISTS (
       SELECT 1 FROM arboresce_authority.organizations o
       JOIN arboresce_authority.memberships m USING (organization_id)
       JOIN arboresce_authority.principals p USING (principal_id)
       WHERE o.organization_id = domains.organization_id
         AND m.domain_id = domains.domain_id
         AND o.state = 'active' AND m.state = 'active' AND p.state = 'active'
     ));

GRANT USAGE ON SCHEMA arboresce_authority TO harness_writer;
GRANT SELECT ON arboresce_authority.organizations,
  arboresce_authority.principals, arboresce_authority.domains,
  arboresce_authority.memberships TO harness_writer;
