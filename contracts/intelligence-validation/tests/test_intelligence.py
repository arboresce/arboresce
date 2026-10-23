"""Independently admitted closed-wire literals; no stage semantic claim."""

import importlib.metadata
import sys

import pytest
from google.protobuf.internal import api_implementation
from intelligence_support import boundary_bytes, fixture, observe, pool, runtime_refusal

from arboresce_intelligence_validation.intelligence_wire import (
    MAX_FRAME_BYTES,
    MAX_MESSAGE_DEPTH,
    WireFailure,
)

WIRE = fixture("wire-vectors")["cases"] + fixture("wire-supplement")["cases"]
BOUNDARIES = fixture("wire-boundaries")["cases"]


def test_complete_independent_inventory_and_runtime_selection():
    assert len(fixture("wire-vectors")["cases"]) == 42
    assert len(WIRE) == 50
    assert sum(row["valid"] for row in WIRE) == 19
    assert len(BOUNDARIES) == 2
    assert len({row["id"] for row in WIRE + BOUNDARIES}) == 52
    assert importlib.metadata.version("protobuf") == "7.36.2"
    assert sys.version_info[:3] == (3, 14, 7)
    assert api_implementation.Type() == "python"
    assert MAX_FRAME_BYTES == 1_048_576
    assert MAX_MESSAGE_DEPTH == 16
    descriptor = pool().FindFileByName("arboresce/intelligence/v1/processing.proto")
    assert descriptor.package == "arboresce.intelligence.v1"
    assert len(descriptor.message_types_by_name) == 20
    assert len(descriptor.enum_types_by_name) == 2
    assert len(descriptor.services_by_name) == 1


@pytest.mark.parametrize("row", WIRE, ids=lambda row: row["id"])
def test_all_frozen_wire_literals(row):
    raw = bytes.fromhex(row["bytes_hex"])
    observed = observe(row["message"], raw)
    assert (observed is not WireFailure.FRAME) == row["valid"]
    assert raw.hex() == row["bytes_hex"]


@pytest.mark.parametrize("row", BOUNDARIES, ids=lambda row: row["id"])
def test_complete_frame_cap_before_decoding(row):
    raw = boundary_bytes(row["prefix_hex"], row["repeat_ascii"], row["repeat_count"])
    assert len(raw) == row["frame_bytes"]
    assert (observe(row["message"], raw) is not WireFailure.FRAME) == row["valid"]


def test_explicit_zero_remains_present_and_absent_stays_absent():
    present, absent = observe("Binding", b"\x28\x00"), observe("Binding", b"")
    assert present.HasField("stage_ordinal")
    assert not absent.HasField("stage_ordinal")


def test_field_order_and_repeated_occurrences_survive_runtime_decoding():
    first = observe("Scope", bytes.fromhex("0a01611201621a0163"))
    other = observe("Scope", bytes.fromhex("1a01631201620a0161"))
    assert first == other
    request = observe("StageRequest", bytes.fromhex("42040a027b7d42040a027b7d"))
    assert len(request.prior_results) == 2
    assert request.prior_results[0].canonical_jcs == b"{}"
    assert request.prior_results[1].canonical_jcs == b"{}"


@pytest.mark.parametrize("value", [None, False, 0, "", bytearray(), memoryview(b"")])
def test_only_exact_immutable_frame_bytes_are_admitted(value):
    assert observe("Scope", value) is WireFailure.FRAME


def test_pinned_runtime_decode_failure_remains_a_static_refusal(monkeypatch):
    assert runtime_refusal(monkeypatch) is WireFailure.FRAME
    assert WireFailure.FRAME.value == "invalid intelligence frame"
