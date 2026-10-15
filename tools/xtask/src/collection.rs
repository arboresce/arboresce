//! Complete local collection with explicit preparation and retained failed output.

use std::{
    collections::{BTreeMap, BTreeSet},
    ffi::OsString,
    fs,
    io::Write,
    os::unix::{ffi::OsStrExt, fs::PermissionsExt},
    path::{Path, PathBuf},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use serde_json::{Map, Value, json};

use crate::{
    Action, collection_artifacts,
    collection_inputs::{self, CollectError, Result},
    collection_modules,
    collection_prepared::{self as prepared, PROFILE},
    collection_profiles, collection_receipts,
    collection_runner::{self as runner, Runner},
    collection_tests, coverage, emit,
    inventory::{self, sha256},
    measurement, strict_json, workspace,
};

pub const USAGE: &str = "usage: arboresce-verify prepare <workspace> <sysroot> <cargo-llvm-cov> <cargo-home> <prepared.json> [<outer-context>]\n       arboresce-verify <workspace> <roles.json> <tests.json> <prepared.json> <fresh-output>\n";
const OUTPUT_FAILURE: &[u8] = b"collector output failed\n";

fn text(value: &Value, key: &str) -> Result<String> {
    value[key]
        .as_str()
        .map(str::to_owned)
        .ok_or(CollectError::Input)
}
fn utf8(bytes: &[u8]) -> Result<&str> {
    std::str::from_utf8(bytes).map_err(|_| CollectError::Input)
}
fn now() -> Result<u64> {
    u64::try_from(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| CollectError::Input)?
            .as_nanos(),
    )
    .map_err(|_| CollectError::Input)
}
fn canonical(path: &str) -> Result<PathBuf> {
    Path::new(path)
        .canonicalize()
        .map_err(|_| CollectError::Input)
}
fn values() -> BTreeMap<String, String> {
    BTreeMap::new()
}

/// Create a new output directory outside the source tree and its ancestors.
pub fn fresh(root: &Path, path: &str) -> Result<PathBuf> {
    let path = Path::new(path);
    let parent = path.parent().ok_or(CollectError::Input)?;
    let parent = parent.canonicalize().map_err(|_| CollectError::Input)?;
    let result = parent.join(path.file_name().ok_or(CollectError::Input)?);
    if result.starts_with(root) || root.starts_with(&result) {
        return Err(CollectError::Input);
    }
    fs::create_dir(&result).map_err(|_| CollectError::Input)?;
    Ok(result)
}

fn field(raw: &str, prefix: &str) -> Result<String> {
    let matches = raw
        .lines()
        .filter_map(|line| line.strip_prefix(prefix))
        .collect::<Vec<_>>();
    let [value] = matches.as_slice() else {
        return Err(CollectError::Prepared);
    };
    Ok((*value).to_owned())
}

fn versions(commands: &mut Runner) -> Result<Value> {
    let target = commands.target.clone();
    let cargo = commands.success("cargo", &["--version"], 30, &values(), &target)?;
    let rustc = commands.success("rustc", &["-vV"], 30, &values(), &target)?;
    let plugin = commands.success(
        "cargo-llvm-cov",
        &["llvm-cov", "--version"],
        30,
        &values(),
        &target,
    )?;
    let cov = commands.success("llvm-cov", &["--version"], 30, &values(), &target)?;
    let profdata = commands.success("llvm-profdata", &["--version"], 30, &values(), &target)?;
    versions_from([
        &cargo.output.stdout,
        &rustc.output.stdout,
        &plugin.output.stdout,
        &cov.output.stdout,
        &profdata.output.stdout,
    ])
}

/// Decode the five actual selected-tool stdout streams in command order.
pub fn versions_from([cargo, rustc, plugin, cov, profdata]: [&[u8]; 5]) -> Result<Value> {
    let rustc = utf8(rustc)?;
    let llvm_version = |raw: &[u8]| -> Result<String> {
        let value = field(utf8(raw)?, "  LLVM version ")?;
        let (version, suffix) = value.split_once("-rust-").ok_or(CollectError::Prepared)?;
        if suffix != "1.99.0-stable" {
            return Err(CollectError::Prepared);
        }
        Ok(version.to_owned())
    };
    let cargo = utf8(cargo)?
        .strip_prefix("cargo ")
        .and_then(|s| s.split_whitespace().next())
        .ok_or(CollectError::Prepared)?;
    let plugin = utf8(plugin)?
        .strip_prefix("cargo-llvm-cov ")
        .and_then(|s| s.strip_suffix('\n'))
        .ok_or(CollectError::Prepared)?;
    Ok(
        json!({"cargo":cargo,"rustc":field(rustc,"release: ")?,"host":field(rustc,"host: ")?,
        "rust_llvm":field(rustc,"LLVM version: ")?,"cargo-llvm-cov":plugin,
        "llvm-cov":llvm_version(cov)?,"llvm-profdata":llvm_version(profdata)?}),
    )
}

fn observe(root: &Path, base: &Value, commands: &mut Runner, deadline: Instant) -> Result<Value> {
    prepared::check_time(deadline)?;
    let mut actual = base.clone();
    let mut tools = Map::new();
    for name in prepared::TOOLS {
        tools.insert(
            name.to_owned(),
            prepared::tool(Path::new(&text(&base["tools"][name], "program")?), deadline)?,
        );
    }
    actual["tools"] = Value::Object(tools);
    let target = commands.target.clone();
    let compiler = commands.success("rustc", &["--print", "sysroot"], 30, &values(), &target)?;
    prepared::runtime(&actual, utf8(&compiler.output.stdout)?)?;
    actual["resources"] = prepared::resources(Path::new(&text(base, "sysroot")?), deadline)?;
    actual["configs"] = prepared::configs(root, Path::new(&text(base, "cargo_home")?), deadline)?;
    actual["versions"] = versions(commands)?;
    if Instant::now() >= deadline {
        return Err(CollectError::Prepared);
    }
    prepared::validate(&actual)?;
    Ok(actual)
}

fn prepare(args: &[&str]) -> Result<()> {
    let [root, sysroot, plugin, cargo_home, manifest, rest @ ..] = args else {
        return Err(CollectError::Input);
    };
    if rest.len() > 1 {
        return Err(CollectError::Input);
    }
    let root = canonical(root)?;
    let sysroot = canonical(sysroot)?;
    let cargo_home = canonical(cargo_home)?;
    let plugin = std::env::current_dir()
        .map_err(|_| CollectError::Input)?
        .join(plugin);
    let manifest = std::env::current_dir()
        .map_err(|_| CollectError::Input)?
        .join(manifest);
    if manifest.exists() || manifest.starts_with(&root) {
        return Err(CollectError::Input);
    }
    let output = fresh(
        &root,
        &format!("{}.evidence", manifest.to_str().ok_or(CollectError::Input)?),
    )?;
    let deadline = Instant::now() + Duration::from_secs(180);
    let outer = if let Some(context) = rest.first() {
        let context = canonical(context)?;
        json!({"context":context,"environment":prepared::environment()?})
    } else {
        Value::Null
    };
    let mut value = json!({"version":"native-prepared-tools-v1","profile":PROFILE,"sysroot":sysroot,
        "cargo_home":cargo_home,"tools":prepared::paths(&sysroot,&plugin,deadline)?,
        "resources":{},"configs":{},"versions":{},"outer":outer});
    prepared::outer_until(&value, deadline)?;
    let mut commands = Runner::new(&root, &output, &value)?;
    commands.deadline = Some(deadline);
    value = observe(&root, &value, &mut commands, deadline)?;
    prepared::outer_until(&value, deadline)?;
    runner::record(&output.join("prepared.json"), &value)?;
    runner::record(&manifest, &value)?;
    prepared::check_time(deadline)
}

fn metadata(commands: &mut Runner) -> Result<(String, Value)> {
    let target = commands.target.clone();
    let output = commands.success(
        "cargo",
        Action::Architecture.cargo_args(),
        30,
        &values(),
        &target,
    )?;
    let raw = utf8(&output.output.stdout)?.to_owned();
    let parsed = strict_json::parse(&raw).ok_or(CollectError::Input)?;
    Ok((raw, parsed))
}

fn context(
    output: &Path,
    source: &Value,
    deadline: Instant,
) -> Result<(PathBuf, measurement::Request, Value)> {
    let path = output.join("context");
    fs::create_dir(&path).map_err(|_| CollectError::Intake)?;
    for name in ["launches", "exits"] {
        fs::create_dir(path.join(name)).map_err(|_| CollectError::Intake)?;
    }
    let source_sha256 = sha256(format!("{source}\n").as_bytes());
    let run = sha256(format!("{}:{}:{}", output.display(), std::process::id(), now()?).as_bytes());
    let value = json!({"version":"native-measurement-v1","run_id":&run[..32],"source_sha256":source_sha256});
    runner::record(&path.join("context.json"), &value)?;
    let hash = prepared::read_json_until(&path.join("context.json"), 32768, deadline)?.1;
    let request = measurement::Request::open(&path).map_err(|_| CollectError::Input)?;
    let identity = json!({"run_id":value["run_id"],"source_sha256":value["source_sha256"],"context_sha256":hash,
        "context_path":path.as_os_str().as_bytes(),"driver_pid":std::process::id(),"started_ns":now()?,"finished_ns":0});
    Ok((path, request, identity))
}

/// Materialize the selected physical executable identities before collection.
pub fn objects(selection: &Value, deadline: Instant) -> Result<Vec<Value>> {
    let mut result = Vec::new();
    for row in selection["objects"]
        .as_array()
        .ok_or(CollectError::Artifacts)?
    {
        let path = PathBuf::from(text(row, "path")?);
        if path.canonicalize().map_err(|_| CollectError::Artifacts)? != path
            || fs::metadata(&path)
                .map_err(|_| CollectError::Artifacts)?
                .permissions()
                .mode()
                & 0o111
                == 0
        {
            return Err(CollectError::Artifacts);
        }
        let mut row = row.clone();
        row["sha256"] = prepared::fingerprint(&path, 64 << 20, deadline)?["sha256"].clone();
        if row["role"] == "harness" {
            row["test_count"] = row["tests"]
                .as_array()
                .ok_or(CollectError::Selection)?
                .len()
                .into();
        }
        result.push(row);
    }
    Ok(result)
}

/// Read bounded receipts whose physical names match their recorded PIDs.
pub fn receipt_directory(path: &Path, version: &str, deadline: Instant) -> Result<Vec<Value>> {
    prepared::tree(
        path,
        prepared::Limits {
            files: 4096,
            per_file: 65536,
            total: 256 << 20,
            depth: 0,
        },
        deadline,
    )?;
    let mut result = Vec::new();
    for entry in fs::read_dir(path).map_err(|_| CollectError::Intake)? {
        let entry = entry.map_err(|_| CollectError::Intake)?;
        prepared::fingerprint(&entry.path(), 65536, deadline)?;
        let (record, hash) = prepared::read_json_until(&entry.path(), 65536, deadline)?;
        let pid = record["pid"].as_u64().ok_or(CollectError::Conservation)?;
        if record["version"] != version
            || entry.file_name() != OsString::from(format!("{pid}.json"))
        {
            return Err(CollectError::Conservation);
        }
        result.push(json!({"sha256":hash,"record":record}));
    }
    Ok(result)
}

/// Decode the PID from the supported raw-profile filename shape.
pub fn profile_pid(name: &str) -> Result<u64> {
    let rest = name
        .strip_prefix("arboresce-")
        .and_then(|s| s.strip_suffix(".profraw"))
        .ok_or(CollectError::Conservation)?;
    let (pid, signature) = rest.split_once('-').ok_or(CollectError::Conservation)?;
    if signature.is_empty() || pid.is_empty() || !pid.bytes().all(|b| b.is_ascii_digit()) {
        return Err(CollectError::Conservation);
    }
    pid.parse().map_err(|_| CollectError::Conservation)
}

/// Merge every input once through retained, bounded-fan-in LLVM invocations.
pub fn merge(commands: &mut Runner, profiles: &[PathBuf], output: &Path) -> Result<()> {
    let mut inputs = profiles.to_vec();
    let mut level = 0;
    while inputs.len() > 8 {
        let mut directory = output.as_os_str().to_owned();
        directory.push(format!(".merge-{level}"));
        let directory = PathBuf::from(directory);
        fs::create_dir(&directory).map_err(|_| CollectError::Intake)?;
        let mut partials = Vec::new();
        for (index, group) in inputs.chunks(8).enumerate() {
            let partial = directory.join(format!("{index:04}.profdata"));
            merge_one(commands, group, &partial)?;
            partials.push(partial);
        }
        inputs = partials;
        level += 1;
    }
    merge_one(commands, &inputs, output)
}

fn merge_one(commands: &mut Runner, profiles: &[PathBuf], output: &Path) -> Result<()> {
    let mut args = vec!["merge".to_owned(), "-sparse".to_owned()];
    for path in profiles {
        args.push(path.to_str().ok_or(CollectError::Input)?.to_owned());
    }
    args.extend([
        "-o".to_owned(),
        output.to_str().ok_or(CollectError::Input)?.to_owned(),
    ]);
    let tool = commands.program("llvm-profdata")?;
    let target = commands.target.clone();
    if !commands
        .run(&tool, &args, 180, &values(), &target)?
        .output
        .status
        .success()
    {
        return Err(CollectError::Command);
    }
    Ok(())
}

fn export(commands: &mut Runner, objects: &[Value], profile: &Path, path: &Path) -> Result<String> {
    let mut args = vec![
        "export".to_owned(),
        "-format=text".to_owned(),
        format!(
            "-instr-profile={}",
            profile.to_str().ok_or(CollectError::Input)?
        ),
    ];
    for object in objects {
        args.extend(["-object".to_owned(), text(object, "path")?]);
    }
    let tool = commands.program("llvm-cov")?;
    let target = commands.target.clone();
    let actual = commands.run(&tool, &args, 180, &values(), &target)?;
    if !actual.output.status.success() {
        return Err(CollectError::Command);
    }
    runner::write(path, &actual.output.stdout)?;
    Ok(utf8(&actual.output.stdout)?.to_owned())
}

/// Resolve one same-byte Cargo dependency image and its bounded dependency file.
pub fn binary_dependency(object: &Value, deadline: Instant) -> Result<(PathBuf, String)> {
    let image = PathBuf::from(text(object, "path")?);
    let directory = image.parent().ok_or(CollectError::Artifacts)?.join("deps");
    let prefix = format!("{}-", text(object, "name")?.replace('-', "_"));
    let mut matched = None;
    for (index, entry) in fs::read_dir(directory)
        .map_err(|_| CollectError::Artifacts)?
        .enumerate()
    {
        prepared::check_time(deadline)?;
        if index >= 4096 {
            return Err(CollectError::Intake);
        }
        let entry = entry.map_err(|_| CollectError::Artifacts)?;
        let name = entry.file_name();
        let Some(suffix) = name.to_str().and_then(|s| s.strip_prefix(&prefix)) else {
            continue;
        };
        if suffix.len() != 16 || !suffix.bytes().all(|b| b.is_ascii_hexdigit()) {
            continue;
        }
        let candidate = entry.path();
        let actual = fs::symlink_metadata(&candidate).map_err(|_| CollectError::Artifacts)?;
        if !actual.is_file() {
            continue;
        }
        let identity = prepared::fingerprint(&candidate, 64 << 20, deadline)?;
        if identity["sha256"] != object["sha256"] {
            continue;
        }
        if matched.is_some() {
            return Err(CollectError::Artifacts);
        }
        let dependency = prepared::read_text(
            &candidate.with_extension("d"),
            collection_inputs::MAX_PROTOCOL_BYTES as u64,
            deadline,
        )?
        .0;
        matched = Some((candidate, dependency));
    }
    matched.ok_or(CollectError::Artifacts)
}

/// Resolve the complete registered crate-local Rust set from a physical dependency list.
pub fn compiled_sources(
    root: &Path,
    snapshot: &Value,
    dependency_target: &Path,
    dependency: &str,
) -> Result<BTreeSet<String>> {
    let mut actual = BTreeSet::new();
    for source in collection_inputs::dependencies(
        dependency,
        dependency_target.to_str().ok_or(CollectError::Artifacts)?,
    )? {
        let path = root.join(source);
        let path = path.canonicalize().map_err(|_| CollectError::Artifacts)?;
        let relative = path
            .strip_prefix(root)
            .ok()
            .and_then(Path::to_str)
            .ok_or(CollectError::Artifacts)?;
        if !snapshot["files"]
            .as_object()
            .ok_or(CollectError::Artifacts)?
            .contains_key(relative)
        {
            return Err(CollectError::Artifacts);
        }
        if snapshot["sources"]
            .as_array()
            .ok_or(CollectError::Artifacts)?
            .iter()
            .any(|s| s["path"] == relative)
        {
            actual.insert(relative.to_owned());
        } else if path.extension().is_some_and(|e| e == "rs") {
            return Err(CollectError::Artifacts);
        }
    }
    Ok(actual)
}

/// Project a complete supplied export onto the actual crate-local sources.
pub fn binary_counts(
    root: &Path,
    actual: &BTreeSet<String>,
    report: &str,
) -> Result<Map<String, Value>> {
    let report = strict_json::parse(report).ok_or(CollectError::Conservation)?;
    let mut counts = Map::new();
    for file in report["data"][0]["files"]
        .as_array()
        .ok_or(CollectError::Conservation)?
    {
        let filename = text(file, "filename")?;
        let relative = Path::new(&filename)
            .strip_prefix(root)
            .ok()
            .and_then(Path::to_str)
            .ok_or(CollectError::Conservation)?;
        if actual.contains(relative)
            && counts
                .insert(relative.to_owned(), file["summary"]["regions"].clone())
                .is_some()
        {
            return Err(CollectError::Conservation);
        }
    }
    Ok(counts)
}

fn binary_scopes(
    root: &Path,
    snapshot: &Value,
    objects: &[Value],
    profiles: &[Value],
    commands: &mut Runner,
    output: &Path,
    deadline: Instant,
) -> Result<Vec<Value>> {
    let mut result = Vec::new();
    let raw = output.join("raw");
    for object in objects.iter().filter(|o| o["role"] == "binary") {
        let name = text(object, "name")?;
        let (dependency_target, dependency) = binary_dependency(object, deadline)?;
        let actual = compiled_sources(root, snapshot, &dependency_target, &dependency)?;
        let selected = profiles
            .iter()
            .filter(|p| p["object_sha256"] == object["sha256"])
            .collect::<Vec<_>>();
        let paths = selected
            .iter()
            .map(|p| Ok(raw.join(text(p, "name")?)))
            .collect::<Result<Vec<_>>>()?;
        let merged = output.join(format!("binary-{name}.profdata"));
        merge(commands, &paths, &merged)?;
        let report = export(
            commands,
            std::slice::from_ref(object),
            &merged,
            &output.join(format!("binary-{name}.json")),
        )?;
        let counts = binary_counts(root, &actual, &report)?;
        result.push(json!({"name":name,"object_sha256":object["sha256"],"dependency_target":dependency_target,"actual_sources":actual,
            "registered_sources":object["sources"],"profile_pids":selected.iter().map(|p|p["pid"].clone()).collect::<Vec<_>>(),"counts":counts}));
    }
    prepared::check_time(deadline)?;
    Ok(result)
}

/// Final transport failure invalidates even a fully written frame.
pub fn finish(value: &Value, stdout: &mut impl Write, stderr: &mut impl Write) -> u8 {
    finish_until(
        value,
        Instant::now() + Duration::from_secs(180),
        stdout,
        stderr,
    )
}

pub fn finish_until(
    value: &Value,
    deadline: Instant,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> u8 {
    if prepared::check_time(deadline).is_ok()
        && emit(stdout, format!("{value}\n").as_bytes())
        && prepared::check_time(deadline).is_ok()
    {
        0
    } else {
        emit(stderr, OUTPUT_FAILURE);
        1
    }
}

fn collect(args: &[&str]) -> Result<(Value, Instant)> {
    let input_deadline = Instant::now() + Duration::from_secs(180);
    let [root, roles, tests, manifest, output] = args else {
        return Err(CollectError::Input);
    };
    let root = canonical(root)?;
    let root_text = root.to_str().ok_or(CollectError::Input)?;
    let roles_path = PathBuf::from(roles);
    let tests_path = PathBuf::from(tests);
    let manifest_path = PathBuf::from(manifest);
    let (roles_raw, roles_hash) = prepared::read_text(Path::new(roles), 64 << 20, input_deadline)?;
    let (tests, tests_hash) =
        prepared::read_json_until(Path::new(tests), 64 << 20, input_deadline)?;
    let (base, manifest_hash) =
        prepared::read_json_until(Path::new(manifest), 64 << 20, input_deadline)?;
    prepared::validate(&base)?;
    prepared::outer_until(&base, input_deadline)?;
    let ambient = prepared::environment()?;
    let output = fresh(&root, output)?;
    let raw = output.join("raw");
    fs::create_dir(&raw).map_err(|_| CollectError::Intake)?;
    let mut commands = Runner::new(&root, &output, &base)?;
    commands.deadline = Some(input_deadline);
    let observed = observe(&root, &base, &mut commands, input_deadline)?;
    if observed != base {
        return Err(CollectError::Prepared);
    }
    runner::record(&output.join("prepared-before.json"), &observed)?;
    // Input discovery precedes B; this actual metadata is retained and repeated
    // under B after the complete source identity has been frozen.
    let (metadata_raw, metadata_value) = metadata(&mut commands)?;
    workspace::check_workspace(&root, &metadata_raw).map_err(|_| CollectError::Artifacts)?;
    let snapshot = inventory::snapshot(&root, &roles_raw, &metadata_raw)
        .map_err(|_| CollectError::Artifacts)?;
    let before = json!({"source":snapshot["files"],"manifests":{"roles":roles_hash,"tests":tests_hash,"prepared":manifest_hash},
        "tools":observed["tools"].as_object().ok_or(CollectError::Prepared)?.iter().map(|(k,v)|(k.clone(),v["sha256"].clone())).collect::<Map<_,_>>(),
        "resources":observed["resources"],"configs":observed["configs"],
        "native":{"profile":PROFILE,"metadata_sha256":sha256(metadata_raw.as_bytes()),"features":tests["features"],"outer_environment":ambient}});
    runner::record(&output.join("source-before.json"), &before)?;
    runner::record(&output.join("source-inventory.json"), &snapshot)?;
    let (context_path, request, mut identity) = context(&output, &before, input_deadline)?;
    let context_before = prepared::tree(
        &context_path,
        prepared::Limits {
            files: 4096,
            per_file: 65536,
            total: 256 << 20,
            depth: 1,
        },
        input_deadline,
    )?;
    commands.request = Some(request);
    commands.context = Some(context_path.clone());
    let (repeated_raw, repeated_metadata) = metadata(&mut commands)?;
    if repeated_metadata != metadata_value
        || inventory::snapshot(&root, &roles_raw, &repeated_raw)
            .map_err(|_| CollectError::Artifacts)?
            != snapshot
    {
        return Err(CollectError::Changed);
    }
    prepared::check_time(input_deadline)?;
    commands.deadline = None;
    let normal = output.join("normal-target");
    for action in [Action::Format, Action::Check, Action::Clippy] {
        commands.success(
            "cargo",
            action.cargo_args(),
            action.operation_budget().as_secs(),
            &values(),
            &normal,
        )?;
    }
    let target = commands.target.clone();
    let show = commands.success(
        "cargo-llvm-cov",
        &["llvm-cov", "show-env"],
        30,
        &values(),
        &target,
    )?;
    let mut crate_names = BTreeSet::new();
    for package in metadata_value["packages"]
        .as_array()
        .ok_or(CollectError::Artifacts)?
    {
        if metadata_value["workspace_members"]
            .as_array()
            .ok_or(CollectError::Artifacts)?
            .contains(&package["id"])
        {
            let name = text(package, "name")?.replace('-', "_");
            crate_names.insert(name.clone());
            crate_names.insert(format!("{name}_tests"));
            for target in package["targets"]
                .as_array()
                .ok_or(CollectError::Artifacts)?
            {
                crate_names.insert(text(target, "name")?.replace('-', "_"));
            }
        }
    }
    let wrapper = text(&base["tools"]["cargo-llvm-cov"], "program")?;
    let mut instrumentation = collection_inputs::environment(
        utf8(&show.output.stdout)?,
        target.to_str().ok_or(CollectError::Input)?,
        &wrapper,
        &crate_names,
        root.file_name()
            .and_then(|name| name.to_str())
            .ok_or(CollectError::Input)?,
    )?;
    instrumentation.insert(
        "LLVM_PROFILE_FILE".to_owned(),
        format!(
            "{}/arboresce-%p-%12m.profraw",
            raw.to_str().ok_or(CollectError::Input)?
        ),
    );
    runner::record(
        &output.join("instrumentation.json"),
        &json!(instrumentation),
    )?;
    let build = commands.success(
        "cargo",
        &[
            "test",
            "--locked",
            "--offline",
            "--workspace",
            "--all-targets",
            "--no-run",
            "--message-format=json",
        ],
        300,
        &instrumentation,
        &target,
    )?;
    let selection = collection_artifacts::select(
        root_text,
        target.to_str().ok_or(CollectError::Input)?,
        &metadata_value,
        &tests,
        utf8(&build.output.stdout)?,
    )?;
    let object_deadline = Instant::now() + Duration::from_secs(180);
    let objects = objects(&selection, object_deadline)?;
    let mut modules = Vec::new();
    for object in &objects {
        let path = PathBuf::from(text(object, "path")?);
        let (bytes, hash) = prepared::read_bytes(&path, 64 << 20, object_deadline)?;
        if object["sha256"] != hash {
            return Err(CollectError::Changed);
        }
        modules.push(collection_modules::object(&bytes)?);
    }
    collection_modules::unique(&modules)?;
    prepared::check_time(object_deadline)?;
    runner::record(&output.join("objects.json"), &json!(objects))?;
    let mut command_roles = Vec::new();
    let mut test_count = 0;
    for object in objects.iter().filter(|o| o["role"] == "harness") {
        let program = PathBuf::from(text(object, "path")?);
        let args = ["--list", "--format=terse"].map(str::to_owned);
        let listed = commands.run(&program, &args, 30, &instrumentation, &target)?;
        if !listed.output.status.success()
            || collection_tests::list(utf8(&listed.output.stdout)?)?
                != collection_tests::selected_names(&object["tests"])?
        {
            return Err(CollectError::Selection);
        }
        command_roles
            .push(json!({"pid":listed.pid,"role":"list","object_sha256":object["sha256"]}));
        let args = ["--show-output", "--test-threads=1", "--color=never"].map(str::to_owned);
        let run = commands.run(
            &program,
            &args,
            if object["required"] == true { 300 } else { 30 },
            &instrumentation,
            &target,
        )?;
        if !run.output.status.success() {
            return Err(CollectError::Execution);
        }
        let events =
            collection_tests::execution(utf8(&run.output.stdout)?, &object["tests"], run.pid)?;
        test_count += events.len();
        runner::record(
            &output.join(format!(
                "harness-{}-{}-{}.json",
                text(object, "package")?,
                text(object, "kind")?,
                text(object, "name")?
            )),
            &json!({"pid":run.pid,"events":events}),
        )?;
        command_roles.push(json!({"pid":run.pid,"role":"run","object_sha256":object["sha256"]}));
    }
    let exercise_deadline = Instant::now() + Duration::from_secs(180);
    commands.deadline = Some(exercise_deadline);
    let mut exercised = Vec::new();
    for object in objects.iter().filter(|o| o["role"] == "binary") {
        let program = PathBuf::from(text(object, "path")?);
        for exercise in collection_artifacts::exercises(object)? {
            let actual =
                commands.run(&program, &exercise.arguments, 30, &instrumentation, &target)?;
            exercised.push(json!({"pid":actual.pid,"object_sha256":object["sha256"],
                "arguments":exercise.arguments,"exit":actual.output.status.code(),
                "stdout_size":actual.output.stdout.len(),"stdout_sha256":sha256(&actual.output.stdout),
                "stderr_size":actual.output.stderr.len(),"stderr_sha256":sha256(&actual.output.stderr)}));
            if actual.output.status.code() != Some(exercise.exit)
                || actual.output.stdout != exercise.stdout
                || actual.output.stderr != exercise.stderr
            {
                runner::record(&output.join("native-exercises.json"), &json!(exercised))?;
                return Err(CollectError::Execution);
            }
        }
    }
    prepared::check_time(exercise_deadline)?;
    runner::record(&output.join("native-exercises.json"), &json!(exercised))?;
    commands.deadline = None;
    for doc in tests["doctests"]
        .as_array()
        .ok_or(CollectError::Selection)?
    {
        let package = text(doc, "package")?;
        let actual = commands.success(
            "cargo",
            &[
                "test",
                "--locked",
                "--offline",
                "--doc",
                "-p",
                &package,
                "--",
                "--show-output",
                "--test-threads=1",
                "--color=never",
            ],
            180,
            &values(),
            &normal,
        )?;
        if doc["tests"] != 0
            || !collection_tests::execution(utf8(&actual.output.stdout)?, &json!([]), actual.pid)?
                .is_empty()
        {
            return Err(CollectError::Selection);
        }
    }
    let profile_deadline = Instant::now() + Duration::from_secs(180);
    commands.deadline = Some(profile_deadline);
    let raw_before = prepared::tree(&raw, prepared::PROFILES, profile_deadline)?;
    let launches = receipt_directory(
        &context_path.join("launches"),
        "native-launch-v1",
        profile_deadline,
    )?;
    let mut profiles = Vec::new();
    let mut raw_paths = Vec::new();
    for (name, row) in raw_before.as_object().ok_or(CollectError::Conservation)? {
        let pid = profile_pid(name)?;
        let launch = launches
            .iter()
            .find(|l| l["record"]["pid"] == pid)
            .ok_or(CollectError::Conservation)?;
        let path = raw.join(name);
        let (bytes, hash) = prepared::read_bytes(&path, 64 << 20, profile_deadline)?;
        if row["sha256"] != hash {
            return Err(CollectError::Conservation);
        }
        let index = objects
            .iter()
            .position(|object| object["sha256"] == launch["record"]["executable_sha256"])
            .ok_or(CollectError::Conservation)?;
        if collection_modules::profile(&bytes)? != modules[index] {
            return Err(CollectError::Conservation);
        }
        let name_text = path.to_str().ok_or(CollectError::Input)?;
        let show = commands.success(
            "llvm-profdata",
            &["show", "--all-functions", "--counts", name_text],
            30,
            &values(),
            &target,
        )?;
        let counts = collection_profiles::inspect(utf8(&show.output.stdout)?)?;
        let modified = fs::metadata(&path)
            .and_then(|m| m.modified())
            .map_err(|_| CollectError::Intake)?
            .duration_since(UNIX_EPOCH)
            .map_err(|_| CollectError::Intake)?
            .as_nanos();
        let mut profile = row.clone();
        profile["name"] = name.clone().into();
        profile["pid"] = pid.into();
        profile["object_sha256"] = launch["record"]["executable_sha256"].clone();
        profile["after_sha256"] = row["sha256"].clone();
        profile["modified_ns"] = u64::try_from(modified)
            .map_err(|_| CollectError::Intake)?
            .into();
        profile["functions"] = counts["functions"].clone();
        profile["positive_functions"] = counts["positive_functions"].clone();
        profiles.push(profile);
        raw_paths.push(path);
    }
    let merged = output.join("all.profdata");
    merge(&mut commands, &raw_paths, &merged)?;
    let report = export(
        &mut commands,
        &objects,
        &merged,
        &output.join("coverage-complete.json"),
    )?;
    let gate = coverage::evaluate(root_text, &snapshot["coverage_roles"].to_string(), &report)
        .map_err(|_| CollectError::Conservation)?;
    let scopes = binary_scopes(
        &root,
        &snapshot,
        &objects,
        &profiles,
        &mut commands,
        &output,
        profile_deadline,
    )?;
    prepared::check_time(profile_deadline)?;
    let final_deadline = Instant::now() + Duration::from_secs(180);
    commands.deadline = Some(final_deadline);
    let final_prepared = observe(&root, &base, &mut commands, final_deadline)?;
    runner::record(&output.join("prepared-after.json"), &final_prepared)?;
    let (final_metadata, final_metadata_value) = metadata(&mut commands)?;
    let final_snapshot = inventory::snapshot(&root, &roles_raw, &final_metadata)
        .map_err(|_| CollectError::Changed)?;
    let mut after = before.clone();
    after["source"] = final_snapshot["files"].clone();
    after["resources"] = final_prepared["resources"].clone();
    after["configs"] = final_prepared["configs"].clone();
    after["tools"] = Value::Object(
        final_prepared["tools"]
            .as_object()
            .ok_or(CollectError::Prepared)?
            .iter()
            .map(|(k, v)| (k.clone(), v["sha256"].clone()))
            .collect(),
    );
    after["manifests"] = json!({"roles":prepared::read_json_until(&roles_path,64<<20,final_deadline)?.1,
        "tests":prepared::read_json_until(&tests_path,64<<20,final_deadline)?.1,"prepared":prepared::read_json_until(&manifest_path,64<<20,final_deadline)?.1});
    after["native"]["metadata_sha256"] = sha256(final_metadata.as_bytes()).into();
    if final_metadata_value != metadata_value
        || final_prepared != base
        || prepared::environment()? != ambient
    {
        return Err(CollectError::Changed);
    }
    runner::record(&output.join("source-after.json"), &after)?;
    let final_raw = prepared::tree(&raw, prepared::PROFILES, final_deadline)?;
    for object in &objects {
        prepared::unchanged_object(
            Path::new(&text(object, "path")?),
            &object["sha256"],
            final_deadline,
        )?;
    }
    if final_raw != raw_before {
        return Err(CollectError::Conservation);
    }
    let mut launches = receipt_directory(
        &context_path.join("launches"),
        "native-launch-v1",
        final_deadline,
    )?;
    let exits = receipt_directory(
        &context_path.join("exits"),
        "native-exit-v1",
        final_deadline,
    )?;
    for launch in &mut launches {
        let record = &launch["record"];
        let hash = &record["executable_sha256"];
        let known = objects
            .iter()
            .find(|o| o["sha256"] == *hash)
            .map(|o| &o["path"])
            .or_else(|| {
                base["tools"]
                    .as_object()
                    .and_then(|m| m.values().find(|t| t["sha256"] == *hash))
                    .map(|t| &t["executable"])
            });
        if !known
            .and_then(Value::as_str)
            .is_some_and(|p| record["executable"] == json!(p.as_bytes()))
        {
            return Err(CollectError::Conservation);
        }
    }
    identity["finished_ns"] = now()?.into();
    let facts = json!({"version":"native-collection-facts-v1","context":identity,"inputs_before":before,"inputs_after":after,
        "objects":objects,"commands":command_roles,"launches":launches,"exits":exits,"profiles":profiles,"raw_after":final_raw,"binary_scopes":scopes});
    runner::record(&output.join("collection-facts.json"), &facts)?;
    let counts = collection_receipts::evaluate(&facts)?;
    // The immutable context file remains identical; receipt directories only gained records.
    let final_context =
        prepared::read_json_until(&context_path.join("context.json"), 32768, final_deadline)?.1;
    if context_before["context.json"]["sha256"] != final_context {
        return Err(CollectError::Conservation);
    }
    prepared::outer_until(&base, final_deadline)?;
    let context_files = prepared::tree(
        &context_path,
        prepared::Limits {
            files: 8193,
            per_file: 65536,
            total: 512 << 20,
            depth: 1,
        },
        final_deadline,
    )?;
    let entries = fs::read_dir(&context_path)
        .map_err(|_| CollectError::Intake)?
        .map(|entry| entry.map(|entry| entry.file_name()))
        .collect::<std::io::Result<BTreeSet<_>>>()
        .map_err(|_| CollectError::Intake)?;
    if entries
        != BTreeSet::from([
            OsString::from("context.json"),
            OsString::from("launches"),
            OsString::from("exits"),
        ])
    {
        return Err(CollectError::Conservation);
    }
    let mut expected = BTreeMap::from([("context.json".to_owned(), final_context)]);
    for (directory, key) in [("launches", "launches"), ("exits", "exits")] {
        for wrapper in facts[key].as_array().ok_or(CollectError::Conservation)? {
            expected.insert(
                format!(
                    "{directory}/{}.json",
                    wrapper["record"]["pid"]
                        .as_u64()
                        .ok_or(CollectError::Conservation)?
                ),
                text(wrapper, "sha256")?,
            );
        }
    }
    let actual = context_files
        .as_object()
        .ok_or(CollectError::Conservation)?
        .iter()
        .map(|(key, row)| Ok((key.clone(), text(row, "sha256")?)))
        .collect::<Result<BTreeMap<_, _>>>()?;
    if actual != expected {
        return Err(CollectError::Conservation);
    }
    let result = json!({"version":"native-verification-result-v1","status":"PASS","required_suites":counts["required_suites"],"tests":test_count,
        "objects":counts["objects"],"profiles":counts["profiles"],"native_binaries":counts["native_binaries"],"strict_scopes":gate.scopes.len()+scopes.len()});
    runner::record(&output.join("result.json"), &result)?;
    if prepared::tree(
        &context_path,
        prepared::Limits {
            files: 8193,
            per_file: 65536,
            total: 512 << 20,
            depth: 1,
        },
        final_deadline,
    )? != context_files
    {
        return Err(CollectError::Conservation);
    }
    prepared::check_time(final_deadline)?;
    Ok((result, final_deadline))
}

pub fn run(args: &[OsString], stdout: &mut impl Write, stderr: &mut impl Write) -> u8 {
    if args.len() == 1 && args[0] == "--help" {
        return if emit(stdout, USAGE.as_bytes()) {
            0
        } else {
            emit(stderr, OUTPUT_FAILURE);
            1
        };
    }
    let args = match args
        .iter()
        .map(|s| s.to_str().ok_or(CollectError::Input))
        .collect::<Result<Vec<_>>>()
    {
        Ok(args) => args,
        Err(error) => {
            emit(stderr, format!("arboresce-verify: {error}\n").as_bytes());
            return 1;
        }
    };
    let prepared = args.first() == Some(&"prepare");
    if (!prepared && args.len() != 5) || (prepared && ![6, 7].contains(&args.len())) {
        return if emit(stderr, USAGE.as_bytes()) { 2 } else { 1 };
    }
    let result = if prepared {
        prepare(&args[1..]).map(|()| None)
    } else {
        collect(&args).map(Some)
    };
    match result {
        Ok(None) => 0,
        Ok(Some((value, deadline))) => finish_until(&value, deadline, stdout, stderr),
        Err(error) => {
            emit(stderr, format!("arboresce-verify: {error}\n").as_bytes());
            1
        }
    }
}
