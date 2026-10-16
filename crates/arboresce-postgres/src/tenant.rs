//! Bounded tenant reads on one reused direct qualification connection.
//!
//! This foundation exposes no product writer, authentication decision, action
//! grant, disclosure receipt or revocation linearization guarantee.

use std::time::Duration;

use arboresce_application::policy::TenantContext;
use arboresce_domain::Identifier;
use sqlx::{Connection, PgConnection, Postgres, Transaction};
use tokio::time::timeout;

use crate::Harness;

const BUDGET: Duration = Duration::from_millis(2000);

/// A bounded failure without supplied values, SQL, paths or foreign identities.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TenantFailure {
    Unavailable,
    UnsafeRole,
    RetainedContext,
    NotFound,
    InvalidStoredData,
    UnknownCompletion,
}

/// The finite variant label is diagnostic text without an underlying SQL cause.
impl std::fmt::Display for TenantFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, formatter)
    }
}

impl std::error::Error for TenantFailure {}

/// A reused physical connection in the explicitly prepared isolated namespace.
///
/// A future product pool must qualify separately. This creates one connection
/// per session, never one per tenant transaction, and reads no ambient DSN.
pub struct TenantSession {
    pub(crate) connection: PgConnection,
}

/// A read transaction whose complete selected context is transaction-local.
///
/// The server's existing finite transaction timeout bounds its whole lifetime.
/// No provider, parser or streaming work belongs inside this transaction.
pub struct TenantTransaction<'a> {
    pub(crate) transaction: Transaction<'a, Postgres>,
    context: TenantContext,
}

impl Harness {
    pub async fn tenant_session(&self) -> Result<TenantSession, TenantFailure> {
        let mut connection = self
            .connect(false)
            .await
            .map_err(|_| TenantFailure::Unavailable)?;
        timeout(BUDGET, validate_role(&mut connection))
            .await
            .map_err(|_| TenantFailure::Unavailable)??;
        Ok(TenantSession { connection })
    }
}

/// Observe the actual server catalogue. Fixture role flags are never authority.
pub(crate) async fn validate_role(connection: &mut PgConnection) -> Result<(), TenantFailure> {
    let allowed: bool = sqlx::query_scalar(
        "SELECT current_user = 'harness_writer' \
         AND current_database() = 'arboresce_migration_harness' \
         AND current_setting('row_security') = 'on' \
         AND NOT (r.rolsuper OR r.rolbypassrls OR r.rolcreaterole OR r.rolcreatedb \
                  OR r.rolreplication) \
         AND NOT pg_has_role(current_user, 'harness_admin', 'MEMBER') \
         AND NOT pg_has_role(current_user, 'harness_migrator', 'MEMBER') \
         AND NOT has_schema_privilege(current_user, 'arboresce_authority', 'CREATE') \
         AND (SELECT count(*) = 4 AND coalesce(bool_and( \
             c.relrowsecurity AND c.relforcerowsecurity AND c.relowner <> r.oid \
             AND has_table_privilege(current_user, c.oid, 'SELECT') \
             AND NOT has_table_privilege(current_user, c.oid, 'INSERT,UPDATE,DELETE,TRUNCATE,TRIGGER') \
           ), false) \
           FROM pg_class c JOIN pg_namespace n ON n.oid = c.relnamespace \
           WHERE n.nspname = 'arboresce_authority' AND c.relkind = 'r' \
             AND c.relname IN ('organizations','principals','domains','memberships')) \
         FROM pg_roles r WHERE r.rolname = current_user",
    )
    .fetch_one(connection)
    .await
    .map_err(|_| TenantFailure::Unavailable)?;
    if !allowed {
        return Err(TenantFailure::UnsafeRole);
    }
    Ok(())
}

impl TenantSession {
    pub async fn begin(
        &mut self,
        context: TenantContext,
    ) -> Result<TenantTransaction<'_>, TenantFailure> {
        timeout(BUDGET, async {
            validate_role(&mut self.connection).await?;
            let retained: bool = sqlx::query_scalar(
                "SELECT nullif(current_setting('arboresce.organization_id',true),'') IS NOT NULL \
                  OR nullif(current_setting('arboresce.domain_id',true),'') IS NOT NULL \
                  OR nullif(current_setting('arboresce.principal_id',true),'') IS NOT NULL",
            )
            .fetch_one(&mut self.connection)
            .await
            .map_err(|_| TenantFailure::Unavailable)?;
            if retained {
                return Err(TenantFailure::RetainedContext);
            }
            let mut transaction = self
                .connection
                .begin()
                .await
                .map_err(|_| TenantFailure::Unavailable)?;
            sqlx::query("SET TRANSACTION ISOLATION LEVEL READ COMMITTED, READ ONLY")
                .execute(&mut *transaction)
                .await
                .map_err(|_| TenantFailure::Unavailable)?;
            sqlx::query(
                "SELECT set_config('arboresce.organization_id',$1,true), \
                 set_config('arboresce.domain_id',$2,true), \
                 set_config('arboresce.principal_id',$3,true)",
            )
            .bind(context.workspace().organization().to_string())
            .bind(context.workspace().domain().to_string())
            .bind(context.principal().to_string())
            .execute(&mut *transaction)
            .await
            .map_err(|_| TenantFailure::Unavailable)?;
            let mut result = TenantTransaction {
                transaction,
                context,
            };
            if result.current_membership().await?.is_none() {
                result.rollback().await?;
                return Err(TenantFailure::NotFound);
            }
            Ok(result)
        })
        .await
        .map_err(|_| TenantFailure::Unavailable)?
    }
}

impl TenantTransaction<'_> {
    /// Selected identity only; this value is not an action-authorization receipt.
    pub const fn context(&self) -> TenantContext {
        self.context
    }

    /// Recheck canonical active records under the current statement snapshot.
    ///
    /// Permission-writer locks and complete action/grant fences remain separate
    /// admission work. This read never authorizes a consequential operation.
    pub async fn current_membership(&mut self) -> Result<Option<Identifier>, TenantFailure> {
        let row: Option<String> = timeout(
            BUDGET,
            sqlx::query_scalar(
                "SELECT m.membership_id::text FROM arboresce_authority.memberships m \
             JOIN arboresce_authority.organizations o USING (organization_id) \
             JOIN arboresce_authority.domains d USING (organization_id,domain_id) \
             JOIN arboresce_authority.principals p USING (principal_id) \
             WHERE m.organization_id=$1::uuid AND m.domain_id=$2::uuid \
               AND m.principal_id=$3::uuid AND m.state='active' AND o.state='active' \
               AND d.state='active' AND p.state='active'",
            )
            .bind(self.context.workspace().organization().to_string())
            .bind(self.context.workspace().domain().to_string())
            .bind(self.context.principal().to_string())
            .fetch_optional(&mut *self.transaction),
        )
        .await
        .map_err(|_| TenantFailure::Unavailable)?
        .map_err(|_| TenantFailure::Unavailable)?;
        row.map(|id| id.parse().map_err(|_| TenantFailure::InvalidStoredData))
            .transpose()
    }

    pub async fn commit(self) -> Result<(), TenantFailure> {
        timeout(BUDGET, self.transaction.commit())
            .await
            .map_err(|_| TenantFailure::UnknownCompletion)?
            .map_err(|_| TenantFailure::UnknownCompletion)
    }

    pub async fn rollback(self) -> Result<(), TenantFailure> {
        timeout(BUDGET, self.transaction.rollback())
            .await
            .map_err(|_| TenantFailure::UnknownCompletion)?
            .map_err(|_| TenantFailure::UnknownCompletion)
    }
}
