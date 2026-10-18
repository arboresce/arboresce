//! Input-only preparation and actual shared reservation transactions.

use crate::{
    authority_service::{Result, Service, preserve_cleanup},
    current_authority,
    implementation::{AuthoritySession, AuthoritySessions},
    shared_seed::{self as seed, number, text},
};
use arboresce_application::policy::{
    TenantContext, Workspace,
    admission::{
        AdmissionPolicy, DispatchOutcome, ReservationClaim, ReservationControl,
        ReservationOperation, Resources,
    },
};
use arboresce_domain::{Clock, UtcTimestamp};
use arboresce_xtask::{collection_prepared, inventory};
use serde_json::{Value, json};
use std::{
    path::Path,
    time::{Duration, Instant},
};

pub fn fixtures() -> Result<(Value, Value)> {
    let additional = package("shared-admission-v1", "shared-admission", 8 << 20)?;
    let (original, _, _) = current_authority::fixtures()?;
    Ok((original, additional))
}

pub fn supplemental_fixtures() -> Result<Value> {
    package(
        "shared-admission-additional-v1",
        "shared-admission-additional",
        4 << 20,
    )
}

pub fn guard_fixtures() -> Result<Value> {
    package(
        "shared-admission-guards-v1",
        "shared-admission-guards",
        4 << 20,
    )
}

pub fn preparation_fixtures() -> Result<Value> {
    package(
        "shared-admission-preparation-v1",
        "shared-admission-preparation",
        4 << 20,
    )
}

fn package(directory: &str, prefix: &str, case_limit: u64) -> Result<Value> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let base = root.join("tests/security/fixtures").join(directory);
    let (profile, _) = collection_prepared::read_json(&base.join("profile.json"), 65536)?;
    for row in profile["sources"]
        .as_array()
        .ok_or("missing shared source pins")?
    {
        let path = text(row, "path")?;
        inventory::portable_path(Path::new(path))?;
        let actual = collection_prepared::fingerprint(
            &root.join(path),
            8 << 20,
            Instant::now() + Duration::from_secs(10),
        )?;
        if actual["sha256"] != row["sha256"] || actual["size"] != row["bytes"] {
            return Err("shared source pin mismatch".into());
        }
    }
    for (path, row) in profile["files"]
        .as_object()
        .ok_or("missing shared file pins")?
    {
        inventory::portable_path(Path::new(path))?;
        let limit = match path.as_str() {
            "README.md" => 16_384,
            "cases.json" => case_limit,
            "schema.json" => 131_072,
            _ => return Err("unregistered shared fixture file".into()),
        };
        let actual = collection_prepared::fingerprint(
            &base.join(path),
            limit,
            Instant::now() + Duration::from_secs(10),
        )?;
        if actual["sha256"] != row["sha256"] || actual["size"] != row["bytes"] {
            return Err("shared fixture pin mismatch".into());
        }
    }
    let (additional, _) = collection_prepared::read_json(&base.join("cases.json"), case_limit)?;
    if additional["fixture_version"] != format!("{prefix}-boundaries-v1")
        || profile["fixture_version"] != format!("{prefix}-profile-v1")
        || profile["runtime_projection"]["allowed"] != json!(["common_inputs", "case.input"])
    {
        return Err("unsupported shared fixture projection".into());
    }
    Ok(additional)
}

fn unsigned(value: &Value, key: &str) -> Result<u64> {
    value[key]
        .as_u64()
        .filter(|n| *n <= i64::MAX as u64)
        .ok_or_else(|| "invalid shared unsigned input".into())
}

pub fn caller(value: &Value) -> Result<TenantContext> {
    Ok(TenantContext::new(
        Workspace::new(
            text(value, "organization_id")?.parse()?,
            text(value, "domain_id")?.parse()?,
        ),
        text(value, "principal_id")?.parse()?,
    ))
}

pub fn policy(value: &Value) -> Result<AdmissionPolicy<'_>> {
    Ok(AdmissionPolicy {
        max_attempt_limit: unsigned(value, "max_attempt_limit")?,
        max_lease_ms: unsigned(value, "max_lease_ms")?,
        max_total_ms: unsigned(value, "max_total_ms")?,
        membership_policy: text(value, "membership_policy")?,
    })
}

pub fn claim(value: &Value) -> Result<ReservationClaim<'_>> {
    Ok(ReservationClaim {
        reservation: text(value, "reservation_id")?.parse()?,
        organization: text(value, "organization_id")?.parse()?,
        principal: text(value, "principal_id")?.parse()?,
        replica: text(value, "replica")?,
        owner: text(value, "owner")?,
        fence: unsigned(value, "fence")?,
        resources: Resources {
            pending: unsigned(value, "pending")?,
            active: unsigned(value, "active")?,
            storage_bytes: unsigned(value, "storage_bytes")?,
            memory_bytes: unsigned(value, "memory_bytes")?,
            cost_microusd: unsigned(value, "cost_microusd")?,
        },
        expires_at: text(value, "expires_at")?.parse()?,
        attempt_limit: unsigned(value, "attempt_limit")?,
        total_deadline: text(value, "total_deadline")?.parse()?,
    })
}

fn outcome(value: &Value) -> Result<DispatchOutcome> {
    Ok(
        match value
            .as_str()
            .ok_or("missing shared dispatch observation")?
        {
            "not_dispatched" => DispatchOutcome::NotDispatched,
            "unknown" => DispatchOutcome::Unknown,
            "accepted_running" => DispatchOutcome::AcceptedRunning,
            "completed" => DispatchOutcome::Completed,
            _ => return Err("invalid shared dispatch observation".into()),
        },
    )
}

pub fn command(value: &Value) -> Result<ReservationControl<'_>> {
    let operation = match text(value, "operation")? {
        "renew" => ReservationOperation::Renew {
            requested_expiry: text(value, "requested_expiry")?.parse()?,
        },
        "release" => ReservationOperation::Release {
            outcome: outcome(&value["dispatch_outcome"])?,
        },
        "cancel" | "timeout" | "dead" | "mark_uncertain" => ReservationOperation::MarkUncertain,
        "reconcile_expired" => ReservationOperation::ReconcileExpired {
            outcome: outcome(&value["dispatch_outcome"])?,
        },
        "begin_attempt" => ReservationOperation::BeginAttempt,
        "complete_not_dispatched" => ReservationOperation::CompleteNotDispatched {
            outcome: outcome(&value["dispatch_outcome"])?,
        },
        _ => return Err("unregistered shared control operation".into()),
    };
    Ok(ReservationControl {
        reservation: text(value, "reservation_id")?.parse()?,
        owner: text(value, "owner")?,
        fence: unsigned(value, "fence")?,
        operation,
    })
}

#[derive(Debug)]
pub struct ClockFailure;
impl std::fmt::Display for ClockFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("declared shared Clock failure")
    }
}
impl std::error::Error for ClockFailure {}

pub struct Samples {
    values: Vec<Option<UtcTimestamp>>,
    pub calls: usize,
}
impl Samples {
    pub fn sequence(value: &Value) -> Result<Self> {
        let rows = value
            .as_array()
            .filter(|v| !v.is_empty() && v.len() <= 3)
            .ok_or("invalid bounded shared Clock input")?;
        let values = rows
            .iter()
            .map(|v| {
                if v.is_null() {
                    Ok(None)
                } else {
                    Ok(Some(
                        v.as_str().ok_or("invalid shared Clock sample")?.parse()?,
                    ))
                }
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(Self { values, calls: 0 })
    }
    pub fn fixed(value: &Value) -> Result<Self> {
        Self::sequence(&json!([value, value, value]))
    }
}
impl Clock for Samples {
    type Error = ClockFailure;
    fn now(&mut self) -> std::result::Result<UtcTimestamp, Self::Error> {
        let index = self.calls;
        self.calls += 1;
        self.values
            .get(index)
            .copied()
            .flatten()
            .ok_or(ClockFailure)
    }
}

/// Observe only typed preparation and the supplied Clock, without a service.
pub fn preparation(input: &Value) -> Result<Value> {
    let value = &input["value"];
    let accepted = match text(input, "helper")? {
        "caller" => caller(value).map(|_| ()),
        "policy" => policy(value).map(|_| ()),
        "claim" => claim(value).map(|_| ()),
        "command" => command(value).map(|_| ()),
        "owned_command" => owned_command(value, "begin_attempt").map(|_| ()),
        "optional_ordinal" => optional_ordinal(value).map(|_| ()),
        "original_claim_input" => {
            return Ok(json!({"normalized":original_claim_input(
                &input["common"], &input["source"], value)?}));
        }
        "clock_diagnostic" => {
            let mut clock = match Samples::sequence(value) {
                Ok(clock) => clock,
                Err(_) => return Ok(json!({"accepted":false})),
            };
            return Ok(match clock.now() {
                Ok(value) => json!({"sample":value.to_string(),"calls":clock.calls}),
                Err(error) => json!({"error":error.to_string(),
                    "debug":format!("{error:?}"),"calls":clock.calls}),
            });
        }
        "clock" => {
            let mut clock = match Samples::sequence(value) {
                Ok(clock) => clock,
                Err(_) => return Ok(json!({"accepted":false})),
            };
            let samples: Vec<_> = (0..4)
                .map(|_| match clock.now() {
                    Ok(value) => value.to_string(),
                    Err(error) => format!("{error:?}"),
                })
                .collect();
            return Ok(json!({"samples":samples,"calls":clock.calls}));
        }
        _ => return Err("unregistered shared preparation helper".into()),
    };
    Ok(json!({"accepted":accepted.is_ok()}))
}

pub async fn reserve(
    session: &mut AuthoritySession,
    common: &Value,
    input: &Value,
    clock: &mut Samples,
    rollback: bool,
) -> Result<String> {
    let request = claim(&input["request"])?;
    let result = session
        .reserve(
            caller(&input["caller"])?,
            policy(&common["policy"])?,
            &request,
            clock,
        )
        .await;
    Ok(match result {
        Ok(fence) if rollback => match fence.rollback().await {
            Ok(()) => "RolledBack".to_owned(),
            Err(e) => e.to_string(),
        },
        Ok(fence) => match fence.commit().await {
            Ok(()) => "Committed".to_owned(),
            Err(e) => e.to_string(),
        },
        Err(e) => e.to_string(),
    })
}

pub async fn control(
    session: &mut AuthoritySession,
    common: &Value,
    input: &Value,
    command_input: &Value,
    clock: &mut Samples,
) -> Result<String> {
    let result = session
        .control_reservation(
            caller(&input["caller"])?,
            policy(&common["policy"])?,
            command(command_input)?,
            clock,
        )
        .await;
    Ok(match result {
        Ok(fence) => match fence.commit().await {
            Ok(value) => format!("{value:?}"),
            Err(e) => e.to_string(),
        },
        Err(e) => e.to_string(),
    })
}

pub async fn one_claim(
    sessions: &AuthoritySessions,
    common: &Value,
    input: &Value,
    rollback: bool,
) -> Result<Value> {
    let mut clock = Samples::sequence(&input["clock_sequence"])?;
    let result = match sessions.admission().await {
        Ok(mut session) => {
            let result = reserve(&mut session, common, input, &mut clock, rollback).await;
            preserve_cleanup(result, session.close().await.map_err(Into::into))?
        }
        Err(error) => error.to_string(),
    };
    Ok(json!({"result":result,"clock_calls":clock.calls}))
}

pub fn owned_command(request: &Value, operation: &str) -> Result<Value> {
    Ok(
        json!({"reservation_id":text(request,"reservation_id")?,"owner":text(request,"owner")?,
        "fence":number(request,"fence")?,"operation":operation,"dispatch_outcome":"not_dispatched"}),
    )
}

pub async fn attempts(service: &Service, common: &Value, input: &Value) -> Result<Value> {
    let operations: &[&str] = match text(input, "recipe")? {
        "bounded-attempts" => &[
            "begin_attempt",
            "complete_not_dispatched",
            "begin_attempt",
            "complete_not_dispatched",
            "begin_attempt",
        ],
        "active-attempt-overlap" => &["begin_attempt", "begin_attempt"],
        "unknown-attempt-retry" => &["begin_attempt", "mark_uncertain", "begin_attempt"],
        "released-id-recognition" => &["release", "claim"],
        _ => return Err("unregistered shared attempt schedule".into()),
    };
    let sessions = service.harness.authority_sessions();
    let mut session = sessions.admission().await?;
    let operation=async {
        let mut results=Vec::new();
        for operation in operations {
            let mut clock=Samples::fixed(&input["now"])?;
            results.push(if *operation=="claim" {reserve(&mut session,common,input,&mut clock,false).await?}
                else {control(&mut session,common,input,&owned_command(&input["request"],operation)?,&mut clock).await?});
        }
        Ok(json!({"result":results.last().ok_or("empty shared attempt schedule")?,"attempt_results":results}))
    }.await;
    preserve_cleanup(operation, session.close().await.map_err(Into::into))
}

pub async fn after(service: &Service, common: &Value) -> Result<Value> {
    let storage = seed::storage(service).await?;
    Ok(
        json!({"ledger_after":seed::ledger(&storage)?,"canonical_storage_after":storage,
        "canonical_authority_after":seed::authority(service,common).await?,
        "provider_dispatch_count":0,"accepted_effect_count":0}),
    )
}

pub fn original_claim_input(common: &Value, input: &Value, command: &Value) -> Result<Value> {
    let organization = text(command, "organization_id")?;
    let domain = common["selected_domains"][organization]
        .as_str()
        .ok_or("unregistered shared workspace")?;
    Ok(
        json!({"request":command,"caller":{"organization_id":organization,
        "domain_id":domain,"principal_id":text(command,"principal_id")?},"now":input["now"],
        "clock_sequence":[input["now"],input["now"],input["now"]]}),
    )
}

pub async fn controlled(
    service: &Service,
    common: &Value,
    input: &Value,
) -> Result<(Value, Value)> {
    let mut evidence = json!({});
    if let Some(fault) = input.get("stored_fault") {
        if text(fault, "operation")? != "delete_live" {
            return Err("unregistered stored shared fault".into());
        }
        let reservation: arboresce_domain::Identifier = text(fault, "reservation_id")?.parse()?;
        let organization = caller(&input["caller"])?.workspace().organization();
        let changed = service
            .with_admin(false, async |connection| {
                Ok(sqlx::query(
                    "DELETE FROM arboresce_admission.reservations \
                WHERE organization_id=$1::uuid AND reservation_id=$2::uuid",
                )
                .bind(organization.to_string())
                .bind(reservation.to_string())
                .execute(connection)
                .await?
                .rows_affected())
            })
            .await?;
        if changed != 1 {
            return Err("stored shared fault did not delete exactly one live row".into());
        }
        // This is the actual faulted state, before adapter invocation and before
        // any later independently declared reset can restore the missing row.
        let before = seed::storage(service).await?;
        evidence["stored_fault"] = json!({"operation":"delete_live","affected_rows":changed,
            "canonical_storage_before":before,"canonical_authority_before":seed::authority(service,common).await?});
    }
    let mut clock = Samples::sequence(&input["clock_sequence"])?;
    let sessions = service.harness.authority_sessions();
    let mut session = sessions.admission().await?;
    let operation = control(&mut session, common, input, &input["command"], &mut clock).await;
    let result = preserve_cleanup(operation, session.close().await.map_err(Into::into))?;
    Ok((json!({"result":result,"clock_calls":clock.calls}), evidence))
}

/// The same optional ordinal is prepared before any seed reset.
pub fn optional_ordinal(input: &Value) -> Result<Option<i64>> {
    input
        .get("next_reservation_order")
        .map(|next| {
            next.as_i64()
                .filter(|v| *v > 0)
                .ok_or_else(|| "invalid shared ordinal input".into())
        })
        .transpose()
}

pub async fn seed_native(service: &Service, common: &Value, input: &Value) -> Result<Value> {
    let next = optional_ordinal(input)?;
    let seeded = seed::seed(service, common, &input["ledger"]).await?;
    if let Some(next) = next {
        service.with_admin(false,async |connection| {
            let changed=sqlx::query("UPDATE arboresce_admission.global_budget SET next_reservation_order=$1 WHERE singleton=1")
                .bind(next).execute(connection).await?.rows_affected();
            if changed!=1 {return Err("missing global ordinal seed row".into());}
            Ok(())
        }).await?;
    }
    Ok(seeded)
}

pub async fn native(service: &Service, common: &Value, input: &Value) -> Result<Value> {
    let recipe = text(input, "recipe")?;
    // Complete the same input-only preparation before any seed or session.
    // The later operation rebuilds its local values from these immutable inputs.
    if recipe == "two-claims-with-capacity" {
        crate::shared_races::prepare_ordered(common, input)?;
    } else if recipe.starts_with("raw-rls-") {
        crate::shared_probes::prepare_rls(common, input)?;
    }
    let seeded = seed_native(service, common, input).await?;
    let (mut result, evidence) = if recipe.starts_with("fault-") {
        (
            crate::shared_probes::fault_claim(service, common, input).await?,
            json!({}),
        )
    } else if recipe.starts_with("raw-rls-") {
        crate::shared_probes::rls(service, common, input).await?
    } else if recipe.starts_with("database-") {
        (
            crate::shared_probes::constraint(service, input).await?,
            json!({}),
        )
    } else {
        match recipe {
            "open-connection-reset-guard" => {
                crate::shared_probes::reset_guard(service, common).await?
            }
            "controlled-lease" => controlled(service, common, input).await?,
            "two-claims-with-capacity" => {
                crate::shared_races::last_slot(service, common, input).await?
            }
            "retained-context" => (
                crate::shared_probes::retained(service, common, input).await?,
                json!({}),
            ),
            "permission-role" => (
                crate::shared_probes::permission(service, common, input).await?,
                json!({}),
            ),
            "lost-commit-ack" | "lost-rollback-ack" => {
                crate::shared_races::ambiguous(service, common, input).await?
            }
            "cancelled-lock-wait" => crate::shared_races::cancelled(service, common, input).await?,
            "permission-revoke-before-claim"
            | "claim-before-permission-revoke"
            | "claim-revoke-then-attempt" => {
                crate::shared_races::permission(service, common, input).await?
            }
            "bounded-attempts"
            | "active-attempt-overlap"
            | "unknown-attempt-retry"
            | "released-id-recognition" => (attempts(service, common, input).await?, json!({})),
            "unknown-membership-policy"
            | "revoked-principal"
            | "revoked-organization"
            | "revoked-domain" => {
                crate::shared_probes::revoke(service, input, recipe).await?;
                (
                    one_claim(&service.harness.authority_sessions(), common, input, false).await?,
                    json!({}),
                )
            }
            "revoked-membership-proved-cleanup" | "revoked-membership-unknown-retains" => {
                crate::shared_probes::revoke(service, input, "revoked-membership-proved-cleanup")
                    .await?;
                let operation = if recipe == "revoked-membership-proved-cleanup" {
                    "release"
                } else {
                    "mark_uncertain"
                };
                let sessions = service.harness.authority_sessions();
                let mut session = sessions.admission().await?;
                let mut clock = Samples::sequence(&input["clock_sequence"])?;
                let operation = control(
                    &mut session,
                    common,
                    input,
                    &owned_command(&input["request"], operation)?,
                    &mut clock,
                )
                .await;
                let actual =
                    preserve_cleanup(operation, session.close().await.map_err(Into::into))?;
                (json!({"result":actual}), json!({"clock_calls":clock.calls}))
            }
            "clock-unavailable-entry"
            | "clock-unavailable-after-locks"
            | "clock-unavailable-before-commit"
            | "clock-rollback-before-commit"
            | "expiry-before-commit"
            | "ordinal-exhaustion"
            | "allocator-collision"
            | "duplicate-recognized-id"
            | "crossed-domain"
            | "positive-rollback" => (
                one_claim(
                    &service.harness.authority_sessions(),
                    common,
                    input,
                    recipe == "positive-rollback",
                )
                .await?,
                json!({}),
            ),
            _ => return Err("unregistered nonrestart shared native recipe".into()),
        }
    };
    let mut evidence = evidence;
    if !recipe.starts_with("clock-")
        && recipe != "expiry-before-commit"
        && recipe != "controlled-lease"
        && recipe != "allocator-collision"
        && let Some(calls) = result
            .as_object_mut()
            .ok_or("invalid shared result")?
            .remove("clock_calls")
    {
        evidence["clock_calls"] = calls;
    }
    let mut semantic = after(service, common).await?;
    semantic
        .as_object_mut()
        .ok_or("invalid shared after observation")?
        .extend(
            result
                .as_object()
                .ok_or("invalid shared result object")?
                .clone(),
        );
    evidence["seed"] = current_authority::retain_observation(&seeded)?;
    evidence["census"] = current_authority::census(service).await?;
    evidence["migration_versions"] = service
        .with_admin(false, async |connection| seed::history(connection).await)
        .await?;
    Ok(json!({"semantic":semantic,"evidence":evidence}))
}

pub async fn restart(
    service: &mut Service,
    common: &Value,
    input: &Value,
) -> std::result::Result<Value, Box<dyn std::error::Error>> {
    use std::cell::RefCell;
    if input["recipe"] != "unavailable-actual-restart" {
        return Err("unregistered shared restart input".into());
    }
    let seeded = seed_native(service, common, input)
        .await
        .map_err(|error| error as Box<dyn std::error::Error>)?;
    let outage = RefCell::new(None);
    let proof=service.restart(
        async |service| Ok(json!({"state":after(service,common).await?,"canonical_rows":seed::complete_authority(service).await?,"migration_history":seed::complete_history(service).await?})),
        async |service| {
            let observed=one_claim(&service.harness.authority_sessions(),common,input,false).await;
            *outage.borrow_mut()=Some(observed);
        },
        async |service,before| {
            let observed=json!({"state":after(service,common).await?,"canonical_rows":seed::complete_authority(service).await?,"migration_history":seed::complete_history(service).await?});
            if &observed!=before {
                return Err("owned restart changed complete shared canonical state".into());
            }
            Ok(())
        },
    ).await?;
    let observed = outage
        .into_inner()
        .ok_or("shared outage operation did not complete")?
        .map_err(|error| error as Box<dyn std::error::Error>)?;
    let mut semantic = after(service, common)
        .await
        .map_err(|error| error as Box<dyn std::error::Error>)?;
    semantic["result"] = observed["result"].clone();
    semantic["actual_owned_restart_required"] = json!(
        proof["initial_pid"] != proof["current_pid"]
            && proof["adopted"]["new_pid"] == proof["current_pid"]
    );
    Ok(
        json!({"semantic":semantic,"evidence":{"seed":current_authority::retain_observation(&seeded).map_err(|error|error as Box<dyn std::error::Error>)?,
        "restart":proof,"outage":observed,"census":current_authority::census(service).await.map_err(|error|error as Box<dyn std::error::Error>)?}}),
    )
}

pub async fn original(service: &Service, common: &Value, input: &Value) -> Result<Value> {
    let commands = input["commands"]
        .as_array()
        .filter(|v| !v.is_empty() && v.len() <= 2)
        .ok_or("invalid original shared command schedule")?;
    if commands.len() == 2 {
        crate::shared_races::prepare_ordered(common, input)?;
    }
    let seeded = seed::seed(service, common, &input["ledger"]).await?;
    let (result, evidence) = if commands.len() == 2 {
        crate::shared_races::last_slot(service, common, input).await?
    } else {
        let command = &commands[0];
        let operation = text(command, "operation")?;
        let result = if operation.starts_with("claim") {
            let request = original_claim_input(common, input, command)?;
            one_claim(
                &service.harness.authority_sessions(),
                common,
                &request,
                operation == "claim_then_transaction_rollback",
            )
            .await?["result"]
                .clone()
        } else {
            let stored = input["ledger"]["reservations"]
                .as_array()
                .ok_or("missing original reservation ledger")?
                .iter()
                .find(|r| r["reservation_id"] == command["reservation_id"])
                .ok_or("missing original owned lease")?;
            let caller = original_claim_input(common, input, stored)?;
            let mut normalized = command.clone();
            for field in ["owner", "fence"] {
                if normalized.get(field).is_none() {
                    normalized[field] = stored[field].clone();
                }
            }
            if operation == "release" && normalized.get("dispatch_outcome").is_none() {
                normalized["dispatch_outcome"] = json!("unknown");
            }
            let sessions = service.harness.authority_sessions();
            let mut session = sessions.admission().await?;
            let mut clock = Samples::fixed(&input["now"])?;
            let operation = control(&mut session, common, &caller, &normalized, &mut clock).await;
            json!(preserve_cleanup(
                operation,
                session.close().await.map_err(Into::into)
            )?)
        };
        (json!({"result":result}), json!({}))
    };
    let mut semantic = json!({"ledger_after":seed::ledger(&seed::storage(service).await?)?,"provider_dispatch_count":0});
    if commands.len() == 2 {
        semantic["decision"] = json!(if result["first"] == "Committed"
            && result["second"] == "QuotaExceeded"
        {
            "one_admitted_one_denied"
        } else {
            "unexpected_schedule_result"
        });
        semantic["admitted_reservation_ids"] = json!([result["first_id"]]);
        semantic["denied_reservation_ids"] = json!([result["second_id"]]);
        semantic["accepted_effect_count"] = json!(0);
    } else {
        semantic["decision"] = json!(match result["result"]
            .as_str()
            .ok_or("missing original outcome")?
        {
            "Committed" => "admit",
            "QuotaExceeded" => "deny",
            "RolledBack" => "rolled_back",
            "Release" => "released",
            "Retain" => "retain",
            "FenceMismatch" => "deny_fence",
            "DeadlineExceeded" => "deny_deadline",
            "Renew" => "renewed",
            _ => "unexpected_outcome",
        });
    }
    let operation = text(&commands[0], "operation")?;
    if matches!(
        operation,
        "cancel" | "timeout" | "dead" | "reconcile_expired"
    ) {
        semantic["retry_dispatch_allowed"] = json!(false);
    }
    if operation != "reconcile_expired" {
        semantic["problem_expectation"] = if result["result"] == "QuotaExceeded" {
            json!({"code":"rate_limited","status":429})
        } else {
            Value::Null
        };
        semantic["disclosure_expectation"] = json!({"foreign_ids":[],"foreign_counts":null});
    }
    Ok(json!({"semantic":semantic,"evidence":{"native":evidence,
        "seed":current_authority::retain_observation(&seeded)?,"result":result,
        "canonical_storage_after":seed::storage(service).await?,"canonical_authority_after":seed::authority(service,common).await?,
        "census":current_authority::census(service).await?}}))
}
