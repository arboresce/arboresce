"""Cross-record conformance checks for structurally validated public values.

These helpers compare independently supplied request, identity and read snapshots.
They do not authenticate actors, calculate canonical digests, establish current
authorization, or implement canonical transactions. Callers must first validate
each value against its owning schema. Diagnostics identify checks, never secrets.
"""

from __future__ import annotations

from typing import Any


def _candidate(value: dict[str, Any]) -> tuple[str, str, str]:
    return value["candidate_id"], value["version"]["value"], value["content_sha256"]


def _record(value: dict[str, Any]) -> tuple[str, str, str]:
    return value["kind"], value["record_id"], value["content_revision"]["value"]


def _expense(value: dict[str, Any]) -> tuple[str, str, str, str, str]:
    return (
        value["organization_id"],
        value["domain_id"],
        value["expense_id"],
        value["version"]["value"],
        value["content_sha256"],
    )


def _scope(value: dict[str, Any], workspace: dict[str, Any]) -> bool:
    return all(value[key] == workspace[key] for key in ("organization_id", "domain_id"))


def export_problems(
    request: dict[str, Any], response: dict[str, Any], payload: dict[str, Any]
) -> list[str]:
    """Join independently frozen export input, artifact and complete payload.

    These comparisons do not calculate emitted bytes or canonical hashes, prove
    source-set completeness, or establish current disclosure and durable admission.
    """
    workspace = request["metadata"]["context"]
    body, result = request["body"], response["result"]
    artifact = result["export"]
    core, reference, control = artifact["core"], artifact["reference"], artifact["control"]
    subject = core["subject"]
    problems: list[str] = []
    if result["command"] != request["metadata"]["command"]:
        problems.append("export response names another command")
    if any(not _scope(value, workspace) for value in (core, reference, control["subject"])):
        problems.append("export artifact belongs to another workspace")
    if any(value["export_id"] != core["export_id"] for value in (reference, control["subject"])):
        problems.append("export artifact identities differ")
    if (
        not _scope(core["profile"], workspace)
        or control["last_event"]["organization_id"] != workspace["organization_id"]
    ):
        problems.append("export profile or lifecycle event belongs to another workspace")
    if core["format"] != body["format"]:
        problems.append("export format differs from the admitted request")
    if subject["observed_control_revision"] != body["expected_control_revision"]:
        problems.append("export subject control differs from the admitted request")
    if request["metadata"]["command"] == "project.export":
        project = payload["project"]
        expected = {"project_id": body["project_id"], "content_revision": body["project_revision"]}
        if subject["kind"] != "project" or subject.get("project") != expected:
            problems.append("export subject differs from the selected project revision")
        if payload["profile"] != "project-export-payload-v1":
            problems.append("export payload has another project profile")
        if (
            project["project_id"] != body["project_id"]
            or project["content_revision"] != body["project_revision"]
            or project["control_revision"] != body["expected_control_revision"]
            or subject.get("snapshot_id") != project["snapshot_id"]
        ):
            problems.append("export payload differs from the selected project snapshot")
        problems.extend(
            readback_problems(
                {"result": {"project": project, "pending": payload["pending"]}},
                {key: workspace[key] for key in ("organization_id", "domain_id")},
                expected,
            )
        )
    else:
        report = payload["report"]
        if subject["kind"] != "project_financial_report" or subject.get("report") != body["report"]:
            problems.append("export subject differs from the selected project report")
        if payload["profile"] != "project-financial-export-payload-v1":
            problems.append("export payload has another financial profile")
        if report["reference"] != body["report"] or not _scope(report["reference"], workspace):
            problems.append("export payload names another project report")
        selected = report["manifest"]["financial_manifest"]["selected"]
        if [record["selected"] for record in payload["records"]] != selected:
            problems.append("export records differ from the frozen financial selections")
        for record in payload["records"]:
            if record["revision"]["reference"] != record["selected"]["reference"]:
                problems.append("export financial content names another exact revision")
    if not core["sealed_at"] < core["expires_at"]:
        problems.append("export artifact expiry does not follow sealing")
    if artifact["observed_at"] < core["sealed_at"]:
        problems.append("export availability was observed before sealing")
    state = artifact["availability"]
    expected_state = (
        "purged"
        if control["bytes_state"] == "purged"
        else "expired"
        if artifact["observed_at"] >= core["expires_at"]
        else "unavailable"
        if control["restriction"] != "unrestricted" or control["bytes_state"] != "available"
        else "available"
    )
    # Current physical availability/disclosure may deny bytes despite a retained
    # available control. Such a denial cannot bypass purge or exclusive expiry.
    if state != expected_state and not (expected_state == "available" and state == "unavailable"):
        problems.append("export availability contradicts its observed controls or expiry")
    if core["format"].endswith("json-v1") and core["emitted"]["sha256"] != core["payload_sha256"]:
        problems.append("export JSON emitted and payload digests differ")
    if payload["source_availability"]["observed_at"] > core["sealed_at"]:
        problems.append("export sources were observed after sealing")
    return problems


def revocation_problems(
    request: dict[str, Any], response: dict[str, Any], identity: dict[str, Any]
) -> list[str]:
    """Compare an exact self-revocation command with its prior identity and result."""
    body, result = request["body"], response["result"]
    principal = result["principal"]
    problems: list[str] = []
    if body["principal_id"] != identity["principal_id"]:
        problems.append("revocation request is not for the authenticated principal")
    if principal["principal_id"] != body["principal_id"]:
        problems.append("revocation result principal differs from the admitted request")
    if body["expected_control_revision"] != identity["control_revision"]:
        problems.append("revocation expected control differs from the prior identity")
    if principal["control_revision"] != result["control_revision"]:
        problems.append("revocation inner and result controls differ")
    expected = int(body["expected_control_revision"]["value"])
    if int(result["control_revision"]["value"]) != expected + 1:
        problems.append("revocation result control is not the next revision")
    if principal["state"] != "revoked" or principal["revoked_at"] is None:
        problems.append("revocation result is not a dated revoked identity")
    if identity["state"] != "active":
        problems.append("revocation prior identity is not active")
    for key in ("kind", "identity_profile", "created_at"):
        if principal[key] != identity[key]:
            problems.append(f"revocation changed the original {key}")
    return problems


def binding_problems(
    request: dict[str, Any],
    response: dict[str, Any],
    workspace: dict[str, Any],
    capture: dict[str, Any],
    project_id: str,
) -> list[str]:
    """Compare a project binding with the selected scope and exact source capture.

    The supplied capture is an independently obtained acquisition reference;
    matching its identifiers does not prove current authorization or sealed bytes.
    """
    body = request["body"]
    binding = response["result"]["binding"]
    problems: list[str] = []
    if not _scope(request["metadata"]["context"], workspace):
        problems.append("binding request belongs to another workspace")
    if not _scope(capture, workspace):
        problems.append("binding source capture belongs to another workspace")
    if body["source_context"] != workspace or not _scope(capture, body["source_context"]):
        problems.append("binding source context differs from the selected capture workspace")
    if body["project_id"] != project_id:
        problems.append("binding request names another project")
    if (
        body["capture_id"] != capture["capture_id"]
        or body["capture_revision"] != capture["version"]
    ):
        problems.append("binding request names another capture revision")
    for key in (
        "project_id",
        "capture_id",
        "capture_revision",
        "binding_revision",
        "source_context",
    ):
        if binding[key] != body[key]:
            problems.append(f"binding result changed the admitted {key}")
    return problems


def report_problems(request: dict[str, Any], response: dict[str, Any]) -> list[str]:
    """Check frozen project selection joins, not server arithmetic or human proof."""
    body = request["body"]
    report = response["result"]["report"]
    reference, manifest = report["reference"], report["manifest"]
    financial = manifest["financial_manifest"]
    problems: list[str] = []
    if not _scope(reference, request["metadata"]["context"]) or not _scope(
        financial["context"], reference
    ):
        problems.append("project report belongs to another workspace")
    if (
        reference["project_id"] != body["project_id"]
        or manifest["project"]["project_id"] != body["project_id"]
    ):
        problems.append("project report names another project")
    if manifest["currency"] != body["currency"]:
        problems.append("project report changed the selected currency")
    for key in ("date_policy", "period", "include_undated"):
        if financial[key] != body[key]:
            problems.append(f"project report changed the selected {key}")
    if any(total["currency"] != manifest["currency"] for total in financial["totals"]):
        problems.append("project report totals contain another currency")
    if len(financial["totals"]) > 1:
        problems.append("project report repeats a currency total")
    if sum(total["record_count"] for total in financial["totals"]) != len(financial["selected"]):
        problems.append("project report total counts differ from the frozen selection")
    selections = manifest["selected_associations"]
    if len(selections) != len(financial["selected"]):
        problems.append("project report association selection is incomplete")
    seen: set[tuple[str, str, str, str, str]] = set()
    for index, entry in enumerate(selections):
        selected, association, event = (
            entry["selected"],
            entry["association"],
            entry["review_event"],
        )
        if index >= len(financial["selected"]) or selected != financial["selected"][index]:
            problems.append(
                "project report association selection differs from the financial selection"
            )
        expense = selected["reference"]
        key = _expense(expense)
        if key in seen:
            problems.append("project report repeats an exact selected expense")
        seen.add(key)
        if not _scope(expense, reference) or not _scope(association["workspace"], reference):
            problems.append("project report selection belongs to another workspace")
        if association["project"]["project_id"] != reference["project_id"]:
            problems.append("project report association names another project")
        if association["expense"] != expense:
            problems.append("project report association names another expense revision")
        confirmation = selected["confirmation"]
        if confirmation["reference"] != expense:
            problems.append("project report confirmation names another expense revision")
        if int(confirmation["control_revision"]["value"]) > int(
            selected["observed_control_revision"]["value"]
        ):
            problems.append("project report confirmation follows the observed expense control")
        if (
            not _scope(event, reference)
            or event["expense"] != expense
            or event["expense_version"] != expense["version"]
            or event["content_sha256"] != expense["content_sha256"]
        ):
            problems.append("project report review event names another expense or scope")
        if event["association"] != {
            k: association[k]
            for k in ("association_id", "association_revision", "association_sha256")
        }:
            problems.append("project report review event names another association revision")
        if (
            event["financial_review_event_id"] != confirmation["financial_review_event_id"]
            or event["reviewer"] != confirmation["reviewer"]
            or event["decision"] != "confirm"
            or event["executor_scope"]["executor"] != confirmation["executor"]
            or event["executor_scope"]["delegation_id"] != confirmation["delegation_id"]
        ):
            problems.append("project report review event differs from the recorded confirmation")
        # The schema requires fixed-width canonical UTC millisecond timestamps.
        instant = confirmation["recorded_at"]
        if not event["recorded_at"] <= instant < event["expires_at"]:
            problems.append("project report confirmation falls outside the review event lifetime")
        if event["revoked_at"] is not None and event["revoked_at"] <= instant:
            problems.append("project report review event was revoked before confirmation")
        if instant > financial["selection_cutoff"]:
            problems.append("project report confirmation follows the selection cutoff")
        if association["provenance"]["recorded_at"] > event["recorded_at"]:
            problems.append("project report association follows its financial review")
    excluded: set[tuple[str, str, str, str, str]] = set()
    for entry in manifest["excluded_associations"]:
        identity = entry["identity"]
        expense = identity["expense"]
        key = _expense(expense)
        if key in excluded or key in seen:
            problems.append("project report exclusion repeats or selects an exact expense")
        excluded.add(key)
        if not _scope(identity["workspace"], reference) or not _scope(expense, reference):
            problems.append("project report exclusion belongs to another workspace")
        if identity["project_id"] != reference["project_id"]:
            problems.append("project report excluded association names another project")
        required_state = {
            "proposed": "proposed",
            "unresolved": "proposed",
            "voided": "voided",
            "superseded": "superseded",
        }.get(entry["reason"])
        if required_state is not None and entry["revision_state"] != required_state:
            problems.append("project report exclusion reason differs from its revision state")
    for expense_exclusion in financial["exclusions"]:
        matched = [
            entry
            for entry in manifest["excluded_associations"]
            if entry["identity"]["expense"] == expense_exclusion["reference"]
        ]
        if not matched:
            problems.append("project report financial exclusion has no returned association")
        for entry in matched:
            if (
                entry["observed_expense_control_revision"]
                != expense_exclusion["observed_control_revision"]
            ):
                problems.append("project report exclusion expense controls differ")
            if entry["revision_state"] != expense_exclusion["revision_state"]:
                problems.append("project report exclusion expense states differ")
    return problems


def group_problems(group: dict[str, Any], response_capture: str) -> list[str]:
    """Check group scope/capture joins, uniqueness and continuation identity."""
    problems: list[str] = []
    workspace = group["workspace"]
    if response_capture != group["capture_id"]:
        problems.append("group capture differs from the response capture")
    continuation = group["continuation"]
    if continuation is not None and continuation["snapshot_id"] != group["snapshot_id"]:
        problems.append("group continuation names another snapshot")
    seen: set[tuple[str, str, str]] = set()
    for resource in group["items"]:
        item = resource["item"]
        if item["workspace"] != workspace or not _scope(item["candidate"]["context"], workspace):
            problems.append("group item belongs to another workspace")
        if item["candidate"]["context"]["purpose"] != "project_update":
            problems.append("group candidate has another purpose")
        if item["project"]["project_id"] != group["project"]["project_id"]:
            problems.append("group item belongs to another project")
        capture = item["origin"]["capture"]
        if (
            not _scope(capture, workspace)
            or capture["capture_id"] != group["capture_id"]
            or capture["version"] != group["content_revision"]
        ):
            problems.append("group item names another capture revision")
        candidate = _candidate(item["candidate"])
        if candidate in seen:
            problems.append("group repeats an exact candidate")
        seen.add(candidate)
        review = resource["review"]
        if review is not None:
            if review["candidate"] != item["candidate"]:
                problems.append("group review names another candidate")
            if resource["revision_state"] == "superseded":
                # The retained decision predates the shared supersession mutation.
                if int(review["control_revision"]["value"]) >= int(
                    resource["control_revision"]["value"]
                ):
                    problems.append("group retained review does not precede supersession")
            elif review["control_revision"] != resource["control_revision"]:
                problems.append("group review and current resource controls differ")
            if "mapping" in review:
                if (
                    review["mapping"]["item_id"] != item["item_id"]
                    or review["mapping"]["candidate"] != item["candidate"]
                ):
                    problems.append("group accepted mapping names another item or candidate")
                target = review["mapping"]["target"]
                if (
                    target["kind"] != item["record_kind"]
                    or target["record_id"] != item["target"]["record"]["record_id"]
                ):
                    problems.append("group accepted mapping names another target")
                prior = item["target"].get("expected")
                expected_version = 1 if prior is None else int(prior["value"]) + 1
                if int(target["content_revision"]["value"]) != expected_version:
                    problems.append("group accepted target has an unexpected revision")
    return problems


def readback_problems(
    response: dict[str, Any], workspace: dict[str, Any], project: dict[str, Any]
) -> list[str]:
    """Compare a complete readback with its selected scope and exact history."""
    view = response["result"]["project"]
    problems: list[str] = []
    if view["workspace"] != workspace:
        problems.append("readback belongs to another workspace")
    if (
        view["project_id"] != project["project_id"]
        or view["content_revision"] != project["content_revision"]
    ):
        problems.append("readback belongs to another project revision")
    records: set[tuple[str, str, str]] = set()
    mappings = view["mappings"]
    for record in view["records"]:
        key = record["content"]["kind"], record["record_id"], record["content_revision"]["value"]
        if key in records:
            problems.append("readback repeats an exact record")
        records.add(key)
        mapping = record.get("accepted_mapping")
        if mapping is not None:
            if _record(mapping["target"]) != key:
                problems.append("readback embedded mapping names another record")
            if not _scope(mapping["candidate"]["context"], workspace):
                problems.append("readback embedded mapping belongs to another workspace")
            if mapping["candidate"]["context"]["purpose"] != "project_update":
                problems.append("readback embedded mapping has another purpose")
            if mapping not in mappings:
                problems.append("readback embedded mapping has no returned mapping")
    for mapping in mappings:
        if _record(mapping["target"]) not in records:
            problems.append("readback mapping has no exact returned record")
        if not _scope(mapping["candidate"]["context"], workspace):
            problems.append("readback mapping belongs to another workspace")
        if mapping["candidate"]["context"]["purpose"] != "project_update":
            problems.append("readback mapping has another purpose")
        if not any(review.get("mapping") == mapping for review in view["review_history"]):
            problems.append("readback mapping has no accepted review history")
    reviewed: set[tuple[str, str, str]] = set()
    for review in view["review_history"]:
        reviewed.add(_candidate(review["candidate"]))
        if not _scope(review["candidate"]["context"], workspace):
            problems.append("readback review belongs to another workspace")
        if review["candidate"]["context"]["purpose"] != "project_update":
            problems.append("readback review has another purpose")
        if "mapping" in review:
            if review["candidate"] != review["mapping"]["candidate"]:
                problems.append("readback accepted review and mapping candidates differ")
            if review["mapping"] not in mappings:
                problems.append("readback accepted review has no returned mapping")
    for provenance in view["provenance"]:
        if _record(provenance["record"]) not in records:
            problems.append("readback provenance has no exact returned record")
        if provenance["accepted_revision"] != provenance["record"]["content_revision"]:
            problems.append("readback provenance and record revisions differ")
    expenses: set[tuple[str, str, str, str, str]] = set()
    for expense in view["expenses"]:
        reference = expense["revision"]["reference"]
        if not _scope(reference, workspace):
            problems.append("readback expense belongs to another workspace")
        key = _expense(reference)
        if key in expenses:
            problems.append("readback repeats an exact expense")
        expenses.add(key)
    for association in view["expense_associations"]:
        if (
            association["workspace"] != workspace
            or association["project"]["project_id"] != project["project_id"]
        ):
            problems.append("readback expense association belongs to another scope")
        if _expense(association["expense"]) not in expenses:
            problems.append("readback association has no exact returned expense")
    pending: set[tuple[str, str, str]] = set()
    for resource in response["result"]["pending"]:
        item = resource["item"]
        candidate = _candidate(item["candidate"])
        if candidate in pending:
            problems.append("readback repeats an exact pending candidate")
        pending.add(candidate)
        if item["workspace"] != workspace or item["project"]["project_id"] != project["project_id"]:
            problems.append("readback pending item belongs to another scope")
        if not _scope(item["candidate"]["context"], workspace):
            problems.append("readback pending candidate belongs to another workspace")
        if item["candidate"]["context"]["purpose"] != "project_update":
            problems.append("readback pending candidate has another purpose")
        if resource["revision_state"] != "pending" or resource["review"] is not None:
            problems.append("readback pending item is already reviewed")
        if _candidate(item["candidate"]) in reviewed:
            problems.append("readback pending candidate already has recorded review history")
        target = item["target"]
        if target.get("absent") is True and any(
            record[1] == target["record"]["record_id"] for record in records
        ):
            problems.append("readback pending create target already exists")
    return problems
