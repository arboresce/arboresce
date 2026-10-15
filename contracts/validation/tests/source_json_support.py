"""Expand frozen inputs and observe actual source results, without gold generation."""

from __future__ import annotations

import copy
import json
from collections.abc import Callable, Iterator
from functools import cache
from pathlib import Path
from typing import Any, cast

from arboresce_contract_validation import source_json as source_module
from arboresce_contract_validation.source_json import (
    CanonicalJsonSelection,
    SourceFailure,
    SourceJson,
)

type Row = dict[str, Any]
ROOT = Path(__file__).resolve().parents[3]


@cache
def fixture(name: str) -> Row:
    if name == "python":
        path = Path(__file__).parent / "fixtures/source-json-python-v1/cases.json"
    else:
        path = ROOT / f"crates/arboresce-domain/tests/fixtures/{name}/cases.json"
    return cast("Row", json.loads(path.read_text(encoding="utf-8")))


def pointer_fixture() -> list[Row]:
    path = ROOT / "crates/arboresce-domain/tests/fixtures/primitives-v1/semantics.json"
    return cast("list[Row]", json.loads(path.read_text(encoding="utf-8"))["groups"]["json_pointer"])


def recipe(value: Any) -> bytes:
    if isinstance(value, str):
        return value.encode("utf-8")
    if "parts" in value:
        return b"".join(recipe(part) for part in value["parts"])
    if "repeat" in value:
        return cast("str", value["repeat"]).encode("utf-8") * value["count"]
    count = cast("int", value["object_members"])
    indices = range(count - 1, -1, -1) if value["reverse"] else range(count)
    parts = [
        json.dumps(f"{value['key_prefix']}{index:06d}", ensure_ascii=False).encode("utf-8")
        + b":"
        + value["value_utf8"].encode("utf-8")
        for index in indices
    ]
    for member in value.get("additional_members", []):
        parts.append(
            json.dumps(member["key"], ensure_ascii=False).encode("utf-8")
            + b":"
            + member["value_utf8"].encode("utf-8")
        )
    return b"{" + b",".join(parts) + b"}"


def source_input(row: Row) -> bytes:
    if "source_bytes" in row:
        return bytes(row["source_bytes"])
    return recipe(row["source_utf8"] if "source_utf8" in row else row["source_recipe"])


def observe(data: object, pointer: object = "") -> Row:
    source = SourceJson.parse(data)
    if isinstance(source, SourceFailure):
        return {"stage": source.stage}
    selection = source.select(pointer)
    if isinstance(selection, SourceFailure):
        return {"stage": selection.stage}
    return {
        "stage": "success",
        "canonical_utf8": selection.as_bytes().decode("utf-8"),
        "canonical_bytes": selection.byte_length(),
        "sha256": selection.sha256().hex(),
        "canonical_type": type(selection.as_bytes()).__name__,
        "digest_type": type(selection.sha256()).__name__,
        "digest_length": len(selection.sha256()),
    }


def admitted(text: str, pointer: str = "") -> tuple[SourceJson, CanonicalJsonSelection]:
    source = SourceJson.parse(text.encode("utf-8"))
    if isinstance(source, SourceFailure):
        raise ValueError("fixture source was not admitted")
    selection = source.select(pointer)
    if isinstance(selection, SourceFailure):
        raise ValueError("fixture selection was not admitted")
    return source, selection


def hostile_values(text: str, pointer: str) -> tuple[bytes, str, list[int]]:
    counter = [0]

    def unexpected() -> Any:
        counter[0] += 1
        raise RuntimeError("unexpected conversion callback")

    class HostileBytes(bytes):
        def __bytes__(self) -> bytes:
            return unexpected()

        def __len__(self) -> int:
            return unexpected()

        def __iter__(self) -> Iterator[int]:
            return unexpected()

        def __getitem__(self, _key: Any, /) -> Any:
            return unexpected()

    class HostileStr(str):
        def __str__(self) -> str:
            return unexpected()

        def __len__(self) -> int:
            return unexpected()

        def __iter__(self) -> Iterator[str]:
            return unexpected()

        def __getitem__(self, _key: Any, /) -> Any:
            return unexpected()

        def encode(self, encoding: str = "utf-8", errors: str = "strict") -> bytes:
            return unexpected()

    raw = bytes.__new__(HostileBytes, text.encode("utf-8"))
    token = str.__new__(HostileStr, pointer)
    return raw, token, counter


def typed_observation(row: Row, *, pointer_kind: bool) -> tuple[Row, int]:
    text, pointer = cast("str", row["source_utf8"]), cast("str", row["pointer"])
    hostile_bytes, hostile_str, counter = hostile_values(text, pointer)
    if pointer_kind:
        raw: object = text.encode("utf-8")
        encoded = pointer.encode("utf-8") if row["kind"] in ("bytes", "memoryview") else b""
        choices: dict[str, object] = {
            "str": pointer,
            "str_subclass": hostile_str,
            "bytes": encoded,
            "memoryview": memoryview(encoded),
            "none": None,
            "bool": True,
            "int": 0,
        }
        # Surrogate pointer rows never require encoding or a hostile subclass.
        token: object = choices[row["kind"]]
    else:
        choices = {
            "bytes": text.encode("utf-8"),
            "bytearray": bytearray(text.encode("utf-8")),
            "memoryview": memoryview(text.encode("utf-8")),
            "str": text,
            "bytes_subclass": hostile_bytes,
            "none": None,
            "bool": True,
            "int": 0,
        }
        raw, token = choices[row["kind"]], pointer
    counter[0] = 0
    result = observe(raw, token)
    return result, counter[0]


def exception_name(operation: Callable[[], object]) -> str:
    try:
        operation()
    except Exception as error:
        return type(error).__name__
    return "no exception"


def api_observation(row: Row) -> Any:
    action = row["action"]
    source_type, selection_type = cast("Any", SourceJson), cast("Any", CanonicalJsonSelection)
    if action == "source_constructor":
        return [
            exception_name(call)
            for call in [
                lambda: source_type(),
                lambda: source_type(b"null"),
                lambda: source_type(_nodes=()),
            ]
        ]
    if action == "selection_constructor":
        return [
            exception_name(call)
            for call in [
                lambda: selection_type(),
                lambda: selection_type(b"null", bytes(32)),
                lambda: selection_type(_canonical=b"null", _digest=bytes(32)),
            ]
        ]
    source, selection = admitted(row["source_utf8"], row.get("pointer", ""))
    if action == "source_assignment":
        return exception_name(lambda: setattr(source, "_nodes", ()))
    if action == "source_deletion":
        return exception_name(lambda: delattr(source, "_nodes"))
    if action == "selection_assignment":
        return exception_name(lambda: setattr(selection, "_canonical", b"false"))
    if action == "selection_deletion":
        return exception_name(lambda: delattr(selection, "_digest"))
    if action == "source_dict":
        return exception_name(lambda: vars(source))
    if action == "selection_dict":
        return exception_name(lambda: vars(selection))
    if action in ("copy", "deepcopy"):
        other = copy.copy(selection) if action == "copy" else copy.deepcopy(selection)
        return dict(equal=other == selection, same_object=other is selection)
    if action == "foreign_equality":
        return cast("Any", selection) == b"null"
    if action == "bytes_mutation":

        def mutate() -> None:
            data = cast("Any", selection.as_bytes())
            data[0] = 0x78

        return exception_name(mutate)
    if action == "getters":
        return dict(
            canonical_utf8=selection.as_bytes().decode("utf-8"),
            byte_length=selection.byte_length(),
            sha256=selection.sha256().hex(),
            canonical_type=type(selection.as_bytes()).__name__,
        )
    if action == "digest":
        return dict(
            type=type(selection.sha256()).__name__,
            length=len(selection.sha256()),
            sha256=selection.sha256().hex(),
        )
    raise ValueError("unknown frozen API action")


def adapter_observation(row: Row) -> Any:
    factory = cast("Callable[[str, int], str]", vars(source_module)["_key"])
    action = row["action"]
    if action == "constructor":
        key_type = vars(source_module)["_CanonicalKey"]
        return [
            exception_name(call)
            for call in [
                lambda: key_type(),
                lambda: key_type("a"),
                lambda: key_type("a", 0),
                lambda: key_type(_ordinal=0),
            ]
        ]
    key = factory(row["text"], row["ordinal"])
    if action == "assignment":
        return exception_name(lambda: setattr(key, row["attribute"], row["assigned_value"]))
    if action == "deletion":
        return exception_name(lambda: delattr(key, row["attribute"]))
    if action == "self":
        other: object = key
    elif action == "plain":
        other = row["text"]
    elif action == "capsule":
        other = factory(row["other_text"], row["other_ordinal"])
    else:
        raise ValueError("unknown frozen adapter action")
    return dict(
        equal=key == other,
        unequal=key != other,
        reflected_equal=other == key,
        reflected_unequal=other != key,
        expected_hash=hash(key),
        utf16=key.encode("utf-16be"),
        utf8=key.encode("utf-8"),
    )


def number_observation(token: str) -> Row:
    consumer = cast(
        "Callable[[str], tuple[float, bytes]]", vars(source_module)["_number_from_token"]
    )
    rejection = cast("type[Exception]", vars(source_module)["_Reject"])
    try:
        _, canonical = consumer(token)
    except rejection:
        return {"accepted": False}
    return {"accepted": True, "canonical_utf8": canonical.decode("ascii")}
