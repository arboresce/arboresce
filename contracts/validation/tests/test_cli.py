"""Execute the actual command across a process boundary and preserve complete streams."""

from __future__ import annotations

import json
import os

import pytest
from support import boundary_cases, cli_arguments, materialize, run_cli

PROCESS_CASES = [
    row
    for row in boundary_cases(
        "parser-order",
        "cli-preflight",
        "output",
        "reference",
        "resolution",
        "retrieval",
        "cli-usage",
    )
    if "barrier" not in row["setup"]
]


@pytest.mark.parametrize("case", PROCESS_CASES, ids=lambda row: row["id"])
def test_cli_process(case, tmp_path):
    setup, expected = case["setup"], case["expected"]
    root = tmp_path / "original"
    materialize(root, setup.get("files", {}))
    if case["id"] == "preflight-later-unreadable":
        assert os.geteuid() != 0
        assert not os.access(root / "contracts/second.json", os.R_OK)
    if case["area"] == "retrieval":
        (root / "outside.schema.json").write_text('{"const":7}')
    result = run_cli(root, cli_arguments(case), tmp_path / "process")
    assert result.returncode == expected["exit"], (result.stdout, result.stderr)
    for field in ("stdout", "stderr"):
        value = getattr(result, field)
        if field in expected:
            assert value == expected[field]
        for text in expected.get(field + "_contains", []):
            assert text in value
        for text in expected.get(field + "_contains_additional", []):
            assert text in value
    for text in expected.get("diagnostic_contains", []):
        assert text in result.stderr
    for text in expected.get("output_forbids", []):
        assert text not in result.stdout + result.stderr
    if "json_stdout" in expected:
        assert json.loads(result.stdout) == expected["json_stdout"]
    if expected.get("validation_marker_absent"):
        assert "Schema validation errors were encountered." not in result.stdout + result.stderr
        assert "ok -- validation done" not in result.stdout + result.stderr
    assert len(result.coverage_files) == 1


@pytest.mark.parametrize(
    "case",
    [
        row
        for row in boundary_cases("resolution")
        if "relocated" in row["action"] or "both roots" in row["action"]
    ],
    ids=lambda row: row["id"],
)
def test_relocated_process(case, tmp_path):
    setup, expected = case["setup"], case["expected"]
    root = tmp_path / "relocated"
    materialize(root, setup["files"])
    result = run_cli(root, cli_arguments(case), tmp_path / "process")
    assert result.returncode == expected["exit"]
    assert result.stdout == expected["stdout"]
    assert result.stderr == expected["stderr"]


def test_fresh_processes_observe_mutation_and_restoration(tmp_path):
    (case,) = boundary_cases("freshness")
    setup, expected = case["setup"], case["expected"]
    root = tmp_path / "mirror"
    materialize(root, setup["files"])
    library = root / "contracts/lib.schema.json"
    original = library.read_bytes()
    results = []
    for index, data in enumerate((original, b'{"const":18}', original)):
        library.write_bytes(data)
        results.append(run_cli(root, cli_arguments(case), tmp_path / f"process-{index}"))
    assert [result.returncode for result in results] == expected["exits"]
    assert results[0].stdout == results[2].stdout == "ok -- validation done\n"
    assert expected["middle_message"] in results[1].stdout
    assert [result.stderr for result in results] == ["", "", ""]
    assert library.read_bytes() == original
    assert len({result.pid for result in results}) == 3
    assert len({path for result in results for path in result.coverage_files}) == 3


PROCESS_BARRIERS = [
    row
    for row in boundary_cases("snapshot-integration", "parser-order")
    if row["area"] == "snapshot-integration" or "barrier" in row["setup"]
]

assert [row["id"] for row in PROCESS_BARRIERS] == [
    "snapshot-role-entry",
    "snapshot-role-reference",
    "snapshot-role-instance",
    "post-preflight-later-disappears-after-parse-invalid",
    "post-preflight-later-disappears-after-validation-invalid",
]


@pytest.mark.parametrize("case", PROCESS_BARRIERS, ids=lambda row: row["id"])
def test_process_barriers(case, tmp_path):
    setup, expected = case["setup"], case["expected"]
    root = tmp_path / "mirror"
    materialize(root, setup["files"])
    request = (
        {"kind": "grow", "target": setup["target"]}
        if "target" in setup
        else {
            "kind": "remove",
            "target": setup["instances"][1],
            "after_close": setup["instances"][0],
        }
    )
    result = run_cli(root, cli_arguments(case), tmp_path / "process", observation=request)
    assert result.returncode == expected["exit"], (result.stdout, result.stderr)
    data = result.observation
    assert data is not None and not data["errors"] and not data.get("error_overflow")
    events = data["events"]
    target = str(root / request["target"])
    assert data["target"] == target and data["barrier_entered"]
    assert [row["event"] for row in events if row["event"].startswith("barrier-")] == [
        "barrier-ready",
        "barrier-ack",
    ]
    assert len(data["ipc_closed"]) == len(set(data["ipc_closed"])) == 2
    assert set(data["parent_mutation"]["closed_pipe_roles"]) == {
        "ready_read",
        "ready_write",
        "ack_read",
        "ack_write",
    }
    opened = [(row["fd"], row["occurrence"]) for row in events if row["event"] == "descriptor-open"]
    closed = [
        (row["fd"], row["occurrence"]) for row in events if row["event"] == "descriptor-close"
    ]
    assert opened and sorted(opened) == sorted(closed) and not data["live_descriptors"]
    assert data["root_fds"] == [None] and data["streams_closed"] == [True] * len(setup["instances"])
    assert target not in data["snapshots"]
    errors = [row for row in events if row["event"] == "read-error" and row["path"] == target]
    assert len(errors) == 1
    reaches_parser = any(row["event"] == "decode" and row["path"] == target for row in events)
    assert reaches_parser is expected.get("unstable_target_reaches_parser", False)
    for text in expected.get("diagnostic_contains", []) + expected.get("stderr_contains", []):
        assert text in result.stderr
    for text in expected.get(
        "output_forbids", ["Schema validation errors were encountered.", "ok -- validation done"]
    ):
        assert text not in result.stdout + result.stderr
    if request["kind"] == "grow":
        assert errors[0]["type"] == "InputAdmissionError" and errors[0]["reason"] == "changed"
        assert (
            sum(row["event"] == "old-reader" for row in events)
            == expected["old_local_schema_reader_calls"]
        )
        assert (
            not any(row["event"] == "fallback" for row in events)
            and data["preload_errors"] == [None]
        ) is expected["no_schema_fallback"]
        assert expected["all_opened_descriptors_closed"] and expected["validation_marker_absent"]
        mutation = data["parent_mutation"]
        assert mutation["after"]["bytes"] == mutation["before"]["bytes"] + 1
        assert mutation["after"]["sha256"] != mutation["before"]["sha256"]
    else:
        assert result.stdout == expected["stdout"] and data["parent_mutation"]["after"] == {
            "absent": True
        }
        assert errors[0]["type"] == "FileNotFoundError" and errors[0]["filename"] == target
        first = str(root / setup["instances"][0])
        assert (
            sum(path.endswith(".schema.json") for path in data["snapshots"])
            == expected["schema_snapshots"]
        )
        assert (
            sum(row.get(setup["instances"][0], 0) for row in data["validation_errors"])
            == expected["first_instance_validation_error_count"]
        )
        assert not any(row["event"] == "report-result" for row in events)
        mapping = {
            ("preflight", first): "preflight:first-open-close",
            ("preflight", target): "preflight:second-open-close",
            ("decode", first): "parse:first",
            ("stream-close", first): "close:first",
            ("meta-check", None): "schema-load-and-meta-check",
            ("validation-error", first): "validate:first-error",
            ("read-error", target): "second-read-error; no report of accumulated result",
        }
        assert [
            mapping[key] for row in events if (key := (row["event"], row.get("path"))) in mapping
        ] == expected["events"]
        if expected["schema_snapshots"]:
            assert next(
                index for index, row in enumerate(events) if row["event"] == "entry-schema"
            ) < next(index for index, row in enumerate(events) if row["event"] == "meta-check")
    assert len(result.coverage_files) == 1
