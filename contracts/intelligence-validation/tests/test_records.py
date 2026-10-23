"""Independently admitted canonical-record expectations stay in assertions."""

import hashlib
from dataclasses import FrozenInstanceError

import pytest
from record_support import (
    accepted_limits,
    canonicalizer_refusal,
    intake_form,
    observe_record,
    record_bytes,
    record_inputs,
)

from arboresce_intelligence_validation.record_codec import (
    DecodedRecord,
    RecordCodec,
    RecordFailure,
    RecordLimits,
    decode_record,
)

INPUTS = record_inputs()
RECORDS = INPUTS["cases"]
LIMITS = INPUTS["limit_inputs"]
FORMS = INPUTS["input_forms"]


def test_complete_independently_admitted_record_inventory():
    assert len(RECORDS) == len({row["id"] for row in RECORDS}) == 80
    assert sum(row["expected"]["admitted"] for row in RECORDS) == 26
    assert {row["codec"] for row in RECORDS} == {codec.value for codec in RecordCodec}
    assert len(RecordCodec) == 14
    assert len(LIMITS) == 12 and len(FORMS) == 6


@pytest.mark.parametrize("row", RECORDS, ids=lambda row: row["id"])
def test_all_frozen_complete_record_inputs(row):
    data = record_bytes(row["payload"])
    assert len(data) == row["expected"]["bytes"]
    assert hashlib.sha256(data).hexdigest() == row["expected"]["sha256"]
    observed = observe_record(
        row["codec"], data, row["limits"]["max_record_bytes"], row["limits"]["max_json_depth"]
    )
    assert isinstance(observed, DecodedRecord) == row["expected"]["admitted"]
    if row["expected"]["admitted"]:
        assert observed.canonical_jcs == data
        assert observed.sha256 == row["expected"]["sha256"]
        assert observed.codec.value == row["codec"]
    else:
        assert observed is RecordFailure.RECORD


@pytest.mark.parametrize("row", LIMITS, ids=lambda row: row["id"])
def test_limits_require_explicit_exact_integers(row):
    assert (
        accepted_limits(row["max_record_bytes"], row["max_json_depth"]) == row["expected_accepted"]
    )


@pytest.mark.parametrize("row", FORMS, ids=lambda row: row["form"])
def test_only_exact_immutable_record_bytes_are_admitted(row):
    data = record_bytes("records/representation-record.jcs")
    observed = observe_record("RepresentationRecord", intake_form(row["form"], data), 1_048_576, 64)
    assert isinstance(observed, DecodedRecord) == row["expected_admitted"]


def test_read_results_are_immutable_and_redact_input_from_repr():
    data = record_bytes("records/candidate-proposal-record.jcs")
    observed = observe_record("CandidateProposalRecord", data, 1_048_576, 64)
    assert isinstance(observed, DecodedRecord)
    assert repr(observed) == "DecodedRecord(<redacted>)"
    with pytest.raises(FrozenInstanceError):
        observed.canonical_jcs = b"replacement"


def test_canonicalizer_errors_remain_static_without_input(monkeypatch):
    data = record_bytes("records/representation-record.jcs")
    assert canonicalizer_refusal(monkeypatch, data) is RecordFailure.RECORD
    assert RecordFailure.RECORD.value == "invalid intelligence record"


def test_plain_string_codec_cannot_bypass_own_identity_checks():
    data = record_bytes("records/record-wrong-own-digest-candidateproposalrecord.jcs")
    assert (
        decode_record("CandidateProposalRecord", data, RecordLimits(1_048_576, 64))
        is RecordFailure.RECORD
    )


def test_limits_require_the_checked_configuration_type():
    data = record_bytes("records/representation-record.jcs")
    assert decode_record(RecordCodec.REPRESENTATION, data, None) is RecordFailure.RECORD
