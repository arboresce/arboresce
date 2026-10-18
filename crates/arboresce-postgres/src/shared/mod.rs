//! Canonical shared reservations in the isolated direct qualification namespace.
//! Reuse the bounded authority sessions; no product or bootstrap writer.

use arboresce_application::policy::{
    TenantContext,
    admission::{
        AdmissionFailure as Failure, AdmissionPolicy, ReservationClaim, ReservationControl,
        ReservationDecision,
    },
};
use arboresce_domain::{Clock, Identifier, UtcTimestamp};
use sqlx::PgConnection;
use tokio::time::timeout_at;

use super::authority::{AuthorityFailure, AuthoritySession, Guard};

mod catalogue;
mod claim;
mod control;
pub(crate) mod rows;

fn authority_failure(value: AuthorityFailure) -> Failure {
    match value {
        AuthorityFailure::UnsafeRole => Failure::UnsafeRole,
        AuthorityFailure::RetainedContext => Failure::RetainedContext,
        AuthorityFailure::UnknownCompletion => Failure::UnknownCompletion,
        AuthorityFailure::InvalidStoredData => Failure::InvalidStoredData,
        _ => Failure::Unavailable,
    }
}

fn database_failure(value: sqlx::Error) -> Failure {
    if value.as_database_error().and_then(|e| e.code()).as_deref() == Some("23505") {
        Failure::Conflict
    } else {
        Failure::Unavailable
    }
}

enum Boundary {
    Live {
        expires_at: UtcTimestamp,
        total_deadline: UtcTimestamp,
    },
    Reconciliation,
}

struct Pending<T> {
    result: T,
    reservation: Identifier,
    observed_at: UtcTimestamp,
    boundary: Boundary,
}

/// A short owned transaction, before any external work. Only positive COMMIT
/// accepts its result. Dropping/cancelling keeps the reused physical session
/// poisoned until its separately bounded positive close; it is not a receipt.
pub struct ReservationFence<'session, 'clock, C: Clock, T> {
    guard: Guard<'session>,
    clock: &'clock mut C,
    caller: TenantContext,
    pending: Pending<T>,
}

impl<C: Clock, T> ReservationFence<'_, '_, C, T> {
    pub async fn commit(mut self) -> Result<T, Failure> {
        let result = timeout_at(self.guard.deadline, async {
            let now = self.clock.now().map_err(|_| Failure::ClockUnavailable)?;
            if now < self.pending.observed_at {
                return Err(Failure::ClockUnavailable);
            }
            if let Boundary::Live {
                expires_at,
                total_deadline,
            } = self.pending.boundary
            {
                if now >= total_deadline {
                    return Err(Failure::DeadlineExceeded);
                }
                if now >= expires_at {
                    return Err(Failure::Expired);
                }
            }
            // Both rows are already locked. This final trusted sample advances
            // neither the original lifetime nor the maximum attempt count.
            rows::observe(
                self.guard.connection(),
                self.caller,
                self.pending.reservation,
                now,
            )
            .await
        })
        .await
        .unwrap_or(Err(Failure::Unavailable));
        if let Err(error) = result {
            self.guard.finish(false).await.map_err(authority_failure)?;
            return Err(error);
        }
        self.guard.finish(true).await.map_err(authority_failure)?;
        Ok(self.pending.result)
    }

    /// A positive rollback accepts no reservation or control change.
    pub async fn rollback(self) -> Result<(), Failure> {
        self.guard.finish(false).await.map_err(authority_failure)
    }
}

async fn reject<T>(guard: Guard<'_>, error: Failure) -> Result<T, Failure> {
    guard.finish(false).await.map_err(authority_failure)?;
    Err(error)
}

impl AuthoritySession {
    /// Lock/recheck current canonical membership and all shared classes in one
    /// finite transaction. The returned fence must commit before dispatch.
    pub async fn reserve<'session, 'clock, C: Clock>(
        &'session mut self,
        caller: TenantContext,
        policy: AdmissionPolicy<'_>,
        request: &ReservationClaim<'_>,
        clock: &'clock mut C,
    ) -> Result<ReservationFence<'session, 'clock, C, ()>, Failure> {
        policy.validate()?;
        let entered = clock.now().map_err(|_| Failure::ClockUnavailable)?;
        arboresce_application::policy::admission::check_claim(policy, caller, entered, request)?;
        let mut guard = self
            .begin_context(caller, None, false)
            .await
            .map_err(authority_failure)?;
        let result = timeout_at(guard.deadline, async {
            catalogue::validate(guard.connection()).await?;
            claim::prepare(guard.connection(), caller, policy, request, clock, entered).await
        })
        .await
        .unwrap_or(Err(Failure::Unavailable));
        match result {
            Ok(pending) => Ok(ReservationFence {
                guard,
                clock,
                caller,
                pending,
            }),
            Err(error) => reject(guard, error).await,
        }
    }

    /// Load the current owned/fenced lease from SQL; callers cannot submit a
    /// prevalidated canonical lease. Reconciliation observations come only from
    /// the separately qualified internal adapter, never an HTTP authority flag.
    pub async fn control_reservation<'session, 'clock, C: Clock>(
        &'session mut self,
        caller: TenantContext,
        policy: AdmissionPolicy<'_>,
        command: ReservationControl<'_>,
        clock: &'clock mut C,
    ) -> Result<ReservationFence<'session, 'clock, C, ReservationDecision>, Failure> {
        policy.validate()?;
        let entered = clock.now().map_err(|_| Failure::ClockUnavailable)?;
        let mut guard = self
            .begin_context(caller, None, false)
            .await
            .map_err(authority_failure)?;
        let result = timeout_at(guard.deadline, async {
            catalogue::validate(guard.connection()).await?;
            control::prepare(guard.connection(), caller, policy, command, clock, entered).await
        })
        .await
        .unwrap_or(Err(Failure::Unavailable));
        match result {
            Ok(pending) => Ok(ReservationFence {
                guard,
                clock,
                caller,
                pending,
            }),
            Err(error) => reject(guard, error).await,
        }
    }
}

async fn exactly_one(
    connection: &mut PgConnection,
    caller: TenantContext,
    reservation: Identifier,
    sql: &'static str,
) -> Result<(), Failure> {
    let result = sqlx::query(sql)
        .bind(caller.workspace().organization().to_string())
        .bind(reservation.to_string())
        .execute(connection)
        .await
        .map_err(database_failure)?;
    if result.rows_affected() != 1 {
        return Err(Failure::InvalidStoredData);
    }
    Ok(())
}
