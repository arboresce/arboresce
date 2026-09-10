"""Observe real CLI calls at one explicit pipe barrier without replacing a method."""

from __future__ import annotations

import argparse
import contextlib
import hashlib
import json
import math
import os
import runpy
import select
import sys
import sysconfig
import time
from collections.abc import Callable, Generator
from pathlib import Path
from types import FrameType
from typing import Any, cast


@contextlib.contextmanager
def pipe_channels(
    enabled: bool,
) -> Generator[tuple[dict[str, int], Callable[[str], None], list[str]]]:
    channels: dict[str, int] = {}
    closed: list[str] = []

    def close(name: str) -> None:
        fd = channels.pop(name, None)
        if fd is not None:
            os.close(fd)
            closed.append(name)

    with contextlib.ExitStack() as ownership:
        for reader, writer in (
            (("ready_read", "ready_write"), ("ack_read", "ack_write")) if enabled else ()
        ):
            read_fd, write_fd = os.pipe()
            channels[reader], channels[writer] = read_fd, write_fd
            ownership.callback(close, reader)
            ownership.callback(close, writer)
        yield channels, close, closed


def receive_token(fd: int, token: bytes, deadline: float) -> None:
    remaining = deadline - time.monotonic()
    if not math.isfinite(deadline) or remaining <= 0:
        raise TimeoutError("barrier original deadline expired")
    readable, _, _ = select.select([fd], [], [], remaining)
    if not readable:
        raise TimeoutError("barrier original deadline expired")
    if os.read(fd, 1) != token:
        raise ValueError("barrier token mismatch")


def mutate_and_acknowledge(path: Path, kind: str, ack_fd: int) -> dict[str, Any]:
    if kind == "grow":
        with path.open("ab") as stream:
            if stream.write(b" ") != 1:
                raise OSError("incomplete growth mutation")
        with path.open("rb") as stream:
            data = stream.read(65537)
        if len(data) > 65536:
            raise ValueError("mutation evidence exceeds limit")
        observed = {"bytes": len(data), "sha256": hashlib.sha256(data).hexdigest()}
    elif kind == "remove":
        path.unlink()
        try:
            path.lstat()
        except FileNotFoundError:
            observed = {"absent": True}
        else:
            raise ValueError("removed mutation target remains present")
    else:
        raise ValueError("unknown process observation mutation")
    if os.write(ack_fd, b"A") != 1:
        raise OSError("incomplete barrier acknowledgment")
    return observed


def save_observation(path: Path, data: dict[str, Any], original: BaseException | None) -> None:
    try:
        encoded = (json.dumps(data, allow_nan=False) + "\n").encode()
        if len(encoded) > 65536:
            raise ValueError("observation exceeds 64 KiB")
        with path.open("xb") as stream:
            if stream.write(encoded) != len(encoded):
                raise OSError("incomplete observation write")
    except BaseException as error:
        if original is not None:
            raise BaseExceptionGroup("target and observation failures", [original, error]) from None
        raise


def source_record(filename: str) -> dict[str, Any]:
    with Path(filename).open("rb") as stream:
        data = stream.read(1024 * 1024 + 1)
    if len(data) > 1024 * 1024:
        raise ValueError("observation source exceeds limit")
    return {"path": filename, "bytes": len(data), "sha256": hashlib.sha256(data).hexdigest()}


def emit_event(state: dict[str, Any], event: str, **fields: Any) -> None:
    if len(state["events"]) >= 1024:
        raise ValueError("observation event limit")
    state["events"].append({"event": event, "monotonic": time.monotonic(), **fields})


def record_observation_failure(state: dict[str, Any], error: BaseException) -> None:
    if len(state["errors"]) < 16:
        state["errors"].append({"type": type(error).__name__, "message": str(error)})
    else:
        state["error_overflow"] = True


def close_ipc(state: dict[str, Any], fds: tuple[int, int]) -> None:
    closed_ipc: list[int] = []
    state["ipc_closed"] = closed_ipc
    for fd in fds:
        try:
            os.close(fd)
            closed_ipc.append(fd)
        except OSError as error:
            record_observation_failure(state, error)


def record_descriptor_close(
    state: dict[str, Any], live: dict[int, int], fd: int | None, event: str
) -> None:
    if fd is not None:
        if event == "c_exception":
            raise OSError("owned descriptor close failed")
        emit_event(state, "descriptor-close", fd=fd, occurrence=live.pop(fd))


def dispatch_observation(
    frame: FrameType,
    event: str,
    arg: Any,
    callback: Callable[[FrameType, str, Any], Any],
    admitted_names: set[str],
    state: dict[str, Any],
    continuation: Any,
) -> Any:
    if frame.f_code.co_filename not in admitted_names:
        return None
    try:
        sys.call_tracing(callback, (frame, event, arg))
    except BaseException as error:
        record_observation_failure(state, error)
        if not isinstance(error, Exception):
            raise
    return continuation


def run_observed(args: argparse.Namespace, command: list[str]) -> None:
    if sys.gettrace() is not None or sys.getprofile() is not None:
        raise RuntimeError("process observations require the sys.monitoring core")
    cli = Path(args.cli).absolute()
    target = Path(args.target).absolute()
    first = Path(args.after_close).absolute() if args.after_close else None
    package = Path(sysconfig.get_path("purelib")) / "check_jsonschema"
    names = (
        "instance_loader.py",
        "parsers/__init__.py",
        "parsers/json_.py",
        "schema_loader/main.py",
        "schema_loader/readers.py",
        "result.py",
        "reporter.py",
    )
    paths = {name: str(package / name) for name in names}
    cli_names = {str(args.cli), str(cli)}
    context_name = str(contextlib.__file__)
    json_name = json.loads.__code__.co_filename
    admitted_names = set(paths.values()) | cli_names | {context_name, json_name}
    state: dict[str, Any] = {
        "events": [],
        "errors": [],
        "barrier_entered": False,
        "argv": [args.cli, *command],
        "original_argv": sys.orig_argv,
        "target": str(target),
        "after_close": str(first) if first else None,
        "original_deadline": args.deadline,
        "inputs": [],
    }
    for filename in sorted(set(paths.values()) | {str(cli), context_name, json_name}):
        state["inputs"].append(source_record(filename))
    stores: list[Any] = []
    streams: list[Any] = []
    loaders: list[Any] = []
    results: list[Any] = []
    live: dict[int, int] = {}
    failed: set[int] = set()
    exceptions: dict[int, BaseException] = {}

    def emit(event: str, **fields: Any) -> None:
        emit_event(state, event, **fields)

    def observation_failure(error: BaseException) -> None:
        record_observation_failure(state, error)

    def barrier(**fields: Any) -> None:
        state["barrier_entered"] = True
        emit("barrier-ready", **fields)
        if os.write(args.ready_fd, b"R") != 1:
            raise OSError("incomplete barrier ready")
        receive_token(args.ack_fd, b"A", args.deadline)
        emit("barrier-ack")

    def trace(frame: FrameType, event: str, arg: Any) -> Any:
        filename, qual = frame.f_code.co_filename, frame.f_code.co_qualname
        if filename not in admitted_names:
            return None
        local = frame.f_locals
        try:
            if event == "exception":
                failed.add(id(frame))
                error = cast(BaseException, arg[1])
                if filename in cli_names and qual in {"SnapshotStore.read", "SnapshotStream.read"}:
                    if id(error) not in exceptions:
                        exceptions[id(error)] = error
                        path = local.get("path", getattr(local.get("self"), "name", ""))
                        emit(
                            "read-error",
                            path=os.path.abspath(path),
                            type=type(error).__name__,
                            reason=getattr(error, "reason", None),
                            filename=getattr(error, "filename", None),
                            notes=getattr(error, "__notes__", []),
                            cause=type(error.__cause__).__name__ if error.__cause__ else None,
                        )
            if event == "line" and filename in cli_names:
                sites = {
                    ("SnapshotStore.__init__", 108): "fd",
                    ("SnapshotStore.opened", 157): "fd",
                    ("SnapshotStore.schema_paths.<locals>.visit", 253): "child_fd",
                }
                key = sites.get((qual, frame.f_lineno))
                if key:
                    fd = local[key]
                    if fd in live:
                        raise ValueError("overlapping owned descriptor occurrence")
                    live[fd] = len(state["events"])
                    emit("descriptor-open", fd=fd, occurrence=live[fd])
                if qual == "LocalRegistryLoader.prepare_registry" and frame.f_lineno == 363:
                    emit("fallback")
            if event == "call":
                if filename in cli_names:
                    owners = {
                        "SnapshotStore.__init__": stores,
                        "SnapshotStream.__init__": streams,
                        "LocalRegistryLoader.__init__": loaders,
                    }
                    if qual in owners:
                        owners[qual].append(local["self"])
                if filename == paths["result.py"] and qual == "CheckResult.__init__":
                    results.append(local["self"])
                if filename == paths["schema_loader/readers.py"]:
                    if type(local.get("self")).__name__ == "LocalSchemaReader":
                        emit("old-reader", method=qual)
                if filename == paths["reporter.py"] and qual == "Reporter.report_result":
                    emit("report-result")
                if filename == json_name and qual == "loads":
                    caller = frame.f_back
                    if (
                        caller is not None
                        and caller.f_code.co_filename == paths["parsers/json_.py"]
                    ):
                        while caller is not None:
                            if caller.f_code.co_qualname == "ParserSet.parse_data_with_path":
                                payload = local["s"]
                                emit(
                                    "decode",
                                    path=os.path.abspath(caller.f_locals["path"]),
                                    bytes=len(payload),
                                    sha256=hashlib.sha256(payload).hexdigest(),
                                )
                                break
                            caller = caller.f_back
            if event == "return":
                normal = id(frame) not in failed
                failed.discard(id(frame))
                if normal and filename in cli_names:
                    if qual == "SnapshotStore.preflight":
                        emit("preflight", path=os.path.abspath(local["filename"]))
                    if qual == "SnapshotStore.read" and arg is not None:
                        emit("snapshot", path=str(arg.path), bytes=len(arg.data))
                    if qual == "EntryLoader.get_schema" and arg is not None:
                        emit("entry-schema", path=str(local["self"].entry))
                if (
                    normal
                    and filename == paths["schema_loader/main.py"]
                    and qual == "_check_schema"
                ):
                    emit("meta-check")
                if (
                    normal
                    and filename == paths["result.py"]
                    and qual == "CheckResult.record_validation_error"
                ):
                    path = str(local["path"])
                    emit(
                        "validation-error",
                        path=os.path.abspath(path),
                        count=len(local["self"].validation_errors[path]),
                    )
        except Exception as error:
            observation_failure(error)
        return trace

    def profile(frame: FrameType, event: str, arg: object) -> None:
        filename, qual, local = frame.f_code.co_filename, frame.f_code.co_qualname, frame.f_locals
        try:
            if (
                filename in cli_names
                and qual == "SnapshotStore.read"
                and event == "c_call"
                and arg is os.read
            ):
                if args.kind == "grow" and local["path"] == target and not state["barrier_entered"]:
                    before = local["before"]
                    barrier(
                        path=str(target),
                        fd=local["fd"],
                        requested=local["requested"],
                        before=[
                            before.st_dev,
                            before.st_ino,
                            before.st_mode,
                            before.st_size,
                            before.st_mtime_ns,
                            before.st_ctime_ns,
                        ],
                    )
            if (
                filename == paths["instance_loader.py"]
                and qual == "InstanceLoader.iter_files"
                and event == "c_return"
            ):
                stream = local.get("file")
                if (
                    stream is not None
                    and getattr(arg, "__name__", None) == "close"
                    and getattr(arg, "__self__", None) is stream
                ):
                    emit("stream-close", path=os.path.abspath(stream.name), closed=stream.closed)
                    if (
                        args.kind == "remove"
                        and Path(stream.name).absolute() == first
                        and not state["barrier_entered"]
                    ):
                        barrier(path=str(first), closed=stream.closed)
            if arg is os.close and event in {"c_return", "c_exception"}:
                fd = None
                if filename in cli_names and qual in {
                    "SnapshotStore.close",
                    "SnapshotStore.__init__",
                }:
                    fd = local["fd"]
                elif filename in cli_names and qual == "SnapshotStore.schema_paths.<locals>.visit":
                    fd = local["child_fd"]
                elif (
                    filename == context_name
                    and qual == "_BaseExitStack._create_cb_wrapper.<locals>._exit_wrapper"
                ):
                    if local["callback"] is os.close:
                        fd = local["args"][0]
                record_descriptor_close(state, live, fd, event)
        except Exception as error:
            observation_failure(error)

    def trace_dispatch(frame: FrameType, event: str, arg: Any) -> Any:
        return dispatch_observation(frame, event, arg, trace, admitted_names, state, trace_dispatch)

    def profile_dispatch(frame: FrameType, event: str, arg: object) -> None:
        dispatch_observation(frame, event, arg, profile, admitted_names, state, None)

    previous_argv = sys.argv
    try:
        sys.argv = [args.cli, *command]
        sys.settrace(trace_dispatch)
        sys.setprofile(profile_dispatch)
        runpy.run_path(args.cli, run_name="__main__")
    finally:
        original = sys.exception()
        sys.setprofile(None)
        sys.settrace(None)
        sys.argv = previous_argv
        state["live_descriptors"] = list(live)
        state["root_fds"] = [store.root_fd for store in stores]
        state["streams_closed"] = [stream.closed for stream in streams]
        state["snapshots"] = sorted({str(path) for store in stores for path in store.snapshots})
        state["preload_errors"] = [
            None if loader.preload_error is None else str(loader.preload_error)
            for loader in loaders
        ]
        state["validation_errors"] = [
            {name: len(errors) for name, errors in result.validation_errors.items()}
            for result in results
        ]
        close_ipc(state, (args.ready_fd, args.ack_fd))
        save_observation(Path(args.observation), state, original)


def main() -> None:
    parser = argparse.ArgumentParser()
    for name in ("cli", "observation", "target", "after-close"):
        parser.add_argument("--" + name, required=True)
    for name in ("ready-fd", "ack-fd"):
        parser.add_argument("--" + name, type=int, required=True)
    parser.add_argument("--deadline", type=float, required=True)
    parser.add_argument("--kind", choices=("grow", "remove"), required=True)
    parser.add_argument("command", nargs=argparse.REMAINDER)
    args = parser.parse_args()
    if not args.command or args.command[0] != "--":
        parser.error("complete CLI arguments must follow --")
    run_observed(args, args.command[1:])


if __name__ == "__main__":
    main()
