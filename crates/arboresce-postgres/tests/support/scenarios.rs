//! Execute admitted core scenario inputs; literal expected results stay in tests.

use super::support::{Records, Service};
use crate::{Harness, MigrationFailure, SchemaVersion, migration};
use arboresce_application::{MigrationRecord, MigrationStore, WriteOutcome};
use serde_json::Value;
use sqlx::{
    SqlSafeStr,
    migrate::{Migrate, Migration, MigrationType},
};
use std::time::{Duration, Instant};
use tokio::{task::JoinSet, time::timeout_at};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub struct Locked {
    pub holder: i32,
    pub waiters: Vec<i32>,
    pub relations_before_release: (bool, bool),
    pub outcomes: Vec<std::result::Result<(), MigrationFailure>>,
    pub elapsed: Duration,
}

/// Observe the same real advisory lock, without a fourth control connection.
pub async fn locked(service: &Service, simultaneous: bool) -> Result<Locked> {
    let mut holder = service.admin(false).await?;
    let holder_pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&mut holder)
        .await?;
    let baseline: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM pg_stat_activity WHERE \
        datname=current_database() AND usename='harness_migrator' \
        AND application_name='arboresce_harness_migrator'",
    )
    .fetch_one(&mut holder)
    .await?;
    if baseline != 0 {
        return Err("migration worker baseline is occupied".into());
    }
    holder.lock().await?;
    let started = Instant::now();
    let barrier = started + Duration::from_secs(2);
    let mut workers = JoinSet::new();
    for _ in 0..if simultaneous { 2 } else { 1 } {
        let harness = Harness::new(service.socket()).map_err(|_| "invalid fixture endpoint")?;
        workers.spawn_local(async move {
            harness
                .migrate(if simultaneous {
                    SchemaVersion::Annotation
                } else {
                    SchemaVersion::Initial
                })
                .await
        });
    }
    let mut released = false;
    let result: Result<Locked> = async {
        let waiters = loop {
            let pids: Vec<i32> = timeout_at(
                barrier.into(),
                sqlx::query_scalar(
                    "SELECT DISTINCT w.pid FROM pg_locks w JOIN pg_locks h ON \
                h.locktype='advisory' AND h.granted AND h.pid=pg_backend_pid() \
                AND w.classid=h.classid AND w.objid=h.objid AND w.objsubid=h.objsubid \
                AND w.database IS NOT DISTINCT FROM h.database \
                JOIN pg_stat_activity a ON a.pid=w.pid WHERE w.locktype='advisory' \
                AND NOT w.granted AND a.datname=current_database() \
                AND a.usename='harness_migrator' \
                AND a.application_name='arboresce_harness_migrator' \
                AND a.wait_event_type='Lock' AND a.wait_event='advisory' ORDER BY w.pid",
                )
                .fetch_all(&mut holder),
            )
            .await??;
            if pids.len() == workers.len() {
                break pids;
            }
            timeout_at(
                barrier.into(),
                tokio::time::sleep(Duration::from_millis(10)),
            )
            .await?;
        };
        let mut relations_before_release = timeout_at(
            barrier.into(),
            sqlx::query_as(
                "SELECT to_regclass('public._sqlx_migrations') IS NOT NULL,\
            to_regclass('migration_harness.records') IS NOT NULL",
            )
            .fetch_one(&mut holder),
        )
        .await??;
        let end = started + Duration::from_secs(if simultaneous { 15 } else { 7 });
        if simultaneous {
            timeout_at(end.into(), holder.unlock()).await??;
            released = true;
        }
        let mut outcomes = Vec::new();
        while let Some(outcome) = timeout_at(end.into(), workers.join_next()).await? {
            outcomes.push(outcome?);
        }
        if !simultaneous {
            relations_before_release = timeout_at(
                end.into(),
                sqlx::query_as(
                    "SELECT to_regclass('public._sqlx_migrations') IS NOT NULL,\
                to_regclass('migration_harness.records') IS NOT NULL",
                )
                .fetch_one(&mut holder),
            )
            .await??;
        }
        Ok(Locked {
            holder: holder_pid,
            waiters,
            relations_before_release,
            outcomes,
            elapsed: started.elapsed(),
        })
    }
    .await;
    // All failure paths release the exact holder and join only these workers.
    workers.abort_all();
    let cleanup = Instant::now() + Duration::from_secs(1);
    let unlocked: Result<()> = if released {
        Ok(())
    } else {
        match timeout_at(cleanup.into(), holder.unlock()).await {
            Ok(result) => result.map_err(Into::into),
            Err(error) => Err(error.into()),
        }
    };
    drop(holder);
    let joined = timeout_at(cleanup.into(), async {
        while workers.join_next().await.is_some() {}
    })
    .await;
    unlocked?;
    joined?;
    result
}

#[derive(Default)]
pub struct Observed {
    pub migration: Option<std::result::Result<(), MigrationFailure>>,
    pub write: Option<WriteOutcome>,
    pub read: Option<MigrationRecord>,
}

pub async fn run(service: &Service, records: &Records, steps: &[Value]) -> Result<Observed> {
    let mut result = Observed::default();
    let mut writer = Some(Harness::new(service.socket()).map_err(|_| "invalid fixture endpoint")?);
    let mut unavailable = false;
    for raw in steps {
        let step = raw.as_str().ok_or("invalid scenario step")?;
        if let Some(version) = step.strip_prefix("migrate:") {
            let version = match version {
                "1" => SchemaVersion::Initial,
                "2" => SchemaVersion::Annotation,
                _ => return Err("unsupported schema fixture".into()),
            };
            result.migration = Some(service.harness.migrate(version).await);
        } else if let Some(name) = step.strip_prefix("insert:") {
            if !unavailable {
                service.writer_identity(true).await?;
            }
            result.write = Some(
                writer
                    .as_ref()
                    .ok_or("writer was closed")?
                    .insert(records.get(name)?)
                    .await,
            );
        } else if let Some(annotation) = step.strip_prefix("annotate:") {
            let (name, text) = annotation
                .split_once(':')
                .ok_or("invalid annotation step")?;
            service.annotate(records.get(name)?, text).await?;
        } else {
            match step {
                "read-original-columns" | "new-direct-reader" => {
                    let reader =
                        Harness::new(service.socket()).map_err(|_| "invalid fixture endpoint")?;
                    let record = records.get("exact-large")?;
                    result.read = reader
                        .read(record.organization(), record.record())
                        .await
                        .map_err(|_| "read failed")?;
                }
                "close-writer" => {
                    drop(writer.take());
                }
                "select-owned-absent-socket" => {
                    let socket = service.socket().join("absent");
                    if socket.try_exists()? {
                        return Err("absent socket fixture is occupied".into());
                    }
                    writer = Some(Harness::new(socket).map_err(|_| "invalid fixture endpoint")?);
                    unavailable = true;
                }
                "attempt-altered-version-1" => {
                    result.migration = Some(
                        migration::apply(
                            &service.harness,
                            vec![Migration::new(
                                1,
                                "altered initial".into(),
                                MigrationType::Simple,
                                "SELECT 1;".into_sql_str(),
                                false,
                            )],
                        )
                        .await,
                    );
                }
                "attempt-version-3-create-then-divide-by-zero" => {
                    let mut plan = migration::selected(SchemaVersion::Annotation);
                    plan.push(Migration::new(
                        3,
                        "atomic failing probe".into(),
                        MigrationType::Simple,
                        "CREATE TABLE migration_harness.failed_probe (value integer); SELECT 1/0;"
                            .into_sql_str(),
                        false,
                    ));
                    result.migration = Some(migration::apply(&service.harness, plan).await);
                }
                "admin-remove-only-version-1-receipt" => {
                    let mut control = service.admin(false).await?;
                    let deleted =
                        sqlx::query("DELETE FROM public._sqlx_migrations WHERE version=1")
                            .execute(&mut control)
                            .await?;
                    if deleted.rows_affected() != 1 {
                        return Err("history gap setup failed".into());
                    }
                }
                _ => return Err("scenario belongs to a separate service boundary".into()),
            }
        }
    }
    Ok(result)
}

pub async fn constraint(service: &Service, records: &Records, case: &Value) -> Result<String> {
    let record = records.get("exact-large")?;
    let mut values = [
        format!("'{}'::uuid", record.organization()),
        format!("'{}'::uuid", record.record()),
        record.revision().value().to_string(),
        format!("DATE '{}'", record.date()),
        "''".into(),
    ];
    let field = case["field"].as_str().ok_or("invalid constraint field")?;
    let index = [
        "organization_id",
        "record_id",
        "content_revision",
        "record_date",
        "annotation",
    ]
    .iter()
    .position(|name| *name == field)
    .ok_or("unsupported constraint field")?;
    values[index] = case["sql_value"]
        .as_str()
        .ok_or("invalid constraint literal")?
        .to_owned();
    // Only independently frozen self-authored qualification SQL literals enter
    // this statement. Product record writes above always use parameter binds.
    let sql = sqlx::AssertSqlSafe(format!(
        "INSERT INTO migration_harness.records \
        (organization_id,record_id,content_revision,record_date,annotation) VALUES ({})",
        values.join(",")
    ));
    let mut connection = service.admin(false).await?;
    match sqlx::query(sql).execute(&mut connection).await {
        Ok(_) => Ok("accepted".into()),
        Err(error) => Ok(error
            .as_database_error()
            .and_then(|e| e.code())
            .ok_or("constraint had no SQLSTATE")?
            .into_owned()),
    }
}
