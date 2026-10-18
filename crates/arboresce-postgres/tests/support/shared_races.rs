//! Owned two-backend barriers and ambiguous terminal observations.

use crate::{
    authority_probes, authority_races,
    authority_service::{Result, Service, preserve_cleanup, with_connection},
    current_authority,
    implementation::{AuthorityFailure, Harness, PermissionChange},
    relay,
    shared_admission::{self as shared, Samples},
    shared_seed::text,
};
use serde_json::{Value, json};
use sqlx::Connection;
use std::{
    future::{Future, poll_fn},
    pin::Pin,
    task::Poll,
};

/// Drive the actual competing operation while waiting for its independent SQL
/// lock proof. Completing before the barrier is a failed schedule, never proof.
async fn wait_for_lock<F: Future>(
    mut pending: Pin<&mut F>,
    barrier: impl Future<Output = Result<Value>>,
) -> Result<Value> {
    let mut barrier = Box::pin(barrier);
    poll_fn(|context| {
        if pending.as_mut().poll(context).is_ready() {
            return Poll::Ready(Err(
                "shared operation completed before the conflicting lock barrier".into(),
            ));
        }
        barrier.as_mut().poll(context)
    })
    .await
}

pub struct PreparedPermission<'a> {
    caller: arboresce_application::policy::TenantContext,
    policy: arboresce_application::policy::admission::AdmissionPolicy<'a>,
    claim: arboresce_application::policy::admission::ReservationClaim<'a>,
    request: arboresce_application::policy::authority::EvaluationRequest,
    change: PermissionChange,
    clock: Samples,
    permission_clock: current_authority::SuppliedClock,
    permission_first: bool,
}

fn revision(value: &Value) -> Result<arboresce_domain::ControlRevision> {
    if text(value, "kind")? != "control" {
        return Err("invalid shared race control revision".into());
    }
    Ok(text(value, "value")?.parse()?)
}

/// Prepare both injected clocks and every typed participant before two sessions.
pub fn prepare_permission<'a>(
    common: &'a Value,
    input: &'a Value,
) -> Result<PreparedPermission<'a>> {
    use arboresce_application::policy::{
        ReferenceKind, ScopedReference, authority::EvaluationRequest,
    };
    let caller = shared::caller(&input["caller"])?;
    let policy = shared::policy(&common["policy"])?;
    let claim = shared::claim(&input["request"])?;
    let _: arboresce_domain::UtcTimestamp = text(input, "now")?.parse()?;
    let permission_clock = current_authority::SuppliedClock::new(&input["now"]);
    let clock = Samples::sequence(&input["clock_sequence"])?;
    let selected = &input["caller"];
    let resource = common["authority_snapshot"]["resources"]
        .as_array()
        .ok_or("missing shared race resources")?
        .iter()
        .find(|row| {
            row["workspace"]["organization_id"] == selected["organization_id"]
                && row["workspace"]["domain_id"] == selected["domain_id"]
        })
        .ok_or("missing owned shared race project")?;
    let kind = match text(resource, "kind")? {
        "project" => ReferenceKind::Project,
        "expense" => ReferenceKind::Expense,
        _ => return Err("unregistered shared race resource kind".into()),
    };
    let membership = common["authority_snapshot"]["memberships"]
        .as_array()
        .ok_or("missing shared race memberships")?
        .iter()
        .find(|row| {
            row["workspace"]["organization_id"] == selected["organization_id"]
                && row["principal"]["principal_id"] == selected["principal_id"]
        })
        .ok_or("missing shared revocation target")?;
    let request = EvaluationRequest {
        executor: caller.principal(),
        selected_workspace: Some(caller.workspace()),
        resource: ScopedReference::new(caller.workspace(), kind, text(resource, "id")?.parse()?),
        action: "project.change.review".to_owned(),
        purpose: "project_support".to_owned(),
        selected_chain: None,
        expected_control: Some(revision(&resource["control_revision"])?),
        claimed_authorizer: None,
    };
    let change = PermissionChange::RevokeMembership {
        principal: caller.principal(),
        expected: revision(&membership["revision"])?,
    };
    let permission_first = match text(input, "recipe")? {
        "permission-revoke-before-claim" => true,
        "claim-before-permission-revoke" | "claim-revoke-then-attempt" => false,
        _ => return Err("unregistered shared permission schedule".into()),
    };
    Ok(PreparedPermission {
        caller,
        policy,
        claim,
        request,
        change,
        clock,
        permission_clock,
        permission_first,
    })
}

/// Deterministic Future controls qualify this same guard, not a SQL lock.
pub async fn guard(input: &Value) -> Result<Value> {
    match input["operation_ready"]
        .as_bool()
        .ok_or("invalid shared guard readiness")?
    {
        true => {
            let mut pending = Box::pin(std::future::ready(()));
            match wait_for_lock(pending.as_mut(), std::future::pending()).await {
                Ok(_) => Err("completed shared guard unexpectedly reached its barrier".into()),
                Err(error) => Ok(json!({"error":error.to_string()})),
            }
        }
        false => {
            let mut pending = Box::pin(std::future::pending::<()>());
            let value = input
                .get("barrier")
                .ok_or("missing shared guard barrier")?
                .clone();
            let barrier = wait_for_lock(pending.as_mut(), std::future::ready(Ok(value))).await?;
            Ok(json!({"barrier":barrier}))
        }
    }
}

pub async fn permission(
    service: &Service,
    common: &Value,
    input: &Value,
) -> Result<(Value, Value)> {
    let PreparedPermission {
        caller,
        policy,
        claim: typed,
        request,
        change,
        mut clock,
        mut permission_clock,
        permission_first,
    } = prepare_permission(common, input)?;
    // Independently created factories cannot manufacture a shared local semaphore barrier.
    let admission_sessions = service.harness.authority_sessions();
    let permission_sessions = service.harness.authority_sessions();
    let mut admission = admission_sessions.admission().await?;
    let mut permission = permission_sessions.permission().await?;
    let operation=async {
        let admission_pid=admission.backend_id().await?;
        let permission_pid=permission.backend_id().await?;
        if admission_pid==permission_pid {return Err("shared race did not own two physical backends".into());}
        let (claim_result,barrier)=if permission_first {
            let fence=permission.fence(&request,change,&mut permission_clock).await?;
            let mut pending=Box::pin(admission.reserve(caller,policy,&typed,&mut clock));
            let barrier=wait_for_lock(pending.as_mut(),authority_races::blocker(service,admission_pid,permission_pid)).await?;
            fence.commit().await?;
            let result=match pending.await {Ok(fence)=>match fence.commit().await {
                Ok(())=>"Committed".to_owned(),Err(error)=>error.to_string()},Err(error)=>error.to_string()};
            (result,barrier)
        } else {
            let fence=admission.reserve(caller,policy,&typed,&mut clock).await?;
            let mut pending=Box::pin(permission.fence(&request,change,&mut permission_clock));
            let barrier=wait_for_lock(pending.as_mut(),authority_races::blocker(service,permission_pid,admission_pid)).await?;
            let result=match fence.commit().await {Ok(())=>"Committed".to_owned(),Err(error)=>error.to_string()};
            pending.await?.commit().await?;
            (result,barrier)
        };
        let mut semantic=json!({"result":claim_result,"native_two_sessions":admission_pid!=permission_pid,
            "first":if permission_first {"permission"}else{"claim"},
            "waited_for_conflicting_lock":barrier["blockers"]==json!([if permission_first {permission_pid}else{admission_pid}])});
        if input["recipe"]=="claim-revoke-then-attempt" {
            let mut clock=Samples::fixed(&input["now"])?;
            let attempt=shared::control(&mut admission,common,input,
                &shared::owned_command(&input["request"],"begin_attempt")?,&mut clock).await?;
            semantic["attempt_results"]=json!([claim_result,"MembershipRevoked",attempt]);
            semantic["result"]=json!(attempt);
        }
        Ok((semantic,json!({"admission_pid":admission_pid,"permission_pid":permission_pid,
            "barrier":barrier,"permission_clock_calls":permission_clock.calls})))
    }.await;
    let a = admission.close().await.map_err(Into::into);
    let p = permission.close().await.map_err(Into::into);
    preserve_cleanup(operation, preserve_cleanup(a, p))
}

pub struct PreparedOrdered<'a> {
    commands: &'a [Value],
    second_operation: &'a str,
    first: arboresce_application::policy::admission::ReservationClaim<'a>,
    second: arboresce_application::policy::admission::ReservationClaim<'a>,
    first_caller: arboresce_application::policy::TenantContext,
    second_caller: arboresce_application::policy::TenantContext,
    policy: arboresce_application::policy::admission::AdmissionPolicy<'a>,
    a_clock: Samples,
    b_clock: Samples,
}

/// Prepare both complete commands and clocks without database access.
pub fn prepare_ordered<'a>(common: &'a Value, input: &'a Value) -> Result<PreparedOrdered<'a>> {
    let commands = input["commands"]
        .as_array()
        .filter(|v| v.len() == 2)
        .ok_or("invalid shared last-slot schedule")?;
    let first_input = super::shared_admission::original_claim_input(common, input, &commands[0])?;
    let second_input = super::shared_admission::original_claim_input(common, input, &commands[1])?;
    let second_operation = text(&commands[1], "operation")?;
    if text(&commands[0], "operation")? != "claim_lock_validate_commit"
        || !matches!(
            second_operation,
            "wait_lock_recheck_deny" | "wait_lock_recheck_admit"
        )
    {
        return Err("unregistered shared ordered claim operation".into());
    }
    let first = shared::claim(&commands[0])?;
    let second = shared::claim(&commands[1])?;
    let first_caller = shared::caller(&first_input["caller"])?;
    let second_caller = shared::caller(&second_input["caller"])?;
    let policy = shared::policy(&common["policy"])?;
    let a_clock = Samples::fixed(&input["now"])?;
    let b_clock = Samples::fixed(&input["now"])?;
    Ok(PreparedOrdered {
        commands,
        second_operation,
        first,
        second,
        first_caller,
        second_caller,
        policy,
        a_clock,
        b_clock,
    })
}

pub async fn last_slot(service: &Service, common: &Value, input: &Value) -> Result<(Value, Value)> {
    let PreparedOrdered {
        commands,
        second_operation,
        first,
        second,
        first_caller,
        second_caller,
        policy,
        mut a_clock,
        mut b_clock,
    } = prepare_ordered(common, input)?;
    let sessions_a = service.harness.authority_sessions();
    let sessions_b = service.harness.authority_sessions();
    let mut a = sessions_a.admission().await?;
    let mut b = sessions_b.admission().await?;
    let operation = async {
        let a_pid = a.backend_id().await?;
        let b_pid = b.backend_id().await?;
        if a_pid == b_pid {
            return Err("shared ordered claims did not own two physical backends".into());
        }
        let fence = a
            .reserve(first_caller, policy, &first, &mut a_clock)
            .await?;
        let mut pending = Box::pin(b.reserve(second_caller, policy, &second, &mut b_clock));
        let barrier = wait_for_lock(
            pending.as_mut(),
            authority_races::blocker(service, b_pid, a_pid),
        )
        .await?;
        fence.commit().await?;
        let second = match pending.await {
            Ok(fence) => {
                fence.commit().await?;
                "Committed".to_owned()
            }
            Err(error) => error.to_string(),
        };
        Ok((
            json!({"first":"Committed","second":second,"first_id":commands[0]["reservation_id"],
            "second_id":commands[1]["reservation_id"]}),
            json!({"pids":[a_pid,b_pid],"barrier":barrier,
                "operations":[commands[0]["operation"],second_operation],
                "clock_calls":[a_clock.calls,b_clock.calls]}),
        ))
    }
    .await;
    let a = a.close().await.map_err(Into::into);
    let b = b.close().await.map_err(Into::into);
    preserve_cleanup(operation, preserve_cleanup(a, b))
}

pub async fn ambiguous(service: &Service, common: &Value, input: &Value) -> Result<(Value, Value)> {
    let rollback = input["recipe"] == "lost-rollback-ack";
    let relay = relay::Relay::start(
        &service.harness.socket,
        if rollback {
            relay::Mode::Rollback
        } else {
            relay::Mode::Commit
        },
    )?;
    let harness =
        Harness::new(relay.directory.clone()).map_err(|_| "invalid owned shared relay target")?;
    let sessions = harness.authority_sessions();
    let mut session = sessions.admission().await?;
    let operation=async {
        let pid=session.backend_id().await?;
        let slots_before=sessions.available_slots();
        let mut clock=Samples::sequence(&input["clock_sequence"])?;
        let result=shared::reserve(&mut session,common,input,&mut clock,rollback).await?;
        let poisoned=session.backend_id().await==Err(AuthorityFailure::UnknownCompletion);
        let charged=sessions.available_slots()==slots_before;
        Ok((json!({"result":result,"session_poisoned":poisoned,
            "slot_charged_until_positive_close":charged,"positive_owned_close_required":poisoned && charged}),
            json!({"pid":pid,"slots_before":slots_before,"slots_before_close":sessions.available_slots()})))
    }.await;
    let operation = preserve_cleanup(operation, session.close().await.map_err(Into::into));
    let frames = relay.finish()?;
    let received = frames.0?;
    let sent = frames.1?;
    let (semantic, mut evidence) = operation?;
    let pid = i32::try_from(
        evidence["pid"]
            .as_i64()
            .ok_or("absent shared relay backend")?,
    )?;
    evidence["positive_close_backend_absent"] =
        json!(authority_probes::absent(service, pid).await?);
    evidence["slots_after_close"] = json!(sessions.available_slots());
    evidence["relay"] = json!({"sent_bytes":sent.bytes,"sent_frames":sent.frames,"sent_dropped":sent.dropped,
        "received_bytes":received.bytes,"received_frames":received.frames,"received_dropped":received.dropped});
    if received.dropped != 1 || sessions.available_slots() != 2 {
        return Err(
            "shared relay lacked its one real dropped completion or positive owned close".into(),
        );
    }
    Ok((semantic, evidence))
}

pub async fn cancelled(service: &Service, common: &Value, input: &Value) -> Result<(Value, Value)> {
    with_connection(authority_probes::raw_role(service,true).await?,async |blocker| {
        let mut transaction=blocker.begin().await?;
        sqlx::query("SELECT set_config('arboresce.organization_id',$1,true),set_config('arboresce.domain_id',$2,true),set_config('arboresce.principal_id',$3,true),set_config('arboresce.chain_id','',true)")
            .bind(text(&input["caller"],"organization_id")?).bind(text(&input["caller"],"domain_id")?)
            .bind(text(&input["caller"],"principal_id")?).execute(&mut *transaction).await?;
        let owner:i32=sqlx::query_scalar("SELECT pg_backend_pid() FROM arboresce_authority.namespace_guard WHERE singleton=1 FOR UPDATE")
            .fetch_one(&mut *transaction).await?;
        let sessions=service.harness.authority_sessions();
        let mut session=sessions.admission().await?;
        let operation=async {
            let pid=session.backend_id().await?;
            let request=shared::claim(&input["request"])?;
            let mut clock=Samples::sequence(&input["clock_sequence"])?;
            let barrier={
                let mut pending=Box::pin(session.reserve(shared::caller(&input["caller"])?,shared::policy(&common["policy"])?,&request,&mut clock));
                wait_for_lock(pending.as_mut(),authority_races::blocker(service,pid,owner)).await?
            };
            let poisoned=session.backend_id().await==Err(AuthorityFailure::UnknownCompletion);
            let charged=sessions.available_slots()==1;
            Ok((json!({"result":if poisoned {"UnknownCompletion"}else{"Reusable"},
                "session_poisoned":poisoned,"slot_charged_until_positive_close":charged,
                "positive_owned_close_required":poisoned && charged}),json!({"pid":pid,"owner":owner,"barrier":barrier})))
        }.await;
        let close=session.close().await.map_err(Into::into);
        let release=transaction.rollback().await.map_err(Into::into);
        let (semantic,mut evidence)=preserve_cleanup(operation,preserve_cleanup(close,release))?;
        let pid=i32::try_from(evidence["pid"].as_i64().ok_or("absent cancelled shared backend")?)?;
        evidence["positive_close_backend_absent"]=json!(authority_probes::absent(service,pid).await?);
        evidence["slots_after_close"]=json!(sessions.available_slots());
        Ok((semantic,evidence))
    }).await
}
