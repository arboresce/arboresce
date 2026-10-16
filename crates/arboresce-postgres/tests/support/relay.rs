//! Bounded actual Unix traffic relay and independently checked frame policy.

use std::{
    fs,
    os::unix::{
        fs::{DirBuilderExt, PermissionsExt},
        net::UnixListener,
    },
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicU64, Ordering},
        mpsc,
    },
    thread::{self, JoinHandle},
};
use std::{
    io::{self, Read, Write},
    net::Shutdown,
    os::unix::net::UnixStream,
    time::{Duration, Instant},
};

static NEXT: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy)]
pub enum Mode {
    Commit,
    Rollback,
}

#[derive(Default, Debug)]
pub struct Stats {
    pub bytes: u64,
    pub frames: u64,
    pub dropped: u64,
}

fn invalid() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        "relay protocol or resource limit",
    )
}

pub fn body_length(declared: u32) -> io::Result<usize> {
    if !(4..=1_048_575).contains(&declared) {
        return Err(invalid());
    }
    Ok((declared - 4) as usize)
}

pub fn classify(mode: Mode, kind: u8, declared: u32, body: &[u8]) -> io::Result<bool> {
    if body_length(declared)? != body.len() {
        return Err(invalid());
    }
    if kind != b'C' {
        return Ok(false);
    }
    let Some(tag) = body.strip_suffix(&[0]) else {
        return Err(invalid());
    };
    if tag.is_empty() || tag.contains(&0) {
        return Err(invalid());
    }
    Ok(matches!(
        (mode, tag),
        (Mode::Commit, b"COMMIT") | (Mode::Rollback, b"ROLLBACK")
    ))
}

pub fn account(bytes: u64, frames: u64, incoming: u64, server: bool) -> io::Result<(u64, u64)> {
    let bytes = bytes.checked_add(incoming).ok_or_else(invalid)?;
    let frames = frames.checked_add(u64::from(server)).ok_or_else(invalid)?;
    if bytes > 8_388_608 || frames > 4096 {
        return Err(invalid());
    }
    Ok((bytes, frames))
}

fn remaining(deadline: Instant) -> io::Result<Duration> {
    let remaining = deadline.saturating_duration_since(Instant::now());
    if remaining.is_zero() {
        return Err(io::Error::new(io::ErrorKind::TimedOut, "relay deadline"));
    }
    Ok(remaining.min(Duration::from_millis(2000)))
}

enum Transfer<'a> {
    Read(&'a mut [u8]),
    Write(&'a [u8]),
}

impl Transfer<'_> {
    fn timeout(&self, stream: &UnixStream, remaining: Duration) -> io::Result<()> {
        match self {
            Self::Read(_) => stream.set_read_timeout(Some(remaining)),
            Self::Write(_) => stream.set_write_timeout(Some(remaining)),
        }
    }

    fn perform(&mut self, stream: &mut UnixStream) -> io::Result<usize> {
        match self {
            Self::Read(bytes) => stream.read(bytes),
            Self::Write(bytes) => stream.write(bytes),
        }
    }
}

/// Both directions use the same deadline and closed-peer policy.
fn transfer(
    stream: &mut UnixStream,
    mut operation: Transfer<'_>,
    deadline: Instant,
) -> io::Result<usize> {
    loop {
        let result = match operation.timeout(stream, remaining(deadline)?) {
            Ok(()) => operation.perform(stream),
            Err(error) if error.kind() == io::ErrorKind::InvalidInput => {
                // Some kernels reject socket options after peer shutdown. Probe
                // only nonblocking I/O; a failed timeout never permits blocking I/O.
                stream.set_nonblocking(true)?;
                let result = operation.perform(stream);
                stream.set_nonblocking(false)?;
                if result
                    .as_ref()
                    .is_err_and(|e| e.kind() == io::ErrorKind::WouldBlock)
                {
                    return Err(error);
                }
                result
            }
            Err(error) => return Err(error),
        };
        match result {
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            value => return value,
        }
    }
}

fn read_some(stream: &mut UnixStream, bytes: &mut [u8], deadline: Instant) -> io::Result<usize> {
    transfer(stream, Transfer::Read(bytes), deadline)
}

fn read_all(stream: &mut UnixStream, mut bytes: &mut [u8], deadline: Instant) -> io::Result<()> {
    while !bytes.is_empty() {
        let count = read_some(stream, bytes, deadline)?;
        if count == 0 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "truncated relay frame",
            ));
        }
        bytes = &mut bytes[count..];
    }
    Ok(())
}

fn write_all(stream: &mut UnixStream, mut bytes: &[u8], deadline: Instant) -> io::Result<()> {
    while !bytes.is_empty() {
        let count = transfer(stream, Transfer::Write(bytes), deadline)?;
        if count == 0 {
            return Err(io::Error::new(
                io::ErrorKind::WriteZero,
                "relay write unavailable",
            ));
        }
        bytes = &bytes[count..];
    }
    Ok(())
}

pub fn server(
    source: &mut UnixStream,
    target: &mut UnixStream,
    mode: Mode,
    deadline: Instant,
) -> io::Result<Stats> {
    let mut stats = Stats::default();
    loop {
        let mut header = [0u8; 5];
        if read_some(source, &mut header[..1], deadline)? == 0 {
            return Ok(stats);
        }
        read_all(source, &mut header[1..], deadline)?;
        let declared = u32::from_be_bytes([header[1], header[2], header[3], header[4]]);
        let size = body_length(declared)?;
        let (bytes, frames) = account(stats.bytes, stats.frames, (size + 5) as u64, true)?;
        let mut body = vec![0u8; size];
        read_all(source, &mut body, deadline)?;
        stats.bytes = bytes;
        stats.frames = frames;
        if classify(mode, header[0], declared, &body)? {
            stats.dropped = 1;
            return Ok(stats);
        }
        write_all(target, &header, deadline)?;
        write_all(target, &body, deadline)?;
    }
}

pub fn client(
    source: &mut UnixStream,
    target: &mut UnixStream,
    deadline: Instant,
) -> io::Result<Stats> {
    let mut stats = Stats::default();
    let mut buffer = [0u8; 65536];
    loop {
        let count = read_some(source, &mut buffer, deadline)?;
        if count == 0 {
            return Ok(stats);
        }
        (stats.bytes, stats.frames) = account(stats.bytes, stats.frames, count as u64, false)?;
        write_all(target, &buffer[..count], deadline)?;
    }
}

/// Closing both directions bounds the peer worker after loss or failure.
pub fn close(first: &UnixStream, second: &UnixStream) {
    let _ = first.shutdown(Shutdown::Both);
    let _ = second.shutdown(Shutdown::Both);
}

pub struct Physical {
    pub result: io::Result<Stats>,
    pub forwarded: Vec<u8>,
    pub elapsed: Duration,
    pub peer: io::Result<PhysicalPeer>,
}

#[derive(Debug, Default)]
pub struct PhysicalPeer {
    pub boundary_acknowledged: bool,
    pub extra_byte_sent: bool,
}

/// Real Unix peers drive the admitted framing and resource boundaries.
pub fn physical(id: &str) -> io::Result<Physical> {
    let (mut source, mut producer) = UnixStream::pair()?;
    let (mut target, mut consumer) = UnixStream::pair()?;
    producer.set_write_timeout(Some(Duration::from_secs(2)))?;
    consumer.set_read_timeout(Some(Duration::from_secs(12)))?;
    let (received, complete) = mpsc::channel();
    thread::scope(|scope| {
        let reader = thread::Builder::new()
            .stack_size(1 << 20)
            .spawn_scoped(scope, move || {
                let mut bytes = Vec::new();
                let mut received = Some(received);
                let mut buffer = [0; 65536];
                let result = (|| {
                    while bytes.len() < 8_388_609 {
                        let limit = buffer.len().min(8_388_609 - bytes.len());
                        let count = consumer.read(&mut buffer[..limit])?;
                        if count == 0 {
                            break;
                        }
                        bytes.extend_from_slice(&buffer[..count]);
                        if bytes.len() >= 8_388_608
                            && let Some(sender) = received.take()
                        {
                            let _ = sender.send(());
                        }
                    }
                    Ok::<_, io::Error>(())
                })();
                (bytes, result)
            })?;
        let writer = thread::Builder::new()
            .stack_size(1 << 20)
            .spawn_scoped(scope, move || {
                let mut witness = PhysicalPeer::default();
                match id {
                    "migration-relay-physical-client-bytes-exact"
                    | "migration-relay-physical-client-bytes-one-over" => {
                        let chunk = [0; 65536];
                        for _ in 0..128 {
                            producer.write_all(&chunk)?;
                        }
                        complete
                            .recv_timeout(Duration::from_secs(2))
                            .map_err(|_| invalid())?;
                        witness.boundary_acknowledged = true;
                        if id.ends_with("one-over") {
                            producer.write_all(&[0])?;
                            witness.extra_byte_sent = true;
                        }
                    }
                    "migration-relay-physical-truncated-header" => {
                        producer.write_all(&[b'Z', 0, 0])?
                    }
                    "migration-relay-physical-truncated-body" => {
                        producer.write_all(&[b'Z', 0, 0, 0, 11, 1, 2, 3])?
                    }
                    "migration-helper-relay-physical-invalid-length" => {
                        producer.write_all(&[b'Z', 0, 0, 0, 3])?;
                        return Ok(witness);
                    }
                    "migration-helper-relay-physical-invalid-tag" => {
                        producer.write_all(&[b'C', 0, 0, 0, 5, b'X'])?;
                        return Ok(witness);
                    }
                    "migration-helper-relay-physical-body-io-deadline" => {
                        producer.write_all(&[b'Z', 0, 0, 0, 8])?;
                        thread::sleep(Duration::from_millis(2300));
                        // Drop the held endpoint after successful production;
                        // the tested reader has already reached its deadline.
                        return Ok(witness);
                    }
                    "migration-relay-physical-io-deadline" => {
                        thread::sleep(Duration::from_millis(2300))
                    }
                    "migration-relay-physical-total-deadline" => {
                        let end = Instant::now() + Duration::from_secs(11);
                        while Instant::now() < end {
                            producer.write_all(&[b'Z', 0, 0, 0, 4])?;
                            thread::sleep(Duration::from_millis(100));
                        }
                    }
                    "migration-relay-physical-bytes-exact"
                    | "migration-relay-physical-bytes-one-over" => {
                        let mut frame = vec![0; 1 << 20];
                        frame[..5].copy_from_slice(&[b'Z', 0, 15, 255, 255]);
                        for _ in 0..if id.ends_with("one-over") { 9 } else { 8 } {
                            producer.write_all(&frame)?;
                        }
                    }
                    "migration-relay-physical-frames-exact"
                    | "migration-relay-physical-frames-one-over" => {
                        for _ in 0..if id.ends_with("one-over") { 4097 } else { 4096 } {
                            producer.write_all(&[b'Z', 0, 0, 0, 4])?;
                        }
                    }
                    _ => return Err(invalid()),
                }
                producer.shutdown(Shutdown::Write)?;
                Ok(witness)
            })?;
        let started = Instant::now();
        let end = started + Duration::from_secs(10);
        let result = if id.contains("-client-") {
            client(&mut source, &mut target, end)
        } else {
            server(&mut source, &mut target, Mode::Commit, end)
        };
        let elapsed = started.elapsed();
        let source_shutdown = source.shutdown(Shutdown::Both);
        let target_shutdown = target.shutdown(Shutdown::Both);
        // Release the owned endpoints before waiting, even if shutdown failed.
        drop(source);
        drop(target);
        let (forwarded, drainer) = reader.join().map_err(|_| invalid())?;
        let peer = writer.join().map_err(|_| invalid())?;
        println!(
            "postgres physical_relay={id} source_shutdown={source_shutdown:?} target_shutdown={target_shutdown:?} drainer={drainer:?} peer={peer:?} drained_bytes={}",
            forwarded.len()
        );
        drainer?;
        Ok(Physical {
            result,
            forwarded,
            elapsed,
            peer,
        })
    })
}

pub struct Relay {
    pub directory: PathBuf,
    server: Option<JoinHandle<io::Result<Stats>>>,
    client: Option<JoinHandle<io::Result<Stats>>>,
}

pub(super) fn owned_directory(prefix: &str, next: &AtomicU64) -> io::Result<PathBuf> {
    for _ in 0..32 {
        let path = PathBuf::from(format!(
            "/private/tmp/{prefix}-{}-{}",
            std::process::id(),
            next.fetch_add(1, Ordering::Relaxed)
        ));
        match fs::DirBuilder::new().mode(0o700).create(&path) {
            Ok(()) => {
                return Ok(path);
            }
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }
    Err(invalid())
}

impl Relay {
    pub fn start(destination: &Path, mode: Mode) -> io::Result<Self> {
        let mut relay = Self {
            directory: owned_directory("arboresce-relay", &NEXT)?,
            server: None,
            client: None,
        };
        let socket = relay.directory.join(".s.PGSQL.5432");
        let listener = UnixListener::bind(&socket)?;
        fs::set_permissions(&socket, fs::Permissions::from_mode(0o700))?;
        listener.set_nonblocking(true)?;
        let deadline = Instant::now() + Duration::from_millis(10000);
        let (send, receive) = mpsc::sync_channel::<(UnixStream, UnixStream)>(1);
        relay.server = Some(
            thread::Builder::new()
                .name("migration-relay-server".into())
                .stack_size(1 << 20)
                .spawn(move || {
                    let (mut source, mut target) = receive
                        .recv_timeout(remaining(deadline)?)
                        .map_err(|error| {
                            io::Error::new(
                                match error {
                                    mpsc::RecvTimeoutError::Timeout => io::ErrorKind::TimedOut,
                                    mpsc::RecvTimeoutError::Disconnected => {
                                        io::ErrorKind::BrokenPipe
                                    }
                                },
                                "relay pair unavailable",
                            )
                        })?;
                    let result = server(&mut source, &mut target, mode, deadline);
                    close(&source, &target);
                    result
                })?,
        );
        let backend = destination.join(".s.PGSQL.5432");
        relay.client = Some(
            thread::Builder::new()
                .name("migration-relay-client".into())
                .stack_size(1 << 20)
                .spawn(move || {
                    let accept_deadline = Instant::now() + Duration::from_millis(2000);
                    let (mut front, _) = loop {
                        remaining(accept_deadline)?;
                        match listener.accept() {
                            Ok(pair) => break pair,
                            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                                thread::sleep(Duration::from_millis(1))
                            }
                            Err(error) => return Err(error),
                        }
                    };
                    front.set_nonblocking(false)?;
                    let runtime = tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()?;
                    let mut back = runtime
                        .block_on(async {
                            tokio::time::timeout(
                                Duration::from_millis(1000),
                                tokio::net::UnixStream::connect(backend),
                            )
                            .await
                        })
                        .map_err(|_| {
                            io::Error::new(io::ErrorKind::TimedOut, "relay backend unavailable")
                        })??
                        .into_std()?;
                    back.set_nonblocking(false)?;
                    send.send((back.try_clone()?, front.try_clone()?))
                        .map_err(|_| {
                            io::Error::new(io::ErrorKind::BrokenPipe, "relay peer unavailable")
                        })?;
                    let result = client(&mut front, &mut back, deadline);
                    close(&front, &back);
                    result
                })?,
        );
        Ok(relay)
    }

    pub fn finish(mut self) -> io::Result<(io::Result<Stats>, io::Result<Stats>)> {
        let server = self
            .server
            .take()
            .ok_or_else(invalid)?
            .join()
            .map_err(|_| invalid())?;
        let client = self
            .client
            .take()
            .ok_or_else(invalid)?
            .join()
            .map_err(|_| invalid())?;
        Ok((server, client))
    }
}

/// Actual peer/worker failures, with setup errors kept separate from observations.
pub fn lifecycle(recipe: &str) -> io::Result<serde_json::Value> {
    let started = Instant::now();
    let server_target_closed = recipe == "server-target-read-closed";
    if recipe == "target-read-closed" || server_target_closed {
        let (mut source, mut producer) = UnixStream::pair()?;
        let (mut target, consumer) = UnixStream::pair()?;
        consumer.shutdown(Shutdown::Read)?;
        drop(consumer);
        let (bytes, field): (&[u8], _) = if server_target_closed {
            (&[b'Z', 0, 0, 0, 4], "server_error_kind")
        } else {
            (&[1], "client_error_kind")
        };
        producer.write_all(bytes)?;
        let end = started + Duration::from_secs(1);
        let result = if server_target_closed {
            server(&mut source, &mut target, Mode::Commit, end)
        } else {
            client(&mut source, &mut target, end)
        };
        close(&source, &target);
        let mut observed = serde_json::json!({"producer_bytes":bytes.len(),"target_read_shutdown":true,"target_peer_closed":true,"elapsed_ms":started.elapsed().as_millis()});
        observed[field] =
            serde_json::json!(result.err().map(|error| format!("{:?}", error.kind())));
        return Ok(observed);
    }
    struct Backend(PathBuf);
    impl Drop for Backend {
        fn drop(&mut self) {
            let _ = fs::remove_dir(&self.0);
        }
    }
    let backend = Backend(owned_directory("arboresce-relay-backend", &NEXT)?);
    let backend_path = backend.0.clone();
    let relay = Relay::start(&backend.0, Mode::Commit)?;
    let directory = relay.directory.clone();
    let socket = directory.join(".s.PGSQL.5432");
    let mut front = None;
    let observed = match recipe {
        "no-frontend-drop" => {
            drop(relay);
            serde_json::json!({"worker_results_observed":false})
        }
        "no-frontend-finish" | "absent-backend" => {
            if recipe == "absent-backend" {
                front = Some(UnixStream::connect(&socket)?);
            }
            let (server, client) = relay.finish()?;
            serde_json::json!({"worker_results_observed":true,"server_error_kind":server.err().map(|e|format!("{:?}",e.kind())),"client_error_kind":client.err().map(|e|format!("{:?}",e.kind()))})
        }
        _ => return Err(invalid()),
    };
    drop(front);
    drop(backend);
    let mut observed = observed;
    observed["elapsed_ms"] = serde_json::json!(started.elapsed().as_millis());
    observed["relay_directory"] = serde_json::json!(directory);
    observed["relay_directory_absent"] = serde_json::json!(!directory.try_exists()?);
    observed["relay_socket_absent"] = serde_json::json!(!socket.try_exists()?);
    observed["backend_directory_absent"] = serde_json::json!(!backend_path.try_exists()?);
    Ok(observed)
}

impl Drop for Relay {
    fn drop(&mut self) {
        for handle in [&mut self.server, &mut self.client] {
            if let Some(worker) = handle.take() {
                let _ = worker.join();
            }
        }
        let _ = fs::remove_file(self.directory.join(".s.PGSQL.5432"));
        let _ = fs::remove_dir(&self.directory);
    }
}
