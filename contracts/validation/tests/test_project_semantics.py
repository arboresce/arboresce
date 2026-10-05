"""Independent scope, control and history conformance regressions."""

from copy import deepcopy

import pytest
from project_semantics_support import case, prior_identity

from arboresce_contract_validation.project_semantics import (
    group_problems,
    readback_problems,
    revocation_problems,
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
