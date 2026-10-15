use std::{
    fs,
    io::{self, Read, Write},
    path::Path,
    process::{Child, Output},
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
    if let Err(ref error) = result {
        cleanup(child, trace)?;
        writeln!(
            trace,
            "xtask native pid={process_id} error={:?} cleanup=reaped",
            error.kind()
        )?;
    }
    result
}
