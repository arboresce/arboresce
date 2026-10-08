"""Independent scope, control and history conformance regressions."""

from copy import deepcopy
from decimal import Decimal

import pytest
from project_semantics_support import (
    binding_inputs,
    byte_vector,
    case,
    expense_history,
    export_inputs,
    prior_identity,
    rehash_association_negative,
    rehash_report_negative,
    report_history,
    transport_inputs,
    validate_target,
)

from arboresce_contract_validation.project_exports import verify_artifact
from arboresce_contract_validation.project_semantics import (
    artifact_read_problems,
    binding_problems,
    delivery_problems,
    delivery_read_problems,
    export_problems,
    group_problems,
    readback_problems,
    report_problems,
    revocation_problems,
)


@pytest.mark.parametrize(
    ("operation", "identity", "symbol", "actions"),
    [
        (
            "project_change.group",
            "project-group-current-read-selectors",
            "project_change_group_request",
            ["project.change.read"],
        ),
        (
            "project_change.group",
            "project-group-exact-read-selectors",
            "project_change_group_request",
            ["project.change.read"],
        ),
        (
            "project_change.group",
            "project-group-continuation-read-selectors",
            "project_change_group_request",
            ["project.change.read"],
        ),
        (
            "project.readback",
            "project-readback-request-mvp-valid",
            "project_readback_request",
            ["project.read"],
        ),
        (
            "project_report.get",
            "project-report-get-request-composed-valid",
            "project_report_get_request",
            ["project.read", "report.read"],
        ),
    ],
)
def test_actual_normalized_read_passes_its_registered_request_symbol(
    operation, identity, symbol, actions
):
    inventory = case("mvp-cli-additional-operation-inventory-mvp-valid")
    validate_target(inventory, "mvp_cli_additional_operation_inventory")
    registration = next(row for row in inventory if row["operation"] == operation)
    assert registration["method"] == "GET"
    assert registration["request_contract"] == "project-workflow." + symbol
    assert registration["required_actions"] == actions
    request = case(identity)
    assert "idempotency_key" not in request["metadata"]
    validate_target(request, registration["request_contract"].split(".", 1)[1])
    with pytest.raises(ValueError, match="value violates owning contract"):
        validate_target(request, registration["success_response"].split(".", 1)[1])


@pytest.mark.parametrize("state", ["confirmed", "superseded", "voided"])
def test_readback_and_export_retain_exact_historical_confirmation(state):
    request, exported, payload = export_inputs()
    payload["project"]["expenses"][0] = expense_history(state)
    response = case("project-readback-response-mvp-valid")
    response["result"].update(project=payload["project"], pending=payload["pending"])
    validate_target(response, "project_readback_response")
    validate_target(payload, "project_export_payload")
    assert (
        readback_problems(
            response,
            payload["project"]["workspace"],
            {key: payload["project"][key] for key in ("project_id", "content_revision")},
        )
        == []
    )
    assert export_problems(request, exported, payload) == []


@pytest.mark.parametrize("state", ["confirmed", "superseded", "voided"])
@pytest.mark.parametrize(
    ("path", "replacement", "diagnostic"),
    [
        (
            ("reference", "expense_id"),
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "readback confirmation names another expense revision",
        ),
        (
            ("reference", "organization_id"),
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "readback confirmation names another expense revision",
        ),
        (
            ("reference", "domain_id"),
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "readback confirmation names another expense revision",
        ),
        (
            ("reference", "version", "value"),
            "2",
            "readback confirmation names another expense revision",
        ),
        (
            ("reference", "content_sha256"),
            "f" * 64,
            "readback confirmation names another expense revision",
        ),
        (
            ("control_revision", "value"),
            "4",
            "readback confirmation follows the observed expense control",
        ),
    ],
)
def test_schema_valid_confirmation_mismatches_fail_readback_and_export(
    state, path, replacement, diagnostic
):
    request, exported, payload = export_inputs()
    payload["project"]["expenses"][0] = expense_history(state)
    target = payload["project"]["expenses"][0]["confirmation"]
    for component in path[:-1]:
        target = target[component]
    target[path[-1]] = replacement
    response = case("project-readback-response-mvp-valid")
    response["result"].update(project=payload["project"], pending=payload["pending"])
    validate_target(response, "project_readback_response")
    validate_target(payload, "project_export_payload")
    assert readback_problems(
        response,
        payload["project"]["workspace"],
        {key: payload["project"][key] for key in ("project_id", "content_revision")},
    ) == [diagnostic]
    assert export_problems(request, exported, payload) == [diagnostic]


@pytest.mark.parametrize(
    ("path", "replacement", "diagnostic"),
    [
        (
            ("organization_id",),
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "readback expense successor belongs to another aggregate",
        ),
        (
            ("domain_id",),
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "readback expense successor belongs to another aggregate",
        ),
        (
            ("expense_id",),
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "readback expense successor belongs to another aggregate",
        ),
        (
            ("version", "value"),
            "3",
            "readback expense successor is not the immediate revision",
        ),
    ],
)
def test_schema_valid_expense_successor_mismatches_fail_readback_and_export(
    path, replacement, diagnostic
):
    request, exported, payload = export_inputs()
    payload["project"]["expenses"][0] = expense_history("superseded")
    target = payload["project"]["expenses"][0]["superseded_by"]
    for component in path[:-1]:
        target = target[component]
    target[path[-1]] = replacement
    response = case("project-readback-response-mvp-valid")
    response["result"].update(project=payload["project"], pending=payload["pending"])
    validate_target(response, "project_readback_response")
    validate_target(payload, "project_export_payload")
    assert readback_problems(
        response,
        payload["project"]["workspace"],
        {key: payload["project"][key] for key in ("project_id", "content_revision")},
    ) == [diagnostic]
    assert export_problems(request, exported, payload) == [diagnostic]


def test_transport_profile_versions_bind_one_immutable_digest():
    identities = {}
    for name in (
        "project-json-v1",
        "project-csv-v1",
        "project-financial-json-v1",
        "project-financial-csv-v1",
    ):
        reference = transport_inputs(name)[1]["result"]["artifact"]["core"]["profile"]
        key = (
            reference["organization_id"],
            reference["domain_id"],
            reference["profile_id"],
            reference["version"]["kind"],
            reference["version"]["value"],
        )
        if key in identities:
            assert identities[key] == reference["content_sha256"]
        identities[key] = reference["content_sha256"]
    assert len(identities) == 2


@pytest.mark.parametrize(
    ("index", "target", "required"),
    [
        (2, "project_export_content_request", "expected_control_revision"),
        (5, "project_export_delivery_profile", "max_staged_bytes_per_operation"),
        (6, "project_export_delivery_admission_inputs", "authority_expires_at"),
    ],
)
def test_transport_structural_guard_rejects_missing_admission_inputs(index, target, required):
    value = transport_inputs("project-json-v1")[index]
    del value[required]
    with pytest.raises(ValueError, match="value violates owning contract"):
        validate_target(value, target)


@pytest.mark.parametrize(
    "name",
    ["project-json-v1", "project-csv-v1", "project-financial-json-v1", "project-financial-csv-v1"],
)
def test_complete_transport_literals_and_actual_bytes(name):
    (
        read_request,
        read_response,
        request,
        receipt,
        response,
        profile,
        admission,
        delivery_request,
        delivery_response,
    ) = transport_inputs(name)
    artifact = read_response["result"]["artifact"]
    for target, value in (
        ("project_export_artifact_get_request", read_request),
        ("project_export_artifact_get_response", read_response),
        ("project_export_content_request", request),
        ("project_export_delivery_receipt", receipt),
        ("project_export_content_response", response),
        ("project_export_delivery_profile", profile),
        ("project_export_delivery_admission_inputs", admission),
        ("project_export_delivery_get_request", delivery_request),
        ("project_export_delivery_get_response", delivery_response),
    ):
        validate_target(value, target)
    assert artifact_read_problems(read_request, read_response) == []
    assert delivery_read_problems(delivery_request, delivery_response, artifact) == []
    assert delivery_problems(request, artifact, receipt, response, profile, admission) == []
    assert verify_artifact(artifact, byte_vector(name)[1]) == byte_vector(name)[2]


@pytest.mark.parametrize(
    ("index", "path", "value", "diagnostic"),
    [
        (
            0,
            ("body", "export_id"),
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "artifact read names another export",
        ),
        (
            0,
            ("metadata", "context", "domain_id"),
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "artifact read belongs to another workspace",
        ),
        (
            1,
            ("result", "artifact", "control", "subject", "export_id"),
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "artifact read identities differ",
        ),
        (
            1,
            ("result", "artifact", "control", "last_event", "organization_id"),
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "artifact read lifecycle event belongs to another organization",
        ),
        (
            1,
            ("result", "artifact", "reference", "artifact_record_sha256"),
            "0" * 64,
            "artifact read core digest differs",
        ),
        (
            1,
            ("result", "artifact", "observed_at"),
            "2026-10-03T12:05:00.000Z",
            "artifact read observation interval is invalid",
        ),
        (
            1,
            ("result", "artifact", "core", "expires_at"),
            "2026-10-03T12:05:00.000Z",
            "artifact read observation interval is invalid",
        ),
        (
            1,
            ("result", "artifact", "availability"),
            "expired",
            "artifact read availability differs",
        ),
    ],
)
def test_artifact_read_cross_record_substitutions(index, path, value, diagnostic):
    values = transport_inputs("project-json-v1")[:2]
    node = values[index]
    for key in path[:-1]:
        node = node[key]
    node[path[-1]] = value
    assert diagnostic in artifact_read_problems(*values)


def test_artifact_read_financial_subject_scope():
    request, response = transport_inputs("project-financial-json-v1")[:2]
    response["result"]["artifact"]["core"]["subject"]["report"]["domain_id"] = (
        "ffffffff-ffff-4fff-8fff-ffffffffffff"
    )
    assert "artifact read subject belongs to another workspace" in artifact_read_problems(
        request, response
    )


@pytest.mark.parametrize(
    ("changes", "availability"),
    [
        (
            {
                "bytes_state": "purged",
                "restriction": "revoked",
                "purge_state": "complete",
                "control_revision": {"kind": "control", "value": "5"},
            },
            "purged",
        ),
        (
            {"restriction": "revoked", "control_revision": {"kind": "control", "value": "2"}},
            "unavailable",
        ),
        (
            {"bytes_state": "unavailable", "control_revision": {"kind": "control", "value": "2"}},
            "unavailable",
        ),
        ({}, "unavailable"),
    ],
)
def test_artifact_read_availability_precedence(changes, availability):
    request, response = transport_inputs("project-json-v1")[:2]
    artifact = response["result"]["artifact"]
    artifact["control"].update(changes)
    artifact["availability"] = availability
    validate_target(response, "project_export_artifact_get_response")
    assert artifact_read_problems(request, response) == []


def test_artifact_read_exclusive_expiry():
    request, response = transport_inputs("project-json-v1")[:2]
    artifact = response["result"]["artifact"]
    artifact["observed_at"] = artifact["core"]["expires_at"]
    artifact["availability"] = "expired"
    validate_target(response, "project_export_artifact_get_response")
    assert artifact_read_problems(request, response) == []


@pytest.mark.parametrize(
    "format_name",
    ["project-json-v1", "project-csv-v1", "project-financial-json-v1", "project-financial-csv-v1"],
)
@pytest.mark.parametrize(
    ("section", "field", "replacement", "diagnostic"),
    [
        ("profile", "profile_id", "ffffffff-ffff-4fff-8fff-ffffffffffff", "profile differs"),
        ("profile", "content_sha256", "0" * 64, "profile differs"),
        ("profile", "domain_id", "ffffffff-ffff-4fff-8fff-ffffffffffff", "profile differs"),
        ("policy", "organization_id", "ffffffff-ffff-4fff-8fff-ffffffffffff", "policy belongs"),
        ("policy", "domain_id", "ffffffff-ffff-4fff-8fff-ffffffffffff", "policy belongs"),
    ],
)
def test_retained_delivery_immutable_profile_and_policy_scopes(
    format_name, section, field, replacement, diagnostic
):
    values = transport_inputs(format_name)
    request, response, artifact = values[7], values[8], values[1]["result"]["artifact"]
    response["result"]["delivery"][section][field] = replacement
    validate_target(response, "project_export_delivery_get_response")
    assert any(
        diagnostic in problem for problem in delivery_read_problems(request, response, artifact)
    )


@pytest.mark.parametrize(
    "format_name",
    ["project-json-v1", "project-csv-v1", "project-financial-json-v1", "project-financial-csv-v1"],
)
@pytest.mark.parametrize(
    ("admitted", "expires", "valid"),
    [
        ("2026-10-03T12:05:01.000Z", "2026-10-03T12:10:01.000Z", True),
        ("2026-10-03T12:05:00.999Z", "2026-10-03T12:05:33.000Z", False),
        ("2026-10-03T12:05:03.000Z", "2026-10-03T12:05:02.000Z", False),
        ("2026-10-03T12:05:03.000Z", "2026-10-03T12:05:03.000Z", False),
        ("2026-10-03T12:10:01.000Z", "2026-10-03T12:10:01.001Z", False),
        ("2026-10-03T12:05:03.000Z", "2026-10-03T12:10:01.001Z", False),
    ],
)
def test_retained_delivery_historical_interval(format_name, admitted, expires, valid):
    values = transport_inputs(format_name)
    request, response, artifact = values[7], values[8], values[1]["result"]["artifact"]
    receipt = response["result"]["delivery"]
    receipt["admitted_at"], receipt["transfer_expires_at"] = admitted, expires
    # Present expiry does not rewrite an earlier admitted immutable receipt.
    artifact["observed_at"] = artifact["core"]["expires_at"]
    artifact["availability"] = "expired"
    validate_target(response, "project_export_delivery_get_response")
    validate_target(artifact, "project_export_artifact_resource")
    problems = delivery_read_problems(request, response, artifact)
    assert ("delivery read historical interval is invalid" not in problems) is valid


@pytest.mark.parametrize("selector", ["organization_id", "domain_id"])
def test_delivery_read_coherent_receipt_cannot_replace_artifact_workspace(selector):
    values = transport_inputs("project-json-v1")
    request, response, artifact = values[7], values[8], values[1]["result"]["artifact"]
    request["metadata"]["context"][selector] = "ffffffff-ffff-4fff-8fff-ffffffffffff"
    response["result"]["delivery"]["context"][selector] = request["metadata"]["context"][selector]
    validate_target(request, "project_export_delivery_get_request")
    validate_target(response, "project_export_delivery_get_response")
    validate_target(artifact, "project_export_artifact_resource")
    assert "delivery read artifact belongs to another workspace" in delivery_read_problems(
        request, response, artifact
    )


@pytest.mark.parametrize(
    ("index", "path", "value", "diagnostic"),
    [
        (
            0,
            ("body", "delivery_id"),
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "delivery read names another delivery",
        ),
        (
            1,
            ("result", "delivery", "context", "domain_id"),
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "delivery read belongs to another workspace",
        ),
        (
            1,
            ("result", "delivery", "artifact", "export_id"),
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "delivery read differs from the exact artifact",
        ),
        (
            1,
            ("result", "delivery", "emitted", "sha256"),
            "0" * 64,
            "delivery read differs from the exact artifact",
        ),
    ],
)
def test_delivery_read_substitutions(index, path, value, diagnostic):
    values = transport_inputs("project-json-v1")
    selected = (values[7], values[8], values[1]["result"]["artifact"])
    node = selected[index]
    for key in path[:-1]:
        node = node[key]
    node[path[-1]] = value
    assert diagnostic in delivery_read_problems(*selected)


@pytest.mark.parametrize(
    ("index", "path", "value", "diagnostic"),
    [
        (
            0,
            ("artifact", "artifact_record_sha256"),
            "0" * 64,
            "delivery names another exact artifact",
        ),
        (0, ("expected_control_revision", "value"), "99", "delivery control precondition differs"),
        (0, ("context", "purpose"), "other", "delivery context differs from admission"),
        (
            2,
            ("context", "domain_id"),
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "delivery context differs from admission",
        ),
        (
            2,
            ("executor", "principal_id"),
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "delivery attribution differs from admission",
        ),
        (2, ("emitted", "byte_length"), 1, "delivery emitted identity differs"),
        (2, ("profile", "content_sha256"), "0" * 64, "delivery selected profile differs"),
        (4, ("transfer_buffer_bytes",), 1, "delivery selected profile differs"),
        (
            2,
            ("policy", "policy_id"),
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "delivery policy differs or belongs to another scope",
        ),
        (
            4,
            ("lifecycle_policy", "organization_id"),
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "delivery policy differs or belongs to another scope",
        ),
        (
            4,
            ("lifecycle_policy", "domain_id"),
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "delivery policy differs or belongs to another scope",
        ),
        (4, ("transfer_idle_ms",), 60001, "delivery selected profile capacities are inconsistent"),
        (
            4,
            ("max_active_transfers_per_organization",),
            9,
            "delivery selected profile capacities are inconsistent",
        ),
        (1, ("availability",), "unavailable", "delivery artifact is unavailable at admission"),
        (
            5,
            ("admitted_at",),
            "2026-10-03T12:05:00.000Z",
            "delivery artifact is unavailable at admission",
        ),
        (
            5,
            ("admitted_at",),
            "2026-10-03T12:10:01.000Z",
            "delivery artifact is unavailable at admission",
        ),
        (5, ("authority_expires_at",), None, "delivery admission deadline is unknown"),
        (5, ("server_deadline",), None, "delivery admission deadline is unknown"),
        (4, ("artifact_lifetime_ms",), 1, "delivery fixed or transfer deadline differs"),
        (
            2,
            ("transfer_expires_at",),
            "2026-10-03T12:05:34.000Z",
            "delivery fixed or transfer deadline differs",
        ),
        (
            5,
            ("authority_expires_at",),
            "2026-10-03T12:05:03.000Z",
            "delivery fixed or transfer deadline differs",
        ),
        (
            4,
            ("artifact_lifetime_ms",),
            9007199254740991,
            "delivery deadline arithmetic is unrepresentable",
        ),
        (3, ("headers", "Content-Length"), "1", "delivery binary headers differ"),
        (
            3,
            ("headers", "Arboresce-Delivery-Id"),
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "delivery binary headers differ",
        ),
    ],
)
def test_delivery_admission_and_binary_response_substitutions(index, path, value, diagnostic):
    values = transport_inputs("project-json-v1")
    selected = (
        values[2],
        values[1]["result"]["artifact"],
        values[3],
        values[4],
        values[5],
        values[6],
    )
    node = selected[index]
    for key in path[:-1]:
        node = node[key]
    node[path[-1]] = value
    assert diagnostic in delivery_problems(*selected)


@pytest.mark.parametrize("financial", [False, True])
def test_export_complete_literals(financial):
    assert export_problems(*export_inputs(financial)) == []


@pytest.mark.parametrize(
    ("financial", "index", "path", "replacement", "diagnostic"),
    [
        (
            False,
            1,
            ("result", "command"),
            "project_report.export",
            "export response names another command",
        ),
        (
            False,
            1,
            ("result", "export", "core", "organization_id"),
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "export artifact belongs to another workspace",
        ),
        (
            False,
            1,
            ("result", "export", "reference", "export_id"),
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "export artifact identities differ",
        ),
        (
            False,
            1,
            ("result", "export", "control", "subject", "export_id"),
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "export artifact identities differ",
        ),
        (
            False,
            1,
            ("result", "export", "core", "profile", "domain_id"),
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "export profile or lifecycle event belongs to another workspace",
        ),
        (
            False,
            1,
            ("result", "export", "control", "last_event", "organization_id"),
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "export profile or lifecycle event belongs to another workspace",
        ),
        (
            False,
            0,
            ("body", "format"),
            "project-csv-v1",
            "export format differs from the admitted request",
        ),
        (
            False,
            0,
            ("body", "expected_control_revision", "value"),
            "3",
            "export subject control differs from the admitted request",
        ),
        (
            False,
            1,
            ("result", "export", "core", "subject", "project", "project_id"),
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "export subject differs from the selected project revision",
        ),
        (
            False,
            2,
            ("profile",),
            "project-financial-export-payload-v1",
            "export payload has another project profile",
        ),
        (
            False,
            2,
            ("project", "project_id"),
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "export payload differs from the selected project snapshot",
        ),
        (
            False,
            2,
            ("project", "content_revision", "value"),
            "2",
            "export payload differs from the selected project snapshot",
        ),
        (
            False,
            2,
            ("project", "control_revision", "value"),
            "3",
            "export payload differs from the selected project snapshot",
        ),
        (
            False,
            1,
            ("result", "export", "core", "subject", "snapshot_id"),
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "export payload differs from the selected project snapshot",
        ),
        (
            False,
            2,
            ("project", "workspace", "domain_id"),
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "readback belongs to another workspace",
        ),
        (
            True,
            1,
            ("result", "export", "core", "subject", "report", "content_sha256"),
            "f" * 64,
            "export subject differs from the selected project report",
        ),
        (
            True,
            2,
            ("profile",),
            "project-export-payload-v1",
            "export payload has another financial profile",
        ),
        (
            True,
            2,
            ("report", "reference", "content_sha256"),
            "f" * 64,
            "export payload names another project report",
        ),
        (True, 2, ("records",), [], "export records differ from the frozen financial selections"),
        (
            True,
            2,
            ("records", 0, "revision", "reference", "version", "value"),
            "2",
            "export financial content names another exact revision",
        ),
        (
            False,
            1,
            ("result", "export", "core", "expires_at"),
            "2026-10-03T12:05:01.000Z",
            "export artifact expiry does not follow sealing",
        ),
        (
            False,
            1,
            ("result", "export", "observed_at"),
            "2026-10-03T12:05:00.000Z",
            "export availability was observed before sealing",
        ),
        (
            False,
            1,
            ("result", "export", "observed_at"),
            "2026-10-03T12:10:01.000Z",
            "export availability contradicts its observed controls or expiry",
        ),
        (
            False,
            1,
            ("result", "export", "core", "emitted", "sha256"),
            "f" * 64,
            "export JSON emitted and payload digests differ",
        ),
        (
            False,
            2,
            ("source_availability", "observed_at"),
            "2026-10-03T12:05:01.001Z",
            "export sources were observed after sealing",
        ),
    ],
)
def test_export_semantic_mismatches(financial, index, path, replacement, diagnostic):
    values = export_inputs(financial)
    target = values[index]
    for member in path[:-1]:
        target = target[member]
    target[path[-1]] = replacement
    assert diagnostic in export_problems(*values)


@pytest.mark.parametrize("availability", ["expired", "unavailable", "purged"])
def test_export_retained_availability_literals(availability):
    request, response, payload = export_inputs()
    response["result"]["export"] = case("project-export-artifact-" + availability)
    assert export_problems(request, response, payload) == []


def test_export_unavailable_bytes_with_retained_available_control():
    request, response, payload = export_inputs()
    response["result"]["export"]["availability"] = "unavailable"
    assert export_problems(request, response, payload) == []


def test_export_expiry_precedes_missing_bytes():
    request, response, payload = export_inputs()
    response["result"]["export"] = case("project-export-artifact-unavailable")
    response["result"]["export"]["observed_at"] = "2026-10-03T12:10:01.000Z"
    assert "export availability contradicts its observed controls or expiry" in export_problems(
        request, response, payload
    )


def test_export_csv_has_distinct_emitted_digest():
    request, response, payload = export_inputs()
    request["body"]["format"] = "project-csv-v1"
    response["result"]["export"]["core"].update(format="project-csv-v1", media_type="text/csv")
    response["result"]["export"]["core"]["emitted"]["sha256"] = "f" * 64
    assert export_problems(request, response, payload) == []


@pytest.mark.parametrize("financial", [False, True])
def test_export_cannot_substitute_other_subject_kind(financial):
    request, response, payload = export_inputs(financial)
    other = export_inputs(not financial)[1]["result"]["export"]["core"]["subject"]
    response["result"]["export"]["core"]["subject"] = other
    diagnostic = (
        "export subject differs from the selected project report"
        if financial
        else "export subject differs from the selected project revision"
    )
    assert diagnostic in export_problems(request, response, payload)


@pytest.mark.parametrize("field", ["reference", "control"])
def test_export_every_identity_must_match_workspace(field):
    request, response, payload = export_inputs()
    identity = response["result"]["export"][field]
    if field == "control":
        identity = identity["subject"]
    identity["domain_id"] = "ffffffff-ffff-4fff-8fff-ffffffffffff"
    assert "export artifact belongs to another workspace" in export_problems(
        request, response, payload
    )


@pytest.mark.parametrize(
    "identity",
    [
        "project-change-group-mvp-valid",
        "project-change-group-continuation-complete",
        "project-change-group-accepted-complete",
        "project-change-group-empty-scoped-page",
    ],
)
def test_group_literals(identity):
    group = case(identity)
    assert group_problems(group, group["capture_id"]) == []


@pytest.mark.parametrize(
    ("path", "replacement", "diagnostic"),
    [
        (
            ("workspace", "organization_id"),
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "group item belongs to another workspace",
        ),
        (
            ("items", 0, "item", "candidate", "context", "domain_id"),
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "group item belongs to another workspace",
        ),
        (
            ("project", "project_id"),
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "group item belongs to another project",
        ),
        (
            ("items", 0, "item", "origin", "capture", "domain_id"),
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "group item names another capture revision",
        ),
        (
            ("capture_id",),
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "group item names another capture revision",
        ),
        (("content_revision", "value"), "2", "group item names another capture revision"),
        (
            ("continuation", "snapshot_id"),
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "group continuation names another snapshot",
        ),
        (
            ("items", 0, "review", "candidate", "content_sha256"),
            "f" * 64,
            "group review names another candidate",
        ),
        (
            ("items", 0, "review", "control_revision", "value"),
            "3",
            "group review and current resource controls differ",
        ),
        (
            ("items", 0, "review", "mapping", "item_id"),
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "group accepted mapping names another item or candidate",
        ),
        (
            ("items", 0, "review", "mapping", "candidate", "content_sha256"),
            "f" * 64,
            "group accepted mapping names another item or candidate",
        ),
    ],
)
def test_group_semantic_mismatches(path, replacement, diagnostic):
    group = case("project-change-group-accepted-complete")
    group["continuation"] = {"snapshot_id": group["snapshot_id"], "cursor": "page_2"}
    target = group
    for component in path[:-1]:
        target = target[component]
    target[path[-1]] = replacement
    assert diagnostic in group_problems(group, group["capture_id"])


def test_group_response_capture_and_duplicate():
    group = case("project-change-group-mvp-valid")
    assert group_problems(group, "other") == ["group capture differs from the response capture"]
    group["items"].append(deepcopy(group["items"][0]))
    assert group_problems(group, group["capture_id"]) == [
        "group repeats an exact candidate",
        "group repeats an exact item revision",
    ]


def test_group_candidate_purpose():
    group = case("project-change-group-mvp-valid")
    group["items"][0]["item"]["candidate"]["context"]["purpose"] = "financial_record"
    assert group_problems(group, group["capture_id"]) == ["group candidate has another purpose"]


@pytest.mark.parametrize(
    ("field", "replacement", "diagnostic"),
    [
        ("kind", "decision", "group accepted mapping names another target"),
        (
            "record_id",
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "group accepted mapping names another target",
        ),
        (
            "content_revision",
            {"kind": "content", "value": "2"},
            "group accepted target has an unexpected revision",
        ),
    ],
)
def test_group_mapping_target(field, replacement, diagnostic):
    group = case("project-change-group-accepted-complete")
    group["items"][0]["review"]["mapping"]["target"][field] = replacement
    assert diagnostic in group_problems(group, group["capture_id"])


def test_group_accepts_exact_revised_target_transition():
    group = case("project-change-group-accepted-complete")
    resource = group["items"][0]
    resource["item"]["operation"] = "revise"
    target = resource["item"]["target"]
    target.pop("absent")
    target["expected"] = {"kind": "content", "value": "1"}
    resource["review"]["mapping"]["target"]["content_revision"]["value"] = "2"
    assert group_problems(group, group["capture_id"]) == []


@pytest.mark.parametrize("review_control", ["3", "4"])
def test_superseded_review_cannot_follow_or_equal_current_control(review_control):
    group = case("project-change-group-mvp-valid")
    group["items"] = [case("project-change-resource-superseded-reviewed")]
    group["items"][0]["review"]["control_revision"]["value"] = review_control
    assert group_problems(group, group["capture_id"]) == [
        "group retained review does not precede supersession"
    ]


def test_superseded_review_keeps_prior_control():
    group = case("project-change-group-mvp-valid")
    group["items"] = [case("project-change-resource-superseded-reviewed")]
    assert group_problems(group, group["capture_id"]) == []


def test_rejected_group_has_no_mapping():
    group = case("project-change-group-mvp-valid")
    group["items"] = [case("project-change-resource-rejected")]
    assert group_problems(group, group["capture_id"]) == []


@pytest.mark.parametrize(
    ("path", "replacement", "diagnostic"),
    [
        (
            ("body", "principal_id"),
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "revocation request is not for the authenticated principal",
        ),
        (
            ("result", "principal", "principal_id"),
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "revocation result principal differs from the admitted request",
        ),
        (
            ("body", "expected_control_revision", "value"),
            "2",
            "revocation expected control differs from the prior identity",
        ),
        (
            ("result", "principal", "control_revision", "value"),
            "1",
            "revocation inner and result controls differ",
        ),
        (
            ("result", "control_revision", "value"),
            "3",
            "revocation result control is not the next revision",
        ),
        (
            ("result", "principal", "state"),
            "active",
            "revocation result is not a dated revoked identity",
        ),
        (
            ("result", "principal", "revoked_at"),
            None,
            "revocation result is not a dated revoked identity",
        ),
        (("identity", "state"), "revoked", "revocation prior identity is not active"),
        (("result", "principal", "kind"), "agent", "revocation changed the original kind"),
        (
            ("result", "principal", "identity_profile", "public_key"),
            "f" * 64,
            "revocation changed the original identity_profile",
        ),
        (
            ("result", "principal", "created_at"),
            "2026-09-29T12:00:00.000Z",
            "revocation changed the original created_at",
        ),
    ],
)
def test_revocation_semantic_mismatches(path, replacement, diagnostic):
    request = case("principal-revoke-request-mvp-valid")
    response = case("principal-revoke-response-mvp-valid")
    identity = prior_identity()
    inputs = {"body": request["body"], "result": response["result"], "identity": identity}
    target = inputs
    for component in path[:-1]:
        target = target[component]
    target[path[-1]] = replacement
    assert diagnostic in revocation_problems(request, response, identity)


def test_revocation_literal():
    request = case("principal-revoke-request-mvp-valid")
    response = case("principal-revoke-response-mvp-valid")
    identity = prior_identity()
    assert revocation_problems(request, response, identity) == []


def test_binding_literal():
    workspace, capture, project_id = binding_inputs()
    assert (
        binding_problems(
            case("project-capture-binding-request-mvp-valid"),
            case("project-capture-binding-response-mvp-valid"),
            workspace,
            capture,
            project_id,
        )
        == []
    )


@pytest.mark.parametrize(
    ("path", "replacement", "diagnostic"),
    [
        (
            ("request", "metadata", "context", "domain_id"),
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "binding request belongs to another workspace",
        ),
        (
            ("capture", "domain_id"),
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "binding source capture belongs to another workspace",
        ),
        (
            ("request", "body", "source_context", "organization_id"),
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "binding source context differs from the selected capture workspace",
        ),
        (
            ("request", "body", "project_id"),
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "binding request names another project",
        ),
        (
            ("request", "body", "capture_id"),
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "binding request names another capture revision",
        ),
        (
            ("capture", "version", "value"),
            "2",
            "binding request names another capture revision",
        ),
        (
            ("response", "result", "binding", "project_id"),
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "binding result changed the admitted project_id",
        ),
        (
            ("response", "result", "binding", "capture_id"),
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "binding result changed the admitted capture_id",
        ),
        (
            ("response", "result", "binding", "capture_revision", "value"),
            "2",
            "binding result changed the admitted capture_revision",
        ),
        (
            ("response", "result", "binding", "binding_revision", "value"),
            "2",
            "binding result changed the admitted binding_revision",
        ),
        (
            ("response", "result", "binding", "source_context", "domain_id"),
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "binding result changed the admitted source_context",
        ),
    ],
)
def test_binding_semantic_mismatches(path, replacement, diagnostic):
    workspace, capture, project_id = binding_inputs()
    request = case("project-capture-binding-request-mvp-valid")
    response = case("project-capture-binding-response-mvp-valid")
    inputs = {"request": request, "response": response, "capture": capture}
    target = inputs
    for component in path[:-1]:
        target = target[component]
    target[path[-1]] = replacement
    assert diagnostic in binding_problems(request, response, workspace, capture, project_id)


def test_report_literal():
    assert (
        report_problems(
            case("project-report-create-request-mvp-valid"),
            case("project-report-create-response-mvp-valid"),
        )
        == []
    )


@pytest.mark.parametrize(
    ("path", "replacement", "diagnostic"),
    [
        (
            ("reference", "domain_id"),
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "project report belongs to another workspace",
        ),
        (
            ("reference", "project_id"),
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "project report names another project",
        ),
        (
            ("manifest", "project", "project_id"),
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "project report names another project",
        ),
        (("manifest", "currency"), "USD", "project report changed the selected currency"),
        (
            ("manifest", "financial_manifest", "context", "domain_id"),
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "project report belongs to another workspace",
        ),
        (
            ("manifest", "financial_manifest", "date_policy", "version", "value"),
            "2",
            "project report changed the selected date_policy",
        ),
        (
            ("manifest", "financial_manifest", "period", "start_date"),
            "2026-09-02",
            "project report changed the selected period",
        ),
        (
            ("manifest", "financial_manifest", "include_undated"),
            False,
            "project report changed the selected include_undated",
        ),
        (
            ("manifest", "financial_manifest", "totals", 0, "currency"),
            "USD",
            "project report totals contain another currency",
        ),
        (
            ("manifest", "financial_manifest", "totals", 0, "record_count"),
            2,
            "project report total counts differ from the frozen selection",
        ),
        (
            ("manifest", "selected_associations", 0, "selected", "reference", "content_sha256"),
            "f" * 64,
            "project report association selection differs from the financial selection",
        ),
        (
            ("manifest", "selected_associations", 0, "association", "workspace", "domain_id"),
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "project report selection belongs to another workspace",
        ),
        (
            ("manifest", "selected_associations", 0, "association", "project", "project_id"),
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "project report association names another project",
        ),
        (
            ("manifest", "selected_associations", 0, "association", "expense", "version", "value"),
            "2",
            "project report association names another expense revision",
        ),
        (
            (
                "manifest",
                "selected_associations",
                0,
                "selected",
                "confirmation",
                "reference",
                "version",
                "value",
            ),
            "2",
            "project report confirmation names another expense revision",
        ),
        (
            (
                "manifest",
                "selected_associations",
                0,
                "selected",
                "confirmation",
                "control_revision",
                "value",
            ),
            "3",
            "project report confirmation follows the observed expense control",
        ),
        (
            ("manifest", "selected_associations", 0, "review_event", "domain_id"),
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "project report review event names another expense or scope",
        ),
        (
            ("manifest", "selected_associations", 0, "review_event", "expense", "version", "value"),
            "2",
            "project report review event names another expense or scope",
        ),
        (
            ("manifest", "selected_associations", 0, "review_event", "expense_version", "value"),
            "2",
            "project report review event names another expense or scope",
        ),
        (
            ("manifest", "selected_associations", 0, "review_event", "content_sha256"),
            "f" * 64,
            "project report review event names another expense or scope",
        ),
        (
            (
                "manifest",
                "selected_associations",
                0,
                "review_event",
                "association",
                "association_revision",
                "value",
            ),
            "2",
            "project report review event names another association revision",
        ),
        (
            ("manifest", "selected_associations", 0, "review_event", "financial_review_event_id"),
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "project report review event differs from the recorded confirmation",
        ),
        (
            ("manifest", "selected_associations", 0, "review_event", "reviewer", "principal_id"),
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "project report review event differs from the recorded confirmation",
        ),
        (
            ("manifest", "selected_associations", 0, "review_event", "decision"),
            "void",
            "project report review event differs from the recorded confirmation",
        ),
        (
            (
                "manifest",
                "selected_associations",
                0,
                "review_event",
                "executor_scope",
                "executor",
                "principal_id",
            ),
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "project report review event differs from the recorded confirmation",
        ),
        (
            (
                "manifest",
                "selected_associations",
                0,
                "review_event",
                "executor_scope",
                "delegation_id",
            ),
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "project report review event differs from the recorded confirmation",
        ),
        (
            ("manifest", "selected_associations", 0, "review_event", "recorded_at"),
            "2026-09-08T12:01:30.000Z",
            "project report confirmation falls outside the review event lifetime",
        ),
        (
            ("manifest", "selected_associations", 0, "review_event", "expires_at"),
            "2026-09-08T12:01:00.000Z",
            "project report confirmation falls outside the review event lifetime",
        ),
        (
            ("manifest", "selected_associations", 0, "review_event", "revoked_at"),
            "2026-09-08T12:00:40.000Z",
            "project report review event was revoked before confirmation",
        ),
        (
            ("manifest", "financial_manifest", "selection_cutoff"),
            "2026-09-08T12:00:59.000Z",
            "project report confirmation follows the selection cutoff",
        ),
        (
            ("manifest", "selected_associations", 0, "association", "provenance", "recorded_at"),
            "2026-09-08T12:00:40.000Z",
            "project report association follows its financial review",
        ),
        (
            ("manifest", "excluded_associations", 0, "identity", "workspace", "domain_id"),
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "project report exclusion belongs to another workspace",
        ),
        (
            ("manifest", "excluded_associations", 0, "identity", "expense", "domain_id"),
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "project report exclusion belongs to another workspace",
        ),
        (
            ("manifest", "excluded_associations", 0, "identity", "project_id"),
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "project report excluded association names another project",
        ),
        (
            ("manifest", "excluded_associations", 0, "revision_state"),
            "confirmed",
            "project report exclusion reason differs from its revision state",
        ),
    ],
)
def test_report_semantic_mismatches(path, replacement, diagnostic):
    request = case("project-report-create-request-mvp-valid")
    response = case("project-report-create-response-mvp-valid")
    target = response["result"]["report"]
    for component in path[:-1]:
        target = target[component]
    target[path[-1]] = replacement
    if path[-1] == "revoked_at":
        response["result"]["report"]["manifest"]["selected_associations"][0]["review_event"][
            "state"
        ] = "revoked"
    assert diagnostic in report_problems(request, response)


def test_report_selection_completeness_and_duplicates():
    request = case("project-report-create-request-mvp-valid")
    response = case("project-report-create-response-mvp-valid")
    manifest = response["result"]["report"]["manifest"]
    entry = deepcopy(manifest["selected_associations"][0])
    manifest["selected_associations"].append(entry)
    assert "project report association selection is incomplete" in report_problems(
        request, response
    )
    assert (
        "project report association selection differs from the financial selection"
        in report_problems(request, response)
    )
    entry["selected"]["observed_control_revision"]["value"] = "3"
    manifest["financial_manifest"]["selected"].append(deepcopy(entry["selected"]))
    manifest["financial_manifest"]["totals"][0]["record_count"] = 2
    assert "project report repeats an exact selected expense" in report_problems(request, response)
    manifest["excluded_associations"].append(deepcopy(manifest["excluded_associations"][0]))
    assert "project report exclusion repeats or selects an exact expense" in report_problems(
        request, response
    )


def test_report_excluded_expense_cannot_also_be_selected():
    request = case("project-report-create-request-mvp-valid")
    response = case("project-report-create-response-mvp-valid")
    manifest = response["result"]["report"]["manifest"]
    manifest["excluded_associations"][0]["identity"]["expense"] = deepcopy(
        manifest["financial_manifest"]["selected"][0]["reference"]
    )
    assert "project report exclusion repeats or selects an exact expense" in report_problems(
        request, response
    )


def test_report_financial_exclusions_need_associations():
    request = case("project-report-create-request-mvp-valid")
    response = case("project-report-create-response-mvp-valid")
    response["result"]["report"]["manifest"]["excluded_associations"] = []
    assert "project report financial exclusion has no returned association" in report_problems(
        request, response
    )


def test_report_exclusion_expense_control_joins():
    request = case("project-report-create-request-mvp-valid")
    response = case("project-report-create-response-mvp-valid")
    response["result"]["report"]["manifest"]["excluded_associations"][0][
        "observed_expense_control_revision"
    ]["value"] = "2"
    rehash_report_negative(response)
    assert report_problems(request, response) == [
        "project report exclusion expense controls differ"
    ]


def test_report_exclusion_expense_state_joins_preserve_separate_reasons():
    request = case("project-report-create-request-mvp-valid")
    response = case("project-report-create-response-mvp-valid")
    manifest = response["result"]["report"]["manifest"]
    manifest["excluded_associations"][0].update(revision_state="confirmed", reason="ineligible")
    rehash_report_negative(response)
    assert report_problems(request, response) == ["project report exclusion expense states differ"]


def test_report_totals_have_one_currency_bucket():
    request = case("project-report-create-request-mvp-valid")
    response = case("project-report-create-response-mvp-valid")
    totals = response["result"]["report"]["manifest"]["financial_manifest"]["totals"]
    extra = deepcopy(totals[0])
    extra["gross"]["known_net"] = "51.00"
    totals.append(extra)
    assert "project report repeats a currency total" in report_problems(request, response)


def test_report_retains_confirmation_after_later_event_revocation():
    request = case("project-report-create-request-mvp-valid")
    response = case("project-report-create-response-mvp-valid")
    response["result"]["report"] = report_history("retained_revocation")
    validate_target(response, "project_report_create_response")
    assert report_problems(request, response) == []


def test_report_empty_selection_and_other_exclusion_policy():
    request = case("project-report-create-request-mvp-valid")
    response = case("project-report-create-response-mvp-valid")
    response["result"]["report"] = report_history("empty_selection")
    validate_target(response, "project_report_create_response")
    assert report_problems(request, response) == []


def test_readback_literal():
    response = case("project-readback-response-mvp-valid")
    view = response["result"]["project"]
    project = {"project_id": view["project_id"], "content_revision": view["content_revision"]}
    assert readback_problems(response, view["workspace"], project) == []


@pytest.mark.parametrize(
    ("path", "replacement", "diagnostic"),
    [
        (
            ("workspace", "domain_id"),
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "readback belongs to another workspace",
        ),
        (
            ("project_id",),
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "readback belongs to another project revision",
        ),
        (("content_revision", "value"), "2", "readback belongs to another project revision"),
        (
            ("mappings", 0, "target", "content_revision", "value"),
            "2",
            "readback mapping has no exact returned record",
        ),
        (
            ("mappings", 0, "candidate", "context", "domain_id"),
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "readback mapping belongs to another workspace",
        ),
        (
            ("review_history", 0, "candidate", "context", "domain_id"),
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "readback review belongs to another workspace",
        ),
        (
            ("review_history", 0, "mapping", "item_id"),
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "readback accepted review has no returned mapping",
        ),
        (
            ("provenance", 0, "record", "record_id"),
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "readback provenance has no exact returned record",
        ),
        (
            ("provenance", 0, "accepted_revision", "value"),
            "2",
            "readback provenance and record revisions differ",
        ),
        (
            ("expenses", 0, "revision", "reference", "organization_id"),
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "readback expense belongs to another workspace",
        ),
        (
            ("expense_associations", 0, "workspace", "domain_id"),
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "readback expense association belongs to another scope",
        ),
        (
            ("expense_associations", 0, "project", "project_id"),
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "readback expense association belongs to another scope",
        ),
        (
            ("expense_associations", 0, "expense", "version", "value"),
            "2",
            "readback association has no exact returned expense",
        ),
        (
            ("expense_associations", 0, "expense", "content_sha256"),
            "f" * 64,
            "readback association has no exact returned expense",
        ),
    ],
)
def test_readback_semantic_mismatches(path, replacement, diagnostic):
    response = case("project-readback-response-mvp-valid")
    view = response["result"]["project"]
    workspace = deepcopy(view["workspace"])
    project = deepcopy(
        {"project_id": view["project_id"], "content_revision": view["content_revision"]}
    )
    target = view
    for component in path[:-1]:
        target = target[component]
    target[path[-1]] = replacement
    assert diagnostic in readback_problems(response, workspace, project)


def test_readback_duplicates():
    response = case("project-readback-response-mvp-valid")
    view = response["result"]["project"]
    view["records"].append(deepcopy(view["records"][0]))
    view["expenses"].append(deepcopy(view["expenses"][0]))
    project = {"project_id": view["project_id"], "content_revision": view["content_revision"]}
    assert readback_problems(response, view["workspace"], project) == [
        "readback repeats an exact record",
        "readback repeats an exact expense",
    ]


@pytest.mark.parametrize(
    ("path", "replacement", "diagnostic"),
    [
        (
            ("candidate", "context", "domain_id"),
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "readback embedded mapping belongs to another workspace",
        ),
        (
            ("candidate", "context", "purpose"),
            "financial_record",
            "readback embedded mapping has another purpose",
        ),
        (
            ("target", "record_id"),
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "readback embedded mapping names another record",
        ),
        (
            ("item_id",),
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "readback embedded mapping has no returned mapping",
        ),
    ],
)
def test_readback_embedded_mapping_mismatches(path, replacement, diagnostic):
    response = case("project-readback-response-mvp-valid")
    view = response["result"]["project"]
    target = view["records"][0]["accepted_mapping"]
    for component in path[:-1]:
        target = target[component]
    target[path[-1]] = replacement
    project = {"project_id": view["project_id"], "content_revision": view["content_revision"]}
    assert diagnostic in readback_problems(response, view["workspace"], project)


def test_readback_review_candidate_matches_mapping():
    response = case("project-readback-response-mvp-valid")
    view = response["result"]["project"]
    view["review_history"][0]["candidate"]["content_sha256"] = "f" * 64
    project = {"project_id": view["project_id"], "content_revision": view["content_revision"]}
    assert readback_problems(response, view["workspace"], project) == [
        "readback accepted review and mapping candidates differ"
    ]


def test_readback_mapping_requires_accepted_history():
    response = case("project-readback-response-mvp-valid")
    view = response["result"]["project"]
    view["review_history"] = []
    project = {"project_id": view["project_id"], "content_revision": view["content_revision"]}
    assert readback_problems(response, view["workspace"], project) == [
        "readback mapping has no accepted review history"
    ]


def test_readback_pending_candidates_are_unique():
    response = case("project-readback-response-mvp-valid")
    view = response["result"]["project"]
    response["result"]["pending"].append(deepcopy(response["result"]["pending"][0]))
    project = {"project_id": view["project_id"], "content_revision": view["content_revision"]}
    assert readback_problems(response, view["workspace"], project) == [
        "readback repeats an exact pending candidate",
        "readback pending repeats an exact item revision",
    ]


def test_readback_records_without_candidate_history():
    response = case("project-readback-response-mvp-valid")
    view = response["result"]["project"]
    view["records"][0].pop("accepted_mapping")
    view["mappings"] = []
    view["review_history"] = []
    project = {"project_id": view["project_id"], "content_revision": view["content_revision"]}
    assert readback_problems(response, view["workspace"], project) == []


@pytest.mark.parametrize("collection", ["mappings", "review_history", "pending"])
def test_readback_candidate_purpose(collection):
    response = case("project-readback-response-mvp-valid")
    view = response["result"]["project"]
    if collection == "pending":
        response["result"]["pending"][0]["item"]["candidate"]["context"]["purpose"] = (
            "financial_record"
        )
        diagnostic = "readback pending candidate has another purpose"
    else:
        view[collection][0]["candidate"]["context"]["purpose"] = "financial_record"
        diagnostic = (
            "readback mapping has another purpose"
            if collection == "mappings"
            else "readback review has another purpose"
        )
    project = {"project_id": view["project_id"], "content_revision": view["content_revision"]}
    assert diagnostic in readback_problems(response, view["workspace"], project)


def test_unrelated_project_head_change_preserves_independent_bases():
    group = case("project-change-group-mvp-valid")
    group["project"]["content_revision"]["value"] = "2"
    assert group_problems(group, group["capture_id"]) == []
    response = case("project-readback-response-mvp-valid")
    view = response["result"]["project"]
    view["content_revision"]["value"] = "2"
    project = {"project_id": view["project_id"], "content_revision": view["content_revision"]}
    assert readback_problems(response, view["workspace"], project) == []


@pytest.mark.parametrize(
    ("path", "replacement", "diagnostic"),
    [
        (
            ("item", "workspace", "domain_id"),
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "readback pending item belongs to another scope",
        ),
        (
            ("item", "project", "project_id"),
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "readback pending item belongs to another scope",
        ),
        (
            ("item", "candidate", "context", "domain_id"),
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "readback pending candidate belongs to another workspace",
        ),
        (("revision_state",), "accepted", "readback pending item is already reviewed"),
        (("review",), {}, "readback pending item is already reviewed"),
        (
            ("item", "target", "record", "record_id"),
            "88888888-8888-4888-8888-888888888888",
            "readback pending create target already exists",
        ),
    ],
)
def test_readback_pending_mismatches(path, replacement, diagnostic):
    response = case("project-readback-response-mvp-valid")
    view = response["result"]["project"]
    target = response["result"]["pending"][0]
    for component in path[:-1]:
        target = target[component]
    target[path[-1]] = replacement
    project = {"project_id": view["project_id"], "content_revision": view["content_revision"]}
    assert diagnostic in readback_problems(response, view["workspace"], project)


def test_readback_cannot_return_accepted_candidate_as_pending():
    response = case("project-readback-response-mvp-valid")
    view = response["result"]["project"]
    pending = response["result"]["pending"][0]
    pending["item"]["candidate"] = deepcopy(view["review_history"][0]["candidate"])
    project = {"project_id": view["project_id"], "content_revision": view["content_revision"]}
    assert readback_problems(response, view["workspace"], project) == [
        "readback pending candidate already has recorded review history"
    ]


def test_empty_readback_and_noncreate_pending():
    response = case("project-readback-response-mvp-valid")
    view = response["result"]["project"]
    for key in (
        "records",
        "mappings",
        "review_history",
        "expenses",
        "expense_associations",
        "provenance",
    ):
        view[key] = []
    item = response["result"]["pending"][0]["item"]
    item["operation"] = "revise"
    item["target"].pop("absent")
    item["target"]["expected"] = {"kind": "content", "value": "1"}
    project = {"project_id": view["project_id"], "content_revision": view["content_revision"]}
    assert readback_problems(response, view["workspace"], project) == []


@pytest.mark.parametrize("role", ["supporting", "refuting", "qualifying"])
def test_group_rejects_foreign_evidence_ownership(role):
    group = case("project-change-group-accepted-complete")
    item = group["items"][0]["item"]
    item["evidence"][role] = [
        {
            "organization_id": "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "observation_id": "22222222-2222-4222-9222-222222222222",
        }
    ]
    validate_target(group, "project_change_group")
    assert "group item evidence belongs to another organization" in group_problems(
        group, group["capture_id"]
    )


def test_group_rejects_self_dependency():
    group = case("project-change-group-accepted-complete")
    item = group["items"][0]["item"]
    item["dependencies"] = [
        {"item_id": item["item_id"], "content_revision": {"kind": "content", "value": "1"}}
    ]
    validate_target(group, "project_change_group")
    assert "group item dependencies contain a cycle" in group_problems(group, group["capture_id"])


@pytest.mark.parametrize("size", [2, 3])
def test_group_rejects_nonself_dependency_cycles(size):
    group = case("project-change-group-mvp-valid")
    resource = group["items"][0]
    group["items"] = [deepcopy(resource) for _ in range(size)]
    for index, entry in enumerate(group["items"]):
        item = entry["item"]
        item["item_id"] = f"63000000-0000-4000-8000-{index + 1:012x}"
        item["candidate"]["candidate_id"] = f"63000000-0000-4000-8000-{index + 11:012x}"
        item["dependencies"] = [
            {
                "item_id": f"63000000-0000-4000-8000-{(index + 1) % size + 1:012x}",
                "content_revision": {"kind": "content", "value": "1"},
            }
        ]
    validate_target(group, "project_change_group")
    assert "group item dependencies contain a cycle" in group_problems(group, group["capture_id"])


@pytest.mark.parametrize(
    ("path", "replacement", "diagnostic"),
    [
        (
            ("candidate_id",),
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "group item successor belongs to another candidate or context",
        ),
        (
            ("context", "organization_id"),
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "group item successor belongs to another candidate or context",
        ),
        (
            ("context", "domain_id"),
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "group item successor belongs to another candidate or context",
        ),
        (
            ("context", "purpose"),
            "project_read",
            "group item successor belongs to another candidate or context",
        ),
        (("version", "value"), "99", "group item successor is not the immediate revision"),
    ],
)
def test_group_rejects_inconsistent_candidate_successor(path, replacement, diagnostic):
    group = case("project-change-group-mvp-valid")
    group["items"] = [case("project-change-resource-superseded-reviewed")]
    target = group["items"][0]["superseded_by"]
    for component in path[:-1]:
        target = target[component]
    target[path[-1]] = replacement
    validate_target(group, "project_change_group")
    assert diagnostic in group_problems(group, group["capture_id"])


def test_group_rejects_two_candidates_for_one_item_revision():
    group = case("project-change-group-mvp-valid")
    duplicate = deepcopy(group["items"][0])
    duplicate["item"]["candidate"]["candidate_id"] = "ffffffff-ffff-4fff-8fff-ffffffffffff"
    group["items"].append(duplicate)
    validate_target(group, "project_change_group")
    assert "group repeats an exact item revision" in group_problems(group, group["capture_id"])


@pytest.mark.parametrize("conflicting", [False, True])
def test_readback_and_export_reject_duplicate_exact_candidate_verdicts(conflicting):
    request, exported, payload = export_inputs()
    history = payload["project"]["review_history"]
    duplicate = deepcopy(history[0])
    duplicate["review_id"] = "ffffffff-ffff-4fff-8fff-ffffffffffff"
    if conflicting:
        duplicate["verdict"] = "rejected"
        duplicate.pop("mapping")
    history.append(duplicate)
    response = case("project-readback-response-mvp-valid")
    response["result"].update(project=payload["project"], pending=payload["pending"])
    validate_target(response, "project_readback_response")
    validate_target(payload, "project_export_payload")
    assert "readback repeats a verdict for an exact candidate" in readback_problems(
        response,
        payload["project"]["workspace"],
        {key: payload["project"][key] for key in ("project_id", "content_revision")},
    )
    assert "readback repeats a verdict for an exact candidate" in export_problems(
        request, exported, payload
    )


@pytest.mark.parametrize("financial", [False, True])
@pytest.mark.parametrize(
    ("path", "replacement", "diagnostic"),
    [
        (
            ("captured_for", "expense_id"),
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "financial attribution names another captured revision",
        ),
        (
            ("captured_for", "organization_id"),
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "financial attribution names another captured revision",
        ),
        (
            ("captured_for", "domain_id"),
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "financial attribution names another captured revision",
        ),
        (
            ("captured_for", "version", "value"),
            "2",
            "financial attribution names another captured revision",
        ),
        (
            ("captured_for", "content_sha256"),
            "f" * 64,
            "financial attribution names another captured revision",
        ),
        (
            ("origin", "source_revision", "organization_id"),
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "financial attribution origin belongs to another aggregate",
        ),
        (
            ("origin", "source_revision", "domain_id"),
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "financial attribution origin belongs to another aggregate",
        ),
        (
            ("origin", "source_revision", "expense_id"),
            "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "financial attribution origin belongs to another aggregate",
        ),
        (
            ("origin", "source_revision", "version", "value"),
            "2",
            "financial attribution origin has an inconsistent revision",
        ),
        (
            ("origin", "source_revision", "content_sha256"),
            "f" * 64,
            "financial attribution origin has an inconsistent revision",
        ),
        (("origin", "target"), "/gross", "financial attribution origin names another field"),
        (
            ("origin", "value_sha256"),
            "f" * 64,
            "financial attribution resolved-value digest differs",
        ),
        (
            ("origin", "recorded_at"),
            "2026-10-03T12:00:00.000Z",
            "financial attribution origin follows the retained revision",
        ),
    ],
)
def test_complete_financial_attribution_mismatches_fail_composed_reads(
    financial, path, replacement, diagnostic
):
    request, exported, payload = export_inputs(financial)
    revision = (
        payload["records"][0]["revision"]
        if financial
        else payload["project"]["expenses"][0]["revision"]
    )
    target = revision["attribution"][0]
    for component in path[:-1]:
        target = target[component]
    target[path[-1]] = replacement
    validate_target(
        payload, "project_financial_export_payload" if financial else "project_export_payload"
    )
    assert diagnostic in export_problems(request, exported, payload)
    if not financial:
        response = case("project-readback-response-mvp-valid")
        response["result"].update(project=payload["project"], pending=payload["pending"])
        validate_target(response, "project_readback_response")
        assert diagnostic in readback_problems(
            response,
            payload["project"]["workspace"],
            {key: payload["project"][key] for key in ("project_id", "content_revision")},
        )


@pytest.mark.parametrize("financial", [False, True])
@pytest.mark.parametrize("mutation", ["missing", "reordered", "origin-alias", "unknown-target"])
def test_complete_financial_field_associations_cannot_be_omitted_or_reassigned(financial, mutation):
    request, exported, payload = export_inputs(financial)
    revision = (
        payload["records"][0]["revision"]
        if financial
        else payload["project"]["expenses"][0]["revision"]
    )
    entries = revision["attribution"]
    if mutation == "missing":
        entries.pop()
        diagnostic = "financial attributions differ from the complete field bindings"
    elif mutation == "reordered":
        entries.reverse()
        diagnostic = "financial attributions differ from the complete field bindings"
    elif mutation == "origin-alias":
        entries[1]["origin"]["association_id"] = entries[0]["origin"]["association_id"]
        diagnostic = "financial attribution repeats an origin association"
    else:
        entries[0]["target"] = "/components/tax_components/15/amount"
        entries[0]["origin"]["target"] = entries[0]["target"]
        diagnostic = "financial attribution target is not an exact atomic unit"
    validate_target(
        payload, "project_financial_export_payload" if financial else "project_export_payload"
    )
    assert diagnostic in export_problems(request, exported, payload)


@pytest.mark.parametrize("financial", [False, True])
@pytest.mark.parametrize(
    "member",
    [
        "organization_id",
        "domain_id",
        "project_id",
        "expense_id",
        "expense_version",
        "expense_content_sha256",
    ],
)
def test_association_preimage_mismatches_fail_every_composition(financial, member):
    request, exported, payload = export_inputs(financial)
    association = (
        payload["report"]["manifest"]["selected_associations"][0]["association"]
        if financial
        else payload["project"]["expense_associations"][0]
    )
    replacement = (
        {"kind": "content", "value": "2"}
        if member == "expense_version"
        else "f" * 64
        if member == "expense_content_sha256"
        else "ffffffff-ffff-4fff-8fff-ffffffffffff"
    )
    association["content_preimage"][member] = replacement
    validate_target(
        payload, "project_financial_export_payload" if financial else "project_export_payload"
    )
    assert "association content preimage differs from its exposed identities" in export_problems(
        request, exported, payload
    )
    if not financial:
        response = case("project-readback-response-mvp-valid")
        response["result"].update(project=payload["project"], pending=payload["pending"])
        validate_target(response, "project_readback_response")
        assert (
            "association content preimage differs from its exposed identities"
            in readback_problems(
                response,
                payload["project"]["workspace"],
                {key: payload["project"][key] for key in ("project_id", "content_revision")},
            )
        )
    report_request = case("project-report-create-request-mvp-valid")
    report_response = case("project-report-create-response-mvp-valid")
    report_response["result"]["report"]["manifest"]["selected_associations"][0]["association"][
        "content_preimage"
    ][member] = replacement
    validate_target(report_response, "project_report_create_response")
    assert "association content preimage differs from its exposed identities" in report_problems(
        report_request, report_response
    )


@pytest.mark.parametrize("financial", [False, True])
def test_association_without_preimage_cannot_qualify_composed_read(financial):
    request, exported, payload = export_inputs(financial)
    association = (
        payload["report"]["manifest"]["selected_associations"][0]["association"]
        if financial
        else payload["project"]["expense_associations"][0]
    )
    del association["content_preimage"]
    validate_target(
        payload, "project_financial_export_payload" if financial else "project_export_payload"
    )
    assert "association has no complete content preimage" in export_problems(
        request, exported, payload
    )


def test_group_candidate_version_cannot_have_two_content_digests():
    group = case("project-change-group-mvp-valid")
    duplicate = deepcopy(group["items"][0])
    duplicate["item"]["item_id"] = "ffffffff-ffff-4fff-8fff-ffffffffffff"
    duplicate["item"]["candidate"]["content_sha256"] = "f" * 64
    group["items"].append(duplicate)
    validate_target(group, "project_change_group")
    assert "group repeats a candidate version with another digest" in group_problems(
        group, group["capture_id"]
    )


@pytest.mark.parametrize("kind", ["verdict", "expense"])
def test_readback_and_export_immutable_version_cannot_hide_behind_digest_alias(kind):
    request, exported, payload = export_inputs()
    view = payload["project"]
    if kind == "verdict":
        duplicate = deepcopy(view["review_history"][0])
        duplicate["review_id"] = "ffffffff-ffff-4fff-8fff-ffffffffffff"
        duplicate["verdict"] = "rejected"
        del duplicate["mapping"]
        duplicate["candidate"]["content_sha256"] = "f" * 64
        view["review_history"].append(duplicate)
        diagnostic = "readback review has contradictory digests for a candidate version"
    else:
        duplicate = deepcopy(view["expenses"][0])
        duplicate["revision"]["reference"]["content_sha256"] = "f" * 64
        view["expenses"].append(duplicate)
        diagnostic = "readback has contradictory digests for an expense revision"
    response = case("project-readback-response-mvp-valid")
    response["result"].update(project=view, pending=payload["pending"])
    validate_target(response, "project_readback_response")
    validate_target(payload, "project_export_payload")
    assert diagnostic in readback_problems(
        response, view["workspace"], {key: view[key] for key in ("project_id", "content_revision")}
    )
    assert diagnostic in export_problems(request, exported, payload)


@pytest.mark.parametrize("role", ["supporting", "refuting", "qualifying"])
def test_pending_readback_and_export_share_evidence_scope_checks(role):
    request, exported, payload = export_inputs()
    payload["pending"][0]["item"]["evidence"][role] = [
        {
            "organization_id": "ffffffff-ffff-4fff-8fff-ffffffffffff",
            "observation_id": "22222222-2222-4222-9222-222222222222",
        }
    ]
    response = case("project-readback-response-mvp-valid")
    response["result"].update(project=payload["project"], pending=payload["pending"])
    validate_target(response, "project_readback_response")
    validate_target(payload, "project_export_payload")
    view = payload["project"]
    diagnostic = "readback pending item evidence belongs to another organization"
    assert diagnostic in readback_problems(response, view["workspace"], view)
    assert diagnostic in export_problems(request, exported, payload)


@pytest.mark.parametrize("size", [1, 2, 3])
def test_pending_readback_and_export_share_returned_dependency_cycle_checks(size):
    request, exported, payload = export_inputs()
    resource = payload["pending"][0]
    payload["pending"] = [deepcopy(resource) for _ in range(size)]
    for index, entry in enumerate(payload["pending"]):
        item = entry["item"]
        item["item_id"] = f"65000000-0000-4000-8000-{index + 1:012x}"
        item["candidate"]["candidate_id"] = f"65000000-0000-4000-8000-{index + 11:012x}"
        item["dependencies"] = [
            {
                "item_id": f"65000000-0000-4000-8000-{(index + 1) % size + 1:012x}",
                "content_revision": deepcopy(item["candidate"]["version"]),
            }
        ]
    response = case("project-readback-response-mvp-valid")
    response["result"].update(project=payload["project"], pending=payload["pending"])
    validate_target(response, "project_readback_response")
    validate_target(payload, "project_export_payload")
    view = payload["project"]
    diagnostic = "readback pending item dependencies contain a cycle"
    assert diagnostic in readback_problems(response, view["workspace"], view)
    assert diagnostic in export_problems(request, exported, payload)


def test_pending_readback_and_export_reject_candidate_version_digest_alias():
    request, exported, payload = export_inputs()
    duplicate = deepcopy(payload["pending"][0])
    duplicate["item"]["candidate"]["content_sha256"] = "f" * 64
    payload["pending"].append(duplicate)
    response = case("project-readback-response-mvp-valid")
    response["result"].update(project=payload["project"], pending=payload["pending"])
    validate_target(response, "project_readback_response")
    validate_target(payload, "project_export_payload")
    view = payload["project"]
    diagnostic = "readback pending repeats a candidate version with another digest"
    assert diagnostic in readback_problems(response, view["workspace"], view)
    assert diagnostic in export_problems(request, exported, payload)


@pytest.mark.parametrize("surface", ["group", "pending", "provenance"])
def test_every_project_lineage_surface_rejects_foreign_observations(surface):
    request, exported, payload = export_inputs()
    group = case("project-change-group-accepted-complete")
    lineage = (
        group["items"][0]["item"]["field_lineage"]
        if surface == "group"
        else payload["pending"][0]["item"]["field_lineage"]
        if surface == "pending"
        else payload["project"]["provenance"][0]["field_lineage"]
    )
    lineage[0]["entries"] = [
        {
            "category": "source_observation",
            "observation": {
                "organization_id": "ffffffff-ffff-4fff-8fff-ffffffffffff",
                "observation_id": "22222222-2222-4222-9222-222222222222",
            },
        }
    ]
    if surface == "group":
        validate_target(group, "project_change_group")
        assert "group item lineage observation belongs to another organization" in group_problems(
            group, group["capture_id"]
        )
    else:
        response = case("project-readback-response-mvp-valid")
        response["result"].update(project=payload["project"], pending=payload["pending"])
        validate_target(response, "project_readback_response")
        validate_target(payload, "project_export_payload")
        diagnostic = (
            "readback pending item lineage observation belongs to another organization"
            if surface == "pending"
            else "readback provenance lineage observation belongs to another organization"
        )
        assert diagnostic in readback_problems(
            response, payload["project"]["workspace"], payload["project"]
        )
        assert diagnostic in export_problems(request, exported, payload)


@pytest.mark.parametrize("surface", ["group", "pending"])
@pytest.mark.parametrize("mutation", ["missing", "duplicate"])
def test_project_item_lineage_covers_changed_fields_once(surface, mutation):
    request, exported, payload = export_inputs()
    group = case("project-change-group-accepted-complete")
    item = group["items"][0]["item"] if surface == "group" else payload["pending"][0]["item"]
    if mutation == "missing":
        item["field_lineage"] = [deepcopy(item["field_lineage"][0])]
        item["field_lineage"][0]["field"] = "scope"
        suffix = "lineage omits a declared changed field"
    else:
        item["field_lineage"].append(deepcopy(item["field_lineage"][0]))
        suffix = "lineage repeats a field"
    if surface == "group":
        validate_target(group, "project_change_group")
        assert "group item " + suffix in group_problems(group, group["capture_id"])
    else:
        response = case("project-readback-response-mvp-valid")
        response["result"].update(project=payload["project"], pending=payload["pending"])
        validate_target(response, "project_readback_response")
        validate_target(payload, "project_export_payload")
        diagnostic = "readback pending item " + suffix
        assert diagnostic in readback_problems(
            response, payload["project"]["workspace"], payload["project"]
        )
        assert diagnostic in export_problems(request, exported, payload)


@pytest.mark.parametrize("conflicting", [False, True])
def test_readback_and_export_reject_repeated_association_revision(conflicting):
    request, exported, payload = export_inputs()
    view = payload["project"]
    duplicate = deepcopy(view["expense_associations"][0])
    if conflicting:
        expense = deepcopy(view["expenses"][0])
        foreign = "ffffffff-ffff-4fff-8fff-ffffffffffff"
        expense["revision"]["reference"]["expense_id"] = foreign
        for attribution in expense["revision"]["attribution"]:
            attribution["captured_for"]["expense_id"] = foreign
            attribution["origin"]["source_revision"]["expense_id"] = foreign
        if expense["confirmation"] is not None:
            expense["confirmation"]["reference"]["expense_id"] = foreign
        view["expenses"].append(expense)
        duplicate["expense"]["expense_id"] = foreign
        duplicate["content_preimage"]["expense_id"] = foreign
        rehash_association_negative(duplicate)
    view["expense_associations"].append(duplicate)
    response = case("project-readback-response-mvp-valid")
    response["result"].update(project=view, pending=payload["pending"])
    validate_target(response, "project_readback_response")
    validate_target(payload, "project_export_payload")
    diagnostic = (
        "readback has contradictory digests for an association revision"
        if conflicting
        else "readback repeats an exact association revision"
    )
    assert readback_problems(response, view["workspace"], view) == [diagnostic]
    assert export_problems(request, exported, payload) == [diagnostic]


@pytest.mark.parametrize("conflicting", [False, True])
def test_report_and_financial_export_reject_repeated_association_revision(conflicting):
    request, exported, payload = export_inputs(True)
    report = payload["report"]
    manifest = report["manifest"]
    financial = manifest["financial_manifest"]
    duplicate = deepcopy(manifest["selected_associations"][0])
    if conflicting:
        foreign = "ffffffff-ffff-4fff-8fff-ffffffffffff"
        selected, association, event = (
            duplicate["selected"],
            duplicate["association"],
            duplicate["review_event"],
        )
        selected["reference"]["expense_id"] = foreign
        selected["confirmation"]["reference"]["expense_id"] = foreign
        selected["confirmation"]["financial_review_event_id"] = foreign
        association["expense"]["expense_id"] = foreign
        association["content_preimage"]["expense_id"] = foreign
        rehash_association_negative(association)
        event["expense"]["expense_id"] = foreign
        event["financial_review_event_id"] = foreign
        event["association"]["association_sha256"] = association["association_sha256"]
        record = deepcopy(payload["records"][0])
        record["selected"] = deepcopy(selected)
        record["revision"]["reference"]["expense_id"] = foreign
        for attribution in record["revision"]["attribution"]:
            attribution["captured_for"]["expense_id"] = foreign
            attribution["origin"]["source_revision"]["expense_id"] = foreign
        payload["records"].append(record)
    manifest["selected_associations"].append(duplicate)
    financial["selected"].append(deepcopy(duplicate["selected"]))
    for total in financial["totals"]:
        total["record_count"] *= 2
        for bucket in [
            "gross",
            "subtotal",
            "tax",
            "gratuity",
            "service_charges",
            "discount",
            "business_allocation",
        ]:
            values = total[bucket]
            values["known_net"] = format(Decimal(values["known_net"]) * 2, ".2f")
            for count in [
                "unknown_record_count",
                "unknown_list_record_count",
                "known_null_component_count",
            ]:
                values[count] *= 2
    rehash_report_negative({"result": {"report": report}})
    exported["result"]["export"]["core"]["subject"]["report"] = deepcopy(report["reference"])
    request["body"]["report"] = deepcopy(report["reference"])
    if not conflicting:
        # Exact duplicate selections are already forbidden by the owning shape.
        with pytest.raises(ValueError, match="value violates owning contract"):
            validate_target(payload, "project_financial_export_payload")
        return
    validate_target(payload, "project_financial_export_payload")
    diagnostic = (
        "project report has contradictory digests for an association revision"
        if conflicting
        else "project report repeats an exact association revision"
    )
    assert diagnostic in export_problems(request, exported, payload)
    creation_request = case("project-report-create-request-mvp-valid")
    creation_response = case("project-report-create-response-mvp-valid")
    creation_response["result"]["report"] = report
    validate_target(creation_response, "project_report_create_response")
    assert diagnostic in report_problems(creation_request, creation_response)


@pytest.mark.parametrize("member", ["organization_id", "domain_id"])
def test_pending_origin_capture_scope_is_shared_by_group_readback_and_export(member):
    request, exported, payload = export_inputs()
    payload["pending"][0]["item"]["origin"]["capture"][member] = (
        "ffffffff-ffff-4fff-8fff-ffffffffffff"
    )
    response = case("project-readback-response-mvp-valid")
    response["result"].update(project=payload["project"], pending=payload["pending"])
    validate_target(response, "project_readback_response")
    validate_target(payload, "project_export_payload")
    view = payload["project"]
    diagnostic = "readback pending item origin capture belongs to another workspace"
    assert readback_problems(response, view["workspace"], view) == [diagnostic]
    assert export_problems(request, exported, payload) == [diagnostic]
    group = case("project-change-group-mvp-valid")
    group["items"][0]["item"]["origin"]["capture"][member] = "ffffffff-ffff-4fff-8fff-ffffffffffff"
    validate_target(group, "project_change_group")
    assert group_problems(group, group["capture_id"]) == [
        "group item names another capture revision"
    ]
