//! Exact owned service preparation for cohorts that never restart PostgreSQL.

use crate::implementation::Harness;
use arboresce_xtask::collection_services;
use serde_json::{Value, json};
use sqlx::{Connection, PgConnection};
use std::{
    fs::Metadata,
    io::Read,
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
    time::Duration,
};
use tokio::time::{Instant, timeout, timeout_at};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Debug)]
pub struct OperationAndCleanupFailure {
    pub operation: Box<dyn std::error::Error + Send + Sync>,
    pub cleanup: Box<dyn std::error::Error + Send + Sync>,
}

impl std::fmt::Display for OperationAndCleanupFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "operation failed: {}; owned cleanup failed: {}",
            self.operation, self.cleanup
        )
    }
}

impl std::error::Error for OperationAndCleanupFailure {}

/// Keep the actual operation error (including its type) when cleanup succeeds.
/// A second cleanup failure must not erase the original operation failure.
pub fn preserve_cleanup<T>(operation: Result<T>, cleanup: Result<()>) -> Result<T> {
    match (operation, cleanup) {
        (operation, Ok(())) => operation,
        (Ok(_), Err(cleanup)) => Err(cleanup),
        (Err(operation), Err(cleanup)) => {
            Err(Box::new(OperationAndCleanupFailure { operation, cleanup }))
        }
    }
}

pub async fn with_connection<T>(
    mut connection: PgConnection,
    operation: impl AsyncFnOnce(&mut PgConnection) -> Result<T>,
) -> Result<T> {
    let result = operation(&mut connection).await;
    let cleanup = match timeout(Duration::from_secs(1), connection.close()).await {
        Ok(result) => result.map_err(Into::into),
        Err(error) => Err(error.into()),
    };
    preserve_cleanup(result, cleanup)
}

/// Ordinary raw probes already own one transaction and require rollback.
/// An operation error never bypasses that awaited terminal or erases its error.
/// This does not own AuthoritySession, fences or ambiguous completions.
pub async fn with_rollback<T>(
    connection: &mut PgConnection,
    operation: impl AsyncFnOnce(&mut PgConnection) -> Result<T>,
) -> Result<T> {
    let mut transaction = connection.begin().await?;
    let result = operation(&mut transaction).await;
    preserve_cleanup(result, transaction.rollback().await.map_err(Into::into))
}

/// One bounded JSON query implementation for actual control observations.
/// Callers supply compiled SQL fragments and bind every variable value.
pub async fn query_json(
    connection: &mut PgConnection,
    sql: impl sqlx::SqlSafeStr,
    values: &[String],
    bound: usize,
) -> Result<Value> {
    let mut query = sqlx::query_scalar::<_, String>(sql);
    for value in values {
        query = query.bind(value);
    }
    fetch_json(connection, query, bound).await
}

/// Preserve the parameter types selected by the caller's original bind calls.
/// Only bounded result fetching and JSON decoding are shared here.
pub async fn fetch_json(
    connection: &mut PgConnection,
    query: sqlx::query::QueryScalar<'_, sqlx::Postgres, String, sqlx::postgres::PgArguments>,
    bound: usize,
) -> Result<Value> {
    let raw = query.fetch_one(connection).await?;
    if raw.len() > bound {
        return Err("owned JSON observation bound exceeded".into());
    }
    Ok(serde_json::from_str(&raw)?)
}

#[derive(Debug)]
pub struct DatabaseCensusFailure {
    pub observed_backends: i64,
    pub timeout: Option<tokio::time::error::Elapsed>,
}

impl std::fmt::Display for DatabaseCensusFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "owned database still has live sessions: observed={}, timeout={:?}",
            self.observed_backends, self.timeout
        )
    }
}

impl std::error::Error for DatabaseCensusFailure {}

pub struct Service {
    pub harness: Harness,
    record: Value,
    data: PathBuf,
    server_pid: u64,
}

#[derive(Clone, Copy)]
enum GuardLayout {
    Directory,
    Symlink,
    Hardlink,
    Regular,
    DistinctFiles,
}

enum Statement {
    Query(&'static str),
    Batch(&'static str),
}

/// The native query and representation tests share this exact comparison.
/// Representation tests do not establish a server's actual identity.
pub fn profile(actual: &Value, record: &Value) -> Result<()> {
    for field in [
        "system_identifier",
        "postmaster_start",
        "server_version_num",
        "data",
        "socket",
    ] {
        if actual[field] != record[field] {
            return Err("owned PostgreSQL identity changed".into());
        }
    }
    if actual["listen"] != ""
        || actual["connections"] != 16
        || actual["encoding"] != "UTF8"
        || actual["fsync"] != "on"
        || actual["synchronous_commit"] != "on"
        || !actual["address"].is_null()
    {
        return Err("owned PostgreSQL profile changed".into());
    }
    Ok(())
}

/// Compare actual metadata observations; a distinct-file test is not a race.
pub fn pid_metadata(metadata: &Metadata, opened: &Metadata) -> Result<()> {
    if metadata.dev() != opened.dev() || metadata.ino() != opened.ino() {
        return Err("owned PID file changed".into());
    }
    Ok(())
}

pub fn pid_file(path: &Path, expected_pid: u64) -> Result<()> {
    let metadata = std::fs::symlink_metadata(path)?;
    if !metadata.is_file() || metadata.len() > 4096 || metadata.nlink() != 1 {
        return Err("invalid owned PID file".into());
    }
    let file = std::fs::File::open(path)?;
    pid_metadata(&metadata, &file.metadata()?)?;
    let mut pid = String::new();
    file.take(4097).read_to_string(&mut pid)?;
    if pid.len() > 4096
        || pid.lines().next().and_then(|v| v.parse::<u64>().ok()) != Some(expected_pid)
    {
        return Err("owned PostgreSQL PID changed".into());
    }
    Ok(())
}

/// Input-only preparation and evaluation for representation/owned-file guards.
/// The caller owns independent assertions; this never targets a server PID file.
pub fn helper_guard(input: &Value) -> Result<Value> {
    use std::{
        fs,
        os::unix::fs::symlink,
        sync::atomic::{AtomicU64, Ordering},
    };
    static NEXT_GUARD: AtomicU64 = AtomicU64::new(0);

    let result = match input["operation"].as_str() {
        Some("profile_guard") => profile(&input["observation"], &input["record"]),
        Some("pid_file_guard") => {
            // Validate all constructor inputs before touching the owned tree.
            let layout = match input["layout"].as_str() {
                Some("directory") => GuardLayout::Directory,
                Some("symlink") => GuardLayout::Symlink,
                Some("hardlink_pair") => GuardLayout::Hardlink,
                Some("regular") => GuardLayout::Regular,
                Some("two_distinct_regular_files") => GuardLayout::DistinctFiles,
                _ => return Err("unadmitted helper layout".into()),
            };
            let expected_pid = input["expected_pid"]
                .as_u64()
                .ok_or("missing fixture PID")?;
            let contents = if let Some(bytes) = input["bytes"].as_u64() {
                let fill = input["fill"].as_str().ok_or("missing guard fill")?;
                if bytes > 4097 || fill.len() != 1 {
                    return Err("guard fixture fill exceeds its bound".into());
                }
                fill.repeat(bytes as usize)
            } else {
                let contents = input["contents"].as_str().unwrap_or("");
                if contents.len() > 4097 {
                    return Err("guard fixture contents exceed their bound".into());
                }
                contents.to_owned()
            };
            let parent = Path::new(env!("CARGO_TARGET_TMPDIR"));
            fs::create_dir_all(parent)?;
            let owned = parent.join(format!(
                "authority-helper-guard-{}-{}",
                std::process::id(),
                NEXT_GUARD.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&owned)?;
            let path = owned.join("postmaster.pid");
            let result = (|| -> Result<()> {
                match layout {
                    GuardLayout::Directory => fs::create_dir(&path)?,
                    GuardLayout::Symlink => {
                        fs::write(owned.join("pid-target"), &contents)?;
                        symlink("pid-target", &path)?;
                    }
                    GuardLayout::Hardlink => {
                        fs::write(&path, &contents)?;
                        fs::hard_link(&path, owned.join("pid-alias"))?;
                    }
                    GuardLayout::Regular | GuardLayout::DistinctFiles => {
                        fs::write(&path, &contents)?
                    }
                }
                if matches!(layout, GuardLayout::DistinctFiles) {
                    let other = owned.join("other.pid");
                    fs::write(&other, &contents)?;
                    pid_metadata(
                        &fs::symlink_metadata(&path)?,
                        &fs::File::open(other)?.metadata()?,
                    )
                } else {
                    pid_file(&path, expected_pid)
                }
            })();
            // Only disposable literal inputs in this exclusively created
            // directory are removed. Guard outcomes remain recorded by callers.
            preserve_cleanup(result, fs::remove_dir_all(owned).map_err(Into::into))
        }
        _ => return Err("unadmitted helper guard".into()),
    };
    Ok(json!({"accepted":result.is_ok(),"failure":result.err().map(|e|e.to_string())}))
}

impl Service {
    pub fn from_record(record: Value) -> Result<Self> {
        let (socket, data, server_pid): (PathBuf, PathBuf, u64) = serde_json::from_value(json!([
            record["socket"],
            record["data"],
            record["server_pid"]
        ]))?;
        let harness = Harness::new(socket).map_err(|_| "invalid owned harness target")?;
        Ok(Self {
            harness,
            record,
            data,
            server_pid,
        })
    }

    pub fn record(&self) -> &Value {
        &self.record
    }

    pub fn data(&self) -> &Path {
        &self.data
    }

    /// Run an operation on one actual admin session and positively close it,
    /// including on an operation error. Preserve both errors if closing fails.
    pub async fn with_admin<T>(
        &self,
        control: bool,
        operation: impl AsyncFnOnce(&mut PgConnection) -> Result<T>,
    ) -> Result<T> {
        with_connection(self.admin(control).await?, operation).await
    }

    pub async fn json(
        &self,
        control: bool,
        sql: impl sqlx::SqlSafeStr,
        values: &[String],
        bound: usize,
    ) -> Result<Value> {
        self.with_admin(control, async |connection| {
            query_json(connection, sql, values, bound).await
        })
        .await
    }

    async fn statements(&self, control: bool, statements: &[Statement]) -> Result<()> {
        self.with_admin(control, async |connection| {
            for statement in statements {
                match statement {
                    Statement::Query(sql) => sqlx::query(*sql).execute(&mut *connection).await,
                    Statement::Batch(sql) => sqlx::raw_sql(*sql).execute(&mut *connection).await,
                }?;
            }
            Ok(())
        })
        .await
    }

    pub async fn open() -> Result<Self> {
        let input = PathBuf::from(
            std::env::var_os(collection_services::SELECTOR)
                .ok_or("required isolated PostgreSQL input is absent")?,
        );
        let bound = collection_services::observe(
            &input,
            std::time::Instant::now() + Duration::from_secs(30),
        )?;
        let service = Self::from_record(bound["record"].clone())?;
        service.identity().await?;
        service.empty_database().await?;
        service.require_fresh_roles().await?;
        service.statements(true, &[Statement::Batch("CREATE ROLE harness_authority_admission LOGIN NOINHERIT NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS CONNECTION LIMIT 2;
            CREATE ROLE harness_authority_permission LOGIN NOINHERIT NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS CONNECTION LIMIT 2;
            ALTER ROLE harness_authority_admission SET statement_timeout='2000ms';
            ALTER ROLE harness_authority_admission SET transaction_timeout='2000ms';
            ALTER ROLE harness_authority_admission SET lock_timeout='500ms';
            ALTER ROLE harness_authority_permission SET statement_timeout='2000ms';
            ALTER ROLE harness_authority_permission SET transaction_timeout='2000ms';
            ALTER ROLE harness_authority_permission SET lock_timeout='500ms';")]).await?;
        Ok(service)
    }

    pub async fn require_fresh_roles(&self) -> Result<()> {
        self.with_admin(true, async |connection| {
        let existing:i64=sqlx::query_scalar("SELECT count(*) FROM pg_roles WHERE rolname IN ('harness_authority_admission','harness_authority_permission')")
            .fetch_one(connection).await?;
        if existing != 0 {
            return Err("new cohort roles already exist; no adoption".into());
        }
        Ok(())
        }).await
    }

    pub async fn admin(&self, control: bool) -> Result<PgConnection> {
        let options = self
            .harness
            .options(true)
            .map_err(|_| "invalid owned configuration")?
            .username("harness_admin")
            .database(if control {
                "postgres"
            } else {
                "arboresce_migration_harness"
            })
            .application_name("arboresce_authority_control");
        Ok(timeout(Duration::from_secs(1), PgConnection::connect_with(&options)).await??)
    }

    pub async fn identity(&self) -> Result<()> {
        self.with_admin(true, async |connection| {
        let actual=timeout(Duration::from_secs(2),query_json(connection,
            "SELECT json_build_object('system_identifier',(SELECT system_identifier::text FROM pg_control_system()),\
             'postmaster_start',pg_postmaster_start_time()::text,'server_version_num',current_setting('server_version_num')::integer,\
             'data',current_setting('data_directory'),'socket',current_setting('unix_socket_directories'),\
             'listen',current_setting('listen_addresses'),'connections',current_setting('max_connections')::integer,\
             'encoding',current_setting('server_encoding'),'fsync',current_setting('fsync'),\
             'synchronous_commit',current_setting('synchronous_commit'),'address',inet_server_addr())::text", &[], 16384)).await??;
        profile(&actual, &self.record)?;
        pid_file(&self.data.join("postmaster.pid"), self.server_pid)
        }).await
    }

    /// Actual zero-backend census, not a delay or permission to terminate peers.
    async fn empty_database(&self) -> Result<()> {
        let end = Instant::now() + Duration::from_secs(1);
        self.with_admin(true, async |connection| -> Result<()> {
        let mut observed_backends = 0;
        loop {
            let count = match timeout_at(end,sqlx::query_scalar::<_, i64>(
                "SELECT count(*) FROM pg_stat_activity WHERE datname='arboresce_migration_harness'")
                .fetch_one(&mut *connection)).await {
                Ok(count) => count?,
                Err(error) => return Err(Box::<dyn std::error::Error + Send + Sync>::from(DatabaseCensusFailure { observed_backends, timeout:Some(error) })),
            };
            observed_backends = count;
            if count == 0 {
                break;
            }
            if Instant::now() >= end {
                return Err(Box::<dyn std::error::Error + Send + Sync>::from(DatabaseCensusFailure {
                    observed_backends,
                    timeout: None,
                }));
            }
            tokio::task::yield_now().await;
        }
        Ok(())
        }).await
    }

    pub async fn reset(&self) -> Result<()> {
        self.restore(false).await
    }

    async fn restore(&self, legacy: bool) -> Result<()> {
        self.identity().await?;
        self.empty_database().await?;
        let statements = if legacy {
            [
                Statement::Query("DROP DATABASE arboresce_migration_harness"),
                Statement::Batch(
                    "DROP ROLE harness_authority_admission; DROP ROLE harness_authority_permission;",
                ),
                Statement::Query(
                    "CREATE DATABASE arboresce_migration_harness OWNER harness_migrator TEMPLATE template0 ENCODING 'UTF8' LC_COLLATE 'C' LC_CTYPE 'C' STRATEGY FILE_COPY",
                ),
                Statement::Batch(
                    "REVOKE ALL ON DATABASE arboresce_migration_harness FROM PUBLIC;
            GRANT CONNECT ON DATABASE arboresce_migration_harness TO harness_writer;",
                ),
            ]
        } else {
            [
                Statement::Batch("REVOKE harness_migrator FROM harness_authority_admission,harness_authority_permission;
            ALTER ROLE harness_authority_admission NOINHERIT NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS CONNECTION LIMIT 2;
            ALTER ROLE harness_authority_permission NOINHERIT NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS CONNECTION LIMIT 2;"),
                Statement::Query("DROP DATABASE arboresce_migration_harness"),
                Statement::Query("CREATE DATABASE arboresce_migration_harness OWNER harness_migrator TEMPLATE template0 ENCODING 'UTF8' LC_COLLATE 'C' LC_CTYPE 'C' CONNECTION LIMIT 2 STRATEGY FILE_COPY"),
                Statement::Batch("REVOKE ALL ON DATABASE arboresce_migration_harness FROM PUBLIC;
            GRANT CONNECT ON DATABASE arboresce_migration_harness TO harness_writer,harness_authority_admission,harness_authority_permission;"),
            ]
        };
        self.statements(true, &statements).await?;
        self.statements(
            false,
            &[Statement::Query("REVOKE ALL ON SCHEMA public FROM PUBLIC")],
        )
        .await
    }

    /// Restore the unchanged initial database/roles before the old final cohort.
    pub async fn finish(self) -> Result<()> {
        self.restore(true).await
    }
}
