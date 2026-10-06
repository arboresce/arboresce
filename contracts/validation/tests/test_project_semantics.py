"""Independent scope, control and history conformance regressions."""

from copy import deepcopy

import pytest
from project_semantics_support import binding_inputs, case, export_inputs, prior_identity

from arboresce_contract_validation.project_semantics import (
    binding_problems,
    export_problems,
    group_problems,
    readback_problems,
    report_problems,
    revocation_problems,
)


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
    assert group_problems(group, group["capture_id"]) == ["group repeats an exact candidate"]


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
    assert report_problems(request, response) == [
        "project report exclusion expense controls differ"
    ]


def test_report_exclusion_expense_state_joins_preserve_separate_reasons():
    request = case("project-report-create-request-mvp-valid")
    response = case("project-report-create-response-mvp-valid")
    manifest = response["result"]["report"]["manifest"]
    manifest["excluded_associations"][0].update(revision_state="confirmed", reason="ineligible")
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
    entry = response["result"]["report"]["manifest"]["selected_associations"][0]
    entry["review_event"].update(state="revoked", revoked_at="2026-09-08T12:01:30.000Z")
    response["result"]["report"]["manifest"]["project"]["content_revision"]["value"] = "2"
    assert report_problems(request, response) == []


def test_report_empty_selection_and_other_exclusion_policy():
    request = case("project-report-create-request-mvp-valid")
    response = case("project-report-create-response-mvp-valid")
    manifest = response["result"]["report"]["manifest"]
    manifest["selected_associations"] = []
    manifest["financial_manifest"]["selected"] = []
    manifest["financial_manifest"]["totals"] = []
    manifest["excluded_associations"][0]["reason"] = "outside_period"
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
        "readback repeats an exact pending candidate"
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
