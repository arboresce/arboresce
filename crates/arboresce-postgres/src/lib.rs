//! Direct isolated migration and transaction foundation; no product admission.

use std::{path::PathBuf, time::Duration};

use arboresce_application::{
    MigrationRecord, MigrationStore, ReadFailure, WriteFailure, WriteOutcome,
};
use arboresce_domain::Identifier;
use sqlx::{
    Connection, PgConnection, Row,
    postgres::{PgConnectOptions, PgSslMode},
};
use tokio::time::timeout;

pub(crate) mod migration;
pub use migration::{MigrationFailure, SchemaVersion};
pub(crate) mod tenant;
pub use tenant::{TenantFailure, TenantSession, TenantTransaction};
pub(crate) mod authority;
pub use authority::{
    AdmissionFence, AuthorityFailure, AuthoritySession, AuthoritySessions, PermissionChange,
    PermissionFence, PermissionSession,
};

/// An input-independent configuration rejection, without paths or credentials.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidConfiguration;

/// A direct connection target for the explicitly prepared synthetic harness.
///
/// This cannot select a product database, a remote transport or an ambient DSN.
pub struct Harness {
    pub(crate) socket: PathBuf,
}

impl Harness {
    pub fn new(socket: PathBuf) -> Result<Self, InvalidConfiguration> {
        use std::os::unix::ffi::OsStrExt;
        if !socket.is_absolute()
            || socket.as_os_str().as_bytes().len() > 80
            || socket.components().any(|c| {
                !matches!(
                    c,
                    std::path::Component::RootDir | std::path::Component::Normal(_)
                )
            })
            || socket.as_os_str().as_bytes().contains(&0)
            || std::env::vars_os().any(|(key, _)| key.as_bytes().starts_with(b"PG"))
        {
            return Err(InvalidConfiguration);
        }
        Ok(Self { socket })
    }

    pub(crate) fn options(
        &self,
        migration: bool,
    ) -> Result<PgConnectOptions, InvalidConfiguration> {
        use std::os::unix::ffi::OsStrExt;
        // The upstream constructor reads PG* defaults. Reject them before it runs;
        // do not read a password file or mutate this process's environment.
        if std::env::vars_os().any(|(key, _)| key.as_bytes().starts_with(b"PG")) {
            return Err(InvalidConfiguration);
        }
        Ok(PgConnectOptions::new_without_pgpass()
            .host("localhost")
            .socket(&self.socket)
            .port(5432)
            .username(if migration {
                "harness_migrator"
            } else {
                "harness_writer"
            })
            .password("")
            .database("arboresce_migration_harness")
            .ssl_mode(PgSslMode::Disable)
            .statement_cache_capacity(0)
            .application_name(if migration {
                "arboresce_harness_migrator"
            } else {
                "arboresce_harness_writer"
            })
            .options([
                ("search_path", "pg_catalog,public"),
                ("DateStyle", "ISO,YMD"),
                ("TimeZone", "UTC"),
                (
                    "statement_timeout",
                    if migration { "10000" } else { "2000" },
                ),
                (
                    "transaction_timeout",
                    if migration { "15000" } else { "2000" },
                ),
                ("lock_timeout", if migration { "5000" } else { "500" }),
            ]))
    }

    pub(crate) async fn connect(&self, migration: bool) -> Result<PgConnection, ()> {
        let options = self.options(migration).map_err(|_| ())?;
        timeout(
            Duration::from_millis(1000),
            PgConnection::connect_with(&options),
        )
        .await
        .map_err(|_| ())?
        .map_err(|_| ())
    }

    /// Apply only the selected forward harness migrations, with strict history.
    pub async fn migrate(&self, version: SchemaVersion) -> Result<(), MigrationFailure> {
        migration::apply(self, migration::selected(version)).await
    }
}

impl MigrationStore for Harness {
    async fn insert(&self, record: MigrationRecord) -> WriteOutcome {
        let Ok(mut connection) = self.connect(false).await else {
            return WriteOutcome::NotCommitted(WriteFailure::Unavailable);
        };
        let Ok(Ok(mut transaction)) =
            timeout(Duration::from_millis(2000), connection.begin()).await
        else {
            return WriteOutcome::NotCommitted(WriteFailure::Unavailable);
        };
        let result = timeout(Duration::from_millis(2000), async {
            let statement = sqlx::query(
                "INSERT INTO migration_harness.records \
                (organization_id,record_id,content_revision,record_date) \
                VALUES ($1::uuid,$2::uuid,$3,$4::date)",
            )
            .bind(record.organization().to_string())
            .bind(record.record().to_string())
            .bind(record.revision().value() as i64)
            .bind(record.date().to_string())
            .execute(&mut *transaction)
            .await;
            match statement {
                Ok(_) => match transaction.commit().await {
                    Ok(()) => WriteOutcome::Committed(record),
                    Err(_) => WriteOutcome::Unknown,
                },
                Err(error) => {
                    let failure = if error.as_database_error().and_then(|e| e.code()).as_deref()
                        == Some("23505")
                    {
                        WriteFailure::Conflict
                    } else {
                        WriteFailure::Rejected
                    };
                    match transaction.rollback().await {
                        Ok(()) => WriteOutcome::NotCommitted(failure),
                        Err(_) => WriteOutcome::Unknown,
                    }
                }
            }
        })
        .await;
        result.unwrap_or(WriteOutcome::Unknown)
    }

    async fn read(
        &self,
        organization: Identifier,
        record: Identifier,
    ) -> Result<Option<MigrationRecord>, ReadFailure> {
        let mut connection = self
            .connect(false)
            .await
            .map_err(|_| ReadFailure::Unavailable)?;
        let row = timeout(Duration::from_millis(2000),
            sqlx::query("SELECT organization_id::text,record_id::text,content_revision::text,record_date::text \
                FROM migration_harness.records WHERE organization_id=$1::uuid AND record_id=$2::uuid")
                .bind(organization.to_string()).bind(record.to_string()).fetch_optional(&mut connection))
            .await.map_err(|_| ReadFailure::Unavailable)?
            .map_err(|_| ReadFailure::Unavailable)?;
        row.map(|row| {
            let field = |name| {
                row.try_get::<String, _>(name)
                    .map_err(|_| ReadFailure::InvalidStoredData)
            };
            Ok(MigrationRecord::new(
                field("organization_id")?
                    .parse()
                    .map_err(|_| ReadFailure::InvalidStoredData)?,
                field("record_id")?
                    .parse()
                    .map_err(|_| ReadFailure::InvalidStoredData)?,
                field("content_revision")?
                    .parse()
                    .map_err(|_| ReadFailure::InvalidStoredData)?,
                field("record_date")?
                    .parse()
                    .map_err(|_| ReadFailure::InvalidStoredData)?,
            ))
        })
        .transpose()
    }
}
