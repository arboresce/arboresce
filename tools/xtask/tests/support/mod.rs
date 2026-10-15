use std::{
    fs,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    process::{Child, Command, Output, Stdio},
    sync::atomic::{AtomicU64, Ordering},
    thread,
    time::{Duration, Instant},
};

static NEXT: AtomicU64 = AtomicU64::new(0);
const MAX_OUTPUT_BYTES: u64 = 8 * 1024 * 1024;

pub struct PreparedRoot {
    pub path: PathBuf,
}

impl PreparedRoot {
    pub fn new() -> io::Result<Self> {
        let parent = Path::new(env!("CARGO_TARGET_TMPDIR"));
        fs::create_dir_all(parent)?;
        let path = parent.join(format!(
            "arboresce-xtask-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path)?;
        Ok(Self { path })
    }

    /// Generate syntax-policy inputs, not compiled or executable product code.
    pub fn metadata(&self) -> io::Result<serde_json::Value> {
        let mut packages = Vec::new();
        let mut members = Vec::new();
        let mut nodes = Vec::new();
        for (name, source) in [
            ("arboresce", "pub fn n() { println!(\"Arboresce\"); }"),
            ("arboresce-domain", "pub fn n() -> u8 { 1 }"),
        ] {
            let directory = self.path.join("crates").join(name);
            fs::create_dir_all(directory.join("src"))?;
            fs::write(
                directory.join("Cargo.toml"),
                format!("[package]\nname=\"{name}\"\nversion=\"0.0.0\"\nedition=\"2024\"\n"),
            )?;
            fs::write(directory.join("src/lib.rs"), source)?;
            packages.push(serde_json::json!({
                "id":name,"name":name,"version":"0.0.0","manifest_path":directory.join("Cargo.toml"),
                "targets":[{"name":name,"kind":["lib"],"src_path":directory.join("src/lib.rs")}]
            }));
            members.push(name);
            nodes.push(serde_json::json!({"id":name,"deps":[]}));
        }
        Ok(
            serde_json::json!({"workspace_root":self.path,"packages":packages,"workspace_members":members,"resolve":{"nodes":nodes}}),
        )
    }

    #[cfg(unix)]
    pub fn cargo_peer(&self) -> io::Result<PathBuf> {
        let directory = self.path.join("bin");
        fs::create_dir(&directory)?;
        std::os::unix::fs::symlink(
            env!("CARGO_BIN_EXE_arboresce-xtask-test-peer"),
            directory.join("cargo"),
        )?;
        Ok(directory)
    }
}

impl Drop for PreparedRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[derive(Clone, Copy)]
pub enum ClosedStream {
    Stdout,
    Stderr,
}

pub struct Observation {
    pub budget: Duration,
    pub poll: Duration,
    pub initial_delay: Duration,
    pub closed: Option<ClosedStream>,
}

impl Default for Observation {
    fn default() -> Self {
        Self {
            budget: Duration::from_secs(30),
            poll: Duration::from_millis(5),
            initial_delay: Duration::ZERO,
            closed: None,
        }
    }
}

fn closed_pipe() -> io::Result<Stdio> {
    let (reader, writer) = io::pipe()?;
    drop(reader);
    Ok(Stdio::from(writer))
}

fn read_output(path: &Path) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    fs::File::open(path)?
        .take(MAX_OUTPUT_BYTES + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_OUTPUT_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "native output limit exceeded",
        ));
    }
    Ok(bytes)
}

fn cleanup(child: &mut Child) -> io::Result<()> {
    let initial = child.try_wait()?;
    println!(
        "xtask cleanup observation {}",
        serde_json::json!({"pid":child.id(),"was_running":initial.is_none(),"exit":initial.and_then(|status| status.code())})
    );
    if initial.is_none() {
        child.kill()?;
        let deadline = Instant::now() + Duration::from_secs(2);
        while child.try_wait()?.is_none() {
            if Instant::now() >= deadline {
                return Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "native cleanup did not finish",
                ));
            }
            thread::sleep(Duration::from_millis(1));
        }
    }
    println!("xtask cleanup pid={} reaped=true", child.id());
    Ok(())
}

/// Actual native processes share one bounded observation/collection path.
/// Regular task-owned files avoid blocking pipe-reader joins and inherited pipes.
pub fn observe(command: &mut Command) -> io::Result<Output> {
    observe_with(command, Observation::default())
}

pub fn observe_with(command: &mut Command, options: Observation) -> io::Result<Output> {
    let prepared = PreparedRoot::new()?;
    let stdout_path = prepared.path.join("stdout");
    let stderr_path = prepared.path.join("stderr");
    let stdout = fs::File::create(&stdout_path)?;
    let stderr = fs::File::create(&stderr_path)?;
    command.stdout(if matches!(options.closed, Some(ClosedStream::Stdout)) {
        closed_pipe()?
    } else {
        Stdio::from(stdout)
    });
    command.stderr(if matches!(options.closed, Some(ClosedStream::Stderr)) {
        closed_pipe()?
    } else {
        Stdio::from(stderr)
    });
    let started = Instant::now();
    let mut child = command.spawn()?;
    let process_id = child.id();
    let deadline = started + options.budget;
    thread::sleep(options.initial_delay);
    let result = (|| {
        loop {
            if Instant::now() >= deadline {
                return Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "native outcome observed too late",
                ));
            }
            if let Some(status) = child.try_wait()? {
                println!(
                    "xtask native exited pid={process_id} exit={:?}",
                    status.code()
                );
                let stdout = read_output(&stdout_path)?;
                let stderr = read_output(&stderr_path)?;
                if Instant::now() >= deadline {
                    return Err(io::Error::new(
                        io::ErrorKind::TimedOut,
                        "native collection finished too late",
                    ));
                }
                println!("xtask native pid={process_id} exit={:?}", status.code());
                println!(
                    "xtask observation {}",
                    serde_json::json!({"pid":process_id,"exit":status.code(),"stdout":stdout,"stderr":stderr,"within_budget":true})
                );
                return Ok(Output {
                    status,
                    stdout,
                    stderr,
                });
            }
            thread::sleep(options.poll);
        }
    })();
    if let Err(ref error) = result {
        cleanup(&mut child)?;
        println!(
            "xtask native pid={process_id} error={:?} cleanup=reaped",
            error.kind()
        );
    }
    result
}

pub struct FailedOutput {
    pub write_fails: bool,
    pub bytes: Vec<u8>,
}
impl Write for FailedOutput {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.write_fails {
            Err(io::Error::other("write failed"))
        } else {
            self.bytes.extend_from_slice(bytes);
            Ok(bytes.len())
        }
    }
    fn flush(&mut self) -> io::Result<()> {
        Err(io::Error::other("flush failed"))
    }
}
pub mod report_inputs;
