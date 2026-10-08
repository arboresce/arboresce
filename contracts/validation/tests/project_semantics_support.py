"""Load independently authored literal project conformance inputs."""

import copy
import csv
import hashlib
import io
import json
from functools import cache
from pathlib import Path
from typing import Any, Protocol, cast

from jsonschema.protocols import Validator
from support import validator_for

from arboresce_contract_validation.cli import LocalRegistryLoader, SnapshotStore

BYTE_FIXTURES = Path(__file__).resolve().parent / "fixtures/project-export-bytes"


class SchemaEvolution(Protocol):
    """The consumed specialization of the prepared validator keyword API."""

    def evolve(self, *, schema: dict[str, Any]) -> Validator:
        """Specialize the loaded validator for one explicit owning target."""
        ...


@cache
def owning_validator() -> SchemaEvolution:
    """Prepare one real immutable registry snapshot for this test process."""
    contracts = Path(__file__).resolve().parents[2]
    entry = contracts / "public/v1/project-workflow.schema.json"
    with SnapshotStore(contracts) as store:
        loader = LocalRegistryLoader(store, str(entry))
        return cast("SchemaEvolution", validator_for(loader, {}))


def validate_target(value: dict[str, Any], target: str) -> None:
    """Validate actual complete inputs before semantic conformance comparisons."""
    entry = Path(__file__).resolve().parents[2] / "public/v1/project-workflow.schema.json"
    validator = owning_validator().evolve(schema={"$ref": entry.as_uri() + "#/$defs/" + target})
    if not validator.is_valid(value):
        raise ValueError("value violates owning contract")


def byte_fixture(name: str) -> bytes:
    """Read a literal independently reviewed byte input, never regenerate it."""
    return (BYTE_FIXTURES / name).read_bytes()


def byte_vector(format_name: str) -> tuple[dict[str, Any], bytes, dict[str, Any]]:
    inventory = json.loads((BYTE_FIXTURES.parent / "project-export-byte-vectors.json").read_text())
    vector = next(item for item in inventory["vectors"] if item["id"] == format_name)
    return (
        json.loads(byte_fixture(vector["artifact"])),
        byte_fixture(vector["emitted"]),
        json.loads(byte_fixture(vector["payload"])),
    )


def csv_rows(data: bytes) -> list[list[str]]:
    return list(csv.reader(io.StringIO(data.decode("utf-8"), newline=""), strict=True))


def csv_mutation(rows: list[list[str]]) -> bytes:
    """Frame deliberately altered negative inputs; not a positive export oracle."""
    stream = io.StringIO(newline="")
    csv.writer(stream, quoting=csv.QUOTE_ALL, lineterminator="\r\n").writerows(rows)
    return stream.getvalue().encode("utf-8")


def rehash_ascii_core(artifact: dict[str, Any]) -> None:
    """Rehash ASCII-only negative cores independently of the reader under test."""
    core = json.dumps(artifact["core"], sort_keys=True, separators=(",", ":"))
    assert core.isascii()
    artifact["reference"]["artifact_record_sha256"] = hashlib.sha256(core.encode()).hexdigest()


def rehash_plan_negative(response: dict[str, Any], operation: dict[str, Any]) -> None:
    """Rehash a closed ASCII-only negative plan without creating a positive oracle."""
    plan = response["result"]["plan"]
    core = json.dumps(plan["core"], sort_keys=True, separators=(",", ":"))
    assert core.isascii()
    digest = hashlib.sha256(core.encode()).hexdigest()
    plan["plan_sha256"] = digest
    operation["execution"]["plan"]["plan_sha256"] = digest


def rehash_stage_result_negative(operation: dict[str, Any]) -> None:
    """Rehash deliberately inconsistent ASCII retained output, never positive gold."""
    output = operation["execution"]["stage_results"][0]
    encoded = json.dumps(output, sort_keys=True, separators=(",", ":"))
    assert encoded.isascii()
    operation["execution"]["stages"][0]["accepted_result"]["result_sha256"] = hashlib.sha256(
        encoded.encode()
    ).hexdigest()


def rehash_plan_input_negative(response: dict[str, Any], operation: dict[str, Any]) -> None:
    """Keep negative input/plan digests coherent without generating positive gold."""
    encoded = json.dumps(operation["input"], sort_keys=True, separators=(",", ":"))
    assert encoded.isascii()
    core = response["result"]["plan"]["core"]
    core["input_sha256"] = hashlib.sha256(encoded.encode()).hexdigest()
    core["stages"][0]["input_sha256"] = core["input_sha256"]
    rehash_plan_negative(response, operation)


def case(identity: str) -> dict[str, Any]:
    manifest = Path(__file__).resolve().parents[1] / "project-workflow-v1.cases.json"
    cases = json.loads(manifest.read_text(encoding="utf-8"))["cases"]
    return cast(
        "dict[str, Any]", next(value["value"] for value in cases if value["id"] == identity)
    )


def expense_history(state: str) -> dict[str, Any]:
    """Load an independently reviewed retained confirmation snapshot."""
    path = BYTE_FIXTURES.parent / "project-readback-expense-history.json"
    values = json.loads(path.read_bytes())
    return cast("dict[str, Any]", values["snapshots"][state])


def report_history(identity: str) -> dict[str, Any]:
    """Load a complete independently reviewed retained report identity."""
    path = BYTE_FIXTURES.parent / "project-report-history-composition.json"
    values = json.loads(path.read_bytes())
    return cast("dict[str, Any]", values["reports"][identity])


def rehash_report_negative(response: dict[str, Any]) -> None:
    """Keep deliberately inconsistent report hashes coherent, never positive gold."""
    report = response["result"]["report"]
    preimage = {"kind": "arboresce.project-financial-report.v1", "manifest": report["manifest"]}
    encoded = json.dumps(preimage, ensure_ascii=False, sort_keys=True, separators=(",", ":"))
    assert encoded.isascii()
    report["reference"]["content_sha256"] = hashlib.sha256(encoded.encode()).hexdigest()


def prior_identity() -> dict[str, Any]:
    """Supply a literal prior identity without deriving it from the result."""
    return {
        "principal_id": "33333333-3333-4333-8333-333333333333",
        "kind": "human",
        "identity_profile": {
            "profile": "arboresce.nostr-identity.v1",
            "version": "nip01-v1",
            "public_key": "79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798",
            "standard_revisions": {
                "nip01": {"standard": "nip-01", "revision": "nip-01/2026-10-02"}
            },
        },
        "state": "active",
        "control_revision": {"kind": "control", "value": "1"},
        "created_at": "2026-09-30T12:00:00.000Z",
        "revoked_at": None,
    }


def binding_inputs() -> tuple[dict[str, Any], dict[str, Any], str]:
    """Supply selected scope and source identity independently of wire results."""
    workspace = {
        "organization_id": "11111111-1111-4111-8111-111111111111",
        "domain_id": "22222222-2222-4222-9222-222222222222",
    }
    capture = {
        "organization_id": "11111111-1111-4111-8111-111111111111",
        "domain_id": "22222222-2222-4222-9222-222222222222",
        "capture_id": "22222222-2222-4222-9222-222222222222",
        "version": {"kind": "content", "value": "1"},
    }
    return workspace, capture, "33333333-3333-4333-8333-333333333333"


def export_inputs(
    financial: bool = False,
) -> tuple[dict[str, Any], dict[str, Any], dict[str, Any]]:
    """Load independent literal admission, artifact and payload inputs."""
    prefix = "project-report" if financial else "project"
    payload = "project-financial" if financial else "project"
    return (
        case(prefix + "-export-request-mvp-valid"),
        case(prefix + "-export-response-mvp-valid"),
        case(payload + "-export-payload-complete"),
    )


def transport_inputs(format_name: str) -> tuple[dict[str, Any], ...]:
    """Load complete literal wire values and separate declared admission inputs."""
    targets = (
        "project_export_artifact_get_request",
        "project_export_artifact_get_response",
        "project_export_content_request",
        "project_export_delivery_receipt",
        "project_export_content_response",
        "project_export_delivery_profile",
        "project_export_delivery_admission_inputs",
        "project_export_delivery_get_request",
        "project_export_delivery_get_response",
    )
    return tuple(case("transport-" + format_name + "-" + target) for target in targets)


def operation_inputs(format_name: str) -> tuple[dict[str, Any], ...]:
    """Load independently approved complete operation, source and profile inputs."""
    prefix = "async-" + format_name + "-"
    targets = (
        "command-request",
        "project_export_operation_plan_record",
        "project_export_operation_profile",
        "project_export_operation_admission_inputs",
    )
    return (
        *(case(prefix + target) for target in targets),
        transport_inputs(format_name)[1]["result"]["artifact"],
    )


def operation_arguments() -> list[dict[str, Any]]:
    """Load complete reviewed arguments without generating any expected result."""
    request, plan, profile, admission, artifact = operation_inputs("project-json-v1")
    operation = case("async-project-json-v1-succeeded-project_export_operation_resource")
    return [request, operation, plan, profile, admission, artifact]


def rehash_cancellation_negative(response: dict[str, Any]) -> None:
    """Coherently alter an ASCII negative core, independently of compared code."""
    record = response["result"]["cancellation"]
    data = json.dumps(record["core"], sort_keys=True, separators=(",", ":"), ensure_ascii=False)
    assert data.isascii()
    record["cancellation_sha256"] = hashlib.sha256(data.encode()).hexdigest()


def foreign_exposure() -> dict[str, Any]:
    """Read an approved foreign exposure solely for cross-record negative mutations."""
    manifest = Path(__file__).resolve().parents[1] / "lifecycle-v1.cases.json"
    rows = json.loads(manifest.read_text(encoding="utf-8"))["cases"]
    return cast(
        "dict[str, Any]",
        next(
            row["value"]
            for row in rows
            if row["id"] == "public-exposure-entry-completed-settled-not-required"
        ),
    )


def same_workspace_exposure_negative(
    operation: dict[str, Any], admission: dict[str, Any]
) -> dict[str, Any]:
    """Re-scope a reviewed foreign input solely for negative ledger comparisons."""
    exposure = foreign_exposure()
    worker = operation["execution"]["attempts"][0]
    for name in ("profile", "policy"):
        for key in ("organization_id", "domain_id"):
            exposure[name][key] = admission["context"][key]
    exposure.update(
        stage_id=operation["execution"]["stages"][0]["stage_id"],
        attempt_id=worker["attempt_id"],
        authority_audit_event_id=worker["execution"]["authority_audit_event_id"],
        admitted_at="2026-10-03T12:05:00.200Z",
        finished_at="2026-10-03T12:05:00.500Z",
    )
    return exposure


def reversed_counter_history_negative(name: str, state: str, *, tied: bool) -> dict[str, Any]:
    """Assemble a negative public history with a probe counter preceding its cause."""
    operation = case("async-" + name + "-" + state + "-project_export_operation_resource")
    exposure = same_workspace_exposure_negative(operation, operation_inputs(name)[3])
    exposure.update(state="uncertain", receipt={"kind": "pending"}, settlement="reserved")
    exposure["reconciliation"] = {
        "state": "pending",
        "attempts_started": 1,
        "last_checked_at": "2026-10-03T12:05:02.500Z",
        "next_attempt_at": "2026-10-03T12:05:03.000Z",
        "deadline": operation["execution"]["timing"]["reconciliation_deadline"],
        "diagnostic": None,
    }
    operation["exposures"]["entries"] = [exposure]
    operation["control"]["reconciliation"] = "pending"
    business = operation["execution"]["attempts"][0]
    business["ordinal"] = 2
    probe = copy.deepcopy(business)
    probe.update(
        attempt_id="ffffffff-ffff-4fff-8fff-ffffffffffff",
        ordinal=1,
        subject={"kind": "exposure_reconciliation", "exposure_id": exposure["exposure_id"]},
        state="failed",
        started_at="2026-10-03T12:05:02.000Z",
        deadline="2026-10-03T12:05:22.000Z",
        finished_at="2026-10-03T12:05:02.500Z",
        failure={"class": "transient", "code": "dependency_unavailable"},
    )
    probe["execution"]["authority_audit_event_id"] = "ffffffff-ffff-4fff-8fff-ffffffffffff"
    if tied:
        business.update(
            started_at="2026-10-03T12:05:00.200Z",
            deadline="2026-10-03T12:05:20.200Z",
            finished_at="2026-10-03T12:05:00.200Z",
        )
        exposure["finished_at"] = "2026-10-03T12:05:00.200Z"
        probe.update(
            started_at="2026-10-03T12:05:00.200Z",
            deadline="2026-10-03T12:05:20.200Z",
            finished_at="2026-10-03T12:05:00.200Z",
        )
        exposure["reconciliation"].update(
            last_checked_at="2026-10-03T12:05:00.200Z",
            next_attempt_at="2026-10-03T12:05:00.300Z",
        )
    operation["execution"]["attempts"] = [probe, business]
    return operation


def counter_history_cancellation_negative(
    name: str, operation: dict[str, Any], *, replayed: bool
) -> tuple[dict[str, Any], dict[str, Any], dict[str, Any]]:
    """Keep a negative cancellation's independent record, fingerprint and clocks coherent."""
    prefix = "async-" + name + "-cancel-already_terminal-"
    request, response, decision = (
        case(prefix + suffix) for suffix in ("request", "response", "decision-inputs")
    )
    revision = operation["control"]["control_revision"]
    request["body"]["expected_control_revision"] = copy.deepcopy(revision)
    core = response["result"]["cancellation"]["core"]
    core.update(
        recorded_at="2026-10-03T12:05:04.000Z",
        expected_control_revision=copy.deepcopy(revision),
        observed_state=operation["state"],
        observed_control_revision=copy.deepcopy(revision),
    )
    for key in ("context", "execution", "recorded_at", "operation", "expected_control_revision"):
        decision[key] = copy.deepcopy(core[key])
    response["result"]["operation"] = copy.deepcopy(operation)
    rehash_cancellation_negative(response)
    preimage = {
        "method": "POST",
        "resource": "/v1/operations/" + operation["operation_id"] + "/cancel",
        "contract_version": "v1",
        "purpose": request["metadata"]["context"]["purpose"],
        "body": request,
    }
    encoded = json.dumps(preimage, sort_keys=True, separators=(",", ":"))
    assert encoded.isascii()
    response["idempotency_receipt"].update(
        request_fingerprint=hashlib.sha256(
            b"arboresce.command.v1" + bytes([0]) + encoded.encode()
        ).hexdigest(),
        expires_at="2026-10-10T12:05:04.000Z",
        tombstone_expires_at="2026-11-02T12:05:04.000Z",
        replayed=replayed,
    )
    return request, response, decision


def rehash_association_negative(association: dict[str, Any]) -> None:
    """Rehash a deliberately changed ASCII association, never a positive oracle."""
    encoded = json.dumps(association["content_preimage"], sort_keys=True, separators=(",", ":"))
    assert encoded.isascii()
    association["association_sha256"] = hashlib.sha256(encoded.encode()).hexdigest()
