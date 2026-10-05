"""Load independently authored literal project conformance inputs."""

import json
from pathlib import Path
from typing import Any, cast


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
