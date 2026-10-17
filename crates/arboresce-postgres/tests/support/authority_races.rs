//! Real backend barriers and complete observations in the owned namespace.

use crate::implementation::{
    AdmissionFence, AuthorityFailure, AuthoritySession, PermissionChange, PermissionSession,
};
use crate::{
    authority_probes,
    authority_service::{Result, Service, fetch_json, query_json, with_rollback},
    current_authority as shared,
};
use serde_json::{Value, json};
use sqlx::PgConnection;
use std::time::Duration;
use tokio::time::{Instant, timeout};

fn identities(input: &Value, session: &str) -> Value {
    input["preallocated_test_identities"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["session"] == session)
        .unwrap()
        .clone()
}

pub fn authority_index(state: &Value, family: &str, target: &Value) -> Result<usize> {
    let (key, scope) = match family {
        "memberships" => ("membership_id", "workspace"),
        "grants" => ("grant_id", "scope"),
        "policies" => ("policy", "workspace"),
        _ => return Err("unadmitted authority family".into()),
    };
    let mut matches = state[family]
        .as_array()
        .ok_or("invalid authority readback")?
        .iter()
        .enumerate()
        .filter(|(_, row)| {
            row[key] == target[key]
                && row[scope] == target[scope]
                && (family != "memberships"
                    || row["principal"]["principal_id"] == target["principal"]["principal_id"])
        });
    let (index, _) = matches.next().ok_or("scoped authority row missing")?;
    if matches.next().is_some() {
        return Err("ambiguous scoped authority row".into());
    }
    Ok(index)
}

fn mutation(input: &Value) -> Result<PermissionChange> {
    let before = &input["canonical_snapshot"];
    let mut after = before.clone();
    let row = &input["revocation_change"];
    let family = if row.get("membership_id").is_some() {
        "memberships"
    } else if row.get("grant_id").is_some() {
        "grants"
    } else {
        "policies"
    };
    let index = authority_index(&after, family, row)?;
    after[family][index] = row.clone();
    shared::change(before, &after).ok_or("missing permission mutation".into())
}

async fn complete(
    fence: AdmissionFence<'_, '_, '_, shared::SuppliedClock>,
    ids: &Value,
    cas: bool,
) -> std::result::Result<Value, AuthorityFailure> {
    let observation = shared::observation(fence.observation());
    if cas {
        fence
            .compare_and_marker(
                shared::id(&ids["marker_id"]),
                shared::id(&ids["observation_id"]),
            )
            .await?;
    } else {
        fence
            .commit_marker(
                shared::id(&ids["marker_id"]),
                shared::id(&ids["observation_id"]),
            )
            .await?;
    }
    Ok(observation)
}

fn actor_result(
    result: std::result::Result<Value, AuthorityFailure>,
    ids: &Value,
    pid: i32,
    clock_calls: usize,
) -> Value {
    match result {
        Ok(observation) => {
            json!({"decision":"admit","problem":null,"committed_observation_id":ids["observation_id"],"observation":observation,"pid":pid,"clock_calls":clock_calls})
        }
        Err(error) => {
            json!({"decision":"deny","problem":shared::problem(error),"failure":error.to_string(),"pid":pid,"clock_calls":clock_calls})
        }
    }
}

async fn join_actor(task: tokio::task::JoinHandle<Result<Value>>) -> Result<Value> {
    timeout(Duration::from_secs(2), task).await??
}

async fn admit(
    mut session: AuthoritySession,
    request: Value,
    ids: Value,
    sample: Value,
    cas: bool,
) -> Result<Value> {
    let request = shared::request(&request);
    let config = shared::configuration();
    let mut clock = shared::SuppliedClock::new(&sample);
    let pid = session.backend_id().await?;
    let result = match session.fence(&request, &config, &mut clock).await {
        Ok(fence) => complete(fence, &ids, cas).await,
        Err(error) => Err(error),
    };
    session.close().await?;
    Ok(actor_result(result, &ids, pid, clock.calls))
}

async fn revoke(
    mut session: PermissionSession,
    request: Value,
    change: PermissionChange,
    sample: Value,
) -> Result<Value> {
    let pid = session.backend_id().await?;
    let request = shared::request(&request);
    let mut clock = shared::SuppliedClock::new(&sample);
    session
        .fence(&request, change, &mut clock)
        .await?
        .commit()
        .await?;
    session.close().await?;
    Ok(json!({"operation":"revoke","positive_commit":true,"pid":pid,"clock_calls":clock.calls}))
}

fn normalized_markers(raw: &Value) -> (Value, Value) {
    let revision = |v: &Value| json!({"kind":"control","value":v.as_i64().unwrap().to_string()});
    let mut markers = Vec::new();
    let mut observations = Vec::new();
    for row in raw.as_array().unwrap() {
        let workspace =
            json!({"organization_id":row["organization_id"],"domain_id":row["domain_id"]});
        markers.push(json!({"marker_id":row["marker_id"],"workspace":workspace,"executor_principal_id":row["executor_principal_id"],
            "selected_chain_id":row["selected_chain_id"],"action":row["action"],"purpose":row["purpose"],
            "resource":{"workspace":workspace,"kind":row["resource_kind"],"id":row["resource_id"]},
            "admitted_policy_epoch":revision(&row["admitted_policy_epoch"]),"admitted_membership_revision":revision(&row["admitted_membership_revision"]),
            "admitted_grant_revision":if row["admitted_grant_revision"].is_null() {Value::Null} else {revision(&row["admitted_grant_revision"])},
            "admitted_resource_control":revision(&row["admitted_resource_control"])}));
        observations.push(json!({"observation_id":row["observation_id"],"marker_id":row["marker_id"],"transaction_outcome":"committed", "executor_principal_id":row["executor_principal_id"],"selected_chain_id":row["selected_chain_id"]}));
    }
    (json!(markers), json!(observations))
}

pub async fn run(
    service: &Service,
    input: &Value,
    recipe: &Value,
    clock_input: &Value,
) -> Result<Value> {
    let seed_observation = shared::retain_observation(
        &shared::seed(service, &input["canonical_snapshot"], recipe).await?,
    )?;
    let sample = clock_input["authority_races"]["current_permission_and_final_sample"].clone();
    let sessions = service.harness.authority_sessions();
    let config = shared::configuration();
    let cas = input.get("request_a").is_some();
    let request = input[if cas { "request_a" } else { "request" }].clone();
    let typed = shared::request(&request);
    let mut first_clock = shared::SuppliedClock::new(&sample);
    let mut order = Vec::new();
    let (first_result, second_result, barrier, pids) = if cas {
        let mut first = sessions.admission().await?;
        let mut second = sessions.admission().await?;
        let first_pid = first.backend_id().await?;
        let second_pid = second.backend_id().await?;
        let fence = first.fence(&typed, &config, &mut first_clock).await?;
        let ids_a = identities(input, "S1");
        let ids_b = identities(input, "S2");
        let request_b = input["request_b"].clone();
        let second_sample = sample.clone();
        let task =
            tokio::spawn(async move { admit(second, request_b, ids_b, second_sample, true).await });
        let proof = blocker(service, second_pid, first_pid).await?;
        let observation = complete(fence, &ids_a, true).await?;
        first.close().await?;
        let second = join_actor(task).await?;
        (
            actor_result(Ok(observation), &ids_a, first_pid, first_clock.calls),
            second,
            proof,
            vec![first_pid, second_pid],
        )
    } else if input["schedule"][0]["operation"] == "revoke" {
        let mut first = sessions.permission().await?;
        let mut second = sessions.admission().await?;
        let first_pid = first.backend_id().await?;
        let second_pid = second.backend_id().await?;
        let fence = first
            .fence(&typed, mutation(input)?, &mut first_clock)
            .await?;
        let ids = identities(input, "S2");
        let second_request = request.clone();
        let second_sample = sample.clone();
        let task =
            tokio::spawn(
                async move { admit(second, second_request, ids, second_sample, false).await },
            );
        let proof = blocker(service, second_pid, first_pid).await?;
        fence.commit().await?;
        order.push("revoke");
        first.close().await?;
        let second = join_actor(task).await?;
        order.push("admit");
        (
            json!({"operation":"revoke","positive_commit":true}),
            second,
            proof,
            vec![first_pid, second_pid],
        )
    } else {
        let mut first = sessions.admission().await?;
        let mut second = sessions.permission().await?;
        let first_pid = first.backend_id().await?;
        let second_pid = second.backend_id().await?;
        let fence = first.fence(&typed, &config, &mut first_clock).await?;
        let ids = identities(input, "S1");
        let second_request = request.clone();
        let second_sample = sample.clone();
        let change = mutation(input)?;
        let task =
            tokio::spawn(
                async move { revoke(second, second_request, change, second_sample).await },
            );
        let proof = blocker(service, second_pid, first_pid).await?;
        let observation = complete(fence, &ids, false).await?;
        order.push("admit");
        first.close().await?;
        let second = join_actor(task).await?;
        order.push("revoke");
        (
            actor_result(Ok(observation), &ids, first_pid, first_clock.calls),
            second,
            proof,
            vec![first_pid, second_pid],
        )
    };
    for pid in &pids {
        authority_probes::absent(service, *pid).await?;
    }
    let before_s3 = shared::markers(service).await?;
    let mut fresh_observation = Value::Null;
    let mut fresh_pid = Value::Null;
    if !cas {
        let mut fresh = sessions.admission().await?;
        let pid = fresh.backend_id().await?;
        if pids.contains(&pid) {
            return Err("S3 did not observe a distinct fresh backend".into());
        }
        fresh_pid = json!(pid);
        let fresh_request = shared::request(&input["post_revocation_request"]);
        let mut clock = shared::SuppliedClock::new(&sample);
        let result = match fresh.fence(&fresh_request, &config, &mut clock).await {
            Ok(fence) => {
                let observed = shared::observation(fence.observation());
                fence.rollback().await?;
                Ok(observed)
            }
            Err(error) => Err(error),
        };
        fresh.close().await?;
        authority_probes::absent(service, pid).await?;
        let after_s3 = shared::markers(service).await?;
        if before_s3 != after_s3 {
            return Err("fresh denied/read-only probe changed markers".into());
        }
        fresh_observation = match result {
            Ok(observation) => {
                json!({"session":"S3","decision":"admit","problem_expectation":null,"marker_rows_added":[],"foreign_ids_disclosed":[],"observation":observation})
            }
            Err(error) => {
                json!({"session":"S3","decision":"deny","problem_expectation":shared::problem(error),"marker_rows_added":[],"foreign_ids_disclosed":[]})
            }
        };
    }
    let raw_markers = shared::markers(service).await?;
    let committed: Vec<String> = [&first_result, &second_result]
        .into_iter()
        .filter_map(|v| v["committed_observation_id"].as_str().map(str::to_owned))
        .collect();
    if raw_markers.as_array().unwrap().iter().any(|r| {
        !r["observation_id"]
            .as_str()
            .is_some_and(|id| committed.iter().any(|known| known == id))
    }) {
        return Err("stored marker lacks its positive owned completion".into());
    }
    let (markers, observations) = normalized_markers(&raw_markers);
    let mut observer_clock = shared::SuppliedClock::new(&sample);
    let state = shared::snapshot(service, &mut observer_clock, false).await?;
    let census = shared::census(service).await?;
    let admission = if first_result.get("decision").is_some() {
        &first_result
    } else {
        &second_result
    };
    let mut result = json!({"canonical_linearization":order,"accepted_effect_count":raw_markers.as_array().unwrap().len(),
        "admission_problem_expectation":admission["problem"],"admission_decision":admission["decision"],"canonical_snapshot_after":state,
        "marker_rows_after":markers,"admission_observations_after":observations,"canonical_command_receipts_after":[],"product_effects_after":[],"writer_enabled_after":false,
        "locks_released":census["writers"]==0,"foreign_ids_disclosed":[],"post_revocation_observation":fresh_observation,
        "barrier":barrier,"actor_results":[first_result,second_result],"former_writer_pids":pids,"fresh_s3_pid":fresh_pid,"census":census,"accounting":shared::accounting(service).await?,"seed_observation":seed_observation});
    if cas {
        let ids_a = identities(input, "S1");
        let ids_b = identities(input, "S2");
        result["accepted_commands"] = json!([ids_a["command_identity"]]);
        result["conflicted_commands"] = if result["actor_results"][1]["failure"] == "Conflict" {
            json!([ids_b["command_identity"]])
        } else {
            json!([])
        };
        result["resource_control_after"] =
            shared::observed_resource(&result["canonical_snapshot_after"], &request["resource"])?
                .ok_or("CAS resource readback missing")?["control_revision"]
                .clone();
        result["duplicate_effect_count"] = json!(
            raw_markers
                .as_array()
                .unwrap()
                .len()
                .saturating_sub(committed.len())
        );
        result["second_writer_problem_expectation"] = result["actor_results"][1]["problem"].clone();
    } else {
        let changed = &input["revocation_change"];
        result["admission_after_revoke"] =
            result["post_revocation_observation"]["decision"].clone();
        result["post_revocation_disclosure"] =
            result["post_revocation_observation"]["decision"].clone();
        result["denied_effect_count"] = json!(
            raw_markers
                .as_array()
                .unwrap()
                .len()
                .saturating_sub(committed.len())
        );
        if changed.get("membership_id").is_some() {
            let index =
                authority_index(&result["canonical_snapshot_after"], "memberships", changed)?;
            let actual = &result["canonical_snapshot_after"]["memberships"][index];
            let state = actual["state"].clone();
            let revision = actual["revision"].clone();
            result["membership_active_after"] = json!(state == "active");
            result["membership_revision_after"] = revision;
        } else if changed.get("grant_id").is_some() {
            let index = authority_index(&result["canonical_snapshot_after"], "grants", changed)?;
            let actual = &result["canonical_snapshot_after"]["grants"][index];
            let state = actual["state"].clone();
            let revision = actual["revision"].clone();
            result["target_authority_after"] = state;
            result["revision_after"] = revision;
        } else {
            let index = authority_index(&result["canonical_snapshot_after"], "policies", changed)?;
            let actual = &result["canonical_snapshot_after"]["policies"][index];
            let denied = !actual["purposes"]
                .as_array()
                .unwrap()
                .contains(&request["purpose"]);
            let epoch = actual["epoch"].clone();
            result["target_authority_after"] = json!(if denied { "revoked" } else { "active" });
            result["revision_after"] = epoch;
        }
    }
    Ok(result)
}

pub async fn blocker(service: &Service, waiter: i32, owner: i32) -> Result<Value> {
    service.with_admin(true, async |connection| {
    let started = Instant::now();
    let proof=timeout(Duration::from_millis(400),async {
        loop {
            let query=sqlx::query_scalar::<_,String>("SELECT jsonb_build_object('waiter',$1::integer,'owner',$2::integer,'blockers',pg_blocking_pids($1),'waiter_state',(SELECT state FROM pg_stat_activity WHERE pid=$1),'wait_event_type',(SELECT wait_event_type FROM pg_stat_activity WHERE pid=$1),'owner_state',(SELECT state FROM pg_stat_activity WHERE pid=$2),'locks',(SELECT coalesce(jsonb_agg(jsonb_build_object('pid',pid,'type',locktype,'relation',relation,'page',page,'tuple',tuple,'transaction',transactionid::text,'mode',mode,'granted',granted) ORDER BY pid,locktype,mode),'[]'::jsonb) FROM pg_locks WHERE pid IN ($1,$2) AND locktype IN ('transactionid','tuple')))::text").bind(waiter).bind(owner);
            let actual=fetch_json(&mut *connection,query,16384).await?;
            if actual["blockers"]==json!([owner]) && actual["wait_event_type"]=="Lock" {return Ok::<_,Box<dyn std::error::Error+Send+Sync>>(actual);}
            tokio::task::yield_now().await;
        }
    }).await??;
    let mut proof = proof;
    proof["barrier_elapsed_ms"] = json!(started.elapsed().as_millis());
    Ok(proof)
    }).await
}

pub fn milliseconds(value: &str) -> Result<u64> {
    if let Some(v) = value.strip_suffix("ms") {
        return Ok(v.parse()?);
    }
    if let Some(v) = value.strip_suffix('s') {
        return v
            .parse::<u64>()?
            .checked_mul(1000)
            .ok_or("timeout conversion overflow".into());
    }
    Err("unadmitted actual PostgreSQL timeout unit".into())
}

pub fn probe_query(input: &Value, request: &Value) -> Result<(String, Vec<String>)> {
    let probe = &input["probe"];
    let relation = probe["relation"].as_str().unwrap_or("resources");
    let org = shared::text(&request["selected_workspace"]["organization_id"]).to_owned();
    let domain = shared::text(&request["selected_workspace"]["domain_id"]).to_owned();
    let key = probe["key"].as_str().unwrap_or("");
    let (table, predicate, values) = match relation {
        "workspace_policies" => (
            "workspace_policies",
            "organization_id=$1::uuid AND domain_id=$2::uuid",
            vec![org, domain],
        ),
        "source_policies" => ("source_policies", "policy_id=$1", vec![key.to_owned()]),
        "organizations" => (
            "organizations",
            "organization_id=$1::uuid",
            vec![key.to_owned()],
        ),
        "domains" => (
            "domains",
            "organization_id=$1::uuid AND domain_id=$2::uuid",
            vec![org, key.to_owned()],
        ),
        "principals" => ("principals", "principal_id=$1::uuid", vec![key.to_owned()]),
        "memberships" => (
            "memberships",
            "organization_id=$1::uuid AND membership_id=$2::uuid",
            vec![org, key.to_owned()],
        ),
        "immutable_chains" => (
            "immutable_chains",
            "organization_id=$1::uuid AND domain_id=$2::uuid AND chain_id=$3::uuid",
            vec![org, domain, key.to_owned()],
        ),
        "immutable_members" => {
            let ordinal = match key {
                "ordinal0" => "0",
                "ordinal1" => "1",
                _ => return Err("unadmitted sealed probe ordinal".into()),
            };
            (
                "immutable_members",
                "organization_id=$1::uuid AND domain_id=$2::uuid AND chain_id=$3::uuid AND ordinal=$4::smallint",
                vec![
                    org,
                    domain,
                    shared::text(&request["selected_chain_id"]).to_owned(),
                    ordinal.to_owned(),
                ],
            )
        }
        "grants" => (
            "grants",
            "organization_id=$1::uuid AND domain_id=$2::uuid AND grant_id=$3::uuid",
            vec![org, domain, key.to_owned()],
        ),
        "resources" => {
            let resource = input["probe_resource_id"].as_str().unwrap_or(key);
            (
                "resources",
                "organization_id=$1::uuid AND domain_id=$2::uuid AND resource_kind=$3 AND resource_id=$4::uuid",
                vec![
                    org,
                    domain,
                    shared::text(&request["resource"]["kind"]).to_owned(),
                    resource.to_owned(),
                ],
            )
        }
        _ => return Err("unadmitted tuple probe relation".into()),
    };
    let mode = probe["mode"]
        .as_str()
        .or(input["probe_mode"].as_str())
        .ok_or("missing tuple probe mode")?;
    let mode = match mode {
        "FOR NO KEY UPDATE" => "FOR NO KEY UPDATE",
        "FOR SHARE" => "FOR SHARE",
        _ => return Err("unadmitted tuple probe mode".into()),
    };
    // Every syntax fragment is one compiled alternative above; all keys bind.
    Ok((
        format!(
            "SELECT to_jsonb(t)::text FROM arboresce_authority.{table} t WHERE {predicate} {mode}"
        ),
        values,
    ))
}

async fn acquire(
    connection: PgConnection,
    request: Value,
    sql: String,
    values: Vec<String>,
) -> Result<Value> {
    crate::authority_service::with_connection(connection, async |connection| {
        with_rollback(connection, async |transaction| {
            authority_probes::context(transaction, &request, &json!([])).await?;
            sqlx::query(
                "SELECT epoch FROM arboresce_authority.namespace_guard WHERE singleton=1 FOR SHARE",
            )
            .execute(&mut *transaction)
            .await?;
            let row = query_json(transaction, sqlx::AssertSqlSafe(sql), &values, 16384).await?;
            Ok(row)
        })
        .await
    })
    .await
}

pub async fn child_lock(
    service: &Service,
    input: &Value,
    request: &Value,
    clock: &mut shared::SuppliedClock,
) -> Result<Value> {
    let request = request.clone();
    let selected = shared::request(&request);
    let config = shared::configuration();
    let sessions = service.harness.authority_sessions();
    let mut first = sessions.admission().await?;
    let first_pid = first.backend_id().await?;
    let mut second = authority_probes::raw_role(service, true).await?;
    let second_pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&mut second)
        .await?;
    let settings: (String, String) = sqlx::query_as(
        "SELECT current_setting('lock_timeout'),current_setting('transaction_timeout')",
    )
    .fetch_one(&mut second)
    .await?;
    let census = shared::census(service).await?;
    let fence = first.fence(&selected, &config, clock).await?;
    let observation = shared::observation(fence.observation());
    let (sql, values) = probe_query(input, &request)?;
    let second_request = request.clone();
    let task = tokio::spawn(async move { acquire(second, second_request, sql, values).await });
    let (proof, row, after_release) = if input["operation"] == "unselected_child_lock_probe" {
        // A successful complete second transaction must precede first-terminal.
        let row = timeout(Duration::from_millis(400), task).await???;
        fence.rollback().await?;
        (Value::Null, row, false)
    } else {
        let proof = blocker(service, second_pid, first_pid).await?;
        fence.rollback().await?;
        let row = timeout(Duration::from_millis(400), task).await???;
        (proof, row, true)
    };
    let same_workspace = row
        .get("organization_id")
        .is_none_or(|v| *v == request["selected_workspace"]["organization_id"])
        && row
            .get("domain_id")
            .is_none_or(|v| *v == request["selected_workspace"]["domain_id"]);
    let key = &input["probe"]["key"];
    let same_key = match input["probe"]["relation"].as_str() {
        Some("workspace_policies") => same_workspace,
        Some("source_policies") => row["policy_id"] == *key,
        Some("organizations") => row["organization_id"] == *key,
        Some("domains") => row["domain_id"] == *key,
        Some("principals") => row["principal_id"] == *key,
        Some("memberships") => row["membership_id"] == *key,
        Some("immutable_chains") => row["chain_id"] == *key,
        Some("immutable_members") => {
            row["chain_id"] == request["selected_chain_id"]
                && row["ordinal"]
                    == if key == "ordinal0" {
                        json!(0)
                    } else {
                        json!(1)
                    }
        }
        Some("grants") => row["grant_id"] == *key,
        _ => row["resource_id"] == observation["resource_id"],
    };
    let same_backend = first.backend_id().await? == first_pid;
    first.close().await?;
    authority_probes::absent(service, first_pid).await?;
    authority_probes::absent(service, second_pid).await?;
    Ok(
        json!({"observed_blocker":if proof["blockers"]==json!([first_pid]) {"exact admitting backend"} else {"none"},
        "blocking_before_release":!proof.is_null(),"acquires_after_positive_owner_terminal":after_release,
        "same_selected_row":same_key && same_workspace,"ordinary_lock_ms_max":milliseconds(&settings.0)?,"ordinary_transaction_ms_max":milliseconds(&settings.1)?,
        "blocked":!proof.is_null(),"acquires_before_owner_terminal":!after_release,"barrier":proof,"row":row,
        "same_admitting_backend":same_backend,"writer_pids":[first_pid,second_pid],"census_during":census}),
    )
}
