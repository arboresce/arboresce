//! Typed bounded row reads and counter updates under the held canonical locks.

use arboresce_application::policy::{
    TenantContext,
    admission::{
        AdmissionFailure as Failure, Capacity, DispatchOutcome, GlobalResources, ReservationClaim,
        ReservationLease, ReservationState, Resources,
    },
};
use arboresce_domain::{Identifier, UtcTimestamp};
use sqlx::{PgConnection, Row, postgres::PgRow};

use super::super::authority::rows as checked;
use super::{authority_failure, database_failure};

pub(crate) struct Budgets {
    pub capacity: Capacity,
    pub observed_at: UtcTimestamp,
    pub next_order: u64,
}

fn number(row: &PgRow, name: &str) -> Result<u64, Failure> {
    let value: i64 = row.try_get(name).map_err(|_| Failure::InvalidStoredData)?;
    value.try_into().map_err(|_| Failure::InvalidStoredData)
}

pub(super) async fn budgets(
    connection: &mut PgConnection,
    caller: TenantContext,
) -> Result<Budgets, Failure> {
    let sql = format!(
        "SELECT active_limit,memory_limit,active_used,memory_used,next_reservation_order,{} \
         FROM arboresce_admission.global_budget WHERE singleton=1 FOR UPDATE",
        checked::instant("last_observed_at")
    );
    let global = sqlx::query(sqlx::AssertSqlSafe(sql))
        .fetch_optional(&mut *connection)
        .await
        .map_err(database_failure)?
        .ok_or(Failure::InvalidStoredData)?;
    let organization = sqlx::query(
        "SELECT pending_limit,active_limit,storage_limit,memory_limit,cost_limit, \
         pending_used,active_used,storage_used,memory_used,cost_used \
         FROM arboresce_admission.organization_budgets WHERE organization_id=$1::uuid FOR UPDATE",
    )
    .bind(caller.workspace().organization().to_string())
    .fetch_optional(connection)
    .await
    .map_err(database_failure)?
    .ok_or(Failure::InvalidStoredData)?;
    decode_budgets(&global, &organization)
}

pub(crate) fn decode_budgets(global: &PgRow, organization: &PgRow) -> Result<Budgets, Failure> {
    let capacity = Capacity {
        organization_limits: Resources {
            pending: number(organization, "pending_limit")?,
            active: number(organization, "active_limit")?,
            storage_bytes: number(organization, "storage_limit")?,
            memory_bytes: number(organization, "memory_limit")?,
            cost_microusd: number(organization, "cost_limit")?,
        },
        organization_used: Resources {
            pending: number(organization, "pending_used")?,
            active: number(organization, "active_used")?,
            storage_bytes: number(organization, "storage_used")?,
            memory_bytes: number(organization, "memory_used")?,
            cost_microusd: number(organization, "cost_used")?,
        },
        global_limits: GlobalResources {
            active: number(global, "active_limit")?,
            memory_bytes: number(global, "memory_limit")?,
        },
        global_used: GlobalResources {
            active: number(global, "active_used")?,
            memory_bytes: number(global, "memory_used")?,
        },
    };
    let next_order = number(global, "next_reservation_order")?;
    if next_order == 0 {
        return Err(Failure::InvalidStoredData);
    }
    Ok(Budgets {
        capacity,
        observed_at: checked::time(global, "last_observed_at").map_err(authority_failure)?,
        next_order,
    })
}

pub(crate) struct Recognition {
    pub organization: Identifier,
    pub reservation: Identifier,
    pub principal: Identifier,
    pub owner: String,
    pub replica: String,
    pub fence: u64,
    pub attempt_limit: u64,
    pub attempts_used: u64,
    pub total_deadline: UtcTimestamp,
    pub admitted_at: UtcTimestamp,
    pub last_observed_at: UtcTimestamp,
    pub state: ReservationState,
}

fn state(row: &PgRow) -> Result<ReservationState, Failure> {
    match checked::text(row, "state")
        .map_err(authority_failure)?
        .as_str()
    {
        "reserved" => Ok(ReservationState::Reserved),
        "active" => Ok(ReservationState::Active),
        "reconciliation_required" => Ok(ReservationState::ReconciliationRequired),
        "released" => Ok(ReservationState::Released),
        _ => Err(Failure::InvalidStoredData),
    }
}

pub(super) async fn recognition(
    connection: &mut PgConnection,
    caller: TenantContext,
    reservation: Identifier,
) -> Result<Option<Recognition>, Failure> {
    let sql = format!(
        "SELECT organization_id::text,reservation_id::text,principal_id::text,fence, \
         attempt_limit::bigint AS attempt_limit,attempts_used::bigint AS attempts_used, \
         {},{},{},{},{},{} FROM arboresce_admission.reservation_recognition \
         WHERE organization_id=$1::uuid AND reservation_id=$2::uuid FOR UPDATE",
        checked::token("owner"),
        checked::token("replica"),
        checked::token("state"),
        checked::instant("total_deadline"),
        checked::instant("admitted_at"),
        checked::instant("last_observed_at")
    );
    let row = sqlx::query(sqlx::AssertSqlSafe(sql))
        .bind(caller.workspace().organization().to_string())
        .bind(reservation.to_string())
        .fetch_optional(connection)
        .await
        .map_err(database_failure)?;
    row.as_ref().map(decode_recognition).transpose()
}

pub(crate) fn decode_recognition(row: &PgRow) -> Result<Recognition, Failure> {
    Ok(Recognition {
        organization: checked::id(row, "organization_id").map_err(authority_failure)?,
        reservation: checked::id(row, "reservation_id").map_err(authority_failure)?,
        principal: checked::id(row, "principal_id").map_err(authority_failure)?,
        owner: checked::text(row, "owner").map_err(authority_failure)?,
        replica: checked::text(row, "replica").map_err(authority_failure)?,
        fence: number(row, "fence")?,
        attempt_limit: number(row, "attempt_limit")?,
        attempts_used: number(row, "attempts_used")?,
        total_deadline: checked::time(row, "total_deadline").map_err(authority_failure)?,
        admitted_at: checked::time(row, "admitted_at").map_err(authority_failure)?,
        last_observed_at: checked::time(row, "last_observed_at").map_err(authority_failure)?,
        state: state(row)?,
    })
}

pub(crate) struct LiveLease {
    recognition: Recognition,
    resources: Resources,
    expires_at: UtcTimestamp,
    outcome: Option<DispatchOutcome>,
}

impl LiveLease {
    pub fn view(&self) -> ReservationLease<'_> {
        let row = &self.recognition;
        ReservationLease {
            claim: ReservationClaim {
                reservation: row.reservation,
                organization: row.organization,
                principal: row.principal,
                owner: &row.owner,
                replica: &row.replica,
                fence: row.fence,
                attempt_limit: row.attempt_limit,
                total_deadline: row.total_deadline,
                expires_at: self.expires_at,
                resources: self.resources,
            },
            admitted_at: row.admitted_at,
            last_observed_at: row.last_observed_at,
            attempts_used: row.attempts_used,
            state: row.state,
            dispatch_outcome: self.outcome,
        }
    }
}

pub(super) async fn live(
    connection: &mut PgConnection,
    caller: TenantContext,
    recognition: Recognition,
) -> Result<LiveLease, Failure> {
    let sql = format!(
        "SELECT pending,active,storage_bytes,memory_bytes,cost_microusd,{}, \
         CASE WHEN dispatch_outcome IS NULL THEN NULL \
          WHEN octet_length(dispatch_outcome)<=32 THEN dispatch_outcome END AS dispatch_outcome, \
         dispatch_outcome IS NOT NULL AS outcome_present \
         FROM arboresce_admission.reservations WHERE organization_id=$1::uuid \
         AND reservation_id=$2::uuid FOR UPDATE",
        checked::instant("expires_at")
    );
    let row = sqlx::query(sqlx::AssertSqlSafe(sql))
        .bind(caller.workspace().organization().to_string())
        .bind(recognition.reservation.to_string())
        .fetch_optional(connection)
        .await
        .map_err(database_failure)?
        .ok_or(Failure::InvalidStoredData)?;
    decode_live(&row, recognition)
}

pub(crate) fn decode_live(row: &PgRow, recognition: Recognition) -> Result<LiveLease, Failure> {
    let present: bool = row
        .try_get("outcome_present")
        .map_err(|_| Failure::InvalidStoredData)?;
    let outcome = if present {
        Some(
            match checked::text(row, "dispatch_outcome")
                .map_err(authority_failure)?
                .as_str()
            {
                "not_dispatched" => DispatchOutcome::NotDispatched,
                "unknown" => DispatchOutcome::Unknown,
                "accepted_running" => DispatchOutcome::AcceptedRunning,
                "completed" => DispatchOutcome::Completed,
                _ => return Err(Failure::InvalidStoredData),
            },
        )
    } else {
        None
    };
    let lease = LiveLease {
        recognition,
        resources: Resources {
            pending: number(row, "pending")?,
            active: number(row, "active")?,
            storage_bytes: number(row, "storage_bytes")?,
            memory_bytes: number(row, "memory_bytes")?,
            cost_microusd: number(row, "cost_microusd")?,
        },
        expires_at: checked::time(row, "expires_at").map_err(authority_failure)?,
        outcome,
    };
    lease.view().validate()?;
    Ok(lease)
}

pub(super) async fn update_counters(
    connection: &mut PgConnection,
    caller: TenantContext,
    organization: Resources,
    global: GlobalResources,
    next_order: u64,
) -> Result<(), Failure> {
    let global_result = sqlx::query(
        "UPDATE arboresce_admission.global_budget SET active_used=$1,memory_used=$2, \
         next_reservation_order=$3 WHERE singleton=1",
    )
    .bind(global.active as i64)
    .bind(global.memory_bytes as i64)
    .bind(next_order as i64)
    .execute(&mut *connection)
    .await
    .map_err(database_failure)?;
    let organization_result = sqlx::query(
        "UPDATE arboresce_admission.organization_budgets SET pending_used=$2,active_used=$3, \
         storage_used=$4,memory_used=$5,cost_used=$6 WHERE organization_id=$1::uuid",
    )
    .bind(caller.workspace().organization().to_string())
    .bind(organization.pending as i64)
    .bind(organization.active as i64)
    .bind(organization.storage_bytes as i64)
    .bind(organization.memory_bytes as i64)
    .bind(organization.cost_microusd as i64)
    .execute(connection)
    .await
    .map_err(database_failure)?;
    if global_result.rows_affected() != 1 || organization_result.rows_affected() != 1 {
        return Err(Failure::InvalidStoredData);
    }
    Ok(())
}

pub(super) async fn observe(
    connection: &mut PgConnection,
    caller: TenantContext,
    reservation: Identifier,
    now: UtcTimestamp,
) -> Result<(), Failure> {
    let global = sqlx::query(
        "UPDATE arboresce_admission.global_budget SET last_observed_at=$1::timestamptz WHERE singleton=1",
    ).bind(now.to_string()).execute(&mut *connection).await.map_err(database_failure)?;
    let recognition = sqlx::query(
        "UPDATE arboresce_admission.reservation_recognition SET last_observed_at=$3::timestamptz \
         WHERE organization_id=$1::uuid AND reservation_id=$2::uuid",
    )
    .bind(caller.workspace().organization().to_string())
    .bind(reservation.to_string())
    .bind(now.to_string())
    .execute(connection)
    .await
    .map_err(database_failure)?;
    if global.rows_affected() != 1 || recognition.rows_affected() != 1 {
        return Err(Failure::InvalidStoredData);
    }
    Ok(())
}
