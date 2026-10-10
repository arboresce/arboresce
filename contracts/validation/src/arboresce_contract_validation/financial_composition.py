"""Conformance for independently reviewed, structurally validated composition inputs.

These checks compare declared complete observations and postimages. They perform
no writes, authenticate no person, qualify no assurance mechanism and establish
no live authority or transaction. Callers validate every public value against
its owning schema first. Diagnostics contain no submitted content or credentials.
"""

from __future__ import annotations

import hashlib
from datetime import datetime, timedelta
from typing import Any

from arboresce_contract_validation.project_exports import ExportReadError, canonical_bytes
from arboresce_contract_validation.project_semantics import (
    association_problems,
    financial_revision_problems,
)

MAX_COUNTER = 999_999_999_999_999_999


def _digest(value: Any) -> str:
    return hashlib.sha256(canonical_bytes(value)).hexdigest()


def _canonical_problems(*values: Any) -> list[str]:
    try:
        for value in values:
            canonical_bytes(value)
    except ExportReadError:
        return ["composition input violates bounded canonical subset"]
    return []


def _association_reference(association: dict[str, Any]) -> dict[str, Any]:
    return {
        key: association[key]
        for key in ("association_id", "association_revision", "association_sha256")
    }


def _successor(before: dict[str, Any], after: dict[str, Any]) -> bool:
    value = int(before["value"])
    return value < MAX_COUNTER and after == {"kind": before["kind"], "value": str(value + 1)}


def _project_problems(
    workspace: dict[str, Any],
    reference: dict[str, Any],
    control: dict[str, Any],
    observation: dict[str, Any],
) -> list[str]:
    project = observation["project"]
    if (
        project["workspace"] != {key: workspace[key] for key in ("organization_id", "domain_id")}
        or reference != {key: project[key] for key in ("project_id", "content_revision")}
        or control != observation["control_revision"]
    ):
        return ["composition independently observed project fence differs"]
    return []


def content_problems(content: dict[str, Any], association: dict[str, Any]) -> list[str]:
    """Join complete immutable identities without inferring current heads/controls."""
    problems = _canonical_problems(content, association)
    if problems:
        return problems
    problems.extend(association_problems(association))
    reference, preimage = content["reference"], content["preimage"]
    expected = {
        "kind": "arboresce.financial-composition-content.v1",
        "workspace": association["workspace"],
        "project": association["project"],
        "expense": association["expense"],
        "association": _association_reference(association),
    }
    if preimage != expected:
        problems.append("composition complete content identities differ")
    if any(
        reference[key] != association["workspace"][key] for key in ("organization_id", "domain_id")
    ):
        problems.append("composition reference belongs to another workspace")
    if reference["content_sha256"] != _digest(preimage):
        problems.append("composition content digest differs")
    return problems


def _binding_history_problems(bindings: list[dict[str, Any]]) -> list[str]:
    problems: list[str] = []
    seen: set[str] = set()
    confirmed_revisions: set[tuple[str, str, str, str]] = set()
    for binding in bindings:
        confirmation, event, association = (
            binding["expense_confirmation"],
            binding["event"],
            binding["association"],
        )
        if event["financial_review_event_id"] in seen:
            problems.append("composition financial binding history repeats an event")
        seen.add(event["financial_review_event_id"])
        reference = confirmation["reference"]
        revision_identity = (
            reference["organization_id"],
            reference["domain_id"],
            reference["expense_id"],
            reference["version"]["value"],
        )
        if revision_identity in confirmed_revisions:
            problems.append("composition financial binding history repeats a confirmed revision")
        confirmed_revisions.add(revision_identity)
        if (
            event["financial_review_event_id"] != confirmation["financial_review_event_id"]
            or event["expense"] != confirmation["reference"]
            or event["reviewer"] != confirmation["reviewer"]
            or event["executor_scope"]
            != {
                "executor": confirmation["executor"],
                "delegation_id": confirmation["delegation_id"],
            }
            or not event["recorded_at"] <= confirmation["recorded_at"] < event["expires_at"]
        ):
            problems.append("composition retained financial confirmation joins differ")
        expected = None if association is None else _association_reference(association)
        if (
            event["assurance"]["state"] != "qualified"
            or event["assurance"]["review_policy"] != event["review_policy"]
            or event["decision"] != "confirm"
            or event["expense_version"] != confirmation["reference"]["version"]
            or event["content_sha256"] != confirmation["reference"]["content_sha256"]
            or int(confirmation["control_revision"]["value"]) < 2
            or any(
                event[key] != confirmation["reference"][key]
                for key in ("organization_id", "domain_id")
            )
            or (
                event["state"] == "revoked"
                and not (
                    event["recorded_at"] <= event["revoked_at"]
                    and confirmation["recorded_at"] < event["revoked_at"]
                )
            )
        ):
            problems.append("composition original financial assurance declaration differs")
        if event["association"] != expected:
            problems.append("composition historical association binding differs")
        if association is not None:
            problems.extend(association_problems(association))
            if association["expense"] != confirmation["reference"]:
                problems.append("composition historical association names another expense")
            if association["provenance"]["recorded_at"] > event["recorded_at"]:
                problems.append("composition historical review precedes association creation")
    return problems


def _expense_snapshot_problems(snapshot: dict[str, Any], observed_at: str) -> list[str]:
    revision, confirmation = snapshot["revision"], snapshot["confirmation"]
    problems = financial_revision_problems(revision)
    if revision["recorded_at"] > observed_at:
        problems.append("composition selected expense does not yet exist")
    if confirmation is not None and (
        confirmation["reference"] != revision["reference"]
        or int(confirmation["control_revision"]["value"]) < 2
        or int(confirmation["control_revision"]["value"])
        > int(snapshot["control_revision"]["value"])
        or not revision["recorded_at"] <= confirmation["recorded_at"] <= observed_at
    ):
        problems.append("composition selected expense original confirmation differs")
    return problems


def _spent_event_problems(snapshot: dict[str, Any], event_id: str) -> list[str]:
    confirmation = snapshot["confirmation"]
    if confirmation is not None and confirmation["financial_review_event_id"] == event_id:
        return ["composition event was already consumed by the original expense confirmation"]
    return []


def proposal_problems(vector: dict[str, Any], profile: dict[str, Any]) -> list[str]:
    """Compare complete independently obtained first/correction fences and outputs."""
    problems = _canonical_problems(
        *(
            vector[key]
            for key in (
                "request",
                "before_composition",
                "before_association",
                "expense",
                "after_composition",
                "after_association",
                "known_absence",
                "financial_bindings",
                "project",
            )
        )
    )
    if problems:
        return problems
    request, before, prior, expense = (
        vector["request"],
        vector["before_composition"],
        vector["before_association"],
        vector["expense"],
    )
    body, output, association = (
        request["body"],
        vector["after_composition"],
        vector["after_association"],
    )
    mode, fences, absence = body["mode"], body["fences"], vector["known_absence"]
    problems.extend(content_problems(body["composition"], body["association"]))
    terminal = output["revision"]["provenance"]["recorded_at"]
    if body["association"]["provenance"]["recorded_at"] > terminal:
        problems.append("composition new association follows proposal terminal")
    input_times = [expense["revision"]["recorded_at"]]
    input_times.extend(
        binding["expense_confirmation"]["recorded_at"] for binding in vector["financial_bindings"]
    )
    if prior is not None:
        input_times.append(prior["association"]["provenance"]["recorded_at"])
    if before is not None:
        input_times.append(before["revision"]["provenance"]["recorded_at"])
        if before["confirmation"] is not None:
            input_times.append(before["confirmation"]["recorded_at"])
    if body["association"]["provenance"]["recorded_at"] < max(input_times):
        problems.append("composition new association precedes required inputs")
    problems.extend(_expense_snapshot_problems(expense, terminal))
    problems.extend(_binding_history_problems(vector["financial_bindings"]))
    problems.extend(
        _project_problems(
            request["metadata"]["context"],
            body["association"]["project"],
            fences["project_control_revision"],
            vector["project"],
        )
    )
    content = body["composition"]
    workspace = {
        key: request["metadata"]["context"][key] for key in ("organization_id", "domain_id")
    }
    if body["association"]["workspace"] != workspace:
        problems.append("composition proposal belongs to another workspace")
    bindings = vector["financial_bindings"]
    if any(binding["expense_confirmation"]["recorded_at"] > terminal for binding in bindings):
        problems.append("composition retained financial history follows proposal")
    reference = expense["revision"]["reference"]
    if not bindings or any(
        any(
            binding["expense_confirmation"]["reference"][key] != reference[key]
            for key in ("organization_id", "domain_id", "expense_id")
        )
        or int(binding["expense_confirmation"]["reference"]["version"]["value"])
        > int(reference["version"]["value"])
        or (
            expense["revision_state"] == "proposed"
            and binding["expense_confirmation"]["reference"]["version"] == reference["version"]
        )
        or (
            binding["expense_confirmation"]["reference"]["version"] == reference["version"]
            and (
                binding["expense_confirmation"]["reference"] != reference
                or binding["expense_confirmation"] != expense["confirmation"]
            )
        )
        for binding in bindings
    ):
        problems.append("composition required financial binding history differs")
    if expense["revision_state"] == "confirmed" and (
        expense["confirmation"]["reference"] != reference
        or not any(
            binding["expense_confirmation"] == expense["confirmation"] for binding in bindings
        )
    ):
        problems.append("composition selected original confirmation is absent or contradictory")
    if (
        body["association"]["expense"] != expense["revision"]["reference"]
        or fences["expense_control_revision"] != expense["control_revision"]
        or expense["revision_state"] not in {"confirmed", "proposed"}
        or (
            expense["revision_state"] == "proposed"
            and expense["revision"]["reference"]["version"]["value"] == "1"
        )
    ):
        problems.append("composition proposal current expense fence differs")
    if (
        output["revision"]["content"] != content
        or association["association"] != body["association"]
        or output["state"] != "proposed"
        or output["confirmation"] is not None
        or output["superseded_by"] is not None
        or any(
            vector["after_response"]["result"][key] != value
            for key, value in {
                "composition": output,
                "association": association,
                "expense": expense,
                "project": vector["project"],
            }.items()
        )
    ):
        problems.append("composition proposal complete postimage differs")
    if not absence["target_relationship"]:
        problems.append("composition proposal target relationship absence is not established")
    if mode == "first_unassociated":
        if (
            prior is not None
            or not absence["association"]
            or fences["association"] is not None
            or fences["association_control_revision"] is not None
        ):
            problems.append("composition first association absence differs")
        if (
            body["association"]["association_revision"] != {"kind": "content", "value": "1"}
            or association["control_revision"] != {"kind": "control", "value": "1"}
            or body["association"]["expected_control_revision"] != {"kind": "control", "value": "1"}
        ):
            problems.append("composition first association counters differ")
        if (
            expense["revision_state"] != "confirmed"
            or any(binding["event"]["association"] is not None for binding in bindings)
            or not any(
                binding["expense_confirmation"] == expense["confirmation"] for binding in bindings
            )
        ):
            problems.append("composition original unassociated confirmation differs")
    elif prior is None:
        problems.append("composition actual association predecessor is missing")
    else:
        predecessor = prior["association"]
        problems.extend(association_problems(predecessor))
        if predecessor["provenance"]["recorded_at"] > terminal:
            problems.append("composition actual association predecessor does not yet exist")
        if predecessor["workspace"] != body["association"]["workspace"] or any(
            predecessor["expense"][key] != body["association"]["expense"][key]
            for key in ("organization_id", "domain_id", "expense_id")
        ):
            problems.append("composition actual association predecessor ownership differs")
        if (
            fences["association"] != _association_reference(predecessor)
            or fences["association_control_revision"] != prior["control_revision"]
            or body["association"]["expected_control_revision"] != prior["control_revision"]
        ):
            problems.append("composition actual association fence differs")
        if (
            body["association"]["association_id"] != predecessor["association_id"]
            or not _successor(
                predecessor["association_revision"], body["association"]["association_revision"]
            )
            or not _successor(prior["control_revision"], association["control_revision"])
        ):
            problems.append("composition association successor counters differ")
        if body["association"]["content_preimage"] == predecessor["content_preimage"]:
            problems.append("composition association is a normalized no-op")
    if mode == "correction":
        if before is None:
            problems.append("composition actual composition predecessor is missing")
        else:
            previous = before["revision"]["content"]["reference"]
            if before["revision"]["provenance"]["recorded_at"] > terminal:
                problems.append("composition actual composition predecessor does not yet exist")
            if prior is not None:
                problems.extend(
                    content_problems(before["revision"]["content"], prior["association"])
                )
            if before["state"] == "confirmed" and prior is not None:
                retained = before["confirmation"]
                if (
                    retained["reference"] != previous
                    or retained["association"] != _association_reference(prior["association"])
                    or int(retained["control_revision"]["value"]) < 2
                    or int(retained["association_control_revision"]["value"]) < 2
                    or int(retained["control_revision"]["value"])
                    > int(before["control_revision"]["value"])
                    or int(retained["association_control_revision"]["value"])
                    > int(prior["control_revision"]["value"])
                    or not before["revision"]["provenance"]["recorded_at"]
                    <= retained["recorded_at"]
                    <= output["revision"]["provenance"]["recorded_at"]
                ):
                    problems.append("composition retained predecessor confirmation differs")
                if retained["expense_disposition"] == "confirmed_now":
                    original = [
                        binding["expense_confirmation"]
                        for binding in bindings
                        if binding["expense_confirmation"]["reference"]
                        == before["revision"]["content"]["preimage"]["expense"]
                    ]
                    if len(original) != 1 or any(
                        retained[key] != original[0][key]
                        for key in (
                            "financial_review_event_id",
                            "reviewer",
                            "executor",
                            "delegation_id",
                            "authority_audit_event_id",
                            "recorded_at",
                        )
                    ):
                        problems.append(
                            "composition retained compound and base confirmation differ"
                        )
            if (
                fences["composition"] != previous
                or fences["composition_control_revision"] != before["control_revision"]
                or before["state"] not in {"proposed", "confirmed"}
            ):
                problems.append("composition current predecessor fence differs")
            if (
                any(
                    content["reference"][key] != previous[key]
                    for key in ("organization_id", "domain_id", "composition_id")
                )
                or not _successor(
                    previous["content_revision"], content["reference"]["content_revision"]
                )
                or not _successor(before["control_revision"], output["control_revision"])
            ):
                problems.append("composition successor counters differ")
    elif (
        before is not None
        or not absence["composition"]
        or fences["composition"] is not None
        or fences["composition_control_revision"] is not None
    ):
        problems.append("composition first aggregate absence differs")
    elif content["reference"]["content_revision"] != {"kind": "content", "value": "1"} or output[
        "control_revision"
    ] != {"kind": "control", "value": "1"}:
        problems.append("composition first aggregate counters differ")
    problems.extend(
        _receipt_problems(
            request,
            vector["after_response"]["idempotency_receipt"],
            "/v1/financial-compositions",
            output["revision"]["provenance"]["recorded_at"],
            profile,
        )
    )
    return problems


def review_problems(vector: dict[str, Any]) -> list[str]:
    """Compare a compound issuance observation; labels confer no actual assurance."""
    problems = _canonical_problems(
        *(
            vector[key]
            for key in (
                "challenge",
                "event",
                "proof",
                "rendering",
                "composition",
                "association",
                "expense",
                "controls",
                "project",
            )
        )
    )
    if problems:
        return problems
    challenge, event, proof, rendering = (
        vector["challenge"],
        vector["event"],
        vector["proof"],
        vector["rendering"],
    )
    composition, association, expense, controls = (
        vector["composition"],
        vector["association"]["association"],
        vector["expense"],
        vector["controls"],
    )
    content = composition["revision"]["content"]
    problems.extend(content_problems(content, association))
    problems.extend(_expense_snapshot_problems(expense, challenge["base"]["issued_at"]))
    problems.extend(
        _project_problems(
            content["preimage"]["workspace"],
            association["project"],
            controls["project"],
            vector["project"],
        )
    )
    base, issued = challenge["base"], event["base"]
    problems.extend(_spent_event_problems(expense, issued["financial_review_event_id"]))
    reference = expense["revision"]["reference"]
    expected_base = {
        "kind": "arboresce.financial-review-content.v1",
        "organization_id": reference["organization_id"],
        "domain_id": reference["domain_id"],
        "purpose": base["purpose"],
        "expense_id": reference["expense_id"],
        "association": _association_reference(association),
        "expense_version": reference["version"],
        "content_sha256": reference["content_sha256"],
        "decision": "confirm",
        "review_policy": base["review_policy"],
        "assurance_profile": base["assurance"]["assurance_profile"],
        "reviewer_id": proof["reviewer"]["principal_id"],
        "executor_scope": base["executor_scope"],
        "expires_at": base["expires_at"],
    }
    expected = {
        "kind": "arboresce.financial-composition-review-content.v1",
        "base": expected_base,
        "composition": content["reference"],
        "project": association["project"],
        "controls": controls,
    }
    digest = _digest(expected)
    if challenge["preimage"] != expected or event["preimage"] != expected:
        problems.append("composition compound review preimage differs")
    if (
        any(value["review_sha256"] != digest for value in (challenge, event, proof, rendering))
        or proof["proof"]["digest"] != digest
    ):
        problems.append("composition compound review digest differs")
    if (
        controls["composition"] != composition["control_revision"]
        or controls["association"] != vector["association"]["control_revision"]
        or controls["expense"] != expense["control_revision"]
        or composition["state"] != "proposed"
        or expense["revision_state"] not in {"proposed", "confirmed"}
        or association["expense"] != reference
    ):
        problems.append("composition review current targets or controls differ")
    exact = (
        "challenge_id",
        "review_policy",
        "assurance",
        "organization_id",
        "domain_id",
        "purpose",
        "expense",
        "association",
        "expense_version",
        "content_sha256",
        "decision",
        "expires_at",
        "executor_scope",
    )
    if any(issued[key] != base[key] for key in exact) or (
        base["expense"] != reference
        or base["association"] != expected_base["association"]
        or base["expense_version"] != reference["version"]
        or base["content_sha256"] != reference["content_sha256"]
        or proof["challenge_id"] != base["challenge_id"]
        or proof["reviewer"] != issued["reviewer"]
        or proof["assurance"] != base["assurance"]
        or proof["decision"] != base["decision"]
        or base["decision"] != "confirm"
        or base["assurance"]["state"] != "qualified"
        or base["assurance"]["review_policy"] != base["review_policy"]
        or issued["state"] != "issued"
        or issued["revoked_at"] is not None
    ):
        problems.append("composition compound event base joins differ")
    if not (
        composition["revision"]["provenance"]["recorded_at"] <= base["issued_at"]
        and association["provenance"]["recorded_at"] <= base["issued_at"]
        and expense["revision"]["recorded_at"] <= base["issued_at"]
        and base["issued_at"] <= proof["asserted_at"] <= issued["recorded_at"] < base["expires_at"]
    ):
        problems.append("composition compound review chronology differs")
    display = {
        "composition": content,
        "association": association,
        "expense_content": expense["revision"]["content"],
        "review_preimage": expected,
    }
    text = "Arboresce financial composition review v1\n" + canonical_bytes(display).decode("utf-8")
    if (
        rendering["text"] != text
        or rendering["rendering_sha256"] != hashlib.sha256(text.encode("utf-8")).hexdigest()
    ):
        problems.append("composition exact consequential rendering differs")
    return problems


def _fingerprint(request: dict[str, Any], route: str) -> str:
    preimage = {
        "method": "POST",
        "resource": route,
        "contract_version": "v1",
        "purpose": request["metadata"]["context"]["purpose"],
        "body": request,
    }
    return hashlib.sha256(b"arboresce.command.v1\0" + canonical_bytes(preimage)).hexdigest()


def _receipt_problems(
    request: dict[str, Any],
    receipt: dict[str, Any],
    route: str,
    terminal: str,
    profile: dict[str, Any],
) -> list[str]:
    problems: list[str] = []
    replay, diagnostic = profile["replay_retention_ms"], profile["diagnostic_retention_ms"]
    if (
        not 604_800_000 <= replay < diagnostic <= 9_007_199_254_740_991
        or diagnostic < 2_592_000_000
    ):
        problems.append("composition selected retention profile differs")
    try:
        instant = datetime.fromisoformat(terminal)
        deadlines = tuple(
            (instant + timedelta(milliseconds=duration))
            .isoformat(timespec="milliseconds")
            .replace("+00:00", "Z")
            for duration in (replay, diagnostic)
        )
    except OverflowError:
        problems.append("composition original receipt deadlines are unrepresentable")
    else:
        if deadlines != (receipt["expires_at"], receipt["tombstone_expires_at"]):
            problems.append("composition original receipt deadlines differ")
    if (
        receipt["request_fingerprint"] != _fingerprint(request, route)
        or receipt["key_digest"]
        != hashlib.sha256(request["metadata"]["idempotency"]["key"].encode("ascii")).hexdigest()
        or receipt["replayed"] is not False
    ):
        problems.append("composition original receipt command identity differs")
    return problems


def confirmation_problems(vector: dict[str, Any], profile: dict[str, Any]) -> list[str]:
    """Check declared fresh confirmation outputs and the selected global event key."""
    problems = _canonical_problems(
        *(
            vector[key]
            for key in (
                "request",
                "before_composition",
                "before_association",
                "before_expense",
                "event",
                "project",
            )
        ),
        *(
            vector["after"]["result"][key]
            for key in ("composition", "association", "expense", "project")
        ),
        profile,
    )
    if problems:
        return problems
    request, before, prior, expense, event = (
        vector["request"],
        vector["before_composition"],
        vector["before_association"],
        vector["before_expense"],
        vector["event"],
    )
    body, result = request["body"], vector["after"]["result"]
    after, association, selected = result["composition"], result["association"], result["expense"]
    content, base, preimage = before["revision"]["content"], event["base"], event["preimage"]
    problems.extend(_spent_event_problems(expense, base["financial_review_event_id"]))
    problems.extend(content_problems(content, prior["association"]))
    problems.extend(_expense_snapshot_problems(expense, base["recorded_at"]))
    problems.extend(
        _project_problems(
            request["metadata"]["context"],
            body["project"],
            body["controls"]["project"],
            vector["project"],
        )
    )
    expected_controls = {
        "project": vector["project"]["control_revision"],
        "expense": expense["control_revision"],
        "association": prior["control_revision"],
        "composition": before["control_revision"],
    }
    if (
        body["composition"] != content["reference"]
        or body["project"] != content["preimage"]["project"]
        or body["expense"] != expense["revision"]["reference"]
        or body["expense"] != prior["association"]["expense"]
        or body["expense"] != content["preimage"]["expense"]
        or body["association"] != _association_reference(prior["association"])
        or body["controls"] != expected_controls
        or body["financial_review_event_id"] != base["financial_review_event_id"]
        or before["state"] != "proposed"
        or expense["revision_state"] not in {"proposed", "confirmed"}
    ):
        problems.append("composition fresh confirmation fences differ")
    if any(
        request["metadata"]["context"][key] != content["reference"][key]
        for key in ("organization_id", "domain_id")
    ):
        problems.append("composition confirmation belongs to another workspace")
    expected_base = {
        "kind": "arboresce.financial-review-content.v1",
        "organization_id": base["organization_id"],
        "domain_id": base["domain_id"],
        "purpose": base["purpose"],
        "expense_id": body["expense"]["expense_id"],
        "association": body["association"],
        "expense_version": body["expense"]["version"],
        "content_sha256": body["expense"]["content_sha256"],
        "decision": "confirm",
        "review_policy": base["review_policy"],
        "assurance_profile": base["assurance"]["assurance_profile"],
        "reviewer_id": base["reviewer"]["principal_id"],
        "executor_scope": base["executor_scope"],
        "expires_at": base["expires_at"],
    }
    expected = {
        "kind": "arboresce.financial-composition-review-content.v1",
        "base": expected_base,
        "composition": body["composition"],
        "project": body["project"],
        "controls": expected_controls,
    }
    confirmation = after["confirmation"]
    if (
        preimage != expected
        or event["review_sha256"] != _digest(preimage)
        or base["expense"] != body["expense"]
        or base["association"] != body["association"]
        or base["expense_version"] != body["expense"]["version"]
        or base["content_sha256"] != body["expense"]["content_sha256"]
        or any(base[key] != content["reference"][key] for key in ("organization_id", "domain_id"))
        or base["decision"] != "confirm"
        or base["state"] != "issued"
        or base["revoked_at"] is not None
        or base["assurance"]["state"] != "qualified"
        or base["assurance"]["review_policy"] != base["review_policy"]
        or not base["recorded_at"] <= confirmation["recorded_at"] < base["expires_at"]
        or before["revision"]["provenance"]["recorded_at"] > base["recorded_at"]
        or prior["association"]["provenance"]["recorded_at"] > base["recorded_at"]
        or expense["revision"]["recorded_at"] > base["recorded_at"]
    ):
        problems.append("composition fresh compound authority declaration differs")
    if (
        after["revision"] != before["revision"]
        or after["state"] != "confirmed"
        or after["superseded_by"] is not None
        or association["association"] != prior["association"]
        or result["project"] != vector["project"]
        or not _successor(before["control_revision"], after["control_revision"])
        or not _successor(prior["control_revision"], association["control_revision"])
        or confirmation["reference"] != content["reference"]
        or confirmation["control_revision"] != after["control_revision"]
        or confirmation["association"] != body["association"]
        or confirmation["association_control_revision"] != association["control_revision"]
        or confirmation["financial_review_event_id"] != base["financial_review_event_id"]
        or confirmation["reviewer"] != base["reviewer"]
        or {"executor": confirmation["executor"], "delegation_id": confirmation["delegation_id"]}
        != base["executor_scope"]
    ):
        problems.append("composition complete confirmation postimage differs")
    if expense["revision_state"] == "confirmed":
        if selected != expense or confirmation["expense_disposition"] != "already_confirmed":
            problems.append("composition original confirmed expense was changed")
    else:
        retained = selected["confirmation"]
        if (
            confirmation["expense_disposition"] != "confirmed_now"
            or selected["revision"] != expense["revision"]
            or selected["revision_state"] != "confirmed"
            or selected["superseded_by"] is not None
            or not _successor(expense["control_revision"], selected["control_revision"])
            or retained["reference"] != body["expense"]
            or retained["control_revision"] != selected["control_revision"]
            or retained["financial_review_event_id"] != base["financial_review_event_id"]
            or any(
                retained[key] != confirmation[key]
                for key in (
                    "reviewer",
                    "executor",
                    "delegation_id",
                    "authority_audit_event_id",
                    "recorded_at",
                )
            )
        ):
            problems.append("composition proposed expense confirmation differs")
    previous, observed = vector["prior_consumptions"], vector["after_consumptions"]
    if len(observed) != len(previous) + 1 or observed[:-1] != previous:
        problems.append("composition single event consumption append differs")
    else:
        consumption = observed[-1]
        if any(
            row["financial_review_event_id"] == base["financial_review_event_id"]
            for row in previous
        ) or (
            consumption["financial_review_event_id"] != base["financial_review_event_id"]
            or consumption["composition"] != body["composition"]
            or consumption["expense"] != body["expense"]
            or consumption["association"] != body["association"]
            or consumption["executor_scope"] != base["executor_scope"]
            or consumption["recorded_at"] != confirmation["recorded_at"]
            or consumption["audit_event_id"] != confirmation["audit_event_id"]
        ):
            problems.append("composition selected global event consumption differs")
    route = "/v1/financial-compositions/" + body["composition"]["composition_id"] + "/confirm"
    if observed and observed[-1]["request_fingerprint"] != _fingerprint(request, route):
        problems.append("composition consumption original command fingerprint differs")
    problems.extend(
        _receipt_problems(
            request,
            vector["after"]["idempotency_receipt"],
            route,
            confirmation["recorded_at"],
            profile,
        )
    )
    return problems
