"""Load admitted literal records and observe the actual bounded reader."""

from __future__ import annotations

import json
from functools import cache
from pathlib import Path
from typing import Any, cast

import pytest
import rfc8785

from arboresce_intelligence_validation.record_codec import (
    DecodedRecord,
    RecordCodec,
    RecordFailure,
    RecordLimits,
    decode_record,
)

type Row = dict[str, Any]
ROOT = Path(__file__).resolve().parents[3]


@cache
def record_inputs() -> Row:
    path = ROOT / "tests/contract/intelligence/v1/record-codecs.json"
    return cast("Row", json.loads(path.read_text("utf-8")))


def record_bytes(payload: str) -> bytes:
    return (ROOT / "tests/contract/intelligence/v1" / payload).read_bytes()


def observe_record(
    codec: str, data: object, max_bytes: int, depth: int
) -> DecodedRecord | RecordFailure:
    return decode_record(RecordCodec(codec), data, RecordLimits(max_bytes, depth))


def accepted_limits(max_bytes: Any, depth: Any) -> bool:
    try:
        RecordLimits(max_bytes, depth)
    except ValueError:
        return False
    return True


def intake_form(form: str, data: bytes) -> object:
    return {
        "bytes": lambda: data,
        "text": lambda: data.decode("utf-8"),
        "bytearray": lambda: bytearray(data),
        "memoryview": lambda: memoryview(data),
        "none": lambda: None,
        "integer": lambda: 0,
    }[form]()


def canonicalizer_refusal(
    monkeypatch: pytest.MonkeyPatch, data: bytes
) -> DecodedRecord | RecordFailure:
    def reject(*_args: Any, **_kwargs: Any) -> None:
        raise rfc8785.CanonicalizationError("private input must not escape")

    monkeypatch.setattr(rfc8785, "dump", reject)
    return observe_record("RepresentationRecord", data, 1_048_576, 64)
