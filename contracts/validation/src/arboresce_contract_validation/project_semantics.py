"""Cross-record conformance checks for structurally validated public values.

These helpers compare independently supplied request, identity and read snapshots.
Export and retained financial checks compare declared core, profile, content,
report and association digests and internal field joins with bounded encoding.
They do not authenticate actors, establish
current authorization, or implement canonical transactions. Callers first validate
each value against its owning schema. Diagnostics identify checks, never secrets.
"""

from __future__ import annotations

import hashlib
from datetime import datetime, timedelta
from typing import Any

from arboresce_contract_validation.project_exports import ExportReadError, canonical_bytes


def artifact_read_problems(request: dict[str, Any], response: dict[str, Any]) -> list[str]:
    """Join structurally validated exact read selectors and complete artifact metadata."""
    artifact = response["result"]["artifact"]
    core, reference, control = artifact["core"], artifact["reference"], artifact["control"]
    workspace = request["metadata"]["context"]
    problems: list[str] = []
    if request["body"]["export_id"] != reference["export_id"]:
        problems.append("artifact read names another export")
    if any(
        not _scope(value, workspace)
        for value in (core, reference, control["subject"], core["profile"])
    ):
        problems.append("artifact read belongs to another workspace")
    if any(value["export_id"] != core["export_id"] for value in (reference, control["subject"])):
        problems.append("artifact read identities differ")
    if control["last_event"]["organization_id"] != workspace["organization_id"]:
        problems.append("artifact read lifecycle event belongs to another organization")
    if core["subject"]["kind"] == "project_financial_report" and not _scope(
        core["subject"]["report"], workspace
    ):
        problems.append("artifact read subject belongs to another workspace")
    if hashlib.sha256(canonical_bytes(core)).hexdigest() != reference["artifact_record_sha256"]:
        problems.append("artifact read core digest differs")
    if not core["sealed_at"] < core["expires_at"] or artifact["observed_at"] < core["sealed_at"]:
        problems.append("artifact read observation interval is invalid")
    expected = (
        {"purged"}
        if control["bytes_state"] == "purged"
        else {"expired"}
        if artifact["observed_at"] >= core["expires_at"]
        else {"unavailable"}
        if control["restriction"] != "unrestricted" or control["bytes_state"] != "available"
        else {"available", "unavailable"}
    )
    if artifact["availability"] not in expected:
        problems.append("artifact read availability differs")
    return problems


def delivery_read_problems(
    request: dict[str, Any], response: dict[str, Any], artifact: dict[str, Any]
) -> list[str]:
    """Compare exact retained delivery lookup; it confers no admission grant."""
    delivery = response["result"]["delivery"]
    problems: list[str] = []
    if delivery["delivery_id"] != request["body"]["delivery_id"]:
        problems.append("delivery read names another delivery")
    if not _scope(delivery["context"], request["metadata"]["context"]):
        problems.append("delivery read belongs to another workspace")
    if any(
        not _scope(value, request["metadata"]["context"])
        for value in (
            artifact["core"],
            artifact["reference"],
            artifact["control"]["subject"],
            artifact["core"]["profile"],
        )
    ):
        problems.append("delivery read artifact belongs to another workspace")
    if (
        delivery["artifact"] != artifact["reference"]
        or delivery["emitted"] != artifact["core"]["emitted"]
    ):
        problems.append("delivery read differs from the exact artifact")
    core = artifact["core"]
    if delivery["profile"] != core["profile"]:
        problems.append("delivery read profile differs from the exact artifact")
    policy = delivery["policy"]
    if policy["organization_id"] != core["organization_id"] or (
        policy["policy_scope"] == "domain" and policy["domain_id"] != core["domain_id"]
    ):
        problems.append("delivery read policy belongs to another scope")
    if (
        not core["sealed_at"]
        <= delivery["admitted_at"]
        < delivery["transfer_expires_at"]
        <= core["expires_at"]
    ):
        problems.append("delivery read historical interval is invalid")
    return problems


def delivery_problems(
    request: dict[str, Any],
    artifact: dict[str, Any],
    delivery: dict[str, Any],
    response: dict[str, Any],
    profile: dict[str, Any],
    admission: dict[str, Any],
) -> list[str]:
    """Compare validated declared admission snapshots, never authenticate a receipt.

    Callers qualify current authority and validate each complete input first.
    Byte reconstruction/identity is independently checked by verify_artifact.
    These static joins implement no live authorization, reservation or streaming.
    """
    core, reference = artifact["core"], artifact["reference"]
    problems: list[str] = []
    if request["artifact"] != reference or delivery["artifact"] != reference:
        problems.append("delivery names another exact artifact")
    if request["expected_control_revision"] != artifact["control"]["control_revision"]:
        problems.append("delivery control precondition differs")
    if (
        request["context"] != admission["context"]
        or delivery["context"] != admission["context"]
        or not _scope(core, admission["context"])
    ):
        problems.append("delivery context differs from admission")
    if any(
        delivery[key] != admission[key]
        for key in ("executor", "delegation_id", "authority_audit_event_id", "admitted_at")
    ):
        problems.append("delivery attribution differs from admission")
    if delivery["emitted"] != core["emitted"]:
        problems.append("delivery emitted identity differs")
    if (
        delivery["profile"] != core["profile"]
        or core["profile"]["content_sha256"] != hashlib.sha256(canonical_bytes(profile)).hexdigest()
    ):
        problems.append("delivery selected profile differs")
    policy = profile["lifecycle_policy"]
    if (
        delivery["policy"] != policy
        or policy["organization_id"] != core["organization_id"]
        or (policy["policy_scope"] == "domain" and policy["domain_id"] != core["domain_id"])
    ):
        problems.append("delivery policy differs or belongs to another scope")
    if (
        profile["transfer_idle_ms"] > profile["transfer_total_ms"]
        or profile["max_active_transfers_per_organization"] > profile["max_active_transfers_global"]
    ):
        problems.append("delivery selected profile capacities are inconsistent")
    if (
        artifact["availability"] != "available"
        or not core["sealed_at"] <= admission["admitted_at"] < core["expires_at"]
    ):
        problems.append("delivery artifact is unavailable at admission")
    authority_expiry, server_deadline = (
        admission["authority_expires_at"],
        admission["server_deadline"],
    )
    if authority_expiry is None or server_deadline is None:
        problems.append("delivery admission deadline is unknown")
    else:

        def after(timestamp: str, duration: int) -> str:
            value = datetime.fromisoformat(timestamp) + timedelta(
                seconds=duration // 1000, milliseconds=duration % 1000
            )
            return value.isoformat(timespec="milliseconds").replace("+00:00", "Z")

        try:
            fixed_expiry = after(core["sealed_at"], profile["artifact_lifetime_ms"])
            transfer_limit = after(admission["admitted_at"], profile["transfer_total_ms"])
            expected = min(core["expires_at"], authority_expiry, server_deadline, transfer_limit)
            if (
                core["expires_at"] != fixed_expiry
                or delivery["transfer_expires_at"] != expected
                or expected <= admission["admitted_at"]
            ):
                problems.append("delivery fixed or transfer deadline differs")
        except OverflowError:
            problems.append("delivery deadline arithmetic is unrepresentable")
    expected_headers = {
        "Content-Type": core["media_type"],
        "Content-Length": str(core["emitted"]["byte_length"]),
        "Arboresce-Export-Id": core["export_id"],
        "Arboresce-Delivery-Id": delivery["delivery_id"],
        "Arboresce-Artifact-Sha256": core["emitted"]["sha256"],
        "Arboresce-Transfer-Expires-At": delivery["transfer_expires_at"],
    }
    if response["headers"] != expected_headers:
        problems.append("delivery binary headers differ")
    return problems


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


def _financial_values(content: dict[str, Any]) -> dict[str, Any]:
    values = {
        "/" + key: content[key]
        for key in (
            "kind",
            "currency",
            "gross",
            "expense_date",
            "source_observed_at",
            "merchant",
            "original",
            "business_purpose",
            "business_allocation",
            "reconciliation",
        )
    }
    components = content["components"]
    values.update(
        {"/components/" + key: components[key] for key in ("subtotal", "gratuity", "discount")}
    )
    for name in ("tax_components", "service_charges"):
        items = components[name]
        if items is None or not items:
            values["/components/" + name] = items
        else:
            for index, item in enumerate(items):
                for key in ("label", "amount"):
                    values[f"/components/{name}/{index}/{key}"] = item[key]
    return values


def _financial_revision_problems(revision: dict[str, Any]) -> list[str]:
    """Compare complete retained field associations, without inventing predecessor truth."""
    reference, content = revision["reference"], revision["content"]
    problems: list[str] = []
    values = _financial_values(content)
    bindings = content["field_attribution"]
    targets = [binding["target"] for binding in bindings]
    if targets != sorted(set(targets)):
        problems.append("financial field bindings are not uniquely ordered")
    if not {key for key, value in values.items() if value is not None}.issubset(targets):
        problems.append("financial field bindings omit a known unit")
    for binding in bindings:
        basis = [(item["organization_id"], item["observation_id"]) for item in binding["basis"]]
        if basis != sorted(set(basis)) or any(
            key[0] != reference["organization_id"] for key in basis
        ):
            problems.append("financial field evidence is foreign or not uniquely ordered")
    original = content["original"]
    if original is not None and not _scope(original, reference):
        problems.append("financial original reference belongs to another workspace")
    attributions = revision["attribution"]
    if [item["target"] for item in attributions] != targets:
        problems.append("financial attributions differ from the complete field bindings")
    seen: set[str] = set()
    for attribution in attributions:
        origin = attribution["origin"]
        if attribution["captured_for"] != reference:
            problems.append("financial attribution names another captured revision")
        source = origin["source_revision"]
        if any(
            source[key] != reference[key] for key in ("organization_id", "domain_id", "expense_id")
        ):
            problems.append("financial attribution origin belongs to another aggregate")
        source_version, version = (
            int(source["version"]["value"]),
            int(reference["version"]["value"]),
        )
        if source_version > version or (source_version == version and source != reference):
            problems.append("financial attribution origin has an inconsistent revision")
        if source_version < version and attribution["prior_association_id"] is None:
            problems.append("financial carried attribution has no predecessor association")
        if origin["association_id"] in seen:
            problems.append("financial attribution repeats an origin association")
        seen.add(origin["association_id"])
        target = attribution["target"]
        if origin["target"] != target:
            problems.append("financial attribution origin names another field")
        if origin["recorded_at"] > revision["recorded_at"]:
            problems.append("financial attribution origin follows the retained revision")
        if target not in values:
            problems.append("financial attribution target is not an exact atomic unit")
        else:
            try:
                digest = hashlib.sha256(canonical_bytes(values[target])).hexdigest()
            except ExportReadError:
                problems.append("financial attributed value cannot be canonically encoded")
            else:
                if digest != origin["value_sha256"]:
                    problems.append("financial attribution resolved-value digest differs")
    preimage = {
        "kind": "arboresce.financial-content.v1",
        "organization_id": reference["organization_id"],
        "domain_id": reference["domain_id"],
        "content": content,
    }
    try:
        encoded = canonical_bytes(preimage)
    except ExportReadError:
        problems.append("financial content cannot be canonically encoded")
    else:
        if (
            len(encoded) > 131_072
            or hashlib.sha256(encoded).hexdigest() != reference["content_sha256"]
        ):
            problems.append("financial content preimage exceeds its bound or has another digest")
    return problems


def _association_problems(association: dict[str, Any]) -> list[str]:
    """Join the complete association preimage to its exposed resource identities."""
    workspace, expense = association["workspace"], association["expense"]
    problems: list[str] = []
    if not _scope(expense, workspace):
        problems.append("association expense belongs to another workspace")
    preimage = association.get("content_preimage")
    if preimage is None:
        return problems + ["association has no complete content preimage"]
    expected = {
        "kind": "arboresce.expense-association-content.v1",
        **workspace,
        "project_id": association["project"]["project_id"],
        "expense_id": expense["expense_id"],
        "expense_version": expense["version"],
        "expense_content_sha256": expense["content_sha256"],
    }
    if preimage != expected:
        problems.append("association content preimage differs from its exposed identities")
    if hashlib.sha256(canonical_bytes(preimage)).hexdigest() != association["association_sha256"]:
        problems.append("association content preimage digest differs")
    return problems


def association_problems(association: dict[str, Any]) -> list[str]:
    """Compare a structurally validated complete association's identities/digest."""
    return _association_problems(association)


def financial_revision_problems(revision: dict[str, Any]) -> list[str]:
    """Compare complete validated financial content and effective attribution."""
    return _financial_revision_problems(revision)


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
            problems.extend(_financial_revision_problems(record["revision"]))
        problems.extend(_report_composition_problems(report))
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
    if not _scope(reference, request["metadata"]["context"]):
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
    problems.extend(_report_composition_problems(report))
    return problems


def _report_composition_problems(report: dict[str, Any]) -> list[str]:
    """Compare retained composition without manufacturing its creation request."""
    reference, manifest = report["reference"], report["manifest"]
    financial = manifest["financial_manifest"]
    problems: list[str] = []
    if not _scope(financial["context"], reference):
        problems.append("project report belongs to another workspace")
    if manifest["project"]["project_id"] != reference["project_id"]:
        problems.append("project report names another project")
    preimage = {"kind": "arboresce.project-financial-report.v1", "manifest": manifest}
    try:
        digest = hashlib.sha256(canonical_bytes(preimage)).hexdigest()
    except ExportReadError:
        problems.append("project report cannot be canonically encoded")
    else:
        if digest != reference["content_sha256"]:
            problems.append("project report content preimage digest differs")
    if any(total["currency"] != manifest["currency"] for total in financial["totals"]):
        problems.append("project report totals contain another currency")
    if len(financial["totals"]) > 1:
        problems.append("project report repeats a currency total")
    if sum(total["record_count"] for total in financial["totals"]) != len(financial["selected"]):
        problems.append("project report total counts differ from the frozen selection")
    selections = manifest["selected_associations"]
    problems.extend(
        _association_collection_problems(
            [entry["association"] for entry in selections]
            + [
                {"workspace": entry["identity"]["workspace"], **entry["identity"]["association"]}
                for entry in manifest["excluded_associations"]
            ],
            "project report",
        )
    )
    if len(selections) != len(financial["selected"]):
        problems.append("project report association selection is incomplete")
    seen: set[tuple[str, str, str, str, str]] = set()
    for index, entry in enumerate(selections):
        selected, association, event = (
            entry["selected"],
            entry["association"],
            entry["review_event"],
        )
        problems.extend(_association_problems(association))
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


def _lineage_problems(
    lineage: list[dict[str, Any]],
    workspace: dict[str, Any],
    label: str,
    fields: list[str] | None = None,
) -> list[str]:
    """Compare declared fields and scope present in the returned lineage itself."""
    problems: list[str] = []
    covered: set[str] = set()
    for field in lineage:
        if field["field"] in covered:
            problems.append(f"{label} lineage repeats a field")
        covered.add(field["field"])
        for entry in field["entries"]:
            if (
                entry["category"] == "source_observation"
                and entry["observation"]["organization_id"] != workspace["organization_id"]
            ):
                problems.append(f"{label} lineage observation belongs to another organization")
    if fields is not None and not set(fields).issubset(covered):
        problems.append(f"{label} lineage omits a declared changed field")
    return problems


def _association_collection_problems(
    associations: list[dict[str, Any]],
    label: str,
) -> list[str]:
    """Require a single immutable identity per scoped association revision."""
    problems: list[str] = []
    seen: dict[tuple[str, str, str, str], str] = {}
    for association in associations:
        workspace = association["workspace"]
        key = (
            workspace["organization_id"],
            workspace["domain_id"],
            association["association_id"],
            association["association_revision"]["value"],
        )
        digest = association["association_sha256"]
        if key in seen:
            if seen[key] != digest:
                problems.append(f"{label} has contradictory digests for an association revision")
            else:
                problems.append(f"{label} repeats an exact association revision")
        seen[key] = digest
    return problems


def _change_collection_problems(
    resources: list[dict[str, Any]],
    workspace: dict[str, Any],
    label: str,
) -> list[str]:
    """Share intrinsic item, lineage and returned dependency-graph comparisons."""
    problems: list[str] = []
    seen: set[tuple[str, str, str]] = set()
    candidate_versions: set[tuple[str, str]] = set()
    dependencies: dict[tuple[str, str], set[tuple[str, str]]] = {}
    for resource in resources:
        item = resource["item"]
        if not _scope(item["origin"]["capture"], workspace):
            problems.append(
                "group item names another capture revision"
                if label == "group"
                else f"{label} item origin capture belongs to another workspace"
            )
        candidate = _candidate(item["candidate"])
        if candidate in seen:
            problems.append(
                "readback repeats an exact pending candidate"
                if label == "readback pending"
                else f"{label} repeats an exact candidate"
            )
        elif candidate[:2] in candidate_versions:
            problems.append(f"{label} repeats a candidate version with another digest")
        seen.add(candidate)
        candidate_versions.add(candidate[:2])
        key = item["item_id"], item["candidate"]["version"]["value"]
        if key in dependencies:
            problems.append(f"{label} repeats an exact item revision")
        dependencies.setdefault(key, set()).update(
            {
                (dependency["item_id"], dependency["content_revision"]["value"])
                for dependency in item["dependencies"]
                if "item_id" in dependency
            }
        )
        for evidence in item["evidence"].values():
            if any(entry["organization_id"] != workspace["organization_id"] for entry in evidence):
                problems.append(f"{label} item evidence belongs to another organization")
        successor = resource["superseded_by"]
        if successor is not None:
            if (
                successor["context"] != item["candidate"]["context"]
                or successor["candidate_id"] != item["candidate"]["candidate_id"]
            ):
                problems.append(f"{label} item successor belongs to another candidate or context")
            if int(successor["version"]["value"]) != int(item["candidate"]["version"]["value"]) + 1:
                problems.append(f"{label} item successor is not the immediate revision")
        problems.extend(
            _lineage_problems(
                item["field_lineage"],
                workspace,
                f"{label} item",
                item["changed_fields"],
            )
        )
    unresolved = set(dependencies)
    while unresolved:
        ready = {key for key in unresolved if not (dependencies[key] & unresolved)}
        if not ready:
            problems.append(f"{label} item dependencies contain a cycle")
            break
        unresolved.difference_update(ready)
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
    problems.extend(_change_collection_problems(group["items"], workspace, "group"))
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
            capture["capture_id"] != group["capture_id"]
            or capture["version"] != group["content_revision"]
        ):
            problems.append("group item names another capture revision")
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
    reviewed_versions: set[tuple[str, str]] = set()
    for review in view["review_history"]:
        candidate = _candidate(review["candidate"])
        if candidate in reviewed:
            problems.append("readback repeats a verdict for an exact candidate")
        elif candidate[:2] in reviewed_versions:
            problems.append("readback review has contradictory digests for a candidate version")
        reviewed.add(candidate)
        reviewed_versions.add(candidate[:2])
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
        problems.extend(
            _lineage_problems(provenance["field_lineage"], workspace, "readback provenance")
        )
        if _record(provenance["record"]) not in records:
            problems.append("readback provenance has no exact returned record")
        if provenance["accepted_revision"] != provenance["record"]["content_revision"]:
            problems.append("readback provenance and record revisions differ")
    expenses: set[tuple[str, str, str, str, str]] = set()
    expense_versions: set[tuple[str, str, str, str]] = set()
    for expense in view["expenses"]:
        problems.extend(_financial_revision_problems(expense["revision"]))
        reference = expense["revision"]["reference"]
        if not _scope(reference, workspace):
            problems.append("readback expense belongs to another workspace")
        key = _expense(reference)
        if key in expenses:
            problems.append("readback repeats an exact expense")
        elif key[:4] in expense_versions:
            problems.append("readback has contradictory digests for an expense revision")
        expenses.add(key)
        expense_versions.add(key[:4])
        confirmation = expense["confirmation"]
        if confirmation is not None:
            if confirmation["reference"] != reference:
                problems.append("readback confirmation names another expense revision")
            if int(confirmation["control_revision"]["value"]) > int(
                expense["control_revision"]["value"]
            ):
                problems.append("readback confirmation follows the observed expense control")
        successor = expense["superseded_by"]
        if successor is not None:
            if any(
                successor[key] != reference[key]
                for key in ("organization_id", "domain_id", "expense_id")
            ):
                problems.append("readback expense successor belongs to another aggregate")
            if int(successor["version"]["value"]) != int(reference["version"]["value"]) + 1:
                problems.append("readback expense successor is not the immediate revision")
    problems.extend(_association_collection_problems(view["expense_associations"], "readback"))
    for association in view["expense_associations"]:
        problems.extend(_association_problems(association))
        if (
            association["workspace"] != workspace
            or association["project"]["project_id"] != project["project_id"]
        ):
            problems.append("readback expense association belongs to another scope")
        if _expense(association["expense"]) not in expenses:
            problems.append("readback association has no exact returned expense")
    problems.extend(
        _change_collection_problems(response["result"]["pending"], workspace, "readback pending")
    )
    for resource in response["result"]["pending"]:
        item = resource["item"]
        if item["workspace"] != workspace or item["project"]["project_id"] != project["project_id"]:
            problems.append("readback pending item belongs to another scope")
        if not _scope(item["candidate"]["context"], workspace):
            problems.append("readback pending candidate belongs to another workspace")
        if item["candidate"]["context"]["purpose"] != "project_update":
            problems.append("readback pending candidate has another purpose")
        if resource["revision_state"] != "pending" or resource["review"] is not None:
            problems.append("readback pending item is already reviewed")
        if _candidate(item["candidate"])[:2] in reviewed_versions:
            problems.append("readback pending candidate already has recorded review history")
        target = item["target"]
        if target.get("absent") is True and any(
            record[1] == target["record"]["record_id"] for record in records
        ):
            problems.append("readback pending create target already exists")
    return problems
