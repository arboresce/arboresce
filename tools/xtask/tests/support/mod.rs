use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};

static NEXT: AtomicU64 = AtomicU64::new(0);

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
    arboresce_xtask::process::observe(
        &mut child,
        &stdout_path,
        &stderr_path,
        arboresce_xtask::process::Observation {
            budget: options.budget,
            poll: options.poll,
            initial_delay: options.initial_delay,
        },
        started,
        &mut std::io::stdout().lock(),
    )
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

/// A one-shot trace fault and optional real delay at a complete line boundary.
pub struct ObservationTrace {
    pub bytes: Vec<u8>,
    pub fail_after_lines: Option<usize>,
    pub delay_after_line: usize,
    pub delay: Duration,
    failed: bool,
    lines: usize,
}

impl ObservationTrace {
    pub fn new(fail_after_lines: Option<usize>, delay_after_line: usize, delay: Duration) -> Self {
        Self {
            bytes: Vec::new(),
            fail_after_lines,
            delay_after_line,
            delay,
            failed: false,
            lines: 0,
        }
    }
}

impl Write for ObservationTrace {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if !self.failed && self.fail_after_lines == Some(self.lines) {
            self.failed = true;
            return Err(io::Error::other("controlled trace write failure"));
        }
        self.bytes.extend_from_slice(bytes);
        for byte in bytes {
            if *byte == b'\n' {
                self.lines += 1;
                if self.lines == self.delay_after_line {
                    std::thread::sleep(self.delay);
                }
            }
        }
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
#[cfg(unix)]
pub mod inventory_inputs;
pub mod report_inputs;
