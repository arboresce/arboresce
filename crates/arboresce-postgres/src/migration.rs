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
    if version == SchemaVersion::Annotation {
        result.push(Migration::new(
            2,
            "bounded annotation".into(),
            MigrationType::Simple,
            include_str!("../migrations/2_annotation.sql").into_sql_str(),
            false,
        ));
    }
    result
}

fn classify(error: MigrateError) -> MigrationFailure {
    if matches!(&error, MigrateError::Execute(source) | MigrateError::ExecuteMigration(source, _)
        if source.as_database_error().is_none())
    {
        return MigrationFailure::Unknown;
    }
    let code = match &error {
        MigrateError::Execute(error) | MigrateError::ExecuteMigration(error, _) => {
            error.as_database_error().and_then(|e| e.code())
        }
        _ => None,
    };
    if code.as_deref() == Some("55P03") {
        MigrationFailure::LockTimeout
    } else {
        match error {
            MigrateError::VersionMissing(_) => MigrationFailure::MissingAppliedVersion,
            MigrateError::VersionMismatch(_) => MigrationFailure::ChecksumMismatch,
            MigrateError::Dirty(_) => MigrationFailure::HistoryIntegrity,
            _ => MigrationFailure::Statement,
        }
    }
}

async fn prefix(
    connection: &mut PgConnection,
    migrations: &[Migration],
) -> Result<(), MigrationFailure> {
    let exists: Option<String> =
        sqlx::query_scalar("SELECT to_regclass('public._sqlx_migrations')::text")
            .fetch_one(&mut *connection)
            .await
            .map_err(|_| MigrationFailure::HistoryIntegrity)?;
    if exists.is_none() {
        return Ok(());
    }
    let history = sqlx::query(
        "SELECT version,success,checksum FROM public._sqlx_migrations ORDER BY version LIMIT 4",
    )
    .fetch_all(&mut *connection)
    .await
    .map_err(|_| MigrationFailure::HistoryIntegrity)?;
    for (index, row) in history.iter().enumerate() {
        let version = row
            .try_get::<i64, _>("version")
            .map_err(|_| MigrationFailure::HistoryIntegrity)?;
        let Some(expected) = migrations.get(index) else {
            return Err(MigrationFailure::MissingAppliedVersion);
        };
        if version != expected.version
            || !row
                .try_get::<bool, _>("success")
                .map_err(|_| MigrationFailure::HistoryIntegrity)?
        {
            return Err(MigrationFailure::HistoryIntegrity);
        }
        let checksum: Vec<u8> = row
            .try_get("checksum")
            .map_err(|_| MigrationFailure::HistoryIntegrity)?;
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
