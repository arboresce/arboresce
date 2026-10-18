//! Exact selected schema, policies, privileges and migration history.
//! PostgreSQL deparser comparisons preserve operators, casts and grouping.

use std::collections::BTreeMap;

use arboresce_application::policy::admission::AdmissionFailure as Failure;
use sqlx::{Decode, PgConnection, Postgres, Row, Type, postgres::PgRow};

use super::super::{SchemaVersion, authority::catalogue_policy::normalized, migration};

struct Table {
    name: &'static str,
    columns: &'static str,
    insert: &'static str,
    update: &'static str,
}

const TABLES: [Table; 4] = [
    Table {
        name: "global_budget",
        columns: "singleton,active_limit,memory_limit,active_used,memory_used,last_observed_at,next_reservation_order",
        insert: "",
        update: "active_used,memory_used,last_observed_at,next_reservation_order",
    },
    Table {
        name: "organization_budgets",
        columns: "organization_id,pending_limit,active_limit,storage_limit,memory_limit,cost_limit,pending_used,active_used,storage_used,memory_used,cost_used",
        insert: "",
        update: "pending_used,active_used,storage_used,memory_used,cost_used",
    },
    Table {
        name: "reservation_recognition",
        columns: "organization_id,reservation_id,principal_id,owner,replica,fence,attempt_limit,attempts_used,total_deadline,admitted_at,last_observed_at,state,reservation_order",
        insert: "organization_id,reservation_id,principal_id,owner,replica,fence,attempt_limit,attempts_used,total_deadline,admitted_at,last_observed_at,state,reservation_order",
        update: "attempts_used,last_observed_at,state",
    },
    Table {
        name: "reservations",
        columns: "organization_id,reservation_id,principal_id,replica,pending,active,storage_bytes,memory_bytes,cost_microusd,owner,fence,admitted_at,expires_at,attempt_limit,total_deadline,state,dispatch_outcome",
        insert: "organization_id,reservation_id,principal_id,replica,pending,active,storage_bytes,memory_bytes,cost_microusd,owner,fence,admitted_at,expires_at,attempt_limit,total_deadline,state",
        update: "expires_at,state",
    },
];

fn field<T>(row: &PgRow, key: &str) -> Result<T, Failure>
where
    for<'r> T: Decode<'r, Postgres> + Type<Postgres>,
{
    row.try_get(key).map_err(|_| Failure::UnsafeRole)
}

fn expected_type(table: &str, column: &str) -> &'static str {
    match column {
        "organization_id" | "reservation_id" | "principal_id" => "arboresce_authority.identifier",
        "singleton" | "attempt_limit" | "attempts_used" => "smallint",
        "last_observed_at" | "admitted_at" | "expires_at" | "total_deadline" => {
            "timestamp with time zone"
        }
        "owner" | "replica" | "state" | "dispatch_outcome" => "text",
        _ if table == "global_budget"
            || table == "organization_budgets"
            || table == "reservations"
            || column == "fence"
            || column == "reservation_order" =>
        {
            "bigint"
        }
        _ => "unsupported",
    }
}

async fn tables(connection: &mut PgConnection) -> Result<(), Failure> {
    let safe: bool = sqlx::query_scalar(
        "SELECT coalesce((SELECT pg_get_userbyid(nspowner)='harness_migrator' \
         AND NOT has_schema_privilege('harness_authority_admission',oid,'CREATE') \
         AND has_schema_privilege('harness_authority_admission',oid,'USAGE') \
         AND NOT has_schema_privilege('harness_authority_permission',oid,'USAGE,CREATE') \
         AND NOT has_schema_privilege('harness_writer',oid,'USAGE,CREATE') \
         AND NOT EXISTS (SELECT 1 FROM aclexplode(nspacl) a \
             WHERE a.grantee NOT IN (nspowner,'harness_authority_admission'::regrole) \
                OR (a.is_grantable AND a.grantee<>nspowner)) \
         FROM pg_namespace WHERE nspname='arboresce_admission'),false) \
         AND has_schema_privilege(current_user,'public','USAGE') \
         AND NOT has_schema_privilege(current_user,'public','CREATE') \
         AND NOT EXISTS(SELECT 1 FROM pg_trigger t JOIN pg_class c ON c.oid=t.tgrelid \
          JOIN pg_namespace n ON n.oid=c.relnamespace \
          WHERE n.nspname='arboresce_admission' AND NOT t.tgisinternal) \
         AND (SELECT count(*)=6 AND coalesce(bool_and(i.indisvalid AND i.indisready AND i.indislive \
           AND i.indnkeyatts=i.indnatts AND i.indexprs IS NULL AND i.indpred IS NULL \
           AND am.amname='btree' AND EXISTS(SELECT 1 FROM pg_constraint k \
              WHERE k.conindid=i.indexrelid AND k.contype IN ('p','u'))),false) \
          FROM pg_index i JOIN pg_class c ON c.oid=i.indexrelid \
          JOIN pg_namespace n ON n.oid=c.relnamespace JOIN pg_am am ON am.oid=c.relam \
          WHERE n.nspname='arboresce_admission')",
    ).fetch_one(&mut *connection).await.map_err(|_|Failure::Unavailable)?;
    if !safe {
        return Err(Failure::UnsafeRole);
    }
    let rows=sqlx::query(
        "SELECT c.relname::text,pg_get_userbyid(c.relowner)::text AS owner,c.relkind::text, \
         c.relrowsecurity,c.relforcerowsecurity,c.relispartition, \
         EXISTS(SELECT 1 FROM pg_inherits i WHERE i.inhrelid=c.oid OR i.inhparent=c.oid) AS inherited, \
         EXISTS(SELECT 1 FROM aclexplode(c.relacl) a \
          WHERE a.grantee NOT IN(c.relowner,'harness_authority_admission'::regrole) \
             OR (a.is_grantable AND a.grantee<>c.relowner)) AS forbidden_acl \
         FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace \
         WHERE n.nspname='arboresce_admission' AND c.relkind NOT IN ('i','I','t') \
         ORDER BY c.relname LIMIT 5",
    ).fetch_all(&mut *connection).await.map_err(|_|Failure::Unavailable)?;
    if rows.len() != 4 {
        return Err(Failure::UnsafeRole);
    }
    for row in &rows {
        let name: String = field(row, "relname")?;
        let rls = name != "global_budget";
        if !TABLES.iter().any(|t| t.name == name)
            || field::<String>(row, "owner")? != "harness_migrator"
            || field::<String>(row, "relkind")? != "r"
            || field::<bool>(row, "relrowsecurity")? != rls
            || field::<bool>(row, "relforcerowsecurity")? != rls
            || field::<bool>(row, "relispartition")?
            || field::<bool>(row, "inherited")?
            || field::<bool>(row, "forbidden_acl")?
        {
            return Err(Failure::UnsafeRole);
        }
    }
    let rows=sqlx::query(
        "SELECT c.relname::text,a.attname::text,a.attnum,format_type(a.atttypid,a.atttypmod) AS type, \
         a.attnotnull,a.atthasdef,a.attidentity::text,a.attgenerated::text, \
         co.collname::text AS collation, \
         ARRAY[has_column_privilege('harness_authority_admission',c.oid,a.attnum,'SELECT'), \
          has_column_privilege('harness_authority_permission',c.oid,a.attnum,'SELECT'), \
          has_column_privilege('harness_writer',c.oid,a.attnum,'SELECT')] AS reads, \
         ARRAY[has_column_privilege('harness_authority_admission',c.oid,a.attnum,'INSERT'), \
          has_column_privilege('harness_authority_permission',c.oid,a.attnum,'INSERT'), \
          has_column_privilege('harness_writer',c.oid,a.attnum,'INSERT')] AS inserts, \
         ARRAY[has_column_privilege('harness_authority_admission',c.oid,a.attnum,'UPDATE'), \
          has_column_privilege('harness_authority_permission',c.oid,a.attnum,'UPDATE'), \
          has_column_privilege('harness_writer',c.oid,a.attnum,'UPDATE')] AS updates, \
         EXISTS(SELECT 1 FROM aclexplode(a.attacl) x \
           WHERE x.grantee NOT IN(c.relowner,'harness_authority_admission'::regrole) \
              OR (x.is_grantable AND x.grantee<>c.relowner)) AS forbidden_acl, \
         has_table_privilege('harness_authority_admission',c.oid,'INSERT,UPDATE,TRUNCATE,REFERENCES,TRIGGER') \
          OR has_table_privilege('harness_authority_permission',c.oid,'SELECT,INSERT,UPDATE,DELETE,TRUNCATE,REFERENCES,TRIGGER') \
          OR has_table_privilege('harness_writer',c.oid,'SELECT,INSERT,UPDATE,DELETE,TRUNCATE,REFERENCES,TRIGGER') AS table_write, \
         has_table_privilege('harness_authority_admission',c.oid,'DELETE') AS deletion \
         FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace \
         JOIN pg_attribute a ON a.attrelid=c.oid LEFT JOIN pg_collation co ON co.oid=a.attcollation \
         WHERE n.nspname='arboresce_admission' AND c.relkind='r' AND a.attnum>0 AND NOT a.attisdropped \
         ORDER BY c.relname,a.attnum LIMIT 49",
    ).fetch_all(connection).await.map_err(|_|Failure::Unavailable)?;
    if rows.len() != 48 {
        return Err(Failure::UnsafeRole);
    }
    for row in &rows {
        let table: String = field(row, "relname")?;
        let column: String = field(row, "attname")?;
        let table = TABLES
            .iter()
            .find(|t| t.name == table)
            .ok_or(Failure::UnsafeRole)?;
        let position: i16 = field(row, "attnum")?;
        if position <= 0
            || table.columns.split(',').nth(position as usize - 1) != Some(column.as_str())
            || field::<String>(row, "type")? != expected_type(table.name, &column)
            || field::<bool>(row, "attnotnull")? == (column == "dispatch_outcome")
            || field::<bool>(row, "atthasdef")?
            || !field::<String>(row, "attidentity")?.is_empty()
            || !field::<String>(row, "attgenerated")?.is_empty()
            || (matches!(column.as_str(), "owner" | "replica")
                && field::<Option<String>>(row, "collation")?.as_deref() != Some("C"))
            || field::<Vec<bool>>(row, "reads")? != [true, false, false]
            || field::<Vec<bool>>(row, "inserts")?
                != [table.insert.split(',').any(|v| v == column), false, false]
            || field::<Vec<bool>>(row, "updates")?
                != [table.update.split(',').any(|v| v == column), false, false]
            || field::<bool>(row, "forbidden_acl")?
            || field::<bool>(row, "table_write")?
            || field::<bool>(row, "deletion")? != (table.name == "reservations")
        {
            return Err(Failure::UnsafeRole);
        }
    }
    Ok(())
}

fn exact_time(column: &str) -> String {
    // PostgreSQL puts CASE on a new line after the opening parenthesis. The
    // existing whitespace-token normalizer retains that separator.
    format!(
        "CHECK ( CASE WHEN isfinite({column}) THEN ((({column} >= '0001-01-01 00:00:00+00'::timestamp with time zone) AND ({column} <= '9999-12-31 23:59:59.999+00'::timestamp with time zone)) AND (mod(EXTRACT(microseconds FROM {column}), (1000)::numeric) = (0)::numeric)) ELSE false END)"
    )
}

fn expected_constraints() -> BTreeMap<(String, String), String> {
    let mut result = BTreeMap::new();
    let mut put = |table: &str, name: &str, definition: String| {
        result.insert((table.to_owned(), name.to_owned()), definition);
    };
    put(
        "global_budget",
        "global_budget_pkey",
        "PRIMARY KEY (singleton)".to_owned(),
    );
    put(
        "global_budget",
        "global_budget_singleton_check",
        "CHECK ((singleton = 1))".to_owned(),
    );
    put(
        "global_budget",
        "global_budget_next_reservation_order_check",
        "CHECK ((next_reservation_order > 0))".to_owned(),
    );
    put(
        "global_budget",
        "global_budget_last_observed_at_check",
        exact_time("last_observed_at"),
    );
    put(
        "organization_budgets",
        "organization_budgets_pkey",
        "PRIMARY KEY (organization_id)".to_owned(),
    );
    put("organization_budgets","organization_budgets_fkey_1","FOREIGN KEY (organization_id) REFERENCES arboresce_authority.organizations(organization_id)".to_owned());
    for (table, columns) in [
        (
            "global_budget",
            "active_limit,memory_limit,active_used,memory_used",
        ),
        (
            "organization_budgets",
            "pending_limit,pending_used,active_limit,active_used,storage_limit,storage_used,memory_limit,memory_used,cost_limit,cost_used",
        ),
        (
            "reservations",
            "pending,active,storage_bytes,memory_bytes,cost_microusd",
        ),
    ] {
        for column in columns.split(',') {
            put(
                table,
                &format!("{table}_{column}_check"),
                format!("CHECK (({column} >= 0))"),
            );
        }
    }
    put(
        "reservation_recognition",
        "reservation_recognition_pkey",
        "PRIMARY KEY (organization_id, reservation_id)".to_owned(),
    );
    put("reservation_recognition","reservation_recognition_unique_1","UNIQUE (organization_id, reservation_id, principal_id, owner, replica, fence, attempt_limit, total_deadline, admitted_at, state)".to_owned());
    put(
        "reservation_recognition",
        "reservation_recognition_unique_2",
        "UNIQUE (reservation_order)".to_owned(),
    );
    put("reservation_recognition","reservation_recognition_fkey_1","FOREIGN KEY (organization_id) REFERENCES arboresce_admission.organization_budgets(organization_id)".to_owned());
    put("reservation_recognition","reservation_recognition_fkey_2","FOREIGN KEY (organization_id, principal_id) REFERENCES arboresce_authority.memberships(organization_id, principal_id)".to_owned());
    put(
        "reservation_recognition",
        "reservation_recognition_reservation_order_check",
        "CHECK ((reservation_order > 0))".to_owned(),
    );
    put(
        "reservation_recognition",
        "reservation_recognition_attempts_used_check",
        "CHECK (((attempts_used >= 0) AND (attempts_used <= attempt_limit)))".to_owned(),
    );
    put("reservation_recognition","reservation_recognition_lifetime_check","CHECK (((admitted_at < total_deadline) AND ((total_deadline - admitted_at) <= '01:00:00'::interval)))".to_owned());
    put(
        "reservation_recognition",
        "reservation_recognition_observation_order_check",
        "CHECK ((last_observed_at >= admitted_at))".to_owned(),
    );
    put(
        "reservations",
        "reservations_pkey",
        "PRIMARY KEY (organization_id, reservation_id)".to_owned(),
    );
    put("reservations","reservations_fkey_1","FOREIGN KEY (organization_id, reservation_id, principal_id, owner, replica, fence, attempt_limit, total_deadline, admitted_at, state) REFERENCES arboresce_admission.reservation_recognition(organization_id, reservation_id, principal_id, owner, replica, fence, attempt_limit, total_deadline, admitted_at, state) ON UPDATE CASCADE".to_owned());
    put(
        "reservations",
        "reservations_expiry_order_check",
        "CHECK (((admitted_at < expires_at) AND (expires_at <= total_deadline)))".to_owned(),
    );
    put("reservations","reservations_resource_nonzero_check","CHECK (((pending > 0) OR (active > 0) OR (storage_bytes > 0) OR (memory_bytes > 0) OR (cost_microusd > 0)))".to_owned());
    for table in ["reservation_recognition", "reservations"] {
        for column in ["owner", "replica"] {
            put(
                table,
                &format!("{table}_{column}_check"),
                format!("CHECK (({column} ~ '^[A-Za-z0-9_.-]{{1,96}}$'::text))"),
            );
        }
        put(
            table,
            &format!("{table}_fence_check"),
            "CHECK (((fence >= 1) AND (fence <= '999999999999999999'::bigint)))".to_owned(),
        );
        put(
            table,
            &format!("{table}_attempt_limit_check"),
            "CHECK (((attempt_limit >= 1) AND (attempt_limit <= 8)))".to_owned(),
        );
        for column in ["admitted_at", "total_deadline"] {
            put(
                table,
                &format!("{table}_{column}_check"),
                exact_time(column),
            );
        }
        let values = if table == "reservation_recognition" {
            "'reserved'::text, 'active'::text, 'reconciliation_required'::text, 'released'::text"
        } else {
            "'reserved'::text, 'active'::text, 'reconciliation_required'::text"
        };
        put(
            table,
            &format!("{table}_state_check"),
            format!("CHECK ((state = ANY (ARRAY[{values}])))"),
        );
    }
    put(
        "reservation_recognition",
        "reservation_recognition_last_observed_at_check",
        exact_time("last_observed_at"),
    );
    put(
        "reservations",
        "reservations_expires_at_check",
        exact_time("expires_at"),
    );
    put("reservations","reservations_dispatch_outcome_check","CHECK ((dispatch_outcome = ANY (ARRAY['accepted_running'::text, 'unknown'::text, 'not_dispatched'::text, 'completed'::text])))".to_owned());
    result
}

async fn constraints(connection: &mut PgConnection) -> Result<(), Failure> {
    let rows=sqlx::query(
        "SELECT c.relname::text,k.conname::text,k.convalidated,k.condeferrable,k.condeferred,k.conislocal, \
         CASE WHEN octet_length(pg_get_constraintdef(k.oid))<=8192 THEN pg_get_constraintdef(k.oid) END AS definition \
         FROM pg_constraint k JOIN pg_class c ON c.oid=k.conrelid \
         JOIN pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname='arboresce_admission' \
         ORDER BY c.relname,k.conname LIMIT 56",
    ).fetch_all(connection).await.map_err(|_|Failure::Unavailable)?;
    let expected = expected_constraints();
    if rows.len() != 55 || expected.len() != 55 {
        return Err(Failure::UnsafeRole);
    }
    for (row, (key, definition)) in rows.iter().zip(expected) {
        if (
            field::<String>(row, "relname")?,
            field::<String>(row, "conname")?,
        ) != key
            || !field::<bool>(row, "convalidated")?
            || field::<bool>(row, "condeferrable")?
            || field::<bool>(row, "condeferred")?
            || !field::<bool>(row, "conislocal")?
            || normalized(&field::<String>(row, "definition")?) != normalized(&definition)
        {
            return Err(Failure::UnsafeRole);
        }
    }
    Ok(())
}

fn setting(key: &str) -> String {
    format!("NULLIF(current_setting('arboresce.{key}'::text, true), ''::text)")
}
fn equal(key: &str) -> String {
    format!("(({key})::text = {})", setting(key))
}

async fn policies(connection: &mut PgConnection) -> Result<(), Failure> {
    let rows=sqlx::query(
        "SELECT c.relname::text,p.polname::text,p.polcmd::text,p.polpermissive, \
         CASE WHEN cardinality(p.polroles)=1 THEN ARRAY(SELECT pg_get_userbyid(x)::text FROM unnest(p.polroles) x) END AS roles, \
         CASE WHEN octet_length(pg_get_expr(p.polqual,p.polrelid))<=8192 THEN pg_get_expr(p.polqual,p.polrelid) END AS using_expression, \
         CASE WHEN octet_length(pg_get_expr(p.polwithcheck,p.polrelid))<=8192 THEN pg_get_expr(p.polwithcheck,p.polrelid) END AS check_expression \
         FROM pg_policy p JOIN pg_class c ON c.oid=p.polrelid JOIN pg_namespace n ON n.oid=c.relnamespace \
         WHERE n.nspname='arboresce_admission' ORDER BY c.relname,p.polname LIMIT 7",
    ).fetch_all(connection).await.map_err(|_|Failure::Unavailable)?;
    if rows.len() != 6 {
        return Err(Failure::UnsafeRole);
    }
    let context = format!(
        "({})",
        ["organization_id", "domain_id", "principal_id"]
            .map(|k| format!("({} IS NOT NULL)", setting(k)))
            .join(" AND ")
    );
    for (index, row) in rows.iter().enumerate() {
        let table = [
            "organization_budgets",
            "reservation_recognition",
            "reservations",
        ][index / 2];
        let complete = index % 2 == 0;
        let expected = if complete {
            context.clone()
        } else if table == "organization_budgets" {
            equal("organization_id")
        } else {
            format!(
                "({} AND {})",
                equal("organization_id"),
                equal("principal_id")
            )
        };
        if field::<String>(row, "relname")? != table
            || field::<String>(row, "polname")?
                != if complete {
                    "complete_context"
                } else {
                    "selected_principal"
                }
            || field::<String>(row, "polcmd")? != "*"
            || field::<bool>(row, "polpermissive")? == complete
            || field::<Vec<String>>(row, "roles")? != ["harness_authority_admission"]
            || normalized(&field::<String>(row, "using_expression")?) != normalized(&expected)
            || normalized(&field::<String>(row, "check_expression")?) != normalized(&expected)
        {
            return Err(Failure::UnsafeRole);
        }
    }
    Ok(())
}

async fn history(connection: &mut PgConnection) -> Result<(), Failure> {
    let rows = sqlx::query(
        "SELECT version,success,CASE WHEN octet_length(checksum)=48 THEN checksum END AS checksum \
         FROM public._sqlx_migrations ORDER BY version LIMIT 6",
    )
    .fetch_all(&mut *connection)
    .await
    .map_err(|_| Failure::Unavailable)?;
    let expected = migration::selected(SchemaVersion::SharedAdmission);
    if rows.len() != 5 {
        return Err(Failure::UnsafeRole);
    }
    for (row, expected) in rows.iter().zip(expected) {
        if field::<i64>(row, "version")? != expected.version
            || !field::<bool>(row, "success")?
            || field::<Vec<u8>>(row, "checksum")?.as_slice() != expected.checksum.as_ref()
        {
            return Err(Failure::UnsafeRole);
        }
    }
    let exact: bool=sqlx::query_scalar(
        "SELECT count(*)=6 AND coalesce(bool_and( \
         has_column_privilege(current_user,c.oid,a.attnum,'SELECT')=(a.attname IN ('version','success','checksum')) \
         AND NOT has_column_privilege(current_user,c.oid,a.attnum,'INSERT,UPDATE,REFERENCES') \
         AND NOT has_table_privilege(current_user,c.oid,'INSERT,UPDATE,DELETE,TRUNCATE,REFERENCES,TRIGGER')),false) \
         FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace JOIN pg_attribute a ON a.attrelid=c.oid \
         WHERE n.nspname='public' AND c.relname='_sqlx_migrations' AND a.attnum>0 AND NOT a.attisdropped",
    ).fetch_one(connection).await.map_err(|_|Failure::Unavailable)?;
    if !exact {
        return Err(Failure::UnsafeRole);
    }
    Ok(())
}

pub(super) async fn validate(connection: &mut PgConnection) -> Result<(), Failure> {
    tables(connection).await?;
    constraints(connection).await?;
    policies(connection).await?;
    history(connection).await
}
