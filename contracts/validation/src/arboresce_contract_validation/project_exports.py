"""Independent bounded readers for the reviewed focused export byte profiles.

These conformance tools reconstruct frozen values and compare byte identities.
They are not canonical writers, delivery admission, authentication, source-set
completeness, financial calculation or actual-human assurance implementations.
"""

from __future__ import annotations

import csv
import hashlib
import io
import json
import re
from collections.abc import Iterator, Sequence
from pathlib import Path
from typing import Any, NoReturn, Protocol, cast

from check_jsonschema.formats import FormatOptions
from check_jsonschema.regex_variants import RegexImplementation, RegexVariantName
from jsonschema.protocols import Validator

from arboresce_contract_validation.cli import LocalRegistryLoader, SnapshotStore

type Value = None | bool | int | float | str | list[Value] | dict[str, Value]

MAX_BYTES = 67_108_864
MAX_NODES = 1_048_576
MAX_DEPTH = 32
MAX_CELL_UNITS = 32_760
MAX_CELL_BYTES = 65_536
MAX_INTEGER = 9_007_199_254_740_991
CONTRACTS = Path(__file__).resolve().parents[3]
PROJECT_HEADER: tuple[str, ...] = tuple(
    (
        "row_kind,organization_id,domain_id,project_id,content_revision,snapshot_id,"
        "path,value_type,value"
    ).split(",")
)
FINANCIAL_HEADER: tuple[str, ...] = tuple(
    (
        "row_kind,report_id,report_sha256,record_id,content_revision,content_sha256,"
        "observed_control_revision,record_kind,currency,gross,expense_date,merchant,"
        "business_purpose,reconciliation_state,exclusion_reason,bucket,known_net,complete,"
        "unknown_record_count,unknown_list_record_count,known_null_component_count,path,"
        "value_type,value"
    ).split(",")
)
FORMATS: dict[str, str] = {
    "project-json-v1": "project_export_payload",
    "project-csv-v1": "project_export_payload",
    "project-financial-json-v1": "project_financial_export_payload",
    "project-financial-csv-v1": "project_financial_export_payload",
}


class ExportReadError(ValueError):
    """An export violates a declared byte, tree, shape or identity rule."""


class _SchemaEvolution(Protocol):
    """The consumed specialization of the pinned validator's keyword API."""

    def evolve(self, *, schema: dict[str, Any]) -> Validator: ...


class _Inapplicable:
    pass


INAPPLICABLE = _Inapplicable()
type Cell = Value | _Inapplicable


def _utf8(data: bytes) -> str:
    if len(data) > MAX_BYTES or data.startswith(b"\xef\xbb\xbf"):
        raise ExportReadError("stream bound or BOM")
    try:
        return data.decode("utf-8")
    except UnicodeDecodeError as error:
        raise ExportReadError("invalid UTF-8") from error


def _uint(text: str) -> int:
    if len(text) > 16 or re.fullmatch(r"0|[1-9][0-9]*", text) is None:
        raise ExportReadError("noncanonical unsigned integer")
    value = int(text)
    if value > MAX_INTEGER:
        raise ExportReadError("unsigned integer bound")
    return value


def _bad_number(text: str) -> NoReturn:
    raise ExportReadError("unsupported JSON number")


def _pairs(pairs: list[tuple[str, Value]]) -> dict[str, Value]:
    result: dict[str, Value] = {}
    for key, value in pairs:
        if key in result:
            raise ExportReadError("duplicate JSON key")
        result[key] = value
    return result


def _string(value: str) -> None:
    try:
        value.encode("utf-8")
    except UnicodeEncodeError as error:
        raise ExportReadError("invalid Unicode scalar") from error


def _tree_bounds(value: Value) -> None:
    stack = [(value, 1)]
    nodes = 0
    while stack:
        current, depth = stack.pop()
        nodes += 1
        if nodes > MAX_NODES:
            raise ExportReadError("node bound")
        if isinstance(current, dict | list):
            if depth > MAX_DEPTH:
                raise ExportReadError("container depth bound")
            if isinstance(current, dict):
                for key in current:
                    _string(key)
                children = current.values()
            else:
                children = current
            stack.extend((child, depth + 1) for child in children)
        elif isinstance(current, str):
            _string(current)
        elif current is None or isinstance(current, bool):
            continue
        elif isinstance(current, int) and 0 <= current <= MAX_INTEGER:
            continue
        else:
            raise ExportReadError("unsupported tree value")


def canonical_bytes(value: Value) -> bytes:
    """Conformance serialization for the reviewed unsigned-integer JCS subset."""
    _tree_bounds(value)

    def ordered(item: Value) -> Value:
        if isinstance(item, dict):
            return {
                key: ordered(item[key]) for key in sorted(item, key=lambda k: k.encode("utf-16-be"))
            }
        if isinstance(item, list):
            return [ordered(child) for child in item]
        return item

    data = json.dumps(
        ordered(value), ensure_ascii=False, separators=(",", ":"), allow_nan=False
    ).encode("utf-8")
    if len(data) > MAX_BYTES:
        raise ExportReadError("canonical stream bound")
    return data


def read_json_tree(data: bytes) -> Value:
    """Read exact canonical bounded bytes, retaining all null and container values."""
    text = _utf8(data)
    try:
        value = cast(
            "Value",
            json.loads(
                text,
                object_pairs_hook=_pairs,
                parse_int=_uint,
                parse_float=_bad_number,
                parse_constant=_bad_number,
            ),
        )
    except json.JSONDecodeError as error:
        raise ExportReadError("invalid JSON") from error
    except RecursionError as error:
        raise ExportReadError("container depth bound") from error
    if canonical_bytes(value) != data:
        raise ExportReadError("noncanonical JSON bytes")
    return value


def _forbidden(code: int) -> bool:
    return code < 32 or 127 <= code <= 159 or code in (0x2028, 0x2029)


def read_cell(text: str) -> Cell:
    """Decode the exact shared typed-cell grammar once, without normalization."""
    _string(text)
    if (
        len(text.encode("utf-16-be")) // 2 > MAX_CELL_UNITS
        or len(text.encode("utf-8")) > MAX_CELL_BYTES
    ):
        raise ExportReadError("cell bound")
    if text == "text:x:":
        return INAPPLICABLE
    if text == "text:n:":
        return None
    if text in ("text:b:true", "text:b:false"):
        return text == "text:b:true"
    if text.startswith("text:i:"):
        return _uint(text[7:])
    if not text.startswith("text:s:"):
        raise ExportReadError("invalid typed cell")
    source = text[7:]
    result: list[str] = []
    index = 0
    while index < len(source):
        scalar = source[index]
        if scalar != "\\":
            if _forbidden(ord(scalar)):
                raise ExportReadError("raw forbidden control")
            result.append(scalar)
            index += 1
            continue
        index += 1
        if index == len(source):
            raise ExportReadError("dangling escape")
        if source[index] == "\\":
            result.append("\\")
            index += 1
            continue
        digits = source[index + 1 : index + 5]
        if source[index] != "u" or re.fullmatch(r"[0-9A-F]{4}", digits) is None:
            raise ExportReadError("invalid string escape")
        code = int(digits, 16)
        if not _forbidden(code):
            raise ExportReadError("noncanonical string escape")
        result.append(chr(code))
        index += 5
    return "".join(result)


def _csv_rows(data: bytes, header: Sequence[str]) -> list[list[str]]:
    text = _utf8(data)
    if not data.endswith(b"\r\n"):
        raise ExportReadError("missing final CRLF")
    rows: list[list[str]] = []
    offset = 0
    try:
        for row in csv.reader(io.StringIO(text, newline=""), strict=True):
            if len(row) != len(header):
                raise ExportReadError("CSV width")
            framed = (
                ",".join('"' + cell.replace('"', '""') + '"' for cell in row) + "\r\n"
            ).encode("utf-8")
            if data[offset : offset + len(framed)] != framed:
                raise ExportReadError("noncanonical CSV framing")
            offset += len(framed)
            rows.append(row)
    except csv.Error as error:
        raise ExportReadError("invalid CSV") from error
    if offset != len(data) or not rows or rows.pop(0) != list(header):
        raise ExportReadError("CSV header")
    return rows


def _pointer(key: str) -> str:
    return key.replace("~", "~0").replace("/", "~1")


def _fields(rows: list[tuple[Cell, Cell, Cell]]) -> Value:
    if len(rows) > MAX_NODES:
        raise ExportReadError("node bound")
    index = 0

    def node(pointer: str, depth: int) -> Value:
        nonlocal index
        if index >= len(rows):
            raise ExportReadError("missing node")
        path, kind, value = rows[index]
        index += 1
        if path != pointer:
            raise ExportReadError("noncanonical field pointer or order")
        if kind in ("object", "array"):
            if depth > MAX_DEPTH:
                raise ExportReadError("container depth bound")
            if type(value) is not int or value < 0:
                raise ExportReadError("container count type")
            count = value
            if count > MAX_NODES or count > len(rows) - index:
                raise ExportReadError("container count")
            if kind == "array":
                return [node(pointer + "/" + str(n), depth + 1) for n in range(count)]
            result: dict[str, Value] = {}
            keys: list[str] = []
            for _ in range(count):
                if index >= len(rows):
                    raise ExportReadError("missing field node")
                next_path = rows[index][0]
                if not isinstance(next_path, str) or not next_path.startswith(pointer + "/"):
                    raise ExportReadError("missing field parent")
                segment = next_path[len(pointer) + 1 :]
                if "/" in segment or re.search(r"~(?![01])", segment):
                    raise ExportReadError("invalid pointer escape")
                key = segment.replace("~1", "/").replace("~0", "~")
                if _pointer(key) != segment or key in result:
                    raise ExportReadError("duplicate field or noncanonical pointer")
                keys.append(key)
                result[key] = node(pointer + "/" + segment, depth + 1)
            if keys != sorted(keys, key=lambda k: k.encode("utf-16-be")):
                raise ExportReadError("noncanonical object order")
            return result
        if kind == "null" and value is None:
            return None
        if kind == "boolean" and type(value) is bool:
            return value
        if kind == "integer" and type(value) is int:
            return value
        if kind == "string" and isinstance(value, str):
            return value
        raise ExportReadError("field scalar type")

    value = node("", 1)
    if index != len(rows):
        raise ExportReadError("extra field nodes")
    _tree_bounds(value)
    return value


def read_field_tree(data: bytes) -> Value:
    """Read the encoding-only three-column tree convention used by byte vectors."""
    rows = _csv_rows(data, ["path", "value_type", "value"])
    if len(rows) > MAX_NODES:
        raise ExportReadError("node bound")
    return _fields([(read_cell(row[0]), read_cell(row[1]), read_cell(row[2])) for row in rows])


def _shape(value: dict[str, Any], target: str) -> None:
    entry = CONTRACTS / "public/v1/project-workflow.schema.json"
    with SnapshotStore(CONTRACTS) as store:
        loader = LocalRegistryLoader(store, str(entry))
        regex = RegexImplementation(RegexVariantName.default)
        base = loader.get_validator(str(entry), {}, FormatOptions(regex_impl=regex), regex, False)
        validator = cast("_SchemaEvolution", base).evolve(
            schema={"$ref": entry.as_uri() + "#/$defs/" + target}
        )
        if not validator.is_valid(value):
            raise ExportReadError("value violates owning contract")


def _financial_overviews(payload: dict[str, Any], format_name: str) -> Iterator[dict[str, Any]]:
    report = payload["report"]["reference"]

    def row(**assigned: Any) -> dict[str, Any]:
        assigned.update(report_id=report["report_id"], report_sha256=report["content_sha256"])
        return {key: assigned.get(key, INAPPLICABLE) for key in FINANCIAL_HEADER}

    yield row(row_kind="report", value_type="string", value=format_name)
    for item in payload["records"]:
        reference, content = item["selected"]["reference"], item["revision"]["content"]
        yield row(
            row_kind="record",
            record_id=reference["expense_id"],
            content_revision=reference["version"]["value"],
            content_sha256=reference["content_sha256"],
            observed_control_revision=item["selected"]["observed_control_revision"]["value"],
            record_kind=content["kind"],
            currency=content["currency"],
            gross=content["gross"],
            expense_date=content["expense_date"],
            merchant=content["merchant"],
            business_purpose=content["business_purpose"],
            reconciliation_state=content["reconciliation"]["state"],
        )
    manifest = payload["report"]["manifest"]["financial_manifest"]
    for total in sorted(manifest["totals"], key=lambda value: value["currency"]):
        for bucket in (
            "gross",
            "subtotal",
            "tax",
            "gratuity",
            "service_charges",
            "discount",
            "business_allocation",
        ):
            yield row(
                row_kind="summary", currency=total["currency"], bucket=bucket, **total[bucket]
            )
    for exclusion in manifest["exclusions"]:
        reference = exclusion["reference"]
        yield row(
            row_kind="exclusion",
            record_id=reference["expense_id"],
            content_revision=reference["version"]["value"],
            content_sha256=reference["content_sha256"],
            observed_control_revision=exclusion["observed_control_revision"]["value"],
            exclusion_reason=exclusion["reason"],
        )


def read_export(data: bytes, format_name: str) -> dict[str, Any]:
    """Reconstruct and structurally validate one complete advertised export payload."""
    if format_name not in FORMATS:
        raise ExportReadError("unknown export format")
    if "-json-" in format_name:
        value = read_json_tree(data)
        if not isinstance(value, dict):
            raise ExportReadError("payload must be an object")
        payload = cast("dict[str, Any]", value)
        _shape(payload, FORMATS[format_name])
        return payload
    financial = format_name == "project-financial-csv-v1"
    header = FINANCIAL_HEADER if financial else PROJECT_HEADER
    rows = [dict(zip(header, map(read_cell, row), strict=True)) for row in _csv_rows(data, header)]
    fields = [row for row in rows if row["row_kind"] == "field"]
    value = _fields([(row["path"], row["value_type"], row["value"]) for row in fields])
    if not isinstance(value, dict):
        raise ExportReadError("payload must be an object")
    payload = cast("dict[str, Any]", value)
    _shape(payload, FORMATS[format_name])
    if financial:
        expected = list(_financial_overviews(payload, format_name))
        report = payload["report"]["reference"]
        for row in fields:
            expected.append(
                {
                    key: row[key]
                    if key in ("row_kind", "path", "value_type", "value")
                    else report["report_id"]
                    if key == "report_id"
                    else report["content_sha256"]
                    if key == "report_sha256"
                    else INAPPLICABLE
                    for key in header
                }
            )
        if len(rows) != len(expected) or any(
            any(
                type(row[key]) is not type(wanted[key]) or row[key] != wanted[key] for key in header
            )
            for row, wanted in zip(rows, expected, strict=True)
        ):
            raise ExportReadError("financial identities, overview projections or order")
    else:
        project = payload["project"]
        identity = {
            "organization_id": project["workspace"]["organization_id"],
            "domain_id": project["workspace"]["domain_id"],
            "project_id": project["project_id"],
            "content_revision": project["content_revision"]["value"],
            "snapshot_id": project["snapshot_id"],
        }
        first = {
            **identity,
            "row_kind": "project",
            "path": INAPPLICABLE,
            "value_type": "string",
            "value": format_name,
        }
        if not rows or rows[0] != first or rows[1:] != fields:
            raise ExportReadError("project profile or row order")
        if any(row[key] != expected for row in rows for key, expected in identity.items()):
            raise ExportReadError("project snapshot identity")
    return payload


def verify_artifact(artifact: dict[str, Any], data: bytes) -> dict[str, Any]:
    """Compare complete artifact identity and bytes; no admission/authority assertion."""
    _tree_bounds(artifact)
    _shape(artifact, "project_export_artifact_resource")
    core, reference, control = (
        artifact["core"],
        artifact["reference"],
        artifact["control"]["subject"],
    )
    for key in ("organization_id", "domain_id", "export_id"):
        if core[key] != reference[key] or core[key] != control[key]:
            raise ExportReadError("artifact scope identity")
    if (
        any(core["profile"][key] != core[key] for key in ("organization_id", "domain_id"))
        or artifact["control"]["last_event"]["organization_id"] != core["organization_id"]
    ):
        raise ExportReadError("artifact profile or control scope")
    if hashlib.sha256(canonical_bytes(core)).hexdigest() != reference["artifact_record_sha256"]:
        raise ExportReadError("artifact core digest")
    if (
        artifact["availability"] != "available"
        or artifact["observed_at"] < core["sealed_at"]
        or artifact["observed_at"] >= core["expires_at"]
    ):
        raise ExportReadError("artifact unavailable")
    if (
        len(data) != core["emitted"]["byte_length"]
        or hashlib.sha256(data).hexdigest() != core["emitted"]["sha256"]
    ):
        raise ExportReadError("emitted byte identity")
    payload = read_export(data, core["format"])
    if hashlib.sha256(canonical_bytes(payload)).hexdigest() != core["payload_sha256"]:
        raise ExportReadError("payload digest")
    subject = core["subject"]
    if subject["kind"] == "project":
        project = payload["project"]
        if (
            any(project["workspace"][key] != core[key] for key in ("organization_id", "domain_id"))
            or project["project_id"] != subject["project"]["project_id"]
            or project["content_revision"] != subject["project"]["content_revision"]
            or project["snapshot_id"] != subject["snapshot_id"]
            or project["control_revision"] != subject["observed_control_revision"]
        ):
            raise ExportReadError("project payload subject")
    elif (
        any(subject["report"][key] != core[key] for key in ("organization_id", "domain_id"))
        or payload["report"]["reference"] != subject["report"]
    ):
        raise ExportReadError("financial payload subject")
    return payload
