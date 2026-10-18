-- Isolated shared resource reservations. No product or bootstrap writer.

CREATE SCHEMA arboresce_admission AUTHORIZATION harness_migrator;

REVOKE ALL ON SCHEMA arboresce_admission FROM PUBLIC;

CREATE TABLE arboresce_admission.global_budget (
  singleton smallint NOT NULL,
  active_limit bigint NOT NULL,
  memory_limit bigint NOT NULL,
  active_used bigint NOT NULL,
  memory_used bigint NOT NULL,
  last_observed_at timestamp with time zone NOT NULL,
  next_reservation_order bigint NOT NULL,
  CONSTRAINT global_budget_pkey PRIMARY KEY (singleton),
  CONSTRAINT global_budget_singleton_check CHECK (singleton=1),
  CONSTRAINT global_budget_active_limit_check CHECK (active_limit>=0),
  CONSTRAINT global_budget_memory_limit_check CHECK (memory_limit>=0),
  CONSTRAINT global_budget_active_used_check CHECK (active_used>=0),
  CONSTRAINT global_budget_memory_used_check CHECK (memory_used>=0),
  CONSTRAINT global_budget_next_reservation_order_check CHECK (next_reservation_order>0),
  CONSTRAINT global_budget_last_observed_at_check CHECK (CASE WHEN isfinite(last_observed_at) THEN last_observed_at BETWEEN timestamptz '0001-01-01 00:00:00+00' AND timestamptz '9999-12-31 23:59:59.999+00' AND mod(extract(microseconds FROM last_observed_at),1000)=0 ELSE false END)
);

CREATE TABLE arboresce_admission.organization_budgets (
  organization_id arboresce_authority.identifier NOT NULL,
  pending_limit bigint NOT NULL,
  active_limit bigint NOT NULL,
  storage_limit bigint NOT NULL,
  memory_limit bigint NOT NULL,
  cost_limit bigint NOT NULL,
  pending_used bigint NOT NULL,
  active_used bigint NOT NULL,
  storage_used bigint NOT NULL,
  memory_used bigint NOT NULL,
  cost_used bigint NOT NULL,
  CONSTRAINT organization_budgets_pkey PRIMARY KEY (organization_id),
  CONSTRAINT organization_budgets_fkey_1 FOREIGN KEY (organization_id) REFERENCES arboresce_authority.organizations (organization_id),
  CONSTRAINT organization_budgets_pending_limit_check CHECK (pending_limit>=0),
  CONSTRAINT organization_budgets_pending_used_check CHECK (pending_used>=0),
  CONSTRAINT organization_budgets_active_limit_check CHECK (active_limit>=0),
  CONSTRAINT organization_budgets_active_used_check CHECK (active_used>=0),
  CONSTRAINT organization_budgets_storage_limit_check CHECK (storage_limit>=0),
  CONSTRAINT organization_budgets_storage_used_check CHECK (storage_used>=0),
  CONSTRAINT organization_budgets_memory_limit_check CHECK (memory_limit>=0),
  CONSTRAINT organization_budgets_memory_used_check CHECK (memory_used>=0),
  CONSTRAINT organization_budgets_cost_limit_check CHECK (cost_limit>=0),
  CONSTRAINT organization_budgets_cost_used_check CHECK (cost_used>=0)
);

CREATE TABLE arboresce_admission.reservation_recognition (
  organization_id arboresce_authority.identifier NOT NULL,
  reservation_id arboresce_authority.identifier NOT NULL,
  principal_id arboresce_authority.identifier NOT NULL,
  owner text COLLATE "C" NOT NULL,
  replica text COLLATE "C" NOT NULL,
  fence bigint NOT NULL,
  attempt_limit smallint NOT NULL,
  attempts_used smallint NOT NULL,
  total_deadline timestamp with time zone NOT NULL,
  admitted_at timestamp with time zone NOT NULL,
  last_observed_at timestamp with time zone NOT NULL,
  state text NOT NULL,
  reservation_order bigint NOT NULL,
  CONSTRAINT reservation_recognition_pkey PRIMARY KEY (organization_id,reservation_id),
  CONSTRAINT reservation_recognition_unique_1 UNIQUE (organization_id,reservation_id,principal_id,owner,replica,fence,attempt_limit,total_deadline,admitted_at,state),
  CONSTRAINT reservation_recognition_unique_2 UNIQUE (reservation_order),
  CONSTRAINT reservation_recognition_fkey_1 FOREIGN KEY (organization_id) REFERENCES arboresce_admission.organization_budgets (organization_id),
  CONSTRAINT reservation_recognition_fkey_2 FOREIGN KEY (organization_id,principal_id) REFERENCES arboresce_authority.memberships (organization_id,principal_id),
  CONSTRAINT reservation_recognition_reservation_order_check CHECK (reservation_order>0),
  CONSTRAINT reservation_recognition_owner_check CHECK (owner ~ '^[A-Za-z0-9_.-]{1,96}$'),
  CONSTRAINT reservation_recognition_replica_check CHECK (replica ~ '^[A-Za-z0-9_.-]{1,96}$'),
  CONSTRAINT reservation_recognition_fence_check CHECK (fence BETWEEN 1 AND 999999999999999999),
  CONSTRAINT reservation_recognition_attempt_limit_check CHECK (attempt_limit BETWEEN 1 AND 8),
  CONSTRAINT reservation_recognition_attempts_used_check CHECK (attempts_used BETWEEN 0 AND attempt_limit),
  CONSTRAINT reservation_recognition_state_check CHECK (state IN ('reserved','active','reconciliation_required','released')),
  CONSTRAINT reservation_recognition_total_deadline_check CHECK (CASE WHEN isfinite(total_deadline) THEN total_deadline BETWEEN timestamptz '0001-01-01 00:00:00+00' AND timestamptz '9999-12-31 23:59:59.999+00' AND mod(extract(microseconds FROM total_deadline),1000)=0 ELSE false END),
  CONSTRAINT reservation_recognition_admitted_at_check CHECK (CASE WHEN isfinite(admitted_at) THEN admitted_at BETWEEN timestamptz '0001-01-01 00:00:00+00' AND timestamptz '9999-12-31 23:59:59.999+00' AND mod(extract(microseconds FROM admitted_at),1000)=0 ELSE false END),
  CONSTRAINT reservation_recognition_last_observed_at_check CHECK (CASE WHEN isfinite(last_observed_at) THEN last_observed_at BETWEEN timestamptz '0001-01-01 00:00:00+00' AND timestamptz '9999-12-31 23:59:59.999+00' AND mod(extract(microseconds FROM last_observed_at),1000)=0 ELSE false END),
  CONSTRAINT reservation_recognition_lifetime_check CHECK (admitted_at<total_deadline AND total_deadline-admitted_at<=interval '1 hour'),
  CONSTRAINT reservation_recognition_observation_order_check CHECK (last_observed_at>=admitted_at)
);

CREATE TABLE arboresce_admission.reservations (
  organization_id arboresce_authority.identifier NOT NULL,
  reservation_id arboresce_authority.identifier NOT NULL,
  principal_id arboresce_authority.identifier NOT NULL,
  replica text COLLATE "C" NOT NULL,
  pending bigint NOT NULL,
  active bigint NOT NULL,
  storage_bytes bigint NOT NULL,
  memory_bytes bigint NOT NULL,
  cost_microusd bigint NOT NULL,
  owner text COLLATE "C" NOT NULL,
  fence bigint NOT NULL,
  admitted_at timestamp with time zone NOT NULL,
  expires_at timestamp with time zone NOT NULL,
  attempt_limit smallint NOT NULL,
  total_deadline timestamp with time zone NOT NULL,
  state text NOT NULL,
  dispatch_outcome text,
  CONSTRAINT reservations_pkey PRIMARY KEY (organization_id,reservation_id),
  CONSTRAINT reservations_fkey_1 FOREIGN KEY (organization_id,reservation_id,principal_id,owner,replica,fence,attempt_limit,total_deadline,admitted_at,state) REFERENCES arboresce_admission.reservation_recognition (organization_id,reservation_id,principal_id,owner,replica,fence,attempt_limit,total_deadline,admitted_at,state) ON UPDATE CASCADE,
  CONSTRAINT reservations_pending_check CHECK (pending>=0),
  CONSTRAINT reservations_active_check CHECK (active>=0),
  CONSTRAINT reservations_storage_bytes_check CHECK (storage_bytes>=0),
  CONSTRAINT reservations_memory_bytes_check CHECK (memory_bytes>=0),
  CONSTRAINT reservations_cost_microusd_check CHECK (cost_microusd>=0),
  CONSTRAINT reservations_owner_check CHECK (owner ~ '^[A-Za-z0-9_.-]{1,96}$'),
  CONSTRAINT reservations_replica_check CHECK (replica ~ '^[A-Za-z0-9_.-]{1,96}$'),
  CONSTRAINT reservations_fence_check CHECK (fence BETWEEN 1 AND 999999999999999999),
  CONSTRAINT reservations_attempt_limit_check CHECK (attempt_limit BETWEEN 1 AND 8),
  CONSTRAINT reservations_state_check CHECK (state IN ('reserved','active','reconciliation_required')),
  CONSTRAINT reservations_dispatch_outcome_check CHECK (dispatch_outcome IN ('accepted_running','unknown','not_dispatched','completed')),
  CONSTRAINT reservations_admitted_at_check CHECK (CASE WHEN isfinite(admitted_at) THEN admitted_at BETWEEN timestamptz '0001-01-01 00:00:00+00' AND timestamptz '9999-12-31 23:59:59.999+00' AND mod(extract(microseconds FROM admitted_at),1000)=0 ELSE false END),
  CONSTRAINT reservations_expires_at_check CHECK (CASE WHEN isfinite(expires_at) THEN expires_at BETWEEN timestamptz '0001-01-01 00:00:00+00' AND timestamptz '9999-12-31 23:59:59.999+00' AND mod(extract(microseconds FROM expires_at),1000)=0 ELSE false END),
  CONSTRAINT reservations_total_deadline_check CHECK (CASE WHEN isfinite(total_deadline) THEN total_deadline BETWEEN timestamptz '0001-01-01 00:00:00+00' AND timestamptz '9999-12-31 23:59:59.999+00' AND mod(extract(microseconds FROM total_deadline),1000)=0 ELSE false END),
  CONSTRAINT reservations_expiry_order_check CHECK (admitted_at<expires_at AND expires_at<=total_deadline),
  CONSTRAINT reservations_resource_nonzero_check CHECK (pending>0 OR active>0 OR storage_bytes>0 OR memory_bytes>0 OR cost_microusd>0)
);

REVOKE ALL ON ALL TABLES IN SCHEMA arboresce_admission FROM PUBLIC;

GRANT USAGE ON SCHEMA arboresce_admission TO harness_authority_admission;

GRANT SELECT ON arboresce_admission.global_budget TO harness_authority_admission;

GRANT UPDATE (active_used,memory_used,last_observed_at,next_reservation_order) ON arboresce_admission.global_budget TO harness_authority_admission;

GRANT SELECT ON arboresce_admission.organization_budgets TO harness_authority_admission;

GRANT UPDATE (pending_used,active_used,storage_used,memory_used,cost_used) ON arboresce_admission.organization_budgets TO harness_authority_admission;

ALTER TABLE arboresce_admission.organization_budgets ENABLE ROW LEVEL SECURITY;

ALTER TABLE arboresce_admission.organization_budgets FORCE ROW LEVEL SECURITY;

CREATE POLICY complete_context ON arboresce_admission.organization_budgets AS RESTRICTIVE FOR ALL TO harness_authority_admission USING (nullif(current_setting('arboresce.organization_id',true),'') IS NOT NULL AND nullif(current_setting('arboresce.domain_id',true),'') IS NOT NULL AND nullif(current_setting('arboresce.principal_id',true),'') IS NOT NULL) WITH CHECK (nullif(current_setting('arboresce.organization_id',true),'') IS NOT NULL AND nullif(current_setting('arboresce.domain_id',true),'') IS NOT NULL AND nullif(current_setting('arboresce.principal_id',true),'') IS NOT NULL);

CREATE POLICY selected_principal ON arboresce_admission.organization_budgets FOR ALL TO harness_authority_admission USING (organization_id::text=nullif(current_setting('arboresce.organization_id',true),'')) WITH CHECK (organization_id::text=nullif(current_setting('arboresce.organization_id',true),''));

GRANT SELECT ON arboresce_admission.reservation_recognition TO harness_authority_admission;

GRANT INSERT (organization_id,reservation_id,principal_id,owner,replica,fence,attempt_limit,attempts_used,total_deadline,admitted_at,last_observed_at,state,reservation_order) ON arboresce_admission.reservation_recognition TO harness_authority_admission;

GRANT UPDATE (attempts_used,last_observed_at,state) ON arboresce_admission.reservation_recognition TO harness_authority_admission;

ALTER TABLE arboresce_admission.reservation_recognition ENABLE ROW LEVEL SECURITY;

ALTER TABLE arboresce_admission.reservation_recognition FORCE ROW LEVEL SECURITY;

CREATE POLICY complete_context ON arboresce_admission.reservation_recognition AS RESTRICTIVE FOR ALL TO harness_authority_admission USING (nullif(current_setting('arboresce.organization_id',true),'') IS NOT NULL AND nullif(current_setting('arboresce.domain_id',true),'') IS NOT NULL AND nullif(current_setting('arboresce.principal_id',true),'') IS NOT NULL) WITH CHECK (nullif(current_setting('arboresce.organization_id',true),'') IS NOT NULL AND nullif(current_setting('arboresce.domain_id',true),'') IS NOT NULL AND nullif(current_setting('arboresce.principal_id',true),'') IS NOT NULL);

CREATE POLICY selected_principal ON arboresce_admission.reservation_recognition FOR ALL TO harness_authority_admission USING (organization_id::text=nullif(current_setting('arboresce.organization_id',true),'') AND principal_id::text=nullif(current_setting('arboresce.principal_id',true),'')) WITH CHECK (organization_id::text=nullif(current_setting('arboresce.organization_id',true),'') AND principal_id::text=nullif(current_setting('arboresce.principal_id',true),''));

GRANT SELECT ON arboresce_admission.reservations TO harness_authority_admission;

GRANT INSERT (organization_id,reservation_id,principal_id,replica,pending,active,storage_bytes,memory_bytes,cost_microusd,owner,fence,admitted_at,expires_at,attempt_limit,total_deadline,state) ON arboresce_admission.reservations TO harness_authority_admission;

GRANT UPDATE (expires_at,state) ON arboresce_admission.reservations TO harness_authority_admission;

GRANT DELETE ON arboresce_admission.reservations TO harness_authority_admission;

ALTER TABLE arboresce_admission.reservations ENABLE ROW LEVEL SECURITY;

ALTER TABLE arboresce_admission.reservations FORCE ROW LEVEL SECURITY;

CREATE POLICY complete_context ON arboresce_admission.reservations AS RESTRICTIVE FOR ALL TO harness_authority_admission USING (nullif(current_setting('arboresce.organization_id',true),'') IS NOT NULL AND nullif(current_setting('arboresce.domain_id',true),'') IS NOT NULL AND nullif(current_setting('arboresce.principal_id',true),'') IS NOT NULL) WITH CHECK (nullif(current_setting('arboresce.organization_id',true),'') IS NOT NULL AND nullif(current_setting('arboresce.domain_id',true),'') IS NOT NULL AND nullif(current_setting('arboresce.principal_id',true),'') IS NOT NULL);

CREATE POLICY selected_principal ON arboresce_admission.reservations FOR ALL TO harness_authority_admission USING (organization_id::text=nullif(current_setting('arboresce.organization_id',true),'') AND principal_id::text=nullif(current_setting('arboresce.principal_id',true),'')) WITH CHECK (organization_id::text=nullif(current_setting('arboresce.organization_id',true),'') AND principal_id::text=nullif(current_setting('arboresce.principal_id',true),''));

GRANT USAGE ON SCHEMA public TO harness_authority_admission;

GRANT SELECT (version,success,checksum) ON public._sqlx_migrations TO harness_authority_admission;
