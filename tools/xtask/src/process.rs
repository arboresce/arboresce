use std::{
    fs,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    process::{Child, Command, ExitStatus, Output, Stdio},
    sync::atomic::{AtomicU64, Ordering},
    thread,
    time::{Duration, Instant},
};

const MAX_OUTPUT_BYTES: u64 = 8 * 1024 * 1024;

pub struct Observation {
    pub budget: Duration,
    pub poll: Duration,
    pub initial_delay: Duration,
}

impl Default for Observation {
    fn default() -> Self {
        Self {
            budget: Duration::from_secs(30),
            poll: Duration::from_millis(5),
            initial_delay: Duration::ZERO,
        }
    }
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

fn cleanup(child: &mut Child, trace: &mut impl Write) -> io::Result<()> {
    let initial = child.try_wait()?;
    let first_trace = writeln!(
        trace,
        "xtask cleanup observation {}",
        serde_json::json!({"pid":child.id(),"was_running":initial.is_none(),"exit":initial.and_then(|status| status.code())})
    );
    if initial.is_none() {
        terminate(child)?;
    }
    let last_trace = writeln!(trace, "xtask cleanup pid={} reaped=true", child.id());
    first_trace.and(last_trace)
}

/// Observe an owned child through bounded task-owned files and retain its trace.
/// The caller prepares streams and owns the temporary directory and child.
pub fn observe(
    child: &mut Child,
    stdout_path: &Path,
    stderr_path: &Path,
    options: Observation,
    started: Instant,
    trace: &mut impl Write,
) -> io::Result<Output> {
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
                writeln!(
                    trace,
                    "xtask native exited pid={process_id} exit={:?}",
                    status.code()
                )?;
                let stdout = read_output(stdout_path)?;
                let stderr = read_output(stderr_path)?;
                if Instant::now() >= deadline {
                    return Err(io::Error::new(
                        io::ErrorKind::TimedOut,
                        "native collection finished too late",
                    ));
                }
                writeln!(
                    trace,
                    "xtask native pid={process_id} exit={:?}",
                    status.code()
                )?;
                writeln!(
                    trace,
                    "xtask observation {}",
                    serde_json::json!({"pid":process_id,"exit":status.code(),"stdout":stdout,"stderr":stderr,"within_budget":true})
                )?;
                if Instant::now() >= deadline {
                    return Err(io::Error::new(
                        io::ErrorKind::TimedOut,
                        "native trace collection finished too late",
                    ));
                }
                return Ok(Output {
                    status,
                    stdout,
                    stderr,
                });
            }
            thread::sleep(options.poll);
        }
    })();
    let result = (|| {
        if let Err(ref error) = result {
            cleanup(child, trace)?;
            writeln!(
                trace,
                "xtask native pid={process_id} error={:?} cleanup=reaped",
                error.kind()
            )?;
        }
        result
    })();
    let recorded = crate::measurement::complete(child);
    result.and_then(|output| {
        recorded?;
        if Instant::now() >= deadline {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "native completion recorded too late",
            ));
        }
        Ok(output)
    })
}

/// Kill/reap only this owned Child, including when it has already exited.
pub fn terminate(child: &mut Child) -> io::Result<ExitStatus> {
    if let Some(status) = child.try_wait()? {
        return Ok(status);
    }
    child.kill()?;
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        if let Some(status) = child.try_wait()? {
            return Ok(status);
        }
        if Instant::now() >= deadline {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "native cleanup did not finish",
            ));
        }
        thread::sleep(Duration::from_millis(1));
    }
}

/// Bounded delegated status retains inherited streams and real child status.
pub fn wait(child: &mut Child, budget: Duration) -> io::Result<ExitStatus> {
    let deadline = Instant::now() + budget;
    loop {
        if Instant::now() >= deadline {
            terminate(child)?;
            crate::measurement::complete(child)?;
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "native status observed too late",
            ));
        }
        if let Some(status) = child.try_wait()? {
            crate::measurement::complete(child)?;
            if Instant::now() >= deadline {
                return Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "native status recorded too late",
                ));
            }
            return Ok(status);
        }
        thread::sleep(Duration::from_millis(1));
    }
}

static NEXT_CAPTURE: AtomicU64 = AtomicU64::new(0);
struct Capture(PathBuf);
impl Drop for Capture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// Capture owned streams without pipe joins, using the shared bounded observer.
pub fn output(command: &mut Command) -> io::Result<Output> {
    let path = std::env::temp_dir().join(format!(
        "arboresce-native-{}-{}",
        std::process::id(),
        NEXT_CAPTURE.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&path)?;
    let capture = Capture(path);
    let stdout = capture.0.join("stdout");
    let stderr = capture.0.join("stderr");
    command
        .stdin(Stdio::null())
        .stdout(Stdio::from(fs::File::create_new(&stdout)?))
        .stderr(Stdio::from(fs::File::create_new(&stderr)?));
    let started = Instant::now();
    let mut child = crate::measurement::spawn(command)?;
    observe(
        &mut child,
        &stdout,
        &stderr,
        Observation::default(),
        started,
        &mut Vec::new(),
    )
}
