//! Input-only preparation and actual observations for current SQL authority.

use crate::{
    authority_service::{Result, Service, with_connection},
    implementation::{AuthorityFailure, PermissionChange, SchemaVersion},
};
use arboresce_application::policy::{
    ReferenceKind, ScopedReference, Workspace,
    authority::{AuthorityObservation, EvaluationRequest, PolicyConfiguration},
};
use arboresce_domain::{Clock, ControlRevision, Identifier, UtcTimestamp};
use arboresce_xtask::{collection_prepared, inventory};
use serde_json::{Value, json};
use sqlx::{PgConnection, Row};
use std::{
    io::Write,
    os::unix::fs::OpenOptionsExt,
    path::Path,
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};

static NEXT_OBSERVATION: AtomicU64 = AtomicU64::new(0);

/// Full native observations stay in the Cargo-owned retained evidence tree.
/// Stdout binds these exact bytes without raising its existing finite ceiling.
pub fn retain_observation(value: &Value) -> Result<Value> {
    let bytes = serde_json::to_vec(value)?;
    if bytes.len() > 2097152 {
        return Err("native observation exceeds retained file bound".into());
    }
    let parent = Path::new(env!("CARGO_TARGET_TMPDIR"));
    std::fs::create_dir_all(parent)?;
    if !std::fs::symlink_metadata(parent)?.is_dir() {
        return Err("native observation parent is not a real directory".into());
    }
    let path = parent.join(format!(
        "authority-observation-{}-{}.json",
        std::process::id(),
        NEXT_OBSERVATION.fetch_add(1, Ordering::Relaxed)
    ));
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&path)?;
    file.write_all(&bytes)?;
    file.flush()?;
    drop(file);
    let actual =
        collection_prepared::fingerprint(&path, 2097152, Instant::now() + Duration::from_secs(10))?;
    if actual["sha256"] != inventory::sha256(&bytes) || actual["size"] != bytes.len() as u64 {
        return Err("retained native observation readback differs".into());
    }
    Ok(json!({"path":path,"sha256":actual["sha256"],"size":actual["size"]}))
}

/// Complete bounded actual constraint identities, definitions and key columns.
pub async fn constraints(connection: &mut PgConnection) -> Result<Value> {
    let rows = sqlx::query("SELECT jsonb_build_object(\
        'relation',CASE WHEN c.conrelid=0 THEN '(domain) '||c.contypid::regtype::text ELSE c.conrelid::regclass::text END,\
        'name',c.conname,'type',c.contype::text,'validated',c.convalidated,\
        'deferrable',c.condeferrable,'deferred',c.condeferred,'noinherit',c.connoinherit,\
        'columns',coalesce((SELECT jsonb_agg(a.attname ORDER BY k.ordinal) FROM unnest(c.conkey) WITH ORDINALITY k(number,ordinal) JOIN pg_attribute a ON a.attrelid=c.conrelid AND a.attnum=k.number),'[]'::jsonb),\
        'referenced_relation',CASE WHEN c.confrelid<>0 THEN c.confrelid::regclass::text END,\
        'referenced_columns',coalesce((SELECT jsonb_agg(a.attname ORDER BY k.ordinal) FROM unnest(c.confkey) WITH ORDINALITY k(number,ordinal) JOIN pg_attribute a ON a.attrelid=c.confrelid AND a.attnum=k.number),'[]'::jsonb),\
        'definition',CASE WHEN octet_length(pg_get_constraintdef(c.oid,true))<=8192 THEN pg_get_constraintdef(c.oid,true) END)::text AS observed \
        FROM pg_constraint c WHERE c.connamespace='arboresce_authority'::regnamespace \
        ORDER BY c.conrelid,c.conname LIMIT 257")
        .fetch_all(connection).await?;
    if rows.len() > 256 {
        return Err("constraint catalogue lookahead exceeded".into());
    }
    let mut actual = Vec::new();
    for row in rows {
        let raw: String = row.try_get("observed")?;
        if raw.len() > 16384 {
            return Err("constraint catalogue record exceeded".into());
        }
        let value: Value = serde_json::from_str(&raw)?;
        if !value["definition"].is_string() {
            return Err("constraint definition exceeds SQL output guard".into());
        }
        actual.push(value);
    }
    // OIDs change on owned resets; only actual stable logical catalogue fields
    // participate in equality, without dropping their definitions or ordering.
    actual.sort_by_key(|v| {
        (
            v["relation"].as_str().unwrap().to_owned(),
            v["name"].as_str().unwrap().to_owned(),
        )
    });
    Ok(json!(actual))
}

pub async fn controlled_drop(
    connection: &mut PgConnection,
    relation: &str,
    name: &str,
    sql: &'static str,
) -> Result<Value> {
    let before = constraints(connection).await?;
    let selected = format!("arboresce_authority.{relation}");
    let rest: Vec<_> = before
        .as_array()
        .ok_or("invalid before constraint vector")?
        .iter()
        .filter(|v| !(v["relation"] == selected && v["name"] == name))
        .cloned()
        .collect();
    if rest.len() + 1 != before.as_array().unwrap().len() {
        return Err("controlled constraint target is not exactly one actual row".into());
    }
    let before_ref = retain_observation(&before)?;
    sqlx::query(sql).execute(&mut *connection).await?;
    let after = constraints(connection).await?;
    let after_ref = retain_observation(&after)?;
    if after != json!(rest) {
        return Err("controlled constraint delta changed unrelated catalogue".into());
    }
    Ok(
        json!({"relation":relation,"constraint":name,"before":before_ref,"after":after_ref,
        "exact_named_removal":true,"all_other_constraints_unchanged":true}),
    )
}

pub fn fixtures() -> Result<(Value, Value, Value)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let base = root.join("tests/security/fixtures/current-sql-authority-v1");
    let mut names = std::fs::read_dir(&base)?
        .map(|entry| {
            let entry = entry?;
            if !entry.file_type()?.is_file() {
                return Err("nonregular current SQL package entry".into());
            }
            Ok(entry.file_name())
        })
        .collect::<Result<Vec<_>>>()?;
    names.sort();
    if names
        != [
            "README.md",
            "cases.json",
            "profile.json",
            "realization.json",
            "schema.json",
        ]
        .map(std::ffi::OsString::from)
    {
        return Err("current SQL package paths differ".into());
    }
    let (profile, _) = collection_prepared::read_json(&base.join("profile.json"), 65536)?;
    bounded_json(&profile)?;
    for (path, record) in profile["historical_sources"]
        .as_object()
        .ok_or("missing historical source context")?
    {
        inventory::portable_path(Path::new(path))?;
        let bytes = record["source_utf8"]
            .as_str()
            .ok_or("missing inert historical bytes")?
            .as_bytes();
        if inventory::sha256(bytes) != record["sha256"]
            || bytes.len() as u64
                != record["bytes"]
                    .as_u64()
                    .ok_or("missing historical length")?
        {
            return Err("historical migration context mismatch".into());
        }
    }
    for (path, expected) in profile["source_sha256"]
        .as_object()
        .ok_or("missing source pins")?
    {
        inventory::portable_path(Path::new(path))?;
        let actual = collection_prepared::fingerprint(
            &root.join(path),
            8388608,
            Instant::now() + Duration::from_secs(10),
        )?;
        if actual["sha256"] != *expected {
            return Err("current SQL source pin mismatch".into());
        }
    }
    let mut total = std::fs::metadata(base.join("profile.json"))?.len();
    for (path, expected) in profile["package_files"]
        .as_object()
        .ok_or("missing package pins")?
    {
        inventory::portable_path(Path::new(path))?;
        let actual = collection_prepared::fingerprint(
            &base.join(path),
            262144,
            Instant::now() + Duration::from_secs(10),
        )?;
        if actual["sha256"] != expected["sha256"] || actual["size"] != expected["bytes"] {
            return Err("current SQL package pin mismatch".into());
        }
        total += actual["size"].as_u64().ok_or("missing package bytes")?;
    }
    if total > 524288 {
        return Err("current SQL package limit exceeded".into());
    }
    let (authority, _) = collection_prepared::read_json(
        &root.join("tests/security/fixtures/authority-v1/cases.json"),
        8388608,
    )?;
    let (additional, _) = collection_prepared::read_json(&base.join("cases.json"), 262144)?;
    let (realization, _) = collection_prepared::read_json(&base.join("realization.json"), 262144)?;
    let (schema, _) = collection_prepared::read_json(&base.join("schema.json"), 262144)?;
    for value in [&additional, &realization, &schema] {
        bounded_json(value)?;
    }
    if additional["cases"]
        .as_array()
        .ok_or("missing SQL cases")?
        .len()
        + additional["error_cases"]
            .as_array()
            .ok_or("missing SQL error cases")?
            .len()
        + additional["guard_cases"]
            .as_array()
            .ok_or("missing helper guard cases")?
            .len()
        + additional["migration_error_cases"]
            .as_array()
            .ok_or("missing migration classification cases")?
            .len()
        + additional["cleanup_cases"]
            .as_array()
            .ok_or("missing cleanup result cases")?
            .len()
        + additional["race_guard_cases"]
            .as_array()
            .ok_or("missing race helper preparation cases")?
            .len()
        > 256
    {
        return Err("current SQL package case limit exceeded".into());
    }
    Ok((authority, additional, realization))
}

fn bounded_json(value: &Value) -> Result<()> {
    let mut pending = vec![(value, 1)];
    while let Some((value, depth)) = pending.pop() {
        if depth > 32 {
            return Err("current SQL package JSON depth exceeded".into());
        }
        match value {
            Value::Array(values) => pending.extend(values.iter().map(|v| (v, depth + 1))),
            Value::Object(values) => pending.extend(values.values().map(|v| (v, depth + 1))),
            _ => {}
        }
    }
    Ok(())
}

pub fn input_configuration() -> Result<Value> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let (profile, _) = collection_prepared::read_json(
        &root.join("tests/security/fixtures/current-sql-authority-v1/profile.json"),
        65536,
    )?;
    Ok(profile["trusted_clock_inputs"].clone())
}

pub fn id(v: &Value) -> Identifier {
    v.as_str().unwrap().parse().unwrap()
}
pub fn text(v: &Value) -> &str {
    v.as_str().unwrap()
}
pub fn revision(v: &Value) -> ControlRevision {
    assert_eq!(v["kind"], "control");
    text(&v["value"]).parse().unwrap()
}
pub fn workspace(v: &Value) -> Workspace {
    Workspace::new(id(&v["organization_id"]), id(&v["domain_id"]))
}
fn optional_text(v: &Value) -> Option<&str> {
    v.as_str()
}
fn strings(v: &Value) -> Vec<String> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|s| text(s).to_owned())
        .collect()
}
fn revision_number(v: &Value) -> i64 {
    revision(v).value() as i64
}
pub fn request(v: &Value) -> EvaluationRequest {
    let r = &v["resource"];
    EvaluationRequest {
        executor: id(&v["executor_principal_id"]),
        selected_workspace: (!v["selected_workspace"].is_null())
            .then(|| workspace(&v["selected_workspace"])),
        resource: ScopedReference::new(
            workspace(&r["workspace"]),
            match text(&r["kind"]) {
                "project" => ReferenceKind::Project,
                "expense" => ReferenceKind::Expense,
                _ => panic!("unadmitted resource family"),
            },
            id(&r["id"]),
        ),
        action: text(&v["action"]).to_owned(),
        purpose: text(&v["purpose"]).to_owned(),
        selected_chain: v["selected_chain_id"].as_str().map(|s| s.parse().unwrap()),
        expected_control: (!v["expected_resource_control"].is_null())
            .then(|| revision(&v["expected_resource_control"])),
        claimed_authorizer: v["claimed_authorizer"].as_str().map(|s| s.parse().unwrap()),
    }
}
pub fn configuration() -> PolicyConfiguration {
    PolicyConfiguration {
        supported_policies: vec!["authority_fixture_v1".to_owned()],
        supported_source_policies: vec!["source_current".to_owned()],
    }
}
#[derive(Debug)]
pub struct SuppliedClockFailure;

impl std::fmt::Display for SuppliedClockFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("declared trusted Clock failure")
    }
}

impl std::error::Error for SuppliedClockFailure {}

pub struct SuppliedClock {
    pub value: UtcTimestamp,
    pub calls: usize,
    samples: Vec<UtcTimestamp>,
    unavailable: bool,
}
impl SuppliedClock {
    pub fn new(v: &Value) -> Self {
        let value = text(v).parse().unwrap();
        Self {
            value,
            calls: 0,
            samples: vec![value],
            unavailable: false,
        }
    }
    pub fn sequence(v: &Value) -> Self {
        let samples: Vec<UtcTimestamp> = v
            .as_array()
            .unwrap()
            .iter()
            .map(|v| text(v).parse().unwrap())
            .collect();
        assert_eq!(samples.len(), 2);
        Self {
            value: samples[0],
            calls: 0,
            samples,
            unavailable: false,
        }
    }

    pub fn failing(value: &Value, trusted: &Value) -> Result<Self> {
        if trusted["port"] != "Clock::now"
            || trusted["result"] != "declared error without a timestamp"
            || trusted["calls"] != 1
        {
            return Err("unadmitted trusted clock failure configuration".into());
        }
        let mut clock = Self::new(value);
        clock.unavailable = true;
        Ok(clock)
    }
}
impl Clock for SuppliedClock {
    type Error = SuppliedClockFailure;
    fn now(&mut self) -> std::result::Result<UtcTimestamp, Self::Error> {
        let index = self.calls;
        self.calls += 1;
        if self.unavailable {
            Err(SuppliedClockFailure)
        } else {
            self.value = self.samples[index.min(self.samples.len() - 1)];
            Ok(self.value)
        }
    }
}

/// Literal structured error mappings only, never an observed SQL/I/O cause.
pub fn migration_error_observation(input: &Value) -> Result<Value> {
    use sqlx::migrate::MigrateError;
    let version = || {
        input["version"]
            .as_i64()
            .ok_or("missing literal migration version")
    };
    let error = match input["variant"].as_str() {
        Some("VersionMissing") => MigrateError::VersionMissing(version()?),
        Some("VersionMismatch") => MigrateError::VersionMismatch(version()?),
        Some("Dirty") => MigrateError::Dirty(version()?),
        Some("ExecuteUnclassifiedTransport") => {
            MigrateError::Execute(sqlx::Error::Io(std::io::Error::other(
                input["message"]
                    .as_str()
                    .ok_or("missing literal transport message")?
                    .to_owned(),
            )))
        }
        _ => return Err("unadmitted literal migration error variant".into()),
    };
    Ok(json!({"failure":format!("{:?}", crate::implementation::migration::classify(error))}))
}
/// Input-only finite Result and constructor observations remain measured.
/// Evaluate separately admitted finite inputs without database effects.
#[derive(Debug)]
struct OperationSentinel(serde_json::Value);
#[derive(Debug)]
struct CleanupSentinel(serde_json::Value);
impl std::fmt::Display for OperationSentinel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "operation sentinel {}", self.0)
    }
}
impl std::fmt::Display for CleanupSentinel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "cleanup sentinel {}", self.0)
    }
}
impl std::error::Error for OperationSentinel {}
impl std::error::Error for CleanupSentinel {}

pub fn cleanup_result(input: &serde_json::Value) -> serde_json::Value {
    let value = Box::new(input["operation"]["value"].clone());
    let value_address = std::ptr::from_ref(value.as_ref());
    let operation = Box::new(OperationSentinel(input["operation"]["error"].clone()));
    let operation_address = std::ptr::from_ref(operation.as_ref());
    let cleanup = Box::new(CleanupSentinel(input["cleanup"]["error"].clone()));
    let cleanup_address = std::ptr::from_ref(cleanup.as_ref());
    let operation: crate::authority_service::Result<_> =
        if input["operation"].get("error").is_some() {
            Err(operation)
        } else {
            Ok(value)
        };
    let cleanup: crate::authority_service::Result<()> = if input["cleanup"].get("error").is_some() {
        Err(cleanup)
    } else {
        Ok(())
    };
    let mut observed = json!({"value":null,"operation_error":null,"cleanup_error":null,
        "value_identity":null,"operation_error_identity":null,"cleanup_error_identity":null,"paired":false});
    match crate::authority_service::preserve_cleanup(operation, cleanup) {
        Ok(value) => {
            observed["value_identity"] = json!(std::ptr::eq(value_address, value.as_ref()));
            observed["value"] = *value;
        }
        Err(error) => {
            // Render the real error in addition to checking its typed payload.
            // No physical cause is attributed to these finite sentinels.
            println!("finite cleanup result error={error}");
            if let Some(pair) =
                error.downcast_ref::<crate::authority_service::OperationAndCleanupFailure>()
            {
                let operation = pair.operation.downcast_ref::<OperationSentinel>().unwrap();
                let cleanup = pair.cleanup.downcast_ref::<CleanupSentinel>().unwrap();
                observed["operation_error"] = operation.0.clone();
                observed["cleanup_error"] = cleanup.0.clone();
                observed["operation_error_identity"] =
                    json!(std::ptr::eq(operation_address, operation));
                observed["cleanup_error_identity"] = json!(std::ptr::eq(cleanup_address, cleanup));
                observed["paired"] = json!(true);
            } else if let Some(operation) = error.downcast_ref::<OperationSentinel>() {
                observed["operation_error"] = operation.0.clone();
                observed["operation_error_identity"] =
                    json!(std::ptr::eq(operation_address, operation));
            } else {
                let cleanup = error.downcast_ref::<CleanupSentinel>().unwrap();
                observed["cleanup_error"] = cleanup.0.clone();
                observed["cleanup_error_identity"] = json!(std::ptr::eq(cleanup_address, cleanup));
            }
        }
    }
    observed
}

pub fn race_preparation(input: &serde_json::Value) -> serde_json::Value {
    let result: crate::authority_service::Result<serde_json::Value> = match input["helper"]
        .as_str()
        .unwrap()
    {
        "milliseconds" => crate::authority_races::milliseconds(input["value"].as_str().unwrap())
            .map(|value| json!({"accepted":true,"milliseconds":value})),
        "authority_index" => crate::authority_races::authority_index(
            &input["state"],
            input["family"].as_str().unwrap(),
            &input["target"],
        )
        .map(|index| json!({"accepted":true,"index":index})),
        "probe_query" => crate::authority_races::probe_query(&input["query"], &input["request"])
            .map(|(sql, values)| json!({"accepted":true,"sql":sql,"values":values})),
        "history_statement" => crate::authority_probes::history_statement(text(&input["fault"]))
            .map(|_| json!({"accepted":true})),
        _ => panic!("unadmitted finite preparation input"),
    };
    result.unwrap_or_else(|error| {
        let failure = if error.is::<std::num::ParseIntError>() {
            "integer_parse".to_owned()
        } else {
            error.to_string()
        };
        json!({"accepted":false,"failure":failure})
    })
}

pub fn control(v: ControlRevision) -> Value {
    json!({"kind":"control","value":v.to_string()})
}
pub fn observation(v: AuthorityObservation<'_>) -> Value {
    json!({"executor_principal_id":v.executor().to_string(),
        "workspace":{"organization_id":v.workspace().organization().to_string(),"domain_id":v.workspace().domain().to_string()},
        "action":v.action(),"purpose":v.purpose(),"resource_id":v.resource().workspace().require_reference(v.resource()).unwrap().to_string(),
        "selected_chain_id":v.chain().map(|id|id.to_string()),"membership_id":v.membership().to_string(),
        "membership_revision":control(v.membership_revision()),"policy_epoch":control(v.policy_epoch()),"resource_control":control(v.resource_control())})
}
pub fn problem(failure: AuthorityFailure) -> Value {
    let (code, status) = match failure {
        AuthorityFailure::InvalidRequest => ("invalid_request", 400),
        AuthorityFailure::Unauthenticated => ("unauthenticated", 401),
        AuthorityFailure::NotFound => ("not_found", 404),
        AuthorityFailure::Forbidden => ("forbidden", 403),
        AuthorityFailure::Conflict => ("conflict", 409),
        _ => return json!({"internal_failure":failure.to_string()}),
    };
    json!({"code":code,"status":status})
}

/// Only the five exact independently reviewed corruption recipes are allowed.
async fn faults(connection: &mut PgConnection, faults: &Value, old: bool) -> Result<Value> {
    let mut observations = Vec::new();
    for fault in faults.as_array().ok_or("missing controlled faults")? {
        let key = (text(&fault["relation"]), text(&fault["constraint"]));
        let (is_old, sql) = match key {
            ("principals", "principals_identity_version_check") => (
                true,
                "ALTER TABLE arboresce_authority.principals DROP CONSTRAINT principals_identity_version_check",
            ),
            ("grants", "grants_scope_tenant") => (
                false,
                "ALTER TABLE arboresce_authority.grants DROP CONSTRAINT grants_scope_tenant",
            ),
            ("immutable_members", "immutable_members_scope_tenant") => (
                false,
                "ALTER TABLE arboresce_authority.immutable_members DROP CONSTRAINT immutable_members_scope_tenant",
            ),
            ("grants", "grants_parent_fkey") => (
                false,
                "ALTER TABLE arboresce_authority.grants DROP CONSTRAINT grants_parent_fkey",
            ),
            ("immutable_members", "immutable_members_chain_grant_key") => (
                false,
                "ALTER TABLE arboresce_authority.immutable_members DROP CONSTRAINT immutable_members_chain_grant_key",
            ),
            _ => return Err("unadmitted controlled constraint fault".into()),
        };
        if old == is_old {
            observations.push(controlled_drop(connection, key.0, key.1, sql).await?);
        }
    }
    Ok(json!(observations))
}

pub async fn seed(service: &Service, snapshot: &Value, recipe: &Value) -> Result<Value> {
    service.reset().await?;
    service
        .harness
        .migrate(SchemaVersion::TenantAuthority)
        .await
        .map_err(|e| format!("tenant migration failed: {e:?}"))?;
    let (v3_normal, old_faults) = service.with_admin(false, async |connection| {
    let v3_normal = retain_observation(&constraints(&mut *connection).await?)?;
    let old_faults = faults(
        &mut *connection,
        &recipe["controlled_constraint_faults"],
        true,
    )
    .await?;
    for v in snapshot["organizations"].as_array().unwrap() {
        sqlx::query("INSERT INTO arboresce_authority.organizations (organization_id,state) VALUES ($1::uuid,$2)")
            .bind(text(&v["organization_id"])).bind(text(&v["state"])).execute(&mut *connection).await?;
    }
    for v in snapshot["domains"].as_array().unwrap() {
        sqlx::query("INSERT INTO arboresce_authority.domains (organization_id,domain_id,state) VALUES ($1::uuid,$2::uuid,$3)")
            .bind(text(&v["organization_id"])).bind(text(&v["domain_id"])).bind(text(&v["state"])).execute(&mut *connection).await?;
    }
    for v in snapshot["principals"].as_array().unwrap() {
        let p = &v["identity_profile"];
        sqlx::query("INSERT INTO arboresce_authority.principals (principal_id,identity_profile,identity_version,public_key,state) VALUES ($1::uuid,$2,$3,decode($4,'hex'),$5)")
            .bind(text(&v["principal_id"])).bind(text(&p["profile"])).bind(text(&p["version"])).bind(text(&p["public_key"])).bind(text(&v["state"]))
            .execute(&mut *connection).await?;
    }
    for v in snapshot["memberships"].as_array().unwrap() {
        sqlx::query("INSERT INTO arboresce_authority.memberships (organization_id,domain_id,principal_id,membership_id,state) VALUES ($1::uuid,$2::uuid,$3::uuid,$4::uuid,$5)")
            .bind(text(&v["workspace"]["organization_id"])).bind(text(&v["workspace"]["domain_id"])).bind(text(&v["principal"]["principal_id"]))
            .bind(text(&v["membership_id"])).bind(text(&v["state"])).execute(&mut *connection).await?;
    }
    Ok((v3_normal, old_faults))
    }).await?;
    service
        .harness
        .migrate(SchemaVersion::CurrentAuthority)
        .await
        .map_err(|e| format!("current migration failed: {e:?}"))?;
    service.with_admin(false, async |connection| {
    let normal_constraints = constraints(&mut *connection).await?;
    let new_faults = faults(
        &mut *connection,
        &recipe["controlled_constraint_faults"],
        false,
    )
    .await?;
    for v in snapshot["organizations"].as_array().unwrap() {
        sqlx::query(
            "UPDATE arboresce_authority.organizations SET name=$2 WHERE organization_id=$1::uuid",
        )
        .bind(text(&v["organization_id"]))
        .bind(text(&v["name"]))
        .execute(&mut *connection)
        .await?;
    }
    for v in snapshot["domains"].as_array().unwrap() {
        sqlx::query("UPDATE arboresce_authority.domains SET name=$3 WHERE organization_id=$1::uuid AND domain_id=$2::uuid")
            .bind(text(&v["organization_id"])).bind(text(&v["domain_id"])).bind(text(&v["name"])).execute(&mut *connection).await?;
    }
    for v in snapshot["principals"].as_array().unwrap() {
        let p = &v["identity_profile"]["standard_revisions"]["nip01"];
        let legacy = &recipe["legacy_input"];
        let selected = legacy["relation"] == "principals"
            && legacy["target"]["principal_id"] == v["principal_id"];
        let missing = |field: &str| {
            selected
                && legacy["columns"]
                    .as_array()
                    .is_some_and(|columns| columns.iter().any(|column| column == field))
        };
        sqlx::query("UPDATE arboresce_authority.principals SET kind=$2,control_revision=$3,created_at=$4::timestamptz,revoked_at=$5::timestamptz,identity_standard=$6,identity_standard_revision=$7 WHERE principal_id=$1::uuid")
            .bind(text(&v["principal_id"])).bind((!missing("kind")).then(|| text(&v["kind"])))
            .bind((!missing("control_revision")).then(|| revision_number(&v["control_revision"])))
            .bind((!missing("created_at")).then(|| text(&v["created_at"])))
            .bind(optional_text(&v["revoked_at"]))
            .bind((!missing("identity_standard")).then(|| text(&p["standard"])))
            .bind((!missing("identity_standard_revision")).then(|| text(&p["revision"])))
            .execute(&mut *connection).await?;
    }
    for v in snapshot["memberships"].as_array().unwrap() {
        let legacy = &recipe["legacy_input"];
        let selected = legacy["relation"] == "memberships"
            && legacy["target"]["organization_id"] == v["workspace"]["organization_id"]
            && legacy["target"]["domain_id"] == v["workspace"]["domain_id"]
            && legacy["target"]["principal_id"] == v["principal"]["principal_id"]
            && legacy["target"]["membership_id"] == v["membership_id"];
        let missing = |field: &str| {
            selected
                && legacy["columns"]
                    .as_array()
                    .is_some_and(|columns| columns.iter().any(|column| column == field))
        };
        let nullable = legacy["legacy_setup"]["related_nullable_keys"]
            .as_array()
            .is_some_and(|rows| {
                rows.iter().any(|r| {
                    r["target"]["organization_id"] == v["workspace"]["organization_id"]
                        && r["target"]["domain_id"] == v["workspace"]["domain_id"]
                        && r["target"]["principal_id"] == v["principal"]["principal_id"]
                        && r["target"]["membership_id"] == v["membership_id"]
                })
            });
        let principal_kind =
            (!nullable && !missing("principal_kind")).then(|| text(&v["principal"]["kind"]));
        sqlx::query("UPDATE arboresce_authority.memberships SET principal_kind=$5,roles=$6,policy=$7,control_revision=$8 WHERE organization_id=$1::uuid AND domain_id=$2::uuid AND principal_id=$3::uuid AND membership_id=$4::uuid")
            .bind(text(&v["workspace"]["organization_id"])).bind(text(&v["workspace"]["domain_id"]))
            .bind(text(&v["principal"]["principal_id"])).bind(text(&v["membership_id"])).bind(principal_kind)
            .bind((!missing("roles")).then(|| strings(&v["roles"])))
            .bind((!missing("policy")).then(|| text(&v["policy"])))
            .bind((!missing("control_revision")).then(|| revision_number(&v["revision"])))
            .execute(&mut *connection).await?;
    }
    for v in snapshot["policies"].as_array().unwrap() {
        sqlx::query("INSERT INTO arboresce_authority.workspace_policies (organization_id,domain_id,policy,epoch,state,read_roles,direct_review_roles,purposes,grant_root_actions,source_policy_ids) VALUES ($1::uuid,$2::uuid,$3,$4,$5,$6,$7,$8,$9,$10)")
            .bind(text(&v["workspace"]["organization_id"])).bind(text(&v["workspace"]["domain_id"])).bind(text(&v["policy"]))
            .bind(revision_number(&v["epoch"])).bind(text(&v["state"])).bind(strings(&v["read_roles"])).bind(strings(&v["direct_review_roles"]))
            .bind(strings(&v["purposes"])).bind(strings(&v["grant_root_actions"])).bind(strings(&v["source_policy_ids"])).execute(&mut *connection).await?;
    }
    for v in snapshot["source_policies"].as_array().unwrap() {
        sqlx::query("INSERT INTO arboresce_authority.source_policies (policy_id,state,expires_at,permitted_principals,permitted_purposes) VALUES ($1,$2,$3::timestamptz,$4::text[]::arboresce_authority.identifier[],$5)")
            .bind(text(&v["policy_id"])).bind(text(&v["state"])).bind(text(&v["expires_at"]))
            .bind(strings(&v["permitted_principals"])).bind(strings(&v["permitted_purposes"])).execute(&mut *connection).await?;
    }
    for v in snapshot["resources"].as_array().unwrap() {
        sqlx::query("INSERT INTO arboresce_authority.resources (organization_id,domain_id,resource_kind,resource_id,name,control_revision,state) VALUES ($1::uuid,$2::uuid,$3,$4::uuid,$5,$6,$7)")
            .bind(text(&v["workspace"]["organization_id"])).bind(text(&v["workspace"]["domain_id"])).bind(text(&v["kind"])).bind(text(&v["id"]))
            .bind(text(&v["name"])).bind(revision_number(&v["control_revision"])).bind(text(&v["state"])).execute(&mut *connection).await?;
    }
    for v in snapshot["party_aliases"].as_array().unwrap() {
        sqlx::query("INSERT INTO arboresce_authority.party_aliases (organization_id,domain_id,party_id,alias,principal_id) VALUES ($1::uuid,$2::uuid,$3::uuid,$4,$5::uuid)")
            .bind(text(&v["workspace"]["organization_id"])).bind(text(&v["workspace"]["domain_id"])).bind(text(&v["party_id"]))
            .bind(text(&v["alias"])).bind(optional_text(&v["principal_id"])).execute(&mut *connection).await?;
    }
    let owner = workspace(&recipe["authority_owner"]);
    let omitted = |relation: &str| {
        recipe["legacy_input"]["legacy_setup"]["omit_unselected_relations"]
            .as_array()
            .is_some_and(|rows| rows.iter().any(|v| v == relation))
    };
    for v in snapshot["grants"].as_array().unwrap() {
        if omitted("grants") {
            continue;
        }
        seed_grant(&mut *connection, v, owner, None).await?;
    }
    for v in snapshot["immutable_chains"].as_array().unwrap() {
        if omitted("immutable_chains") {
            continue;
        }
        sqlx::query("INSERT INTO arboresce_authority.immutable_chains (organization_id,domain_id,chain_id,scope_interpretation) VALUES ($1::uuid,$2::uuid,$3::uuid,$4)")
            .bind(owner.organization().to_string()).bind(owner.domain().to_string()).bind(text(&v["chain_id"])).bind(text(&v["scope_interpretation"]))
            .execute(&mut *connection).await?;
        for (ordinal, member) in v["members"].as_array().unwrap().iter().enumerate() {
            seed_grant(
                &mut *connection,
                member,
                owner,
                Some((text(&v["chain_id"]), ordinal as i16)),
            )
            .await?;
        }
    }
    let seeded_constraints = constraints(&mut *connection).await?;
    Ok(
        json!({"v3_normal_constraints":v3_normal,"old_faults":old_faults,
        "normal_constraints":normal_constraints,"new_faults":new_faults,
        "seeded_constraints":seeded_constraints}),
    )
    }).await
}

async fn seed_grant(
    connection: &mut PgConnection,
    v: &Value,
    owner: Workspace,
    sealed: Option<(&str, i16)>,
) -> Result<()> {
    let (table, extra, values) = if sealed.is_some() {
        ("immutable_members", ",chain_id,ordinal", ",$22::uuid,$23")
    } else {
        ("grants", "", "")
    };
    // All syntax fragments are the two compiled alternatives above; data is bound.
    let sql = format!(
        "INSERT INTO arboresce_authority.{table} (organization_id,domain_id,grant_id,issuer_id,issuer_kind,subject_id,subject_kind,parent_grant_id,scope_organization_id,scope_domain_id,scope_kind,scope_resource_id,actions,purposes,delegable,issued_at,expires_at,policy,control_revision,state,revoked_at{extra}) VALUES ($1::uuid,$2::uuid,$3::uuid,$4::uuid,$5,$6::uuid,$7,$8::uuid,$9::uuid,$10::uuid,$11,$12::uuid,$13,$14,$15,$16::timestamptz,$17::timestamptz,$18,$19,$20,$21::timestamptz{values})"
    );
    let mut query = sqlx::query(sqlx::AssertSqlSafe(sql))
        .bind(owner.organization().to_string())
        .bind(owner.domain().to_string())
        .bind(text(&v["grant_id"]))
        .bind(text(&v["issuer"]["principal_id"]))
        .bind(text(&v["issuer"]["kind"]))
        .bind(text(&v["subject"]["principal_id"]))
        .bind(text(&v["subject"]["kind"]))
        .bind(optional_text(&v["parent_grant_id"]))
        .bind(text(&v["scope"]["organization_id"]))
        .bind(text(&v["scope"]["domain_id"]))
        .bind(text(&v["scope"]["resource_kind"]))
        .bind(optional_text(&v["scope"]["resource_id"]))
        .bind(strings(&v["actions"]))
        .bind(strings(&v["purposes"]))
        .bind(v["delegable"].as_bool().unwrap())
        .bind(text(&v["issued_at"]))
        .bind(text(&v["expires_at"]))
        .bind(text(&v["policy"]))
        .bind(revision_number(&v["revision"]))
        .bind(text(&v["state"]))
        .bind(optional_text(&v["revoked_at"]));
    if let Some((chain, ordinal)) = sealed {
        query = query.bind(chain).bind(ordinal);
    }
    query.execute(connection).await?;
    Ok(())
}

/// Infer a permitted mutation from complete independent before/after INPUT rows.
/// No expected verdict or expected after-state enters this selection.
pub fn change(before: &Value, after: &Value) -> Option<PermissionChange> {
    for (old, new) in before["memberships"]
        .as_array()
        .unwrap()
        .iter()
        .zip(after["memberships"].as_array().unwrap())
    {
        if old != new {
            return Some(PermissionChange::RevokeMembership {
                principal: id(&old["principal"]["principal_id"]),
                expected: revision(&old["revision"]),
            });
        }
    }
    for (old, new) in before["grants"]
        .as_array()
        .unwrap()
        .iter()
        .zip(after["grants"].as_array().unwrap())
    {
        if old != new {
            return Some(PermissionChange::RevokeGrant {
                grant: id(&old["grant_id"]),
                expected: revision(&old["revision"]),
            });
        }
    }
    for (old, new) in before["policies"]
        .as_array()
        .unwrap()
        .iter()
        .zip(after["policies"].as_array().unwrap())
    {
        if old != new {
            return Some(PermissionChange::RestrictPolicy {
                expected: revision(&old["epoch"]),
                purposes: strings(&new["purposes"]),
            });
        }
    }
    for (old, new) in before["resources"]
        .as_array()
        .unwrap()
        .iter()
        .zip(after["resources"].as_array().unwrap())
    {
        if old != new {
            return Some(PermissionChange::AdvanceResource {
                expected: revision(&old["control_revision"]),
            });
        }
    }
    None
}

pub fn recipe(realization: &Value, case_id: &str) -> Value {
    let row = realization["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"] == case_id)
        .unwrap();
    json!({"controlled_constraint_faults":row["controlled_constraint_faults"],
        "authority_owner":realization["authority_owner_mapping"]["workspace"]})
}

/// Select only actual trusted preparation inputs; no oracle record leaves this loader.
pub fn probe_inputs() -> Result<Value> {
    let (authority, _, _) = fixtures()?;
    let profile = input_configuration()?;
    let identities = &authority["cases"]
        .as_array()
        .ok_or("missing authority cases")?
        .iter()
        .find(|c| c["group"] == "authority_races")
        .ok_or("missing actual identity input")?["input"]["preallocated_test_identities"][0];
    Ok(
        json!({"clock":profile["authority_races"]["initial_snapshot_sample"],
        "effect_identities":identities,"reader_failure":profile["reader_failure"]}),
    )
}

const STATE_QUERIES: [(&str, &str); 11] = [
    (
        "organizations",
        "SELECT organization_id::text AS key,jsonb_build_object('organization_id',organization_id,'name',name,'state',state) AS value FROM arboresce_authority.organizations",
    ),
    (
        "domains",
        "SELECT organization_id::text||domain_id::text AS key,jsonb_build_object('organization_id',organization_id,'domain_id',domain_id,'name',name,'state',state) AS value FROM arboresce_authority.domains",
    ),
    (
        "principals",
        "SELECT principal_id::text AS key,jsonb_build_object('principal_id',principal_id,'kind',kind,'identity_profile',jsonb_build_object('profile',identity_profile,'version',identity_version,'public_key',encode(public_key,'hex'),'standard_revisions',jsonb_build_object('nip01',jsonb_build_object('standard',identity_standard,'revision',identity_standard_revision))),'state',state,'control_revision',jsonb_build_object('kind','control','value',control_revision::text),'created_at',to_char(created_at AT TIME ZONE 'UTC','YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),'revoked_at',to_char(revoked_at AT TIME ZONE 'UTC','YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"')) AS value FROM arboresce_authority.principals",
    ),
    (
        "memberships",
        "SELECT organization_id::text||membership_id::text AS key,jsonb_build_object('membership_id',membership_id,'principal',jsonb_build_object('principal_id',principal_id,'kind',principal_kind),'workspace',jsonb_build_object('organization_id',organization_id,'domain_id',domain_id),'roles',roles,'policy',policy,'revision',jsonb_build_object('kind','control','value',control_revision::text),'state',state) AS value FROM arboresce_authority.memberships",
    ),
    (
        "policies",
        "SELECT organization_id::text||domain_id::text AS key,jsonb_build_object('workspace',jsonb_build_object('organization_id',organization_id,'domain_id',domain_id),'policy',policy,'epoch',jsonb_build_object('kind','control','value',epoch::text),'state',state,'read_roles',read_roles,'direct_review_roles',direct_review_roles,'purposes',purposes,'grant_root_actions',grant_root_actions,'source_policy_ids',source_policy_ids) AS value FROM arboresce_authority.workspace_policies",
    ),
    (
        "source_policies",
        "SELECT policy_id AS key,jsonb_build_object('policy_id',policy_id,'state',state,'expires_at',to_char(expires_at AT TIME ZONE 'UTC','YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'),'permitted_principals',permitted_principals,'permitted_purposes',permitted_purposes) AS value FROM arboresce_authority.source_policies",
    ),
    (
        "resources",
        "SELECT organization_id::text||domain_id::text||resource_kind||resource_id::text AS key,jsonb_build_object('workspace',jsonb_build_object('organization_id',organization_id,'domain_id',domain_id),'kind',resource_kind,'id',resource_id,'name',name,'control_revision',jsonb_build_object('kind','control','value',control_revision::text),'state',state) AS value FROM arboresce_authority.resources",
    ),
    (
        "party_aliases",
        "SELECT organization_id::text||domain_id::text||party_id::text AS key,jsonb_build_object('workspace',jsonb_build_object('organization_id',organization_id,'domain_id',domain_id),'party_id',party_id,'alias',alias,'principal_id',principal_id) AS value FROM arboresce_authority.party_aliases",
    ),
    (
        "grants",
        "SELECT grant_id::text AS key,to_jsonb(g)-'organization_id'-'domain_id'-'lock_guard' AS value FROM arboresce_authority.grants g",
    ),
    (
        "immutable_chains",
        "SELECT chain_id::text AS key,jsonb_build_object('chain_id',chain_id,'scope_interpretation',scope_interpretation) AS value FROM arboresce_authority.immutable_chains",
    ),
    (
        "immutable_members",
        "SELECT chain_id::text||lpad(ordinal::text,2,'0') AS key,to_jsonb(m)-'organization_id'-'domain_id'-'lock_guard' AS value FROM arboresce_authority.immutable_members m",
    ),
];

fn grant_output(v: &Value) -> Value {
    let revision =
        json!({"kind":"control","value":v["control_revision"].as_i64().unwrap().to_string()});
    let instant = |name: &str| {
        v[name]
            .as_str()
            .map(|s| s.replace(' ', "T").replace("+00:00", "Z"))
    };
    // JSONB timestamptz serialization omits fractional zeroes; parse its actual
    // value and retain millisecond precision, which the normal schema enforces.
    let time = |name: &str| {
        instant(name).map(|s| {
            let s = if s.len() == 20 {
                s.replace('Z', ".000Z")
            } else {
                s
            };
            // Preserve a controlled malformed SQL value in the observation.
            // Only the production decoder decides whether it is admissible.
            s.parse::<UtcTimestamp>()
                .map(|v| v.to_string())
                .unwrap_or(s)
        })
    };
    json!({"grant_id":v["grant_id"],"issuer":{"principal_id":v["issuer_id"],"kind":v["issuer_kind"]},
        "subject":{"principal_id":v["subject_id"],"kind":v["subject_kind"]},"parent_grant_id":v["parent_grant_id"],
        "scope":{"organization_id":v["scope_organization_id"],"domain_id":v["scope_domain_id"],"resource_kind":v["scope_kind"],"resource_id":v["scope_resource_id"]},
        "actions":v["actions"],"purposes":v["purposes"],"delegable":v["delegable"],"issued_at":time("issued_at"),"expires_at":time("expires_at"),
        "policy":v["policy"],"revision":revision,"state":v["state"],"revoked_at":time("revoked_at")})
}

/// Complete database observations; names, aliases and seals come from SQL rows.
/// The supplied timestamp is the actual trusted clock used by this observation.
pub async fn snapshot(
    service: &Service,
    clock: &mut SuppliedClock,
    financial_null: bool,
) -> Result<Value> {
    let now = clock.now()?;
    service.with_admin(false, async |connection| {
    let mut result = json!({"now":now.to_string()});
    for (name, projection) in STATE_QUERIES {
        // Both framing and projection are compiled constants, with no fixture SQL.
        let sql = format!(
            "SELECT coalesce(jsonb_agg(value ORDER BY key),'[]'::jsonb)::text FROM ({projection}) rows"
        );
        let raw: String = sqlx::query_scalar(sqlx::AssertSqlSafe(sql))
            .fetch_one(&mut *connection)
            .await?;
        if raw.len() > 131072 {
            return Err("canonical SQL observation exceeds bound".into());
        }
        result[name] = serde_json::from_str(&raw)?;
    }
    result["grants"] = Value::Array(
        result["grants"]
            .as_array()
            .unwrap()
            .iter()
            .map(grant_output)
            .collect(),
    );
    let members = result
        .as_object_mut()
        .unwrap()
        .remove("immutable_members")
        .unwrap();
    for chain in result["immutable_chains"].as_array_mut().unwrap() {
        chain["members"] = Value::Array(
            members
                .as_array()
                .unwrap()
                .iter()
                .filter(|m| m["chain_id"] == chain["chain_id"])
                .map(grant_output)
                .collect(),
        );
    }
    if financial_null {
        let families:Vec<String>=sqlx::query_scalar("SELECT table_name FROM information_schema.tables WHERE table_schema='arboresce_authority' ORDER BY table_name")
            .fetch_all(&mut *connection).await?;
        if families
            .iter()
            .any(|s| s.contains("financial") || s.contains("review_event"))
        {
            return Err("unexpected financial effect relation".into());
        }
        result["financial_review_event"] = Value::Null;
    }
    Ok(result)
    }).await
}

/// Compare unordered relations by their actual complete keys; ordered sealed
/// members and every inner policy/action/source array remain unchanged.
pub fn relation_order(v: &Value) -> Value {
    let mut value = v.clone();
    for (name, key) in [
        ("organizations", "organization_id"),
        ("domains", "domain_id"),
        ("principals", "principal_id"),
        ("memberships", "membership_id"),
        ("policies", "workspace"),
        ("source_policies", "policy_id"),
        ("resources", "id"),
        ("party_aliases", "party_id"),
        ("grants", "grant_id"),
        ("immutable_chains", "chain_id"),
    ] {
        value[name]
            .as_array_mut()
            .unwrap()
            .sort_by_key(|row| row[key].to_string());
    }
    value
}

pub async fn census(service: &Service) -> Result<Value> {
    service.json(true, "SELECT jsonb_build_object('writers',count(*) FILTER (WHERE datname='arboresce_migration_harness' AND usename IN ('harness_writer','harness_authority_admission','harness_authority_permission')),'controls',count(*) FILTER (WHERE usename IN ('harness_admin','harness_migrator')),'backends',coalesce(jsonb_agg(jsonb_build_object('pid',pid,'role',usename,'state',state) ORDER BY pid) FILTER (WHERE datname='arboresce_migration_harness'),'[]'::jsonb))::text FROM pg_stat_activity WHERE backend_type='client backend'", &[], 131072).await
}

pub async fn accounting(service: &Service) -> Result<Value> {
    accounting_with_versions(service, false).await
}

/// Only the four controlled ledger shape faults use actual JSON scalar versions.
/// The ordinary typed query and every other count remain the same.
pub async fn accounting_for_history_shape(service: &Service) -> Result<Value> {
    accounting_with_versions(service, true).await
}

async fn accounting_with_versions(service: &Service, controlled_shape: bool) -> Result<Value> {
    with_connection(service.admin(false).await?, async |connection| {
    let mut counts = json!({});
    for table in crate::implementation::authority::catalogue_policy::TABLES {
        // The exact fixed source inventory chooses syntax, never caller strings.
        let sql = format!("SELECT count(*) FROM arboresce_authority.{table}");
        let n: i64 = sqlx::query_scalar(sqlx::AssertSqlSafe(sql))
            .fetch_one(&mut *connection)
            .await?;
        counts[table] = json!(n);
    }
    let receipts: i64 = sqlx::query_scalar("SELECT count(*) FROM public._sqlx_migrations")
        .fetch_one(&mut *connection)
        .await?;
    let total: i64 = counts
        .as_object()
        .unwrap()
        .values()
        .map(|n| n.as_i64().unwrap())
        .sum();
    let versions = if controlled_shape {
        let raw: String = sqlx::query_scalar("SELECT coalesce(jsonb_agg(to_jsonb(v.version) ORDER BY v.version),'[]'::jsonb)::text FROM public._sqlx_migrations v")
            .fetch_one(&mut *connection).await?;
        serde_json::from_str::<Value>(&raw)?
    } else {
        let versions: Vec<i64> =
            sqlx::query_scalar("SELECT version FROM public._sqlx_migrations ORDER BY version")
                .fetch_all(&mut *connection)
                .await?;
        json!(versions)
    };
    let extra: i64 = sqlx::query_scalar("SELECT count(*) FROM migration_harness.records")
        .fetch_one(&mut *connection)
        .await?;
    let epoch: i64 = sqlx::query_scalar(
        "SELECT epoch FROM arboresce_authority.namespace_guard WHERE singleton=1",
    )
    .fetch_one(&mut *connection)
    .await?;
    Ok(
        json!({"canonical_rows":counts,"canonical_total":total,"migration_receipts":receipts,"migration_versions":versions,"other_user_rows":extra,"physical_user_rows":total+receipts+extra,"namespace_epoch":epoch}),
    )
    }).await
}

pub async fn markers(service: &Service) -> Result<Value> {
    service.json(false, "SELECT coalesce(jsonb_agg(to_jsonb(m) ORDER BY marker_id),'[]'::jsonb)::text FROM arboresce_authority.qualification_markers m", &[], 131072).await
}

pub async fn current(service: &Service, input: &Value, recipe: &Value) -> Result<Value> {
    let seed_observation =
        retain_observation(&seed(service, &input["canonical_snapshot"], recipe).await?)?;
    let request = request(&input["request"]);
    let config = configuration();
    let sessions = service.harness.authority_sessions();
    let mut session = sessions.admission().await?;
    let backend = session.backend_id().await?;
    let mut clock = SuppliedClock::new(&input["canonical_snapshot"]["now"]);
    let initial = session.observe_current(&request, &config, &mut clock).await;
    let final_input = &input["final_effect_snapshot"];
    let separate_final = !final_input.is_null();
    if separate_final {
        initial?;
        let mut permission = sessions.permission().await?;
        clock = SuppliedClock::new(&final_input["now"]);
        if let Some(change) = change(&input["canonical_snapshot"], final_input) {
            permission
                .fence(&request, change, &mut clock)
                .await?
                .commit()
                .await?;
        } else {
            // The unchanged final-fence input still observes a separate,
            // positively rolled-back permission transaction on the same DB.
            let control = initial
                .as_ref()
                .map(|v| v.resource_control())
                .map_err(|e| *e)?;
            permission
                .fence(
                    &request,
                    PermissionChange::AdvanceResource { expected: control },
                    &mut clock,
                )
                .await?
                .rollback()
                .await?;
        }
        permission.close().await?;
    }
    let final_observation = match session.fence(&request, &config, &mut clock).await {
        Ok(fence) => {
            let actual = observation(fence.observation());
            fence.rollback().await?;
            Ok(actual)
        }
        Err(failure) => Err(failure),
    };
    let same_backend = session.backend_id().await? == backend;
    session.close().await?;
    let (decision, problem, visible, executor, chain) = match final_observation {
        Ok(o) => (
            "admit",
            Value::Null,
            json!([o["resource_id"]]),
            o["executor_principal_id"].clone(),
            o["selected_chain_id"].clone(),
        ),
        Err(failure) => (
            "deny",
            problem(failure),
            json!([]),
            Value::Null,
            Value::Null,
        ),
    };
    let adapter_clock_calls = clock.calls;
    let state = snapshot(
        service,
        &mut clock,
        input["canonical_snapshot"]
            .get("financial_review_event")
            .is_some(),
    )
    .await?;
    Ok(
        json!({"decision":{"decision":decision,"visible_resource_ids":visible,"effects":[],
        "actual_executor_principal_id":executor,"selected_chain_id":chain,"human_assurance_issued":false,
        "problem_expectation":problem,"disclosure_expectation":{"foreign_ids":[],"foreign_counts":null}},
        "canonical_state_after":state,"markers":markers(service).await?,"accounting":accounting(service).await?,
        "same_backend":same_backend,"clock_calls":adapter_clock_calls,"census":census(service).await?,"seed_observation":seed_observation}),
    )
}

pub fn observed_resource<'a>(state: &'a Value, resource: &Value) -> Result<Option<&'a Value>> {
    if !resource["id"].is_string()
        || !resource["kind"].is_string()
        || !resource["workspace"]["organization_id"].is_string()
        || !resource["workspace"]["domain_id"].is_string()
    {
        return Err("incomplete observed resource key".into());
    }
    let mut matches = state["resources"]
        .as_array()
        .ok_or("invalid resource readback")?
        .iter()
        .filter(|row| {
            row["id"] == resource["id"]
                && row["kind"] == resource["kind"]
                && row["workspace"] == resource["workspace"]
        });
    let first = matches.next();
    if matches.next().is_some() {
        return Err("ambiguous observed resource key".into());
    }
    Ok(first)
}

pub async fn held_effect(
    service: &Service,
    input: &Value,
    base: &Value,
    recipe: &Value,
) -> Result<Value> {
    let seed_observation =
        retain_observation(&seed(service, &base["canonical_snapshot"], recipe).await?)?;
    let mut raw_request = base["request"].clone();
    if let Some(patch) = input["patch"].as_object() {
        for (key, value) in patch {
            raw_request[key] = value.clone();
        }
    }
    let request = request(&raw_request);
    let config = configuration();
    let mut before_clock = SuppliedClock::new(&base["canonical_snapshot"]["now"]);
    let before = snapshot(service, &mut before_clock, false).await?;
    let before_accounting = accounting(service).await?;
    let sessions = service.harness.authority_sessions();
    let mut session = sessions.admission().await?;
    let backend = session.backend_id().await?;
    let mut clock = SuppliedClock::sequence(&input["trusted_clock_samples"]);
    let fence = session.fence(&request, &config, &mut clock).await?;
    let initial = observation(fence.observation());
    let effect = match text(&input["effect"]) {
        "isolated_synthetic_marker" => {
            fence
                .commit_marker(id(&input["marker_id"]), id(&input["observation_id"]))
                .await
        }
        "qualified_synthetic_cas" => {
            fence
                .compare_and_marker(id(&input["marker_id"]), id(&input["observation_id"]))
                .await
        }
        _ => return Err("unadmitted held-fence effect".into()),
    };
    let adapter_calls = clock.calls;
    let same_backend = session.backend_id().await? == backend;
    let idle = service
        .with_admin(true, async |connection| {
            Ok(sqlx::query_scalar::<_, bool>(
                "SELECT state='idle' AND xact_start IS NULL FROM pg_stat_activity WHERE pid=$1",
            )
            .bind(backend)
            .fetch_one(connection)
            .await?)
        })
        .await?;
    if !idle {
        return Err("final effect has no actual positive terminal".into());
    }
    session.close().await?;
    let after = snapshot(service, &mut clock, false).await?;
    let after_accounting = accounting(service).await?;
    let markers = markers(service).await?;
    let resource = &raw_request["resource"];
    let control_of = |state: &Value| -> Result<Value> {
        Ok(observed_resource(state, resource)?
            .ok_or("held resource readback missing")?["control_revision"]["value"]
            .clone())
    };
    let before_control = control_of(&before)?;
    let after_control = control_of(&after)?;
    let mut normalized_before = before.clone();
    normalized_before["now"] = after["now"].clone();
    let authority_changed = relation_order(&normalized_before) != relation_order(&after)
        || !markers.as_array().unwrap().is_empty()
        || before_accounting["namespace_epoch"] != after_accounting["namespace_epoch"];
    for row in normalized_before["resources"].as_array_mut().unwrap() {
        if row["id"] == resource["id"]
            && row["kind"] == resource["kind"]
            && row["workspace"] == resource["workspace"]
        {
            row["control_revision"]["value"] = after_control.clone();
        }
    }
    let unchanged = relation_order(&normalized_before) == relation_order(&after)
        && before_accounting["namespace_epoch"] == after_accounting["namespace_epoch"];
    let (failure, semantic, decision, terminal) = match effect {
        Ok(()) => (Value::Null, Value::Null, "admit", "commit"),
        Err(failure) => (
            json!(failure.to_string()),
            problem(failure),
            "deny",
            "rollback",
        ),
    };
    let observation = if failure.is_null() {
        json!({"observation_id":markers[0]["observation_id"],"marker_id":markers[0]["marker_id"],
        "transaction_outcome":"committed","executor_principal_id":markers[0]["executor_principal_id"],"selected_chain_id":markers[0]["selected_chain_id"]})
    } else {
        Value::Null
    };
    Ok(
        json!({"initial_decision":"admit","initial_observation":initial,"failure":failure,"semantic_problem":semantic,"decision":decision,
        "marker_rows_added":markers.as_array().unwrap().len(),"resource_control_before":before_control,"resource_control_after":after_control,
        "trusted_clock_calls":adapter_calls,"positive_terminal":terminal,"same_physical_backend":same_backend,"authority_changed":authority_changed,
        "unrelated_canonical_rows_unchanged":unchanged,"marker_row":markers.as_array().unwrap().first(),"observation":observation,
        "canonical_command_receipts":[],"product_effects":[],"human_assurance_issued":false,
        "before":before,"after":after,"accounting_before":before_accounting,"accounting_after":after_accounting,"census":census(service).await?,"seed_observation":seed_observation}),
    )
}

pub fn error_observation(input: &Value) -> Value {
    let failure = match text(&input["variant"]) {
        "InvalidRequest" => AuthorityFailure::InvalidRequest,
        "Unauthenticated" => AuthorityFailure::Unauthenticated,
        "NotFound" => AuthorityFailure::NotFound,
        "Forbidden" => AuthorityFailure::Forbidden,
        "Conflict" => AuthorityFailure::Conflict,
        "InvalidStoredData" => AuthorityFailure::InvalidStoredData,
        "Unavailable" => AuthorityFailure::Unavailable,
        "UnsafeRole" => AuthorityFailure::UnsafeRole,
        "RetainedContext" => AuthorityFailure::RetainedContext,
        "UnknownCompletion" => AuthorityFailure::UnknownCompletion,
        _ => panic!("unadmitted SQL authority failure"),
    };
    let boxed: Box<dyn std::error::Error> = Box::new(failure);
    json!({"display":boxed.to_string(),"debug":format!("{failure:?}"),"error_source":boxed.source().map(|e|e.to_string()),
        "boxed_variant_preserved":boxed.downcast_ref::<AuthorityFailure>()==Some(&failure)})
}
