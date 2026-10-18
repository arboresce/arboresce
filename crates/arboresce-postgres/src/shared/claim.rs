//! Parent-first current caller locks and atomic shared capacity claims.

use arboresce_application::policy::{
    TenantContext,
    admission::{
        self, AdmissionFailure as Failure, AdmissionPolicy, GlobalResources, ReservationClaim,
        Resources,
    },
    authority::{PrincipalKind, RecordState},
};
use arboresce_domain::{Clock, UtcTimestamp};
use sqlx::PgConnection;

use super::super::authority::{read, rows as checked};
use super::{Boundary, Pending, authority_failure, database_failure, rows};

/// Namespace SHARE is already held. Each statement finishes before the next
/// parent discovery, so a wait cannot preserve a stale multi-table snapshot.
pub(super) async fn lock_caller(
    connection: &mut PgConnection,
    caller: TenantContext,
    policy: AdmissionPolicy<'_>,
) -> Result<(), Failure> {
    let organization = caller.workspace().organization().to_string();
    let domain = caller.workspace().domain().to_string();
    let principal = caller.principal().to_string();
    for (sql, parameters) in [
        (
            "SELECT CASE WHEN octet_length(state)<=96 THEN state END AS state FROM \
             arboresce_authority.organizations WHERE organization_id=$1::uuid FOR SHARE",
            vec![organization.clone()],
        ),
        (
            "SELECT CASE WHEN octet_length(state)<=96 THEN state END AS state FROM \
             arboresce_authority.domains WHERE organization_id=$1::uuid AND domain_id=$2::uuid FOR SHARE",
            vec![organization.clone(), domain.clone()],
        ),
    ] {
        let mut query = sqlx::query(sql);
        for value in parameters {
            query = query.bind(value);
        }
        let row = query
            .fetch_optional(&mut *connection)
            .await
            .map_err(database_failure)?
            .ok_or(Failure::Forbidden)?;
        if checked::text(&row, "state").map_err(authority_failure)? != "active" {
            return Err(Failure::Forbidden);
        }
    }
    let sql = format!(
        "SELECT {} FROM arboresce_authority.principals WHERE principal_id=$1::uuid FOR SHARE",
        read::principal_columns()
    );
    let row = sqlx::query(sqlx::AssertSqlSafe(sql))
        .bind(&principal)
        .fetch_optional(&mut *connection)
        .await
        .map_err(database_failure)?
        .ok_or(Failure::Forbidden)?;
    let principal = read::decode_principal(&row).map_err(authority_failure)?;
    if principal.state != RecordState::Active
        || principal.revoked_at.is_some()
        || principal.principal.kind == PrincipalKind::Unsupported
        || principal.identity_profile != "arboresce.nostr-identity.v1"
        || principal.identity_version != "nip01-v1"
    {
        return Err(Failure::Forbidden);
    }
    let sql = format!(
        "SELECT {} FROM arboresce_authority.memberships \
         WHERE organization_id=$1::uuid AND domain_id=$2::uuid AND principal_id=$3::uuid FOR SHARE",
        read::membership_columns()
    );
    let row = sqlx::query(sqlx::AssertSqlSafe(sql))
        .bind(organization)
        .bind(domain)
        .bind(caller.principal().to_string())
        .fetch_optional(connection)
        .await
        .map_err(database_failure)?
        .ok_or(Failure::Forbidden)?;
    let membership = read::decode_membership(&row).map_err(authority_failure)?;
    if membership.state != RecordState::Active
        || membership.policy != policy.membership_policy
        || membership.principal != principal.principal
        || membership.workspace != caller.workspace()
    {
        return Err(Failure::Forbidden);
    }
    Ok(())
}

pub(super) async fn prepare<C: Clock>(
    connection: &mut PgConnection,
    caller: TenantContext,
    policy: AdmissionPolicy<'_>,
    request: &ReservationClaim<'_>,
    clock: &mut C,
    entered: UtcTimestamp,
) -> Result<Pending<()>, Failure> {
    lock_caller(connection, caller, policy).await?;
    let budgets = rows::budgets(connection, caller).await?;
    if rows::recognition(connection, caller, request.reservation)
        .await?
        .is_some()
    {
        return Err(Failure::Conflict);
    }
    let now = clock.now().map_err(|_| Failure::ClockUnavailable)?;
    if now < entered || now < budgets.observed_at {
        return Err(Failure::ClockUnavailable);
    }
    if now >= request.total_deadline {
        return Err(Failure::DeadlineExceeded);
    }
    if now >= request.expires_at {
        return Err(Failure::Expired);
    }
    admission::check_claim(policy, caller, now, request)?;
    admission::check_capacity(budgets.capacity, request.resources)?;
    let next_order = budgets
        .next_order
        .checked_add(1)
        .filter(|next| *next <= admission::COUNTER_MAX)
        .ok_or(Failure::Unavailable)?;
    sqlx::query(
        "INSERT INTO arboresce_admission.reservation_recognition \
         (organization_id,reservation_id,principal_id,owner,replica,fence,attempt_limit,attempts_used, \
          total_deadline,admitted_at,last_observed_at,state,reservation_order) \
         VALUES ($1::uuid,$2::uuid,$3::uuid,$4,$5,$6,$7,0,$8::timestamptz,$9::timestamptz, \
          $9::timestamptz,'reserved',$10)",
    ).bind(request.organization.to_string()).bind(request.reservation.to_string())
    .bind(request.principal.to_string()).bind(request.owner).bind(request.replica)
    .bind(request.fence as i64).bind(request.attempt_limit as i16)
    .bind(request.total_deadline.to_string()).bind(now.to_string()).bind(budgets.next_order as i64)
    .execute(&mut *connection).await.map_err(database_failure)?;
    sqlx::query(
        "INSERT INTO arboresce_admission.reservations \
         (organization_id,reservation_id,principal_id,replica,pending,active,storage_bytes,memory_bytes, \
          cost_microusd,owner,fence,admitted_at,expires_at,attempt_limit,total_deadline,state) \
         VALUES ($1::uuid,$2::uuid,$3::uuid,$4,$5,$6,$7,$8,$9,$10,$11,$12::timestamptz, \
          $13::timestamptz,$14,$15::timestamptz,'reserved')",
    ).bind(request.organization.to_string()).bind(request.reservation.to_string())
    .bind(request.principal.to_string()).bind(request.replica)
    .bind(request.resources.pending as i64).bind(request.resources.active as i64)
    .bind(request.resources.storage_bytes as i64).bind(request.resources.memory_bytes as i64)
    .bind(request.resources.cost_microusd as i64).bind(request.owner).bind(request.fence as i64)
    .bind(now.to_string()).bind(request.expires_at.to_string()).bind(request.attempt_limit as i16)
    .bind(request.total_deadline.to_string())
    .execute(&mut *connection).await.map_err(database_failure)?;
    let used = budgets.capacity.organization_used;
    let wanted = request.resources;
    // check_capacity proved each sum <= its signed storage limit under these
    // same row locks; no update of a configured limit is available to this role.
    let organization = Resources {
        pending: used.pending + wanted.pending,
        active: used.active + wanted.active,
        storage_bytes: used.storage_bytes + wanted.storage_bytes,
        memory_bytes: used.memory_bytes + wanted.memory_bytes,
        cost_microusd: used.cost_microusd + wanted.cost_microusd,
    };
    let global = GlobalResources {
        active: budgets.capacity.global_used.active + wanted.active,
        memory_bytes: budgets.capacity.global_used.memory_bytes + wanted.memory_bytes,
    };
    rows::update_counters(connection, caller, organization, global, next_order).await?;
    Ok(Pending {
        result: (),
        reservation: request.reservation,
        observed_at: now,
        boundary: Boundary::Live {
            expires_at: request.expires_at,
            total_deadline: request.total_deadline,
        },
    })
}
