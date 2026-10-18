//! Pure shared-resource boundaries over explicit supplied values.
//!
//! These observations neither reserve capacity nor authenticate a caller. The
//! canonical adapter must lock, recheck and persist the decision atomically.

use super::TenantContext;
use arboresce_domain::{Identifier, UtcTimestamp};

pub const COUNTER_MAX: u64 = i64::MAX as u64;
pub const FENCE_MAX: u64 = 999_999_999_999_999_999;
const ATTEMPT_MAX: u64 = 8;
const LEASE_MAX_MS: u64 = 1_800_000;
const TOTAL_MAX_MS: u64 = 3_600_000;

/// Finite diagnostics retain neither rejected input nor an underlying cause.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdmissionFailure {
    InvalidConfiguration,
    InvalidInput,
    InvalidStoredData,
    Forbidden,
    NotFound,
    QuotaExceeded,
    FenceMismatch,
    DeadlineExceeded,
    Expired,
    ReconciliationRequired,
    AttemptsExhausted,
    ClockUnavailable,
    Unavailable,
    UnsafeRole,
    RetainedContext,
    UnknownCompletion,
    Conflict,
}

impl std::fmt::Display for AdmissionFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}
impl std::error::Error for AdmissionFailure {}

/// Independently registered configuration, not a caller-selected authority flag.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AdmissionPolicy<'a> {
    pub max_attempt_limit: u64,
    pub max_lease_ms: u64,
    pub max_total_ms: u64,
    pub membership_policy: &'a str,
}

pub fn token(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 96
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'.' | b'-'))
}

impl AdmissionPolicy<'_> {
    pub fn validate(self) -> Result<(), AdmissionFailure> {
        if self.max_attempt_limit == 0
            || self.max_attempt_limit > ATTEMPT_MAX
            || self.max_lease_ms == 0
            || self.max_lease_ms > LEASE_MAX_MS
            || self.max_total_ms == 0
            || self.max_total_ms > TOTAL_MAX_MS
            || self.max_lease_ms > self.max_total_ms
            || !token(self.membership_policy)
        {
            return Err(AdmissionFailure::InvalidConfiguration);
        }
        Ok(())
    }
}

/// Exact nonnegative resource units, bounded separately before SQL conversion.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Resources {
    pub pending: u64,
    pub active: u64,
    pub storage_bytes: u64,
    pub memory_bytes: u64,
    pub cost_microusd: u64,
}

impl Resources {
    pub const fn values(self) -> [u64; 5] {
        [
            self.pending,
            self.active,
            self.storage_bytes,
            self.memory_bytes,
            self.cost_microusd,
        ]
    }

    pub fn valid(self) -> bool {
        self.values().into_iter().all(|value| value <= COUNTER_MAX)
    }

    pub fn nonzero(self) -> bool {
        self.values().into_iter().any(|value| value != 0)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GlobalResources {
    pub active: u64,
    pub memory_bytes: u64,
}

impl GlobalResources {
    fn valid(self) -> bool {
        self.active <= COUNTER_MAX && self.memory_bytes <= COUNTER_MAX
    }
}

/// This is a supplied snapshot, not evidence that a database lock was taken.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Capacity {
    pub organization_limits: Resources,
    pub organization_used: Resources,
    pub global_limits: GlobalResources,
    pub global_used: GlobalResources,
}

fn fits(used: u64, requested: u64, limit: u64) -> bool {
    used <= limit && requested <= limit - used
}

/// Check every class before any mutation; subtraction avoids counter overflow.
/// A reduced limit cannot erase already accepted responsibility.
pub fn check_capacity(capacity: Capacity, requested: Resources) -> Result<(), AdmissionFailure> {
    if !capacity.organization_limits.valid()
        || !capacity.organization_used.valid()
        || !capacity.global_limits.valid()
        || !capacity.global_used.valid()
    {
        return Err(AdmissionFailure::InvalidStoredData);
    }
    if !requested.valid() {
        return Err(AdmissionFailure::InvalidInput);
    }
    if capacity
        .organization_used
        .values()
        .into_iter()
        .zip(requested.values())
        .zip(capacity.organization_limits.values())
        .any(|((used, requested), limit)| !fits(used, requested, limit))
        || !fits(
            capacity.global_used.active,
            requested.active,
            capacity.global_limits.active,
        )
        || !fits(
            capacity.global_used.memory_bytes,
            requested.memory_bytes,
            capacity.global_limits.memory_bytes,
        )
    {
        return Err(AdmissionFailure::QuotaExceeded);
    }
    Ok(())
}

/// A normalized requested claim. Worker labels and fences are not credentials.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReservationClaim<'a> {
    pub reservation: Identifier,
    pub organization: Identifier,
    pub principal: Identifier,
    pub replica: &'a str,
    pub resources: Resources,
    pub owner: &'a str,
    pub fence: u64,
    pub expires_at: UtcTimestamp,
    pub attempt_limit: u64,
    pub total_deadline: UtcTimestamp,
}

fn claim_shape(claim: &ReservationClaim<'_>) -> bool {
    token(claim.owner)
        && token(claim.replica)
        && claim.resources.valid()
        && claim.resources.nonzero()
        && (1..=FENCE_MAX).contains(&claim.fence)
        && (1..=ATTEMPT_MAX).contains(&claim.attempt_limit)
}

fn milliseconds(later: UtcTimestamp, earlier: UtcTimestamp) -> i64 {
    // Checked UTC years 0001..9999 keep every difference inside signed i64.
    later.unix_milliseconds() - earlier.unix_milliseconds()
}

/// Validate only the request/configuration/supplied caller boundary.
/// Canonical current membership and capacity remain the adapter's responsibility.
pub fn check_claim(
    policy: AdmissionPolicy<'_>,
    caller: TenantContext,
    now: UtcTimestamp,
    claim: &ReservationClaim<'_>,
) -> Result<(), AdmissionFailure> {
    policy.validate()?;
    if !claim_shape(claim)
        || claim.attempt_limit > policy.max_attempt_limit
        || claim.expires_at <= now
        || claim.expires_at > claim.total_deadline
        || milliseconds(claim.expires_at, now) > policy.max_lease_ms as i64
        || milliseconds(claim.total_deadline, now) > policy.max_total_ms as i64
    {
        return Err(AdmissionFailure::InvalidInput);
    }
    if claim.organization != caller.workspace().organization()
        || claim.principal != caller.principal()
    {
        return Err(AdmissionFailure::Forbidden);
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReservationState {
    Reserved,
    Active,
    ReconciliationRequired,
    Released,
}

/// An independently obtained internal reconciliation observation. Construction
/// alone supplies no external receipt; HTTP callers cannot select this evidence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DispatchOutcome {
    NotDispatched,
    Unknown,
    AcceptedRunning,
    Completed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReservationLease<'a> {
    pub claim: ReservationClaim<'a>,
    pub admitted_at: UtcTimestamp,
    pub last_observed_at: UtcTimestamp,
    pub attempts_used: u64,
    pub state: ReservationState,
    pub dispatch_outcome: Option<DispatchOutcome>,
}

impl ReservationLease<'_> {
    pub fn validate(self) -> Result<(), AdmissionFailure> {
        if !claim_shape(&self.claim)
            || self.attempts_used > self.claim.attempt_limit
            || self.admitted_at >= self.claim.expires_at
            || self.claim.expires_at > self.claim.total_deadline
            || self.admitted_at >= self.claim.total_deadline
            || milliseconds(self.claim.total_deadline, self.admitted_at) > TOTAL_MAX_MS as i64
            || self.last_observed_at < self.admitted_at
        {
            return Err(AdmissionFailure::InvalidStoredData);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReservationOperation {
    Renew { requested_expiry: UtcTimestamp },
    Release { outcome: DispatchOutcome },
    MarkUncertain,
    ReconcileExpired { outcome: DispatchOutcome },
    BeginAttempt,
    CompleteNotDispatched { outcome: DispatchOutcome },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReservationControl<'a> {
    pub reservation: Identifier,
    pub owner: &'a str,
    pub fence: u64,
    pub operation: ReservationOperation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReservationDecision {
    Renew,
    Release,
    Retain,
    Attempt,
    Ready,
}

fn reconciled_release(
    lease: ReservationLease<'_>,
    outcome: DispatchOutcome,
) -> Result<ReservationDecision, AdmissionFailure> {
    if outcome != DispatchOutcome::NotDispatched {
        return Ok(ReservationDecision::Retain);
    }
    if matches!(
        lease.dispatch_outcome,
        Some(DispatchOutcome::AcceptedRunning | DispatchOutcome::Completed)
    ) {
        return Err(AdmissionFailure::ReconciliationRequired);
    }
    Ok(ReservationDecision::Release)
}

/// Observe a supplied lease without mutating it. The database adapter must load
/// the current lease itself, hold the owner/fence and counters together, and
/// positively complete its transaction before returning an accepted control.
pub fn check_control(
    policy: AdmissionPolicy<'_>,
    caller: TenantContext,
    now: UtcTimestamp,
    lease: ReservationLease<'_>,
    command: ReservationControl<'_>,
) -> Result<ReservationDecision, AdmissionFailure> {
    policy.validate()?;
    lease.validate()?;
    if !token(command.owner) || !(1..=FENCE_MAX).contains(&command.fence) {
        return Err(AdmissionFailure::InvalidInput);
    }
    if caller.workspace().organization() != lease.claim.organization
        || caller.principal() != lease.claim.principal
    {
        return Err(AdmissionFailure::Forbidden);
    }
    if command.reservation != lease.claim.reservation || lease.state == ReservationState::Released {
        return Err(AdmissionFailure::NotFound);
    }
    if command.owner != lease.claim.owner || command.fence != lease.claim.fence {
        return Err(AdmissionFailure::FenceMismatch);
    }
    if now < lease.admitted_at || now < lease.last_observed_at {
        return Err(AdmissionFailure::ClockUnavailable);
    }
    if !matches!(
        command.operation,
        ReservationOperation::ReconcileExpired { .. }
    ) {
        if now >= lease.claim.total_deadline {
            return Err(AdmissionFailure::DeadlineExceeded);
        }
        if now >= lease.claim.expires_at {
            return Err(AdmissionFailure::Expired);
        }
    }
    match command.operation {
        ReservationOperation::Renew { requested_expiry } => {
            if lease.state == ReservationState::ReconciliationRequired {
                return Err(AdmissionFailure::ReconciliationRequired);
            }
            if requested_expiry > lease.claim.total_deadline {
                return Err(AdmissionFailure::DeadlineExceeded);
            }
            if requested_expiry <= now
                || requested_expiry < lease.claim.expires_at
                || milliseconds(requested_expiry, now) > policy.max_lease_ms as i64
            {
                return Err(AdmissionFailure::InvalidInput);
            }
            Ok(ReservationDecision::Renew)
        }
        ReservationOperation::Release { outcome } => reconciled_release(lease, outcome),
        ReservationOperation::MarkUncertain => Ok(ReservationDecision::Retain),
        ReservationOperation::BeginAttempt => {
            if lease.state != ReservationState::Reserved {
                return Err(AdmissionFailure::ReconciliationRequired);
            }
            if lease.attempts_used >= lease.claim.attempt_limit.min(policy.max_attempt_limit) {
                return Err(AdmissionFailure::AttemptsExhausted);
            }
            Ok(ReservationDecision::Attempt)
        }
        ReservationOperation::CompleteNotDispatched { outcome } => {
            if lease.state == ReservationState::Reserved || lease.attempts_used == 0 {
                return Err(AdmissionFailure::InvalidInput);
            }
            if reconciled_release(lease, outcome)? == ReservationDecision::Release {
                Ok(ReservationDecision::Ready)
            } else {
                Ok(ReservationDecision::Retain)
            }
        }
        ReservationOperation::ReconcileExpired { outcome } => {
            if now < lease.claim.expires_at {
                return Err(AdmissionFailure::InvalidInput);
            }
            reconciled_release(lease, outcome)
        }
    }
}
