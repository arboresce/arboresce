//! Fenced bounded lease transitions, retaining every unresolved responsibility.

use arboresce_application::policy::{
    TenantContext,
    admission::{
        self, AdmissionFailure as Failure, AdmissionPolicy, GlobalResources, ReservationControl,
        ReservationDecision as Decision, ReservationOperation as Operation, ReservationState,
        Resources,
    },
};
use arboresce_domain::{Clock, UtcTimestamp};
use sqlx::PgConnection;

use super::{Boundary, Pending, claim, database_failure, exactly_one, rows};

fn subtract(used: Resources, release: Resources) -> Result<Resources, Failure> {
    Ok(Resources {
        pending: used
            .pending
            .checked_sub(release.pending)
            .ok_or(Failure::InvalidStoredData)?,
        active: used
            .active
            .checked_sub(release.active)
            .ok_or(Failure::InvalidStoredData)?,
        storage_bytes: used
            .storage_bytes
            .checked_sub(release.storage_bytes)
            .ok_or(Failure::InvalidStoredData)?,
        memory_bytes: used
            .memory_bytes
            .checked_sub(release.memory_bytes)
            .ok_or(Failure::InvalidStoredData)?,
        cost_microusd: used
            .cost_microusd
            .checked_sub(release.cost_microusd)
            .ok_or(Failure::InvalidStoredData)?,
    })
}

pub(super) async fn prepare<C: Clock>(
    connection: &mut PgConnection,
    caller: TenantContext,
    policy: AdmissionPolicy<'_>,
    command: ReservationControl<'_>,
    clock: &mut C,
    entered: UtcTimestamp,
) -> Result<Pending<Decision>, Failure> {
    if matches!(
        command.operation,
        Operation::Renew { .. } | Operation::BeginAttempt
    ) {
        claim::lock_caller(connection, caller, policy).await?;
    }
    // Cleanup acquires namespace SHARE and the same ordered budget/lease locks.
    // It discovers no new authority and need not read parent rows hidden by
    // revoked-membership RLS. Ownership/fence remain canonical and mandatory.
    let budgets = rows::budgets(connection, caller).await?;
    let recognition = rows::recognition(connection, caller, command.reservation)
        .await?
        .ok_or(Failure::NotFound)?;
    if recognition.state == ReservationState::Released {
        return Err(Failure::NotFound);
    }
    let stored = rows::live(connection, caller, recognition).await?;
    let lease = stored.view();
    let now = clock.now().map_err(|_| Failure::ClockUnavailable)?;
    if now < entered || now < budgets.observed_at {
        return Err(Failure::ClockUnavailable);
    }
    let decision = admission::check_control(policy, caller, now, lease, command)?;
    match decision {
        Decision::Renew => {
            if let Operation::Renew { requested_expiry } = command.operation {
                let changed = sqlx::query(
                    "UPDATE arboresce_admission.reservations SET expires_at=$3::timestamptz \
                     WHERE organization_id=$1::uuid AND reservation_id=$2::uuid",
                ).bind(caller.workspace().organization().to_string()).bind(command.reservation.to_string())
                .bind(requested_expiry.to_string()).execute(&mut *connection).await.map_err(database_failure)?;
                if changed.rows_affected()!=1 { return Err(Failure::InvalidStoredData); }
            }
        }
        Decision::Retain => exactly_one(connection, caller, command.reservation,
            "UPDATE arboresce_admission.reservation_recognition SET state='reconciliation_required' \
             WHERE organization_id=$1::uuid AND reservation_id=$2::uuid").await?,
        Decision::Attempt => exactly_one(connection, caller, command.reservation,
            "UPDATE arboresce_admission.reservation_recognition SET attempts_used=attempts_used+1,state='active' \
             WHERE organization_id=$1::uuid AND reservation_id=$2::uuid").await?,
        Decision::Ready => exactly_one(connection, caller, command.reservation,
            "UPDATE arboresce_admission.reservation_recognition SET state='reserved' \
             WHERE organization_id=$1::uuid AND reservation_id=$2::uuid").await?,
        Decision::Release => {
            exactly_one(connection, caller, command.reservation,
                "DELETE FROM arboresce_admission.reservations WHERE organization_id=$1::uuid AND reservation_id=$2::uuid").await?;
            exactly_one(connection, caller, command.reservation,
                "UPDATE arboresce_admission.reservation_recognition SET state='released' \
                 WHERE organization_id=$1::uuid AND reservation_id=$2::uuid").await?;
            let organization = subtract(budgets.capacity.organization_used, lease.claim.resources)?;
            let global = GlobalResources {
                active: budgets.capacity.global_used.active.checked_sub(lease.claim.resources.active).ok_or(Failure::InvalidStoredData)?,
                memory_bytes: budgets.capacity.global_used.memory_bytes.checked_sub(lease.claim.resources.memory_bytes).ok_or(Failure::InvalidStoredData)?,
            };
            rows::update_counters(connection, caller, organization, global, budgets.next_order).await?;
        }
    }
    let boundary = if matches!(command.operation, Operation::ReconcileExpired { .. }) {
        Boundary::Reconciliation
    } else {
        let expires_at = match command.operation {
            Operation::Renew { requested_expiry } => requested_expiry,
            _ => lease.claim.expires_at,
        };
        Boundary::Live {
            expires_at,
            total_deadline: lease.claim.total_deadline,
        }
    };
    Ok(Pending {
        result: decision,
        reservation: command.reservation,
        observed_at: now,
        boundary,
    })
}
