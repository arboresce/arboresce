//! Observe real role, owner, RLS, policy and effective column privileges.

use super::{
    AuthorityFailure as Failure, Role,
    catalogue_policy::{self, Policy, TABLES},
};
use sqlx::{Decode, PgConnection, Postgres, Row, Type, postgres::PgRow};

/// Preserve checked field decoding and the catalogue's row-error classification.
fn field<T>(row: &PgRow, key: &str) -> Result<T, Failure>
where
    for<'r> T: Decode<'r, Postgres> + Type<Postgres>,
{
    row.try_get(key).map_err(|_| Failure::UnsafeRole)
}

fn columns(table: &str) -> Option<Vec<&'static str>> {
    let names = match table {
        "organizations" => "organization_id,state,name,lock_guard",
        "domains" => "organization_id,domain_id,state,name,lock_guard",
        "principals" => {
            "principal_id,identity_profile,identity_version,public_key,state,kind,control_revision,created_at,revoked_at,identity_standard,identity_standard_revision,lock_guard"
        }
        "memberships" => {
            "organization_id,principal_id,domain_id,membership_id,state,principal_kind,roles,policy,control_revision,lock_guard"
        }
        "namespace_guard" => "singleton,epoch,lock_guard",
        "workspace_policies" => {
            "organization_id,domain_id,policy,epoch,state,read_roles,direct_review_roles,purposes,grant_root_actions,source_policy_ids,lock_guard"
        }
        "source_policies" => {
            "policy_id,state,expires_at,permitted_principals,permitted_purposes,lock_guard"
        }
        "resources" => {
            "organization_id,domain_id,resource_kind,resource_id,name,control_revision,state,lock_guard"
        }
        "party_aliases" => "organization_id,domain_id,party_id,alias,principal_id",
        "grants" => {
            "organization_id,domain_id,grant_id,issuer_id,issuer_kind,subject_id,subject_kind,parent_grant_id,scope_organization_id,scope_domain_id,scope_kind,scope_resource_id,actions,purposes,delegable,issued_at,expires_at,policy,control_revision,state,revoked_at,lock_guard"
        }
        "immutable_chains" => "organization_id,domain_id,chain_id,scope_interpretation,lock_guard",
        "immutable_members" => {
            "organization_id,domain_id,chain_id,ordinal,grant_id,issuer_id,issuer_kind,subject_id,subject_kind,parent_grant_id,scope_organization_id,scope_domain_id,scope_kind,scope_resource_id,actions,purposes,delegable,issued_at,expires_at,policy,control_revision,state,revoked_at,lock_guard"
        }
        "qualification_markers" => {
            "organization_id,domain_id,marker_id,observation_id,executor_principal_id,selected_chain_id,action,purpose,resource_kind,resource_id,admitted_policy_epoch,admitted_membership_revision,admitted_grant_revision,admitted_resource_control"
        }
        _ => return None,
    };
    Some(names.split(',').collect())
}

fn update(role: &str, table: &str, column: &str) -> bool {
    if role == "harness_writer" {
        return false;
    }
    if role == "harness_authority_admission" {
        return (column == "lock_guard"
            && !matches!(
                table,
                "resources" | "party_aliases" | "qualification_markers"
            ))
            || (table == "resources" && column == "control_revision");
    }
    match table {
        "namespace_guard" => column == "epoch",
        "memberships" => matches!(column, "state" | "control_revision"),
        "grants" => matches!(column, "state" | "control_revision" | "revoked_at"),
        "workspace_policies" => matches!(column, "state" | "epoch" | "purposes"),
        "resources" => column == "control_revision",
        "organizations" | "domains" | "principals" | "source_policies" | "immutable_chains"
        | "immutable_members" => column == "lock_guard",
        _ => false,
    }
}

async fn roles(connection: &mut PgConnection, role: Role) -> Result<(), Failure> {
    let allowed:bool=sqlx::query_scalar(
        "SELECT current_user=$1 AND current_database()='arboresce_migration_harness' \
         AND current_setting('server_version_num')='170011' AND current_setting('row_security')='on' \
         AND current_setting('statement_timeout')='2s' AND current_setting('transaction_timeout')='2s' \
         AND current_setting('lock_timeout')='500ms' AND current_setting('TimeZone')='UTC' \
         AND NOT has_database_privilege(current_user,current_database(),'CREATE,TEMPORARY') \
         AND NOT has_schema_privilege(current_user,'arboresce_authority','CREATE') \
         AND (SELECT datconnlimit=2 AND pg_get_userbyid(datdba)='harness_migrator' FROM pg_database WHERE datname=current_database()) \
         AND (SELECT count(*)=2 AND coalesce(bool_and(rolcanlogin AND NOT (rolsuper OR rolinherit \
             OR rolcreatedb OR rolcreaterole OR rolreplication OR rolbypassrls) AND rolconnlimit=2 \
             AND NOT EXISTS (SELECT 1 FROM pg_auth_members m WHERE m.member=r.oid OR m.roleid=r.oid)),false) \
             FROM pg_roles r WHERE rolname IN ('harness_authority_admission','harness_authority_permission')) \
         AND (SELECT count(*)=1 AND coalesce(bool_and(rolcanlogin AND NOT (rolsuper OR rolcreatedb \
             OR rolcreaterole OR rolreplication OR rolbypassrls) AND rolconnlimit=2),false) \
             FROM pg_roles WHERE rolname='harness_writer') \
         AND (SELECT pg_get_userbyid(nspowner)='harness_migrator' \
             AND NOT EXISTS (SELECT 1 FROM aclexplode(nspacl) WHERE grantee=0) \
             FROM pg_namespace WHERE nspname='arboresce_authority')",
    ).bind(role.name()).fetch_one(&mut *connection).await.map_err(|_|Failure::Unavailable)?;
    if !allowed {
        return Err(Failure::UnsafeRole);
    }
    let bad:i64=sqlx::query_scalar(
        "SELECT (SELECT count(*) FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace \
            WHERE n.nspname NOT LIKE 'pg_%' AND n.nspname<>'information_schema') \
          + (SELECT count(*) FROM pg_trigger t JOIN pg_class c ON c.oid=t.tgrelid \
             JOIN pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname='arboresce_authority' AND NOT t.tgisinternal)",
    ).fetch_one(connection).await.map_err(|_|Failure::Unavailable)?;
    if bad != 0 {
        return Err(Failure::UnsafeRole);
    }
    Ok(())
}

async fn tables(connection: &mut PgConnection) -> Result<(), Failure> {
    let rows=sqlx::query(
        "SELECT c.relname::text,pg_get_userbyid(c.relowner)::text AS owner,c.relkind::text,\
            c.relrowsecurity,c.relforcerowsecurity,c.relispartition,\
            EXISTS(SELECT 1 FROM pg_inherits i WHERE i.inhrelid=c.oid OR i.inhparent=c.oid) AS inherited,\
            EXISTS(SELECT 1 FROM aclexplode(c.relacl) WHERE grantee=0 OR (is_grantable AND grantee<>c.relowner)) AS forbidden_acl \
         FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace \
         WHERE n.nspname='arboresce_authority' AND c.relkind NOT IN ('i','I','t') ORDER BY c.relname LIMIT 14",
    ).fetch_all(connection).await.map_err(|_|Failure::Unavailable)?;
    if rows.len() != 13 {
        return Err(Failure::UnsafeRole);
    }
    for row in rows {
        let name: String = field(&row, "relname")?;
        let owner: String = field(&row, "owner")?;
        let kind: String = field(&row, "relkind")?;
        let flag = |key| field::<bool>(&row, key);
        if !TABLES.contains(&name.as_str())
            || owner != "harness_migrator"
            || kind != "r"
            || !flag("relrowsecurity")?
            || !flag("relforcerowsecurity")?
            || flag("relispartition")?
            || flag("inherited")?
            || flag("forbidden_acl")?
        {
            return Err(Failure::UnsafeRole);
        }
    }
    Ok(())
}

async fn privileges(connection: &mut PgConnection) -> Result<(), Failure> {
    let actual=sqlx::query(
        "SELECT c.relname::text,a.attname::text,r.role,\
            has_column_privilege(r.role,c.oid,a.attnum,'SELECT') AS read_allowed,\
            has_column_privilege(r.role,c.oid,a.attnum,'INSERT') AS insert_allowed,\
            has_column_privilege(r.role,c.oid,a.attnum,'UPDATE') AS update_allowed,\
            has_column_privilege(r.role,c.oid,a.attnum,'REFERENCES') AS reference_allowed,\
            has_table_privilege(r.role,c.oid,'INSERT,UPDATE,DELETE,TRUNCATE,TRIGGER,REFERENCES') AS table_write,\
            has_schema_privilege(r.role,n.oid,'CREATE') AS schema_create,\
            has_schema_privilege(r.role,n.oid,'USAGE') AS schema_usage,\
            EXISTS(SELECT 1 FROM aclexplode(a.attacl) WHERE grantee=0 OR (is_grantable AND grantee<>c.relowner)) AS forbidden_acl \
         FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace \
         JOIN pg_attribute a ON a.attrelid=c.oid AND a.attnum>0 AND NOT a.attisdropped \
         CROSS JOIN (VALUES ('harness_authority_admission'::text),('harness_authority_permission'::text),('harness_writer'::text)) r(role) \
         WHERE n.nspname='arboresce_authority' AND c.relkind='r' ORDER BY c.relname,a.attnum,r.role LIMIT 1025",
    ).fetch_all(connection).await.map_err(|_|Failure::Unavailable)?;
    let count = TABLES.iter().try_fold(0_usize, |count, table| {
        columns(table)
            .map(|names| count + names.len())
            .ok_or(Failure::UnsafeRole)
    })? * 3;
    if actual.len() != count {
        return Err(Failure::UnsafeRole);
    }
    for row in actual {
        let table: String = field(&row, "relname")?;
        let column: String = field(&row, "attname")?;
        let role: String = field(&row, "role")?;
        if !columns(&table).is_some_and(|names| names.contains(&column.as_str())) {
            return Err(Failure::UnsafeRole);
        }
        let read = match role.as_str() {
            "harness_authority_admission" => true,
            "harness_authority_permission" => table != "qualification_markers",
            "harness_writer" => matches!(
                table.as_str(),
                "organizations" | "domains" | "principals" | "memberships"
            ),
            _ => return Err(Failure::UnsafeRole),
        };
        let insert = role == "harness_authority_admission" && table == "qualification_markers";
        let flag = |key| field::<bool>(&row, key);
        if flag("read_allowed")? != read
            || flag("insert_allowed")? != insert
            || flag("update_allowed")? != update(&role, &table, &column)
            || flag("reference_allowed")?
            || flag("table_write")?
            || flag("schema_create")?
            || !flag("schema_usage")?
            || flag("forbidden_acl")?
        {
            return Err(Failure::UnsafeRole);
        }
    }
    Ok(())
}

async fn policies(connection: &mut PgConnection) -> Result<(), Failure> {
    let rows=sqlx::query(
        "SELECT c.relname::text,p.polname::text,p.polcmd::text,p.polpermissive,\
          CASE WHEN cardinality(p.polroles) BETWEEN 1 AND 2 THEN \
            ARRAY(SELECT CASE WHEN ids.role_oid=0 THEN 'public'::text ELSE pg_get_userbyid(ids.role_oid)::text END \
                FROM unnest(p.polroles) ids(role_oid) ORDER BY 1) END AS roles,\
          CASE WHEN octet_length(pg_get_expr(p.polqual,p.polrelid))<=8192 THEN pg_get_expr(p.polqual,p.polrelid) END AS using_expression,\
          CASE WHEN octet_length(pg_get_expr(p.polwithcheck,p.polrelid))<=8192 THEN pg_get_expr(p.polwithcheck,p.polrelid) END AS check_expression \
         FROM pg_policy p JOIN pg_class c ON c.oid=p.polrelid JOIN pg_namespace n ON n.oid=c.relnamespace \
         WHERE n.nspname='arboresce_authority' ORDER BY c.relname,p.polname LIMIT 70",
    ).fetch_all(connection).await.map_err(|_|Failure::Unavailable)?;
    let expected = catalogue_policy::expected();
    if rows.len() != expected.len() {
        return Err(Failure::UnsafeRole);
    }
    for (row, expected) in rows.iter().zip(expected) {
        let expression = |key| {
            field::<Option<String>>(row, key)
                .map(|value| value.map(|value| catalogue_policy::normalized(&value)))
        };
        let actual = Policy {
            table: field(row, "relname")?,
            name: field(row, "polname")?,
            command: field(row, "polcmd")?,
            permissive: field(row, "polpermissive")?,
            roles: field(row, "roles")?,
            using: expression("using_expression")?,
            check: expression("check_expression")?,
        };
        if actual != expected {
            return Err(Failure::UnsafeRole);
        }
    }
    Ok(())
}

pub(crate) async fn validate(connection: &mut PgConnection, role: Role) -> Result<(), Failure> {
    roles(connection, role).await?;
    tables(connection).await?;
    privileges(connection).await?;
    policies(connection).await
}
