"""Materialize admitted input bytes and load the fixed committed descriptor."""

from __future__ import annotations

import json
from functools import cache
from pathlib import Path
from typing import Any, cast

import pytest
from google.protobuf import descriptor_pb2, descriptor_pool, message_factory
from google.protobuf.message import DecodeError, Message

from arboresce_intelligence_validation.intelligence_wire import WireFailure, decode_frame

type Row = dict[str, Any]
ROOT = Path(__file__).resolve().parents[3]


@cache
def fixture(name: str) -> Row:
    return cast(
        "Row",
        json.loads((ROOT / f"tests/contract/intelligence/v1/{name}.json").read_text("utf-8")),
    )


@cache
def pool() -> descriptor_pool.DescriptorPool:
    raw = (ROOT / "contracts/internal/proto/arboresce/intelligence/v1/processing.pb").read_bytes()
    descriptor = descriptor_pb2.FileDescriptorSet()
    descriptor.ParseFromString(raw)
    result = descriptor_pool.DescriptorPool()
    for entry in descriptor.file:
        result.AddSerializedFile(entry.SerializeToString())
    return result


def message_type(name: str) -> type[Message]:
    return message_factory.GetMessageClass(
        pool().FindMessageTypeByName(f"arboresce.intelligence.v1.{name}")
    )


def observe(name: str, data: object) -> Message | WireFailure:
    return decode_frame(message_type(name), data)


def boundary_bytes(prefix_hex: str, repeat_ascii: str, repeat_count: int) -> bytes:
    return bytes.fromhex(prefix_hex) + repeat_ascii.encode("ascii") * repeat_count


def runtime_refusal(monkeypatch: pytest.MonkeyPatch) -> Message | WireFailure:
    kind = message_type("Scope")

    def reject(_self: Message, _data: bytes) -> int:
        raise DecodeError("private input must not escape")

    monkeypatch.setattr(kind, "ParseFromString", reject)
    return observe("Scope", b"")
