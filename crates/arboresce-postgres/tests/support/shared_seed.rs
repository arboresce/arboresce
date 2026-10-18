//! Seed only literal inputs, then observe complete canonical SQL state.

use crate::{
    authority_service::{Result, Service, query_json, with_connection},
    current_authority,
    implementation::SchemaVersion,
};
use serde_json::{Value, json};
use sqlx::PgConnection;

#[derive(Debug)]
pub struct SeedMigrationFailure(pub crate::implementation::MigrationFailure);
impl std::fmt::Display for SeedMigrationFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "shared seed migration failed: {:?}", self.0)
    }
}
impl std::error::Error for SeedMigrationFailure {}

pub fn text<'a>(value: &'a Value, key: &str) -> Result<&'a str> {
    value[key]
        .as_str()
        .ok_or_else(|| "invalid shared text input".into())
}

pub fn number(value: &Value, key: &str) -> Result<i64> {
    value[key]
        .as_i64()
        .filter(|v| *v >= 0)
        .ok_or_else(|| "invalid shared integer input".into())
}

fn rows(value: &Value) -> Result<&Vec<Value>> {
    value
        .as_array()
        .filter(|v| v.len() <= 128)
        .ok_or_else(|| "shared relation input bound exceeded".into())
}

#[derive(Clone, Copy)]
struct ResourceBind([i64; 5]);

impl ResourceBind {
    fn prepare(value: &Value) -> Result<Self> {
        Ok(Self([
            number(value, "pending")?,
            number(value, "active")?,
            number(value, "storage_bytes")?,
            number(value, "memory_bytes")?,
            number(value, "cost_microusd")?,
        ]))
    }

    fn view(self) -> Value {
        let [pending, active, storage_bytes, memory_bytes, cost_microusd] = self.0;
        json!({"pending":pending,"active":active,"storage_bytes":storage_bytes,
            "memory_bytes":memory_bytes,"cost_microusd":cost_microusd})
    }
}

struct OrganizationBind {
    organization: arboresce_domain::Identifier,
    limits: ResourceBind,
    used: ResourceBind,
}

struct ReservationBind<'a> {
    organization: arboresce_domain::Identifier,
    reservation: arboresce_domain::Identifier,
    principal: arboresce_domain::Identifier,
    owner: &'a str,
    replica: &'a str,
    resources: ResourceBind,
    fence: i64,
    attempt_limit: i16,
    attempts_used: i16,
    total_deadline: arboresce_domain::UtcTimestamp,
    admitted_at: arboresce_domain::UtcTimestamp,
    last_observed_at: arboresce_domain::UtcTimestamp,
    expires_at: arboresce_domain::UtcTimestamp,
    state: &'a str,
    dispatch_outcome: Option<&'a str>,
    reservation_order: i64,
}

fn identifier(value: &Value, key: &str) -> Result<arboresce_domain::Identifier> {
    Ok(text(value, key)?.parse()?)
}

fn instant(value: &Value) -> Result<arboresce_domain::UtcTimestamp> {
    Ok(value
        .as_str()
        .ok_or("invalid shared seed timestamp")?
        .parse()?)
}

/// Every shared SQL bind is prepared before the existing canonical reset.
/// This does not duplicate the canonical snapshot's separate validation.
pub struct PreparedSeed<'a> {
    authority_owner: arboresce_application::policy::Workspace,
    global_limits: [i64; 2],
    global_used: [i64; 2],
    observed_at: arboresce_domain::UtcTimestamp,
    next_order: i64,
    organizations: Vec<OrganizationBind>,
    reservations: Vec<ReservationBind<'a>>,
}

impl<'a> PreparedSeed<'a> {
    pub fn new(common: &'a Value, ledger: &'a Value) -> Result<Self> {
        let owner = &common["authority_snapshot"]["grants"][0]["scope"];
        let authority_owner = arboresce_application::policy::Workspace::new(
            identifier(owner, "organization_id")?,
            identifier(owner, "domain_id")?,
        );
        let admitted_at = instant(&common["seed_admitted_at"])?;
        let observed_at = instant(&common["seed_last_observed_at"])?;
        let limits = ledger["organization_limits"]
            .as_object()
            .filter(|v| v.len() <= 128)
            .ok_or("invalid shared organization inputs")?;
        let mut organizations = Vec::with_capacity(limits.len());
        for (organization, limits) in limits {
            organizations.push(OrganizationBind {
                organization: organization.parse()?,
                limits: ResourceBind::prepare(limits)?,
                used: ResourceBind::prepare(&ledger["used"][organization])?,
            });
        }
        let source = rows(&ledger["reservations"])?;
        let next_order = match ledger.get("next_reservation_order") {
            Some(value) => value.as_i64().ok_or("invalid reservation ordinal")?,
            None => source.len() as i64 + 1, // The preceding row bound is 128.
        };
        let mut reservations = Vec::with_capacity(source.len());
        for (index, row) in source.iter().enumerate() {
            let state = text(row, "state")?;
            let attempts_used = match row.get("attempts_used") {
                Some(value) => i16::try_from(value.as_i64().ok_or("invalid attempts input")?)?,
                None => i16::from(state == "active"),
            };
            let dispatch_outcome = match row.get("dispatch_outcome") {
                None | Some(Value::Null) => None,
                Some(value) => Some(value.as_str().ok_or("invalid shared dispatch seed")?),
            };
            reservations.push(ReservationBind {
                organization: identifier(row, "organization_id")?,
                reservation: identifier(row, "reservation_id")?,
                principal: identifier(row, "principal_id")?,
                owner: text(row, "owner")?,
                replica: text(row, "replica")?,
                resources: ResourceBind::prepare(row)?,
                fence: number(row, "fence")?,
                attempt_limit: i16::try_from(number(row, "attempt_limit")?)?,
                attempts_used,
                total_deadline: instant(&row["total_deadline"])?,
                admitted_at: row
                    .get("admitted_at")
                    .map(instant)
                    .transpose()?
                    .unwrap_or(admitted_at),
                last_observed_at: row
                    .get("last_observed_at")
                    .map(instant)
                    .transpose()?
                    .unwrap_or(observed_at),
                expires_at: instant(&row["expires_at"])?,
                state,
                dispatch_outcome,
                reservation_order: index as i64 + 1, // The complete source is bounded to 128.
            });
        }
        Ok(Self {
            authority_owner,
            global_limits: [
                number(&ledger["global_limits"], "active")?,
                number(&ledger["global_limits"], "memory_bytes")?,
            ],
            global_used: [
                number(&ledger["global_used"], "active")?,
                number(&ledger["global_used"], "memory_bytes")?,
            ],
            observed_at,
            next_order,
            organizations,
            reservations,
        })
    }

    /// Project the same actual typed values consumed by seed(), not source JSON.
    pub fn observation(&self) -> Value {
        let mut ledger = json!({"organization_limits":{},"global_limits":{
            "active":self.global_limits[0],"memory_bytes":self.global_limits[1]},
            "used":{},"global_used":{"active":self.global_used[0],"memory_bytes":self.global_used[1]},
            "next_reservation_order":self.next_order,"reservations":[]});
        for row in &self.organizations {
            let organization = row.organization.to_string();
            ledger["organization_limits"][&organization] = row.limits.view();
            ledger["used"][&organization] = row.used.view();
        }
        let leases: Vec<_> = self.reservations.iter().map(|row| {
            let [pending, active, storage_bytes, memory_bytes, cost_microusd] = row.resources.0;
            json!({"organization_id":row.organization.to_string(),"reservation_id":row.reservation.to_string(),
                    "principal_id":row.principal.to_string(),"owner":row.owner,"replica":row.replica,
                    "pending":pending,"active":active,"storage_bytes":storage_bytes,
                    "memory_bytes":memory_bytes,"cost_microusd":cost_microusd,
                    "fence":row.fence,"attempt_limit":row.attempt_limit,"attempts_used":row.attempts_used,
                    "total_deadline":row.total_deadline.to_string(),"admitted_at":row.admitted_at.to_string(),
                    "last_observed_at":row.last_observed_at.to_string(),"expires_at":row.expires_at.to_string(),
                    "state":row.state,"dispatch_outcome":row.dispatch_outcome,"reservation_order":row.reservation_order})
        }).collect();
        ledger["reservations"] = json!(leases);
        json!({"prepared":true,"authority_owner":{"organization_id":self.authority_owner.organization().to_string(),
            "domain_id":self.authority_owner.domain().to_string()},"ledger":ledger})
    }
}

pub async fn seed(service: &Service, common: &Value, ledger: &Value) -> Result<Value> {
    let prepared = PreparedSeed::new(common, ledger)?;
    let recipe = json!({"controlled_constraint_faults":[], "authority_owner":{
        "organization_id":prepared.authority_owner.organization().to_string(),
        "domain_id":prepared.authority_owner.domain().to_string()}});
    let canonical_seed =
        current_authority::seed(service, &common["authority_snapshot"], &recipe).await?;
    service
        .harness
        .migrate(SchemaVersion::SharedAdmission)
        .await
        .map_err(|error| {
            Box::new(SeedMigrationFailure(error)) as Box<dyn std::error::Error + Send + Sync>
        })?;
    service.with_admin(false, async |connection| {
        sqlx::query("INSERT INTO arboresce_admission.global_budget \
            (singleton,active_limit,memory_limit,active_used,memory_used,last_observed_at,next_reservation_order) \
            VALUES (1,$1,$2,$3,$4,$5::timestamptz,$6)")
            .bind(prepared.global_limits[0]).bind(prepared.global_limits[1])
            .bind(prepared.global_used[0]).bind(prepared.global_used[1])
            .bind(prepared.observed_at.to_string()).bind(prepared.next_order)
            .execute(&mut *connection).await?;
        for row in &prepared.organizations {
            sqlx::query("INSERT INTO arboresce_admission.organization_budgets \
                (organization_id,pending_limit,active_limit,storage_limit,memory_limit,cost_limit, \
                pending_used,active_used,storage_used,memory_used,cost_used) \
                VALUES ($1::uuid,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)")
                .bind(row.organization.to_string())
                .bind(row.limits.0[0]).bind(row.limits.0[1]).bind(row.limits.0[2]).bind(row.limits.0[3]).bind(row.limits.0[4])
                .bind(row.used.0[0]).bind(row.used.0[1]).bind(row.used.0[2]).bind(row.used.0[3]).bind(row.used.0[4])
                .execute(&mut *connection).await?;
        }
        for row in &prepared.reservations {
            sqlx::query("INSERT INTO arboresce_admission.reservation_recognition \
                (organization_id,reservation_id,principal_id,owner,replica,fence,attempt_limit,attempts_used, \
                 total_deadline,admitted_at,last_observed_at,state,reservation_order) \
                 VALUES ($1::uuid,$2::uuid,$3::uuid,$4,$5,$6,$7,$8,$9::timestamptz,$10::timestamptz,$11::timestamptz,$12,$13)")
                .bind(row.organization.to_string()).bind(row.reservation.to_string()).bind(row.principal.to_string())
                .bind(row.owner).bind(row.replica).bind(row.fence).bind(row.attempt_limit).bind(row.attempts_used)
                .bind(row.total_deadline.to_string()).bind(row.admitted_at.to_string()).bind(row.last_observed_at.to_string())
                .bind(row.state).bind(row.reservation_order).execute(&mut *connection).await?;
            sqlx::query("INSERT INTO arboresce_admission.reservations \
                (organization_id,reservation_id,principal_id,replica,pending,active,storage_bytes,memory_bytes, \
                 cost_microusd,owner,fence,admitted_at,expires_at,attempt_limit,total_deadline,state,dispatch_outcome) \
                 VALUES ($1::uuid,$2::uuid,$3::uuid,$4,$5,$6,$7,$8,$9,$10,$11,$12::timestamptz,$13::timestamptz,$14,$15::timestamptz,$16,$17)")
                .bind(row.organization.to_string()).bind(row.reservation.to_string()).bind(row.principal.to_string())
                .bind(row.replica).bind(row.resources.0[0]).bind(row.resources.0[1]).bind(row.resources.0[2])
                .bind(row.resources.0[3]).bind(row.resources.0[4]).bind(row.owner).bind(row.fence)
                .bind(row.admitted_at.to_string()).bind(row.expires_at.to_string()).bind(row.attempt_limit)
                .bind(row.total_deadline.to_string()).bind(row.state).bind(row.dispatch_outcome)
                .execute(&mut *connection).await?;
        }
        Ok(())
    }).await?;
    Ok(canonical_seed)
}

const RELATIONS: [(&str, &str, &str); 4] = [
    (
        "global_budget",
        "SELECT g.*,singleton AS observation_order FROM arboresce_admission.global_budget g",
        "last_observed_at",
    ),
    (
        "organization_budgets",
        "SELECT b.*,organization_id AS observation_order FROM arboresce_admission.organization_budgets b",
        "",
    ),
    (
        "reservation_recognition",
        "SELECT r.*,reservation_order AS observation_order FROM arboresce_admission.reservation_recognition r",
        "total_deadline,admitted_at,last_observed_at",
    ),
    (
        "reservations",
        "SELECT l.*,r.reservation_order AS observation_order FROM arboresce_admission.reservations l JOIN arboresce_admission.reservation_recognition r USING (organization_id,reservation_id)",
        "total_deadline,admitted_at,expires_at",
    ),
];

pub async fn storage(service: &Service) -> Result<Value> {
    with_connection(service.admin(false).await?, async |connection| {
        let mut result = json!({});
        for (name, projection, timestamps) in RELATIONS {
            let mut value = "to_jsonb(r)-'observation_order'".to_owned();
            for column in timestamps.split(',').filter(|v| !v.is_empty()) {
                value.push_str(&format!(" || jsonb_build_object('{column}',to_char(r.{column} AT TIME ZONE 'UTC','YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'))"));
            }
            let sql = format!("SELECT coalesce(jsonb_agg({value} ORDER BY observation_order),'[]'::jsonb)::text FROM ({projection}) r");
            let observed = query_json(connection, sqlx::AssertSqlSafe(sql), &[], 2 << 20).await?;
            if observed.as_array().is_none_or(|rows|rows.len() > 128) {
                return Err("shared SQL relation bound exceeded".into());
            }
            result[name] = observed;
        }
        Ok(result)
    }).await
}

/// Project only actual SQL rows into the original public ledger representation.
pub fn ledger(storage: &Value) -> Result<Value> {
    let global = rows(&storage["global_budget"])?
        .first()
        .ok_or("missing global budget")?;
    let mut result = json!({"organization_limits":{},"global_limits":{
        "active":global["active_limit"],"memory_bytes":global["memory_limit"]},
        "used":{},"global_used":{"active":global["active_used"],"memory_bytes":global["memory_used"]},
        "reservations":[]});
    for row in rows(&storage["organization_budgets"])? {
        let organization = text(row, "organization_id")?;
        for (output, suffix) in [("organization_limits", "limit"), ("used", "used")] {
            result[output][organization] = json!({"pending":row[format!("pending_{suffix}")],
                "active":row[format!("active_{suffix}")],"storage_bytes":row[format!("storage_{suffix}")],
                "memory_bytes":row[format!("memory_{suffix}")],"cost_microusd":row[format!("cost_{suffix}")]});
        }
    }
    let mut reservations = Vec::new();
    for row in rows(&storage["reservations"])? {
        let mut row = row.clone();
        let object = row
            .as_object_mut()
            .ok_or("invalid reservation observation")?;
        object.remove("admitted_at");
        if object.get("dispatch_outcome").is_some_and(Value::is_null) {
            object.remove("dispatch_outcome");
        }
        reservations.push(row);
    }
    result["reservations"] = json!(reservations);
    Ok(result)
}

pub async fn authority(service: &Service, common: &Value) -> Result<Value> {
    let now = text(&common["authority_snapshot"], "now")?;
    let mut clock = current_authority::SuppliedClock::new(&json!(now));
    current_authority::snapshot(service, &mut clock, false).await
}

pub async fn history(connection: &mut PgConnection) -> Result<Value> {
    query_json(connection,"SELECT coalesce(jsonb_agg(version ORDER BY version),'[]'::jsonb)::text FROM public._sqlx_migrations",&[],4096).await
}

pub async fn complete_history(service: &Service) -> Result<Value> {
    service.json(false,"SELECT coalesce(jsonb_agg(to_jsonb(m) ORDER BY version),'[]'::jsonb)::text FROM public._sqlx_migrations m",&[],131072).await
}

/// Restart also conserves raw canonical columns omitted by the public projection,
/// including namespace epochs, lock guards and qualification markers.
pub async fn complete_authority(service: &Service) -> Result<Value> {
    service.with_admin(false, async |connection| {
        let mut result = json!({});
        for (table, order) in [
            ("namespace_guard", "singleton"),
            ("organizations", "organization_id"),
            ("domains", "organization_id,domain_id"),
            ("principals", "principal_id"),
            ("memberships", "organization_id,principal_id"),
            ("workspace_policies", "organization_id,domain_id"),
            ("source_policies", "policy_id"),
            ("resources", "organization_id,domain_id,resource_kind,resource_id"),
            ("party_aliases", "organization_id,domain_id,party_id"),
            ("grants", "grant_id"),
            ("immutable_chains", "organization_id,domain_id,chain_id"),
            ("immutable_members", "organization_id,domain_id,chain_id,ordinal"),
            ("qualification_markers", "organization_id,domain_id,marker_id"),
        ] {
            let sql = format!("SELECT coalesce(jsonb_agg(to_jsonb(t) ORDER BY {order}),'[]'::jsonb)::text FROM arboresce_authority.{table} t");
            let rows = query_json(connection, sqlx::AssertSqlSafe(sql), &[], 131072).await?;
            if rows.as_array().is_none_or(|rows| rows.len() > 128) {
                return Err("canonical restart relation bound exceeded".into());
            }
            result[table] = rows;
        }
        Ok(result)
    }).await
}
