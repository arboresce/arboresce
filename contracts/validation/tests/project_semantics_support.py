"""Load independently authored literal project conformance inputs."""

import csv
import hashlib
import io
import json
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


def validate_target(value: dict[str, Any], target: str) -> None:
    """Validate actual complete inputs before semantic conformance comparisons."""
    contracts = Path(__file__).resolve().parents[2]
    entry = contracts / "public/v1/project-workflow.schema.json"
    with SnapshotStore(contracts) as store:
        loader = LocalRegistryLoader(store, str(entry))
        base = validator_for(loader, value)
        validator = cast("SchemaEvolution", base).evolve(
            schema={"$ref": entry.as_uri() + "#/$defs/" + target}
        )
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


def case(identity: str) -> dict[str, Any]:
    manifest = Path(__file__).resolve().parents[1] / "project-workflow-v1.cases.json"
    cases = json.loads(manifest.read_text(encoding="utf-8"))["cases"]
    return cast(
        "dict[str, Any]", next(value["value"] for value in cases if value["id"] == identity)
    )


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
