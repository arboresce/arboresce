//! Parent-fenced discovery, actual selected tuple locks and fresh typed reads.

use arboresce_application::policy::authority::{
    CanonicalProjection, DomainRecord, EvaluationRequest, GrantRecord, GrantScope, ImmutableChain,
    MembershipRecord, OrganizationRecord, PrincipalKind, PrincipalRecord, RecordState,
    ResourceRecord, SourcePolicy, WorkspacePolicy,
};
use arboresce_application::policy::principal::IdentityKey;
use arboresce_application::policy::{ScopedReference, Workspace};
use arboresce_domain::{Clock, Identifier};
use sqlx::{AssertSqlSafe, PgConnection, Row, postgres::PgRow};

use super::{AuthorityFailure as Failure, rows};

const TENANT: &str = "organization_id=$1::uuid AND domain_id=$2::uuid";
const ID_COLUMNS: &str = "organization_id::text,domain_id::text";

struct Keys {
    tenant: Vec<String>,
    chain: Vec<String>,
    target: Vec<String>,
    participants: Vec<String>,
    grants: Vec<String>,
    sources: Vec<String>,
    has_chain: bool,
}

pub(crate) fn principal_columns() -> String {
    [
        "principal_id::text,control_revision,revoked_at IS NOT NULL AS revoked_at_present"
            .to_owned(),
        rows::token("kind"),
        rows::token("identity_profile"),
        rows::token("identity_version"),
        rows::token("state"),
        rows::token("identity_standard"),
        rows::provenance("identity_standard_revision"),
        rows::instant("created_at"),
        rows::instant("revoked_at"),
        "CASE WHEN octet_length(public_key)=32 THEN encode(public_key,'hex') END AS public_key"
            .to_owned(),
    ]
    .join(",")
}

pub(crate) fn decode_principal(row: &PgRow) -> Result<PrincipalRecord, Failure> {
    // These NULL legacy fields never acquire an inferred identity profile/kind.
    rows::text(row, "identity_standard")?;
    rows::text(row, "identity_standard_revision")?;
    Ok(PrincipalRecord {
        principal: rows::principal(row, "principal_id", "kind")?,
        identity_profile: rows::text(row, "identity_profile")?,
        identity_version: rows::text(row, "identity_version")?,
        public_key: rows::text(row, "public_key")?,
        revision: rows::revision(row, "control_revision")?,
        state: rows::state(row)?,
        created_at: rows::time(row, "created_at")?,
        revoked_at: rows::optional_time(row, "revoked_at")?,
    })
}

pub(crate) fn membership_columns() -> String {
    [
        format!("{ID_COLUMNS},membership_id::text,principal_id::text,control_revision"),
        rows::token("principal_kind"),
        rows::tokens("roles"),
        rows::token("policy"),
        rows::token("state"),
    ]
    .join(",")
}

pub(crate) fn decode_membership(row: &PgRow) -> Result<MembershipRecord, Failure> {
    Ok(MembershipRecord {
        id: rows::id(row, "membership_id")?,
        principal: rows::principal(row, "principal_id", "principal_kind")?,
        workspace: rows::workspace(row)?,
        roles: rows::set(row, "roles")?,
        policy: rows::text(row, "policy")?,
        revision: rows::revision(row, "control_revision")?,
        state: rows::state(row)?,
    })
}

fn grant_columns() -> String {
    [format!("{ID_COLUMNS},grant_id::text,issuer_id::text,subject_id::text,parent_grant_id::text,\
        scope_organization_id::text,scope_domain_id::text,scope_resource_id::text,delegable,control_revision,\
        revoked_at IS NOT NULL AS revoked_at_present"),
        rows::token("issuer_kind"),rows::token("subject_kind"),rows::token("scope_kind"),
        rows::tokens("actions"),rows::tokens("purposes"),rows::token("policy"),rows::token("state"),
        rows::instant("issued_at"),rows::instant("expires_at"),rows::instant("revoked_at"),
    ].join(",")
}

fn decode_grant(row: &PgRow) -> Result<GrantRecord, Failure> {
    let workspace = Workspace::new(
        rows::id(row, "scope_organization_id")?,
        rows::id(row, "scope_domain_id")?,
    );
    let kind = rows::text(row, "scope_kind")?;
    let target = rows::optional_id(row, "scope_resource_id")?;
    let scope = match (kind.as_str(), target) {
        ("workspace", None) => GrantScope::Workspace(workspace),
        (_, Some(id)) => GrantScope::Exact(ScopedReference::new(workspace, rows::kind(&kind)?, id)),
        _ => return Err(Failure::InvalidStoredData),
    };
    Ok(GrantRecord {
        id: rows::id(row, "grant_id")?,
        issuer: rows::principal(row, "issuer_id", "issuer_kind")?,
        subject: rows::principal(row, "subject_id", "subject_kind")?,
        parent: rows::optional_id(row, "parent_grant_id")?,
        scope,
        actions: rows::set(row, "actions")?,
        purposes: rows::set(row, "purposes")?,
        delegable: row
            .try_get("delegable")
            .map_err(|_| Failure::InvalidStoredData)?,
        issued_at: rows::time(row, "issued_at")?,
        expires_at: rows::time(row, "expires_at")?,
        policy: rows::text(row, "policy")?,
        revision: rows::revision(row, "control_revision")?,
        state: rows::state(row)?,
        revoked_at: rows::optional_time(row, "revoked_at")?,
    })
}

async fn fetch(
    connection: &mut PgConnection,
    table: &'static str,
    columns: &str,
    predicate: &str,
    order: &str,
    parameters: &[String],
    budget: &mut rows::RowBudget,
) -> Result<Vec<PgRow>, Failure> {
    // Table/column/predicate/order originate only in compiled callers. Values are
    // bound separately; no request, fixture or policy text becomes SQL syntax.
    let limit = if matches!(table, "immutable_members" | "grants") {
        budget.lookahead().min(9)
    } else {
        budget.lookahead()
    };
    let sql = format!(
        "SELECT {columns} FROM arboresce_authority.{table} WHERE {predicate} ORDER BY {order} LIMIT {limit}"
    );
    let mut query = sqlx::query(AssertSqlSafe(sql));
    for value in parameters {
        query = query.bind(value);
    }
    let result = query
        .fetch_all(connection)
        .await
        .map_err(|_| Failure::Unavailable)?;
    if matches!(table, "immutable_members" | "grants") && result.len() > 8 {
        return Err(Failure::InvalidStoredData);
    }
    budget.take(result.len())?;
    Ok(result)
}

/// Authentication staging reads only the exact executor under the parent fence.
/// The final read/decision repeats this after all selected tuple waits.
async fn executor(
    connection: &mut PgConnection,
    request: &EvaluationRequest,
) -> Result<(), Failure> {
    let mut budget = rows::RowBudget::new();
    let raw = fetch(
        connection,
        "principals",
        &principal_columns(),
        "principal_id=$1::uuid",
        "principal_id",
        &[request.executor.to_string()],
        &mut budget,
    )
    .await?;
    let row = raw.first().ok_or(Failure::Unauthenticated)?;
    if raw.len() != 1 {
        return Err(Failure::InvalidStoredData);
    }
    let p = decode_principal(row)?;
    if p.state != RecordState::Active
        || p.revoked_at.is_some()
        || p.principal.kind == PrincipalKind::Unsupported
        || IdentityKey::parse(&p.identity_profile, &p.identity_version, &p.public_key).is_err()
    {
        return Err(Failure::Unauthenticated);
    }
    Ok(())
}

pub(crate) async fn load(
    connection: &mut PgConnection,
    request: &EvaluationRequest,
    permission: bool,
    exclusive_target: bool,
    clock: &mut impl Clock,
) -> Result<CanonicalProjection, Failure> {
    executor(connection, request).await?;
    let workspace = request.selected_workspace.ok_or(Failure::InvalidRequest)?;
    let tenant = vec![
        workspace.organization().to_string(),
        workspace.domain().to_string(),
    ];
    let mut chain = tenant.clone();
    chain.push(
        request
            .selected_chain
            .map(|id| id.to_string())
            .unwrap_or_default(),
    );
    let mut participants = vec![request.executor.to_string()];
    let mut grants = Vec::new();
    if request.selected_chain.is_some() {
        let sql = format!(
            "SELECT issuer_id::text,subject_id::text,grant_id::text \
            FROM arboresce_authority.immutable_members WHERE {TENANT} AND chain_id=$3::uuid ORDER BY ordinal LIMIT 9"
        );
        // Only TENANT and fixed column syntax enter this string. Chain values
        // below are always binds, including unknown or malformed selections.
        let members = sqlx::query(AssertSqlSafe(sql))
            .bind(&chain[0])
            .bind(&chain[1])
            .bind(&chain[2])
            .fetch_all(&mut *connection)
            .await
            .map_err(|_| Failure::Unavailable)?;
        if members.len() > 8 {
            return Err(Failure::InvalidStoredData);
        }
        for row in members {
            participants.push(rows::id(&row, "issuer_id")?.to_string());
            participants.push(rows::id(&row, "subject_id")?.to_string());
            grants.push(rows::id(&row, "grant_id")?.to_string());
        }
    }
    // Deduplicate only lock-selection keys. Sealed records/ordinals are never
    // deduplicated, reordered by grant, or reconstructed from mutable grants.
    participants.sort();
    participants.dedup();
    grants.sort();
    grants.dedup();
    let policy_sql = format!(
        "SELECT {} FROM arboresce_authority.workspace_policies \
        WHERE {TENANT} LIMIT 2",
        rows::tokens("source_policy_ids")
    );
    // The generated guard contains a compiled column name, never stored text.
    let policy_rows = sqlx::query(AssertSqlSafe(policy_sql))
        .bind(&tenant[0])
        .bind(&tenant[1])
        .fetch_all(&mut *connection)
        .await
        .map_err(|_| Failure::Unavailable)?;
    if policy_rows.len() > 1 {
        return Err(Failure::InvalidStoredData);
    }
    let mut sources = policy_rows
        .first()
        .map(|r| rows::set(r, "source_policy_ids"))
        .transpose()?
        .unwrap_or_default();
    sources.sort();
    sources.dedup();
    for id in &sources {
        lock(
            connection,
            "source_policies",
            "policy_id=$1",
            "policy_id",
            std::slice::from_ref(id),
            permission,
        )
        .await?;
    }
    lock(
        connection,
        "workspace_policies",
        TENANT,
        "organization_id,domain_id",
        &tenant,
        permission,
    )
    .await?;
    lock(
        connection,
        "organizations",
        "organization_id=$1::uuid",
        "organization_id",
        &tenant[..1],
        permission,
    )
    .await?;
    lock(
        connection,
        "domains",
        TENANT,
        "organization_id,domain_id",
        &tenant,
        permission,
    )
    .await?;
    for id in &participants {
        lock(
            connection,
            "principals",
            "principal_id=$1::uuid",
            "principal_id",
            std::slice::from_ref(id),
            permission,
        )
        .await?;
    }
    for id in &participants {
        lock(
            connection,
            "memberships",
            "organization_id=$1::uuid AND principal_id=$2::uuid",
            "organization_id,principal_id",
            &[tenant[0].clone(), id.clone()],
            permission,
        )
        .await?;
    }
    if request.selected_chain.is_some() {
        let predicate = format!("{TENANT} AND chain_id=$3::uuid");
        lock(
            connection,
            "immutable_chains",
            &predicate,
            "chain_id",
            &chain,
            permission,
        )
        .await?;
        lock(
            connection,
            "immutable_members",
            &predicate,
            "ordinal",
            &chain,
            permission,
        )
        .await?;
        for id in &grants {
            lock(
                connection,
                "grants",
                &format!("{TENANT} AND grant_id=$3::uuid"),
                "grant_id",
                &[tenant[0].clone(), tenant[1].clone(), id.clone()],
                permission,
            )
            .await?;
        }
    }
    let mut target = tenant.clone();
    target.push(rows::resource_name(request.resource.kind()).to_owned());
    target.push(rows::reference_id(request.resource)?.to_string());
    let target_predicate = format!("{TENANT} AND resource_kind=$3 AND resource_id=$4::uuid");
    lock(
        connection,
        "resources",
        &target_predicate,
        "organization_id,domain_id,resource_kind,resource_id",
        &target,
        permission || exclusive_target,
    )
    .await?;

    // Every authority value below comes from a new post-lock statement. Parent
    // SHARE/UPDATE remains held through the caller's positive terminal outcome.
    projection(
        connection,
        Keys {
            tenant,
            chain,
            target,
            participants,
            grants,
            sources,
            has_chain: request.selected_chain.is_some(),
        },
        clock,
    )
    .await
}

async fn projection(
    connection: &mut PgConnection,
    keys: Keys,
    clock: &mut impl Clock,
) -> Result<CanonicalProjection, Failure> {
    let mut budget = rows::RowBudget::new();
    budget.take(1)?; // The independently acquired namespace row is also counted.
    let mut principals = Vec::new();
    for id in &keys.participants {
        for row in fetch(
            connection,
            "principals",
            &principal_columns(),
            "principal_id=$1::uuid",
            "principal_id",
            std::slice::from_ref(id),
            &mut budget,
        )
        .await?
        {
            principals.push(decode_principal(&row)?);
        }
    }
    let organizations = fetch(
        connection,
        "organizations",
        &format!("organization_id::text,{}", rows::token("state")),
        "organization_id=$1::uuid",
        "organization_id",
        &keys.tenant[..1],
        &mut budget,
    )
    .await?
    .iter()
    .map(|row| {
        Ok(OrganizationRecord {
            id: rows::id(row, "organization_id")?,
            state: rows::state(row)?,
        })
    })
    .collect::<Result<Vec<_>, Failure>>()?;
    let domains = fetch(
        connection,
        "domains",
        &format!("{ID_COLUMNS},{}", rows::token("state")),
        TENANT,
        "organization_id,domain_id",
        &keys.tenant,
        &mut budget,
    )
    .await?
    .iter()
    .map(|row| {
        Ok(DomainRecord {
            workspace: rows::workspace(row)?,
            state: rows::state(row)?,
        })
    })
    .collect::<Result<Vec<_>, Failure>>()?;
    let membership_columns = membership_columns();
    let mut memberships = Vec::new();
    for id in &keys.participants {
        for row in fetch(
            connection,
            "memberships",
            &membership_columns,
            "organization_id=$1::uuid AND principal_id=$2::uuid",
            "organization_id,principal_id",
            &[keys.tenant[0].clone(), id.clone()],
            &mut budget,
        )
        .await?
        {
            memberships.push(decode_membership(&row)?);
        }
    }
    let policy_columns = [
        format!("{ID_COLUMNS},epoch"),
        rows::token("policy"),
        rows::token("state"),
        rows::tokens("read_roles"),
        rows::tokens("direct_review_roles"),
        rows::tokens("purposes"),
        rows::tokens("grant_root_actions"),
        rows::tokens("source_policy_ids"),
    ]
    .join(",");
    let policies = fetch(
        connection,
        "workspace_policies",
        &policy_columns,
        TENANT,
        "organization_id,domain_id",
        &keys.tenant,
        &mut budget,
    )
    .await?
    .iter()
    .map(|row| {
        Ok(WorkspacePolicy {
            workspace: rows::workspace(row)?,
            policy: rows::text(row, "policy")?,
            epoch: rows::revision(row, "epoch")?,
            state: rows::state(row)?,
            read_roles: rows::set(row, "read_roles")?,
            direct_review_roles: rows::set(row, "direct_review_roles")?,
            purposes: rows::set(row, "purposes")?,
            grant_root_actions: rows::set(row, "grant_root_actions")?,
            source_policy_ids: rows::set(row, "source_policy_ids")?,
        })
    })
    .collect::<Result<Vec<_>, Failure>>()?;
    let source_columns = [
        rows::token("policy_id"),
        rows::token("state"),
        rows::instant("expires_at"),
        rows::audience(),
        rows::tokens("permitted_purposes"),
    ]
    .join(",");
    let mut source_policies = Vec::new();
    for id in &keys.sources {
        for row in fetch(
            connection,
            "source_policies",
            &source_columns,
            "policy_id=$1",
            "policy_id",
            std::slice::from_ref(id),
            &mut budget,
        )
        .await?
        {
            let permitted_principals = rows::set(&row, "permitted_principals")?
                .iter()
                .map(|v| {
                    v.parse::<Identifier>()
                        .map_err(|_| Failure::InvalidStoredData)
                })
                .collect::<Result<Vec<_>, _>>()?;
            source_policies.push(SourcePolicy {
                policy: rows::text(&row, "policy_id")?,
                state: rows::state(&row)?,
                expires_at: rows::time(&row, "expires_at")?,
                permitted_principals,
                permitted_purposes: rows::set(&row, "permitted_purposes")?,
            });
        }
    }
    let resource_columns = [
        format!("{ID_COLUMNS},resource_id::text,control_revision"),
        rows::token("resource_kind"),
        rows::token("state"),
    ]
    .join(",");
    let resources = fetch(
        connection,
        "resources",
        &resource_columns,
        &format!("{TENANT} AND resource_kind=$3 AND resource_id=$4::uuid"),
        "organization_id,domain_id,resource_kind,resource_id",
        &keys.target,
        &mut budget,
    )
    .await?
    .iter()
    .map(|row| {
        Ok(ResourceRecord {
            reference: rows::resource(row)?,
            revision: rows::revision(row, "control_revision")?,
            state: rows::state(row)?,
        })
    })
    .collect::<Result<Vec<_>, Failure>>()?;
    let mut grants = Vec::new();
    for id in &keys.grants {
        for row in fetch(
            connection,
            "grants",
            &grant_columns(),
            &format!("{TENANT} AND grant_id=$3::uuid"),
            "grant_id",
            &[keys.tenant[0].clone(), keys.tenant[1].clone(), id.clone()],
            &mut budget,
        )
        .await?
        {
            grants.push(decode_grant(&row)?);
        }
    }
    let mut immutable_chains = Vec::new();
    if keys.has_chain {
        let predicate = format!("{TENANT} AND chain_id=$3::uuid");
        let columns = format!("chain_id::text,{}", rows::token("scope_interpretation"));
        for row in fetch(
            connection,
            "immutable_chains",
            &columns,
            &predicate,
            "chain_id",
            &keys.chain,
            &mut budget,
        )
        .await?
        {
            let raw = fetch(
                connection,
                "immutable_members",
                &format!("ordinal,{}", grant_columns()),
                &predicate,
                "ordinal",
                &keys.chain,
                &mut budget,
            )
            .await?;
            let mut members = Vec::new();
            for (index, record) in raw.iter().enumerate() {
                let ordinal: i16 = record
                    .try_get("ordinal")
                    .map_err(|_| Failure::InvalidStoredData)?;
                if usize::try_from(ordinal).ok() != Some(index) {
                    return Err(Failure::InvalidStoredData);
                }
                members.push(decode_grant(record)?);
            }
            immutable_chains.push(ImmutableChain {
                id: rows::id(&row, "chain_id")?,
                scope_interpretation: rows::text(&row, "scope_interpretation")?,
                members,
            });
        }
    }
    // The trusted clock is sampled after every possible tuple wait and reread.
    let now = clock.now().map_err(|_| Failure::Unavailable)?;
    Ok(CanonicalProjection {
        now,
        organizations,
        domains,
        principals,
        memberships,
        policies,
        source_policies,
        resources,
        grants,
        immutable_chains,
    })
}

async fn lock(
    connection: &mut PgConnection,
    table: &'static str,
    predicate: &str,
    order: &str,
    parameters: &[String],
    exclusive: bool,
) -> Result<(), Failure> {
    let mode = if exclusive { "FOR UPDATE" } else { "FOR SHARE" };
    let sql = format!(
        "SELECT 1 FROM arboresce_authority.{table} WHERE {predicate} ORDER BY {order} LIMIT 33 {mode}"
    );
    // Compiled table/predicate/order plus fixed tuple mode; values stay bound.
    let mut query = sqlx::query(AssertSqlSafe(sql));
    for value in parameters {
        query = query.bind(value);
    }
    let result = query
        .fetch_all(connection)
        .await
        .map_err(|_| Failure::Unavailable)?;
    if result.len() > 32 {
        return Err(Failure::InvalidStoredData);
    }
    Ok(())
}
