//! Actual owned SQL probes. Expected values never enter these producers.

use crate::{
    authority_service::{Result, Service, query_json, with_connection, with_rollback},
    current_authority as shared,
    implementation::{AuthorityFailure, PermissionChange, SchemaVersion},
};
use serde_json::{Value, json};
use sqlx::{Connection, PgConnection, Row};
use std::{
    io::{Read, Seek, SeekFrom},
    os::unix::fs::MetadataExt,
    path::PathBuf,
    time::Duration,
};
use tokio::time::timeout;

pub async fn raw_role(service: &Service, permission: bool) -> Result<PgConnection> {
    let role = if permission {
        "harness_authority_permission"
    } else {
        "harness_authority_admission"
    };
    let options = service
        .harness
        .options(false)
        .map_err(|_| "invalid owned role options")?
        .username(role)
        .application_name("arboresce_authority_probe");
    Ok(timeout(Duration::from_secs(1), PgConnection::connect_with(&options)).await??)
}

pub async fn context(
    connection: &mut PgConnection,
    request: &Value,
    missing: &Value,
) -> Result<()> {
    let absent = |key: &str| {
        missing
            .as_array()
            .is_some_and(|rows| rows.iter().any(|v| v == key))
    };
    let field = |key: &str, v: &Value| {
        if absent(key) {
            String::new()
        } else {
            v.as_str().unwrap_or("").to_owned()
        }
    };
    sqlx::query("SELECT set_config('arboresce.organization_id',$1,true),set_config('arboresce.domain_id',$2,true),set_config('arboresce.principal_id',$3,true),set_config('arboresce.chain_id',$4,true)")
        .bind(field("organization_id",&request["selected_workspace"]["organization_id"]))
        .bind(field("domain_id",&request["selected_workspace"]["domain_id"]))
        .bind(field("principal_id",&request["executor_principal_id"]))
        .bind(field("chain_id",&request["selected_chain_id"]))
        .execute(connection).await?;
    Ok(())
}

async fn catalogue_value(service: &Service) -> Result<Value> {
    service.with_admin(false, async |connection| {
    let mut actual = query_json(&mut *connection, "SELECT jsonb_build_object('roles',(SELECT jsonb_agg(jsonb_build_object('name',rolname,'superuser',rolsuper,'bypass',rolbypassrls,'inherit',rolinherit,'connections',rolconnlimit) ORDER BY rolname) FROM pg_roles WHERE rolname IN ('harness_authority_admission','harness_authority_permission')),'memberships',(SELECT coalesce(jsonb_agg(jsonb_build_object('member',pg_get_userbyid(member),'role',pg_get_userbyid(roleid)) ORDER BY member,roleid),'[]'::jsonb) FROM pg_auth_members WHERE member IN (SELECT oid FROM pg_roles WHERE rolname IN ('harness_authority_admission','harness_authority_permission'))),'tables',(SELECT jsonb_agg(jsonb_build_object('name',c.relname,'owner',pg_get_userbyid(c.relowner),'rls',c.relrowsecurity,'force',c.relforcerowsecurity,'acl',c.relacl) ORDER BY c.relname) FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname='arboresce_authority' AND c.relkind='r'),'policies',(SELECT jsonb_agg(jsonb_build_object('table',c.relname,'name',p.polname,'command',p.polcmd,'permissive',p.polpermissive,'roles',p.polroles,'using',pg_get_expr(p.polqual,p.polrelid),'check',pg_get_expr(p.polwithcheck,p.polrelid)) ORDER BY c.relname,p.polname) FROM pg_policy p JOIN pg_class c ON c.oid=p.polrelid JOIN pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname='arboresce_authority'))::text", &[], 131072).await?;
    let constraints = shared::constraints(&mut *connection).await?;
    let extra = query_json(&mut *connection, "SELECT jsonb_build_object(\
        'schemas',(SELECT jsonb_agg(jsonb_build_object('name',nspname,'owner',pg_get_userbyid(nspowner),'acl',nspacl) ORDER BY nspname) FROM pg_namespace WHERE nspname IN ('public','arboresce_authority')),\
        'columns',(SELECT coalesce(jsonb_agg(value ORDER BY relation,ordinal),'[]'::jsonb) FROM (SELECT c.relname AS relation,a.attnum AS ordinal,jsonb_build_object('table',c.relname,'name',a.attname,'ordinal',a.attnum,'type',format_type(a.atttypid,a.atttypmod),'not_null',a.attnotnull,'default',pg_get_expr(d.adbin,d.adrelid),'acl',a.attacl) AS value FROM pg_attribute a JOIN pg_class c ON c.oid=a.attrelid JOIN pg_namespace n ON n.oid=c.relnamespace LEFT JOIN pg_attrdef d ON d.adrelid=c.oid AND d.adnum=a.attnum WHERE n.nspname='arboresce_authority' AND c.relkind='r' AND a.attnum>0 AND NOT a.attisdropped ORDER BY c.relname,a.attnum LIMIT 1025) bounded),\
        'functions',(SELECT coalesce(jsonb_agg(value ORDER BY schema,name,arguments),'[]'::jsonb) FROM (SELECT n.nspname AS schema,p.proname AS name,pg_get_function_identity_arguments(p.oid) AS arguments,jsonb_build_object('schema',n.nspname,'name',p.proname,'arguments',pg_get_function_identity_arguments(p.oid),'definition',pg_get_functiondef(p.oid),'acl',p.proacl,'security_definer',p.prosecdef) AS value FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace WHERE n.nspname NOT LIKE 'pg_%' AND n.nspname<>'information_schema' ORDER BY n.nspname,p.proname,pg_get_function_identity_arguments(p.oid) LIMIT 65) bounded))::text", &[], 131072).await?;
    actual["constraints"] = constraints;
    for (key, value) in extra
        .as_object()
        .ok_or("invalid extra catalogue")?
    {
        actual[key] = value.clone();
    }
    if serde_json::to_vec(&actual)?.len() > 131072 {
        return Err("complete catalogue observation bound exceeded".into());
    }
    Ok(actual)
    }).await
}

async fn catalogue(service: &Service) -> Result<Value> {
    shared::retain_observation(&catalogue_value(service).await?)
}

/// Preserve full physical values, including malformed precision and infinity,
/// independently of the normal canonical presentation used by assertions.
async fn physical_authority(service: &Service) -> Result<Value> {
    let fields: Vec<_> = crate::implementation::authority::catalogue_policy::TABLES
        .iter()
        .map(|table| format!("'{table}',(SELECT coalesce(jsonb_agg(value ORDER BY value::text),'[]'::jsonb) FROM (SELECT to_jsonb(r) AS value FROM arboresce_authority.{table} r LIMIT 34) bounded)"))
        .collect();
    let sql = format!("SELECT jsonb_build_object({})::text", fields.join(","));
    let value = service
        .json(false, sqlx::AssertSqlSafe(sql), &[], 131072)
        .await?;
    if value
        .as_object()
        .ok_or("invalid physical authority")?
        .values()
        .any(|v| v.as_array().is_none_or(|rows| rows.len() >= 34))
    {
        return Err("physical authority row lookahead exceeded".into());
    }
    Ok(value)
}

fn catalogue_without_target(mut value: Value, input: &Value) -> Result<(Value, Value)> {
    let fault = shared::text(&input["fault"]);
    let (group, field) = match fault {
        "policy_using" => ("policies", Some("using")),
        "policy_roles" => ("policies", Some("roles")),
        "policy_check" => ("policies", Some("check")),
        "column_insert" | "column_references" | "column_grant_option" => ("columns", Some("acl")),
        "revoke_select" | "table_delete" | "public_select" => ("tables", Some("acl")),
        "revoke_schema_usage" => ("schemas", Some("acl")),
        "disable_rls" => ("tables", Some("rls")),
        "extra_relation" => ("tables", None),
        "extra_column" => ("columns", None),
        "user_function" => ("functions", None),
        _ => return Err("unadmitted isolated catalogue projection".into()),
    };
    let selected = |row: &Value| match group {
        "policies" => row["table"] == input["table"] && row["name"] == input["policy"],
        "columns" => row["table"] == input["table"] && row["name"] == input["column"],
        "schemas" => row["name"] == input["schema"],
        "functions" => {
            row["schema"] == input["schema"]
                && row["name"] == input["function"]
                && row["arguments"] == ""
        }
        _ => row["name"] == input["table"],
    };
    let rows = value[group]
        .as_array_mut()
        .ok_or("missing catalogue projection group")?;
    let mut targets = Vec::new();
    for row in rows.iter_mut().filter(|v| selected(v)) {
        if let Some(field) = field {
            let field = row
                .as_object_mut()
                .ok_or("invalid catalogue target")?
                .remove(field)
                .ok_or("missing selected catalogue field")?;
            targets.push(field);
        } else {
            targets.push(row.clone());
        }
    }
    if field.is_none() {
        rows.retain(|v| !selected(v));
    } else if targets.len() != 1 {
        return Err("isolated catalogue field target is not unique".into());
    }
    if fault == "extra_relation" {
        let columns = value["columns"]
            .as_array_mut()
            .ok_or("missing actual columns")?;
        targets.extend(
            columns
                .iter()
                .filter(|v| v["table"] == input["table"])
                .cloned(),
        );
        columns.retain(|v| v["table"] != input["table"]);
    }
    Ok((value, json!(targets)))
}

async fn isolated_catalogue(service: &Service, input: &Value) -> Result<Value> {
    let before = catalogue_value(service).await?;
    let sql = match shared::text(&input["fault"]) {
        "policy_using" => {
            "ALTER POLICY internal_complete_context ON arboresce_authority.memberships USING (true)"
        }
        "policy_roles" => {
            "ALTER POLICY internal_complete_context ON arboresce_authority.memberships TO harness_authority_admission"
        }
        "policy_check" => {
            "ALTER POLICY internal_complete_context ON arboresce_authority.memberships WITH CHECK (true)"
        }
        "revoke_select" => {
            "REVOKE SELECT ON arboresce_authority.memberships FROM harness_authority_admission"
        }
        "column_insert" => {
            "GRANT INSERT(state) ON arboresce_authority.memberships TO harness_authority_admission"
        }
        "column_references" => {
            "GRANT REFERENCES(state) ON arboresce_authority.memberships TO harness_authority_admission"
        }
        "table_delete" => {
            "GRANT DELETE ON arboresce_authority.memberships TO harness_authority_admission"
        }
        "revoke_schema_usage" => {
            "REVOKE USAGE ON SCHEMA arboresce_authority FROM harness_authority_admission"
        }
        "column_grant_option" => {
            "GRANT UPDATE(lock_guard) ON arboresce_authority.memberships TO harness_authority_admission WITH GRANT OPTION"
        }
        "public_select" => "GRANT SELECT ON arboresce_authority.memberships TO PUBLIC",
        "disable_rls" => "ALTER TABLE arboresce_authority.memberships DISABLE ROW LEVEL SECURITY",
        "extra_relation" => {
            "CREATE TABLE arboresce_authority.qualification_extra_bound (value integer)"
        }
        "extra_column" => {
            "ALTER TABLE arboresce_authority.memberships ADD COLUMN qualification_extra_bound integer"
        }
        "user_function" => {
            "CREATE FUNCTION public.qualification_extra_bound() RETURNS integer LANGUAGE sql AS 'SELECT 1'"
        }
        _ => return Err("unadmitted isolated catalogue fault".into()),
    };
    service
        .with_admin(false, async |connection| {
            sqlx::query(sql).execute(connection).await?;
            Ok(())
        })
        .await?;
    let after = catalogue_value(service).await?;
    let (before_rest, before_target) = catalogue_without_target(before.clone(), input)?;
    let (after_rest, after_target) = catalogue_without_target(after.clone(), input)?;
    Ok(
        json!({"catalogue_target_changed":before_target != after_target,
        "unrelated_catalogue_unchanged":before_rest == after_rest,
        "catalogue_boundary_before":shared::retain_observation(&before)?,
        "catalogue_boundary_after":shared::retain_observation(&after)?,
        "catalogue_target_before":before_target,"catalogue_target_after":after_target}),
    )
}

async fn fault(service: &Service, input: &Value) -> Result<()> {
    let operation = shared::text(&input["operation"]);
    let sql = match operation {
        "role_fault" => match shared::text(&input["attribute"]) {
            "rolsuper" => "ALTER ROLE harness_authority_admission SUPERUSER",
            "rolbypassrls" => "ALTER ROLE harness_authority_admission BYPASSRLS",
            _ => return Err("unadmitted role attribute".into()),
        },
        "role_membership_fault" => "GRANT harness_migrator TO harness_authority_admission",
        "table_security_fault" => {
            "ALTER TABLE arboresce_authority.memberships NO FORCE ROW LEVEL SECURITY"
        }
        "column_acl_fault" => {
            "GRANT UPDATE(state) ON arboresce_authority.memberships TO harness_authority_admission"
        }
        "policy_fault" | "direct_guard_update_with_extra_permissive_policy" => {
            "CREATE POLICY extra_permissive_update ON arboresce_authority.memberships FOR UPDATE TO PUBLIC USING (true) WITH CHECK (true)"
        }
        _ => return Err("unadmitted catalogue fault".into()),
    };
    service
        .with_admin(false, async |connection| {
            sqlx::query(sql).execute(&mut *connection).await?;
            Ok(())
        })
        .await
}

async fn adapter(
    service: &Service,
    raw_request: &Value,
    clock: &mut shared::SuppliedClock,
    ids: &Value,
    cas: bool,
) -> Result<Value> {
    let sessions = service.harness.authority_sessions();
    let mut session = match sessions.admission().await {
        Ok(session) => session,
        Err(error) => return Ok(json!({"failure":error.to_string()})),
    };
    let request = shared::request(raw_request);
    let config = shared::configuration();
    let result = match session.fence(&request, &config, clock).await {
        Ok(fence) => {
            let observed = shared::observation(fence.observation());
            let terminal = if cas {
                fence
                    .compare_and_marker(
                        shared::id(&ids["marker_id"]),
                        shared::id(&ids["observation_id"]),
                    )
                    .await
            } else {
                fence.rollback().await
            };
            terminal.map(|()| observed)
        }
        Err(error) => Err(error),
    };
    session.close().await?;
    Ok(match result {
        Ok(observed) => json!({"failure":null,"decision":"admit","observation":observed}),
        Err(error) => {
            json!({"failure":error.to_string(),"semantic_problem":shared::problem(error)})
        }
    })
}

async fn direct_dml(service: &Service, input: &Value, request: &Value) -> Result<Value> {
    let permission = input["role"] == "harness_authority_permission";
    let (rejected, code, message, column) = with_connection(
        raw_role(service, permission).await?,
        async |connection| {
            with_rollback(connection, async |transaction| {
    context(transaction, request, &json!([])).await?;
    sqlx::query(
        "SELECT epoch FROM arboresce_authority.namespace_guard WHERE singleton=1 FOR SHARE",
    )
    .execute(&mut *transaction)
    .await?;
    let target = &input["target"];
    let (sql, column) = if input["table"] == "immutable_members" {
        (
            "UPDATE arboresce_authority.immutable_members SET control_revision=$4::bigint WHERE organization_id=$1::uuid AND domain_id=$2::uuid AND chain_id=$3::uuid AND ordinal=$5::smallint",
            "control_revision",
        )
    } else if input["operation"] == "direct_authority_update" {
        (
            "UPDATE arboresce_authority.memberships SET state=$4 WHERE organization_id=$1::uuid AND domain_id=$2::uuid AND principal_id=$3::uuid AND membership_id=$5::uuid",
            "state",
        )
    } else {
        (
            "UPDATE arboresce_authority.memberships SET lock_guard=$4::boolean WHERE organization_id=$1::uuid AND domain_id=$2::uuid AND principal_id=$3::uuid AND membership_id=$5::uuid",
            "lock_guard",
        )
    };
    let value = match &input["value"] {
        Value::Bool(v) => v.to_string(),
        v => shared::text(v).to_owned(),
    };
    let mut query = sqlx::query(sql)
        .bind(shared::text(&target["organization_id"]))
        .bind(shared::text(&target["domain_id"]));
    query = if permission {
        query
            .bind(shared::text(&target["chain_id"]))
            .bind(value)
            .bind(target["ordinal"].as_i64().unwrap() as i16)
    } else {
        query
            .bind(shared::text(&target["principal_id"]))
            .bind(value)
            .bind(shared::text(&target["membership_id"]))
    };
    let result = query.execute(&mut *transaction).await;
    let (rejected, code, message) = match result {
        Ok(r) => (false, None, json!({"affected":r.rows_affected()})),
        Err(error) => {
            let database = error
                .as_database_error()
                .ok_or("DML probe failed outside PostgreSQL")?;
            (
                true,
                database.code().map(|v| v.to_string()),
                json!(database.message()),
            )
        }
    };
    Ok((rejected, code, message, column))
            }).await
        },
    ).await?;
    let ordinary = if input["operation"] == "direct_guard_update_with_extra_permissive_policy" {
        let sessions = service.harness.authority_sessions();
        match sessions.admission().await {
            Err(error) => json!(error.to_string()),
            Ok(session) => {
                session.close().await?;
                Value::Null
            }
        }
    } else {
        Value::Null
    };
    if rejected && code.as_deref() != Some("42501") {
        return Err("DML failed for an unrelated SQL cause".into());
    }
    let restrictive = rejected
        && column == "lock_guard"
        && message
            .as_str()
            .is_some_and(|m| m.contains("row-level security"));
    Ok(
        json!({"database_statement":if restrictive && !ordinary.is_null() {"rejected by restrictive WITHCHECKfalse"} else if rejected {"rejected"} else {"accepted"},
        "sqlstate":code,"diagnostic":message,"ordinary_begin":ordinary}),
    )
}

async fn context_probe(
    service: &Service,
    input: &Value,
    request: &Value,
    clock: &mut shared::SuppliedClock,
) -> Result<Value> {
    if input["operation"] == "direct_missing_context" {
        return with_connection(raw_role(service, false).await?, async |connection| {
            with_rollback(connection, async |transaction| {
                context(transaction, request, &input["missing"]).await?;
                let mut counts = json!({});
                let mut total = 0_i64;
                for table in crate::implementation::authority::catalogue_policy::TABLES {
                    let sql = format!("SELECT count(*) FROM arboresce_authority.{table}");
                    let count: i64 = sqlx::query_scalar(sqlx::AssertSqlSafe(sql))
                        .fetch_one(&mut *transaction)
                        .await?;
                    counts[table] = json!(count);
                    total += count;
                }
                Ok(json!({"visible_authority_rows":total,"table_counts":counts}))
            })
            .await
        })
        .await;
    }
    let sessions = service.harness.authority_sessions();
    let mut session = sessions.admission().await?;
    if input["operation"] == "retained_session_context" {
        sqlx::query("SELECT set_config('arboresce.chain_id',$1,false)")
            .bind(shared::text(&input["value"]))
            .execute(session.connection.as_mut().ok_or("absent owned session")?)
            .await?;
        let raw = shared::request(request);
        let config = shared::configuration();
        let result = session.observe_current(&raw, &config, clock).await;
        session.close().await?;
        return Ok(json!({"failure":result.err().map(|e|e.to_string())}));
    }
    let raw = shared::request(request);
    let config = shared::configuration();
    let before = session.backend_id().await?;
    session.observe_current(&raw, &config, clock).await?;
    let retained:bool=sqlx::query_scalar("SELECT nullif(current_setting('arboresce.organization_id',true),'') IS NOT NULL OR nullif(current_setting('arboresce.domain_id',true),'') IS NOT NULL OR nullif(current_setting('arboresce.principal_id',true),'') IS NOT NULL OR nullif(current_setting('arboresce.chain_id',true),'') IS NOT NULL")
        .fetch_one(session.connection.as_mut().ok_or("absent owned session")?).await?;
    session.observe_current(&raw, &config, clock).await?;
    let after = session.backend_id().await?;
    session.close().await?;
    Ok(
        json!({"retained_context":retained,"same_physical_backend":before==after,"backend_before":before,"backend_after":after}),
    )
}

async fn slots(service: &Service, input: &Value) -> Result<Value> {
    let sessions = service.harness.authority_sessions();
    let mut first = sessions.admission().await?;
    let first_pid = first.backend_id().await?;
    let mut second = sessions.permission().await?;
    let second_pid = second.backend_id().await?;
    let before = shared::census(service).await?;
    if input["operation"] == "two_slot_then_third_open" {
        // A fresh factory cannot fail on the first factory's local semaphore.
        // Its actual PostgreSQL handshake must encounter the database limit.
        let other = service.harness.authority_sessions();
        let rejection = match other.admission().await {
            Err(e) => Some(e),
            Ok(session) => {
                session.close().await?;
                None
            }
        };
        let raw = raw_role(service, false).await;
        let server = match raw {
            Err(error) => error
                .downcast_ref::<sqlx::Error>()
                .and_then(|e| e.as_database_error())
                .and_then(|e| e.code())
                .map(|v| v.to_string()),
            Ok(connection) => {
                with_connection(connection, async |_| Ok(())).await?;
                None
            }
        };
        let after = shared::census(service).await?;
        first.close().await?;
        second.close().await?;
        if server.as_deref() != Some("53300") {
            return Err("third connection did not observe server capacity rejection".into());
        }
        return Ok(
            json!({"positive_writer_backends":before["writers"],"third_open":if rejection==Some(AuthorityFailure::Unavailable) {"rejected"} else {"accepted"},
            "combined_writer_census_max":before["writers"].as_i64().unwrap().max(after["writers"].as_i64().unwrap()),"admin_decision_count":0,
            "writer_pids":[first_pid,second_pid],"server_sqlstate":server,"census_before":before,"census_after":after}),
        );
    }
    first.close().await?;
    absent(service, first_pid).await?;
    let closed = shared::census(service).await?;
    let old_absent = closed["backends"]
        .as_array()
        .unwrap()
        .iter()
        .all(|r| r["pid"] != first_pid);
    let mut fresh = sessions.admission().await?;
    let fresh_pid = fresh.backend_id().await?;
    let retained:bool=sqlx::query_scalar("SELECT nullif(current_setting('arboresce.organization_id',true),'') IS NOT NULL OR nullif(current_setting('arboresce.domain_id',true),'') IS NOT NULL OR nullif(current_setting('arboresce.principal_id',true),'') IS NOT NULL OR nullif(current_setting('arboresce.chain_id',true),'') IS NOT NULL")
        .fetch_one(fresh.connection.as_mut().ok_or("absent owned replacement")?).await?;
    let replaced = shared::census(service).await?;
    second.close().await?;
    fresh.close().await?;
    Ok(
        json!({"combined_writer_census_max":before["writers"].as_i64().unwrap().max(replaced["writers"].as_i64().unwrap()),
        "replacement_backend":if fresh_pid!=first_pid && fresh_pid!=second_pid {"distinct from both former backends"} else {"reused"},
        "former_backend_closed":old_absent,"retained_context":retained,"writer_pids":[first_pid,second_pid,fresh_pid],"census_before":before,"census_closed":closed,"census_after":replaced}),
    )
}

async fn scalar_fault(connection: &mut PgConnection, input: &Value) -> Result<(u64, Value)> {
    let relation = shared::text(&input["relation"]);
    let field = input["field"].as_str().unwrap_or("created_at");
    let (column, cast, guard) = match (relation, field) {
        ("principals", "identity_profile") => (
            "identity_profile",
            "text",
            crate::implementation::authority::rows::token("identity_profile"),
        ),
        ("principals", "identity_version") => (
            "identity_version",
            "text",
            crate::implementation::authority::rows::token("identity_version"),
        ),
        ("principals", "public_key") => (
            "public_key",
            "bytea",
            "CASE WHEN octet_length(public_key)=32 THEN encode(public_key,'hex') END AS public_key"
                .to_owned(),
        ),
        ("memberships", "state") => (
            "state",
            "text",
            crate::implementation::authority::rows::token("state"),
        ),
        ("principals", "created_at") => (
            "created_at",
            "timestamptz",
            crate::implementation::authority::rows::instant("created_at"),
        ),
        ("principals", "revoked_at") => (
            "revoked_at",
            "timestamptz",
            crate::implementation::authority::rows::instant("revoked_at"),
        ),
        ("principals", "identity_standard_revision") => (
            "identity_standard_revision",
            "text",
            crate::implementation::authority::rows::provenance("identity_standard_revision"),
        ),
        ("principals" | "memberships" | "grants" | "immutable_members", "control_revision") => {
            ("control_revision", "bigint", "control_revision".to_owned())
        }
        ("grants", "issued_at") => (
            "issued_at",
            "timestamptz",
            crate::implementation::authority::rows::instant("issued_at"),
        ),
        ("grants", "expires_at") => (
            "expires_at",
            "timestamptz",
            crate::implementation::authority::rows::instant("expires_at"),
        ),
        _ => return Err("unadmitted scalar fault field".into()),
    };
    let target = &input["target"];
    let (predicate, keys) = match relation {
        "principals" => (
            "principal_id=$1::uuid",
            vec![shared::text(&target["principal_id"]).to_owned()],
        ),
        "memberships" => (
            "organization_id=$1::uuid AND domain_id=$2::uuid AND principal_id=$3::uuid AND membership_id=$4::uuid",
            vec![
                shared::text(&target["organization_id"]).to_owned(),
                shared::text(&target["domain_id"]).to_owned(),
                shared::text(&target["principal_id"]).to_owned(),
                shared::text(&target["membership_id"]).to_owned(),
            ],
        ),
        "grants" => (
            "organization_id=$1::uuid AND domain_id=$2::uuid AND grant_id=$3::uuid",
            vec![
                shared::text(&target["organization_id"]).to_owned(),
                shared::text(&target["domain_id"]).to_owned(),
                shared::text(&target["grant_id"]).to_owned(),
            ],
        ),
        "immutable_members" => (
            "organization_id=$1::uuid AND domain_id=$2::uuid AND chain_id=$3::uuid AND ordinal=$4::smallint",
            vec![
                shared::text(&target["organization_id"]).to_owned(),
                shared::text(&target["domain_id"]).to_owned(),
                shared::text(&target["chain_id"]).to_owned(),
                target["ordinal"]
                    .as_i64()
                    .ok_or("missing selected ordinal")?
                    .to_string(),
            ],
        ),
        _ => return Err("unadmitted scalar fault relation".into()),
    };
    // SQL identifiers, casts and predicates come exclusively from the above
    // compiled allowlists. Fixture values enter only as bound parameters.
    let sql = format!(
        "UPDATE arboresce_authority.{relation} SET {column}=${}::{cast} WHERE {predicate}",
        keys.len() + 1
    );
    let mut query = sqlx::query(sqlx::AssertSqlSafe(sql));
    for key in &keys {
        query = query.bind(key);
    }
    let changed = query
        .bind(shared::text(&input["value"]))
        .execute(&mut *connection)
        .await?
        .rows_affected();
    let sql = format!(
        "SELECT {column} IS NOT NULL AS present,CASE WHEN octet_length({column}::text)<=128 THEN {column}::text END AS stored,{guard} FROM arboresce_authority.{relation} WHERE {predicate}"
    );
    let mut query = sqlx::query(sqlx::AssertSqlSafe(sql));
    for key in &keys {
        query = query.bind(key);
    }
    let row = query.fetch_one(connection).await?;
    let present: bool = row.try_get("present")?;
    let stored: Option<String> = row.try_get("stored")?;
    let value = if column == "control_revision" {
        json!(row.try_get::<i64, _>(column)?)
    } else {
        json!(row.try_get::<Option<String>, _>(column)?)
    };
    Ok((
        changed,
        json!({"present":present,"stored":stored,"guarded":value}),
    ))
}

async fn prepare_permission(service: &Service, input: &Value) -> Result<Value> {
    service.with_admin(false, async |connection| {
    let mut observations = Vec::new();
    for row in input["prepare"]
        .as_array()
        .ok_or("missing permission setup")?
    {
        let target = &row["target"];
        let workspace = &target["workspace"];
        let workspace_keys = || {
            vec![
                shared::text(&workspace["organization_id"]).to_owned(),
                shared::text(&workspace["domain_id"]).to_owned(),
            ]
        };
        let (relation, column, predicate, keys) = match shared::text(&row["family"]) {
            "membership" => {
                let mut keys = workspace_keys();
                keys.push(shared::text(&target["principal_id"]).to_owned());
                keys.push(shared::text(&target["membership_id"]).to_owned());
                (
                    "memberships",
                    "control_revision",
                    "organization_id=$1::uuid AND domain_id=$2::uuid AND principal_id=$3::uuid AND membership_id=$4::uuid",
                    keys,
                )
            }
            "grant" => {
                let mut keys = workspace_keys();
                keys.push(shared::text(&target["grant_id"]).to_owned());
                (
                    "grants",
                    "control_revision",
                    "organization_id=$1::uuid AND domain_id=$2::uuid AND grant_id=$3::uuid",
                    keys,
                )
            }
            "policy" => {
                let mut keys = workspace_keys();
                keys.push(shared::text(&target["policy"]).to_owned());
                (
                    "workspace_policies",
                    "epoch",
                    "organization_id=$1::uuid AND domain_id=$2::uuid AND policy=$3",
                    keys,
                )
            }
            "resource" => {
                let mut keys = workspace_keys();
                keys.push(shared::text(&target["kind"]).to_owned());
                keys.push(shared::text(&target["id"]).to_owned());
                (
                    "resources",
                    "control_revision",
                    "organization_id=$1::uuid AND domain_id=$2::uuid AND resource_kind=$3 AND resource_id=$4::uuid",
                    keys,
                )
            }
            "namespace" if target["singleton"] == 1 => {
                ("namespace_guard", "epoch", "singleton=1", vec![])
            }
            _ => return Err("unadmitted permission setup family".into()),
        };
        if row["field"] != column {
            return Err("permission setup field differs".into());
        }
        let sql = format!(
            "UPDATE arboresce_authority.{relation} SET {column}=${}::bigint WHERE {predicate}",
            keys.len() + 1
        );
        let mut query = sqlx::query(sqlx::AssertSqlSafe(sql));
        for key in &keys {
            query = query.bind(key);
        }
        if query
            .bind(shared::text(&row["value"]))
            .execute(&mut *connection)
            .await?
            .rows_affected()
            != 1
        {
            return Err("permission setup target is not exactly one row".into());
        }
        let sql =
            format!("SELECT {column}::text FROM arboresce_authority.{relation} WHERE {predicate}");
        let mut query = sqlx::query_scalar::<_, String>(sqlx::AssertSqlSafe(sql));
        for key in &keys {
            query = query.bind(key);
        }
        let actual = query.fetch_one(&mut *connection).await?;
        if actual != shared::text(&row["value"]) {
            return Err("permission setup readback differs".into());
        }
        observations
            .push(json!({"relation":relation,"field":column,"selected_keys":keys,"actual":actual}));
    }
    Ok(json!(observations))
    }).await
}

fn permission_change(input: &Value) -> Result<PermissionChange> {
    let change = &input["change"];
    let expected = shared::text(&change["expected"]).parse()?;
    let target = &change["target"];
    Ok(match shared::text(&change["kind"]) {
        "revoke_membership" => PermissionChange::RevokeMembership {
            principal: shared::id(&target["principal_id"]),
            expected,
        },
        "revoke_grant" => PermissionChange::RevokeGrant {
            grant: shared::id(&target["grant_id"]),
            expected,
        },
        "restrict_policy" => PermissionChange::RestrictPolicy {
            expected,
            purposes: change["purposes"]
                .as_array()
                .ok_or("missing restriction purposes")?
                .iter()
                .map(|p| shared::text(p).to_owned())
                .collect(),
        },
        "advance_resource" => PermissionChange::AdvanceResource { expected },
        _ => return Err("unadmitted permission change".into()),
    })
}

async fn permission_once(
    service: &Service,
    input: &Value,
    request: &Value,
    clock: &mut shared::SuppliedClock,
) -> Result<Value> {
    let sessions = service.harness.authority_sessions();
    let mut session = sessions.permission().await?;
    let pid = session.backend_id().await?;
    let request = shared::request(request);
    let result = match session
        .fence(&request, permission_change(input)?, clock)
        .await
    {
        Ok(fence) => fence.commit().await,
        Err(error) => Err(error),
    };
    session.close().await?;
    Ok(
        json!({"failure":result.err().map(|e|e.to_string()),"positive_commit":result.is_ok(),"pid":pid,"clock_calls":clock.calls}),
    )
}

async fn permission_boundary(service: &Service, input: &Value, request: &Value) -> Result<Value> {
    let samples = input["trusted_clock_samples"]
        .as_array()
        .ok_or("missing permission clock input")?;
    if samples.len() != 1 {
        return Err("permission clock input must contain one reusable sample".into());
    }
    let mut clock = shared::SuppliedClock::new(&samples[0]);
    let first = permission_once(service, input, request, &mut clock).await?;
    let first_state = physical_authority(service).await?;
    if input["replay"] == true {
        let second = permission_once(service, input, request, &mut clock).await?;
        let second_state = physical_authority(service).await?;
        Ok(
            json!({"first_failure":first["failure"],"first_positive_commit":first["positive_commit"],
            "second_failure":second["failure"],"second_authority_changed":first_state!=second_state,
            "first_completion":first,"second_completion":second,
            "physical_first":shared::retain_observation(&first_state)?,"physical_second":shared::retain_observation(&second_state)?}),
        )
    } else {
        Ok(first)
    }
}

fn permission_without_target(mut physical: Value, input: &Value) -> Result<Value> {
    let change = &input["change"];
    let target = &change["target"];
    let workspace = &target["workspace"];
    let (relation, allowed): (&str, &[&str]) = match shared::text(&change["kind"]) {
        "revoke_membership" => ("memberships", &["state", "control_revision"]),
        "revoke_grant" => ("grants", &["state", "control_revision", "revoked_at"]),
        "restrict_policy" => ("workspace_policies", &["epoch", "purposes"]),
        "advance_resource" => ("resources", &["control_revision"]),
        _ => return Err("unadmitted permission readback".into()),
    };
    let mut selected = 0;
    for row in physical[relation]
        .as_array_mut()
        .ok_or("missing physical permission relation")?
    {
        if row["organization_id"] != workspace["organization_id"]
            || row["domain_id"] != workspace["domain_id"]
        {
            continue;
        }
        let matches = match relation {
            "memberships" => {
                row["principal_id"] == target["principal_id"]
                    && row["membership_id"] == target["membership_id"]
            }
            "grants" => row["grant_id"] == target["grant_id"],
            "workspace_policies" => row["policy"] == target["policy"],
            _ => row["resource_kind"] == target["kind"] && row["resource_id"] == target["id"],
        };
        if matches {
            selected += 1;
            for field in allowed {
                row.as_object_mut()
                    .ok_or("invalid physical permission row")?
                    .remove(*field);
            }
        }
    }
    if selected != 1 {
        return Err("physical permission target is not exactly one row".into());
    }
    let namespace = physical["namespace_guard"]
        .as_array_mut()
        .ok_or("missing physical namespace")?;
    if namespace.len() != 1 || namespace[0]["singleton"] != 1 {
        return Err("invalid physical namespace target".into());
    }
    namespace[0]
        .as_object_mut()
        .ok_or("invalid namespace row")?
        .remove("epoch");
    for rows in physical
        .as_object_mut()
        .ok_or("invalid physical permission state")?
        .values_mut()
    {
        rows.as_array_mut()
            .ok_or("invalid physical permission relation")?
            .sort_by_key(Value::to_string);
    }
    Ok(physical)
}

fn canonical_permission_target(state: &Value, input: &Value) -> Result<Value> {
    let target = &input["change"]["target"];
    let family = match shared::text(&input["change"]["kind"]) {
        "restrict_policy" => "policies",
        "revoke_membership" => "memberships",
        _ => return Ok(Value::Null),
    };
    let mut selected = state[family]
        .as_array()
        .ok_or("missing canonical permission relation")?
        .iter()
        .filter(|row| {
            row["workspace"] == target["workspace"]
                && if family == "policies" {
                    row["policy"] == target["policy"]
                } else {
                    row["principal"]["principal_id"] == target["principal_id"]
                        && row["membership_id"] == target["membership_id"]
                }
        });
    let row = selected
        .next()
        .ok_or("missing canonical permission target")?
        .clone();
    if selected.next().is_some() {
        return Err("ambiguous canonical permission target".into());
    }
    Ok(row)
}

async fn stored_fault(service: &Service, input: &Value) -> Result<Value> {
    service.with_admin(false, async |connection| {
    let target = &input["target"];
    let constraint = match input["constraint"].as_str() {
        Some("principals_identity_profile_check") => Some("ALTER TABLE arboresce_authority.principals DROP CONSTRAINT principals_identity_profile_check"),
        Some("principals_identity_version_check") => Some("ALTER TABLE arboresce_authority.principals DROP CONSTRAINT principals_identity_version_check"),
        Some("principals_public_key_check") => Some("ALTER TABLE arboresce_authority.principals DROP CONSTRAINT principals_public_key_check"),
        Some("memberships_state_check") => Some("ALTER TABLE arboresce_authority.memberships DROP CONSTRAINT memberships_state_check"),
        Some("resources_control_revision_check") => Some(
            "ALTER TABLE arboresce_authority.resources DROP CONSTRAINT resources_control_revision_check",
        ),
        Some("principals_created_at_check") => Some(
            "ALTER TABLE arboresce_authority.principals DROP CONSTRAINT principals_created_at_check",
        ),
        Some("principals_revoked_at_check") => Some(
            "ALTER TABLE arboresce_authority.principals DROP CONSTRAINT principals_revoked_at_check",
        ),
        Some("principals_control_revision_check") => Some(
            "ALTER TABLE arboresce_authority.principals DROP CONSTRAINT principals_control_revision_check",
        ),
        Some("principals_identity_standard_revision_check") => Some(
            "ALTER TABLE arboresce_authority.principals DROP CONSTRAINT principals_identity_standard_revision_check",
        ),
        Some("memberships_control_revision_check") => Some(
            "ALTER TABLE arboresce_authority.memberships DROP CONSTRAINT memberships_control_revision_check",
        ),
        Some("grants_issued_at_check") => {
            Some("ALTER TABLE arboresce_authority.grants DROP CONSTRAINT grants_issued_at_check")
        }
        Some("grants_expires_at_check") => {
            Some("ALTER TABLE arboresce_authority.grants DROP CONSTRAINT grants_expires_at_check")
        }
        Some("immutable_members_control_revision_check") => Some(
            "ALTER TABLE arboresce_authority.immutable_members DROP CONSTRAINT immutable_members_control_revision_check",
        ),
        Some("memberships_roles_check") => Some(
            "ALTER TABLE arboresce_authority.memberships DROP CONSTRAINT memberships_roles_check",
        ),
        Some("workspace_policies_policy_check") => Some(
            "ALTER TABLE arboresce_authority.workspace_policies DROP CONSTRAINT workspace_policies_policy_check",
        ),
        None => None,
        _ => return Err("unadmitted stored-data constraint fault".into()),
    };
    let constraint_observation = if let Some(sql) = constraint {
        shared::controlled_drop(
            &mut *connection,
            shared::text(&input["relation"]),
            shared::text(&input["constraint"]),
            sql,
        )
        .await?
    } else {
        Value::Null
    };
    let relation = shared::text(&input["relation"]);
    let (changed, guarded) = match relation {
        "resources" => {
            let value = input["value"]
                .as_str()
                .or(input["resource_control"].as_str())
                .ok_or("missing controlled revision")?;
            let changed=sqlx::query("UPDATE arboresce_authority.resources SET control_revision=$5::bigint WHERE organization_id=$1::uuid AND domain_id=$2::uuid AND resource_kind=$3 AND resource_id=$4::uuid")
                .bind(shared::text(&target["organization_id"])).bind(shared::text(&target["domain_id"]))
                .bind(shared::text(&target["resource_kind"])).bind(shared::text(&target["resource_id"])).bind(value).execute(&mut *connection).await?.rows_affected();
            let actual:i64=sqlx::query_scalar("SELECT control_revision FROM arboresce_authority.resources WHERE organization_id=$1::uuid AND domain_id=$2::uuid AND resource_kind=$3 AND resource_id=$4::uuid")
                .bind(shared::text(&target["organization_id"])).bind(shared::text(&target["domain_id"]))
                .bind(shared::text(&target["resource_kind"])).bind(shared::text(&target["resource_id"])).fetch_one(&mut *connection).await?;
            (changed, json!(actual))
        }
        "principals" => scalar_fault(&mut *connection, input).await?,
        "memberships" if matches!(input["field"].as_str(), Some("control_revision" | "state")) => {
            scalar_fault(&mut *connection, input).await?
        }
        "grants" | "immutable_members" => scalar_fault(&mut *connection, input).await?,
        "memberships" => {
            let values: Vec<String> = input["values"]
                .as_array()
                .ok_or("missing controlled role array")?
                .iter()
                .map(|v| shared::text(v).to_owned())
                .collect();
            if values.len() > 17 {
                return Err("controlled role array exceeds selected setup bound".into());
            }
            let changed=sqlx::query("UPDATE arboresce_authority.memberships SET roles=$5 WHERE organization_id=$1::uuid AND domain_id=$2::uuid AND principal_id=$3::uuid AND membership_id=$4::uuid")
                .bind(shared::text(&target["organization_id"])).bind(shared::text(&target["domain_id"]))
                .bind(shared::text(&target["principal_id"])).bind(shared::text(&target["membership_id"])).bind(values).execute(&mut *connection).await?.rows_affected();
            let guard = crate::implementation::authority::rows::tokens("roles");
            let sql = format!(
                "SELECT {guard} FROM arboresce_authority.memberships WHERE organization_id=$1::uuid AND domain_id=$2::uuid AND principal_id=$3::uuid AND membership_id=$4::uuid"
            );
            let actual: Option<Vec<String>> = sqlx::query_scalar(sqlx::AssertSqlSafe(sql))
                .bind(shared::text(&target["organization_id"]))
                .bind(shared::text(&target["domain_id"]))
                .bind(shared::text(&target["principal_id"]))
                .bind(shared::text(&target["membership_id"]))
                .fetch_one(&mut *connection)
                .await?;
            (
                changed,
                actual.map_or(
                    json!("bounded invalid marker"),
                    |roles| json!({"roles":roles}),
                ),
            )
        }
        "workspace_policies" => {
            let count = input["count"]
                .as_u64()
                .ok_or("missing controlled token length")?;
            if count > 97 {
                return Err("controlled token exceeds selected setup bound".into());
            }
            let value = shared::text(&input["byte"]).repeat(count as usize);
            let changed=sqlx::query("UPDATE arboresce_authority.workspace_policies SET policy=$3 WHERE organization_id=$1::uuid AND domain_id=$2::uuid")
                .bind(shared::text(&target["organization_id"])).bind(shared::text(&target["domain_id"])).bind(value).execute(&mut *connection).await?.rows_affected();
            let guard = crate::implementation::authority::rows::token("policy");
            let sql = format!(
                "SELECT {guard} FROM arboresce_authority.workspace_policies WHERE organization_id=$1::uuid AND domain_id=$2::uuid"
            );
            let actual: Option<String> = sqlx::query_scalar(sqlx::AssertSqlSafe(sql))
                .bind(shared::text(&target["organization_id"]))
                .bind(shared::text(&target["domain_id"]))
                .fetch_one(&mut *connection)
                .await?;
            (
                changed,
                actual.map_or(json!("bounded invalid marker"), |v| json!(v)),
            )
        }
        _ => return Err("unadmitted controlled data relation".into()),
    };
    if changed != 1 {
        return Err("controlled data target did not change exactly one row".into());
    }
    Ok(json!({"guarded":guarded,"constraint_observation":constraint_observation}))
    }).await
}

async fn selected_ordinal(service: &Service, input: &Value, seed: &Value) -> Result<Value> {
    service.with_admin(false, async |connection| {
        let before = shared::constraints(connection).await?;
        let target = &input["target"];
        let ordinal = input["replacement_ordinal"].as_i64().ok_or("missing replacement ordinal")?;
        let changed = sqlx::query("UPDATE arboresce_authority.immutable_members SET ordinal=$5::smallint WHERE organization_id=$1::uuid AND domain_id=$2::uuid AND chain_id=$3::uuid AND ordinal=$4::smallint")
            .bind(shared::text(&target["organization_id"]))
            .bind(shared::text(&target["domain_id"]))
            .bind(shared::text(&target["chain_id"]))
            .bind(target["ordinal"].as_i64().ok_or("missing original ordinal")?)
            .bind(ordinal).execute(&mut *connection).await?.rows_affected();
        if changed != 1 {
            return Err("ordinal fault target is not exactly one row".into());
        }
        let ordinals: Vec<i16> = sqlx::query_scalar("SELECT ordinal FROM arboresce_authority.immutable_members WHERE organization_id=$1::uuid AND domain_id=$2::uuid AND chain_id=$3::uuid ORDER BY ordinal LIMIT 9")
            .bind(shared::text(&target["organization_id"]))
            .bind(shared::text(&target["domain_id"]))
            .bind(shared::text(&target["chain_id"]))
            .fetch_all(&mut *connection).await?;
        if ordinals.len() > 8 {
            return Err("selected ordinal lookahead exceeded".into());
        }
        let after = shared::constraints(connection).await?;
        let normal = before == after && before == seed["normal_constraints"]
            && before == seed["seeded_constraints"]
            && before.as_array().is_some_and(|rows| !rows.is_empty() && rows.iter().all(|row| row["validated"] == true));
        Ok(json!({"selected_ordinals":ordinals,"normal_constraints_enabled":normal,
            "constraint_before":shared::retain_observation(&before)?,
            "constraint_after":shared::retain_observation(&after)?}))
    }).await
}

/// Preserve physical identities and every effective column privilege for the
/// same-count rename. These tuples are actual observations, not expected data.
async fn column_identities(service: &Service) -> Result<Value> {
    service.with_admin(false, async |connection| {
        let value = query_json(connection, "SELECT jsonb_build_object(\
            'privileges',(SELECT coalesce(jsonb_agg(value ORDER BY relation,ordinal,role),'[]'::jsonb) FROM (SELECT c.relname AS relation,a.attnum AS ordinal,r.role,jsonb_build_array(c.oid,c.relname,a.attnum,a.attname,r.role,\
                has_column_privilege(r.role,c.oid,a.attnum,'SELECT'),has_column_privilege(r.role,c.oid,a.attnum,'INSERT'),\
                has_column_privilege(r.role,c.oid,a.attnum,'UPDATE'),has_column_privilege(r.role,c.oid,a.attnum,'REFERENCES'),\
                has_table_privilege(r.role,c.oid,'INSERT,UPDATE,DELETE,TRUNCATE,TRIGGER,REFERENCES')) AS value \
                FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace JOIN pg_attribute a ON a.attrelid=c.oid AND a.attnum>0 AND NOT a.attisdropped \
                CROSS JOIN (VALUES ('harness_authority_admission'::text),('harness_authority_permission'::text),('harness_writer'::text)) r(role) \
                WHERE n.nspname='arboresce_authority' AND c.relkind='r' ORDER BY c.relname,a.attnum,r.role LIMIT 1025) bounded),\
            'constraint_identities',(SELECT coalesce(jsonb_agg(value ORDER BY oid),'[]'::jsonb) FROM (SELECT oid,jsonb_build_array(oid,conrelid,conkey,confrelid,confkey,convalidated,condeferrable,condeferred,connoinherit) AS value FROM pg_constraint WHERE connamespace='arboresce_authority'::regnamespace ORDER BY oid LIMIT 257) bounded))::text", &[], 131072).await?;
        if value["privileges"].as_array().is_none_or(|rows| rows.len() > 1024)
            || value["constraint_identities"].as_array().is_none_or(|rows| rows.len() > 256)
        {
            return Err("column identity catalogue lookahead exceeded".into());
        }
        Ok(value)
    }).await
}

async fn required_column_identity(service: &Service, input: &Value) -> Result<Value> {
    if input["table"] != "memberships"
        || input["column"] != "lock_guard"
        || input["replacement_column"] != "qualification_guard_renamed"
        || input["dependent_checks"] != json!(["memberships_lock_guard_check"])
    {
        return Err("unadmitted required-column rename".into());
    }
    let before = catalogue_value(service).await?;
    let identities_before = column_identities(service).await?;
    service.with_admin(false, async |connection| {
        sqlx::query("ALTER TABLE arboresce_authority.memberships RENAME COLUMN lock_guard TO qualification_guard_renamed")
            .execute(connection).await?;
        Ok(())
    }).await?;
    let after = catalogue_value(service).await?;
    let identities_after = column_identities(service).await?;
    let mut expected = before.clone();
    let mut selected_columns = 0;
    for row in expected["columns"]
        .as_array_mut()
        .ok_or("missing column catalogue")?
    {
        if row["table"] == "memberships" && row["name"] == "lock_guard" {
            row["name"] = json!("qualification_guard_renamed");
            selected_columns += 1;
        }
    }
    let mut selected_checks = 0;
    for row in expected["constraints"]
        .as_array_mut()
        .ok_or("missing constraint catalogue")?
    {
        if row["relation"] == "arboresce_authority.memberships"
            && row["name"] == "memberships_lock_guard_check"
        {
            if row["columns"] != json!(["lock_guard"]) {
                return Err("selected guard constraint columns differ".into());
            }
            row["columns"] = json!(["qualification_guard_renamed"]);
            let definition = row["definition"]
                .as_str()
                .ok_or("missing selected guard definition")?;
            row["definition"] =
                json!(definition.replace("lock_guard", "qualification_guard_renamed"));
            selected_checks += 1;
        }
    }
    let mut expected_identities = identities_before.clone();
    let mut selected_privilege_labels = 0;
    for row in expected_identities["privileges"]
        .as_array_mut()
        .ok_or("missing privilege identity vector")?
    {
        if row[1] == "memberships" && row[3] == "lock_guard" {
            row[3] = json!("qualification_guard_renamed");
            selected_privilege_labels += 1;
        }
    }
    if selected_columns != 1 || selected_checks != 1 || selected_privilege_labels != 3 {
        return Err("required column or its named dependents are not unique".into());
    }
    let columns = after["columns"]
        .as_array()
        .ok_or("missing renamed columns")?;
    Ok(
        json!({"catalogue_column_count_unchanged":before["columns"].as_array().map(Vec::len) == Some(columns.len()),
        "required_name_missing":!columns.iter().any(|row| row["table"] == "memberships" && row["name"] == "lock_guard"),
        "replacement_name_present":columns.iter().any(|row| row["table"] == "memberships" && row["name"] == "qualification_guard_renamed"),
        "only_named_dependents_changed":after == expected && identities_after == expected_identities,
        "catalogue_boundary_before":shared::retain_observation(&before)?,
        "catalogue_boundary_after":shared::retain_observation(&after)?,
        "column_identities_before":shared::retain_observation(&identities_before)?,
        "column_identities_after":shared::retain_observation(&identities_after)?}),
    )
}

async fn legacy(service: &Service, input: &Value, request: &Value, seed: &Value) -> Result<Value> {
    let target = &input["target"];
    let (sql, keys) = match input["relation"].as_str() {
        Some("principals") => (
            "SELECT jsonb_build_object('kind',kind,'control_revision',control_revision,'created_at',created_at,'identity_standard',identity_standard,'identity_standard_revision',identity_standard_revision)::text FROM arboresce_authority.principals WHERE principal_id=$1::uuid",
            vec![shared::text(&target["principal_id"]).to_owned()],
        ),
        Some("memberships") => (
            "SELECT jsonb_build_object('principal_kind',principal_kind,'roles',roles,'policy',policy,'control_revision',control_revision)::text FROM arboresce_authority.memberships WHERE organization_id=$1::uuid AND domain_id=$2::uuid AND principal_id=$3::uuid AND membership_id=$4::uuid",
            vec![
                shared::text(&target["organization_id"]).to_owned(),
                shared::text(&target["domain_id"]).to_owned(),
                shared::text(&target["principal_id"]).to_owned(),
                shared::text(&target["membership_id"]).to_owned(),
            ],
        ),
        _ => return Err("unadmitted selected legacy metadata".into()),
    };
    let (metadata, actual_constraints) = service
        .with_admin(false, async |connection| {
            let metadata = query_json(&mut *connection, sql, &keys, 4096).await?;
            let constraints = shared::constraints(connection).await?;
            Ok((metadata, constraints))
        })
        .await?;
    let normal = actual_constraints == seed["normal_constraints"]
        && actual_constraints == seed["seeded_constraints"]
        && actual_constraints
            .as_array()
            .is_some_and(|v| !v.is_empty() && v.iter().all(|c| c["validated"] == true));
    // This independently replays the selected real decoder. It is not a trace
    // or first-failure-order assertion about the preceding adapter invocation.
    let (relation, decoded, actual_context) = with_connection(
        raw_role(service, false).await?, async |connection| {
            with_rollback(connection, async |transaction| {
    context(transaction, request, &json!([])).await?;
    sqlx::query(
        "SELECT epoch FROM arboresce_authority.namespace_guard WHERE singleton=1 FOR SHARE",
    )
    .execute(&mut *transaction)
    .await?;
    let actual_context: String = sqlx::query_scalar(
        "SELECT jsonb_build_object(\
        'role',current_user,'backend_pid',pg_backend_pid(),\
        'organization_id',current_setting('arboresce.organization_id',true),\
        'domain_id',current_setting('arboresce.domain_id',true),\
        'principal_id',current_setting('arboresce.principal_id',true),\
        'chain_id',current_setting('arboresce.chain_id',true))::text",
    )
    .fetch_one(&mut *transaction)
    .await?;
    let target = &input["target"];
    let (relation, decoded) = match shared::text(&input["relation"]) {
        "principals" => {
            let columns = crate::implementation::authority::read::principal_columns();
            let sql = format!(
                "SELECT {columns} FROM arboresce_authority.principals WHERE principal_id=$1::uuid"
            );
            let row = sqlx::query(sqlx::AssertSqlSafe(sql))
                .bind(shared::text(&target["principal_id"]))
                .fetch_one(&mut *transaction)
                .await?;
            (
                "principals",
                crate::implementation::authority::read::decode_principal(&row).map(|_| ()),
            )
        }
        "memberships" => {
            let columns = crate::implementation::authority::read::membership_columns();
            let sql = format!(
                "SELECT {columns} FROM arboresce_authority.memberships WHERE organization_id=$1::uuid AND domain_id=$2::uuid AND principal_id=$3::uuid AND membership_id=$4::uuid"
            );
            let row = sqlx::query(sqlx::AssertSqlSafe(sql))
                .bind(shared::text(&target["organization_id"]))
                .bind(shared::text(&target["domain_id"]))
                .bind(shared::text(&target["principal_id"]))
                .bind(shared::text(&target["membership_id"]))
                .fetch_one(&mut *transaction)
                .await?;
            (
                "memberships",
                crate::implementation::authority::read::decode_membership(&row).map(|_| ()),
            )
        }
        _ => return Err("unadmitted selected legacy decoder".into()),
    };
    Ok((relation, decoded, actual_context))
            }).await
        },
    ).await?;
    if decoded != Err(AuthorityFailure::InvalidStoredData) {
        return Err("selected production decoder did not reject actual legacy row".into());
    }
    Ok(
        json!({"normal_constraints_enabled":normal,"selected_metadata":metadata,
            "selected_decoder_relation":relation,"actual_selected_decoder_observation":{
                "failure":"InvalidStoredData","context":serde_json::from_str::<Value>(&actual_context)?,
                "terminal":"rollback","original_adapter_call_trace":false},
            "legacy_constraint_observation":shared::retain_observation(&actual_constraints)?}),
    )
}

async fn channel(service: &Service, input: &Value) -> Result<Value> {
    let values: Vec<String> = input["identifiers"]
        .as_array()
        .ok_or("missing channel identifiers")?
        .iter()
        .map(|v| shared::text(v).to_owned())
        .collect();
    if values.len() > 33 {
        return Err("channel exceeds lookahead setup bound".into());
    }
    let rows = with_connection(raw_role(service, false).await?, async |connection| {
        Ok(sqlx::query("SELECT value::uuid::text AS identifier FROM unnest($1::text[]) WITH ORDINALITY AS input(value,ordinal) ORDER BY ordinal LIMIT 33")
            .bind(values).fetch_all(connection).await?)
    }).await?;
    let mut budget = crate::implementation::authority::rows::RowBudget::new();
    let actual = budget.take(rows.len()).and_then(|()| {
        rows.iter()
            .map(|row| crate::implementation::authority::rows::id(row, "identifier"))
            .collect::<std::result::Result<Vec<_>, _>>()
    });
    Ok(match actual {
        Ok(ids) => {
            json!({"decoded_rows":ids.len(),"allocation_bound_preserved":ids.len()<=32 && rows.len()<=33,"authority_admitted":false,"actual_identifiers":ids.iter().map(ToString::to_string).collect::<Vec<_>>()})
        }
        Err(failure) => {
            json!({"failure":failure.to_string(),"allocation_bound_preserved":rows.len()<=33,"authority_admitted":false,"raw_lookahead_rows":rows.len()})
        }
    })
}

async fn history_rows(service: &Service, controlled_shape: bool) -> Result<Value> {
    let sql = if controlled_shape {
        "SELECT coalesce(jsonb_agg(to_jsonb(m) || jsonb_build_object('installed_on',to_char(m.installed_on AT TIME ZONE 'UTC','YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')) ORDER BY version),'[]'::jsonb)::text FROM public._sqlx_migrations m"
    } else {
        "SELECT coalesce(jsonb_agg(to_jsonb(m) ORDER BY version),'[]'::jsonb)::text FROM public._sqlx_migrations m"
    };
    service.json(false, sql, &[], 131072).await
}

/// Observe the malformed ledger itself, including its complete physical identities.
/// The stable projection is asserted separately from the retained raw catalogue.
async fn history_catalogue(service: &Service) -> Result<Value> {
    service.json(false, "SELECT jsonb_build_object( \
        'shape',jsonb_build_object( \
            'schema',n.nspname,'relation',c.relname,'kind',c.relkind::text, \
            'owner',pg_get_userbyid(c.relowner),'row_security',c.relrowsecurity,'force_row_security',c.relforcerowsecurity, \
            'columns',(SELECT coalesce(jsonb_agg(jsonb_build_object( \
                'ordinal',a.attnum,'name',a.attname,'type',format_type(a.atttypid,a.atttypmod),'type_oid',a.atttypid::bigint, \
                'not_null',a.attnotnull,'default',pg_get_expr(d.adbin,d.adrelid), \
                'identity',a.attidentity::text,'generated',a.attgenerated::text,'collation',coll.collname \
                ) ORDER BY a.attnum),'[]'::jsonb) \
                FROM pg_attribute a LEFT JOIN pg_attrdef d ON d.adrelid=a.attrelid AND d.adnum=a.attnum \
                LEFT JOIN pg_collation coll ON coll.oid=a.attcollation \
                WHERE a.attrelid=c.oid AND a.attnum>0 AND NOT a.attisdropped), \
            'constraints',(SELECT coalesce(jsonb_agg(jsonb_build_object( \
                'name',co.conname,'kind',co.contype::text, \
                'key_columns',coalesce((SELECT jsonb_agg(a.attname ORDER BY k.ordinal) \
                    FROM unnest(co.conkey) WITH ORDINALITY k(number,ordinal) \
                    JOIN pg_attribute a ON a.attrelid=co.conrelid AND a.attnum=k.number),'[]'::jsonb), \
                'definition',pg_get_constraintdef(co.oid),'deferrable',co.condeferrable, \
                'deferred',co.condeferred,'validated',co.convalidated \
                ) ORDER BY co.conname),'[]'::jsonb) FROM pg_constraint co WHERE co.conrelid=c.oid), \
            'indexes',(SELECT coalesce(jsonb_agg(jsonb_build_object( \
                'name',ic.relname,'unique',i.indisunique,'primary',i.indisprimary,'valid',i.indisvalid,'ready',i.indisready, \
                'access_method',am.amname, \
                'key_columns',coalesce((SELECT jsonb_agg(a.attname ORDER BY k.ordinal) \
                    FROM unnest(i.indkey::smallint[]) WITH ORDINALITY k(number,ordinal) \
                    JOIN pg_attribute a ON a.attrelid=i.indrelid AND a.attnum=k.number \
                    WHERE k.ordinal<=i.indnkeyatts),'[]'::jsonb), \
                'include_columns',coalesce((SELECT jsonb_agg(a.attname ORDER BY k.ordinal) \
                    FROM unnest(i.indkey::smallint[]) WITH ORDINALITY k(number,ordinal) \
                    JOIN pg_attribute a ON a.attrelid=i.indrelid AND a.attnum=k.number \
                    WHERE k.ordinal>i.indnkeyatts),'[]'::jsonb), \
                'operator_classes',coalesce((SELECT jsonb_agg(op.opcname ORDER BY k.ordinal) \
                    FROM unnest(i.indclass::oid[]) WITH ORDINALITY k(number,ordinal) \
                    JOIN pg_opclass op ON op.oid=k.number),'[]'::jsonb), \
                'collations',coalesce((SELECT jsonb_agg(coll.collname ORDER BY k.ordinal) \
                    FROM unnest(i.indcollation::oid[]) WITH ORDINALITY k(number,ordinal) \
                    LEFT JOIN pg_collation coll ON coll.oid=k.number),'[]'::jsonb), \
                'predicate',pg_get_expr(i.indpred,i.indrelid),'expressions',pg_get_expr(i.indexprs,i.indrelid), \
                'definition',pg_get_indexdef(i.indexrelid) \
                ) ORDER BY ic.relname),'[]'::jsonb) \
                FROM pg_index i JOIN pg_class ic ON ic.oid=i.indexrelid JOIN pg_am am ON am.oid=ic.relam \
                WHERE i.indrelid=c.oid)), \
        'raw',jsonb_build_object( \
            'class',to_jsonb(c), \
            'attributes',(SELECT coalesce(jsonb_agg(to_jsonb(a) ORDER BY a.attnum),'[]'::jsonb) \
                FROM pg_attribute a WHERE a.attrelid=c.oid AND NOT a.attisdropped), \
            'defaults',(SELECT coalesce(jsonb_agg(to_jsonb(d) ORDER BY d.adnum,d.oid),'[]'::jsonb) \
                FROM pg_attrdef d WHERE d.adrelid=c.oid), \
            'constraints',(SELECT coalesce(jsonb_agg(to_jsonb(co) ORDER BY co.oid),'[]'::jsonb) \
                FROM pg_constraint co WHERE co.conrelid=c.oid), \
            'indexes',(SELECT coalesce(jsonb_agg(jsonb_build_object('index',to_jsonb(i),'class',to_jsonb(ic)) ORDER BY i.indexrelid),'[]'::jsonb) \
                FROM pg_index i JOIN pg_class ic ON ic.oid=i.indexrelid WHERE i.indrelid=c.oid)))::text \
        FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace \
        WHERE n.nspname='public' AND c.relname='_sqlx_migrations'", &[], 131072).await
}

async fn service_state(service: &Service) -> Result<Value> {
    service.identity().await?;
    service.json(true, "SELECT jsonb_build_object(\
        'database',(SELECT to_jsonb(d) FROM pg_database d WHERE datname='arboresce_migration_harness'),\
        'roles',(SELECT jsonb_agg(to_jsonb(r)-'rolpassword' ORDER BY rolname) FROM pg_roles r WHERE rolname IN ('harness_authority_admission','harness_authority_permission')),\
        'native',jsonb_build_object('system_identifier',(SELECT system_identifier::text FROM pg_control_system()),'postmaster_start',pg_postmaster_start_time()::text,'data',current_setting('data_directory'),'socket',current_setting('unix_socket_directories')))::text", &[], 16384).await
}

async fn backend_census(
    service: &Service,
    query: sqlx::query::QueryScalar<'_, sqlx::Postgres, i64, sqlx::postgres::PgArguments>,
) -> Result<i64> {
    service
        .with_admin(true, async |connection| {
            Ok(query.fetch_one(connection).await?)
        })
        .await
}

async fn service_boundary(service: &Service, input: &Value) -> Result<Value> {
    let before = service_state(service).await?;
    let physical_before = physical_authority(service).await?;
    let mut observed = match shared::text(&input["operation"]) {
        "service_identity_boundary" => {
            let mut record = service.record().clone();
            record[shared::text(&input["field"])] = input["value"].clone();
            let candidate = Service::from_record(record)?;
            let result = candidate.identity().await;
            json!({"failure":result.err().map(|e|e.to_string())})
        }
        "service_no_adoption_boundary" => {
            let result = service.require_fresh_roles().await;
            json!({"failure":result.err().map(|e|e.to_string())})
        }
        "service_busy_reset_boundary" => {
            let (pid, mut observed) = with_connection(service.admin(false).await?, async |holder| {
                let pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
                    .fetch_one(&mut *holder)
                    .await?;
                let census = backend_census(
                    service,
                    sqlx::query_scalar("SELECT count(*) FROM pg_stat_activity WHERE datname='arboresce_migration_harness' AND pid=$1")
                        .bind(pid),
                ).await?;
                let result = service.reset().await;
                let census_error = result
                    .as_ref()
                    .err()
                    .and_then(|e| e.downcast_ref::<crate::authority_service::DatabaseCensusFailure>());
                let phase_proved = census == 1 && census_error.is_some_and(|e| e.observed_backends > 0);
                let raw_error = result.as_ref().err().map(|e| format!("{e:?}"));
                let actual_after: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
                    .fetch_one(&mut *holder)
                    .await?;
                let after_census = backend_census(
                    service,
                    sqlx::query_scalar("SELECT count(*) FROM pg_stat_activity WHERE datname='arboresce_migration_harness' AND pid=$1")
                        .bind(pid),
                ).await?;
                Ok((pid, json!({"refusal_category":if phase_proved {"busy_database_census"} else {"unclassified_reset_result"},
                    "holder_survived_refusal":actual_after==pid && after_census==1,
                    "destructive_reset_started":!phase_proved,
                    "holder_pid":pid,"holder_census_before":census,"holder_census_after":after_census,
                    "observed_busy_count":census_error.map(|e|e.observed_backends),
                    "census_timed_out":census_error.map(|e|e.timeout.is_some()),"raw_reset_error":raw_error})))
            }).await?;
            let backend_absent = absent(service, pid).await?;
            observed["positive_holder_close"] = json!(true);
            observed["holder_backend_absent"] = json!(backend_absent);
            observed
        }
        _ => return Err("unadmitted owned service boundary".into()),
    };
    let after = service_state(service).await?;
    let physical_after = physical_authority(service).await?;
    observed["database_unchanged"] =
        json!(before["database"] == after["database"] && physical_before == physical_after);
    observed["database_oid_unchanged"] =
        json!(before["database"]["oid"] == after["database"]["oid"]);
    observed["roles_unchanged"] = json!(before["roles"] == after["roles"]);
    observed["native_identity_unchanged"] = json!(before["native"] == after["native"]);
    observed["service_before"] = shared::retain_observation(&before)?;
    observed["service_after"] = shared::retain_observation(&after)?;
    observed["physical_service_before"] = shared::retain_observation(&physical_before)?;
    observed["physical_service_after"] = shared::retain_observation(&physical_after)?;
    Ok(observed)
}

pub struct HistoryStatement {
    sql: &'static str,
    rows_affected: u64,
    controlled_shape: bool,
}

/// A pure compiled allowlist shared by native preparation and finite refusal checks.
pub fn history_statement(fault: &str) -> Result<HistoryStatement> {
    let (sql, rows_affected, controlled_shape) = match fault {
        "unexpected fifth version" => (
            "INSERT INTO public._sqlx_migrations (version,description,installed_on,success,checksum,execution_time) SELECT 5,'unexpected synthetic version',installed_on,true,checksum,0 FROM public._sqlx_migrations WHERE version=4",
            1,
            false,
        ),
        "fourth checksum mismatch" => (
            "UPDATE public._sqlx_migrations SET checksum=set_byte(checksum,0,get_byte(checksum,0)#1) WHERE version=4",
            1,
            false,
        ),
        "fourth success false" => (
            "UPDATE public._sqlx_migrations SET success=false WHERE version=4",
            1,
            false,
        ),
        "second version removed" => (
            "DELETE FROM public._sqlx_migrations WHERE version=2",
            1,
            false,
        ),
        "version column is text" => (
            "ALTER TABLE public._sqlx_migrations ALTER COLUMN version TYPE text USING version::text",
            0,
            true,
        ),
        "success column is text" => (
            "ALTER TABLE public._sqlx_migrations ALTER COLUMN success TYPE text USING success::text",
            0,
            true,
        ),
        "checksum column is hex text" => (
            "ALTER TABLE public._sqlx_migrations ALTER COLUMN checksum TYPE text USING encode(checksum,'hex')",
            0,
            true,
        ),
        "success column is renamed" => (
            "ALTER TABLE public._sqlx_migrations RENAME COLUMN success TO fixture_history_success",
            0,
            true,
        ),
        _ => return Err("unadmitted history fault".into()),
    };
    Ok(HistoryStatement {
        sql,
        rows_affected,
        controlled_shape,
    })
}

async fn history(service: &Service, input: &Value) -> Result<Value> {
    let statement = history_statement(shared::text(&input["fault"]))?;
    let preparation = service
        .with_admin(false, async |connection| {
            let mut fixed_rows = None;
            if statement.controlled_shape {
                let rows = sqlx::query("UPDATE public._sqlx_migrations SET installed_on=$1::timestamptz,execution_time=$2")
                    .bind(shared::text(&input["fixed_history"]["installed_on"]))
                    .bind(input["fixed_history"]["execution_time"].as_i64().ok_or("invalid fixed history execution time")?)
                    .execute(&mut *connection).await?.rows_affected();
                if rows != 4 {
                    return Err("fixed history target mismatch".into());
                }
                fixed_rows = Some(rows);
            }
            let fault_rows = sqlx::query(statement.sql).execute(connection).await?.rows_affected();
            if fault_rows != statement.rows_affected {
                return Err("history fault target mismatch".into());
            }
            Ok(json!({"fixed_history_rows_affected":fixed_rows,"fault_rows_affected":fault_rows}))
        })
        .await?;
    let before_history = history_rows(service, statement.controlled_shape).await?;
    let before_catalogue = catalogue(service).await?;
    let ledger_before = if statement.controlled_shape {
        let value = history_catalogue(service).await?;
        let retained = shared::retain_observation(&value["raw"])?;
        Some((value, retained))
    } else {
        None
    };
    let result = service
        .harness
        .migrate(SchemaVersion::CurrentAuthority)
        .await;
    let after_history = history_rows(service, statement.controlled_shape).await?;
    let after_catalogue = catalogue(service).await?;
    let catalogue_written = before_catalogue["sha256"] != after_catalogue["sha256"]
        || before_catalogue["size"] != after_catalogue["size"];
    let mut observed = json!({"failure":result.err().map(|e|format!("{e:?}")),"schema_or_history_written":before_history!=after_history || catalogue_written,
        "history_before":before_history,"history_after":after_history,"catalogue_before":before_catalogue,"catalogue_after":after_catalogue});
    if let Some((before, retained_before)) = ledger_before {
        let after = history_catalogue(service).await?;
        let unchanged = before["raw"] == after["raw"];
        observed["ledger_shape_before"] = before["shape"].clone();
        observed["ledger_shape_after"] = after["shape"].clone();
        observed["ledger_catalogue_before"] = retained_before;
        observed["ledger_catalogue_after"] = shared::retain_observation(&after["raw"])?;
        observed["ledger_catalogue_unchanged"] = json!(unchanged);
        observed["schema_or_history_written"] =
            json!(before_history != after_history || catalogue_written || !unchanged);
        observed["preparation"] = preparation;
    }
    Ok(observed)
}

pub async fn absent(service: &Service, pid: i32) -> Result<bool> {
    service
        .with_admin(true, async |connection| {
            timeout(Duration::from_secs(1), async {
                loop {
                    let gone: bool = sqlx::query_scalar(
                        "SELECT NOT EXISTS(SELECT 1 FROM pg_stat_activity WHERE pid=$1)",
                    )
                    .bind(pid)
                    .fetch_one(&mut *connection)
                    .await?;
                    if gone {
                        return Ok::<_, sqlx::Error>(());
                    }
                    tokio::task::yield_now().await;
                }
            })
            .await??;
            Ok(true)
        })
        .await
}

struct RequestTrace {
    path: PathBuf,
    device: u64,
    inode: u64,
    offset: u64,
}
impl RequestTrace {
    async fn start(service: &Service) -> Result<Self> {
        let settings: (String, String) = service
            .with_admin(true, async |connection| {
                Ok(sqlx::query_as(
                    "SELECT current_setting('log_statement'),current_setting('log_line_prefix')",
                )
                .fetch_one(connection)
                .await?)
            })
            .await?;
        if settings.0 != "all" || settings.1 != "%m [%p] %a %u %d " {
            return Err("owned statement trace is not the selected profile".into());
        }
        let data = service.data();
        let path = data
            .parent()
            .ok_or("missing owned service base")?
            .join("postgres.log");
        let meta = std::fs::symlink_metadata(&path)?;
        if !meta.is_file()
            || meta.nlink() != 1
            || meta.len() > (64 << 20)
            || meta.uid() != std::fs::metadata(data)?.uid()
        {
            return Err("invalid owned statement log".into());
        }
        Ok(Self {
            path,
            device: meta.dev(),
            inode: meta.ino(),
            offset: meta.len(),
        })
    }
    fn finish(&self) -> Result<Value> {
        let mut file = std::fs::File::open(&self.path)?;
        let meta = file.metadata()?;
        let path = std::fs::symlink_metadata(&self.path)?;
        if !path.is_file()
            || path.nlink() != 1
            || meta.dev() != self.device
            || meta.ino() != self.inode
            || path.dev() != meta.dev()
            || path.ino() != meta.ino()
            || meta.len() < self.offset
            || meta.len() - self.offset > (1 << 20)
        {
            return Err("owned statement trace identity or bound changed".into());
        }
        let length = meta.len() - self.offset;
        file.seek(SeekFrom::Start(self.offset))?;
        let mut bytes = Vec::new();
        file.take(length).read_to_end(&mut bytes)?;
        if bytes.len() as u64 != length {
            return Err("owned statement trace was truncated".into());
        }
        let raw = std::str::from_utf8(&bytes)?;
        let mut statements = Vec::new();
        for line in raw.lines().filter(|line|line.contains(" harness_authority_admission harness_authority_admission arboresce_migration_harness ")) {
            let Some((_,message))=line.split_once("LOG:  ") else {continue;};
            let statement=message.strip_prefix("statement: ").or_else(||message.strip_prefix("execute ").and_then(|s|s.split_once(": ").map(|(_,s)|s)));
            if let Some(statement)=statement {statements.push(statement.to_owned());}
        }
        let mut begins = 0_u64;
        let mut commits = 0_u64;
        let mut rollbacks = 0_u64;
        for statement in &statements {
            for command in statement.split(';').map(str::trim) {
                match command {
                    "BEGIN" => begins += 1,
                    "COMMIT" => commits += 1,
                    "ROLLBACK" => rollbacks += 1,
                    _ => {}
                }
            }
        }
        if begins == 0 || commits + rollbacks == 0 {
            return Err("owned statement trace did not observe the attempted transaction".into());
        }
        Ok(
            json!({"path":self.path,"device":self.device,"inode":self.inode,"offset":self.offset,"end":meta.len(),
            "bytes":length,"sha256":arboresce_xtask::inventory::sha256(&bytes),"statements":statements,
            "begin_requests":begins,"commit_requests":commits,"rollback_requests":rollbacks,
            "automatic_retry_count":(begins-1).max(commits+rollbacks-1)}),
        )
    }
}

async fn terminal(
    service: &Service,
    input: &Value,
    request: &Value,
    clock: &mut shared::SuppliedClock,
    ids: &Value,
) -> Result<Value> {
    let raw_request = shared::request(request);
    let config = shared::configuration();
    if input["operation"] == "drop_fenced_transaction_before_terminal" {
        let sessions = service.harness.authority_sessions();
        let mut session = sessions.admission().await?;
        let pid = session.backend_id().await?;
        let before = sessions.available_slots();
        let fence = session.fence(&raw_request, &config, clock).await?;
        drop(fence);
        let after = sessions.available_slots();
        let next = session
            .observe_current(&raw_request, &config, clock)
            .await
            .err();
        session.close().await?;
        let gone = absent(service, pid).await?;
        return Ok(
            json!({"next_begin":next.map(|e|e.to_string()),"slot_released_before_exact_close":after>before,
            "slots_before":before,"slots_after_drop":after,"slots_after_positive_close":sessions.available_slots(),"old_backend_closed":gone}),
        );
    }
    let mode = if input["operation"] == "lost_commit_completion" {
        crate::relay::Mode::Commit
    } else {
        crate::relay::Mode::Rollback
    };
    let trace = RequestTrace::start(service).await?;
    let relay = crate::relay::Relay::start(&service.harness.socket, mode)?;
    let directory = relay.directory.clone();
    let harness =
        crate::Harness::new(directory.clone()).map_err(|_| "invalid owned relay target")?;
    let sessions = harness.authority_sessions();
    let mut session = sessions.admission().await?;
    let pid = session.backend_id().await?;
    let before = sessions.available_slots();
    let fence = session.fence(&raw_request, &config, clock).await?;
    let result = if input["operation"] == "lost_commit_completion" {
        fence
            .commit_marker(
                shared::id(&ids["marker_id"]),
                shared::id(&ids["observation_id"]),
            )
            .await
    } else {
        fence.rollback().await
    };
    let after = sessions.available_slots();
    let failure = result.err();
    let poisoned = session.backend_id().await.err();
    session.close().await?;
    let (server, client) = relay.finish()?;
    let server = server?;
    let client = client?;
    let gone = absent(service, pid).await?;
    let trace = trace.finish()?;
    if server.dropped != 1 {
        return Err("completion probe did not withhold exactly one selected terminal".into());
    }
    Ok(
        json!({"failure":failure.map(|e|e.to_string()),"automatic_retry_count":trace["automatic_retry_count"],"positive_receipt_count":usize::from(failure.is_none()),
        "slot_released_before_exact_close":after>before,"slots_before":before,"slots_after_unknown":after,"slots_after_positive_close":sessions.available_slots(),
        "next_begin":poisoned.map(|e|e.to_string()),"matching_completions_dropped":server.dropped,"server_frames":server.frames,"client_frames":client.frames,
        "old_backend_closed":gone,"relay_directory_absent":!directory.exists(),"actual_request_trace":trace}),
    )
}

pub async fn observe(
    service: &Service,
    input: &Value,
    base: Option<&Value>,
    recipe: Option<&Value>,
    trusted: &Value,
) -> Result<Value> {
    let operation = shared::text(&input["operation"]);
    let seed_observation = if let (Some(base), Some(recipe)) = (base, recipe) {
        let mut recipe = recipe.clone();
        if operation == "legacy_null_metadata" {
            recipe["legacy_input"] = input.clone();
        }
        shared::seed(service, &base["canonical_snapshot"], &recipe).await?
    } else {
        service.reset().await?;
        service
            .harness
            .migrate(SchemaVersion::CurrentAuthority)
            .await
            .map_err(|e| format!("controlled current migration failed: {e:?}"))?;
        Value::Null
    };
    let sample = base.map_or(&trusted["clock"], |b| &b["canonical_snapshot"]["now"]);
    let mut clock = if operation == "trusted_clock_failure" {
        shared::SuppliedClock::failing(sample, &trusted["reader_failure"])?
    } else {
        shared::SuppliedClock::new(sample)
    };
    let mut request = base.map_or(Value::Null, |b| b["request"].clone());
    if let Some(resource_id) = input["admitting_resource_id"].as_str() {
        let selected =
            base.ok_or("missing admitting resource input")?["canonical_snapshot"]["resources"]
                .as_array()
                .ok_or("missing actual input resources")?
                .iter()
                .find(|r| {
                    r["id"] == resource_id
                        && r["workspace"] == request["resource"]["workspace"]
                        && r["kind"] == request["resource"]["kind"]
                })
                .ok_or("admitting resource is not in selected input scope")?;
        request["resource"]["id"] = selected["id"].clone();
        request["expected_resource_control"] = selected["control_revision"].clone();
    }
    let catalogue_before = catalogue(service).await?;
    let mut setup = Value::Null;
    match operation {
        "isolated_catalogue_boundary" => setup = isolated_catalogue(service, input).await?,
        "catalogue_required_column_identity" => {
            setup = required_column_identity(service, input).await?
        }
        "selected_immutable_ordinal_gap" => {
            setup = selected_ordinal(service, input, &seed_observation).await?
        }
        "permission_boundary" => setup = prepare_permission(service, input).await?,
        "role_fault"
        | "role_membership_fault"
        | "table_security_fault"
        | "column_acl_fault"
        | "policy_fault"
        | "direct_guard_update_with_extra_permissive_policy" => fault(service, input).await?,
        "controlled_stored_fault"
        | "controlled_array_fault"
        | "controlled_token_fault"
        | "replace_current_array" => setup = stored_fault(service, input).await?,
        "qualified_synthetic_cas" => {
            let mut selected = input.clone();
            selected["relation"] = json!("resources");
            setup = stored_fault(service, &selected).await?;
            request["expected_resource_control"] =
                json!({"kind":"control","value":input["expected_control"]});
        }
        "source_audience_and_request_patch" => {
            let values: Vec<String> = input["permitted_principals"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| shared::text(v).to_owned())
                .collect();
            service.with_admin(false, async |connection| {
                if sqlx::query("UPDATE arboresce_authority.source_policies SET permitted_principals=$1::text[]::arboresce_authority.identifier[] WHERE policy_id='source_current'")
                    .bind(values).execute(connection).await?.rows_affected()!=1 {return Err("source audience target mismatch".into());}
                Ok(())
            }).await?;
        }
        "final_request_patch" => {
            let base = base.ok_or("missing initial/final input")?;
            let sessions = service.harness.authority_sessions();
            let mut initial = sessions.admission().await?;
            let typed = shared::request(&request);
            let config = shared::configuration();
            initial.observe_current(&typed, &config, &mut clock).await?;
            initial.close().await?;
            clock = shared::SuppliedClock::new(&base["final_effect_snapshot"]["now"]);
            let change =
                shared::change(&base["canonical_snapshot"], &base["final_effect_snapshot"])
                    .ok_or("missing actual final mutation")?;
            let mut permission = sessions.permission().await?;
            permission
                .fence(&typed, change, &mut clock)
                .await?
                .commit()
                .await?;
            permission.close().await?;
        }
        _ => {}
    }
    if let Some(patch) = input["patch"].as_object() {
        for (key, value) in patch {
            request[key] = value.clone();
        }
    }
    let catalogue_after_setup = catalogue(service).await?;
    let physical_before = physical_authority(service).await?;
    let mut observer_clock = shared::SuppliedClock::new(&json!(clock.value.to_string()));
    let before = shared::snapshot(service, &mut observer_clock, false).await?;
    let before_markers = shared::markers(service).await?;
    let mut observed = match operation {
        "permission_boundary" => permission_boundary(service, input, &request).await?,
        "service_identity_boundary"
        | "service_no_adoption_boundary"
        | "service_busy_reset_boundary" => service_boundary(service, input).await?,
        "direct_guard_update"
        | "direct_authority_update"
        | "direct_guard_update_with_extra_permissive_policy" => {
            direct_dml(service, input, &request).await?
        }
        "retained_session_context" | "direct_missing_context" | "positive_rollback_then_reuse" => {
            context_probe(service, input, &request, &mut clock).await?
        }
        "two_slot_then_third_open" | "two_slot_close_then_replace" => slots(service, input).await?,
        "history_fault_after_four" => history(service, input).await?,
        "lost_commit_completion"
        | "lost_rollback_completion"
        | "drop_fenced_transaction_before_terminal" => {
            terminal(
                service,
                input,
                &request,
                &mut clock,
                &trusted["effect_identities"],
            )
            .await?
        }
        "actual_sql_row_channel" => channel(service, input).await?,
        "selected_child_lock_probe" | "unselected_child_lock_probe" => {
            crate::authority_races::child_lock(service, input, &request, &mut clock).await?
        }
        _ => {
            adapter(
                service,
                &request,
                &mut clock,
                &trusted["effect_identities"],
                operation == "qualified_synthetic_cas",
            )
            .await?
        }
    };
    if operation == "trusted_clock_failure" {
        observed["clock_calls"] = json!(clock.calls);
    }
    let after = shared::snapshot(service, &mut observer_clock, false).await?;
    let physical_after = physical_authority(service).await?;
    let after_markers = shared::markers(service).await?;
    observed["authority_changed"] = json!(
        shared::relation_order(&before) != shared::relation_order(&after)
            || before_markers != after_markers
            || physical_before != physical_after
    );
    observed["marker_rows_added"] = json!(
        after_markers
            .as_array()
            .unwrap()
            .len()
            .checked_sub(before_markers.as_array().unwrap().len())
            .ok_or("marker rows disappeared")?
    );
    if !setup.is_null() {
        observed["sql_output"] = setup.get("guarded").unwrap_or(&setup).clone();
        if !setup["constraint_observation"].is_null() {
            observed["constraint_observation"] = setup["constraint_observation"].clone();
        }
    }
    if matches!(
        operation,
        "isolated_catalogue_boundary"
            | "catalogue_required_column_identity"
            | "selected_immutable_ordinal_gap"
    ) {
        for (key, value) in setup
            .as_object()
            .ok_or("invalid catalogue setup observation")?
        {
            observed[key] = value.clone();
        }
    }
    if operation == "permission_boundary" {
        let before_epoch = &physical_before["namespace_guard"][0]["epoch"];
        let after_epoch = &physical_after["namespace_guard"][0]["epoch"];
        observed["namespace_epoch_after"] = json!(
            after_epoch
                .as_i64()
                .ok_or("invalid observed namespace epoch")?
                .to_string()
        );
        observed["namespace_epoch_unchanged"] = json!(before_epoch == after_epoch);
        observed["target_after"] = canonical_permission_target(&after, input)?;
        observed["unrelated_canonical_rows_unchanged"] = json!(
            permission_without_target(physical_before.clone(), input)?
                == permission_without_target(physical_after.clone(), input)?
        );
    }
    if operation == "legacy_null_metadata" {
        for (key, value) in legacy(service, input, &request, &seed_observation)
            .await?
            .as_object()
            .unwrap()
        {
            observed[key] = value.clone();
        }
        observed["human_inferred"] =
            json!(!observed["failure"].is_string() && input["relation"] == "principals");
        observed["owner_inferred"] =
            json!(!observed["failure"].is_string() && input["relation"] == "memberships");
        observed["seed_failures"] = json!(0);
    }
    let target = &input["target"];
    let resource = if target.get("resource_id").is_some_and(Value::is_string) {
        json!({"id":target["resource_id"],"kind":target["resource_kind"],
            "workspace":{"organization_id":target["organization_id"],"domain_id":target["domain_id"]}})
    } else {
        request["resource"].clone()
    };
    if resource["id"].is_string() {
        observed["resource_control_after"] = shared::observed_resource(&after, &resource)?
            .map(|r| r["control_revision"]["value"].clone())
            .unwrap_or(Value::Null);
    }
    let controlled_history = operation == "history_fault_after_four"
        && history_statement(shared::text(&input["fault"]))?.controlled_shape;
    let accounting = if controlled_history {
        shared::accounting_for_history_shape(service).await?
    } else {
        shared::accounting(service).await?
    };
    observed["migration_versions"] = accounting["migration_versions"].clone();
    observed["accounting"] = accounting;
    observed["before"] = before;
    observed["physical_before"] = shared::retain_observation(&physical_before)?;
    observed["seed_observation"] = shared::retain_observation(&seed_observation)?;
    observed["after"] = after;
    observed["physical_after"] = shared::retain_observation(&physical_after)?;
    observed["catalogue_setup_before"] = catalogue_before;
    observed["catalogue_setup_after"] = catalogue_after_setup;
    observed["markers_after"] = after_markers;
    observed["census"] = shared::census(service).await?;
    Ok(observed)
}

/// Read only the independently admitted finite package and public source pins.
pub fn close_fixtures() -> Result<Value> {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/security/fixtures/owned-transport-close-v1");
    close_fixture_package(
        &base,
        "owned-transport-close-profile-v1",
        "owned-transport-close-cases-v1",
    )
}

pub fn close_guard_fixtures() -> Result<Value> {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/security/fixtures/owned-transport-close-guards-v1");
    close_fixture_package(
        &base,
        "owned-transport-close-guards-profile-v1",
        "owned-transport-close-guards-cases-v1",
    )
}

pub fn census_fixtures() -> Result<Value> {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/security/fixtures/owned-database-census-v1");
    close_fixture_package(
        &base,
        "owned-database-census-profile-v1",
        "owned-database-census-cases-v1",
    )
}

pub fn helper_input_error_fixtures() -> Result<Value> {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/security/fixtures/owned-helper-input-errors-v1");
    close_fixture_package(
        &base,
        "owned-helper-input-errors-profile-v1",
        "owned-helper-input-errors-cases-v1",
    )
}

/// Read only the fixed missing PID path inside the existing owned package copy.
pub fn missing_owned_pid_guard(input: &Value) -> Result<Value> {
    crate::authority_service::close_package_guard(input, |base| {
        crate::authority_service::pid_file(&base.join("postmaster.pid"), 17).map(|()| Value::Null)
    })
}

/// Only the finite input reaches the owned-copy producer and actual loader.
pub fn close_package_guard(input: &Value) -> Result<Value> {
    crate::authority_service::close_package_guard(input, |base| {
        close_fixture_package(
            base,
            "owned-transport-close-profile-v1",
            "owned-transport-close-cases-v1",
        )
    })
}

/// Preserve exact pins or the single fixed directed lock-source context.
pub fn source_pin_matches(source_pin: &Value, actual: &Value) -> bool {
    let (
        Some(path),
        Some(expected_hash),
        Some(expected_bytes),
        Some(actual_hash),
        Some(actual_bytes),
    ) = (
        source_pin["path"].as_str(),
        source_pin["sha256"].as_str(),
        source_pin["bytes"].as_u64(),
        actual["sha256"].as_str(),
        actual["size"].as_u64(),
    )
    else {
        return false;
    };
    if ![expected_hash, actual_hash].into_iter().all(|hash| {
        hash.len() == 64
            && hash
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    }) {
        return false;
    }
    if (expected_hash, expected_bytes) == (actual_hash, actual_bytes) {
        return true;
    }
    let context: Value = serde_json::from_str(include_str!(
        "../../../../tests/security/fixtures/owned-source-lock-context-v1/context.json"
    ))
    .expect("fixed source-lock context must be valid JSON");
    path == "Cargo.lock"
        && context["version"] == "owned-source-lock-context-v1"
        && context["source_path"] == path
        && context["source_authority"]["exact_dependency_graph_sha256"]
            == "40e6539778730e8fd9648190c7fc05c527fbeda5ca66611f811bce692310a2da"
        && context["baseline"]["sha256"] == expected_hash
        && context["baseline"]["bytes"].as_u64() == Some(expected_bytes)
        && context["candidate"]["sha256"] == actual_hash
        && context["candidate"]["bytes"].as_u64() == Some(actual_bytes)
}

fn close_fixture_package(
    base: &std::path::Path,
    profile_version: &str,
    cases_version: &str,
) -> Result<Value> {
    use arboresce_xtask::{collection_prepared, inventory};
    use std::{path::Path, time::Instant};
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut names = std::fs::read_dir(base)?
        .map(|entry| {
            let entry = entry?;
            if !entry.file_type()?.is_file() {
                return Err("nonregular owned close package entry".into());
            }
            Ok(entry.file_name())
        })
        .collect::<Result<Vec<_>>>()?;
    names.sort();
    if names != ["README.md", "cases.json", "profile.json"].map(std::ffi::OsString::from) {
        return Err("owned close package paths differ".into());
    }
    let (profile, _) = collection_prepared::read_json(&base.join("profile.json"), 65536)?;
    for row in profile["sources"]
        .as_array()
        .ok_or("missing owned close source pins")?
    {
        let path = row["path"]
            .as_str()
            .ok_or("missing owned close source path")?;
        inventory::portable_path(Path::new(path))?;
        let actual = collection_prepared::fingerprint(
            &root.join(path),
            8 << 20,
            Instant::now() + Duration::from_secs(10),
        )?;
        if !source_pin_matches(row, &actual) {
            return Err("owned close source pin mismatch".into());
        }
    }
    for (name, row) in profile["files"]
        .as_object()
        .ok_or("missing owned close file pins")?
    {
        let cap = match name.as_str() {
            "README.md" => 16384,
            "cases.json" => 65536,
            _ => return Err("unregistered owned close fixture file".into()),
        };
        let actual = collection_prepared::fingerprint(
            &base.join(name),
            cap,
            Instant::now() + Duration::from_secs(10),
        )?;
        if actual["sha256"] != row["sha256"] || actual["size"] != row["bytes"] {
            return Err("owned close fixture pin mismatch".into());
        }
    }
    let (cases, _) = collection_prepared::read_json(&base.join("cases.json"), 65536)?;
    if profile["fixture_version"] != profile_version
        || cases["fixture_version"] != cases_version
        || profile["runtime_projection"]["allowed"] != json!(["common_inputs", "case.input"])
    {
        return Err("unsupported owned close fixture projection".into());
    }
    Ok(cases)
}

/// Controlled completion observations; no PostgreSQL or socket is constructed.
pub async fn controlled_close(input: &Value) -> Result<Value> {
    use crate::implementation::authority::{TransportCloseFailure, close_completion};
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    };
    use tokio::{sync::oneshot, time::Instant};

    struct Destruction(Arc<AtomicBool>);
    impl Drop for Destruction {
        fn drop(&mut self) {
            self.0.store(true, Ordering::SeqCst);
        }
    }

    let operation = input["operation"]
        .as_str()
        .ok_or("missing close operation")?;
    let deadline = if matches!(operation, "ready_after_deadline" | "pending_at_deadline") {
        Instant::now() - Duration::from_millis(1)
    } else {
        Instant::now() + Duration::from_secs(1)
    };
    let destroyed = Arc::new(AtomicBool::new(false));
    let marker = Destruction(destroyed.clone());
    let (started, ready) = oneshot::channel();
    let (release, waiting) = oneshot::channel();
    let (pending, panic) = match operation {
        "completed" | "ready_after_deadline" => (false, false),
        "panicked" => (false, true),
        "cancelled" | "pending_at_deadline" => (true, false),
        _ => return Err("unadmitted controlled close operation".into()),
    };
    let mut task = tokio::spawn(async move {
        let marker = marker;
        let _ = started.send(());
        if pending {
            let _ = waiting.await;
        }
        if panic {
            panic!("controlled owned-close task panic");
        }
        drop(marker);
    });
    tokio::time::timeout(Duration::from_secs(1), ready).await??;
    if operation == "cancelled" {
        task.abort();
    }
    if operation != "pending_at_deadline" {
        tokio::time::timeout(Duration::from_secs(1), async {
            while !task.is_finished() {
                tokio::task::yield_now().await;
            }
        })
        .await?;
    }
    let result = close_completion(&mut task, deadline).await;
    let failure = result.as_ref().err().map(|failure| match failure {
        TransportCloseFailure::Deadline => "deadline",
        TransportCloseFailure::Task(error) if error.is_cancelled() => "task_cancelled",
        TransportCloseFailure::Task(_) => "task_panic",
    });
    let mut semantic = json!({
        "closed":result.is_ok(),
        "failure":failure,
        "task_finished":task.is_finished(),
        "destruction_completed":destroyed.load(Ordering::SeqCst),
        "later_cleanup_completed":false,
    });
    let evidence = json!({
        "kind":"controlled_task",
        "diagnostic":result.as_ref().err().map(ToString::to_string),
        "source":result.as_ref().err().and_then(|error| std::error::Error::source(error)).map(ToString::to_string),
        "original_deadline_elapsed":Instant::now() > deadline,
    });
    if operation == "pending_at_deadline" {
        release
            .send(())
            .map_err(|_| "controlled task release failed")?;
        tokio::time::timeout(Duration::from_secs(1), &mut task).await??;
        semantic["later_cleanup_completed"] = json!(destroyed.load(Ordering::SeqCst));
    }
    Ok(json!({"semantic":semantic,"evidence":evidence}))
}

/// Actual owned-backend observations; no expected object reaches this producer.
pub async fn native_close(service: &Service, common: &Value, input: &Value) -> Result<Value> {
    if common["close_budget_ms"] != 1000 || common["census_budget_ms"] != 1000 {
        return Err("unsupported owned close budgets".into());
    }
    let operation = input["operation"]
        .as_str()
        .ok_or("missing native close operation")?;
    let peer = matches!(
        operation,
        "raw_peer_terminated" | "authority_peer_terminated" | "authority_peer_poisoned"
    );
    let mut backend = 0;
    let mut before = Value::Null;
    let mut terminated = Value::Null;
    let mut semantic = json!({
        "operation_value":null,"operation_failure":null,"close":"complete",
        "owned_backend_present_before":false,"owned_backend_absent_after":false,
        "peer_termination_acknowledged":null,"slots_before_close":null,"slots_after_close":null,
    });
    let close_observation;
    if matches!(
        operation,
        "raw_value" | "raw_operation_error" | "raw_peer_terminated"
    ) {
        let mut error_address = 0usize;
        let mut raw_close_started = None;
        let result = with_connection(service.admin(false).await?, async |connection| {
            let outcome: Result<i64> = async {
                backend = timeout(
                    Duration::from_secs(1),
                    sqlx::query_scalar("SELECT pg_backend_pid()").fetch_one(&mut *connection),
                )
                .await??;
                before = shared::census(service).await?;
                if operation == "raw_operation_error" {
                    let result = timeout(
                        Duration::from_secs(1),
                        sqlx::query_scalar::<_, i32>("SELECT 1/0").fetch_one(&mut *connection),
                    )
                    .await?;
                    result.map(i64::from).map_err(|error| {
                        let boxed: Box<dyn std::error::Error + Send + Sync> = Box::new(error);
                        error_address = std::ptr::from_ref(boxed.as_ref()) as *const () as usize;
                        boxed
                    })
                } else {
                    let supplied = input["value"].as_i64().ok_or("missing raw close value")?;
                    let value = timeout(
                        Duration::from_secs(1),
                        sqlx::query_scalar::<_, i64>("SELECT $1::bigint")
                            .bind(supplied)
                            .fetch_one(&mut *connection),
                    )
                    .await??;
                    if peer {
                        terminated = terminate_owned_backend(service, backend).await?;
                    }
                    Ok(value)
                }
            }
            .await;
            raw_close_started = Some(tokio::time::Instant::now());
            outcome
        })
        .await;
        let elapsed = raw_close_started
            .ok_or("raw close lacks operation completion")?
            .elapsed();
        if elapsed > Duration::from_secs(1) {
            return Err("raw owned close exceeded deadline".into());
        }
        match result {
            Ok(value) => semantic["operation_value"] = json!(value),
            Err(error) if operation == "raw_operation_error" => {
                let Some(actual) = error.downcast_ref::<sqlx::Error>() else {
                    return Err(error);
                };
                semantic["operation_failure"] = json!({
                    "kind":"sqlx",
                    "sqlstate":actual.as_database_error().and_then(|error|error.code()).map(|code|code.into_owned()),
                    "original_allocation":std::ptr::from_ref(actual) as usize == error_address,
                });
            }
            Err(error) => return Err(error),
        }
        close_observation = json!({"kind":"raw","elapsed_seconds":elapsed.as_secs_f64()});
    } else if matches!(
        operation,
        "authority_value" | "authority_peer_terminated" | "authority_peer_poisoned"
    ) {
        let sessions = service.harness.authority_sessions();
        let mut session = sessions.admission().await?;
        let outcome: Result<()> = async {
            backend = session.backend_id().await?;
            before = shared::census(service).await?;
            if peer {
                terminated = terminate_owned_backend(service, backend).await?;
            }
            if operation == "authority_peer_poisoned" {
                let failure = session.backend_id().await.err();
                semantic["operation_failure"] = failure
                    .map(|failure| json!({"kind":"authority","variant":failure.to_string()}))
                    .unwrap_or(Value::Null);
            }
            Ok(())
        }
        .await;
        semantic["slots_before_close"] = json!(sessions.available_slots());
        let started = tokio::time::Instant::now();
        let closed = session.close().await.map_err(Into::into);
        let elapsed = started.elapsed();
        crate::authority_service::preserve_cleanup(outcome, closed)?;
        if elapsed > Duration::from_secs(1) {
            return Err("authority owned close exceeded deadline".into());
        }
        close_observation = json!({"kind":"authority","elapsed_seconds":elapsed.as_secs_f64()});
        semantic["slots_after_close"] = json!(sessions.available_slots());
    } else {
        return Err("unadmitted native close operation".into());
    }
    service.empty_database().await?;
    let after = shared::census(service).await?;
    semantic["owned_backend_present_before"] = json!(census_contains_backend(&before, backend)?);
    semantic["owned_backend_absent_after"] = json!(!census_contains_backend(&after, backend)?);
    if peer {
        semantic["peer_termination_acknowledged"] = terminated["acknowledged"].clone();
    }
    Ok(json!({"semantic":semantic,"evidence":{
        "backend":backend,"before":before,"after":after,"termination":terminated,"close":close_observation,
    }}))
}

async fn terminate_owned_backend(service: &Service, backend: i32) -> Result<Value> {
    // This PID was read from the exact owned handle, never supplied by a fixture.
    let acknowledged = service
        .with_admin(true, async |connection| {
            Ok(timeout(
                Duration::from_secs(1),
                sqlx::query_scalar::<_, bool>("SELECT pg_terminate_backend($1)")
                    .bind(backend)
                    .fetch_one(connection),
            )
            .await??)
        })
        .await?;
    if !acknowledged {
        return Err("owned backend termination was not acknowledged".into());
    }
    service.empty_database().await?;
    let absent = shared::census(service).await?;
    if census_contains_backend(&absent, backend)? {
        return Err("terminated owned backend remains present".into());
    }
    Ok(json!({"acknowledged":acknowledged,"backend":backend,"census_before_close":absent}))
}

fn census_contains_backend(observation: &Value, backend: i32) -> Result<bool> {
    let rows = observation["backends"]
        .as_array()
        .ok_or("invalid close backend census")?;
    rows.iter().try_fold(false, |found, row| {
        let pid = row["pid"]
            .as_i64()
            .filter(|pid| *pid > 0)
            .ok_or("invalid close census backend identity")?;
        Ok(found || pid == i64::from(backend))
    })
}
