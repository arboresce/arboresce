"""Frozen whole-source, number, pointer and Python interface conformance."""

import copy
import hashlib
import struct
import types

import pytest
import rfc8785
from source_json_support import (
    adapter_observation,
    admitted,
    api_observation,
    exception_name,
    fixture,
    hostile_values,
    number_observation,
    observe,
    pointer_fixture,
    recipe,
    source_input,
    typed_observation,
)

from arboresce_contract_validation import source_json as consumer
from arboresce_contract_validation.source_json import (
    CanonicalJsonSelection,
    SourceFailure,
    SourceJson,
)

SOURCE = fixture("source-json-v1")
SUPPLEMENT = fixture("python")
NUMBERS = fixture("canonical-number-v1")["groups"]
SOURCE_ROWS = (
    SOURCE["literal_cases"]
    + SOURCE["boundary_cases"]
    + SOURCE["property_cases"]
    + SUPPLEMENT["adapter_boundary_cases"]
    + SUPPLEMENT["adapter_literal_cases"]
    + SUPPLEMENT["resource_cases"]
)
TYPED_ROWS = [(False, row) for row in SUPPLEMENT["source_type_cases"]] + [
    (True, row) for row in SUPPLEMENT["pointer_type_cases"]
]


def test_complete_frozen_input_inventory_and_exact_profile():
    assert [
        len(SOURCE[group])
        for group in ["literal_cases", "boundary_cases", "property_cases", "traits"]
    ] == [86, 41, 640, 12]
    assert [len(NUMBERS[group]) for group in ["numbers", "traits", "published"]] == [124, 10, 28]
    groups = [
        "source_type_cases",
        "pointer_type_cases",
        "api_cases",
        "failure_cases",
        "adapter_protocol_cases",
        "adapter_boundary_cases",
        "adapter_literal_cases",
        "resource_cases",
    ]
    assert [len(SUPPLEMENT[group]) for group in groups] == [8, 10, 14, 3, 9, 31, 1, 4]
    all_ids = [row["id"] for group in groups for row in SUPPLEMENT[group]]
    assert len(set(all_ids)) == len(all_ids) == 80
    assert len(pointer_fixture()) == 28
    profile = SOURCE["processing_profile"]
    for field, value in {
        "input_bytes": consumer.MAX_INPUT_BYTES,
        "whole_depth": consumer.MAX_WHOLE_DEPTH,
        "whole_nodes": consumer.MAX_WHOLE_NODES,
        "total_object_members": consumer.MAX_TOTAL_MEMBERS,
        "per_object_members": consumer.MAX_OBJECT_MEMBERS,
        "per_array_elements": consumer.MAX_ARRAY_ELEMENTS,
        "number_token_bytes": consumer.MAX_NUMBER_BYTES,
        "selected_depth": consumer.MAX_SELECTED_DEPTH,
        "selected_nodes": consumer.MAX_SELECTED_NODES,
        "selected_output_bytes": consumer.MAX_SELECTED_BYTES,
    }.items():
        assert profile[field] == value


@pytest.mark.parametrize("row", SOURCE_ROWS, ids=lambda row: row["id"])
def test_complete_original_and_bounded_selected_identity(row):
    original = source_input(row)
    before = hashlib.sha256(original).digest()
    if "original_bytes" in row:
        assert len(original) == row["original_bytes"]
    if "source_sha256" in row:
        assert before.hex() == row["source_sha256"]
    observed = observe(original, row["pointer"])
    expected = row["expected"]
    assert observed["stage"] == expected["stage"]
    if expected["stage"] == "success":
        canonical = recipe(expected.get("canonical_utf8", expected.get("canonical_recipe")))
        assert len(canonical) == expected["canonical_bytes"]
        assert hashlib.sha256(canonical).hexdigest() == expected["sha256"]
        assert observed == dict(
            stage="success",
            canonical_utf8=canonical.decode("utf-8"),
            canonical_bytes=expected["canonical_bytes"],
            sha256=expected["sha256"],
            canonical_type="bytes",
            digest_type="bytes",
            digest_length=32,
        )
    else:
        assert observed == {"stage": expected["stage"]}
    assert hashlib.sha256(original).digest() == before


@pytest.mark.parametrize("row", pointer_fixture(), ids=lambda row: row["id"])
def test_all_original_json_pointer_semantics(row):
    observed = observe(row["source_json"].encode("utf-8"), row["pointer"])
    assert (observed["stage"] == "success") == row["accepted"]
    if row["accepted"]:
        assert observed["canonical_utf8"] == row["selected_canonical_utf8"]
        assert observed["sha256"] == row["selected_sha256"]


@pytest.mark.parametrize("row", NUMBERS["numbers"], ids=lambda row: row["id"])
def test_original_raw_number_admission_without_json_framing_whitespace(row):
    observed = number_observation(row["input"])
    assert observed["accepted"] == row["accepted"]
    if row["accepted"]:
        canonical = observed["canonical_utf8"].encode("ascii")
        assert observed["canonical_utf8"] == row["expected"]["canonical_utf8"]
        assert len(canonical) == row["expected"]["byte_length"]
        assert hashlib.sha256(canonical).hexdigest() == row["expected"]["sha256"]


@pytest.mark.parametrize("row", NUMBERS["published"], ids=lambda row: row["id"])
def test_independent_formatter_against_published_binary64_pairs(row):
    value = struct.unpack(">d", bytes.fromhex(row["bits"]))[0]
    if row["canonical_utf8"] is None:
        with pytest.raises(rfc8785.CanonicalizationError):
            rfc8785.dumps(value)
    else:
        assert rfc8785.dumps(value) == row["canonical_utf8"].encode("ascii")


@pytest.mark.parametrize("row", NUMBERS["traits"], ids=lambda row: row["id"])
def test_shared_number_equivalence_and_selected_value_traits(row):
    _, first = admitted(row["first"])
    _, second = admitted(row["second"])
    assert (first == second) == row["expected_equal"]
    assert len({first, second}) == row["expected_set_size"]
    assert copy.copy(first) == first
    assert copy.deepcopy(first) == first
    assert repr(first) == SUPPLEMENT["api"]["selection_repr"]


@pytest.mark.parametrize("row", SOURCE["traits"], ids=lambda row: row["id"])
def test_shared_source_identity_immutability_and_failure_traits(row):
    identity = row["id"]
    if "left" in row:
        _, first = admitted(row["left"])
        _, second = admitted(row["right"])
        assert (first == second) == row["expected_equal"]
        return
    if identity == "source-json-static-source-error":
        failure = SourceJson.parse(row["source"].encode())
        assert str(failure) == row["display"]
        assert repr(failure) == row["debug"]
        assert not isinstance(failure, BaseException)
        assert getattr(failure, "__cause__", None) is row["error_source"]
        return
    if identity == "source-json-static-selection-error":
        source, _ = admitted(row["source"])
        failure = source.select(row["pointer"])
        assert str(failure) == row["display"]
        assert repr(failure) == row["debug"]
        assert not isinstance(failure, BaseException)
        assert getattr(failure, "__cause__", None) is row["error_source"]
        return
    source, selection = admitted(row["source"], row.get("pointer", ""))
    if identity == "source-json-redacted-source-debug":
        assert repr(source) == row["debug"]
    elif identity == "source-json-redacted-selection-debug":
        assert repr(selection) == row["debug"]
    else:
        assert selection.as_bytes() == row["canonical"].encode("utf-8")
        if "invalid_pointer" in row:
            assert source.select(row["invalid_pointer"]) is SourceFailure.SELECT
        assert source.select(row.get("pointer", "")) == selection
        assert copy.copy(selection) is selection
        assert copy.deepcopy(selection) is selection
        assert hash(copy.copy(selection)) == hash(selection)
        if "byte_length" in row:
            assert selection.byte_length() == row["byte_length"]


@pytest.mark.parametrize("pointer_kind,row", TYPED_ROWS, ids=[row["id"] for _, row in TYPED_ROWS])
def test_exact_types_reject_all_subclasses_before_callbacks(pointer_kind, row):
    observed, callbacks = typed_observation(row, pointer_kind=pointer_kind)
    expected = row["expected"]
    assert {k: observed[k] for k in expected} == expected, row["id"]
    assert callbacks == row["expected_callbacks"], row["id"]


@pytest.mark.parametrize("row", SUPPLEMENT["api_cases"], ids=lambda row: row["id"])
def test_frozen_python_interface_actions(row):
    observed = api_observation(row)
    if row["action"] in ("source_constructor", "selection_constructor"):
        assert observed == [row["expected"]] * 3
    elif row["action"] in ("copy", "deepcopy"):
        assert observed == dict(
            equal=row["expected_equal"], same_object=row["expected_same_object"]
        )
    elif row["action"] == "getters":
        assert observed == {k: row[k] for k in ["canonical_utf8", "byte_length", "sha256"]} | {
            "canonical_type": "bytes"
        }
    elif row["action"] == "digest":
        assert observed == dict(
            type=row["expected_type"], length=row["expected_length"], sha256=row["sha256"]
        )
    else:
        assert observed == row["expected"]


@pytest.mark.parametrize("row", SUPPLEMENT["failure_cases"], ids=lambda row: row["id"])
def test_static_failure_values_have_no_source_context_or_cause(row):
    source = SourceJson.parse(row["source_utf8"].encode())
    failure = source if isinstance(source, SourceFailure) else source.select(row["pointer"])
    assert isinstance(failure, SourceFailure)
    assert failure.stage == row["stage"]
    assert str(failure) == row["display"]
    assert repr(failure) == row["debug"]
    assert isinstance(failure, BaseException) == row["is_exception"]
    assert getattr(failure, "__context__", None) is row["context"]
    assert getattr(failure, "__cause__", None) is None


@pytest.mark.parametrize("row", SUPPLEMENT["adapter_protocol_cases"], ids=lambda row: row["id"])
def test_private_identity_key_protocol_preserves_unicode_content(row):
    observed = adapter_observation(row)
    if "expected" in row:
        assert observed == (
            [row["expected"]] * 4 if row["action"] == "constructor" else row["expected"]
        )
    else:
        assert observed == {
            k: row[k]
            for k in ["equal", "unequal", "reflected_equal", "reflected_unequal", "expected_hash"]
        } | {"utf16": row["text"].encode("utf-16be"), "utf8": row["text"].encode("utf-8")}


def test_hostile_fixture_methods_are_consequential_and_reset_after_preparation():
    raw, token, counter = hostile_values("null", "/a")
    operations = [
        lambda: bytes(raw),
        lambda: len(raw),
        lambda: iter(raw),
        lambda: raw[0],
        lambda: str(token),
        lambda: len(token),
        lambda: iter(token),
        lambda: token[0],
        lambda: token.encode("utf-8"),
    ]
    for count, operation in enumerate(operations, 1):
        with pytest.raises(RuntimeError, match="^unexpected conversion callback$"):
            operation()
        assert counter[0] == count
    counter[0] = 0
    assert observe(raw) == {"stage": "parse"}
    assert observe(b"null", token) == {"stage": "pointer"}
    assert counter[0] == 0


def test_fixture_helper_rejects_invalid_setup_and_unknown_actions():
    assert exception_name(lambda: None) == "no exception"
    with pytest.raises(ValueError, match="fixture source was not admitted"):
        admitted("{")
    with pytest.raises(ValueError, match="fixture selection was not admitted"):
        admitted("null", "/missing")
    with pytest.raises(ValueError, match="unknown frozen API action"):
        api_observation(dict(action="unknown", source_utf8="null"))
    with pytest.raises(ValueError, match="unknown frozen adapter action"):
        adapter_observation(dict(action="unknown", text="a", ordinal=0))


def test_private_writer_rejects_before_growth_and_before_partial_hash_publication(monkeypatch):
    sink_type, rejection = vars(consumer)["_Sink"], vars(consumer)["_Reject"]
    sink = sink_type(2)
    assert sink.write(b"a") == 1
    with pytest.raises(rejection):
        sink.write(b"bc")
    with pytest.raises(rejection):
        sink.finish()
    assert sink.write(b"b") == 1
    assert sink.finish() == b"ab"
    source, _ = admitted("null")

    def short_write(_tree, sink):
        sink.write(b"n")

    monkeypatch.setattr(rfc8785, "dump", short_write)
    assert source.select("") is SourceFailure.SELECT

    def overflowing_write(_tree, sink):
        sink.write(b"null!")

    monkeypatch.setattr(rfc8785, "dump", overflowing_write)
    assert source.select("") is SourceFailure.SELECT


def test_private_key_cannot_admit_caller_subclasses():
    with pytest.raises(TypeError, match="private key subclasses are unsupported"):
        types.new_class("UnsupportedKey", (vars(consumer)["_CanonicalKey"],))


def test_public_result_has_closed_exact_byte_getters():
    source, result = admitted('"é🙂"')
    assert type(source) is SourceJson
    assert type(result) is CanonicalJsonSelection
    assert type(result.as_bytes()) is bytes
    assert type(result.sha256()) is bytes
    assert len(result.sha256()) == 32
