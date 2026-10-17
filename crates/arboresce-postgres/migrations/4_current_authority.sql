-- Isolated synthetic current-authority storage. No product writer is enabled.
ALTER TABLE arboresce_authority.organizations
  ADD COLUMN name text CHECK (octet_length(name) <= 256),
  ADD COLUMN lock_guard boolean NOT NULL DEFAULT false CHECK (NOT lock_guard);
ALTER TABLE arboresce_authority.domains
  ADD COLUMN name text CHECK (octet_length(name) <= 256),
  ADD COLUMN lock_guard boolean NOT NULL DEFAULT false CHECK (NOT lock_guard);
ALTER TABLE arboresce_authority.principals
  ADD COLUMN kind text CHECK (kind IN ('human','agent','service')),
  ADD COLUMN control_revision bigint CHECK (control_revision BETWEEN 1 AND 999999999999999999),
  ADD COLUMN created_at timestamptz CONSTRAINT principals_created_at_check CHECK
    (CASE WHEN isfinite(created_at) THEN
      created_at BETWEEN timestamptz '0001-01-01 00:00:00+00' AND timestamptz '9999-12-31 23:59:59.999+00'
      AND mod(extract(microseconds FROM created_at),1000)=0
    ELSE created_at IS NULL END),
  ADD COLUMN revoked_at timestamptz CHECK
    (CASE WHEN isfinite(revoked_at) THEN
      revoked_at BETWEEN timestamptz '0001-01-01 00:00:00+00' AND timestamptz '9999-12-31 23:59:59.999+00'
      AND mod(extract(microseconds FROM revoked_at),1000)=0
    ELSE revoked_at IS NULL END),
  ADD COLUMN identity_standard text CHECK (identity_standard ~ '^[A-Za-z0-9_.-]{1,96}$'),
  ADD COLUMN identity_standard_revision text CHECK (identity_standard_revision ~ '^[ -~]{1,96}$'),
  ADD COLUMN lock_guard boolean NOT NULL DEFAULT false CHECK (NOT lock_guard),
  ADD CONSTRAINT principals_identity_kind_key UNIQUE (principal_id,kind);
ALTER TABLE arboresce_authority.memberships
  ADD COLUMN principal_kind text CHECK (principal_kind IN ('human','agent','service')),
  ADD COLUMN roles text[] CONSTRAINT memberships_roles_check CHECK
    (cardinality(roles)<=16 AND (cardinality(roles)=0 OR (array_ndims(roles)=1 AND array_lower(roles,1)=1))
      AND array_to_json(roles)::text ~ '^\[("[A-Za-z0-9_.-]{1,96}"(,"[A-Za-z0-9_.-]{1,96}")*)?\]$'),
  ADD COLUMN policy text CHECK (policy ~ '^[A-Za-z0-9_.-]{1,96}$'),
  ADD COLUMN control_revision bigint CHECK (control_revision BETWEEN 1 AND 999999999999999999),
  ADD COLUMN lock_guard boolean NOT NULL DEFAULT false CHECK (NOT lock_guard),
  ADD CONSTRAINT memberships_principal_kind_fkey FOREIGN KEY (principal_id,principal_kind)
    REFERENCES arboresce_authority.principals (principal_id,kind);

CREATE TABLE arboresce_authority.namespace_guard (
  singleton smallint PRIMARY KEY CHECK (singleton=1),
  epoch bigint NOT NULL CHECK (epoch BETWEEN 1 AND 999999999999999999),
  lock_guard boolean NOT NULL DEFAULT false CHECK (NOT lock_guard)
);
INSERT INTO arboresce_authority.namespace_guard (singleton,epoch) VALUES (1,1);

CREATE TABLE arboresce_authority.workspace_policies (
  organization_id arboresce_authority.identifier NOT NULL,
  domain_id arboresce_authority.identifier NOT NULL,
  policy text NOT NULL CONSTRAINT workspace_policies_policy_check CHECK (policy ~ '^[A-Za-z0-9_.-]{1,96}$'),
  epoch bigint NOT NULL CHECK (epoch BETWEEN 1 AND 999999999999999999),
  state text NOT NULL CHECK (state ~ '^[A-Za-z0-9_.-]{1,96}$'),
  read_roles text[] NOT NULL CHECK (cardinality(read_roles)<=16 AND (cardinality(read_roles)=0 OR (array_ndims(read_roles)=1 AND array_lower(read_roles,1)=1)) AND array_to_json(read_roles)::text ~ '^\[("[A-Za-z0-9_.-]{1,96}"(,"[A-Za-z0-9_.-]{1,96}")*)?\]$'),
  direct_review_roles text[] NOT NULL CHECK (cardinality(direct_review_roles)<=16 AND (cardinality(direct_review_roles)=0 OR (array_ndims(direct_review_roles)=1 AND array_lower(direct_review_roles,1)=1)) AND array_to_json(direct_review_roles)::text ~ '^\[("[A-Za-z0-9_.-]{1,96}"(,"[A-Za-z0-9_.-]{1,96}")*)?\]$'),
  purposes text[] NOT NULL CHECK (cardinality(purposes)<=16 AND (cardinality(purposes)=0 OR (array_ndims(purposes)=1 AND array_lower(purposes,1)=1)) AND array_to_json(purposes)::text ~ '^\[("[A-Za-z0-9_.-]{1,96}"(,"[A-Za-z0-9_.-]{1,96}")*)?\]$'),
  grant_root_actions text[] NOT NULL CHECK (cardinality(grant_root_actions)<=16 AND (cardinality(grant_root_actions)=0 OR (array_ndims(grant_root_actions)=1 AND array_lower(grant_root_actions,1)=1)) AND array_to_json(grant_root_actions)::text ~ '^\[("[A-Za-z0-9_.-]{1,96}"(,"[A-Za-z0-9_.-]{1,96}")*)?\]$'),
  source_policy_ids text[] NOT NULL CHECK (cardinality(source_policy_ids)<=16 AND (cardinality(source_policy_ids)=0 OR (array_ndims(source_policy_ids)=1 AND array_lower(source_policy_ids,1)=1)) AND array_to_json(source_policy_ids)::text ~ '^\[("[A-Za-z0-9_.-]{1,96}"(,"[A-Za-z0-9_.-]{1,96}")*)?\]$'),
  lock_guard boolean NOT NULL DEFAULT false CHECK (NOT lock_guard),
  PRIMARY KEY (organization_id,domain_id),
  FOREIGN KEY (organization_id,domain_id) REFERENCES arboresce_authority.domains
);
CREATE TABLE arboresce_authority.source_policies (
  policy_id text PRIMARY KEY CHECK (policy_id ~ '^[A-Za-z0-9_.-]{1,96}$'),
  state text NOT NULL CHECK (state ~ '^[A-Za-z0-9_.-]{1,96}$'),
  expires_at timestamptz NOT NULL CHECK
    (CASE WHEN isfinite(expires_at) THEN
      expires_at BETWEEN timestamptz '0001-01-01 00:00:00+00' AND timestamptz '9999-12-31 23:59:59.999+00'
      AND mod(extract(microseconds FROM expires_at),1000)=0 ELSE false END),
  permitted_principals arboresce_authority.identifier[] NOT NULL CHECK
    (cardinality(permitted_principals)<=32 AND (cardinality(permitted_principals)=0 OR (array_ndims(permitted_principals)=1 AND array_lower(permitted_principals,1)=1)) AND array_position(permitted_principals,NULL) IS NULL),
  permitted_purposes text[] NOT NULL CHECK (cardinality(permitted_purposes)<=16 AND (cardinality(permitted_purposes)=0 OR (array_ndims(permitted_purposes)=1 AND array_lower(permitted_purposes,1)=1)) AND array_to_json(permitted_purposes)::text ~ '^\[("[A-Za-z0-9_.-]{1,96}"(,"[A-Za-z0-9_.-]{1,96}")*)?\]$'),
  lock_guard boolean NOT NULL DEFAULT false CHECK (NOT lock_guard)
);

CREATE TABLE arboresce_authority.resources (
  organization_id arboresce_authority.identifier NOT NULL,
  domain_id arboresce_authority.identifier NOT NULL,
  resource_kind text NOT NULL CHECK (resource_kind ~ '^[A-Za-z0-9_.-]{1,96}$'),
  resource_id arboresce_authority.identifier NOT NULL,
  name text NOT NULL CHECK (octet_length(name)<=256),
  control_revision bigint NOT NULL CONSTRAINT resources_control_revision_check CHECK (control_revision BETWEEN 1 AND 999999999999999999),
  state text NOT NULL CHECK (state ~ '^[A-Za-z0-9_.-]{1,96}$'),
  lock_guard boolean NOT NULL DEFAULT false CHECK (NOT lock_guard),
  PRIMARY KEY (organization_id,domain_id,resource_kind,resource_id),
  FOREIGN KEY (organization_id,domain_id) REFERENCES arboresce_authority.domains
);
CREATE TABLE arboresce_authority.party_aliases (
  organization_id arboresce_authority.identifier NOT NULL,
  domain_id arboresce_authority.identifier NOT NULL,
  party_id arboresce_authority.identifier NOT NULL,
  alias text NOT NULL CHECK (octet_length(alias)<=256),
  principal_id arboresce_authority.identifier,
  PRIMARY KEY (organization_id,domain_id,party_id),
  FOREIGN KEY (organization_id,domain_id) REFERENCES arboresce_authority.domains,
  FOREIGN KEY (organization_id,principal_id) REFERENCES arboresce_authority.memberships (organization_id,principal_id)
);
CREATE TABLE arboresce_authority.grants (
  organization_id arboresce_authority.identifier NOT NULL,
  domain_id arboresce_authority.identifier NOT NULL,
  grant_id arboresce_authority.identifier NOT NULL UNIQUE,
  issuer_id arboresce_authority.identifier NOT NULL,
  issuer_kind text NOT NULL,
  subject_id arboresce_authority.identifier NOT NULL,
  subject_kind text NOT NULL,
  parent_grant_id arboresce_authority.identifier,
  scope_organization_id arboresce_authority.identifier NOT NULL,
  scope_domain_id arboresce_authority.identifier NOT NULL,
  scope_kind text NOT NULL CHECK (scope_kind IN ('project','expense','workspace')),
  scope_resource_id arboresce_authority.identifier,
  actions text[] NOT NULL CHECK (cardinality(actions)<=16 AND (cardinality(actions)=0 OR (array_ndims(actions)=1 AND array_lower(actions,1)=1)) AND array_to_json(actions)::text ~ '^\[("[A-Za-z0-9_.-]{1,96}"(,"[A-Za-z0-9_.-]{1,96}")*)?\]$'),
  purposes text[] NOT NULL CHECK (cardinality(purposes)<=16 AND (cardinality(purposes)=0 OR (array_ndims(purposes)=1 AND array_lower(purposes,1)=1)) AND array_to_json(purposes)::text ~ '^\[("[A-Za-z0-9_.-]{1,96}"(,"[A-Za-z0-9_.-]{1,96}")*)?\]$'),
  delegable boolean NOT NULL,
  issued_at timestamptz NOT NULL CHECK
    (CASE WHEN isfinite(issued_at) THEN issued_at BETWEEN timestamptz '0001-01-01 00:00:00+00' AND timestamptz '9999-12-31 23:59:59.999+00' AND mod(extract(microseconds FROM issued_at),1000)=0 ELSE false END),
  expires_at timestamptz NOT NULL CHECK
    (CASE WHEN isfinite(expires_at) THEN expires_at BETWEEN timestamptz '0001-01-01 00:00:00+00' AND timestamptz '9999-12-31 23:59:59.999+00' AND mod(extract(microseconds FROM expires_at),1000)=0 ELSE false END),
  policy text NOT NULL CHECK (policy ~ '^[A-Za-z0-9_.-]{1,96}$'),
  control_revision bigint NOT NULL CHECK (control_revision BETWEEN 1 AND 999999999999999999),
  state text NOT NULL CHECK (state ~ '^[A-Za-z0-9_.-]{1,96}$'),
  revoked_at timestamptz CHECK
    (CASE WHEN isfinite(revoked_at) THEN revoked_at BETWEEN timestamptz '0001-01-01 00:00:00+00' AND timestamptz '9999-12-31 23:59:59.999+00' AND mod(extract(microseconds FROM revoked_at),1000)=0 ELSE revoked_at IS NULL END),
  lock_guard boolean NOT NULL DEFAULT false CHECK (NOT lock_guard),
  PRIMARY KEY (organization_id,domain_id,grant_id),
  FOREIGN KEY (organization_id,domain_id) REFERENCES arboresce_authority.domains,
  FOREIGN KEY (issuer_id,issuer_kind) REFERENCES arboresce_authority.principals (principal_id,kind),
  FOREIGN KEY (subject_id,subject_kind) REFERENCES arboresce_authority.principals (principal_id,kind),
  CONSTRAINT grants_parent_fkey FOREIGN KEY (organization_id,domain_id,parent_grant_id)
    REFERENCES arboresce_authority.grants (organization_id,domain_id,grant_id),
  CONSTRAINT grants_scope_resource_fkey FOREIGN KEY (scope_organization_id,scope_domain_id,scope_kind,scope_resource_id)
    REFERENCES arboresce_authority.resources (organization_id,domain_id,resource_kind,resource_id),
  CONSTRAINT grants_scope_tenant CHECK (scope_organization_id=organization_id AND scope_domain_id=domain_id),
  CONSTRAINT grants_scope_shape CHECK ((scope_kind='workspace' AND scope_resource_id IS NULL) OR (scope_kind IN ('project','expense') AND scope_resource_id IS NOT NULL)),
  CONSTRAINT grants_lifetime CHECK (issued_at<expires_at)
);
CREATE TABLE arboresce_authority.immutable_chains (
  organization_id arboresce_authority.identifier NOT NULL,
  domain_id arboresce_authority.identifier NOT NULL,
  chain_id arboresce_authority.identifier NOT NULL UNIQUE,
  scope_interpretation text NOT NULL CHECK (scope_interpretation ~ '^[A-Za-z0-9_.-]{1,96}$'),
  lock_guard boolean NOT NULL DEFAULT false CHECK (NOT lock_guard),
  PRIMARY KEY (organization_id,domain_id,chain_id),
  FOREIGN KEY (organization_id,domain_id) REFERENCES arboresce_authority.domains
);
CREATE TABLE arboresce_authority.immutable_members (
  organization_id arboresce_authority.identifier NOT NULL,
  domain_id arboresce_authority.identifier NOT NULL,
  chain_id arboresce_authority.identifier NOT NULL,
  ordinal smallint NOT NULL CHECK (ordinal BETWEEN 0 AND 7),
  grant_id arboresce_authority.identifier NOT NULL,
  issuer_id arboresce_authority.identifier NOT NULL,
  issuer_kind text NOT NULL,
  subject_id arboresce_authority.identifier NOT NULL,
  subject_kind text NOT NULL,
  parent_grant_id arboresce_authority.identifier,
  scope_organization_id arboresce_authority.identifier NOT NULL,
  scope_domain_id arboresce_authority.identifier NOT NULL,
  scope_kind text NOT NULL CHECK (scope_kind IN ('project','expense','workspace')),
  scope_resource_id arboresce_authority.identifier,
  actions text[] NOT NULL CHECK (cardinality(actions)<=16 AND (cardinality(actions)=0 OR (array_ndims(actions)=1 AND array_lower(actions,1)=1)) AND array_to_json(actions)::text ~ '^\[("[A-Za-z0-9_.-]{1,96}"(,"[A-Za-z0-9_.-]{1,96}")*)?\]$'),
  purposes text[] NOT NULL CHECK (cardinality(purposes)<=16 AND (cardinality(purposes)=0 OR (array_ndims(purposes)=1 AND array_lower(purposes,1)=1)) AND array_to_json(purposes)::text ~ '^\[("[A-Za-z0-9_.-]{1,96}"(,"[A-Za-z0-9_.-]{1,96}")*)?\]$'),
  delegable boolean NOT NULL,
  issued_at timestamptz NOT NULL CHECK
    (CASE WHEN isfinite(issued_at) THEN issued_at BETWEEN timestamptz '0001-01-01 00:00:00+00' AND timestamptz '9999-12-31 23:59:59.999+00' AND mod(extract(microseconds FROM issued_at),1000)=0 ELSE false END),
  expires_at timestamptz NOT NULL CHECK
    (CASE WHEN isfinite(expires_at) THEN expires_at BETWEEN timestamptz '0001-01-01 00:00:00+00' AND timestamptz '9999-12-31 23:59:59.999+00' AND mod(extract(microseconds FROM expires_at),1000)=0 ELSE false END),
  policy text NOT NULL CHECK (policy ~ '^[A-Za-z0-9_.-]{1,96}$'),
  control_revision bigint NOT NULL CHECK (control_revision BETWEEN 1 AND 999999999999999999),
  state text NOT NULL CHECK (state ~ '^[A-Za-z0-9_.-]{1,96}$'),
  revoked_at timestamptz CHECK
    (CASE WHEN isfinite(revoked_at) THEN revoked_at BETWEEN timestamptz '0001-01-01 00:00:00+00' AND timestamptz '9999-12-31 23:59:59.999+00' AND mod(extract(microseconds FROM revoked_at),1000)=0 ELSE revoked_at IS NULL END),
  lock_guard boolean NOT NULL DEFAULT false CHECK (NOT lock_guard),
  PRIMARY KEY (organization_id,domain_id,chain_id,ordinal),
  CONSTRAINT immutable_members_chain_grant_key UNIQUE (organization_id,domain_id,chain_id,grant_id),
  CONSTRAINT immutable_members_chain_fkey FOREIGN KEY (organization_id,domain_id,chain_id)
    REFERENCES arboresce_authority.immutable_chains (organization_id,domain_id,chain_id),
  CONSTRAINT immutable_members_grant_fkey FOREIGN KEY (organization_id,domain_id,grant_id)
    REFERENCES arboresce_authority.grants (organization_id,domain_id,grant_id),
  FOREIGN KEY (issuer_id,issuer_kind) REFERENCES arboresce_authority.principals (principal_id,kind),
  FOREIGN KEY (subject_id,subject_kind) REFERENCES arboresce_authority.principals (principal_id,kind),
  CONSTRAINT immutable_members_scope_resource_fkey FOREIGN KEY (scope_organization_id,scope_domain_id,scope_kind,scope_resource_id)
    REFERENCES arboresce_authority.resources (organization_id,domain_id,resource_kind,resource_id),
  CONSTRAINT immutable_members_scope_tenant CHECK (scope_organization_id=organization_id AND scope_domain_id=domain_id),
  CONSTRAINT immutable_members_scope_shape CHECK ((scope_kind='workspace' AND scope_resource_id IS NULL) OR (scope_kind IN ('project','expense') AND scope_resource_id IS NOT NULL)),
  CONSTRAINT immutable_members_lifetime CHECK (issued_at<expires_at)
);
CREATE TABLE arboresce_authority.qualification_markers (
  organization_id arboresce_authority.identifier NOT NULL,
  domain_id arboresce_authority.identifier NOT NULL,
  marker_id arboresce_authority.identifier NOT NULL,
  observation_id arboresce_authority.identifier NOT NULL UNIQUE,
  executor_principal_id arboresce_authority.identifier NOT NULL REFERENCES arboresce_authority.principals,
  selected_chain_id arboresce_authority.identifier,
  action text NOT NULL CHECK (action ~ '^[A-Za-z0-9_.-]{1,96}$'),
  purpose text NOT NULL CHECK (purpose ~ '^[A-Za-z0-9_.-]{1,96}$'),
  resource_kind text NOT NULL CHECK (resource_kind ~ '^[A-Za-z0-9_.-]{1,96}$'),
  resource_id arboresce_authority.identifier NOT NULL,
  admitted_policy_epoch bigint NOT NULL CHECK (admitted_policy_epoch BETWEEN 1 AND 999999999999999999),
  admitted_membership_revision bigint NOT NULL CHECK (admitted_membership_revision BETWEEN 1 AND 999999999999999999),
  admitted_grant_revision bigint CHECK (admitted_grant_revision BETWEEN 1 AND 999999999999999999),
  admitted_resource_control bigint NOT NULL CHECK (admitted_resource_control BETWEEN 1 AND 999999999999999999),
  PRIMARY KEY (organization_id,domain_id,marker_id),
  FOREIGN KEY (organization_id,domain_id,resource_kind,resource_id)
    REFERENCES arboresce_authority.resources (organization_id,domain_id,resource_kind,resource_id),
  FOREIGN KEY (organization_id,domain_id,selected_chain_id)
    REFERENCES arboresce_authority.immutable_chains (organization_id,domain_id,chain_id)
);

-- Fixed migration logic owns these predicates; no fixture is executed as DDL.
-- Old PUBLIC policies remain. Restrictive context policies AND with that OR.
DO $authority_policies$
DECLARE
  relation_name text;
  selected text;
  complete_context CONSTANT text := $expr$
    nullif(current_setting('arboresce.organization_id',true),'') IS NOT NULL
    AND nullif(current_setting('arboresce.domain_id',true),'') IS NOT NULL
    AND nullif(current_setting('arboresce.principal_id',true),'') IS NOT NULL
  $expr$;
  selected_tenant CONSTANT text := $expr$
    organization_id::text=nullif(current_setting('arboresce.organization_id',true),'')
    AND domain_id::text=nullif(current_setting('arboresce.domain_id',true),'')
  $expr$;
  admission_lock_only boolean;
  permission_lock_only boolean;
BEGIN
  FOREACH relation_name IN ARRAY ARRAY['organizations','domains','principals','memberships',
    'namespace_guard','workspace_policies','source_policies','resources','party_aliases',
    'grants','immutable_chains','immutable_members','qualification_markers'] LOOP
    selected := CASE relation_name
      WHEN 'namespace_guard' THEN complete_context
      WHEN 'organizations' THEN $expr$
        organization_id::text=nullif(current_setting('arboresce.organization_id',true),'')
      $expr$
      WHEN 'principals' THEN $expr$
        principal_id::text=nullif(current_setting('arboresce.principal_id',true),'') OR EXISTS
        (SELECT 1 FROM arboresce_authority.immutable_members im
         WHERE im.organization_id::text=nullif(current_setting('arboresce.organization_id',true),'')
           AND im.domain_id::text=nullif(current_setting('arboresce.domain_id',true),'')
           AND im.chain_id::text=nullif(current_setting('arboresce.chain_id',true),'')
           AND (principals.principal_id=im.issuer_id OR principals.principal_id=im.subject_id))
      $expr$
      WHEN 'memberships' THEN $expr$
        organization_id::text=nullif(current_setting('arboresce.organization_id',true),'')
        AND (principal_id::text=nullif(current_setting('arboresce.principal_id',true),'') OR EXISTS
        (SELECT 1 FROM arboresce_authority.immutable_members im
         WHERE im.organization_id::text=nullif(current_setting('arboresce.organization_id',true),'')
           AND im.domain_id::text=nullif(current_setting('arboresce.domain_id',true),'')
           AND im.chain_id::text=nullif(current_setting('arboresce.chain_id',true),'')
           AND (memberships.principal_id=im.issuer_id OR memberships.principal_id=im.subject_id)))
      $expr$
      WHEN 'source_policies' THEN $expr$
        EXISTS (SELECT 1 FROM arboresce_authority.workspace_policies wp
         WHERE wp.organization_id::text=nullif(current_setting('arboresce.organization_id',true),'')
           AND wp.domain_id::text=nullif(current_setting('arboresce.domain_id',true),'')
           AND source_policies.policy_id=ANY(wp.source_policy_ids))
      $expr$
      WHEN 'qualification_markers' THEN $expr$
        organization_id::text=nullif(current_setting('arboresce.organization_id',true),'')
        AND domain_id::text=nullif(current_setting('arboresce.domain_id',true),'')
        AND executor_principal_id::text=nullif(current_setting('arboresce.principal_id',true),'')
      $expr$
      ELSE selected_tenant
    END;
    EXECUTE format('ALTER TABLE arboresce_authority.%I ENABLE ROW LEVEL SECURITY',relation_name);
    EXECUTE format('ALTER TABLE arboresce_authority.%I FORCE ROW LEVEL SECURITY',relation_name);
    EXECUTE format('CREATE POLICY internal_complete_context ON arboresce_authority.%I
      AS RESTRICTIVE FOR ALL TO harness_authority_admission,harness_authority_permission
      USING (%s) WITH CHECK (%s)',relation_name,complete_context,complete_context);
    EXECUTE format('CREATE POLICY internal_selected_read ON arboresce_authority.%I
      FOR SELECT TO %s USING (%s)',relation_name,
      CASE WHEN relation_name='qualification_markers' THEN 'harness_authority_admission'
        ELSE 'harness_authority_admission,harness_authority_permission' END,selected);
    IF relation_name NOT IN ('party_aliases','qualification_markers') THEN
      admission_lock_only := relation_name<>'resources';
      permission_lock_only := relation_name IN
        ('organizations','domains','principals','source_policies','immutable_chains','immutable_members');
      EXECUTE format('CREATE POLICY admission_selected_lock_or_control ON arboresce_authority.%I
        FOR UPDATE TO harness_authority_admission USING (%s) WITH CHECK (%s)',
        relation_name,selected,CASE WHEN admission_lock_only THEN 'false' ELSE selected END);
      EXECUTE format('CREATE POLICY permission_selected_lock_or_change ON arboresce_authority.%I
        FOR UPDATE TO harness_authority_permission USING (%s) WITH CHECK (%s)',
        relation_name,selected,CASE WHEN permission_lock_only THEN 'false' ELSE selected END);
      IF admission_lock_only THEN
        EXECUTE format('CREATE POLICY admission_lock_only_check ON arboresce_authority.%I
          AS RESTRICTIVE FOR UPDATE TO harness_authority_admission USING (%s) WITH CHECK (false)',relation_name,selected);
      END IF;
      IF permission_lock_only THEN
        EXECUTE format('CREATE POLICY permission_lock_only_check ON arboresce_authority.%I
          AS RESTRICTIVE FOR UPDATE TO harness_authority_permission USING (%s) WITH CHECK (false)',relation_name,selected);
      END IF;
    END IF;
  END LOOP;
END
$authority_policies$;

CREATE POLICY admission_scoped_marker ON arboresce_authority.qualification_markers
  FOR INSERT TO harness_authority_admission WITH CHECK
  (organization_id::text=nullif(current_setting('arboresce.organization_id',true),'')
   AND domain_id::text=nullif(current_setting('arboresce.domain_id',true),'')
   AND executor_principal_id::text=nullif(current_setting('arboresce.principal_id',true),''));

GRANT USAGE ON SCHEMA arboresce_authority TO harness_authority_admission,harness_authority_permission;
GRANT SELECT ON arboresce_authority.organizations,arboresce_authority.domains,
  arboresce_authority.principals,arboresce_authority.memberships,
  arboresce_authority.namespace_guard,arboresce_authority.workspace_policies,
  arboresce_authority.source_policies,arboresce_authority.resources,
  arboresce_authority.party_aliases,arboresce_authority.grants,
  arboresce_authority.immutable_chains,arboresce_authority.immutable_members,
  arboresce_authority.qualification_markers TO harness_authority_admission;
GRANT SELECT ON arboresce_authority.organizations,arboresce_authority.domains,
  arboresce_authority.principals,arboresce_authority.memberships,
  arboresce_authority.namespace_guard,arboresce_authority.workspace_policies,
  arboresce_authority.source_policies,arboresce_authority.resources,
  arboresce_authority.party_aliases,arboresce_authority.grants,
  arboresce_authority.immutable_chains,arboresce_authority.immutable_members
  TO harness_authority_permission;
GRANT UPDATE (lock_guard) ON arboresce_authority.organizations,
  arboresce_authority.domains,arboresce_authority.principals,arboresce_authority.memberships,
  arboresce_authority.namespace_guard,arboresce_authority.workspace_policies,
  arboresce_authority.source_policies,arboresce_authority.grants,
  arboresce_authority.immutable_chains,arboresce_authority.immutable_members TO harness_authority_admission;
GRANT UPDATE (lock_guard) ON arboresce_authority.organizations,
  arboresce_authority.domains,arboresce_authority.principals,arboresce_authority.source_policies,
  arboresce_authority.immutable_chains,arboresce_authority.immutable_members TO harness_authority_permission;
GRANT UPDATE (epoch) ON arboresce_authority.namespace_guard TO harness_authority_permission;
GRANT UPDATE (state,control_revision) ON arboresce_authority.memberships TO harness_authority_permission;
GRANT UPDATE (state,control_revision,revoked_at) ON arboresce_authority.grants TO harness_authority_permission;
GRANT UPDATE (state,epoch,purposes) ON arboresce_authority.workspace_policies TO harness_authority_permission;
GRANT UPDATE (control_revision) ON arboresce_authority.resources TO harness_authority_admission,harness_authority_permission;
GRANT INSERT (organization_id,domain_id,marker_id,observation_id,executor_principal_id,
  selected_chain_id,action,purpose,resource_kind,resource_id,admitted_policy_epoch,
  admitted_membership_revision,admitted_grant_revision,admitted_resource_control)
  ON arboresce_authority.qualification_markers TO harness_authority_admission;
