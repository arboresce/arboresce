"""Verify that observations retain real operations and restore their scope."""

from __future__ import annotations

import argparse
import errno
import gc
import json
import math
import os
import socket
import sqlite3
import sys
import time
from collections import Counter
from contextlib import closing
from functools import partial
from pathlib import Path

import pytest
from coverage import Coverage, CoverageData
from coverage.exceptions import DataError
from process_observation.observe_cli import (
    close_ipc,
    dispatch_observation,
    emit_event,
    mutate_and_acknowledge,
    pipe_channels,
    receive_token,
    record_descriptor_close,
    record_observation_failure,
    run_observed,
    save_observation,
    source_record,
)
from process_observation.observe_cli import (
    main as observation_main,
)
from support import (
    CLI_RELATIVE,
    PROJECT,
    before_path_open,
    change_profile_metadata,
    collide_receipts,
    copy_command_sources,
    file_record,
    fixture_document,
    mutate_snapshot,
    observation_callback,
    observe_audit,
    observe_io,
    opened_streams,
    read_evidence,
    recreate_after_unlink,
    replace_fixture,
    require_suite_time,
    run_cli,
    scoped_observation_owner,
    suite_binding,
    validate_process_coverage,
    verify_command_inputs,
    write_receipt,
)
from support import command_copies as command_copies

from arboresce_contract_validation.cli import InputAdmissionError, SnapshotStore


def test_audit_scope_restores_nested_observations(tmp_path):
    path = tmp_path / "value.json"
    path.write_bytes(b"7")
    with observe_audit() as outer:
        before_inner = path.read_bytes()
        with observe_audit() as inner:
            inside_inner = path.read_bytes()
            first, second = socket.socketpair()
            with first, second:
                sent = first.send(b"x")
                received = second.recv(1)
        after_inner = path.read_bytes()
    assert before_inner == inside_inner == after_inner == b"7"
    assert sent == 1 and received == b"x"
    assert [event for event, _ in inner].count("open") == 1, inner
    assert [event for event, _ in inner].count("socket.__new__") == 2, inner
    assert [event for event, _ in outer] == ["open", "open"], outer
    before = list(outer)
    assert path.read_bytes() == b"7"
    assert outer == before


def test_closed_store_cannot_be_observed(tmp_path):
    store = SnapshotStore(tmp_path)
    store.close()
    with pytest.raises(ValueError, match="observe an open store"), observe_io(store):
        pytest.fail("a closed store was accepted")


@pytest.mark.parametrize("kind", ["missing", "duplicate", "empty", "corrupt", "symlink"])
def test_process_coverage_rejects_invalid_files(kind, tmp_path):
    profile = tmp_path / "invalid-profile.db"
    core = tmp_path / "core.log"
    core.write_text("core.py: Using core=sysmon\n")
    profiles = (profile,)
    error = ValueError
    if kind == "missing":
        profiles = ()
    elif kind == "duplicate":
        profiles = (profile, profile)
    elif kind == "symlink":
        target = tmp_path / "target"
        target.write_bytes(b"not a profile")
        profile.symlink_to(target)
    else:
        profile.write_bytes(b"" if kind == "empty" else b"not a SQLite database")
        if kind == "corrupt":
            error = DataError
    with pytest.raises(error):
        validate_process_coverage(profiles, core)
    if kind == "corrupt":
        # Expose a leaked connection in this test under pytest's unraisable-error policy.
        gc.collect()


@pytest.mark.parametrize(
    "kind", ["lines-only", "foreign-source", "empty-arcs", "wrong-core", "valid"]
)
def test_process_coverage_rejects_incompatible_data(kind, tmp_path):
    # Deliberately constructed rejection fixtures never enter actual raw coverage collection.
    profile = tmp_path / "incompatible-profile.db"
    with closing(CoverageData(basename=str(profile))) as data:
        if kind == "lines-only":
            data.add_lines({str(CLI_RELATIVE): {1}})
        else:
            data.add_arcs({str(CLI_RELATIVE): set() if kind == "empty-arcs" else {(1, 2), (2, -1)}})
            data.touch_file(str(CLI_RELATIVE.parent / "__init__.py"))
            if kind == "foreign-source":
                data.add_arcs({"foreign.py": {(1, 2)}})
        data.write()
    core = tmp_path / "core.log"
    core.write_text(
        "core.py: Using core=ctrace\n" if kind == "wrong-core" else "core.py: Using core=sysmon\n"
    )
    if kind == "valid":
        assert validate_process_coverage((profile,), core) is None
    else:
        with pytest.raises(ValueError):
            validate_process_coverage((profile,), core)


def test_evidence_record_bounds_and_exclusive_receipt(tmp_path):
    path = tmp_path / "value"
    path.write_bytes(b"abc")
    record = file_record(path, 3)
    assert record == {
        "path": str(path),
        "bytes": 3,
        "sha256": "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
    }
    with pytest.raises(ValueError, match="invalid evidence file"):
        file_record(path, 2)
    receipt = tmp_path / "receipt.json"
    write_receipt(receipt, record)
    original = receipt.read_bytes()
    with pytest.raises(FileExistsError):
        write_receipt(receipt, {"replacement": True})
    assert receipt.read_bytes() == original


@pytest.mark.parametrize("deadline", ["0", "-1", "nan", "inf", "-inf"])
def test_suite_rejects_expired_or_nonfinite_deadlines(deadline, monkeypatch):
    monkeypatch.setenv("ARBORESCE_TEST_DEADLINE", deadline)
    monkeypatch.setenv("ARBORESCE_TEST_TIMEOUT_PID", "1")
    with pytest.raises(ValueError, match="original suite deadline"):
        suite_binding()


def test_suite_rejects_missing_deadline(monkeypatch):
    monkeypatch.delenv("ARBORESCE_TEST_DEADLINE", raising=False)
    with pytest.raises(KeyError, match="ARBORESCE_TEST_DEADLINE"):
        suite_binding()


def test_suite_rejects_unbound_timeout(monkeypatch):
    monkeypatch.setenv("ARBORESCE_TEST_TIMEOUT_PID", "1")
    with pytest.raises(ValueError, match="outside its external timeout group"):
        suite_binding()


REPORT_FIXTURE = fixture_document("coverage-report.json")


@pytest.mark.parametrize("case", REPORT_FIXTURE["cases"], ids=lambda row: row["id"])
def test_coverage_report_keeps_unexecuted_branches(case, tmp_path):
    # Constructed report data never enters actual process coverage aggregation.
    source = tmp_path / REPORT_FIXTURE["source_filename"]
    source.write_text(REPORT_FIXTURE["source_utf8"])
    profile = tmp_path / REPORT_FIXTURE["profile_filename"]
    with closing(CoverageData(basename=str(profile))) as data:
        data.add_arcs({str(source): {tuple(arc) for arc in case["constructed_arcs"]}})
        data.write()
    reporter = Coverage(
        config_file=str(PROJECT / "tests/coverage.ini"),
        data_file=str(profile),
        data_suffix=False,
        source=[str(tmp_path)],
    )
    for option, expected in REPORT_FIXTURE["config_rule"].items():
        assert reporter.get_option("report:" + option) == expected
    report = tmp_path / "report.json"
    with closing(reporter.get_data()):
        reporter.load()
        reporter.json_report(morfs=[str(source)], outfile=str(report))
    document = json.loads(report.read_text())
    assert document["meta"]["branch_coverage"] is True
    assert set(document["files"]) == {str(source)}
    actual = document["files"][str(source)]
    for name, expected in case["expected"].items():
        assert (actual[name] if isinstance(expected, list) else actual["summary"][name]) == expected


GUARDS = fixture_document("process-observation-guards.json")["cases"]
assert [row["id"] for row in GUARDS] == [
    "barrier-ack-eof",
    "barrier-ack-wrong-token",
    "barrier-original-deadline-expired",
    "barrier-parent-mutation-failure",
    "observation-write-failure-preserves-target-error",
]


@pytest.mark.parametrize("case", GUARDS[:3], ids=lambda row: row["id"])
def test_barrier_rejects_bad_ack(case):
    setup, expected = case["setup"], case["expected"]
    with pipe_channels(True) as (pipes, close, closed):
        if "writer_bytes_utf8" in setup:
            assert os.write(pipes["ack_write"], setup["writer_bytes_utf8"].encode()) == 1
        else:
            close("ack_write")
        deadline = (
            setup["deadline"]
            if isinstance(setup["deadline"], float)
            else suite_binding()["original_deadline"]
        )
        failure = {"ValueError": ValueError, "TimeoutError": TimeoutError}[expected["exception"]]
        with pytest.raises(failure, match=expected["message"]) as observed_error:
            receive_token(pipes["ack_read"], b"A", deadline)
        assert str(observed_error.value) == expected["message"]
        close("ack_write")
        assert (
            os.read(pipes["ack_read"], 1)
            == expected.get("queued_byte_after_failure_utf8", "").encode()
        )
    assert len(closed) == len(set(closed)) == 4 and not pipes


def test_failed_parent_mutation_does_not_acknowledge(tmp_path):
    case = GUARDS[3]
    path = tmp_path / case["setup"]["target"]
    with pipe_channels(True) as (pipes, close, closed):
        with pytest.raises(FileNotFoundError) as failure:
            mutate_and_acknowledge(path, "remove", pipes["ack_write"])
        assert type(failure.value).__name__ == case["expected"]["exception"]
        close("ack_write")
        assert os.read(pipes["ack_read"], 1) == b"" and not path.exists()
    assert len(closed) == 4 and not pipes


def test_observation_write_preserves_actual_target_error(tmp_path):
    case = GUARDS[4]
    output = tmp_path / "observation.json"
    output.write_text(case["setup"]["output_utf8"])
    try:
        (tmp_path / case["setup"]["missing_target"]).read_bytes()
    except FileNotFoundError as original:
        with pytest.raises(ExceptionGroup) as failure:
            save_observation(output, {"observed": True}, original)
        assert failure.value.exceptions[0] is original
        assert [type(error).__name__ for error in failure.value.exceptions] == case["expected"][
            "member_types_in_order"
        ]
    else:
        pytest.fail("missing real target unexpectedly opened")
    assert output.read_text() == case["expected"]["output_utf8_unchanged"]


GUARD_FIXTURE = fixture_document("support-guards.json")
SUPPORT_GUARDS = {row["id"]: row for row in GUARD_FIXTURE["cases"]}


def test_source_copy_and_verification_keep_original_bytes(command_copies):
    owner, mirror, sources, config = command_copies
    relatives = [Path(name) for name in sources]
    assert copy_command_sources(owner, mirror, relatives) == sources
    for row in GUARD_FIXTURE["common_fixture"]["source_files"]:
        assert (owner / row["path"]).read_text() == row["utf8"]
        assert (mirror / row["path"]).read_text() == row["utf8"]
    assert verify_command_inputs(owner, mirror, sources, config) is None


def test_evidence_replacement_is_rejected(tmp_path):
    case = SUPPORT_GUARDS["support-evidence-replaced-before-read"]
    path = tmp_path / "evidence"
    path.write_text(case["setup"]["target_utf8"])
    original_open = Path.open
    with before_path_open(path, "rb", 1, partial(replace_fixture, path, b"None\n")) as opened:
        with pytest.raises(ValueError) as error:
            read_evidence(path, 5)
    assert str(error.value) == case["expected"]["message"].replace("TARGET", str(path))
    assert opened == [path] and Path.open is original_open
    assert path.read_text() == case["expected"]["target_utf8"]
    assert (
        path.with_name("retained-original").read_text()
        == case["expected"]["retained_original_utf8"]
    )


def test_original_suite_deadline_is_not_extended():
    case = SUPPORT_GUARDS["support-expired-original-deadline"]
    with pytest.raises(ValueError) as error:
        require_suite_time({"original_deadline": 0})
    assert str(error.value) == case["expected"]["message"]


@pytest.mark.parametrize("kind", ["canonical", "mirror"])
def test_source_copy_detects_real_replacement(kind, command_copies):
    owner, mirror, sources, _ = command_copies
    # The copy owns destination creation; remove only these test-owned initial files.
    for name in sources:
        (mirror / name).unlink()
    if kind == "canonical":
        case = SUPPORT_GUARDS["support-canonical-source-replaced-during-copy"]
        target, ordinal, failure = owner / CLI_RELATIVE, 2, ValueError
    else:
        case = SUPPORT_GUARDS["support-mirror-source-changed-before-copy-comparison"]
        target, ordinal, failure = mirror / CLI_RELATIVE, 1, RuntimeError
    original_open = Path.open
    with before_path_open(
        target, "rb", ordinal, partial(replace_fixture, target, b"None\n")
    ) as opened:
        with pytest.raises(failure) as error:
            copy_command_sources(owner, mirror, [Path(name) for name in sources])
    assert str(error.value) == case["expected"]["message"]
    assert len(opened) == case["expected"]["forwarded_matching_opens"]
    assert Path.open is original_open
    assert target.read_bytes() == b"None\n"
    assert (
        target.with_name("retained-original").read_text()
        == case["expected"]["retained_original_utf8"]
    )
    if kind == "canonical":
        assert not (mirror / CLI_RELATIVE).exists()
    else:
        assert (owner / CLI_RELATIVE).read_text() == case["expected"]["owner_main_utf8"]


@pytest.mark.parametrize("kind", ["mirror", "canonical", "config"])
def test_command_input_verification_rejects_changes(kind, command_copies):
    owner, mirror, sources, config = command_copies
    common = GUARD_FIXTURE["common_fixture"]
    if kind == "config":
        case = SUPPORT_GUARDS["support-recorded-config-changed"]
        target = owner / common["config"]["path"]
    else:
        case = SUPPORT_GUARDS[f"support-recorded-{kind}-source-changed"]
        target = (mirror if kind == "mirror" else owner) / CLI_RELATIVE
    target.write_text(case["mutation"]["utf8"])
    with pytest.raises(ValueError) as error:
        verify_command_inputs(owner, mirror, sources, config)
    assert str(error.value) == case["expected"]["message"]
    for root in (owner, mirror):
        for row in common["source_files"]:
            path = root / row["path"]
            if path != target:
                assert path.read_text() == row["utf8"]
    if kind != "config":
        assert (owner / common["config"]["path"]).read_text() == common["config"]["utf8"]


@pytest.mark.parametrize("final", [False, True])
def test_prelaunch_collision_preserves_files_and_errors(final, tmp_path):
    key = (
        "support-primary-and-receipt-collisions"
        if final
        else "support-stdout-collision-before-child"
    )
    expected = SUPPORT_GUARDS[key]["expected"]
    root, evidence = tmp_path / "workspace", tmp_path / "process"
    original_open = Path.open
    with before_path_open(
        evidence / "stdout", "xb", 1, partial(collide_receipts, evidence, final=final)
    ):
        with pytest.raises(ExceptionGroup if final else FileExistsError) as failure:
            run_cli(root, ["--help"], evidence)
    assert Path.open is original_open
    assert (evidence / "stdout").read_text() == expected["stdout_utf8"]
    assert not list(evidence.glob(".coverage.*"))
    for path in expected["absent_files"]:
        assert not (evidence / path.removeprefix("EVIDENCE/")).exists()
    if final:
        assert failure.value.message == expected["message"]
        assert [type(error).__name__ for error in failure.value.exceptions] == [
            row["type"] for row in expected["ordered_exceptions"]
        ]
        assert [error.filename for error in failure.value.exceptions] == [
            str(evidence / row["filename"].removeprefix("EVIDENCE/"))
            for row in expected["ordered_exceptions"]
        ]
        assert (evidence / "process.json").read_text() == expected["process_receipt_utf8"]
    else:
        assert failure.value.filename == str(evidence / "stdout")
        receipt = json.loads((evidence / "process.json").read_text())
        assert receipt["error"]["type"] == expected["process_receipt"]["error_type"]
        assert str(evidence / "stdout") in receipt["error"]["message"]
        assert not set(receipt).intersection(expected["process_receipt"]["absent_fields"])


def test_changed_constructed_profile_is_rejected(tmp_path):
    case = SUPPORT_GUARDS["support-valid-profile-metadata-changed-during-inspection"]
    profile = tmp_path / case["setup"]["profile_name"]
    arcs = {
        name: {tuple(arc) for arc in rows} for name, rows in case["setup"]["source_arcs"].items()
    }
    # Constructed guard data is never a raw process qualification profile.
    with closing(CoverageData(basename=str(profile))) as data:
        data.add_arcs(arcs)
        data.touch_file(str(CLI_RELATIVE.parent / "__init__.py"))
        data.write()
    before = profile.read_bytes()
    core = tmp_path / "core.log"
    core.write_text(case["setup"]["core_log_utf8"])
    original_open = Path.open
    with before_path_open(core, "rb", 1, partial(change_profile_metadata, profile)):
        with pytest.raises(ValueError) as error:
            validate_process_coverage((profile,), core)
    assert str(error.value) == case["expected"]["message"]
    assert Path.open is original_open and profile.read_bytes() != before
    with closing(sqlite3.connect(profile)) as connection:
        assert connection.execute("PRAGMA integrity_check").fetchall() == [("ok",)]
        assert connection.execute(
            "SELECT value FROM meta WHERE key = 'fixture_marker'"
        ).fetchall() == [("changed",)]
    with closing(CoverageData(basename=str(profile))) as data:
        data.read()
        assert data.measured_files() == set(arcs)
        assert {name: set(data.arcs(name) or []) for name in data.measured_files()} == arcs
    assert core.read_text() == case["setup"]["core_log_utf8"]


def test_unknown_mutation_preserves_real_file_and_callbacks(tmp_path):
    case = SUPPORT_GUARDS["support-unknown-mutation-control-rejected"]
    path = tmp_path / case["setup"]["file"]
    path.parent.mkdir()
    path.write_text(case["setup"]["utf8"])
    originals = os.open, os.read, os.close
    store = SnapshotStore(path.parent)
    with observe_io(store) as observed:
        callbacks = observed.before_read, observed.after_read
        with mutate_snapshot(path, {"id": case["setup"]["case_id"]}, observed) as mutations:
            with pytest.raises(ValueError) as error:
                store.read(path)
        assert (observed.before_read, observed.after_read) == callbacks
        assert str(error.value) == case["expected"]["message"]
        assert mutations == [case["setup"]["case_id"]] and not observed.reads
    assert Counter(observed.opened) == Counter(observed.closed)
    assert (os.open, os.read, os.close) == originals
    assert path.read_text() == case["expected"]["target_utf8"]


def test_non_target_open_is_forwarded_before_replacement(tmp_path):
    case = SUPPORT_GUARDS["support-before-open-unrelated-path-passes-through"]
    target, other = (tmp_path / case["setup"][name] for name in ("target", "other"))
    target.parent.mkdir()
    target.write_text(case["setup"]["target_utf8"])
    other.write_text(case["setup"]["other_utf8"])
    originals = os.open, os.read, os.close
    store = SnapshotStore(target.parent)
    with observe_io(store) as observed:
        callbacks = observed.before_read, observed.after_read
        with mutate_snapshot(target, {"id": case["setup"]["case_id"]}, observed) as mutations:
            assert store.read(other).data.decode() == case["expected"]["other_read_utf8"]
            assert target.read_text() == case["setup"]["target_utf8"] and not mutations
            with pytest.raises(InputAdmissionError) as error:
                store.read(target)
            assert mutations == [case["setup"]["case_id"]]
        assert (observed.before_read, observed.after_read) == callbacks
    for text in case["expected"]["target_reason_contains"]:
        assert text in str(error.value)
    assert Counter(observed.opened) == Counter(observed.closed)
    assert (os.open, os.read, os.close) == originals
    assert (
        target.with_name("retained-original.json").read_text()
        == case["expected"]["retained_original_utf8"]
    )
    assert other.read_text() == case["expected"]["other_utf8"]


OBSERVATION_BOUNDARIES = fixture_document("process-observation-guards.json")["boundary_cases"]
OBSERVATION_CASES = {row["id"]: row for row in OBSERVATION_BOUNDARIES}


@pytest.mark.parametrize("case", OBSERVATION_BOUNDARIES[:2], ids=lambda row: row["id"])
def test_observation_source_record_bounds(case, tmp_path):
    path = tmp_path / "source.py"
    payload = case["setup"]["file"]
    path.write_bytes(payload["byte"].encode() * payload["repeat"])
    original_open = Path.open
    with opened_streams() as streams:
        if "exception" in case["expected"]:
            with pytest.raises(ValueError) as failure:
                source_record(str(path))
            assert str(failure.value) == case["expected"]["message"]
        else:
            assert source_record(str(path)) == {
                "path": str(path),
                **case["independent_payload_identity"],
            }
    assert Path.open is original_open and len(streams) == 1
    assert streams[0].closed and Path(streams[0].name) == path
    assert file_record(path, payload["repeat"]) == {
        "path": str(path),
        **case["independent_payload_identity"],
    }


def test_observation_event_limit():
    case = OBSERVATION_CASES["event-cap-last-and-first-rejected"]
    setup, expected = case["setup"], case["expected"]
    prior = [dict(setup["event_rows"]) for _ in range(setup["event_count"])]
    state = {"events": list(prior), "errors": []}
    action = case["actions"][0]
    before = time.monotonic()
    emit_event(state, action["event"], **action["fields"])
    after = time.monotonic()
    assert len(state["events"]) == expected["first_count"]
    last = state["events"][-1]
    assert list(last) == expected["first_appended_keys"]
    assert last["event"] == expected["first_event"] and last["path"] == expected["first_path"]
    assert math.isfinite(last["monotonic"]) and before <= last["monotonic"] <= after
    action = case["actions"][1]
    with pytest.raises(ValueError) as failure:
        emit_event(state, action["event"], **action["fields"])
    assert str(failure.value) == expected["second_message"]
    assert len(state["events"]) == expected["second_count"]
    assert state["events"][:-1] == prior and state["events"][-1] is last
    assert state["errors"] == []


def test_observation_failure_limit(tmp_path):
    case = OBSERVATION_CASES["failure-log-sixteen-and-overflow"]
    state, actual = {"events": [], "errors": []}, []
    for index in range(case["setup"]["real_missing_files"]):
        path = tmp_path / f"missing-{index}"
        try:
            path.read_bytes()
        except FileNotFoundError as error:
            actual.append({"type": type(error).__name__, "message": str(error)})
            record_observation_failure(state, error)
        else:
            pytest.fail("missing fixture unexpectedly opened")
    expected = case["expected"]
    assert state["errors"] == actual[: expected["recorded_errors"]]
    assert [row["type"] for row in state["errors"]] == expected["types"]
    assert state["error_overflow"] is expected["error_overflow"] and state["events"] == []


@pytest.mark.parametrize("case", OBSERVATION_BOUNDARIES[4:6], ids=lambda row: row["id"])
def test_observation_ipc_close_order(case):
    state = {"events": [], "errors": []}
    reader, writer = os.pipe()
    if case["id"] == "ipc-close-two-owned-endpoints":
        # Transfer both live endpoints once; the helper owns their actual close attempts.
        assert close_ipc(state, (reader, writer)) is None
        assert state["ipc_closed"] == [reader, writer] and state["errors"] == []
    else:
        with os.fdopen(reader, "rb") as stream:
            assert close_ipc(state, (-1, writer)) is None
            assert state["ipc_closed"] == [writer]
            assert state["errors"] == case["expected"]["errors"]
            assert stream.read(1) == b""
        assert stream.closed
    assert state["events"] == []


@pytest.mark.parametrize("case", OBSERVATION_BOUNDARIES[6:8], ids=lambda row: row["id"])
def test_observation_save_bounds(case, tmp_path):
    output = tmp_path / "observation.json"
    payload = case["setup"]["data"]["x"]
    data = {"x": payload["byte"] * payload["repeat"]}
    if "exception" in case["expected"]:
        with pytest.raises(ValueError) as failure:
            save_observation(output, data, None)
        assert str(failure.value) == case["expected"]["message"] and not output.exists()
    else:
        assert save_observation(output, data, None) is None
        assert file_record(output, case["expected"]["encoded_bytes"]) == {
            "path": str(output),
            **case["independent_encoded_identity"],
        }
        encoded = output.read_bytes()
        assert encoded.endswith(b"\n") and json.loads(encoded) == data


def test_observation_collision_without_target_error(tmp_path):
    case = OBSERVATION_CASES["observation-save-collision-no-original"]
    output = tmp_path / "observation.json"
    output.write_text(case["setup"]["output_utf8"])
    with pytest.raises(FileExistsError) as failure:
        save_observation(output, case["setup"]["data"], None)
    assert failure.value.filename == str(output)
    assert output.read_text() == case["expected"]["output_utf8"]


@pytest.mark.parametrize("case", OBSERVATION_BOUNDARIES[9:11], ids=lambda row: row["id"])
def test_observation_growth_bounds(case, tmp_path):
    target = tmp_path / "target.json"
    payload = case["setup"]["target"]
    target.write_bytes(payload["byte"].encode() * payload["repeat"])
    with pipe_channels(True) as (pipes, close, closed):
        if "exception" in case["expected"]:
            with pytest.raises(ValueError) as failure:
                mutate_and_acknowledge(target, "grow", pipes["ack_write"])
            assert str(failure.value) == case["expected"]["message"]
            close("ack_write")
            assert os.read(pipes["ack_read"], 1) == b""
        else:
            assert (
                mutate_and_acknowledge(target, "grow", pipes["ack_write"])
                == case["independent_expected_after_identity"]
            )
            assert os.read(pipes["ack_read"], 1) == case["expected"]["ack_utf8"].encode()
    assert len(closed) == len(set(closed)) == 4 and not pipes
    assert file_record(target, case["expected"]["target_bytes"]) == {
        "path": str(target),
        **case["independent_expected_after_identity"],
    }
    assert target.read_bytes().endswith(case["expected"]["mutation_suffix_utf8"].encode())


def test_observation_unknown_mutation(tmp_path):
    case = OBSERVATION_CASES["mutation-unknown-kind"]
    target = tmp_path / "target.json"
    target.write_text(case["setup"]["target_utf8"])
    with pipe_channels(True) as (pipes, close, closed):
        with pytest.raises(ValueError) as failure:
            mutate_and_acknowledge(target, case["setup"]["kind"], pipes["ack_write"])
        assert str(failure.value) == case["expected"]["message"]
        close("ack_write")
        assert os.read(pipes["ack_read"], 1) == b""
    assert len(closed) == 4 and not pipes
    assert target.read_text() == case["expected"]["target_utf8"]


def test_observation_empty_pipe_deadline():
    case = OBSERVATION_CASES["empty-pipe-receive-deadline"]
    original_deadline = suite_binding()["original_deadline"]
    with pipe_channels(True) as (pipes, close, closed):
        before = time.monotonic()
        deadline = min(before + 0.25, original_deadline)
        assert deadline > before
        with pytest.raises(TimeoutError) as failure:
            receive_token(pipes["ack_read"], b"A", deadline)
        assert str(failure.value) == case["expected"]["message"]
        assert time.monotonic() >= deadline
        close("ack_write")
        assert os.read(pipes["ack_read"], 1) == b""
    assert len(closed) == 4 and not pipes


@pytest.mark.parametrize("kind", ["trace", "profile"])
def test_observation_existing_owner(kind):
    case = OBSERVATION_CASES[f"existing-{kind}-owner-rejected"]
    previous = sys.gettrace(), sys.getprofile()
    with scoped_observation_owner(kind) as owner:
        current = {"trace": sys.gettrace, "profile": sys.getprofile}[kind]
        assert current() is owner
        with pytest.raises(RuntimeError) as failure, observe_audit() as events:
            run_observed(argparse.Namespace(), [])
        assert str(failure.value) == case["expected"]["message"] and not events
    assert (sys.gettrace(), sys.getprofile()) == previous


def test_observation_command_separator(tmp_path, monkeypatch, capsys):
    case = OBSERVATION_CASES["main-requires-command-separator"]
    target, output = tmp_path / "missing.py", tmp_path / "observation.json"
    original_argv = sys.argv
    with pipe_channels(True) as (pipes, _close, closed), monkeypatch.context() as patch:
        patch.setattr(
            sys,
            "argv",
            [
                "observe_cli.py",
                "--cli",
                str(target),
                "--observation",
                str(output),
                "--target",
                str(target),
                "--after-close",
                "",
                "--ready-fd",
                str(pipes["ready_write"]),
                "--ack-fd",
                str(pipes["ack_read"]),
                "--deadline",
                str(suite_binding()["original_deadline"]),
                "--kind",
                "grow",
            ],
        )
        with pytest.raises(SystemExit) as failure:
            observation_main()
        assert failure.value.code == case["expected"]["code"]
        assert case["expected"]["stderr_contains"] in capsys.readouterr().err
    assert sys.argv is original_argv and len(closed) == 4 and not pipes
    assert not target.exists() and not output.exists()


def test_observation_nonfinite_output(tmp_path):
    case = OBSERVATION_CASES["nonfinite-observation-payload"]
    output = tmp_path / "observation.json"
    with pytest.raises(ValueError) as failure:
        save_observation(output, {"value": float("nan")}, None)
    assert case["expected"]["message_contains"] in str(failure.value)
    assert not output.exists()


DISPATCH_GUARDS = fixture_document("process-observation-guards.json")["dispatch_cases"]


@pytest.mark.parametrize("case", DISPATCH_GUARDS, ids=lambda row: row["id"])
def test_observation_dispatch_uses_actual_frame(case, tmp_path):
    setup, expected = case["setup"], case["expected"]
    path = tmp_path / "input.txt"
    if "file_utf8" in setup:
        path.write_text(setup["file_utf8"])
    fatal = KeyboardInterrupt(setup["fatal_message"]) if "fatal_message" in setup else None
    callback, observed = observation_callback(path, fatal)
    frame = sys._getframe()
    admitted = set() if setup["admitted_names"] == [] else {frame.f_code.co_filename}
    state = {"events": [], "errors": []}
    if fatal is not None:
        with pytest.raises(KeyboardInterrupt) as failure:
            dispatch_observation(frame, "call", path, callback, admitted, state, None)
        assert failure.value is fatal
    else:
        result = dispatch_observation(frame, "call", path, callback, admitted, state, callback)
        assert result is (callback if admitted else None)
    assert len(observed["calls"]) == expected["callback_calls"]
    for received_frame, event, arg in observed["calls"]:
        assert received_frame is frame and event == "call" and arg is path
    assert state["events"] == []
    if "error_type" in expected:
        assert len(observed["errors"]) == len(state["errors"]) == expected["errors_count"]
        error = observed["errors"][0]
        assert type(error).__name__ == expected["error_type"]
        assert state["errors"] == [{"type": type(error).__name__, "message": str(error)}]
        if fatal is not None:
            assert error is fatal and str(error) == expected["error_message"]
        else:
            assert error.filename == str(path) and not path.exists()
        assert observed["reads"] == []
    else:
        assert state["errors"] == expected["state_errors"] and observed["errors"] == []
        assert observed["reads"] == ([expected["read_utf8"].encode()] if admitted else [])
        assert path.read_text() == setup["file_utf8"]


def test_observation_removed_target_reappears(tmp_path):
    case = OBSERVATION_CASES["remove-target-reappears"]
    target, other = tmp_path / "target.json", tmp_path / "other.json"
    target.write_text(case["setup"]["target_utf8"])
    other.write_text("unrelated")
    original_unlink = Path.unlink
    with pipe_channels(True) as (pipes, close, closed):
        with recreate_after_unlink(target, case["expected"]["target_utf8"].encode()) as unlinked:
            other.unlink()
            assert not other.exists() and unlinked == []
            with pytest.raises(ValueError) as failure:
                mutate_and_acknowledge(target, "remove", pipes["ack_write"])
        assert str(failure.value) == case["expected"]["message"]
        assert unlinked == [target] and Path.unlink is original_unlink
        close("ack_write")
        assert os.read(pipes["ack_read"], 1) == b""
    assert len(closed) == 4 and not pipes
    assert target.read_text() == case["expected"]["target_utf8"]


DESCRIPTOR_CLOSE_GUARDS = fixture_document("process-observation-guards.json")[
    "descriptor_close_cases"
]


@pytest.mark.parametrize("case", DESCRIPTOR_CLOSE_GUARDS, ids=lambda row: row["id"])
def test_observation_descriptor_close_bookkeeping(case):
    setup, expected = case["setup"], case["expected"]
    state = {"events": list(setup["initial_events"]), "errors": list(setup["initial_errors"])}
    suite = suite_binding()
    require_suite_time(suite)
    if setup["kind"] == "actual-invalid-descriptor-close":
        live = dict(setup["initial_live"])
        with pytest.raises(OSError) as actual:
            os.close(setup["descriptor"])
        assert type(actual.value).__name__ == expected["actual_exception"]
        assert actual.value.errno == getattr(errno, expected["actual_errno_symbol"])
        assert actual.value.errno == expected["actual_errno"]
        # This is a constructed direct unit input, separate from the actual syscall above.
        with pytest.raises(OSError) as failure:
            record_descriptor_close(state, live, setup["direct_fd"], setup["direct_event"])
        assert type(failure.value).__name__ == expected["helper_exception"]
        assert str(failure.value) == expected["helper_message"]
        assert failure.value.errno is expected["helper_errno"]
        assert failure.value.__cause__ is expected["helper_cause"]
        assert failure.value.__context__ is expected["helper_context"]
        assert (failure.value is actual.value) is expected[
            "helper_exception_is_actual_close_exception"
        ]
        assert live == expected["live"] and state == {
            "events": expected["events"],
            "errors": expected["errors"],
        }
        require_suite_time(suite)
        return

    reader, writer = os.pipe()
    owned, closed = {reader, writer}, []
    try:
        live = {reader: setup["live_occurrence"]} if "live_occurrence" in setup else {}
        before_live = dict(live)
        selected = writer if setup["direct_fd"] is None else reader
        actual_result = os.close(selected)
        owned.remove(selected)
        closed.append(selected)
        assert actual_result is expected["actual_close_result"]
        fd = None if setup["direct_fd"] is None else reader
        # These explicit callback arguments are unit inputs, not captured profile events.
        before = time.monotonic()
        if "helper_exception" in expected:
            with pytest.raises(KeyError) as failure:
                record_descriptor_close(state, live, fd, setup["direct_event"])
            assert type(failure.value).__name__ == expected["helper_exception"]
            assert failure.value.args == (reader,) and str(failure.value) == str(reader)
            assert failure.value.__cause__ is expected["helper_cause"]
            assert failure.value.__context__ is expected["helper_context"]
            assert live == expected["live"] and state["events"] == expected["events"]
        else:
            result = record_descriptor_close(state, live, fd, setup["direct_event"])
            after = time.monotonic()
            assert result is expected["return_value"]
            if fd is None:
                assert live == before_live and reader in owned
                assert state["events"] == expected["events"]
            else:
                assert live == expected["live"] and len(state["events"]) == expected["event_count"]
                event = state["events"][0]
                assert list(event) == expected["event_keys"]
                assert event["event"] == expected["event"] and event["fd"] == reader
                assert event["occurrence"] == expected["event_occurrence"]
                assert math.isfinite(event["monotonic"])
                assert before <= event["monotonic"] <= after < suite["original_deadline"]
        assert state["errors"] == expected["errors"]
    finally:
        for fd in sorted(owned):
            os.close(fd)
            owned.remove(fd)
            closed.append(fd)
    assert not owned and len(closed) == len(set(closed)) == 2
    assert set(closed) == {reader, writer}
    require_suite_time(suite)
