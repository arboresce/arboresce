//! Maintained actual-service input, ownership and namespace support.

use crate::Harness;
use arboresce_application::{MigrationRecord, MigrationStore};
use arboresce_xtask::{collection_prepared, collection_services, inventory, measurement, process};
use serde_json::Value;
use sqlx::{Connection, PgConnection};
use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::Write,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub const SELECTOR_PROBE: &str = "ARBORESCE_QUALIFY_SELECTOR_ABSENT";

#[derive(Clone, Copy)]
pub enum ControlKind {
    Request,
    Ack,
    Stopped,
    Complete,
    Adopted,
}

impl ControlKind {
    fn fields(self) -> (&'static str, &'static str) {
        match self {
            Self::Request => (
                "restart-request-v1.json",
                "postgres-harness-restart-request-v1",
            ),
            Self::Ack => ("restart-ack-v1.json", "postgres-harness-restart-ack-v1"),
            Self::Stopped => (
                "restart-stopped-v1.json",
                "postgres-harness-restart-stopped-v1",
            ),
            Self::Complete => (
                "restart-complete-v1.json",
                "postgres-harness-restart-complete-v1",
            ),
            Self::Adopted => (
                "restart-adopted-v1.json",
                "postgres-harness-restart-adopted-v1",
            ),
        }
    }
}

fn create_new(path: &Path) -> std::io::Result<File> {
    File::options().write(true).create_new(true).open(path)
}

fn observe_child(
    command: &mut Command,
    stdout: &Path,
    stderr: &Path,
    started: Instant,
    budget: Duration,
) -> std::io::Result<Output> {
    let mut child = measurement::spawn(command)?;
    let mut trace = Vec::new();
    let result = process::observe(
        &mut child,
        stdout,
        stderr,
        process::Observation {
            budget,
            ..Default::default()
        },
        started,
        &mut trace,
    );
    print!("{}", String::from_utf8_lossy(&trace));
    result
}

fn control_base(path: &Path) -> Result<&Path> {
    if path.canonicalize()? != path
        || fs::symlink_metadata(path)?.permissions().mode() & 0o777 != 0o700
    {
        return Err("invalid owned control directory".into());
    }
    Ok(path)
}

fn publish_control(base: &Path, kind: ControlKind, value: &Value) -> Result<String> {
    let base = control_base(base)?;
    let name = kind.fields().0;
    let mut bytes = serde_json::to_vec(value)?;
    bytes.push(b'\n');
    if bytes.len() > 4096 {
        return Err("restart control limit exceeded".into());
    }
    let pending = base.join(format!("{name}.pending"));
    let mut file = create_new(&pending)?;
    file.write_all(&bytes)?;
    file.sync_all()?;
    fs::hard_link(&pending, base.join(name))?;
    fs::remove_file(pending)?;
    Ok(inventory::sha256(&bytes))
}

async fn wait_control(
    base: &Path,
    kind: ControlKind,
    request: &str,
    end: Instant,
) -> Result<Value> {
    let (name, version) = kind.fields();
    let path = control_base(base)?.join(name);
    loop {
        if Instant::now() >= end {
            return Err("restart control deadline".into());
        }
        if path.try_exists()? {
            let (value, _) = collection_prepared::read_json_until(&path, 4096, end)?;
            if value["request_sha256"] != request || value["version"] != version {
                return Err("restart control identity mismatch".into());
            }
            println!("postgres restart_control={value}");
            return Ok(value);
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}

pub struct Service {
    input: PathBuf,
    bound: Value,
    pub harness: Harness,
    current_pid: u32,
    current_start: String,
    data: PathBuf,
    pg_ctl: PathBuf,
    postgres: PathBuf,
    lease: u64,
    pub stopped_observations: Vec<(&'static str, Result<()>)>,
    pub stopped_probe_error: Option<tokio::time::error::Elapsed>,
    pub stopped_elapsed: Duration,
}

/// Parse each frozen record once, before scenario effects begin.
pub struct Records(BTreeMap<String, MigrationRecord>);

impl Records {
    pub fn new(fixtures: &Value) -> Result<Self> {
        let mut records = BTreeMap::new();
        for name in fixtures["records"]
            .as_object()
            .ok_or("invalid fixture records")?
            .keys()
        {
            records.insert(name.clone(), Service::record(fixtures, name)?);
        }
        Ok(Self(records))
    }

    pub fn get(&self, name: &str) -> Result<MigrationRecord> {
        self.0
            .get(name)
            .copied()
            .ok_or_else(|| "unknown fixture record".into())
    }
}

impl Service {
    pub fn record(fixtures: &Value, name: &str) -> Result<MigrationRecord> {
        let row = &fixtures["records"][name];
        Ok(MigrationRecord::new(
            row["organization_id"]
                .as_str()
                .ok_or("invalid fixture identifier")?
                .parse()?,
            row["record_id"]
                .as_str()
                .ok_or("invalid fixture identifier")?
                .parse()?,
            row["content_revision"]
                .as_str()
                .ok_or("invalid fixture revision")?
                .parse()?,
            row["date"]
                .as_str()
                .ok_or("invalid fixture date")?
                .parse()?,
        ))
    }

    pub async fn snapshot(&self) -> Result<(Vec<i64>, Vec<Value>, Vec<String>)> {
        let mut connection = self.admin(false).await?;
        let ledger: bool =
            sqlx::query_scalar("SELECT to_regclass('public._sqlx_migrations') IS NOT NULL")
                .fetch_one(&mut connection)
                .await?;
        let versions = if ledger {
            sqlx::query_scalar(
                "SELECT version FROM public._sqlx_migrations ORDER BY version LIMIT 4",
            )
            .fetch_all(&mut connection)
            .await?
        } else {
            Vec::new()
        };
        let table: bool =
            sqlx::query_scalar("SELECT to_regclass('migration_harness.records') IS NOT NULL")
                .fetch_one(&mut connection)
                .await?;
        let raw: Vec<(String, String)> = if table {
            sqlx::query_as(
                "SELECT json_build_object(\
            'organization_id',r.organization_id::text,'record_id',r.record_id::text,\
            'content_revision',r.content_revision::text,'date',r.record_date::text)::text,\
            COALESCE(to_jsonb(r)->>'annotation','') \
            FROM migration_harness.records r ORDER BY r.organization_id,r.record_id LIMIT 33",
            )
            .fetch_all(&mut connection)
            .await?
        } else {
            Vec::new()
        };
        if raw.len() > 32 {
            return Err("synthetic row limit exceeded".into());
        }
        let mut records = Vec::new();
        let mut annotations = Vec::new();
        for (row, annotation) in raw {
            records.push(serde_json::from_str(&row)?);
            annotations.push(annotation);
        }
        Ok((versions, records, annotations))
    }

    pub async fn annotate(&self, record: MigrationRecord, text: &str) -> Result<()> {
        let mut connection = self.admin(false).await?;
        sqlx::query("UPDATE migration_harness.records SET annotation=$3 WHERE organization_id=$1::uuid AND record_id=$2::uuid")
            .bind(record.organization().to_string()).bind(record.record().to_string()).bind(text)
            .execute(&mut connection).await?;
        Ok(())
    }

    /// Conserve observed column and constraint definitions across independent cases.
    pub async fn schema(&self) -> Result<Vec<String>> {
        let mut connection = self.admin(false).await?;
        let rows: Vec<String> = sqlx::query_scalar("SELECT definition FROM (\
            SELECT jsonb_build_object('kind','column','ordinal',a.attnum,'name',a.attname,\
            'type',format_type(a.atttypid,a.atttypmod),'not_null',a.attnotnull,\
            'default',pg_get_expr(d.adbin,d.adrelid))::text AS definition \
            FROM pg_attribute a LEFT JOIN pg_attrdef d ON d.adrelid=a.attrelid AND d.adnum=a.attnum \
            WHERE a.attrelid=to_regclass('migration_harness.records') AND a.attnum>0 AND NOT a.attisdropped \
            UNION ALL SELECT jsonb_build_object('kind','constraint','name',conname,\
            'type',contype,'definition',pg_get_constraintdef(oid))::text \
            FROM pg_constraint WHERE conrelid=to_regclass('migration_harness.records')\
            ) definitions ORDER BY definition LIMIT 33")
            .fetch_all(&mut connection).await?;
        if rows.len() > 32 {
            return Err("synthetic schema limit exceeded".into());
        }
        Ok(rows)
    }

    pub fn text(recipe: &Value) -> Result<String> {
        let value = match recipe["kind"].as_str() {
            Some("literal") => recipe["value"]
                .as_str()
                .ok_or("invalid text literal")?
                .to_owned(),
            Some("repeat") => {
                let text = recipe["value"].as_str().ok_or("invalid text repeat")?;
                let count = usize::try_from(recipe["count"].as_u64().ok_or("invalid text count")?)?;
                if text.len().checked_mul(count).is_none_or(|n| n > 1024) {
                    return Err("fixture text limit exceeded".into());
                }
                text.repeat(count)
            }
            Some("concat") => {
                let mut value = String::new();
                for part in recipe["parts"].as_array().ok_or("invalid text parts")? {
                    let part = Self::text(part)?;
                    if value.len().checked_add(part.len()).is_none_or(|n| n > 1024) {
                        return Err("fixture text limit exceeded".into());
                    }
                    value.push_str(&part);
                }
                value
            }
            _ => return Err("unsupported text recipe".into()),
        };
        if value.len() > 1024 {
            return Err("fixture text limit exceeded".into());
        }
        Ok(value)
    }
    pub async fn open() -> Result<Self> {
        let input = PathBuf::from(
            std::env::var_os(collection_services::SELECTOR)
                .ok_or("required isolated PostgreSQL input is absent")?,
        );
        let bound = collection_services::observe(&input, Instant::now() + Duration::from_secs(30))?;
        Self::from_observed(input, bound).await
    }

    /// Only callers that actually completed the full input observation enter here.
    async fn from_observed(input: PathBuf, bound: Value) -> Result<Self> {
        let record = &bound["record"];
        let fields = [
            &record["socket"],
            &record["server_pid"],
            &record["postmaster_start"],
            &record["data"],
            &record["tools"]["pg_ctl"]["path"],
            &record["tools"]["postgres"]["path"],
            &record["lease_deadline_unix_ms"],
        ];
        let (socket, current_pid, current_start, data, pg_ctl, postgres, lease): (
            PathBuf,
            u32,
            String,
            PathBuf,
            PathBuf,
            PathBuf,
            u64,
        ) = serde_json::from_value(Value::Array(fields.into_iter().cloned().collect()))?;
        let harness = Harness::new(socket).map_err(|_| "invalid harness configuration")?;
        let service = Self {
            input,
            bound,
            harness,
            current_pid,
            current_start,
            data,
            pg_ctl,
            postgres,
            lease,
            stopped_observations: Vec::new(),
            stopped_probe_error: None,
            stopped_elapsed: Duration::ZERO,
        };
        service.identity().await?;
        Ok(service)
    }

    pub async fn live_denial(&self, index: usize, case: &Value) -> Result<Value> {
        let field = case["field"].as_str().ok_or("invalid live case field")?;
        if !["system_identifier", "server_pid", "postmaster_start"].contains(&field) {
            return Err("unsupported live identity field".into());
        }
        let mut changed = self.bound["record"].clone();
        let original = changed[field].clone();
        if original == case["replacement"] {
            return Err("live negative did not change its field".into());
        }
        changed[field] = case["replacement"].clone();
        let path = self.base()?.join(format!("live-input-denial-{index}.json"));
        create_new(&path)?.write_all(&serde_json::to_vec(&changed)?)?;
        let observed =
            collection_services::observe(&path, Instant::now() + Duration::from_secs(30))?;
        let denial = match Self::from_observed(path, observed).await {
            Ok(_) => None,
            Err(error)
                if [
                    "isolated server identity mismatch",
                    "isolated server profile mismatch",
                ]
                .contains(&error.to_string().as_str()) =>
            {
                Some(error.to_string())
            }
            Err(error) => return Err(error),
        };
        self.identity().await?;
        Ok(
            serde_json::json!({"original":original,"replacement":case["replacement"],
            "static_checks_passed":true,"accepted":denial.is_none(),"metadata_denial":denial}),
        )
    }

    pub fn socket(&self) -> PathBuf {
        self.harness.socket.clone()
    }

    pub async fn admin(&self, control: bool) -> Result<PgConnection> {
        let options = self
            .harness
            .options(true)
            .map_err(|_| "invalid harness configuration")?
            .username("harness_admin")
            .database(if control {
                "postgres"
            } else {
                "arboresce_migration_harness"
            })
            .application_name("arboresce_harness_control");
        Ok(tokio::time::timeout(
            Duration::from_millis(1000),
            PgConnection::connect_with(&options),
        )
        .await??)
    }

    async fn identity(&self) -> Result<()> {
        self.identity_at(self.current_pid, &self.current_start)
            .await
    }

    async fn identity_at(&self, expected_pid: u32, expected_start: &str) -> Result<()> {
        let mut connection = self.admin(true).await?;
        let raw: String = sqlx::query_scalar("SELECT json_build_object(\
            'system_identifier',(SELECT system_identifier::text FROM pg_control_system()),\
            'postmaster_start',pg_postmaster_start_time()::text,\
            'server_version_num',current_setting('server_version_num')::integer,\
            'data',current_setting('data_directory'),'socket',current_setting('unix_socket_directories'),\
            'listen',current_setting('listen_addresses'),'connections',current_setting('max_connections')::integer,\
            'encoding',current_setting('server_encoding'),'fsync',current_setting('fsync'),\
            'synchronous_commit',current_setting('synchronous_commit'),'address',inet_server_addr())::text")
            .fetch_one(&mut connection).await?;
        let actual: Value = serde_json::from_str(&raw)?;
        let declared = &self.bound["record"];
        for key in ["system_identifier", "server_version_num", "data", "socket"] {
            if actual[key] != declared[key] {
                return Err("isolated server identity mismatch".into());
            }
        }
        let pid: u32 = std::fs::read_to_string(self.data.join("postmaster.pid"))?
            .lines()
            .next()
            .ok_or("absent server PID")?
            .parse()?;
        let expected_profile = [
            ("postmaster_start", serde_json::json!(expected_start)),
            ("listen", serde_json::json!("")),
            ("connections", serde_json::json!(16)),
            ("encoding", serde_json::json!("UTF8")),
            ("fsync", serde_json::json!("on")),
            ("synchronous_commit", serde_json::json!("on")),
            ("address", serde_json::Value::Null),
        ];
        if pid != expected_pid
            || expected_profile
                .iter()
                .any(|(key, value)| actual[*key] != *value)
        {
            return Err("isolated server profile mismatch".into());
        }
        let bad: i64 = sqlx::query_scalar("SELECT count(*) FROM pg_roles WHERE rolname IN ('harness_migrator','harness_writer') \
            AND (rolsuper OR rolcreatedb OR rolcreaterole OR rolreplication OR rolbypassrls OR rolconnlimit<>2)")
            .fetch_one(&mut connection).await?;
        let count: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM pg_roles WHERE rolname IN ('harness_migrator','harness_writer')",
        )
        .fetch_one(&mut connection)
        .await?;
        if bad != 0 || count != 2 {
            return Err("isolated role profile mismatch".into());
        }
        self.writer_identity(false).await?;
        Ok(())
    }

    pub(super) fn base(&self) -> Result<&Path> {
        let path = self.data.as_path().parent().ok_or("absent owned base")?;
        control_base(path)
    }

    pub(super) fn control(&self, kind: ControlKind, value: &Value) -> Result<String> {
        publish_control(self.base()?, kind, value)
    }

    pub(super) async fn control_wait(
        &self,
        kind: ControlKind,
        request: &str,
        end: Instant,
    ) -> Result<Value> {
        wait_control(self.base()?, kind, request, end).await
    }

    /// Pure control-file policy in a distinct owned directory, without daemon effects.
    pub async fn control_probe(&self, index: usize, case: &Value) -> Result<Value> {
        use std::os::unix::fs::DirBuilderExt;
        let base = self.base()?.join(format!("qualifier-control-{index}"));
        fs::DirBuilder::new().mode(0o700).create(&base)?;
        let kind = match case["kind"].as_str() {
            Some("ack") => ControlKind::Ack,
            Some("stopped") => ControlKind::Stopped,
            Some("adopted") => ControlKind::Adopted,
            _ => return Err("invalid control fixture kind".into()),
        };
        let request = "1".repeat(64);
        let mut value = serde_json::json!({"version":kind.fields().1,"request_sha256":request});
        let path = base.join(kind.fields().0);
        if case["phase"] == "publish" {
            let pending = base.join(format!("{}.pending", kind.fields().0));
            let mut selected = base.clone();
            match case["recipe"].as_str() {
                Some("publish-value") => (),
                Some("base-mode") => fs::set_permissions(&base, fs::Permissions::from_mode(0o770))?,
                Some("base-symlink") => {
                    selected = base.join("owned-alias");
                    std::os::unix::fs::symlink(&base, &selected)?;
                }
                Some("pending-collision") | Some("final-collision") => {
                    let existing = if case["recipe"] == "pending-collision" {
                        &pending
                    } else {
                        &path
                    };
                    let bytes = case["existing_bytes"]
                        .as_str()
                        .ok_or("invalid collision literal")?
                        .as_bytes();
                    create_new(existing)?.write_all(bytes)?;
                }
                _ => return Err("invalid publication fixture recipe".into()),
            }
            let mode = fs::symlink_metadata(&base)?.permissions().mode() & 0o777;
            let alias = selected.canonicalize()? != selected;
            let started = Instant::now();
            let result = publish_control(&selected, kind, &case["value"]);
            // Restore even when publication unexpectedly succeeds or fails differently.
            fs::set_permissions(&base, fs::Permissions::from_mode(0o700))?;
            let end = started + Duration::from_secs(2);
            let mut files = Vec::new();
            for selected in [&path, &pending] {
                files.push(if selected.try_exists()? {
                    let actual = collection_prepared::fingerprint(selected, 4096, end)?;
                    serde_json::json!({"file_bytes":actual["size"],"sha256":actual["sha256"]})
                } else {
                    Value::Null
                });
            }
            let io_kind = result.as_ref().err().and_then(|error| {
                error
                    .downcast_ref::<std::io::Error>()
                    .map(|error| format!("{:?}", error.kind()))
            });
            return Ok(serde_json::json!({
                "accepted":result.is_ok(),"error":result.as_ref().err().map(ToString::to_string),
                "io_error_kind":io_kind,"final":files[0],"pending":files[1],
                "mode_during_attempt":mode,"alias_during_attempt":alias,
                "base_mode_restored":fs::symlink_metadata(&base)?.permissions().mode() & 0o777 == 0o700,
                "elapsed_ms":started.elapsed().as_millis(),"path":path,"pending_path":pending
            }));
        }
        match case["recipe"].as_str() {
            Some("valid") => {
                publish_control(&base, kind, &value)?;
            }
            Some("wrong-hash") | Some("wrong-version") => {
                if case["recipe"] == "wrong-hash" {
                    value["request_sha256"] = serde_json::json!("2".repeat(64));
                } else {
                    value["version"] = serde_json::json!("unsupported-control-version");
                }
                publish_control(&base, kind, &value)?;
            }
            Some("missing") => (),
            Some("directory") => fs::create_dir(&path)?,
            Some("exact") | Some("one-over") => {
                let mut bytes = serde_json::to_vec(&value)?;
                bytes.resize(
                    if case["recipe"] == "exact" {
                        4096
                    } else {
                        4097
                    },
                    b' ',
                );
                fs::write(&path, bytes)?;
            }
            _ => return Err("invalid control fixture recipe".into()),
        }
        let started = Instant::now();
        let end = started + Duration::from_secs(2);
        let result = wait_control(&base, kind, &request, end).await;
        if case["recipe"] != "missing" && Instant::now() >= end {
            return Err("unexpected control fixture deadline".into());
        }
        let metadata = fs::symlink_metadata(&path).ok();
        Ok(
            serde_json::json!({"accepted":result.is_ok(),"value":result.as_ref().ok(),"error":result.as_ref().err().map(ToString::to_string),"path":path,"regular_file":metadata.as_ref().is_some_and(|m|m.is_file()),"directory":metadata.as_ref().is_some_and(|m|m.is_dir()),"bytes":metadata.as_ref().filter(|m|m.is_file()).map(|m|m.len()),"sha256":if metadata.as_ref().is_some_and(|m|m.is_file()){Some(inventory::sha256(&fs::read(&path)?))}else{None},"elapsed_ms":started.elapsed().as_millis(),"request_sha256":request}),
        )
    }

    pub(super) fn native(&self, start: bool, end: Instant) -> Result<()> {
        let phase = if start { "start" } else { "stop" };
        let base = self.base()?;
        let stdout_path = base.join(format!("restart-{phase}.stdout"));
        let stderr_path = base.join(format!("restart-{phase}.stderr"));
        let mut command = Command::new(&self.pg_ctl);
        command
            .current_dir(base)
            .env_clear()
            .env("PATH", "/usr/bin:/bin")
            .env("LC_ALL", "C")
            .env("LANG", "C")
            .env("TZ", "UTC")
            .stdin(Stdio::null())
            .stdout(create_new(&stdout_path)?)
            .stderr(create_new(&stderr_path)?)
            .arg("-D")
            .arg(&self.data)
            .args(["-w", "-t", "10"]);
        if start {
            command
                .arg("-p")
                .arg(&self.postgres)
                .arg("-l")
                .arg(base.join("restart-postgres.log"));
        } else {
            command.args(["-m", "fast"]);
        }
        command.arg(phase);
        let started = Instant::now();
        let budget = end
            .saturating_duration_since(started)
            .min(Duration::from_secs(10));
        if budget.is_zero() {
            return Err("restart native deadline".into());
        }
        let result = observe_child(&mut command, &stdout_path, &stderr_path, started, budget);
        if !result?.status.success() {
            return Err("owned restart command failed".into());
        }
        Ok(())
    }

    fn restart_time(&self) -> Result<u128> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
        if now + 25000 > u128::from(self.lease) {
            return Err("insufficient original restart lease".into());
        }
        Ok(now)
    }

    /// One measured restart, last in this service run; the initial input stays immutable.
    pub async fn restart(&mut self, record: MigrationRecord, annotation: &str) -> Result<()> {
        // Refuse a known-insufficient budget before expensive immutable-input intake.
        self.restart_time()?;
        self.unchanged().await?;
        let before = self.snapshot().await?;
        let old_pid = self.current_pid;
        // Intake consumes time; recheck immediately before any control/native effect.
        let now = self.restart_time()?;
        let end = Instant::now() + Duration::from_secs(25);
        let request = self.control(ControlKind::Request, &serde_json::json!({
            "version":"postgres-harness-restart-request-v1", "input_sha256":self.bound["sha256"],
            "old_pid":old_pid, "old_start":self.current_start, "deadline_unix_ms":u64::try_from(now + 25000)? }))?;
        let (new_pid, new_start) = tokio::time::timeout_at(end.into(), async {
            let ack = self
                .control_wait(
                    ControlKind::Ack,
                    &request,
                    (Instant::now() + Duration::from_secs(2)).min(end),
                )
                .await?;
            if ack["input_sha256"] != self.bound["sha256"]
                || ack["old_pid"] != old_pid
                || ack["old_start"] != self.current_start
                || ack["deadline_unix_ms"] != u64::try_from(now + 25000)?
            {
                return Err("restart acknowledgement mismatch".into());
            }
            let stop_end = (Instant::now() + Duration::from_secs(10)).min(end);
            self.native(false, stop_end)?;
            let data = &self.data;
            if data.join("postmaster.pid").try_exists()?
                || self.socket().join(".s.PGSQL.5432").try_exists()?
            {
                return Err("owned server stop was not observed".into());
            }
            let stopped = self
                .control_wait(
                    ControlKind::Stopped,
                    &request,
                    (Instant::now() + Duration::from_secs(2)).min(stop_end),
                )
                .await?;
            if stopped["old_pid"] != old_pid
                || stopped["old_process_absent"] != true
                || stopped["pidfile_absent"] != true
                || stopped["socket_absent"] != true
            {
                return Err("old server disappearance mismatch".into());
            }
            // Keep actual results, including unexpected failures, until restart/adoption.
            // The independent driver classifies them only after persistence succeeds.
            let started = Instant::now();
            let mut observations = Vec::new();
            let outage = tokio::time::timeout_at(
                (started + Duration::from_secs(2)).min(end).into(),
                async {
                    observations.push(("admin", self.admin(false).await.map(|_| ())));
                    observations.push(("snapshot", self.snapshot().await.map(|_| ())));
                    observations.push(("schema", self.schema().await.map(|_| ())));
                    observations.push(("annotate", self.annotate(record, annotation).await));
                    observations.push(("identity", self.identity().await));
                    observations.push(("writer", self.writer_identity(true).await));
                },
            )
            .await;
            self.stopped_elapsed = started.elapsed();
            self.stopped_observations = observations;
            self.stopped_probe_error = outage.err();
            self.native(true, end)?;
            let new_pid: u32 = fs::read_to_string(data.join("postmaster.pid"))?
                .lines()
                .next()
                .ok_or("absent new PID")?
                .parse()?;
            let mut control = self.admin(true).await?;
            let new_start: String = sqlx::query_scalar("SELECT pg_postmaster_start_time()::text")
                .fetch_one(&mut control)
                .await?;
            drop(control);
            if new_pid == old_pid || new_start == self.current_start {
                return Err("server generation did not advance".into());
            }
            self.identity_at(new_pid, &new_start).await?;
            let fresh = Harness::new(self.socket()).map_err(|_| "invalid restart reader")?;
            if fresh
                .read(record.organization(), record.record())
                .await
                .map_err(|_| "restart read failed")?
                != Some(record)
                || self.snapshot().await? != before
            {
                return Err("owned restart changed exact stored data".into());
            }
            let completion_hash = self.control(
                ControlKind::Complete,
                &serde_json::json!({
            "version":"postgres-harness-restart-complete-v1", "request_sha256":request,
            "old_pid":old_pid, "new_pid":new_pid, "postmaster_start":new_start,
            "system_identifier":self.bound["record"]["system_identifier"] }),
            )?;
            let adopted = self
                .control_wait(ControlKind::Adopted, &request, end)
                .await?;
            if adopted["completion_sha256"] != completion_hash
                || adopted["new_pid"] != new_pid
                || adopted["postmaster_start"] != new_start
                || adopted["system_identifier"] != self.bound["record"]["system_identifier"]
            {
                return Err("successor adoption mismatch".into());
            }
            Ok::<_, Box<dyn std::error::Error>>((new_pid, new_start))
        })
        .await??;
        self.current_pid = new_pid;
        self.current_start = new_start;
        self.unchanged().await?;
        Ok(())
    }

    /// Verify effective current grants, including the recreated namespace.
    pub async fn writer_identity(&self, table_required: bool) -> Result<()> {
        let mut connection = self
            .harness
            .connect(false)
            .await
            .map_err(|_| "writer unavailable")?;
        let raw: String = sqlx::query_scalar("SELECT json_build_object(\
            'role',current_user,'database',current_database(),\
            'database_owner',(SELECT pg_get_userbyid(datdba)=current_user FROM pg_database WHERE datname=current_database()),\
            'database_create',has_database_privilege(current_user,current_database(),'CREATE'),\
            'public_create',has_schema_privilege(current_user,'public','CREATE'),\
            'migration_create',CASE WHEN to_regnamespace('migration_harness') IS NULL THEN false ELSE has_schema_privilege(current_user,'migration_harness','CREATE') END,\
            'migrator_member',pg_has_role(current_user,'harness_migrator','MEMBER'),\
            'admin_member',pg_has_role(current_user,'harness_admin','MEMBER'),\
            'table_present',to_regclass('migration_harness.records') IS NOT NULL,\
            'table_owner',COALESCE((SELECT pg_get_userbyid(relowner)=current_user FROM pg_class WHERE oid=to_regclass('migration_harness.records')),false),\
            'table_insert',CASE WHEN to_regclass('migration_harness.records') IS NULL THEN false ELSE has_table_privilege(current_user,'migration_harness.records','INSERT') END,\
            'table_select',CASE WHEN to_regclass('migration_harness.records') IS NULL THEN false ELSE has_table_privilege(current_user,'migration_harness.records','SELECT') END,\
            'table_update',CASE WHEN to_regclass('migration_harness.records') IS NULL THEN false ELSE has_table_privilege(current_user,'migration_harness.records','UPDATE') END,\
            'table_delete',CASE WHEN to_regclass('migration_harness.records') IS NULL THEN false ELSE has_table_privilege(current_user,'migration_harness.records','DELETE') END,\
            'statement_timeout',current_setting('statement_timeout'),'transaction_timeout',current_setting('transaction_timeout'),\
            'lock_timeout',current_setting('lock_timeout'),'address',inet_server_addr())::text")
            .fetch_one(&mut connection).await?;
        let observed: Value = serde_json::from_str(&raw)?;
        if observed["role"] != "harness_writer"
            || observed["database"] != "arboresce_migration_harness"
            || [
                "database_owner",
                "database_create",
                "public_create",
                "migration_create",
                "migrator_member",
                "admin_member",
                "table_owner",
                "table_update",
                "table_delete",
            ]
            .iter()
            .any(|key| observed[*key] != false)
            || observed["statement_timeout"] != "2s"
            || observed["transaction_timeout"] != "2s"
            || observed["lock_timeout"] != "500ms"
            || !observed["address"].is_null()
            || (table_required
                && ["table_present", "table_insert", "table_select"]
                    .iter()
                    .any(|key| observed[*key] != true))
        {
            return Err("current writer authority mismatch".into());
        }
        println!("postgres writer={observed}");
        Ok(())
    }

    /// Only the identity-checked synthetic namespace is ever recreated.
    pub async fn reset(&self) -> Result<()> {
        self.identity().await?;
        let mut control = self.admin(true).await?;
        for statement in [
            "DROP DATABASE arboresce_migration_harness WITH (FORCE)",
            "CREATE DATABASE arboresce_migration_harness OWNER harness_migrator TEMPLATE template0 \
            ENCODING 'UTF8' LC_COLLATE 'C' LC_CTYPE 'C' STRATEGY FILE_COPY",
            "REVOKE ALL ON DATABASE arboresce_migration_harness FROM PUBLIC",
            "GRANT CONNECT ON DATABASE arboresce_migration_harness TO harness_writer",
        ] {
            sqlx::query(statement).execute(&mut control).await?;
        }
        drop(control);
        let mut database = self.admin(false).await?;
        sqlx::query("REVOKE ALL ON SCHEMA public FROM PUBLIC")
            .execute(&mut database)
            .await?;
        drop(database);
        self.writer_identity(false).await?;
        Ok(())
    }

    pub async fn unchanged(&self) -> Result<()> {
        if collection_services::observe(&self.input, Instant::now() + Duration::from_secs(30))?
            != self.bound
        {
            return Err("isolated service inputs changed".into());
        }
        self.identity().await?;
        Ok(())
    }

    pub(super) fn copy_input(&self, index: usize, record: &Value) -> Result<PathBuf> {
        let path = self.base()?.join(format!("helper-input-{index}.json"));
        create_new(&path)?.write_all(&serde_json::to_vec(record)?)?;
        Ok(path)
    }

    pub(super) fn selector_probe(&self) -> Result<Value> {
        let base = self.base()?;
        let stdout = base.join("selector-probe.stdout");
        let stderr = base.join("selector-probe.stderr");
        let mut command = Command::new(std::env::current_exe()?);
        command
            .args([
                "--exact",
                "real_migration_scenarios",
                "--nocapture",
                "--test-threads=1",
            ])
            .env(SELECTOR_PROBE, "1")
            .env_remove(collection_services::SELECTOR)
            .stdin(Stdio::null())
            .stdout(create_new(&stdout)?)
            .stderr(create_new(&stderr)?);
        let started = Instant::now();
        let outcome = observe_child(
            &mut command,
            &stdout,
            &stderr,
            started,
            Duration::from_secs(5),
        );
        if !outcome?.status.success() {
            return Err("selector probe child failed".into());
        }
        let text = fs::read_to_string(stdout)?;
        let raw = text
            .lines()
            .find_map(|line| {
                line.split_once("postgres selector_denial=")
                    .map(|(_, value)| value)
            })
            .ok_or("absent selector observation")?;
        Ok(serde_json::from_str(raw)?)
    }

    /// Guard failures affect only this admitted synthetic service and owned copies.
    /// Setup/cleanup failures are infrastructure errors, never observed denials.
    pub async fn guard_probe(
        &self,
        index: usize,
        recipe: &str,
        record: MigrationRecord,
    ) -> Result<Value> {
        self.identity().await?;
        let before = self.snapshot().await?;
        let input_hash = inventory::sha256(&fs::read(&self.input)?);
        let mut detail = serde_json::json!({});
        let denial: Result<()> = match recipe {
            "selector-absent" => {
                detail = self.selector_probe()?;
                match detail["error"].as_str() {
                    Some(error) => Err(error.to_owned().into()),
                    None => Ok(()),
                }
            }
            "writer-createdb" | "writer-admin-member" => {
                let (apply, restore) = if recipe == "writer-createdb" {
                    (
                        "ALTER ROLE harness_writer CREATEDB",
                        "ALTER ROLE harness_writer NOCREATEDB",
                    )
                } else {
                    (
                        "GRANT harness_admin TO harness_writer",
                        "REVOKE harness_admin FROM harness_writer",
                    )
                };
                let mut control = self.admin(true).await?;
                sqlx::query(apply).execute(&mut control).await?;
                let denial = self.reset().await;
                // Restore even when the guard unexpectedly succeeds or returns another error.
                let restored = sqlx::query(restore).execute(&mut control).await;
                drop(control);
                restored?;
                self.identity().await?;
                detail["privilege_restored"] = serde_json::json!(true);
                denial
            }
            "input-whitespace" => {
                let path = self.copy_input(index, &self.bound["record"])?;
                let bound =
                    collection_services::observe(&path, Instant::now() + Duration::from_secs(30))?;
                let subject = Self::from_observed(path.clone(), bound).await?;
                File::options().append(true).open(&path)?.write_all(b" ")?;
                detail["static_and_live_initially_passed"] = serde_json::json!(true);
                subject.unchanged().await
            }
            "declared-data" => {
                use std::os::unix::fs::DirBuilderExt;
                let data = self.base()?.join(format!("helper-data-{index}"));
                fs::DirBuilder::new().mode(0o700).create(&data)?;
                let mut changed = self.bound["record"].clone();
                changed["data"] = serde_json::json!(data);
                for name in ["postgresql.conf", "pg_hba.conf", "postgresql.auto.conf"] {
                    let bytes = fs::read(self.data.join(name))?;
                    let destination = data.join(name);
                    create_new(&destination)?.write_all(&bytes)?;
                    changed["files"].as_object_mut().ok_or("invalid copied files")?.insert(
                        destination.to_str().ok_or("invalid copied path")?.to_owned(),
                        serde_json::json!({"size":bytes.len(),"sha256":inventory::sha256(&bytes)}));
                }
                let path = self.copy_input(index, &changed)?;
                let bound =
                    collection_services::observe(&path, Instant::now() + Duration::from_secs(30))?;
                detail["static_checks_passed"] = serde_json::json!(true);
                Self::from_observed(path, bound).await.map(|_| ())
            }
            "short-restart-lease" => {
                let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
                let lease = u64::try_from(now + 60000)?;
                if lease >= self.lease {
                    return Err("insufficient lease for guard preparation".into());
                }
                let mut changed = self.bound["record"].clone();
                changed["lease_deadline_unix_ms"] = serde_json::json!(lease);
                let path = self.copy_input(index, &changed)?;
                let bound =
                    collection_services::observe(&path, Instant::now() + Duration::from_secs(24))?;
                let mut subject = Self::from_observed(path, bound).await?;
                let remaining = u128::from(lease)
                    .checked_sub(SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis())
                    .ok_or("guard preparation exhausted lease")?;
                let wait = remaining
                    .checked_sub(24000)
                    .ok_or("guard preparation left insufficient observation time")?;
                tokio::time::sleep(Duration::from_millis(u64::try_from(wait)?)).await;
                let short_remaining = u128::from(lease)
                    .checked_sub(SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis())
                    .filter(|remaining| (1..25000).contains(remaining))
                    .ok_or("guard did not reach a current short lease")?;
                detail["remaining_ms_before_denial"] = serde_json::json!(short_remaining);
                let denial = subject.restart(record, "").await;
                let valid =
                    SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() < u128::from(lease);
                if !valid {
                    return Err("short lease expired during qualification".into());
                }
                for kind in [
                    ControlKind::Request,
                    ControlKind::Ack,
                    ControlKind::Stopped,
                    ControlKind::Complete,
                    ControlKind::Adopted,
                ] {
                    if self.base()?.join(kind.fields().0).try_exists()? {
                        return Err("short lease published restart control".into());
                    }
                }
                detail["lease_still_valid"] = serde_json::json!(valid);
                detail["restart_controls_absent"] = serde_json::json!(true);
                denial
            }
            _ => return Err("unsupported service guard recipe".into()),
        };
        self.identity().await?;
        if inventory::sha256(&fs::read(&self.input)?) != input_hash
            || self.snapshot().await? != before
        {
            return Err("service guard changed original input or stored state".into());
        }
        Ok(
            serde_json::json!({"error":denial.err().map(|error|error.to_string()),"detail":detail,
            "state_unchanged":true,"original_identity_unchanged":true,"input_sha256":input_hash}),
        )
    }
}
