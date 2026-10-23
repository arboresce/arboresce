"""Bounded fixed-target JCS records, without stage or authority admission."""

from __future__ import annotations

import hashlib
import json
from collections.abc import Buffer
from dataclasses import dataclass
from enum import Enum, StrEnum
from functools import cache
from io import BytesIO
from math import gcd
from pathlib import Path
from typing import Any, NoReturn, Protocol, cast

import rfc8785
from arboresce_contract_validation.cli import LocalRegistryLoader, SnapshotStore
from check_jsonschema.formats import FormatOptions
from check_jsonschema.regex_variants import RegexImplementation, RegexVariantName
from jsonschema.protocols import Validator

MAX_RECORD_BYTES = 1_048_576
MAX_JSON_DEPTH = 64
FINANCIAL_PREIMAGE_BYTES = 131_072
type Row = dict[str, Any]
CONTRACTS = Path(__file__).resolve().parents[3]


class RecordCodec(StrEnum):
    """Program-selected complete targets; no caller-selected schema URI."""

    SOURCE_INPUT = "SourceInput"
    PROCESSING_PROFILE = "ProcessingProfile"
    OPERATION_PLAN = "OperationPlan"
    PRIOR_STAGE_RESULT = "PriorStageResult"
    OPERATION_EXECUTION = "OperationExecution"
    COST_QUOTE = "CostQuoteRecord"
    PRIOR_EXPOSURES = "PriorExposureLedger"
    REPRESENTATION = "RepresentationRecord"
    OBSERVATION = "ObservationRecord"
    PROJECT_CHANGE = "ProjectChangeItemRecord"
    CANDIDATE = "CandidateProposalRecord"
    EXPENSE = "ExpenseProposalRecord"
    SUCCESS = "SuccessfulStage"
    FAILURE = "FailedStage"


_TARGETS = {
    RecordCodec.SOURCE_INPUT: ("internal/v1/intelligence.schema.json", "source_input"),
    RecordCodec.PROCESSING_PROFILE: (
        "internal/v1/intelligence.schema.json",
        "processing_profile_record",
    ),
    RecordCodec.OPERATION_PLAN: ("public/v1/lifecycle.schema.json", "execution_plan_record"),
    RecordCodec.PRIOR_STAGE_RESULT: ("internal/v1/intelligence.schema.json", "stage_result_record"),
    RecordCodec.OPERATION_EXECUTION: ("public/v1/lifecycle.schema.json", "operation_execution"),
    RecordCodec.COST_QUOTE: ("internal/v1/intelligence.schema.json", "cost_quote"),
    RecordCodec.PRIOR_EXPOSURES: ("internal/v1/intelligence.schema.json", "prior_exposure_ledger"),
    RecordCodec.REPRESENTATION: ("public/v1/acquisition.schema.json", "representation"),
    RecordCodec.OBSERVATION: ("public/v1/observation.schema.json", "observation"),
    RecordCodec.PROJECT_CHANGE: ("public/v1/project-workflow.schema.json", "project_change_item"),
    RecordCodec.CANDIDATE: ("internal/v1/intelligence.schema.json", "candidate_proposal"),
    RecordCodec.EXPENSE: ("internal/v1/intelligence.schema.json", "expense_proposal"),
    RecordCodec.SUCCESS: ("internal/v1/intelligence.schema.json", "stage_result_record"),
    RecordCodec.FAILURE: ("internal/v1/intelligence.schema.json", "stage_failure"),
}


class RecordFailure(Enum):
    """Static refusal, with no input-bearing exception or partial record."""

    RECORD = "invalid intelligence record"


@dataclass(frozen=True, slots=True)
class RecordLimits:
    """Mandatory exact integer configuration within the transport ceilings."""

    max_record_bytes: int
    max_json_depth: int

    def __post_init__(self) -> None:
        for value, ceiling in (
            (self.max_record_bytes, MAX_RECORD_BYTES),
            (self.max_json_depth, MAX_JSON_DEPTH),
        ):
            if type(value) is not int or not 1 <= value <= ceiling:
                raise ValueError("invalid intelligence record limits")


@dataclass(frozen=True, slots=True, repr=False)
class DecodedRecord:
    """Immutable read result; no complete stage or authority claim."""

    codec: RecordCodec
    canonical_jcs: bytes
    sha256: str

    def __repr__(self) -> str:
        return "DecodedRecord(<redacted>)"


class _Reject(Exception):
    """Private refusal control flow."""


class _Evolving(Protocol):
    def evolve(self, *, schema: Row) -> Validator: ...


class _Output(BytesIO):
    def __init__(self, limit: int) -> None:
        super().__init__()
        self.limit = limit

    def write(self, data: Buffer, /) -> int:
        if self.tell() + memoryview(data).nbytes > self.limit:
            raise _Reject
        return super().write(data)


def _canonical(value: Any, limit: int) -> bytes:
    with _Output(limit) as sink:
        rfc8785.dump(value, sink)
        return sink.getvalue()


def _digest(value: Any, limit: int) -> str:
    return hashlib.sha256(_canonical(value, limit)).hexdigest()


def _depth(data: bytes, limit: int) -> None:
    depth, quoted, escaped = 0, False, False
    for byte in data:
        if quoted:
            if escaped:
                escaped = False
            elif byte == 92:
                escaped = True
            elif byte == 34:
                quoted = False
        elif byte == 34:
            quoted = True
        elif byte in (91, 123):
            depth += 1
            if depth > limit:
                raise _Reject
        elif byte in (93, 125):
            depth -= 1
            if depth < 0:
                raise _Reject
    if quoted or depth:
        raise _Reject


def _pairs(pairs: list[tuple[str, Any]]) -> Row:
    result: Row = {}
    for key, value in pairs:
        if key in result:
            raise _Reject
        result[key] = value
    return result


def _constant(_value: str) -> NoReturn:
    raise _Reject


@cache
def _registry() -> _Evolving:
    entry = CONTRACTS / "internal/v1/intelligence.schema.json"
    with SnapshotStore(CONTRACTS) as store:
        loader = LocalRegistryLoader(store, str(entry))
        regex = RegexImplementation(RegexVariantName.default)
        return cast(
            "_Evolving",
            loader.get_validator(
                loader.entry_loader.entry, {}, FormatOptions(regex_impl=regex), regex, False
            ),
        )


@cache
def _validator(codec: RecordCodec) -> Validator:
    path, target = _TARGETS[codec]
    return _registry().evolve(schema={"$ref": (CONTRACTS / path).as_uri() + "#/$defs/" + target})


def _fraction(value: Row) -> None:
    if gcd(int(value["numerator"]), int(value["denominator"])) != 1:
        raise _Reject


def _profile(value: Row, data: bytes) -> None:
    core, reference = value["core"], value["reference"]
    expected = {
        "organization_id": core["scope"]["organization_id"],
        "domain_id": core["scope"]["domain_id"],
        "profile_id": core["profile_id"],
        "version": core["version"],
        "content_sha256": _digest(core, MAX_RECORD_BYTES),
    }
    if reference != expected or core["codec_profiles"] != ["intelligence-jcs-v1"]:
        raise _Reject
    limits = core["limits"]
    wrapper_bytes = len(data) + 1 + max(1, (len(data).bit_length() + 6) // 7)
    if len(data) > limits["max_record_bytes"] or wrapper_bytes > limits["max_message_bytes"]:
        raise _Reject
    _depth(data, limits["max_json_depth"])
    _fraction(core["maximum_reserved_usd"])


def _result(value: Row) -> None:
    core = value["core"]
    expected = {
        "organization_id": core["scope"]["organization_id"],
        "domain_id": core["scope"]["domain_id"],
        "operation_id": core["operation"]["operation_id"],
        "stage_id": core["stage_id"],
        "stage_result_id": core["stage_result_id"],
        "input_sha256": core["input_sha256"],
        "profile_sha256": core["profile_sha256"],
        "result_sha256": _digest(core, MAX_RECORD_BYTES),
        "kind": core["output_kind"],
    }
    if value["reference"] != expected or any(
        core["operation"][key] != core["scope"][key] for key in ("organization_id", "domain_id")
    ):
        raise _Reject


def _evidence_key(value: Row) -> tuple[str, str]:
    return value["organization_id"], value["observation_id"]


def _candidate(value: Row) -> Any:
    content = dict(value["content"])
    # The fixed closed schema rejects duplicate references within each role.
    content["evidence"] = {
        role: sorted(entries, key=_evidence_key) for role, entries in content["evidence"].items()
    }
    return {
        "contract_version": "v1",
        "kind": "candidate_content",
        "context": value["reference"]["context"],
        "content": content,
    }


def _expense(value: Row) -> Any:
    content = dict(value["content"])
    bindings = content["field_attribution"]
    if len({row["target"] for row in bindings}) != len(bindings):
        raise _Reject
    # Basis uniqueness is enforced by the complete closed owning schema.
    content["field_attribution"] = [
        {**row, "basis": sorted(row["basis"], key=_evidence_key)}
        for row in sorted(bindings, key=lambda row: row["target"])
    ]
    return {
        "kind": "arboresce.financial-content.v1",
        "organization_id": value["reference"]["organization_id"],
        "domain_id": value["reference"]["domain_id"],
        "content": content,
    }


def _own_identity(codec: RecordCodec, value: Row, data: bytes) -> None:
    if codec is RecordCodec.PROCESSING_PROFILE:
        _profile(value, data)
    elif codec is RecordCodec.OPERATION_PLAN:
        if value["plan_sha256"] != _digest(value["core"], MAX_RECORD_BYTES):
            raise _Reject
    elif codec in (RecordCodec.PRIOR_STAGE_RESULT, RecordCodec.SUCCESS):
        _result(value)
    elif codec in (RecordCodec.CANDIDATE, RecordCodec.EXPENSE):
        if value["reference"]["version"] != {"kind": "content", "value": "1"}:
            raise _Reject
        preimage = _candidate(value) if codec is RecordCodec.CANDIDATE else _expense(value)
        bound = MAX_RECORD_BYTES if codec is RecordCodec.CANDIDATE else FINANCIAL_PREIMAGE_BYTES
        if value["reference"]["content_sha256"] != _digest(preimage, bound):
            raise _Reject
    elif codec is RecordCodec.PROJECT_CHANGE:
        keys = (
            "item_id",
            "operation",
            "record_kind",
            "workspace",
            "project",
            "origin",
            "target",
            "content",
            "changed_fields",
            "statement",
            "scope",
            "qualifications",
            "evidence",
            "field_lineage",
            "unresolved",
            "dependencies",
        )
        preimage = {
            "kind": "arboresce.project-change-content.v1",
            **{key: value[key] for key in keys},
        }
        if value["candidate"]["content_sha256"] != _digest(preimage, MAX_RECORD_BYTES):
            raise _Reject
    elif codec is RecordCodec.COST_QUOTE:
        for key in ("input_price_usd", "output_price_usd", "request_price_usd"):
            _fraction(value[key])
        if value["valid_from"] >= value["expires_at"]:
            raise _Reject


def decode_record(
    codec: RecordCodec, data: object, limits: RecordLimits
) -> DecodedRecord | RecordFailure:
    """Admit exact immutable bytes for one program-selected complete record.

    Successful reads prove only this codec's declared finite scope. Full stage
    joins, selected combined limits, resources and authority remain required.
    """
    if type(codec) is not RecordCodec or type(limits) is not RecordLimits:
        return RecordFailure.RECORD
    if type(data) is not bytes or len(data) > limits.max_record_bytes:
        return RecordFailure.RECORD
    try:
        _depth(data, limits.max_json_depth)
        value = json.loads(data.decode("utf-8"), object_pairs_hook=_pairs, parse_constant=_constant)
        if _canonical(value, limits.max_record_bytes) != data or not _validator(codec).is_valid(
            value
        ):
            raise _Reject
        _own_identity(codec, value, data)
        return DecodedRecord(codec, data, hashlib.sha256(data).hexdigest())
    except _Reject, ValueError, UnicodeError, RecursionError:
        return RecordFailure.RECORD
