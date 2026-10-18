//! Measured typed input preparation and observations; no expected result input.

use arboresce_application::policy::{
    TenantContext, Workspace,
    admission::{
        self, AdmissionFailure, AdmissionPolicy, Capacity, DispatchOutcome, GlobalResources,
        ReservationClaim, ReservationControl, ReservationLease, ReservationOperation,
        ReservationState, Resources,
    },
};
use arboresce_domain::{Identifier, UtcTimestamp};
use serde_json::{Value, json};

#[derive(Debug)]
pub struct PreparationFailure;
impl std::fmt::Display for PreparationFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("invalid shared fixture input")
    }
}
impl std::error::Error for PreparationFailure {}
type Result<T> = std::result::Result<T, PreparationFailure>;

pub fn fixtures() -> Result<Value> {
    let cases = include_str!("../../../../tests/security/fixtures/shared-admission-v1/cases.json");
    let profile =
        include_str!("../../../../tests/security/fixtures/shared-admission-v1/profile.json");
    package(cases, profile, "shared-admission", 8_388_608)
}

pub fn additional_fixtures() -> Result<Value> {
    let cases = include_str!(
        "../../../../tests/security/fixtures/shared-admission-additional-v1/cases.json"
    );
    let profile = include_str!(
        "../../../../tests/security/fixtures/shared-admission-additional-v1/profile.json"
    );
    package(cases, profile, "shared-admission-additional", 4_194_304)
}

fn package(cases: &str, profile: &str, prefix: &str, limit: usize) -> Result<Value> {
    if cases.len() > limit || profile.len() > 65_536 {
        return Err(PreparationFailure);
    }
    let value: Value = serde_json::from_str(cases).map_err(|_| PreparationFailure)?;
    let profile: Value = serde_json::from_str(profile).map_err(|_| PreparationFailure)?;
    if value["fixture_version"] != format!("{prefix}-boundaries-v1")
        || profile["fixture_version"] != format!("{prefix}-profile-v1")
        || profile["runtime_projection"]["allowed"] != json!(["common_inputs", "case.input"])
    {
        return Err(PreparationFailure);
    }
    Ok(value)
}

fn field<'a>(value: &'a Value, key: &str) -> Result<&'a Value> {
    value
        .as_object()
        .and_then(|value| value.get(key))
        .ok_or(PreparationFailure)
}
fn text<'a>(value: &'a Value, key: &str) -> Result<&'a str> {
    field(value, key)?.as_str().ok_or(PreparationFailure)
}
fn number(value: &Value, key: &str) -> Result<u64> {
    field(value, key)?.as_u64().ok_or(PreparationFailure)
}
fn identifier(value: &Value, key: &str) -> Result<Identifier> {
    text(value, key)?.parse().map_err(|_| PreparationFailure)
}
fn instant(value: &Value, key: &str) -> Result<UtcTimestamp> {
    text(value, key)?.parse().map_err(|_| PreparationFailure)
}

fn policy(value: &Value) -> Result<AdmissionPolicy<'_>> {
    Ok(AdmissionPolicy {
        max_attempt_limit: number(value, "max_attempt_limit")?,
        max_lease_ms: number(value, "max_lease_ms")?,
        max_total_ms: number(value, "max_total_ms")?,
        membership_policy: text(value, "membership_policy")?,
    })
}

fn caller(value: &Value) -> Result<TenantContext> {
    Ok(TenantContext::new(
        Workspace::new(
            identifier(value, "organization_id")?,
            identifier(value, "domain_id")?,
        ),
        identifier(value, "principal_id")?,
    ))
}

fn resources(value: &Value) -> Result<Resources> {
    Ok(Resources {
        pending: number(value, "pending")?,
        active: number(value, "active")?,
        storage_bytes: number(value, "storage_bytes")?,
        memory_bytes: number(value, "memory_bytes")?,
        cost_microusd: number(value, "cost_microusd")?,
    })
}

fn global(value: &Value) -> Result<GlobalResources> {
    Ok(GlobalResources {
        active: number(value, "active")?,
        memory_bytes: number(value, "memory_bytes")?,
    })
}

fn claim(value: &Value) -> Result<ReservationClaim<'_>> {
    Ok(ReservationClaim {
        reservation: identifier(value, "reservation_id")?,
        organization: identifier(value, "organization_id")?,
        principal: identifier(value, "principal_id")?,
        replica: text(value, "replica")?,
        resources: resources(value)?,
        owner: text(value, "owner")?,
        fence: number(value, "fence")?,
        expires_at: instant(value, "expires_at")?,
        attempt_limit: number(value, "attempt_limit")?,
        total_deadline: instant(value, "total_deadline")?,
    })
}

fn resource_view(value: Resources) -> Value {
    json!({"pending":value.pending,"active":value.active,"storage_bytes":value.storage_bytes,
        "memory_bytes":value.memory_bytes,"cost_microusd":value.cost_microusd})
}
fn global_view(value: GlobalResources) -> Value {
    json!({"active":value.active,"memory_bytes":value.memory_bytes})
}
fn claim_view(value: ReservationClaim<'_>) -> Value {
    json!({"reservation_id":value.reservation.to_string(),
        "organization_id":value.organization.to_string(),"principal_id":value.principal.to_string(),
        "replica":value.replica,"pending":value.resources.pending,"active":value.resources.active,
        "storage_bytes":value.resources.storage_bytes,"memory_bytes":value.resources.memory_bytes,
        "cost_microusd":value.resources.cost_microusd,"owner":value.owner,"fence":value.fence,
        "expires_at":value.expires_at.to_string(),"attempt_limit":value.attempt_limit,
        "total_deadline":value.total_deadline.to_string()})
}

fn outcome(value: &str) -> Result<DispatchOutcome> {
    Ok(match value {
        "not_dispatched" => DispatchOutcome::NotDispatched,
        "unknown" => DispatchOutcome::Unknown,
        "accepted_running" => DispatchOutcome::AcceptedRunning,
        "completed" => DispatchOutcome::Completed,
        _ => return Err(PreparationFailure),
    })
}
fn outcome_name(value: DispatchOutcome) -> &'static str {
    match value {
        DispatchOutcome::NotDispatched => "not_dispatched",
        DispatchOutcome::Unknown => "unknown",
        DispatchOutcome::AcceptedRunning => "accepted_running",
        DispatchOutcome::Completed => "completed",
    }
}
fn state(value: &str) -> Result<ReservationState> {
    Ok(match value {
        "reserved" => ReservationState::Reserved,
        "active" => ReservationState::Active,
        "reconciliation_required" => ReservationState::ReconciliationRequired,
        "released" => ReservationState::Released,
        _ => return Err(PreparationFailure),
    })
}
fn state_name(value: ReservationState) -> &'static str {
    match value {
        ReservationState::Reserved => "reserved",
        ReservationState::Active => "active",
        ReservationState::ReconciliationRequired => "reconciliation_required",
        ReservationState::Released => "released",
    }
}
fn lease(value: &Value) -> Result<ReservationLease<'_>> {
    let stored_outcome = field(value, "dispatch_outcome")?;
    Ok(ReservationLease {
        claim: claim(value)?,
        admitted_at: instant(value, "admitted_at")?,
        last_observed_at: instant(value, "last_observed_at")?,
        attempts_used: number(value, "attempts_used")?,
        state: state(text(value, "state")?)?,
        dispatch_outcome: if stored_outcome.is_null() {
            None
        } else {
            Some(outcome(stored_outcome.as_str().ok_or(PreparationFailure)?)?)
        },
    })
}
fn lease_view(value: ReservationLease<'_>) -> Value {
    let mut view = claim_view(value.claim);
    view["admitted_at"] = json!(value.admitted_at.to_string());
    view["last_observed_at"] = json!(value.last_observed_at.to_string());
    view["attempts_used"] = json!(value.attempts_used);
    view["state"] = json!(state_name(value.state));
    view["dispatch_outcome"] = json!(value.dispatch_outcome.map(outcome_name));
    view
}
fn control(value: &Value) -> Result<ReservationControl<'_>> {
    let operation = match text(value, "operation")? {
        "renew" => ReservationOperation::Renew {
            requested_expiry: instant(value, "requested_expiry")?,
        },
        "release" => ReservationOperation::Release {
            outcome: outcome(text(value, "dispatch_outcome")?)?,
        },
        "mark_uncertain" => ReservationOperation::MarkUncertain,
        "reconcile_expired" => ReservationOperation::ReconcileExpired {
            outcome: outcome(text(value, "dispatch_outcome")?)?,
        },
        "begin_attempt" => ReservationOperation::BeginAttempt,
        "complete_not_dispatched" => ReservationOperation::CompleteNotDispatched {
            outcome: outcome(text(value, "dispatch_outcome")?)?,
        },
        _ => return Err(PreparationFailure),
    };
    Ok(ReservationControl {
        reservation: identifier(value, "reservation_id")?,
        owner: text(value, "owner")?,
        fence: number(value, "fence")?,
        operation,
    })
}

pub fn claim_observation(input: &Value) -> Result<Value> {
    let request = claim(field(input, "request")?)?;
    let result = admission::check_claim(
        policy(field(input, "policy")?)?,
        caller(field(input, "caller")?)?,
        instant(input, "now")?,
        &request,
    );
    Ok(
        json!({"result":result.map(|()|"Valid".to_owned()).unwrap_or_else(|e|e.to_string()),
        "request_after":claim_view(request)}),
    )
}

pub fn quota_observation(input: &Value) -> Result<Value> {
    let capacity = Capacity {
        organization_limits: resources(field(input, "organization_limits")?)?,
        organization_used: resources(field(input, "organization_used")?)?,
        global_limits: global(field(input, "global_limits")?)?,
        global_used: global(field(input, "global_used")?)?,
    };
    let request = resources(field(input, "requested")?)?;
    let result = admission::check_capacity(capacity, request);
    Ok(
        json!({"result":result.map(|()|"Valid".to_owned()).unwrap_or_else(|e|e.to_string()),
        "budget_after":{"organization_limits":resource_view(capacity.organization_limits),
        "organization_used":resource_view(capacity.organization_used),
        "global_limits":global_view(capacity.global_limits),"global_used":global_view(capacity.global_used),
        "requested":resource_view(request)}}),
    )
}

pub fn control_observation(input: &Value) -> Result<Value> {
    let lease = lease(field(input, "lease")?)?;
    let result = admission::check_control(
        policy(field(input, "policy")?)?,
        caller(field(input, "caller")?)?,
        instant(input, "now")?,
        lease,
        control(field(input, "command")?)?,
    );
    let result = match result {
        Ok(value) => format!("{value:?}"),
        Err(error) => error.to_string(),
    };
    Ok(json!({"result":result,"lease_after":lease_view(lease)}))
}

pub fn finite_observation(input: &Value) -> Result<Value> {
    let failure = match text(input, "failure")? {
        "InvalidConfiguration" => AdmissionFailure::InvalidConfiguration,
        "InvalidInput" => AdmissionFailure::InvalidInput,
        "InvalidStoredData" => AdmissionFailure::InvalidStoredData,
        "Forbidden" => AdmissionFailure::Forbidden,
        "NotFound" => AdmissionFailure::NotFound,
        "QuotaExceeded" => AdmissionFailure::QuotaExceeded,
        "FenceMismatch" => AdmissionFailure::FenceMismatch,
        "DeadlineExceeded" => AdmissionFailure::DeadlineExceeded,
        "Expired" => AdmissionFailure::Expired,
        "ReconciliationRequired" => AdmissionFailure::ReconciliationRequired,
        "AttemptsExhausted" => AdmissionFailure::AttemptsExhausted,
        "ClockUnavailable" => AdmissionFailure::ClockUnavailable,
        "Unavailable" => AdmissionFailure::Unavailable,
        "UnsafeRole" => AdmissionFailure::UnsafeRole,
        "RetainedContext" => AdmissionFailure::RetainedContext,
        "UnknownCompletion" => AdmissionFailure::UnknownCompletion,
        "Conflict" => AdmissionFailure::Conflict,
        _ => return Err(PreparationFailure),
    };
    Ok(json!({"display":failure.to_string(),"debug":format!("{failure:?}")}))
}

pub fn preparation(input: &Value) -> Result<Value> {
    let value = field(input, "value")?;
    match text(input, "helper")? {
        "claim" => claim_observation(value),
        "quota" => quota_observation(value),
        "control" => control_observation(value),
        "finite" => finite_observation(value),
        _ => Err(PreparationFailure),
    }
}
