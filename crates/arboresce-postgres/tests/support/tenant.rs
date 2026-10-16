//! Measured preparation and observations for the isolated tenant foundation.

use std::{
    collections::BTreeSet,
    path::Path,
    time::{Duration, Instant},
};

use arboresce_application::policy::{ReferenceKind, ScopedReference, TenantContext, Workspace};
use arboresce_xtask::{collection_prepared, inventory};
use serde_json::{Value, json};
use sqlx::{Connection, PgConnection, Row};

use crate::{
    implementation::{TenantFailure, TenantSession, TenantTransaction, tenant::validate_role},
    relay,
    support::Service,
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

struct FailedFormatter;

impl std::fmt::Write for FailedFormatter {
    fn write_str(&mut self, _value: &str) -> std::fmt::Result {
        Err(std::fmt::Error)
    }
}

/// Observe the real diagnostic traits, preserving the selected typed error.
pub fn failure(input: &Value) -> Result<Value> {
    let error = match input["variant"].as_str() {
        Some("Unavailable") => TenantFailure::Unavailable,
        Some("UnsafeRole") => TenantFailure::UnsafeRole,
        Some("RetainedContext") => TenantFailure::RetainedContext,
        Some("NotFound") => TenantFailure::NotFound,
        Some("InvalidStoredData") => TenantFailure::InvalidStoredData,
        Some("UnknownCompletion") => TenantFailure::UnknownCompletion,
        _ => return Err("unsupported tenant failure variant".into()),
    };
    let boxed: Box<dyn std::error::Error> = error.into();
    let formatting = std::fmt::write(&mut FailedFormatter, format_args!("{error}"));
    Ok(json!({
        "display": boxed.to_string(),
        "source_is_none": boxed.source().is_none(),
        "boxed_variant_preserved": boxed.downcast_ref::<TenantFailure>() == Some(&error),
        "formatter_failure_propagated": formatting.is_err()
    }))
}

fn check_depth(value: &Value, remaining: usize) -> Result<()> {
    if remaining == 0 {
        return Err("tenant fixture depth exceeded".into());
    }
    match value {
        Value::Array(rows) => {
            for row in rows {
                check_depth(row, remaining - 1)?;
            }
        }
        Value::Object(fields) => {
            for row in fields.values() {
                check_depth(row, remaining - 1)?;
            }
        }
        _ => {}
    }
    Ok(())
}

pub fn fixtures() -> Result<(Value, Value)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let directory = root.join("tests/security/fixtures");
    fixtures_at(&root, &directory)
}

fn fixtures_at(root: &Path, directory: &Path) -> Result<(Value, Value)> {
    let (additional, _) =
        collection_prepared::read_json(&directory.join("tenant-foundation-v1/cases.json"), 131072)?;
    let (authority, authority_hash) =
        collection_prepared::read_json(&directory.join("authority-v1/cases.json"), 8388608)?;
    let (manifest, _) =
        collection_prepared::read_json(&directory.join("authority-v1/sources.json"), 65536)?;
    check_depth(&additional, 16)?;
    check_depth(&authority, 32)?;
    if additional["fixture_version"] != "tenant-foundation-v1"
        || authority["fixture_version"] != "authority-v1"
        || manifest["case_file"]["sha256"] != authority_hash
    {
        return Err("tenant fixture identity mismatch".into());
    }
    for (path, expected) in additional["sources"]
        .as_object()
        .ok_or("missing source pins")?
    {
        inventory::portable_path(Path::new(path))?;
        let observed = collection_prepared::fingerprint(
            &root.join(path),
            8388608,
            Instant::now() + Duration::from_secs(10),
        )?;
        if observed["sha256"] != *expected {
            return Err("tenant source pin mismatch".into());
        }
    }
    for (name, count) in [
        ("api_cases", 21),
        ("constraint_cases", 31),
        ("migration_cases", 4),
        ("canonical_rls_cases", 12),
    ] {
        let rows = additional[name].as_array().ok_or("missing tenant cases")?;
        let ids: BTreeSet<_> = rows.iter().filter_map(|row| row["id"].as_str()).collect();
        if rows.len() != count || ids.len() != count {
            return Err("incomplete tenant cases".into());
        }
    }
    let rows: Vec<_> = authority["cases"]
        .as_array()
        .ok_or("missing authority cases")?
        .iter()
        .filter(|row| row["group"] == "tenant_relations")
        .cloned()
        .collect();
    let ids: BTreeSet<_> = rows
        .iter()
        .map(|row| row["id"].clone().to_string())
        .collect();
    let required: BTreeSet<_> = manifest["required_groups"]["tenant_relations"]
        .as_array()
        .ok_or("missing required tenant group")?
        .iter()
        .map(Value::to_string)
        .collect();
    if rows.len() != 25 || ids != required {
        return Err("incomplete tenant group".into());
    }
    Ok((additional, Value::Array(rows)))
}

fn prepared_files(paths: &[std::path::PathBuf; 3]) -> Result<Value> {
    let mut presence = Vec::new();
    let mut files = Vec::new();
    for (role, path) in ["additional", "authority", "manifest"]
        .into_iter()
        .zip(paths)
    {
        let present = path.try_exists()?;
        presence.push(present);
        if present {
            let mut identity = collection_prepared::fingerprint(
                path,
                8388608,
                Instant::now() + Duration::from_secs(10),
            )?;
            identity["role"] = json!(role);
            files.push(identity);
        }
    }
    Ok(json!({"presence":presence,"files":files}))
}

/// Prepare actual owned files; expected decisions never enter this function.
pub fn preparation(input: &Value) -> Result<Value> {
    use std::os::unix::fs::MetadataExt;
    if input["operation"] == "reference" {
        let outcome = reference(&input["reference"]);
        return Ok(
            json!({"accepted":outcome.is_ok(),"error":outcome.err().map(|error|error.to_string())}),
        );
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let base = root.join("tests/security/fixtures");
    let (mut additional, _) =
        collection_prepared::read_json(&base.join("tenant-foundation-v1/cases.json"), 131072)?;
    let (mut authority, _) =
        collection_prepared::read_json(&base.join("authority-v1/cases.json"), 8388608)?;
    let (mut manifest, _) =
        collection_prepared::read_json(&base.join("authority-v1/sources.json"), 65536)?;
    let recipe = input["recipe"]
        .as_str()
        .ok_or("missing preparation recipe")?;
    match recipe {
        "valid"
        | "manifest-hash"
        | "additional-file-missing"
        | "authority-file-missing"
        | "manifest-file-missing"
        | "additional-json-malformed"
        | "authority-json-malformed"
        | "manifest-json-malformed"
        | "additional-json-duplicate-key"
        | "authority-json-duplicate-key"
        | "additional-byte-limit"
        | "additional-byte-over" => {}
        "depth-limit" | "depth-over" | "authority-depth-limit" | "authority-depth-over" => {
            let mut nested = json!(0);
            let count = match recipe {
                "depth-limit" => 14,
                "depth-over" => 15,
                "authority-depth-limit" => 30,
                _ => 31,
            };
            for _ in 0..count {
                nested = json!([nested]);
            }
            if recipe.starts_with("authority-") {
                authority["depth_probe"] = nested;
            } else {
                additional["depth_probe"] = nested;
            }
        }
        "fixture-version" => additional["fixture_version"] = json!("unknown"),
        "authority-version" => authority["fixture_version"] = json!("unknown"),
        "parent-source-pin" | "absent-source-pin" => {
            let path = if recipe == "parent-source-pin" {
                "../unavailable-tenant-source.txt"
            } else {
                "tests/security/fixtures/tenant-foundation-v1/unavailable-source-pin.txt"
            };
            additional["sources"].as_object_mut().unwrap().insert(
                path.into(),
                json!("0000000000000000000000000000000000000000000000000000000000000000"),
            );
        }
        "missing-pins" => {
            additional.as_object_mut().unwrap().remove("sources");
        }
        "wrong-pin" => {
            let pin = additional["sources"]
                .as_object_mut()
                .unwrap()
                .values_mut()
                .next()
                .unwrap();
            *pin = json!("0000000000000000000000000000000000000000000000000000000000000000");
        }
        "missing-additional-group" => {
            additional.as_object_mut().unwrap().remove("api_cases");
        }
        "missing-additional-case" => {
            additional["api_cases"].as_array_mut().unwrap().pop();
        }
        "duplicate-additional-id" => {
            additional["api_cases"][1]["id"] = additional["api_cases"][0]["id"].clone()
        }
        "missing-authority-cases" => {
            authority.as_object_mut().unwrap().remove("cases");
        }
        "missing-required-group" => {
            manifest["required_groups"]
                .as_object_mut()
                .unwrap()
                .remove("tenant_relations");
        }
        "missing-tenant-case" | "duplicate-tenant-id" => {
            let rows: Vec<_> = authority["cases"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .filter(|row| row["group"] == "tenant_relations")
                .collect();
            let mut rows = rows.into_iter();
            let first = rows.next().unwrap();
            if recipe == "missing-tenant-case" {
                first["group"] = json!("preparation_unselected");
            } else {
                rows.next().unwrap()["id"] = first["id"].clone();
            }
        }
        _ => return Err("unsupported preparation recipe".into()),
    }
    let mut additional_bytes = serde_json::to_vec(&additional)?;
    let mut authority_bytes = serde_json::to_vec(&authority)?;
    match recipe {
        "additional-json-malformed" => additional_bytes = b"{".to_vec(),
        "authority-json-malformed" => authority_bytes = b"{".to_vec(),
        "additional-json-duplicate-key" => {
            drop(
                additional_bytes.splice(
                    1..1,
                    b"\"fixture_version\":\"tenant-foundation-v1\","
                        .iter()
                        .copied(),
                ),
            );
        }
        "authority-json-duplicate-key" => {
            drop(authority_bytes.splice(
                1..1,
                b"\"fixture_version\":\"authority-v1\",".iter().copied(),
            ));
        }
        "additional-byte-limit" | "additional-byte-over" => {
            let count = if recipe == "additional-byte-limit" {
                131072
            } else {
                131073
            };
            if additional_bytes.len() > count {
                return Err("complete additional fixture exceeds padding bound".into());
            }
            additional_bytes.resize(count, b' ');
        }
        _ => {}
    }
    if recipe == "authority-file-missing" {
        manifest["case_file"]["sha256"] = Value::Null;
        manifest["case_file"]["bytes"] = Value::Null;
    } else {
        manifest["case_file"]["sha256"] = json!(inventory::sha256(&authority_bytes));
        manifest["case_file"]["bytes"] = json!(authority_bytes.len());
    }
    if recipe == "manifest-hash" {
        manifest["case_file"]["sha256"] =
            json!("0000000000000000000000000000000000000000000000000000000000000000");
    }
    let absent_pin =
        root.join("tests/security/fixtures/tenant-foundation-v1/unavailable-source-pin.txt");
    let pin_absent_before = if recipe == "absent-source-pin" {
        let absent = !absent_pin.try_exists()?;
        if !absent {
            return Err("declared absent source pin is present".into());
        }
        Some(absent)
    } else {
        None
    };
    let directory = relay::owned_directory(
        "arboresce-tenant-fixture",
        &std::sync::atomic::AtomicU64::new(0),
    )?;
    let before = std::fs::symlink_metadata(&directory)?;
    if !before.is_dir() || before.mode() & 0o777 != 0o700 {
        return Err("preparation directory is not private".into());
    }
    let paths = [
        directory.join("tenant-foundation-v1/cases.json"),
        directory.join("authority-v1/cases.json"),
        directory.join("authority-v1/sources.json"),
    ];
    let observed: Result<Value> = (|| {
        use std::io::Write;
        std::fs::create_dir(directory.join("tenant-foundation-v1"))?;
        std::fs::create_dir(directory.join("authority-v1"))?;
        let manifest_bytes = if recipe == "manifest-json-malformed" {
            b"{".to_vec()
        } else {
            serde_json::to_vec(&manifest)?
        };
        let bytes = [additional_bytes, authority_bytes, manifest_bytes];
        let missing = match recipe {
            "additional-file-missing" => Some(0),
            "authority-file-missing" => Some(1),
            "manifest-file-missing" => Some(2),
            _ => None,
        };
        for (index, (path, bytes)) in paths.iter().zip(bytes).enumerate() {
            if missing == Some(index) {
                continue;
            }
            std::fs::File::options()
                .write(true)
                .create_new(true)
                .open(path)?
                .write_all(&bytes)?;
        }
        let files_before = prepared_files(&paths)?;
        let outcome = fixtures_at(&root, &directory);
        let files_after = prepared_files(&paths)?;
        let pin_absent_after = if recipe == "absent-source-pin" {
            Some(!absent_pin.try_exists()?)
        } else {
            None
        };
        let files = files_before["files"].as_array().unwrap();
        let additional_size = files
            .iter()
            .find(|file| file["role"] == "additional")
            .map(|file| file["size"].clone());
        Ok(
            json!({"accepted":outcome.is_ok(),"error":outcome.err().map(|error|error.to_string()),
                "prepared_files":files,"prepared_file_count":files.len(),
                "prepared_file_presence":files_before["presence"],
                "prepared_file_presence_after_reader":files_after["presence"],
                "prepared_files_after_reader":files_after["files"],
                "additional_file_bytes":additional_size,
                "source_pin_absent_before":pin_absent_before,"source_pin_absent_after":pin_absent_after}),
        )
    })();
    let after = std::fs::symlink_metadata(&directory)?;
    if before.dev() != after.dev() || before.ino() != after.ino() || before.mode() != after.mode() {
        return Err("preparation directory identity changed".into());
    }
    // Remove only the three exact files and two directories owned by this call.
    for path in &paths {
        if path.try_exists()? {
            std::fs::remove_file(path)?;
        }
    }
    for name in ["tenant-foundation-v1", "authority-v1"] {
        let path = directory.join(name);
        if path.try_exists()? {
            std::fs::remove_dir(path)?;
        }
    }
    std::fs::remove_dir(&directory)?;
    let mut observed = observed?;
    observed["directory_removed"] = json!(!directory.try_exists()?);
    observed["directory_mode"] = json!(before.mode() & 0o777);
    observed["directory_identity_preserved"] = json!(true);
    Ok(observed)
}

pub fn context(value: &Value) -> TenantContext {
    TenantContext::new(
        workspace(value),
        value["principal_id"].as_str().unwrap().parse().unwrap(),
    )
}

pub fn workspace(value: &Value) -> Workspace {
    Workspace::new(
        value["organization_id"].as_str().unwrap().parse().unwrap(),
        value["domain_id"].as_str().unwrap().parse().unwrap(),
    )
}

pub fn reference(value: &Value) -> Result<ScopedReference> {
    let kind = match value["kind"].as_str() {
        Some("project") => ReferenceKind::Project,
        Some("party") => ReferenceKind::Party,
        Some("capture") => ReferenceKind::Capture,
        Some("expense") => ReferenceKind::Expense,
        Some("grant") => ReferenceKind::Grant,
        Some("membership") => ReferenceKind::Membership,
        _ => return Err("unknown reference kind".into()),
    };
    Ok(ScopedReference::new(
        workspace(&value["workspace"]),
        kind,
        value["id"]
            .as_str()
            .ok_or("missing reference id")?
            .parse()?,
    ))
}

/// Only a fixed table vocabulary and parameterized input reach SQL.
pub async fn insert(
    connection: &mut PgConnection,
    table: &str,
    row: &Value,
) -> std::result::Result<u64, sqlx::Error> {
    let statement = match table {
        "organizations" => sqlx::query("INSERT INTO arboresce_authority.organizations VALUES ($1::uuid,$2)")
            .bind(row["organization_id"].as_str()).bind(row["state"].as_str()),
        "principals" => sqlx::query("INSERT INTO arboresce_authority.principals VALUES ($1::uuid,$2,$3,decode($4,'hex'),$5)")
            .bind(row["principal_id"].as_str()).bind(row["identity_profile"].as_str()).bind(row["identity_version"].as_str())
            .bind(row["public_key"].as_str()).bind(row["state"].as_str()),
        "domains" => sqlx::query("INSERT INTO arboresce_authority.domains VALUES ($1::uuid,$2::uuid,$3)")
            .bind(row["organization_id"].as_str()).bind(row["domain_id"].as_str()).bind(row["state"].as_str()),
        "memberships" => sqlx::query("INSERT INTO arboresce_authority.memberships VALUES ($1::uuid,$2::uuid,$3::uuid,$4::uuid,$5)")
            .bind(row["organization_id"].as_str()).bind(row["principal_id"].as_str()).bind(row["domain_id"].as_str())
            .bind(row["membership_id"].as_str()).bind(row["state"].as_str()),
        _ => return Err(sqlx::Error::Protocol("unsupported qualification table".into())),
    };
    Ok(statement.execute(connection).await?.rows_affected())
}

pub async fn seed(service: &Service, rows: &Value) -> Result<()> {
    let mut connection = service.admin(false).await?;
    sqlx::raw_sql("TRUNCATE arboresce_authority.memberships, arboresce_authority.domains, arboresce_authority.principals, arboresce_authority.organizations")
        .execute(&mut connection).await?;
    for table in ["organizations", "principals", "domains", "memberships"] {
        for row in rows[table].as_array().ok_or("invalid canonical seed")? {
            insert(&mut connection, table, row).await?;
        }
    }
    Ok(())
}

pub async fn snapshot(service: &Service) -> Result<Value> {
    let mut connection = service.admin(false).await?;
    canonical_rows(&mut connection).await
}

pub async fn history(service: &Service) -> Result<Value> {
    let mut connection = service.admin(false).await?;
    let raw: String = sqlx::query_scalar("SELECT coalesce(jsonb_agg(to_jsonb(t)||jsonb_build_object('checksum',encode(checksum,'hex')) ORDER BY version),'[]')::text FROM public._sqlx_migrations t")
        .fetch_one(&mut connection).await?;
    Ok(serde_json::from_str(&raw)?)
}

/// Deliberately unfiltered: the caller's actual database role and RLS decide visibility.
pub async fn canonical_rows(connection: &mut PgConnection) -> Result<Value> {
    let raw: String = sqlx::query_scalar("SELECT jsonb_build_object(\
      'organizations',(SELECT coalesce(jsonb_agg(to_jsonb(t) ORDER BY organization_id),'[]') FROM arboresce_authority.organizations t),\
      'principals',(SELECT coalesce(jsonb_agg(to_jsonb(t)||jsonb_build_object('public_key',encode(public_key,'hex')) ORDER BY principal_id),'[]') FROM arboresce_authority.principals t),\
      'domains',(SELECT coalesce(jsonb_agg(to_jsonb(t) ORDER BY organization_id,domain_id),'[]') FROM arboresce_authority.domains t),\
      'memberships',(SELECT coalesce(jsonb_agg(to_jsonb(t) ORDER BY organization_id,principal_id),'[]') FROM arboresce_authority.memberships t))::text")
        .fetch_one(connection).await?;
    Ok(serde_json::from_str(&raw)?)
}

pub fn ordered_rows(value: &Value) -> Value {
    let mut copy = value.clone();
    for rows in copy.as_object_mut().unwrap().values_mut() {
        rows.as_array_mut().unwrap().sort_by_key(Value::to_string);
    }
    copy
}

pub async fn catalogue(service: &Service) -> Result<Value> {
    let mut connection = service.admin(false).await?;
    let raw: String = sqlx::query_scalar("SELECT jsonb_build_object(\
      'owner',pg_get_userbyid(n.nspowner),\
      'schema_create',has_schema_privilege('harness_writer',n.oid,'CREATE'),\
      'schema_usage',has_schema_privilege('harness_writer',n.oid,'USAGE'),\
      'public_schema_acl',(SELECT count(*) FROM aclexplode(n.nspacl) WHERE grantee=0),\
      'tables',(SELECT jsonb_object_agg(c.relname,jsonb_build_object(\
        'owner',pg_get_userbyid(c.relowner),'acl',c.relacl::text,'enabled',c.relrowsecurity,'forced',c.relforcerowsecurity,\
        'columns',(SELECT jsonb_agg(jsonb_build_object('name',a.attname,'type',format_type(a.atttypid,a.atttypmod),'not_null',a.attnotnull,'default',pg_get_expr(d.adbin,d.adrelid)) ORDER BY a.attnum) FROM pg_attribute a LEFT JOIN pg_attrdef d ON d.adrelid=a.attrelid AND d.adnum=a.attnum WHERE a.attrelid=c.oid AND a.attnum>0 AND NOT a.attisdropped),\
        'constraints',(SELECT jsonb_agg(jsonb_build_object('name',conname,'kind',contype,'columns',(SELECT jsonb_agg(a.attname ORDER BY s.ordinality) FROM unnest(conkey) WITH ORDINALITY s(num,ordinality) JOIN pg_attribute a ON a.attrelid=c.oid AND a.attnum=s.num),'target',CASE WHEN confrelid=0 THEN NULL ELSE confrelid::regclass::text END,'target_columns',(SELECT jsonb_agg(a.attname ORDER BY s.ordinality) FROM unnest(confkey) WITH ORDINALITY s(num,ordinality) JOIN pg_attribute a ON a.attrelid=confrelid AND a.attnum=s.num),'update',confupdtype,'delete',confdeltype,'match',confmatchtype,'deferrable',condeferrable,'validated',convalidated,'definition',pg_get_constraintdef(oid)) ORDER BY conname) FROM pg_constraint WHERE conrelid=c.oid),\
        'policies',(SELECT jsonb_agg(jsonb_build_object('name',polname,'command',polcmd,'roles',polroles::text,'permissive',polpermissive,'using',pg_get_expr(polqual,polrelid),'with_check',pg_get_expr(polwithcheck,polrelid)) ORDER BY polname) FROM pg_policy WHERE polrelid=c.oid),\
        'select',has_table_privilege('harness_writer',c.oid,'SELECT'),\
        'write',has_table_privilege('harness_writer',c.oid,'INSERT,UPDATE,DELETE,TRUNCATE,TRIGGER')\
      )) FROM pg_class c WHERE c.relnamespace=n.oid AND c.relkind='r'),\
      'domain',(SELECT jsonb_build_object('base_type',format_type(t.typbasetype,t.typtypmod),'not_null',t.typnotnull,'constraints',(SELECT jsonb_agg(jsonb_build_object('name',conname,'validated',convalidated,'definition',pg_get_constraintdef(oid))) FROM pg_constraint WHERE contypid=t.oid)) FROM pg_type t WHERE t.typnamespace=n.oid AND t.typname='identifier'))::text \
      FROM pg_namespace n WHERE n.nspname='arboresce_authority'")
        .fetch_one(&mut connection).await?;
    Ok(serde_json::from_str(&raw)?)
}

pub async fn backend(connection: &mut PgConnection) -> Result<i32> {
    Ok(sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(connection)
        .await?)
}

pub async fn clear_context(connection: &mut PgConnection) -> Result<Value> {
    let (organization,domain,principal): (Option<String>,Option<String>,Option<String>) = sqlx::query_as(
        "SELECT nullif(current_setting('arboresce.organization_id',true),''),nullif(current_setting('arboresce.domain_id',true),''),nullif(current_setting('arboresce.principal_id',true),'')")
        .fetch_one(connection).await?;
    Ok(
        if organization.is_none() && domain.is_none() && principal.is_none() {
            Value::Null
        } else {
            json!({"organization_id":organization,"domain_id":domain,"principal_id":principal})
        },
    )
}

pub async fn set_context(connection: &mut PgConnection, value: &Value, local: bool) -> Result<()> {
    sqlx::query("SELECT set_config('arboresce.organization_id',$1,$3),set_config('arboresce.principal_id',$2,$3)")
        .bind(value["organization_id"].as_str().unwrap_or(""))
        .bind(value["principal_id"].as_str().unwrap_or(""))
        .bind(local).execute(connection).await?;
    Ok(())
}

/// Actual isolated catalogue mutations each change only the named discriminator.
pub async fn role_case(service: &Service, input: &Value) -> Result<Value> {
    let mut admin = service.admin(false).await?;
    let (change, restore) = if input["application_role"]["owns_tenant_table"] == true {
        (
            "ALTER TABLE arboresce_authority.memberships OWNER TO harness_writer",
            "ALTER TABLE arboresce_authority.memberships OWNER TO harness_migrator; SET ROLE harness_migrator; GRANT SELECT ON arboresce_authority.memberships TO harness_writer; RESET ROLE",
        )
    } else if input["application_role"]["bypassrls"] == true {
        (
            "ALTER ROLE harness_writer BYPASSRLS",
            "ALTER ROLE harness_writer NOBYPASSRLS",
        )
    } else if input["application_role"]["superuser"] == true {
        (
            "ALTER ROLE harness_writer SUPERUSER",
            "ALTER ROLE harness_writer NOSUPERUSER",
        )
    } else {
        (
            "ALTER TABLE arboresce_authority.memberships NO FORCE ROW LEVEL SECURITY",
            "ALTER TABLE arboresce_authority.memberships FORCE ROW LEVEL SECURITY",
        )
    };
    sqlx::query(change).execute(&mut admin).await?;
    let result = async {
        let mut ordinary = service.harness.connect(false).await.map_err(|_| "role test connection failed")?;
        let row = sqlx::query("SELECT r.rolsuper,r.rolbypassrls,c.relowner=r.oid AS owns,c.relforcerowsecurity FROM pg_roles r CROSS JOIN pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace WHERE r.rolname=current_user AND n.nspname='arboresce_authority' AND c.relname='memberships'")
            .fetch_one(&mut ordinary).await?;
        let observed = json!({"superuser":row.get::<bool,_>("rolsuper"),"bypassrls":row.get::<bool,_>("rolbypassrls"),"owns_tenant_table":row.get::<bool,_>("owns"),"force_row_security":row.get::<bool,_>("relforcerowsecurity")});
        let result = validate_role(&mut ordinary).await;
        Ok::<_,Box<dyn std::error::Error>>((result,observed))
    }.await;
    // Restoration happens even when the observation fails.
    sqlx::raw_sql(restore).execute(&mut admin).await?;
    let (decision, observed) = result?;
    Ok(json!({"error":decision.err().map(|e| format!("{e:?}")),"observed":observed}))
}

/// A qualification probe, not a product record family or product writer.
pub async fn prepare_probes(service: &Service, input: &Value) -> Result<()> {
    let mut connection = service.admin(false).await?;
    sqlx::raw_sql("DROP SCHEMA IF EXISTS tenant_probe CASCADE; CREATE SCHEMA tenant_probe AUTHORIZATION harness_migrator; REVOKE ALL ON SCHEMA tenant_probe FROM PUBLIC;\
      CREATE TABLE tenant_probe.memberships(organization_id uuid NOT NULL,principal_id uuid NOT NULL,PRIMARY KEY(organization_id,principal_id));\
      CREATE TABLE tenant_probe.records(organization_id uuid NOT NULL,domain_id uuid NOT NULL,record_id uuid NOT NULL,value text NOT NULL,PRIMARY KEY(organization_id,domain_id,record_id));\
      CREATE TABLE tenant_probe.targets(organization_id uuid NOT NULL,domain_id uuid NOT NULL,kind text NOT NULL CHECK(kind IN('project','party','capture','expense','grant','membership')),id uuid NOT NULL,PRIMARY KEY(organization_id,domain_id,kind,id));\
      CREATE TABLE tenant_probe.edges(organization_id uuid NOT NULL,domain_id uuid NOT NULL,kind text NOT NULL,id uuid NOT NULL,FOREIGN KEY(organization_id,domain_id,kind,id) REFERENCES tenant_probe.targets);\
      ALTER TABLE tenant_probe.records OWNER TO harness_migrator; ALTER TABLE tenant_probe.memberships OWNER TO harness_migrator; ALTER TABLE tenant_probe.targets OWNER TO harness_migrator; ALTER TABLE tenant_probe.edges OWNER TO harness_migrator;\
      ALTER TABLE tenant_probe.records ENABLE ROW LEVEL SECURITY; ALTER TABLE tenant_probe.records FORCE ROW LEVEL SECURITY;\
      CREATE POLICY tenant_context ON tenant_probe.records USING (organization_id::text=nullif(current_setting('arboresce.organization_id',true),'') AND EXISTS(SELECT 1 FROM tenant_probe.memberships m WHERE m.organization_id=records.organization_id AND m.principal_id::text=nullif(current_setting('arboresce.principal_id',true),''))) WITH CHECK (organization_id::text=nullif(current_setting('arboresce.organization_id',true),'') AND EXISTS(SELECT 1 FROM tenant_probe.memberships m WHERE m.organization_id=records.organization_id AND m.principal_id::text=nullif(current_setting('arboresce.principal_id',true),'')));\
      GRANT USAGE ON SCHEMA tenant_probe TO harness_writer; GRANT SELECT ON tenant_probe.memberships,tenant_probe.targets TO harness_writer; GRANT SELECT,INSERT ON tenant_probe.records,tenant_probe.edges TO harness_writer")
        .execute(&mut connection).await?;
    for row in input["canonical_active_memberships"]
        .as_array()
        .ok_or("missing probe memberships")?
    {
        sqlx::query("INSERT INTO tenant_probe.memberships VALUES($1::uuid,$2::uuid)")
            .bind(row["organization_id"].as_str())
            .bind(row["principal_id"].as_str())
            .execute(&mut connection)
            .await?;
    }
    for row in input["rows"].as_array().ok_or("missing probe rows")? {
        probe_insert(&mut connection, row).await?;
    }
    for row in input["existing_typed_relationship_targets"]
        .as_array()
        .ok_or("missing probe targets")?
    {
        sqlx::query("INSERT INTO tenant_probe.targets VALUES($1::uuid,$2::uuid,$3,$4::uuid)")
            .bind(row["workspace"]["organization_id"].as_str())
            .bind(row["workspace"]["domain_id"].as_str())
            .bind(row["kind"].as_str())
            .bind(row["id"].as_str())
            .execute(&mut connection)
            .await?;
    }
    Ok(())
}

pub async fn probe_insert(
    connection: &mut PgConnection,
    row: &Value,
) -> std::result::Result<u64, sqlx::Error> {
    Ok(
        sqlx::query("INSERT INTO tenant_probe.records VALUES($1::uuid,$2::uuid,$3::uuid,$4)")
            .bind(row["organization_id"].as_str())
            .bind(row["domain_id"].as_str())
            .bind(row["record_id"].as_str())
            .bind(row["value"].as_str())
            .execute(connection)
            .await?
            .rows_affected(),
    )
}

pub async fn probe_rows(connection: &mut PgConnection) -> Result<Value> {
    let raw: String = sqlx::query_scalar("SELECT coalesce(jsonb_agg(to_jsonb(t) ORDER BY organization_id,record_id),'[]')::text FROM tenant_probe.records t")
        .fetch_one(connection).await?;
    Ok(serde_json::from_str(&raw)?)
}

pub async fn relation(service: &Service, input: &Value) -> Result<Value> {
    let selected = workspace(&input["selected_workspace"]);
    let source = reference(&input["source"])?;
    let target = reference(&input["target"])?;
    let source_id = selected.require_reference(source);
    let target_id = selected.require_reference(target);
    let scoped = source_id.is_ok() && target_id.is_ok();
    let mut connection = service
        .harness
        .connect(false)
        .await
        .map_err(|_| "relation connection unavailable")?;
    let mut tx = connection.begin().await?;
    // Keep the selected organization/domain in the source key. Only the incoming
    // typed target identity is reused; this is the actual composite-FK challenge.
    let result =
        sqlx::query("INSERT INTO tenant_probe.edges VALUES($1::uuid,$2::uuid,$3,$4::uuid)")
            .bind(input["selected_workspace"]["organization_id"].as_str())
            .bind(input["selected_workspace"]["domain_id"].as_str())
            .bind(input["target"]["kind"].as_str())
            .bind(input["target"]["id"].as_str())
            .execute(&mut *tx)
            .await;
    let (code, inserted, rows) = match result {
        Ok(result) => {
            let rows: String = sqlx::query_scalar(
                "SELECT coalesce(jsonb_agg(to_jsonb(t)),'[]')::text FROM tenant_probe.edges t",
            )
            .fetch_one(&mut *tx)
            .await?;
            (
                None,
                result.rows_affected(),
                serde_json::from_str::<Value>(&rows)?,
            )
        }
        Err(error) => {
            let code = error
                .as_database_error()
                .and_then(|error| error.code())
                .ok_or("relationship failed without a database SQLSTATE")?
                .to_string();
            (Some(code), 0, json!([]))
        }
    };
    tx.rollback().await?;
    let remaining: i64 = sqlx::query_scalar("SELECT count(*) FROM tenant_probe.edges")
        .fetch_one(&mut connection)
        .await?;
    Ok(
        json!({"scope_matches":scoped,"resolved_source_id":source_id.ok().map(|id|id.to_string()),"resolved_target_id":target_id.ok().map(|id|id.to_string()),"sqlstate":code,"inserted":inserted,"inserted_rows":rows,"edges_after_rollback":remaining,"kind_preserved":format!("{:?}",target.kind()).to_ascii_lowercase()==input["target"]["kind"].as_str().unwrap()
            && format!("{:?}",source.kind()).to_ascii_lowercase()==input["source"]["kind"].as_str().unwrap(),"workspace_preserved":target.workspace()==workspace(&input["target"]["workspace"])}),
    )
}

pub async fn revoke(service: &Service, table: &str, value: TenantContext) -> Result<()> {
    let statement = match table {
        "organizations" => {
            "UPDATE arboresce_authority.organizations SET state='revoked' WHERE organization_id=$1::uuid"
        }
        "principals" => {
            "UPDATE arboresce_authority.principals SET state='revoked' WHERE principal_id=$2::uuid AND $1::uuid IS NOT NULL"
        }
        "domains" => {
            "UPDATE arboresce_authority.domains SET state='revoked' WHERE organization_id=$1::uuid AND domain_id=$3::uuid AND $2::uuid IS NOT NULL"
        }
        "memberships" => {
            "UPDATE arboresce_authority.memberships SET state='revoked' WHERE organization_id=$1::uuid AND principal_id=$2::uuid"
        }
        _ => return Err("unsupported revocation family".into()),
    };
    let mut admin = service.admin(false).await?;
    // All variants bind the same exact context with explicitly typed parameters.
    let query = sqlx::query(statement).bind(value.workspace().organization().to_string());
    let query = if table == "organizations" {
        query
    } else {
        query.bind(value.principal().to_string())
    };
    let query = if table == "domains" {
        query.bind(value.workspace().domain().to_string())
    } else {
        query
    };
    let affected = query.execute(&mut admin).await?.rows_affected();
    if affected != 1 {
        return Err("revocation did not affect exact selected row".into());
    }
    Ok(())
}

pub async fn canonical_rls(service: &Service, seed_rows: &Value, input: &Value) -> Result<Value> {
    seed(service, seed_rows).await?;
    if let Some(table) = input["revoke_table"].as_str() {
        revoke(service, table, context(&input["context"])).await?;
    }
    let observation = async {
        let mut connection = service
            .harness
            .connect(false)
            .await
            .map_err(|_| "canonical RLS connection unavailable")?;
        let before = backend(&mut connection).await?;
        let mut tx = connection.begin().await?;
        set_context(&mut tx, &input["context"], true).await?;
        sqlx::query("SELECT set_config('arboresce.domain_id',$1,true)")
            .bind(input["context"]["domain_id"].as_str().unwrap_or(""))
            .execute(&mut *tx)
            .await?;
        let visible = canonical_rows(&mut tx).await?;
        tx.rollback().await?;
        Ok::<_, Box<dyn std::error::Error>>(json!({
            "visible_canonical_rows": visible,
            "same_backend": before == backend(&mut connection).await?,
            "positive_rollback": true,
            "context_after_terminal": clear_context(&mut connection).await?
        }))
    }
    .await;
    let after = snapshot(service).await;
    seed(service, seed_rows).await?;
    let mut observed = observation?;
    observed["canonical_rows_after"] = after?;
    observed["canonical_rows_after_restoration"] = snapshot(service).await?;
    observed["catalogue_after_restoration"] = catalogue(service).await?;
    Ok(observed)
}

async fn ordinary_read(service: &Service, input: &Value) -> Result<Value> {
    let mut session: TenantSession = service.harness.tenant_session().await?;
    let before = backend(&mut session.connection).await?;
    let selected = context(&input["context"]);
    let outcome = match session.begin(selected).await {
        Ok(mut tx) => {
            let preserved = tx.context() == selected;
            let membership = tx.current_membership().await?;
            let (isolation,read_only):(String,String)=sqlx::query_as("SELECT current_setting('transaction_isolation'),current_setting('transaction_read_only')").fetch_one(&mut *tx.transaction).await?;
            let terminal = if input["terminal"] == "commit" {
                tx.commit().await
            } else {
                tx.rollback().await
            };
            terminal?;
            json!({"membership_id":membership.map(|id|id.to_string()),"completion":"ok","context_preserved":preserved,"isolation":isolation,"read_only":read_only=="on"})
        }
        Err(error) => json!({"error":format!("{error:?}")}),
    };
    let after = backend(&mut session.connection).await?;
    Ok(
        json!({"outcome":outcome,"same_backend":before==after,"context_after_terminal":clear_context(&mut session.connection).await?}),
    )
}

async fn view_identity(connection: &mut PgConnection) -> Result<Value> {
    let raw:String=sqlx::query_scalar("SELECT jsonb_build_object('oid',c.oid,'owner',c.relowner,'acl',c.relacl::text,'definition',pg_get_viewdef(c.oid,true),'columns',(SELECT jsonb_agg(jsonb_build_array(attnum,attname,atttypid,atttypmod,attnotnull) ORDER BY attnum) FROM pg_attribute WHERE attrelid=c.oid AND attnum>0))::text FROM pg_class c WHERE c.oid='pg_catalog.pg_roles'::regclass").fetch_one(connection).await?;
    Ok(serde_json::from_str(&raw)?)
}

pub async fn api(service: &Service, seed_rows: &Value, input: &Value) -> Result<Value> {
    seed(service, seed_rows).await?;
    let selected = context(&input["context"]);
    let recipe = input["recipe"].as_str().ok_or("missing tenant recipe")?;
    let mut proof = json!({});
    let result: Result<Value> = match recipe {
        "read" => ordinary_read(service, input).await,
        "revoked" => {
            revoke(service, input["table"].as_str().unwrap(), selected).await?;
            ordinary_read(service, input).await
        }
        "retained-context" => {
            let mut session = service.harness.tenant_session().await?;
            let pid = backend(&mut session.connection).await?;
            let setting = format!("arboresce.{}", input["field"].as_str().unwrap());
            sqlx::query("SELECT set_config($1,$2,false)")
                .bind(&setting)
                .bind(input["value"].as_str())
                .execute(&mut session.connection)
                .await?;
            let error = session.begin(selected).await.err();
            let retained = clear_context(&mut session.connection).await?;
            sqlx::query("SELECT set_config($1,'',false)")
                .bind(&setting)
                .execute(&mut session.connection)
                .await?;
            session.begin(selected).await?.rollback().await?;
            proof = json!({"retained":retained});
            Ok(
                json!({"outcome":{"error":error.map(|error|error.to_string())},"same_backend":backend(&mut session.connection).await?==pid,"context_after_terminal":clear_context(&mut session.connection).await?}),
            )
        }
        "corrupt-membership" => {
            let mut admin = service.admin(false).await?;
            sqlx::raw_sql("ALTER DOMAIN arboresce_authority.identifier DROP CONSTRAINT identifier_check; UPDATE arboresce_authority.memberships SET membership_id='00000000-0000-0000-0000-000000000000'::uuid WHERE organization_id='11111111-1111-4111-8111-111111111111'::uuid AND principal_id='01010101-0101-4101-8101-010101010101'::uuid").execute(&mut admin).await?;
            let observed = ordinary_read(service, input).await;
            let corrupt = snapshot(service).await;
            // Restore the exact value before adding the validated domain check.
            sqlx::raw_sql("UPDATE arboresce_authority.memberships SET membership_id='10101010-1010-4010-8010-101010101010'::uuid WHERE organization_id='11111111-1111-4111-8111-111111111111'::uuid AND principal_id='01010101-0101-4101-8101-010101010101'::uuid; ALTER DOMAIN arboresce_authority.identifier ADD CONSTRAINT identifier_check CHECK ((get_byte(uuid_send(VALUE),6)>>4)=4 AND (get_byte(uuid_send(VALUE),8)>>6)=2)").execute(&mut admin).await?;
            proof = json!({"canonical_before_restoration":corrupt?});
            observed
        }
        "catalogue-failure" => {
            let mut session = service.harness.tenant_session().await?;
            let pid = backend(&mut session.connection).await?;
            let mut admin = service.admin(false).await?;
            let before = view_identity(&mut admin).await?;
            sqlx::query(
                "ALTER VIEW pg_catalog.pg_roles RENAME TO arboresce_tenant_roles_unavailable",
            )
            .execute(&mut admin)
            .await?;
            let causal = sqlx::query("SELECT 1 FROM pg_roles")
                .execute(&mut session.connection)
                .await
                .err();
            let error = session.begin(selected).await.err();
            sqlx::query(
                "ALTER VIEW pg_catalog.arboresce_tenant_roles_unavailable RENAME TO pg_roles",
            )
            .execute(&mut admin)
            .await?;
            let after = view_identity(&mut admin).await?;
            session.begin(selected).await?.rollback().await?;
            proof = json!({"view_restored":before==after,"causal_sqlstate":causal.and_then(|e|e.as_database_error().and_then(|d|d.code()).map(|c|c.to_string()))});
            Ok(
                json!({"outcome":{"error":error.map(|error|error.to_string())},"same_backend":backend(&mut session.connection).await?==pid,"context_after_terminal":clear_context(&mut session.connection).await?}),
            )
        }
        "read-query-failure" => {
            let mut session = service.harness.tenant_session().await?;
            let pid = backend(&mut session.connection).await?;
            let mut tx = session.begin(selected).await?;
            let first = tx.current_membership().await?;
            let mut admin = service.admin(false).await?;
            let terminated:bool=sqlx::query_scalar("SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE pid=$1 AND datname=current_database() AND usename='harness_writer'").bind(pid).fetch_one(&mut admin).await?;
            let error = tx.current_membership().await.err();
            drop(tx);
            drop(session);
            let absent: bool = sqlx::query_scalar(
                "SELECT NOT EXISTS(SELECT 1 FROM pg_stat_activity WHERE pid=$1)",
            )
            .bind(pid)
            .fetch_one(&mut admin)
            .await?;
            let mut fresh = service.harness.tenant_session().await?;
            let fresh_context = clear_context(&mut fresh.connection).await?;
            fresh.begin(selected).await?.rollback().await?;
            proof = json!({"terminated_owned_backend":terminated,"old_backend_absent":absent,"positive_first_membership":first.map(|id|id.to_string()),"fresh_context_before":fresh_context,"fresh_transaction_rolled_back":true});
            Ok(
                json!({"outcome":{"error":error.map(|error|error.to_string())},"context_after_terminal":clear_context(&mut fresh.connection).await?}),
            )
        }
        "absent-socket" => {
            use std::os::unix::fs::{DirBuilderExt, MetadataExt};
            let directory = std::path::PathBuf::from(format!(
                "/private/tmp/tenant-absent-{}",
                std::process::id()
            ));
            std::fs::DirBuilder::new().mode(0o700).create(&directory)?;
            let before = std::fs::symlink_metadata(&directory)?;
            if !before.is_dir()
                || before.mode() & 0o777 != 0o700
                || std::fs::read_dir(&directory)?.next().is_some()
            {
                return Err(
                    "absent-socket directory is not an owned empty private directory".into(),
                );
            }
            let result: Option<TenantFailure> = crate::Harness::new(directory.clone())
                .unwrap()
                .tenant_session()
                .await
                .err();
            let after = std::fs::symlink_metadata(&directory)?;
            if before.dev() != after.dev()
                || before.ino() != after.ino()
                || before.mode() != after.mode()
                || !after.is_dir()
                || std::fs::read_dir(&directory)?.next().is_some()
            {
                return Err("absent-socket directory changed during connection".into());
            }
            std::fs::remove_dir(&directory)?;
            proof = json!({"owned_directory_absent":!directory.exists(),"private_empty_directory_observed":true,"directory_identity_preserved":true});
            Ok(
                json!({"outcome":{"error":result.map(|error|error.to_string())},"context_after_terminal":null}),
            )
        }
        "revoke-after-begin" => {
            let mut session = service.harness.tenant_session().await?;
            let pid = backend(&mut session.connection).await?;
            let mut tx = session.begin(selected).await?;
            let before = tx.current_membership().await?;
            revoke(service, "memberships", selected).await?;
            let after = tx.current_membership().await?;
            tx.rollback().await?;
            Ok(
                json!({"outcome":{"membership_id":before.map(|id|id.to_string()),"membership_after_revocation":after.map(|id|id.to_string()),"completion":"ok","action_authorization_claim":false},"same_backend":backend(&mut session.connection).await?==pid,"context_after_terminal":clear_context(&mut session.connection).await?}),
            )
        }
        "commit-completion-lost" | "rollback-completion-lost" => {
            let mode = if recipe == "commit-completion-lost" {
                relay::Mode::Commit
            } else {
                relay::Mode::Rollback
            };
            let relay = relay::Relay::start(&service.socket(), mode)?;
            let directory = relay.directory.clone();
            let harness = crate::Harness::new(directory.clone()).unwrap();
            let mut session = harness.tenant_session().await?;
            let pid = backend(&mut session.connection).await?;
            let mut tx: TenantTransaction<'_> = session.begin(selected).await?;
            let first = tx.current_membership().await?;
            let error = if recipe == "commit-completion-lost" {
                tx.commit().await.err()
            } else {
                tx.rollback().await.err()
            };
            drop(session);
            let (server, client) = relay.finish()?;
            let server = server?;
            let client = client?;
            let mut admin = service.admin(false).await?;
            let absent: bool = sqlx::query_scalar(
                "SELECT NOT EXISTS(SELECT 1 FROM pg_stat_activity WHERE pid=$1)",
            )
            .bind(pid)
            .fetch_one(&mut admin)
            .await?;
            let mut fresh = service.harness.tenant_session().await?;
            let fresh_context = clear_context(&mut fresh.connection).await?;
            fresh.begin(selected).await?.rollback().await?;
            proof = json!({"matching_completions_dropped":server.dropped,"client_frames":client.frames,"old_backend_absent":absent,"relay_directory_absent":!directory.exists(),"fresh_context_before":fresh_context,"fresh_transaction_rolled_back":true,"positive_first_membership":first.map(|id|id.to_string())});
            Ok(
                json!({"outcome":{"error":error.map(|error|error.to_string())},"context_after_terminal":clear_context(&mut fresh.connection).await?}),
            )
        }
        _ => return Err("unsupported tenant API recipe".into()),
    };
    let before_restoration = snapshot(service).await?;
    seed(service, seed_rows).await?;
    let mut observed = result?;
    observed["proof"] = proof;
    observed["canonical_rows_after"] = observed["proof"]
        .get("canonical_before_restoration")
        .cloned()
        .unwrap_or(before_restoration);
    observed["canonical_rows_after_restoration"] = snapshot(service).await?;
    observed["catalogue_after_restoration"] = catalogue(service).await?;
    Ok(observed)
}
