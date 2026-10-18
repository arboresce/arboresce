//! Actual constrained database probes; recipes select compiled statements only.

use crate::{
    authority_probes,
    authority_service::{
        Result, Service, preserve_cleanup, query_json, with_connection, with_rollback,
    },
    shared_admission::{self as shared, Samples},
    shared_seed::text,
};
use serde_json::{Value, json};
use sqlx::Connection;

fn resource_view(value: arboresce_application::policy::admission::Resources) -> Value {
    json!({"pending":value.pending,"active":value.active,"storage_bytes":value.storage_bytes,
        "memory_bytes":value.memory_bytes,"cost_microusd":value.cost_microusd})
}

fn global_view(value: arboresce_application::policy::admission::GlobalResources) -> Value {
    json!({"active":value.active,"memory_bytes":value.memory_bytes})
}

fn state_name(value: arboresce_application::policy::admission::ReservationState) -> &'static str {
    use arboresce_application::policy::admission::ReservationState;
    match value {
        ReservationState::Reserved => "reserved",
        ReservationState::Active => "active",
        ReservationState::ReconciliationRequired => "reconciliation_required",
        ReservationState::Released => "released",
    }
}

fn recognition_view(value: &crate::implementation::shared::rows::Recognition) -> Value {
    json!({"organization_id":value.organization.to_string(),"reservation_id":value.reservation.to_string(),
        "principal_id":value.principal.to_string(),"owner":value.owner,"replica":value.replica,
        "fence":value.fence,"attempt_limit":value.attempt_limit,"attempts_used":value.attempts_used,
        "total_deadline":value.total_deadline.to_string(),"admitted_at":value.admitted_at.to_string(),
        "last_observed_at":value.last_observed_at.to_string(),"state":state_name(value.state)})
}

fn live_view(value: arboresce_application::policy::admission::ReservationLease<'_>) -> Value {
    use arboresce_application::policy::admission::DispatchOutcome;
    let claim = value.claim;
    let outcome = value.dispatch_outcome.map(|v| match v {
        DispatchOutcome::NotDispatched => "not_dispatched",
        DispatchOutcome::Unknown => "unknown",
        DispatchOutcome::AcceptedRunning => "accepted_running",
        DispatchOutcome::Completed => "completed",
    });
    json!({"organization_id":claim.organization.to_string(),"reservation_id":claim.reservation.to_string(),
        "principal_id":claim.principal.to_string(),"owner":claim.owner,"replica":claim.replica,
        "fence":claim.fence,"attempt_limit":claim.attempt_limit,"attempts_used":value.attempts_used,
        "total_deadline":claim.total_deadline.to_string(),"admitted_at":value.admitted_at.to_string(),
        "last_observed_at":value.last_observed_at.to_string(),"state":state_name(value.state),
        "pending":claim.resources.pending,"active":claim.resources.active,"storage_bytes":claim.resources.storage_bytes,
        "memory_bytes":claim.resources.memory_bytes,"cost_microusd":claim.resources.cost_microusd,
        "expires_at":claim.expires_at.to_string(),"dispatch_outcome":outcome})
}

const RECOGNITION_CHANNEL: &str = "SELECT * FROM jsonb_to_record($1::text::jsonb) AS r(organization_id text,reservation_id text, \
     principal_id text,owner text,replica text,fence bigint,attempt_limit bigint,attempts_used bigint, \
     total_deadline text,admitted_at text,last_observed_at text,state text)";

/// Actual read-only PostgreSQL rows enter the SAME production decoder bodies.
/// Only decoder failures become finite observations; SQL/transport/close errors propagate.
pub async fn decoder(service: &Service, input: &Value) -> Result<Value> {
    use crate::implementation::shared::rows;
    let mut observed = with_connection(service.admin(false).await?, async |connection| {
        let backend: i32 = sqlx::query_scalar("SELECT pg_backend_pid()").fetch_one(&mut *connection).await?;
        let kind = text(input, "kind")?;
        let decoded = match kind {
            "budgets" => {
                let global = sqlx::query(
                    "SELECT * FROM jsonb_to_record($1::text::jsonb) AS r(active_limit bigint, \
                     memory_limit bigint,active_used bigint,memory_used bigint,next_reservation_order bigint,last_observed_at text)"
                ).bind(input["global"].to_string()).fetch_one(&mut *connection).await?;
                let organization = sqlx::query(
                    "SELECT * FROM jsonb_to_record($1::text::jsonb) AS r(pending_limit bigint,active_limit bigint, \
                     storage_limit bigint,memory_limit bigint,cost_limit bigint,pending_used bigint,active_used bigint, \
                     storage_used bigint,memory_used bigint,cost_used bigint)"
                ).bind(input["organization"].to_string()).fetch_one(&mut *connection).await?;
                rows::decode_budgets(&global, &organization).map(|value| {
                    json!({"decoded":"budgets","organization_limits":resource_view(value.capacity.organization_limits),
                        "organization_used":resource_view(value.capacity.organization_used),
                        "global_limits":global_view(value.capacity.global_limits),
                        "global_used":global_view(value.capacity.global_used),
                        "observed_at":value.observed_at.to_string(),"next_reservation_order":value.next_order})
                })
            }
            "recognition" => {
                let row = sqlx::query(RECOGNITION_CHANNEL).bind(input["recognition"].to_string())
                    .fetch_one(&mut *connection).await?;
                rows::decode_recognition(&row).map(|value| json!({"decoded":"recognition","recognition":recognition_view(&value)}))
            }
            "live" => {
                let recognition = sqlx::query(RECOGNITION_CHANNEL).bind(input["recognition"].to_string())
                    .fetch_one(&mut *connection).await?;
                let lease = sqlx::query(
                    "SELECT * FROM jsonb_to_record($1::text::jsonb) AS r(pending bigint,active bigint,storage_bytes bigint, \
                     memory_bytes bigint,cost_microusd bigint,expires_at text,dispatch_outcome text,outcome_present boolean)"
                ).bind(input["lease"].to_string()).fetch_one(&mut *connection).await?;
                rows::decode_recognition(&recognition).and_then(|value| rows::decode_live(&lease,value))
                    .map(|value| json!({"decoded":"live","lease":live_view(value.view())}))
            }
            _ => return Err("unregistered shared SQL decoder channel".into()),
        };
        let semantic = match decoded {
            Ok(value) => value,
            Err(error) => json!({"failure":error.to_string()}),
        };
        Ok(json!({"semantic":semantic,"evidence":{"channel":kind,"backend_id":backend,
            "actual_parameterized_driver_rows":true}}))
    }).await?;
    observed["evidence"]["census"] = crate::current_authority::census(service).await?;
    Ok(observed)
}

fn fault(recipe: &str) -> Result<(&'static str, &'static str, bool)> {
    Ok(match recipe {
        "fault-role-superuser" => (
            "ALTER ROLE harness_authority_admission SUPERUSER",
            "ALTER ROLE harness_authority_admission NOSUPERUSER",
            true,
        ),
        "fault-role-bypassrls" => (
            "ALTER ROLE harness_authority_admission BYPASSRLS",
            "ALTER ROLE harness_authority_admission NOBYPASSRLS",
            true,
        ),
        "fault-role-inherit" => (
            "ALTER ROLE harness_authority_admission INHERIT",
            "ALTER ROLE harness_authority_admission NOINHERIT",
            true,
        ),
        "fault-role-createdb" => (
            "ALTER ROLE harness_authority_admission CREATEDB",
            "ALTER ROLE harness_authority_admission NOCREATEDB",
            true,
        ),
        "fault-role-createrole" => (
            "ALTER ROLE harness_authority_admission CREATEROLE",
            "ALTER ROLE harness_authority_admission NOCREATEROLE",
            true,
        ),
        "fault-role-replication" => (
            "ALTER ROLE harness_authority_admission REPLICATION",
            "ALTER ROLE harness_authority_admission NOREPLICATION",
            true,
        ),
        "fault-role-membership" => (
            "GRANT harness_migrator TO harness_authority_admission",
            "REVOKE harness_migrator FROM harness_authority_admission",
            true,
        ),
        "fault-database-temporary" => (
            "GRANT TEMPORARY ON DATABASE arboresce_migration_harness TO harness_authority_admission",
            "REVOKE TEMPORARY ON DATABASE arboresce_migration_harness FROM harness_authority_admission",
            true,
        ),
        "fault-schema-create" => (
            "GRANT CREATE ON SCHEMA arboresce_admission TO harness_authority_admission",
            "REVOKE CREATE ON SCHEMA arboresce_admission FROM harness_authority_admission",
            false,
        ),
        "fault-global-limit-update-privilege" => (
            "GRANT UPDATE (active_limit) ON arboresce_admission.global_budget TO harness_authority_admission",
            "REVOKE UPDATE (active_limit) ON arboresce_admission.global_budget FROM harness_authority_admission",
            false,
        ),
        "fault-reservation-payload-update-privilege" => (
            "GRANT UPDATE (cost_microusd) ON arboresce_admission.reservations TO harness_authority_admission",
            "REVOKE UPDATE (cost_microusd) ON arboresce_admission.reservations FROM harness_authority_admission",
            false,
        ),
        "fault-recognition-delete-privilege" => (
            "GRANT DELETE ON arboresce_admission.reservation_recognition TO harness_authority_admission",
            "REVOKE DELETE ON arboresce_admission.reservation_recognition FROM harness_authority_admission",
            false,
        ),
        "fault-organization-rls-disabled" => (
            "ALTER TABLE arboresce_admission.organization_budgets DISABLE ROW LEVEL SECURITY",
            "ALTER TABLE arboresce_admission.organization_budgets ENABLE ROW LEVEL SECURITY",
            false,
        ),
        "fault-reservation-force-rls-disabled" => (
            "ALTER TABLE arboresce_admission.reservations NO FORCE ROW LEVEL SECURITY",
            "ALTER TABLE arboresce_admission.reservations FORCE ROW LEVEL SECURITY",
            false,
        ),
        "fault-recognition-policy-missing" => (
            "DROP POLICY complete_context ON arboresce_admission.reservation_recognition",
            "CREATE POLICY complete_context ON arboresce_admission.reservation_recognition AS RESTRICTIVE FOR ALL TO harness_authority_admission USING (nullif(current_setting('arboresce.organization_id',true),'') IS NOT NULL AND nullif(current_setting('arboresce.domain_id',true),'') IS NOT NULL AND nullif(current_setting('arboresce.principal_id',true),'') IS NOT NULL) WITH CHECK (nullif(current_setting('arboresce.organization_id',true),'') IS NOT NULL AND nullif(current_setting('arboresce.domain_id',true),'') IS NOT NULL AND nullif(current_setting('arboresce.principal_id',true),'') IS NOT NULL)",
            false,
        ),
        "fault-reservation-identity-fkey-missing" => (
            "ALTER TABLE arboresce_admission.reservations DROP CONSTRAINT reservations_fkey_1",
            "ALTER TABLE arboresce_admission.reservations ADD CONSTRAINT reservations_fkey_1 FOREIGN KEY (organization_id,reservation_id,principal_id,owner,replica,fence,attempt_limit,total_deadline,admitted_at,state) REFERENCES arboresce_admission.reservation_recognition (organization_id,reservation_id,principal_id,owner,replica,fence,attempt_limit,total_deadline,admitted_at,state) ON UPDATE CASCADE",
            false,
        ),
        "fault-quota-check-missing" => (
            "ALTER TABLE arboresce_admission.organization_budgets DROP CONSTRAINT organization_budgets_active_used_check",
            "ALTER TABLE arboresce_admission.organization_budgets ADD CONSTRAINT organization_budgets_active_used_check CHECK (active_used>=0)",
            false,
        ),
        "fault-owner-check-changed" => (
            "ALTER TABLE arboresce_admission.reservation_recognition DROP CONSTRAINT reservation_recognition_owner_check; ALTER TABLE arboresce_admission.reservation_recognition ADD CONSTRAINT reservation_recognition_owner_check CHECK (length(owner)<=96)",
            "ALTER TABLE arboresce_admission.reservation_recognition DROP CONSTRAINT reservation_recognition_owner_check; ALTER TABLE arboresce_admission.reservation_recognition ADD CONSTRAINT reservation_recognition_owner_check CHECK (owner ~ '^[A-Za-z0-9_.-]{1,96}$')",
            false,
        ),
        "fault-unexpected-relation" => (
            "CREATE TABLE arboresce_admission.unexpected_relation (value bigint)",
            "DROP TABLE arboresce_admission.unexpected_relation",
            false,
        ),
        "fault-unexpected-function" => (
            "CREATE FUNCTION arboresce_admission.unexpected_function() RETURNS integer LANGUAGE sql IMMUTABLE AS 'SELECT 1'",
            "DROP FUNCTION arboresce_admission.unexpected_function()",
            false,
        ),
        _ => return Err("unregistered shared catalogue fault".into()),
    })
}

async fn statement(service: &Service, control: bool, sql: &'static str) -> Result<()> {
    service
        .with_admin(control, async |connection| {
            sqlx::raw_sql(sql).execute(connection).await?;
            Ok(())
        })
        .await
}

pub async fn fault_claim(service: &Service, common: &Value, input: &Value) -> Result<Value> {
    let (apply, restore, control) = fault(text(input, "recipe")?)?;
    statement(service, control, apply).await?;
    let operation =
        shared::one_claim(&service.harness.authority_sessions(), common, input, false).await;
    preserve_cleanup(operation, statement(service, control, restore).await)
}

pub async fn retained(service: &Service, common: &Value, input: &Value) -> Result<Value> {
    let sessions = service.harness.authority_sessions();
    let mut session = sessions.admission().await?;
    sqlx::query("SELECT set_config('arboresce.principal_id',$1,false)")
        .bind(text(&input["caller"], "principal_id")?)
        .execute(
            session
                .connection
                .as_mut()
                .ok_or("absent owned admission session")?,
        )
        .await?;
    let mut clock = Samples::sequence(&input["clock_sequence"])?;
    let operation = shared::reserve(&mut session, common, input, &mut clock, false).await;
    let cleanup = session.close().await.map_err(Into::into);
    let result = preserve_cleanup(operation, cleanup)?;
    Ok(json!({"result":result}))
}

pub async fn permission(service: &Service, common: &Value, input: &Value) -> Result<Value> {
    let sessions = service.harness.authority_sessions();
    let mut session = sessions.permission().await?;
    let mut clock = Samples::sequence(&input["clock_sequence"])?;
    let operation = shared::reserve(&mut session.inner, common, input, &mut clock, false).await;
    let cleanup = session.close().await.map_err(Into::into);
    Ok(json!({"result":preserve_cleanup(operation,cleanup)?}))
}

pub async fn revoke(service: &Service, input: &Value, recipe: &str) -> Result<()> {
    service.with_admin(false,async |connection| {
        let caller=&input["caller"];
        let result=match recipe {
            "unknown-membership-policy"=>sqlx::query("UPDATE arboresce_authority.memberships SET policy=$3,control_revision=control_revision+1 WHERE organization_id=$1::uuid AND principal_id=$2::uuid")
                .bind(text(caller,"organization_id")?).bind(text(caller,"principal_id")?)
                .bind(text(input,"stored_membership_policy_override")?).execute(connection).await?,
            "revoked-principal"=>sqlx::query("UPDATE arboresce_authority.principals SET state='revoked',revoked_at=$2::timestamptz,control_revision=control_revision+1 WHERE principal_id=$1::uuid")
                .bind(text(caller,"principal_id")?).bind(text(input,"now")?).execute(connection).await?,
            "revoked-organization"=>sqlx::query("UPDATE arboresce_authority.organizations SET state='revoked' WHERE organization_id=$1::uuid")
                .bind(text(caller,"organization_id")?).execute(connection).await?,
            "revoked-domain"=>sqlx::query("UPDATE arboresce_authority.domains SET state='revoked' WHERE organization_id=$1::uuid AND domain_id=$2::uuid")
                .bind(text(caller,"organization_id")?).bind(text(caller,"domain_id")?).execute(connection).await?,
            "revoked-membership-proved-cleanup"|"revoked-membership-unknown-cleanup"=>sqlx::query("UPDATE arboresce_authority.memberships SET state='revoked',control_revision=control_revision+1 WHERE organization_id=$1::uuid AND principal_id=$2::uuid")
                .bind(text(caller,"organization_id")?).bind(text(caller,"principal_id")?).execute(connection).await?,
            _=>return Err("unregistered shared revocation".into()),
        };
        if result.rows_affected()!=1 {return Err("shared revocation did not affect exactly one canonical row".into());}
        Ok(())
    }).await
}

pub struct PreparedRls<'a> {
    sql: String,
    values: Vec<String>,
    organization: &'a str,
    domain: &'a str,
    principal: &'a str,
}

/// Select a closed query and complete contexts before any database observation.
pub fn prepare_rls<'a>(common: &'a Value, input: &'a Value) -> Result<PreparedRls<'a>> {
    let relation = text(input, "relation")?;
    let (projection, key) = match relation {
        "organization_budgets" => (
            "SELECT to_jsonb(b) AS value FROM arboresce_admission.organization_budgets b WHERE organization_id=$1::uuid",
            "organization",
        ),
        "reservations" => (
            "SELECT to_jsonb(b) AS value FROM arboresce_admission.reservations b WHERE organization_id=$1::uuid AND reservation_id=$2::uuid AND principal_id=$3::uuid",
            "reservation",
        ),
        "reservation_recognition" => (
            "SELECT to_jsonb(b) AS value FROM arboresce_admission.reservation_recognition b WHERE organization_id=$1::uuid AND reservation_id=$2::uuid AND principal_id=$3::uuid",
            "recognition",
        ),
        _ => return Err("unregistered shared RLS relation".into()),
    };
    let sql =
        format!("SELECT coalesce(jsonb_agg(value),'[]'::jsonb)::text FROM ({projection}) rows");
    let mut values = vec![text(input, "selected_organization_id")?.to_owned()];
    if key != "organization" {
        values.push(text(input, "selected_reservation_id")?.to_owned());
        values.push(text(input, "selected_principal_id")?.to_owned());
    }
    let selected = text(input, "selected_organization_id")?;
    let mut organization = selected;
    let mut domain = common["selected_domains"][selected]
        .as_str()
        .ok_or("missing shared selected domain")?;
    let mut principal = text(input, "selected_principal_id")?;
    match text(input, "recipe")? {
        "raw-rls-missing-organization-context" => organization = "",
        "raw-rls-missing-domain-context" => domain = "",
        "raw-rls-missing-principal-context" => principal = "",
        "raw-rls-foreign-tenant-context" => {
            let foreign = common["selected_domains"]
                .as_object()
                .ok_or("invalid selected domains")?
                .iter()
                .find(|(id, _)| id.as_str() != selected)
                .ok_or("missing distinct organization")?;
            organization = foreign.0;
            domain = foreign.1.as_str().ok_or("invalid foreign domain")?;
        }
        "raw-rls-foreign-principal-context" => {
            principal = common["authority_snapshot"]["principals"]
                .as_array()
                .ok_or("missing canonical principals")?
                .iter()
                .find_map(|row| row["principal_id"].as_str().filter(|id| *id != principal))
                .ok_or("missing distinct principal")?;
        }
        _ => return Err("unregistered shared RLS context".into()),
    }
    Ok(PreparedRls {
        sql,
        values,
        organization,
        domain,
        principal,
    })
}

pub async fn rls(service: &Service, common: &Value, input: &Value) -> Result<(Value, Value)> {
    let PreparedRls {
        sql,
        values,
        organization,
        domain,
        principal,
    } = prepare_rls(common, input)?;
    let existing = service
        .json(false, sqlx::AssertSqlSafe(sql.clone()), &values, 131072)
        .await?;
    if existing.as_array().is_none_or(|v| v.len() != 1) {
        return Err("shared RLS target is not one actual existing row".into());
    }
    let scoped=with_connection(authority_probes::raw_role(service,false).await?,async |connection| {
        with_rollback(connection,async |connection| {
            sqlx::query("SELECT set_config('arboresce.organization_id',$1,true),set_config('arboresce.domain_id',$2,true),set_config('arboresce.principal_id',$3,true),set_config('arboresce.chain_id','',true)")
                .bind(organization).bind(domain).bind(principal).execute(&mut *connection).await?;
            query_json(connection,sqlx::AssertSqlSafe(sql),&values,131072).await
        }).await
    }).await?;
    let result = if scoped.as_array().is_some_and(Vec::is_empty) {
        "NoScopedRow"
    } else {
        "ScopedRow"
    };
    let count = scoped
        .as_array()
        .ok_or("invalid shared RLS observation")?
        .len();
    Ok((
        json!({"result":result,"scoped_rows":count,"selected_existing_row_required":existing.as_array().is_some_and(|v|v.len()==1)}),
        json!({"existing_rows":existing,"scoped_rows":scoped}),
    ))
}

/// A real reset refusal must conserve every canonical column and migration row.
pub async fn reset_guard(service: &Service, common: &Value) -> Result<(Value, Value)> {
    let before = json!({
        "state":shared::after(service,common).await?,
        "canonical_rows":crate::shared_seed::complete_authority(service).await?,
        "migration_history":crate::shared_seed::complete_history(service).await?
    });
    let guard = service.refuse_reset_with_owned_connection().await?;
    let after = json!({
        "state":shared::after(service,common).await?,
        "canonical_rows":crate::shared_seed::complete_authority(service).await?,
        "migration_history":crate::shared_seed::complete_history(service).await?
    });
    let semantic = json!({
        "result":"ResetRefused",
        "observed_backends":guard["observed_backends"],
        "canonical_rows_unchanged":before["canonical_rows"]==after["canonical_rows"],
        "migration_history_unchanged":before["migration_history"]==after["migration_history"],
        "owned_connection_closed":guard["owned_connection_closed"],
        "empty_database_after_close":guard["empty_database_after_close"]
    });
    Ok((
        semantic,
        json!({"reset_guard":guard,"before":before,"after":after}),
    ))
}

fn constraint_sql(recipe: &str) -> Result<&'static str> {
    Ok(match recipe {
        "database-valid-pending-increment" => {
            "UPDATE arboresce_admission.reservations SET pending=pending+1 WHERE organization_id=$1::uuid AND reservation_id=$2::uuid"
        }
        "database-negative-pending" => {
            "UPDATE arboresce_admission.reservations SET pending=-1 WHERE organization_id=$1::uuid AND reservation_id=$2::uuid"
        }
        "database-negative-active" => {
            "UPDATE arboresce_admission.reservations SET active=-1 WHERE organization_id=$1::uuid AND reservation_id=$2::uuid"
        }
        "database-negative-storage_bytes" => {
            "UPDATE arboresce_admission.reservations SET storage_bytes=-1 WHERE organization_id=$1::uuid AND reservation_id=$2::uuid"
        }
        "database-negative-memory_bytes" => {
            "UPDATE arboresce_admission.reservations SET memory_bytes=-1 WHERE organization_id=$1::uuid AND reservation_id=$2::uuid"
        }
        "database-negative-cost_microusd" => {
            "UPDATE arboresce_admission.reservations SET cost_microusd=-1 WHERE organization_id=$1::uuid AND reservation_id=$2::uuid"
        }
        "database-zero-fence" => {
            "UPDATE arboresce_admission.reservation_recognition SET fence=0 WHERE organization_id=$1::uuid AND reservation_id=$2::uuid"
        }
        "database-zero-attempt-limit" => {
            "UPDATE arboresce_admission.reservation_recognition SET attempt_limit=0 WHERE organization_id=$1::uuid AND reservation_id=$2::uuid"
        }
        "database-nonascii-owner" => {
            "UPDATE arboresce_admission.reservation_recognition SET owner='é' WHERE organization_id=$1::uuid AND reservation_id=$2::uuid"
        }
        "database-infinite-expiry" => {
            "UPDATE arboresce_admission.reservations SET expires_at='infinity'::timestamptz WHERE organization_id=$1::uuid AND reservation_id=$2::uuid"
        }
        "database-submillisecond-expiry" => {
            "UPDATE arboresce_admission.reservations SET expires_at=expires_at+interval '1 microsecond' WHERE organization_id=$1::uuid AND reservation_id=$2::uuid"
        }
        "database-expiry-before-admission" => {
            "UPDATE arboresce_admission.reservations SET expires_at=admitted_at-interval '1 millisecond' WHERE organization_id=$1::uuid AND reservation_id=$2::uuid"
        }
        "database-expiry-equals-admission" => {
            "UPDATE arboresce_admission.reservations SET expires_at=admitted_at WHERE organization_id=$1::uuid AND reservation_id=$2::uuid"
        }
        "database-expiry-after-total" => {
            "UPDATE arboresce_admission.reservations SET expires_at=total_deadline+interval '1 millisecond' WHERE organization_id=$1::uuid AND reservation_id=$2::uuid"
        }
        "database-unrecognized-state" => {
            "UPDATE arboresce_admission.reservation_recognition SET state='unrecognized' WHERE organization_id=$1::uuid AND reservation_id=$2::uuid"
        }
        "database-missing-membership" => {
            "UPDATE arboresce_admission.reservation_recognition SET principal_id='09090909-0909-4909-8909-090909090909'::uuid WHERE organization_id=$1::uuid AND reservation_id=$2::uuid"
        }
        "database-missing-recognition" => {
            "UPDATE arboresce_admission.reservations SET reservation_id='09090909-0909-4909-8909-090909090909'::uuid WHERE organization_id=$1::uuid AND reservation_id=$2::uuid"
        }
        "database-duplicate-identity" => {
            "INSERT INTO arboresce_admission.reservations SELECT * FROM arboresce_admission.reservations WHERE organization_id=$1::uuid AND reservation_id=$2::uuid"
        }
        "database-signed-overflow" => {
            "UPDATE arboresce_admission.global_budget SET active_used=9223372036854775807::bigint+1 WHERE $1::uuid IS NOT NULL AND $2::uuid IS NOT NULL"
        }
        _ => return Err("unregistered shared database constraint probe".into()),
    })
}

pub async fn constraint(service: &Service, input: &Value) -> Result<Value> {
    let row = input["ledger"]["reservations"]
        .as_array()
        .and_then(|v| v.first())
        .ok_or("missing existing constraint probe target")?;
    let sql = constraint_sql(text(input, "recipe")?)?;
    with_connection(service.admin(false).await?, async |connection| {
        let mut transaction = connection.begin().await?;
        let result = sqlx::query(sql)
            .bind(text(row, "organization_id")?)
            .bind(text(row, "reservation_id")?)
            .execute(&mut *transaction)
            .await;
        let observation = match result {
            Ok(changed) => {
                json!({"result":"AcceptedByDatabase","affected_rows":changed.rows_affected()})
            }
            Err(sqlx::Error::Database(error)) => {
                json!({"result":"RejectedByDatabase","native_sqlstate":error.code()})
            }
            Err(error) => {
                return preserve_cleanup(
                    Err(error.into()),
                    transaction.rollback().await.map_err(Into::into),
                );
            }
        };
        transaction.rollback().await?;
        Ok(observation)
    })
    .await
}
