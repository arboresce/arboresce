"""Real disposable fixtures and forwarding observations for contract validation tests."""

from __future__ import annotations

import hashlib
import json
import math
import os
import sqlite3
import stat
import subprocess
import sys
import time
from collections.abc import Callable, Generator, Iterator
from contextlib import AbstractContextManager, ExitStack, closing, contextmanager
from contextvars import ContextVar
from dataclasses import dataclass, field
from pathlib import Path
from types import FrameType
from typing import IO, Any, cast

import pytest
from check_jsonschema.formats import FormatOptions
from check_jsonschema.regex_variants import RegexImplementation, RegexVariantName
from coverage import CoverageData
from jsonschema.protocols import Validator
from process_observation.observe_cli import mutate_and_acknowledge, pipe_channels, receive_token

from arboresce_contract_validation.cli import LocalRegistryLoader, SnapshotStore, file_identity

type Case = dict[str, Any]
type FileName = str | bytes | os.PathLike[str] | os.PathLike[bytes]

FIXTURES = Path(__file__).parent / "fixtures"
PROJECT = Path(__file__).resolve().parents[1]
CLI_RELATIVE = Path("contracts/validation/src/arboresce_contract_validation/cli.py")
OBSERVATION_RELATIVE = Path("contracts/validation/tests/process_observation/observe_cli.py")
type AuditEvent = tuple[str, tuple[object, ...]]
_AUDIT_SINK: ContextVar[list[AuditEvent] | None] = ContextVar("contract_test_audit", default=None)


class _AuditHook:
    # CPython otherwise suppresses measurement inside real audit callbacks.
    __cantrace__ = True

    def __call__(self, event: str, arguments: tuple[object, ...]) -> None:
        sink = _AUDIT_SINK.get()
        if sink is not None and (event == "open" or event.startswith("socket.")):
            sink.append((event, arguments))


sys.addaudithook(_AuditHook())


@contextmanager
def observe_audit() -> Generator[list[AuditEvent]]:
    """Observe Python's real open/network audit events within the declared call only."""
    events: list[AuditEvent] = []
    token = _AUDIT_SINK.set(events)
    try:
        yield events
    finally:
        _AUDIT_SINK.reset(token)


def fixture_document(name: str) -> Case:
    return cast(Case, json.loads((FIXTURES / name).read_text(encoding="utf-8")))


def boundary_cases(*areas: str) -> list[Case]:
    rows: list[Case] = fixture_document("boundaries.json")["cases"]
    return [row for row in rows if row["area"] in areas]


def write_file(path: Path, description: Case) -> None:
    """Materialize the declared bytes, including real large files and special files."""
    path.parent.mkdir(parents=True, exist_ok=True)
    if "symlink" in description:
        path.symlink_to(description["symlink"])
    elif description.get("kind") == "fifo-with-no-writer":
        os.mkfifo(path)
    elif "repeat_ascii" in description:
        character = description["repeat_ascii"].encode("ascii")
        remaining: int = description["count"]
        with path.open("xb") as stream:
            while remaining:
                amount = min(remaining, 1024 * 1024)
                stream.write(character * amount)
                remaining -= amount
    else:
        text = (
            json.dumps(description["json"], ensure_ascii=False)
            if "json" in description
            else description["utf8"]
        )
        path.write_text(text, encoding="utf-8")
    if "mode" in description:
        path.chmod(0)


def materialize(root: Path, files: dict[str, Case]) -> None:
    (root / "contracts").mkdir(parents=True)
    for name, description in files.items():
        write_file(root / name, description)


def validator_for(loader: LocalRegistryLoader, instance: Any) -> Validator:
    regex = RegexImplementation(RegexVariantName.default)
    return loader.get_validator(
        loader.entry_loader.entry,
        instance,
        FormatOptions(regex_impl=regex),
        regex,
        False,
    )


def cli_arguments(case: Case) -> list[str]:
    setup: Case = case["setup"]
    if "argv" in setup:
        return list(setup["argv"])
    arguments = [
        "--schemafile",
        setup.get("entry", "contracts/entry.schema.json"),
        "--force-filetype",
        "json",
        "--regex-variant",
        "default",
        "--no-cache",
    ]
    if "json_stdout" in case["expected"]:
        arguments += ["--output-format", "json", "--verbose"]
    elif case["id"] == "output-complete-text-errors":
        arguments += ["--verbose"]
    if case["id"] in {"output-traceback-short", "output-traceback-full"}:
        arguments += ["--traceback-mode", case["id"].rsplit("-", 1)[1]]
    return arguments + list(setup["instances"])


@dataclass(frozen=True)
class ProcessOutcome:
    pid: int
    returncode: int
    stdout: str
    stderr: str
    coverage_files: tuple[Path, ...]
    observation: Case | None = None


def read_evidence(path: Path, limit: int) -> tuple[Case, bytes]:
    """Hash the complete stable regular file; reject unsuitable evidence without truncation."""
    before = path.lstat()
    if not stat.S_ISREG(before.st_mode) or before.st_size > limit:
        raise ValueError(f"invalid evidence file: {path}")
    with path.open("rb") as stream:
        opened = os.fstat(stream.fileno())
        data = stream.read(before.st_size + 1)
        finished = os.fstat(stream.fileno())

    if (
        file_identity(before) != file_identity(opened)
        or file_identity(before) != file_identity(finished)
        or file_identity(before) != file_identity(path.lstat())
        or len(data) != before.st_size
    ):
        raise ValueError(f"changed evidence file: {path}")
    return {"path": str(path), "bytes": len(data), "sha256": hashlib.sha256(data).hexdigest()}, data


def file_record(path: Path, limit: int) -> Case:
    return read_evidence(path, limit)[0]


def write_receipt(path: Path, value: Case) -> None:
    with path.open("x", encoding="utf-8") as stream:
        stream.write(json.dumps(value, indent=2, allow_nan=False) + "\n")


def suite_binding() -> Case:
    """Bind this process to the one external timeout and its original deadline."""
    deadline = float(os.environ["ARBORESCE_TEST_DEADLINE"])
    timeout_pid = int(os.environ["ARBORESCE_TEST_TIMEOUT_PID"])
    now = time.monotonic()
    binding = {
        "pid": os.getpid(),
        "ppid": os.getppid(),
        "pgid": os.getpgrp(),
        "sid": os.getsid(0),
        "original_deadline": deadline,
        "observed_monotonic": now,
        "timeout_pid": timeout_pid,
    }
    if not math.isfinite(deadline) or now >= deadline:
        raise ValueError("missing or expired original suite deadline")
    if timeout_pid <= 1 or binding["ppid"] != timeout_pid or binding["pgid"] != timeout_pid:
        raise ValueError("suite is outside its external timeout group")
    return binding


def require_suite_time(suite: Case) -> None:
    if time.monotonic() >= suite["original_deadline"]:
        raise ValueError("original suite deadline expired")


def validate_process_coverage(
    profiles: tuple[Path, ...], core_log: Path, *, observation: bool = False
) -> None:
    """Check the actual child dataset and selected core before admitting its raw profile."""
    if len(profiles) != 1:
        raise ValueError("missing or duplicate process coverage")
    record = file_record(profiles[0], 16 * 1024 * 1024)
    if record["bytes"] == 0:
        raise ValueError("empty process coverage")
    _, core_bytes = read_evidence(core_log, 1024 * 1024)
    core = core_bytes.decode("utf-8")
    if "core.py: Using core=sysmon" not in core or any(
        word in core for word in ("defaulting", "Defaulting", "Falling back", "not usable")
    ):
        raise ValueError("process coverage did not select sysmon")
    with closing(CoverageData(basename=str(profiles[0]))) as data:
        data.read()
        expected = {str(CLI_RELATIVE), str(CLI_RELATIVE.parent / "__init__.py")}
        if observation:
            expected.add(str(OBSERVATION_RELATIVE))
            if not data.lines(str(OBSERVATION_RELATIVE)) or not data.arcs(
                str(OBSERVATION_RELATIVE)
            ):
                raise ValueError("missing process observation helper coverage")
        if (
            not data.has_arcs()
            or data.measured_files() != expected
            or not data.lines(str(CLI_RELATIVE))
            or not data.arcs(str(CLI_RELATIVE))
        ):
            raise ValueError("incomplete or foreign process coverage")
    if file_record(profiles[0], 16 * 1024 * 1024) != record:
        raise ValueError("process coverage changed during inspection")


def copy_command_sources(owner: Path, mirror: Path, relatives: list[Path]) -> dict[str, str]:
    """Copy complete source bytes and retain their original digests for final verification."""
    sources: dict[str, str] = {}
    for relative in relatives:
        source = owner / relative
        original = file_record(source, 1024 * 1024)
        with source.open("rb") as stream:
            data = stream.read(original["bytes"] + 1)
        if len(data) != original["bytes"] or hashlib.sha256(data).hexdigest() != original["sha256"]:
            raise ValueError("canonical source changed during copy")
        destination = mirror / relative
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_bytes(data)
        if destination.read_bytes() != data:
            raise RuntimeError("source copy changed")
        sources[str(relative)] = hashlib.sha256(data).hexdigest()
    return sources


def verify_command_inputs(
    owner: Path, mirror: Path, sources: dict[str, str], configuration: Case
) -> None:
    """Reject changes to either copy of the executable sources or measurement configuration."""
    for name, digest in sources.items():
        if file_record(mirror / name, 1024 * 1024)["sha256"] != digest:
            raise ValueError("copied source changed during execution")
        if file_record(owner / name, 1024 * 1024)["sha256"] != digest:
            raise ValueError("canonical source changed during execution")
    config = owner / "contracts/validation/tests/coverage.ini"
    if file_record(config, 64 * 1024) != configuration:
        raise ValueError("coverage configuration changed during execution")


def run_cli(
    root: Path, arguments: list[str], evidence: Path, *, observation: Case | None = None
) -> ProcessOutcome:
    """Run the copied real entrypoint with explicit per-process instrumentation."""
    suite = suite_binding()
    evidence.mkdir(parents=True)
    temporary = evidence / "tmp"
    temporary.mkdir()
    relatives = [CLI_RELATIVE.parent / "__init__.py", CLI_RELATIVE]
    if observation is not None:
        relatives.append(OBSERVATION_RELATIVE)
    sources = copy_command_sources(PROJECT.parents[1], root, relatives)
    environment = {
        "PATH": "/usr/bin:/bin",
        "LANG": "C",
        "LC_ALL": "C",
        "TZ": "UTC",
        "TMPDIR": str(temporary),
        "__CF_USER_TEXT_ENCODING": f"0x{os.getuid():X}:0x0:0x0",
        "COVERAGE_DEBUG": "core",
        "COVERAGE_DEBUG_FILE": str(evidence / "coverage-core.log"),
    }
    argv = [
        sys.executable,
        "-I",
        "-B",
        "-m",
        "coverage",
        "run",
        "--rcfile",
        str(PROJECT / "tests/coverage.ini"),
        "--source",
        "contracts/validation/src",
        "--data-file",
        str(evidence / ".coverage"),
        str(CLI_RELATIVE),
        *arguments,
    ]
    receipt: Case = {
        "argv": argv,
        "cwd": str(root),
        "environment": environment,
        "case_node": os.environ.get("PYTEST_CURRENT_TEST"),
        "invocation": str(evidence),
        "suite": suite,
        "source_sha256": sources,
        "coverage_config": file_record(PROJECT / "tests/coverage.ini", 64 * 1024),
    }
    process: subprocess.Popen[bytes] | None = None
    observed: Case | None = None
    try:
        with (
            pipe_channels(observation is not None) as (pipes, close_pipe, closed_pipes),
            (evidence / "stdout").open("xb") as stdout,
            (evidence / "stderr").open("xb") as stderr,
        ):
            receipt["pipes"], receipt["closed_pipe_roles"] = dict(pipes), closed_pipes
            if observation is not None:
                receipt["observation_request"] = dict(observation)
                receipt["mutation_before"] = file_record(root / observation["target"], 65536)
                argv[argv.index("--source") + 1] += "," + str(OBSERVATION_RELATIVE.parent)
                argv[-len(arguments) - 1 :] = [
                    str(OBSERVATION_RELATIVE),
                    "--cli",
                    str(CLI_RELATIVE),
                    "--observation",
                    str(evidence / "observation.json"),
                    "--ready-fd",
                    str(pipes["ready_write"]),
                    "--ack-fd",
                    str(pipes["ack_read"]),
                    "--deadline",
                    str(suite["original_deadline"]),
                    "--kind",
                    observation["kind"],
                    "--target",
                    observation["target"],
                    "--after-close",
                    observation.get("after_close", ""),
                    "--",
                    *arguments,
                ]
            require_suite_time(suite)
            with ExitStack() as before_context_wait:
                for name in tuple(pipes):
                    before_context_wait.callback(close_pipe, name)
                with subprocess.Popen(
                    argv,
                    cwd=root,
                    env=environment,
                    stdin=subprocess.DEVNULL,
                    stdout=stdout,
                    stderr=stderr,
                    pass_fds=tuple(
                        pipes[name] for name in ("ready_write", "ack_read") if name in pipes
                    ),
                ) as process:
                    try:
                        pid = process.pid
                        receipt["pid"] = pid
                        receipt["parent_pid"] = os.getpid()
                        receipt["created_monotonic"] = time.monotonic()
                        write_receipt(evidence / "created.json", receipt)
                        close_pipe("ready_write")
                        close_pipe("ack_read")
                        require_suite_time(suite)
                        receipt["pgid_query_attempted"] = True
                        receipt["pgid"] = os.getpgid(pid)
                        require_suite_time(suite)
                        receipt["sid_query_attempted"] = True
                        receipt["sid"] = os.getsid(pid)
                        if receipt["pgid"] != suite["pgid"] or receipt["sid"] != suite["sid"]:
                            raise ValueError("child left the inherited suite group/session")
                        require_suite_time(suite)
                        if observation is not None:
                            receive_token(pipes["ready_read"], b"R", suite["original_deadline"])
                            receipt["ready_monotonic"] = time.monotonic()
                            receipt["mutation_after"] = mutate_and_acknowledge(
                                root / observation["target"],
                                observation["kind"],
                                pipes["ack_write"],
                            )
                            receipt["ack_monotonic"] = time.monotonic()
                        receipt["before_wait_monotonic"] = time.monotonic()
                        write_receipt(evidence / "before-wait.json", receipt)
                        receipt["wait_attempted"] = True
                        returncode = process.wait()
                        receipt["returncode"] = returncode
                        receipt["reaped_and_retired"] = True
                        receipt["wait_end_monotonic"] = time.monotonic()
                        write_receipt(evidence / "wait.json", receipt)
                    finally:
                        before_context_wait.close()
        profiles = tuple(sorted(evidence.glob(".coverage.*")))
        receipt["stdout"], stdout_bytes = read_evidence(evidence / "stdout", 8 * 1024 * 1024)
        receipt["stderr"], stderr_bytes = read_evidence(evidence / "stderr", 8 * 1024 * 1024)
        receipt["coverage"] = [file_record(path, 16 * 1024 * 1024) for path in profiles]
        receipt["core_log"] = file_record(evidence / "coverage-core.log", 1024 * 1024)
        verify_command_inputs(PROJECT.parents[1], root, sources, receipt["coverage_config"])
        validate_process_coverage(
            profiles, evidence / "coverage-core.log", observation=observation is not None
        )
        if observation is not None:
            receipt["observation"], raw = read_evidence(evidence / "observation.json", 65536)
            observed = json.loads(raw)
            assert observed is not None
            observed["parent_mutation"] = {
                "before": receipt["mutation_before"],
                "after": receipt["mutation_after"],
                "ready_monotonic": receipt["ready_monotonic"],
                "ack_monotonic": receipt["ack_monotonic"],
                "closed_pipe_roles": closed_pipes,
            }
        if [file_record(path, 16 * 1024 * 1024) for path in profiles] != receipt["coverage"]:
            raise ValueError("coverage changed after its recorded identity")
        if file_record(evidence / "coverage-core.log", 1024 * 1024) != receipt["core_log"]:
            raise ValueError("coverage core log changed during inspection")
        stdout_text, stderr_text = stdout_bytes.decode("utf-8"), stderr_bytes.decode("utf-8")
        receipt["evidence_accepted"] = True
    except BaseException as error:
        receipt["error"] = {"type": type(error).__name__, "message": str(error)}
        raise
    finally:
        if process is not None:
            receipt["context_returncode"] = process.returncode
            receipt["context_reaped_and_retired"] = process.returncode is not None
        receipt["final_monotonic"] = time.monotonic()
        original_error = sys.exception()
        try:
            write_receipt(evidence / "process.json", receipt)
        except BaseException as error:
            if original_error is not None:
                raise BaseExceptionGroup(
                    "process and receipt failures", [original_error, error]
                ) from None
            raise
    return ProcessOutcome(pid, returncode, stdout_text, stderr_text, profiles, observed)


def snapshot_files(root: Path, setup: Case) -> list[Path]:
    """Expand the independently authored count and payload fixtures literally."""
    if "files" in setup:
        rows: list[Case] = setup["files"]
    elif "path_template" in setup:
        rows = [
            {"path": setup["path_template"].format(index=index), "utf8": setup["utf8"]}
            for index in range(setup["index_start"], setup["index_end"] + 1)
        ]
    else:
        rows = [setup]
    paths: list[Path] = []
    for row in rows:
        path = root / row["path"]
        write_file(path, row)
        paths.append(path)
    return paths


@dataclass
class IOObservation:
    opened: list[int] = field(default_factory=list[int])
    closed: list[int] = field(default_factory=list[int])
    reads: list[tuple[int, int, int]] = field(default_factory=list[tuple[int, int, int]])
    before_read: Callable[[int, int], None] | None = None
    after_read: Callable[[int, bytes], None] | None = None


@contextmanager
def observe_io(store: SnapshotStore) -> Generator[IOObservation]:
    """Forward real calls after root admission; never replace capabilities or results."""
    if store.root_fd is None:
        raise ValueError("observe an open store")
    observed = IOObservation(opened=[store.root_fd])
    real_open, real_close, real_read = os.open, os.close, os.read

    def opened(file: FileName, flags: int, mode: int = 0o777, *, dir_fd: int | None = None) -> int:
        fd = real_open(file, flags, mode, dir_fd=dir_fd)
        observed.opened.append(fd)
        return fd

    def closed(fd: int) -> None:
        real_close(fd)
        observed.closed.append(fd)

    def read(fd: int, count: int) -> bytes:
        if observed.before_read is not None:
            observed.before_read(fd, count)
        block = real_read(fd, count)
        observed.reads.append((fd, count, len(block)))
        if observed.after_read is not None:
            observed.after_read(fd, block)
        return block

    with pytest.MonkeyPatch.context() as patch:
        patch.setattr(os, "open", opened)
        patch.setattr(os, "close", closed)
        patch.setattr(os, "read", read)
        try:
            yield observed
        finally:
            store.close()


@contextmanager
def mutate_snapshot(path: Path, case: Case, observed: IOObservation) -> Generator[list[str]]:
    """Apply the fixed real-file mutation at its declared stat/read boundary."""
    original = path.read_bytes()
    initial = path.stat()
    mutations: list[str] = []
    kind: str = case["id"]

    def mutate() -> None:
        if mutations:
            return
        mutations.append(kind)
        if kind in {"snapshot-race-growth", "snapshot-race-growth-after-read"}:
            with path.open("ab") as stream:
                stream.write(b"d" if kind == "snapshot-race-growth" else b"y")
        elif kind == "snapshot-race-shrink":
            with path.open("wb"):
                pass
        elif kind == "snapshot-race-same-size-write":
            with path.open("r+b") as stream:
                stream.write(b"y")
            os.utime(path, ns=(initial.st_atime_ns, initial.st_mtime_ns + 1_000_000_000))
        elif kind in {"snapshot-race-replace-final", "snapshot-before-open-replacement"}:
            path.rename(path.with_name("retained-original.json"))
            path.write_bytes(original)
        elif kind in {"snapshot-race-replace-parent", "snapshot-race-parent-symlink-substitution"}:
            old_parent = path.parent.with_name("retained-original")
            path.parent.rename(old_parent)
            if kind == "snapshot-race-replace-parent":
                path.parent.mkdir()
                path.write_bytes(original)
            else:
                path.parent.symlink_to(old_parent, target_is_directory=True)
        elif kind == "snapshot-small-grown-over-cap":
            with path.open("r+b") as stream:
                stream.truncate(67108865)
        else:
            raise ValueError(f"unknown mutation: {kind}")

    real_open = os.open

    def before_open(
        file: FileName, flags: int, mode: int = 0o777, *, dir_fd: int | None = None
    ) -> int:
        if file == path.name:
            mutate()
        return real_open(file, flags, mode, dir_fd=dir_fd)

    def before_read(_fd: int, _count: int) -> None:
        mutate()

    def after_read(_fd: int, block: bytes) -> None:
        if block:
            mutate()

    callbacks = observed.before_read, observed.after_read
    with pytest.MonkeyPatch.context() as patch:
        if kind == "snapshot-before-open-replacement":
            patch.setattr(os, "open", before_open)
        elif kind in {"snapshot-race-growth-after-read", "snapshot-race-same-size-write"}:
            observed.after_read = after_read
        else:
            observed.before_read = before_read
        try:
            yield mutations
        finally:
            observed.before_read, observed.after_read = callbacks


@dataclass
class DiscoveryObservation:
    entered: list[Path] = field(default_factory=list[Path])
    depths: list[int] = field(default_factory=list[int])
    buffers: list[int] = field(default_factory=list[int])
    enumerated: list[int] = field(default_factory=list[int])
    scans: list[int] = field(default_factory=list[int])
    stats: list[FileName | int] = field(default_factory=list[FileName | int])
    sorts: int = 0


@contextmanager
def observe_discovery() -> Generator[DiscoveryObservation]:
    """Forward filesystem calls and observe real visitor locals and C sort calls."""
    if sys.gettrace() is not None or sys.getprofile() is not None:
        raise RuntimeError("discovery observations require the qualified sys.monitoring core")
    observed = DiscoveryObservation()
    filename = SnapshotStore.schema_paths.__code__.co_filename

    def observe_frame(frame: FrameType, event: str, _arg: object) -> None:
        if event == "call":
            observed.entered.append(frame.f_locals["path"])
            observed.depths.append(frame.f_locals["depth"])
        if "names" in frame.f_locals:
            observed.buffers.append(len(frame.f_locals["names"]))
        observed.enumerated.append(frame.f_locals["entry_count"])

    def trace(frame: FrameType, event: str, arg: object) -> Any:
        if frame.f_code.co_filename == filename and frame.f_code.co_name == "visit":
            # Admit the actual target before enabling recursive measurement.
            sys.call_tracing(observe_frame, (frame, event, arg))
            return trace
        return None

    def observe_call(_frame: FrameType, event: str, arg: object) -> None:
        if event == "c_call" and getattr(arg, "__name__", "") == "sort":
            observed.sorts += 1

    def profile(frame: FrameType, event: str, arg: object) -> None:
        if frame.f_code.co_filename == filename and frame.f_code.co_name == "visit":
            sys.call_tracing(observe_call, (frame, event, arg))

    real_scandir, real_stat = os.scandir, os.stat

    def scandir(fd: int) -> AbstractContextManager[Iterator[os.DirEntry[str]]]:
        observed.scans.append(fd)
        return real_scandir(fd)

    def inspected(
        file: FileName | int, *, dir_fd: int | None = None, follow_symlinks: bool = True
    ) -> os.stat_result:
        observed.stats.append(file)
        return real_stat(file, dir_fd=dir_fd, follow_symlinks=follow_symlinks)

    with pytest.MonkeyPatch.context() as patch:
        patch.setattr(os, "scandir", scandir)
        patch.setattr(os, "stat", inspected)
        try:
            sys.settrace(trace)
            sys.setprofile(profile)
            yield observed
        finally:
            try:
                sys.setprofile(None)
            finally:
                sys.settrace(None)


@contextmanager
def fail_reference_parse(
    loader: LocalRegistryLoader, failure: BaseException
) -> Generator[list[tuple[Path, Any]]]:
    """Inject a control only after a real referenced-resource parser call returns."""
    real_parse = loader.parsers.parse_data_with_path
    parsed: list[tuple[Path, Any]] = []

    def injected(data: bytes, path: Path, **kwargs: Any) -> Any:
        result = real_parse(data, path, **kwargs)
        parsed.append((path, result))
        raise failure

    with pytest.MonkeyPatch.context() as patch:
        patch.setattr(loader.parsers, "parse_data_with_path", injected)
        yield parsed


@contextmanager
def before_path_open(
    target: Path, mode: str, ordinal: int, action: Callable[[], None]
) -> Generator[list[Path]]:
    """Coordinate one real-file mutation, forwarding every original open unchanged."""
    real_open = Path.open
    matching: list[Path] = []

    def opened(path: Path, *args: Any, **kwargs: Any) -> IO[Any]:
        requested_mode = args[0] if args else kwargs.get("mode", "r")
        if path == target and requested_mode == mode:
            matching.append(path)
            if len(matching) == ordinal:
                action()
        return cast(IO[Any], real_open(path, *args, **kwargs))

    with pytest.MonkeyPatch.context() as patch:
        patch.setattr(Path, "open", opened)
        yield matching


def replace_fixture(path: Path, data: bytes) -> None:
    """Retain the original fixture inode and write the declared replacement bytes."""
    path.rename(path.with_name("retained-original"))
    path.write_bytes(data)


def collide_receipts(evidence: Path, *, final: bool) -> None:
    """Create real exclusive-open collisions before any child can be launched."""
    (evidence / "stdout").write_bytes(b"retained stdout\n")
    if final:
        (evidence / "process.json").write_bytes(b'{"sentinel":"retained"}\n')


def change_profile_metadata(profile: Path) -> None:
    """Change only a constructed guard fixture, never an actual measured child dataset."""
    with closing(sqlite3.connect(profile)) as connection, connection:
        connection.execute(
            "INSERT INTO meta (key, value) VALUES (?, ?)", ("fixture_marker", "changed")
        )


@pytest.fixture
def command_copies(tmp_path: Path) -> tuple[Path, Path, dict[str, str], Case]:
    owner, mirror = tmp_path / "owner", tmp_path / "mirror"
    common = fixture_document("support-guards.json")["common_fixture"]
    for root in (owner, mirror):
        for row in common["source_files"]:
            path = root / row["path"]
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(row["utf8"])
    config = owner / common["config"]["path"]
    config.parent.mkdir(parents=True, exist_ok=True)
    config.write_text(common["config"]["utf8"])
    sources = {row["path"]: row["sha256"] for row in common["source_files"]}
    return owner, mirror, sources, file_record(config, 65536)


@contextmanager
def opened_streams() -> Generator[list[IO[Any]]]:
    """Retain actual streams from scoped opens without taking their close ownership."""
    real_open = Path.open
    streams: list[IO[Any]] = []

    def opened(path: Path, *args: Any, **kwargs: Any) -> IO[Any]:
        stream = cast(IO[Any], real_open(path, *args, **kwargs))
        streams.append(stream)
        return stream

    with pytest.MonkeyPatch.context() as patch:
        patch.setattr(Path, "open", opened)
        yield streams


@contextmanager
def scoped_observation_owner(kind: str) -> Generator[Callable[..., Any]]:
    """Install a real callback solely for observation ownership admission tests."""
    previous_trace, previous_profile = sys.gettrace(), sys.getprofile()

    def owner(_frame: FrameType, _event: str, _arg: object) -> Any:
        return owner

    install = {"trace": sys.settrace, "profile": sys.setprofile}[kind]
    try:
        install(owner)
        yield owner
    finally:
        try:
            sys.setprofile(previous_profile)
        finally:
            sys.settrace(previous_trace)


def observation_callback(
    path: Path, failure: BaseException | None = None
) -> tuple[Callable[[FrameType, str, object], None], Case]:
    """Record direct unit arguments and retain real reads or the actual raised error."""
    observed: Case = {"calls": [], "reads": [], "errors": []}

    def callback(frame: FrameType, event: str, arg: object) -> None:
        observed["calls"].append((frame, event, arg))
        try:
            if failure is not None:
                raise failure
            observed["reads"].append(path.read_bytes())
        except BaseException as error:
            observed["errors"].append(error)
            raise

    return callback, observed


@contextmanager
def recreate_after_unlink(target: Path, replacement: bytes) -> Generator[list[Path]]:
    """Recreate one fixture only after its real unlink has completed."""
    real_unlink = Path.unlink
    completed: list[Path] = []

    def unlinked(path: Path, missing_ok: bool = False) -> None:
        real_unlink(path, missing_ok=missing_ok)
        if path == target:
            completed.append(path)
            path.write_bytes(replacement)

    with pytest.MonkeyPatch.context() as patch:
        patch.setattr(Path, "unlink", unlinked)
        yield completed
