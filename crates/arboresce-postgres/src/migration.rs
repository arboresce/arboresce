//! Forward transactional DDL guarded by the actual direct migration lock.

use sqlx::{
    Connection, PgConnection, Row, SqlSafeStr,
    migrate::{Migrate, MigrateError, Migration, MigrationType, Migrator},
};
use std::time::Duration;
use tokio::time::timeout;

use crate::Harness;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SchemaVersion {
    Initial,
    Annotation,
    TenantAuthority,
    CurrentAuthority,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MigrationFailure {
    Unavailable,
    HistoryIntegrity,
    MissingAppliedVersion,
    ChecksumMismatch,
    LockTimeout,
    Statement,
    Unknown,
}

pub(crate) fn selected(version: SchemaVersion) -> Vec<Migration> {
    let mut result = vec![Migration::new(
        1,
        "typed records".into(),
        MigrationType::Simple,
        include_str!("../migrations/1_records.sql").into_sql_str(),
        false,
    )];
    if matches!(
        version,
        SchemaVersion::Annotation
            | SchemaVersion::TenantAuthority
            | SchemaVersion::CurrentAuthority
    ) {
        result.push(Migration::new(
            2,
            "bounded annotation".into(),
            MigrationType::Simple,
            include_str!("../migrations/2_annotation.sql").into_sql_str(),
            false,
        ));
    }
    if matches!(
        version,
        SchemaVersion::TenantAuthority | SchemaVersion::CurrentAuthority
    ) {
        result.push(Migration::new(
            3,
            "tenant authority foundation".into(),
            MigrationType::Simple,
            include_str!("../migrations/3_tenant_authority.sql").into_sql_str(),
            false,
        ));
    }
    if version == SchemaVersion::CurrentAuthority {
        result.push(Migration::new(
            4,
            "current authority and isolated final fences".into(),
            MigrationType::Simple,
            include_str!("../migrations/4_current_authority.sql").into_sql_str(),
            false,
        ));
    }
    result
}

pub(crate) fn classify(error: MigrateError) -> MigrationFailure {
    match error {
        MigrateError::Execute(source) | MigrateError::ExecuteMigration(source, _) => {
            match source.as_database_error() {
                None => MigrationFailure::Unknown,
                Some(database) => {
                    if database.code().as_deref() == Some("55P03") {
                        MigrationFailure::LockTimeout
                    } else {
                        MigrationFailure::Statement
                    }
                }
            }
        }
        MigrateError::VersionMissing(_) => MigrationFailure::MissingAppliedVersion,
        MigrateError::VersionMismatch(_) => MigrationFailure::ChecksumMismatch,
        MigrateError::Dirty(_) => MigrationFailure::HistoryIntegrity,
        _ => MigrationFailure::Statement,
    }
}

fn history_result<T>(result: Result<T, sqlx::Error>) -> Result<T, MigrationFailure> {
    result.map_err(|_| MigrationFailure::HistoryIntegrity)
}

async fn prefix(
    connection: &mut PgConnection,
    migrations: &[Migration],
) -> Result<(), MigrationFailure> {
    let exists: Option<String> = history_result(
        sqlx::query_scalar("SELECT to_regclass('public._sqlx_migrations')::text")
            .fetch_one(&mut *connection)
            .await,
    )?;
    if exists.is_none() {
        return Ok(());
    }
    let history = history_result(sqlx::query(
        "SELECT version,success,checksum FROM public._sqlx_migrations ORDER BY version LIMIT $1",
    )
    .bind(if migrations.len() == 4 { 5_i64 } else { 4 })
    .fetch_all(&mut *connection)
    .await)?;
    for (index, row) in history.iter().enumerate() {
        let version = history_result(row.try_get::<i64, _>("version"))?;
        let Some(expected) = migrations.get(index) else {
            return Err(MigrationFailure::MissingAppliedVersion);
        };
        if version != expected.version || !history_result(row.try_get::<bool, _>("success"))? {
            return Err(MigrationFailure::HistoryIntegrity);
        }
        let checksum: Vec<u8> = history_result(row.try_get("checksum"))?;
        if checksum.as_slice() != expected.checksum.as_ref() {
            return Err(MigrationFailure::ChecksumMismatch);
        }
    }
    Ok(())
}

pub(crate) async fn apply(
    harness: &Harness,
    migrations: Vec<Migration>,
) -> Result<(), MigrationFailure> {
    let mut connection = harness
        .connect(true)
        .await
        .map_err(|_| MigrationFailure::Unavailable)?;
    let result = timeout(Duration::from_millis(15000), async {
        connection.lock().await.map_err(classify)?;
        // This read-only prefix check runs under the same actual advisory lock
        // before SQLx can create its ledger or replay any missing version.
        let result = match prefix(&mut connection, &migrations).await {
            Ok(()) => {
                let mut migrator = Migrator::with_migrations(migrations);
                // The immutable harness ledger has always been public-qualified.
                // This selects its initial location; it never renames live history.
                migrator.dangerous_set_table_name("public._sqlx_migrations");
                migrator.set_locking(false);
                migrator.run(&mut connection).await.map_err(classify)
            }
            Err(error) => Err(error),
        };
        let unlocked = connection.unlock().await;
        if unlocked.is_err() {
            Err(MigrationFailure::Unknown)
        } else {
            result
        }
    })
    .await
    .unwrap_or(Err(MigrationFailure::Unknown));
    // Closing this owned direct session also releases its advisory lock on error.
    let _ = timeout(Duration::from_millis(1000), connection.close()).await;
    result
}
