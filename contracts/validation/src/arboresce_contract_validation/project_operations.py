"""Cross-record checks for structurally validated focused export operations.

Inputs include independently obtained selected profiles, source/admission and
retained artifact snapshots. These comparisons authenticate no actor or policy,
implement no transaction/worker, and confer no authority from a receipt. They
require complete owning-schema validation before entry. Diagnostics contain no
raw idempotency key, submitted content or credentials.
"""

from __future__ import annotations

import hashlib
from datetime import datetime, timedelta
from typing import Any, cast

from arboresce_contract_validation.project_exports import ExportReadError, canonical_bytes


def _digest(value: dict[str, Any]) -> str:
    return hashlib.sha256(canonical_bytes(value)).hexdigest()


def _canonical_problems(*values: Any) -> list[str]:
    try:
        for value in values:
            canonical_bytes(value)
    except ExportReadError:
        return ["operation input violates bounded canonical subset"]
    return []


def _scope(value: dict[str, Any], workspace: dict[str, Any]) -> bool:
    return all(value[key] == workspace[key] for key in ("organization_id", "domain_id"))


def _after(instant: str, duration: int) -> str:
    value = datetime.fromisoformat(instant) + timedelta(
        seconds=duration // 1000, milliseconds=duration % 1000
    )
    return value.isoformat(timespec="milliseconds").replace("+00:00", "Z")


def command_fingerprint(request: dict[str, Any]) -> str:
    """Hash the complete validated submitted export/cancel command preimage."""
    command, body = request["metadata"]["command"], request["body"]
    if command == "project.export":
        resource = "/v1/projects/" + body["project_id"] + "/exports"
    elif command == "project_report.export":
        resource = "/v1/project-reports/" + body["report"]["report_id"] + "/exports"
    else:
        resource = "/v1/operations/" + body["operation"]["operation_id"] + "/cancel"
    preimage = {
        "method": "POST",
        "resource": resource,
        "contract_version": request["metadata"]["contract_version"],
        "purpose": request["metadata"]["context"]["purpose"],
        "body": request,
    }
    return hashlib.sha256(b"arboresce.command.v1\0" + canonical_bytes(preimage)).hexdigest()


def receipt_problems(
    request: dict[str, Any],
    receipt: dict[str, Any],
    profile: dict[str, Any],
    terminal_at: str | None,
    *,
    replayed: bool,
) -> list[str]:
    """Compare a declared original command/terminal observation with its receipt.

    The original submitted command is an independent protected conformance input;
    this API does not reconstruct it from a public operation or expired receipt.
    """
    problems = _canonical_problems(request, receipt, profile)
    if problems:
        return problems
    key = request["metadata"]["idempotency"]["key"]
    if receipt["key_digest"] != hashlib.sha256(key.encode("ascii")).hexdigest():
        problems.append("receipt key digest differs")
    if receipt["request_fingerprint"] != command_fingerprint(request):
        problems.append("receipt request fingerprint differs")
    if receipt["replayed"] is not replayed:
        problems.append("receipt replay disposition differs")
    retention = profile["retention"]
    if retention["diagnostic_retention_ms"] <= retention["replay_retention_ms"]:
        problems.append("receipt diagnostic retention is not later than replay")
    try:
        expected = (
            (None, None)
            if terminal_at is None
            else (
                _after(terminal_at, retention["replay_retention_ms"]),
                _after(terminal_at, retention["diagnostic_retention_ms"]),
            )
        )
        if (receipt["expires_at"], receipt["tombstone_expires_at"]) != expected:
            problems.append("receipt terminal-relative deadlines differ")
    except OverflowError:
        problems.append("receipt deadline arithmetic is unrepresentable")
    return problems


def _contains_key(value: Any, key: str) -> bool:
    if isinstance(value, str):
        return key in value
    if isinstance(value, dict):
        mapping = cast("dict[str, Any]", value)
        return any(key in name or _contains_key(item, key) for name, item in mapping.items())
    if isinstance(value, list):
        return any(_contains_key(item, key) for item in cast("list[Any]", value))
    return False


def _principal_kinds_conflict(executions: list[dict[str, Any]]) -> bool:
    kinds: dict[str, str] = {}
    for execution in executions:
        principal = execution["executor"]
        identifier = principal["principal_id"]
        if identifier in kinds and kinds[identifier] != principal["kind"]:
            return True
        kinds[identifier] = principal["kind"]
    return False


def _profile_scope_problems(profile: dict[str, Any], context: dict[str, Any]) -> list[str]:
    policy = profile["lifecycle_policy"]
    if (
        not _scope(profile["delivery_profile"], context)
        or policy["organization_id"] != context["organization_id"]
        or (policy["policy_scope"] == "domain" and policy["domain_id"] != context["domain_id"])
    ):
        return ["operation selected delivery or policy scope differs"]
    return []


def _input_subject_problems(
    command: str,
    body: dict[str, Any],
    context: dict[str, Any],
    subject: dict[str, Any] | None,
) -> list[str]:
    problems: list[str] = []
    if command == "project_report.export" and not _scope(body["report"], context):
        problems.append("operation input report belongs to another workspace")
    if subject is None:
        return problems
    if command == "project.export":
        selected = {"project_id": body["project_id"], "content_revision": body["project_revision"]}
        if subject["kind"] != "project" or subject.get("project") != selected:
            problems.append("operation source differs from selected project revision")
    elif subject["kind"] != "project_financial_report" or subject.get("report") != body["report"]:
        problems.append("operation source differs from selected report")
    if subject["observed_control_revision"] != body["expected_control_revision"]:
        problems.append("operation source control differs")
    if subject["kind"] == "project_financial_report" and not _scope(subject["report"], context):
        problems.append("operation source report belongs to another workspace")
    return problems


def operation_problems(
    request: dict[str, Any],
    operation: dict[str, Any],
    plan: dict[str, Any],
    profile: dict[str, Any],
    admission: dict[str, Any],
    artifact: dict[str, Any] | None,
) -> list[str]:
    """Compare complete declared operation history and independently selected inputs."""
    problems = _canonical_problems(request, operation, plan, profile, admission, artifact)
    if problems:
        return problems
    command, context, body = (
        request["metadata"]["command"],
        request["metadata"]["context"],
        request["body"],
    )
    execution, core = operation["execution"], plan["core"]
    reference = admission["operation"]
    if any(operation[key] != reference[key] for key in reference):
        problems.append("operation identity differs from admission")
    if operation["command"] != command or operation["input"] != body:
        problems.append("operation command or complete input differs")
    if context != admission["context"] or operation["context"] != context:
        problems.append("operation context differs from admission")
    if not _scope(reference, context):
        problems.append("operation admission belongs to another workspace")
    expected_cause = {
        "kind": "command",
        "command": command,
        "context": context,
        "execution": admission["execution"],
        "admitted_at": admission["admitted_at"],
    }
    if (
        operation["causal_command"] != expected_cause
        or operation["execution_attribution"] != admission["execution"]
        or operation["origin"]
        != {"kind": "command", "command_id": admission["execution"]["command_id"]}
    ):
        problems.append("operation historical attribution differs")
    if _contains_key(operation, request["metadata"]["idempotency"]["key"]) or _contains_key(
        plan, request["metadata"]["idempotency"]["key"]
    ):
        problems.append("operation or plan retains the raw key")
    expected_plan_ref = {
        "organization_id": reference["organization_id"],
        "domain_id": reference["domain_id"],
        "plan_id": core["plan_id"],
        "plan_sha256": plan["plan_sha256"],
    }
    if execution["plan"] != expected_plan_ref or plan["plan_sha256"] != _digest(core):
        problems.append("operation complete plan identity or digest differs")
    if core["operation"] != reference or core["command"] != command or core["context"] != context:
        problems.append("operation plan admission joins differ")
    if (
        not _scope(core["profile"], context)
        or core["profile"]["content_sha256"] != _digest(profile)
        or core["limits"] != profile["limits"]
    ):
        problems.append("operation selected profile or limits differ")
    problems.extend(_profile_scope_problems(profile, context))
    source = admission["source"]
    problems.extend(_input_subject_problems(command, body, context, source))
    input_digest = _digest(body)
    declaration = core["stages"][0]
    stage = execution["stages"][0]
    if core["input_sha256"] != input_digest or declaration["input_sha256"] != input_digest:
        problems.append("operation complete domain-input digest differs")
    if declaration["profile"] != core["profile"] or any(
        declaration[key] != profile["stage"][key] for key in ("kind", "required", "output_kind")
    ):
        problems.append("operation stage declaration differs from selected profile")
    if any(stage[key] != declaration[key] for key in ("stage_id", "ordinal", "kind", "required")):
        problems.append("operation stage differs from immutable plan")
    problems.extend(
        _execution_problems(operation, profile["limits"], admission["admitted_at"], stage)
    )
    if artifact is not None and artifact["core"]["profile"] != profile["delivery_profile"]:
        problems.append("operation retained artifact delivery profile differs")
    problems.extend(
        _ledger_problems(
            operation, admission["context"], admission["execution"], admission["admitted_at"]
        )
    )
    problems.extend(_output_problems(operation, core, source, input_digest, artifact))
    return problems


def _retained_cause_consistent(operation: dict[str, Any]) -> bool:
    """Compare retained attribution without qualifying current actor authority."""
    causal = operation["causal_command"]
    return (
        causal["context"] == operation["context"]
        and _scope(operation, causal["context"])
        and causal["command"] == operation["command"]
        and causal["execution"] == operation["execution_attribution"]
        and operation["origin"]
        == {"kind": "command", "command_id": causal["execution"]["command_id"]}
    )


def _execution_problems(
    operation: dict[str, Any],
    limits: dict[str, Any],
    admitted_at: str,
    stage: dict[str, Any],
) -> list[str]:
    problems: list[str] = []
    execution = operation["execution"]
    timing, attempts = execution["timing"], execution["attempts"]
    cancellation = operation["control"]["cancellation"]
    cancellation_deadline: str | None = None
    if any(
        count > limits[limit]
        for count, limit in (
            (len(attempts), "max_total_attempts"),
            (len(operation["effects"]["entries"]), "max_canonical_effects"),
            (len(operation["exposures"]["entries"]), "max_exposures"),
        )
    ):
        problems.append("operation retained ledger exceeds selected budget")
    if timing["admitted_at"] != admitted_at:
        problems.append("operation timing differs from admission")
    try:
        work = _after(admitted_at, limits["operation_lifetime_ms"])
        reconciliation = _after(work, limits["reconciliation_lifetime_ms"])
        if cancellation["kind"] == "requested":
            cancellation_deadline = min(
                work, _after(cancellation["requested_at"], limits["cancellation_observation_ms"])
            )
        if timing["work_deadline"] != work or timing["reconciliation_deadline"] != reconciliation:
            problems.append("operation fixed deadlines differ")
        if stage["started_at"] is not None:
            expected_stage = min(work, _after(stage["started_at"], limits["stage_lifetime_ms"]))
            if stage["deadline"] != expected_stage:
                problems.append("operation stage fixed deadline differs")
        for attempt in attempts:
            ceiling = stage["deadline"] if attempt["subject"]["kind"] == "stage" else reconciliation
            if ceiling is None:
                problems.append("operation attempt stage has no declared clock")
                continue
            expected_attempt = min(
                ceiling, _after(attempt["started_at"], limits["attempt_lifetime_ms"])
            )
            if attempt["deadline"] != expected_attempt:
                problems.append("operation attempt fixed deadline differs")
    except OverflowError:
        problems.append("operation deadline arithmetic is unrepresentable")
    start, terminal = timing["started_at"], timing["terminal_at"]
    if start is not None and not timing["admitted_at"] <= start < timing["work_deadline"]:
        problems.append("operation start falls outside work admission")
    if terminal is not None and terminal < (start or timing["admitted_at"]):
        problems.append("operation terminal observation precedes admission or start")
    if stage["started_at"] is not None and (
        start is None or not start <= stage["started_at"] < stage["deadline"]
    ):
        problems.append("operation stage start interval differs")
    if stage["finished_at"] is not None and (
        stage["finished_at"] < (stage["started_at"] or timing["admitted_at"])
        or stage["finished_at"] > timing["work_deadline"]
        or (terminal is not None and stage["finished_at"] > terminal)
        or (stage["deadline"] is not None and stage["finished_at"] > stage["deadline"])
    ):
        problems.append("operation stage finish interval differs")
    if (
        operation["state"] == "succeeded"
        and terminal is not None
        and terminal > timing["work_deadline"]
    ):
        problems.append("operation successful terminalization exceeds fixed work deadline")
    if (
        terminal is not None
        and operation["state"] != "succeeded"
        and terminal > timing["work_deadline"]
    ):
        problems.append("operation terminalization requires unavailable late observation evidence")
    if (
        cancellation["kind"] == "requested"
        and stage["started_at"] is not None
        and stage["started_at"] > cancellation["requested_at"]
    ):
        problems.append("operation stage dispatch follows cancellation")
    if cancellation_deadline is not None and stage["finished_at"] is not None:
        if stage["finished_at"] > cancellation_deadline:
            problems.append("operation business stage exceeds cancellation observation budget")
        if stage["state"] == "cancelled" and stage["finished_at"] < cancellation["requested_at"]:
            problems.append("operation cancelled stage precedes winning cancellation")
    if (
        operation["state"] == "cancelled"
        and cancellation_deadline is not None
        and terminal is not None
        and terminal > cancellation_deadline
    ):
        problems.append(
            "operation cancelled terminalization exceeds cancellation observation budget"
        )
    stage_attempts = [attempt for attempt in attempts if attempt["subject"]["kind"] == "stage"]
    if cancellation["kind"] == "none" and (
        stage["state"] == "cancelled"
        or any(attempt["state"] == "cancelled" for attempt in stage_attempts)
    ):
        problems.append("operation cancelled business work lacks winning cancellation")
    if terminal is not None and (
        stage["state"] not in ("succeeded", "failed", "skipped", "cancelled")
        or any(attempt["state"] == "running" for attempt in stage_attempts)
    ):
        problems.append("operation terminal resource retains unfinished business work")
    if any(attempt["subject"]["stage_id"] != stage["stage_id"] for attempt in stage_attempts):
        problems.append("operation attempt names another stage")
    if len({attempt["attempt_id"] for attempt in attempts}) != len(attempts) or [
        attempt["ordinal"] for attempt in attempts
    ] != list(range(1, len(attempts) + 1)):
        problems.append("operation attempt identities or ordinals differ")
    if any(
        preceding["started_at"] > following["started_at"]
        for preceding, following in zip(attempts, attempts[1:], strict=False)
    ):
        problems.append("operation attempt counters contradict admission chronology")
    if stage["attempts_started"] != len(stage_attempts) or stage["last_attempt_id"] != (
        stage_attempts[-1]["attempt_id"] if stage_attempts else None
    ):
        problems.append("operation stage attempt ledger differs")
    if stage["state"] == "succeeded" and (
        not stage_attempts or stage_attempts[-1]["state"] != "succeeded"
    ):
        problems.append("operation accepted stage has no successful final attempt")
    if stage["state"] == "running" and (
        not stage_attempts or stage_attempts[-1]["state"] != "running"
    ):
        problems.append("operation running stage lacks a running attempt")
    for previous, following in zip(stage_attempts, stage_attempts[1:], strict=False):
        if previous["finished_at"] is None or following["started_at"] < previous["finished_at"]:
            problems.append("operation stage attempt intervals overlap")
    if stage["state"] == "retry_wait" and (
        not stage_attempts
        or stage_attempts[-1]["state"] != "failed"
        or stage_attempts[-1]["failure"]["class"] != "transient"
    ):
        problems.append("operation retry wait lacks a finished transient failed attempt")
    if stage["state"] in ("succeeded", "failed", "cancelled") and any(
        attempt["state"] == "running" for attempt in stage_attempts
    ):
        problems.append("operation terminal stage retains running work")
    for attempt in attempts:
        if not (start or timing["admitted_at"]) <= attempt["started_at"] < attempt["deadline"]:
            problems.append("operation attempt start interval differs")
        if attempt["subject"]["kind"] == "stage":
            if stage["started_at"] is None or attempt["started_at"] < stage["started_at"]:
                problems.append("operation attempt precedes its declared stage")
            if stage["finished_at"] is not None and (
                attempt["finished_at"] is None or attempt["finished_at"] > stage["finished_at"]
            ):
                problems.append("operation attempt outlives its declared stage")
            if (
                cancellation["kind"] == "requested"
                and attempt["started_at"] > cancellation["requested_at"]
            ):
                problems.append("operation attempt dispatch follows cancellation")
            if cancellation_deadline is not None and attempt["finished_at"] is not None:
                if attempt["finished_at"] > cancellation_deadline:
                    problems.append(
                        "operation business attempt exceeds cancellation observation budget"
                    )
                if (
                    attempt["state"] == "cancelled"
                    and attempt["finished_at"] < cancellation["requested_at"]
                ):
                    problems.append("operation cancelled attempt precedes winning cancellation")
        finish = attempt["finished_at"]
        if finish is not None and (
            finish < attempt["started_at"]
            or finish > attempt["deadline"]
            or (
                attempt["subject"]["kind"] == "stage" and terminal is not None and finish > terminal
            )
        ):
            problems.append("operation attempt finish interval differs")
    if stage["progress"]["kind"] == "counted":
        problems.append("operation counted progress lacks independently selected unit and total")
        if stage["progress"]["completed"] > stage["progress"]["total"]:
            problems.append("operation progress exceeds its declared total")
    cancellation = operation["control"]["cancellation"]
    if cancellation["kind"] == "requested" and (
        cancellation["requested_at"] < timing["admitted_at"]
        or (terminal is not None and cancellation["requested_at"] > terminal)
    ):
        problems.append("operation cancellation observation interval differs")
    return problems


def _references_belong(value: Any, workspace: dict[str, Any]) -> bool:
    if isinstance(value, dict):
        mapping = cast("dict[str, Any]", value)
        if any(
            key in mapping and mapping[key] is not None and mapping[key] != workspace[key]
            for key in ("organization_id", "domain_id")
        ):
            return False
        return all(_references_belong(item, workspace) for item in mapping.values())
    return True


def _ledger_problems(
    operation: dict[str, Any],
    workspace: dict[str, Any],
    originating_execution: dict[str, Any],
    admitted_at: str,
) -> list[str]:
    """Join every declared ledger record; qualify external input/receipt bytes separately."""
    problems: list[str] = []
    execution = operation["execution"]
    stages = {stage["stage_id"]: stage for stage in execution["stages"]}
    attempts = {attempt["attempt_id"]: attempt for attempt in execution["attempts"]}
    effects, exposures = operation["effects"]["entries"], operation["exposures"]["entries"]
    audits = [attempt["execution"]["authority_audit_event_id"] for attempt in attempts.values()]
    if (
        len(set(audits)) != len(audits)
        or originating_execution["authority_audit_event_id"] in audits
    ):
        problems.append("operation worker audit identities alias prior execution")
    if _principal_kinds_conflict(
        [originating_execution, *(attempt["execution"] for attempt in attempts.values())]
    ):
        problems.append("operation principal identity has inconsistent kind")
    if len({effect["effect_id"] for effect in effects}) != len(effects):
        problems.append("operation effect identities are not distinct")
    if len({exposure["exposure_id"] for exposure in exposures}) != len(exposures):
        problems.append("operation exposure identities are not distinct")
    for effect in effects:
        if not _references_belong(effect["value"], workspace):
            problems.append("operation effect references belong to another workspace")
        if effect["committed_at"] < admitted_at:
            problems.append("operation effect precedes admission")
        stage_id = effect["stage_id"]
        stage = stages.get(stage_id)
        if stage is None:
            problems.append("operation effect names an absent stage")
            continue
        matching = [
            attempt
            for attempt in attempts.values()
            if attempt["subject"] == {"kind": "stage", "stage_id": stage_id}
            and attempt["execution"]["authority_audit_event_id"]
            == effect["authority_audit_event_id"]
        ]
        if not matching or not any(
            attempt["started_at"]
            <= effect["committed_at"]
            <= (attempt["finished_at"] or attempt["deadline"])
            for attempt in matching
        ):
            problems.append("operation effect lacks matching worker attribution or interval")
        if (
            stage["started_at"] is None
            or effect["committed_at"] < stage["started_at"]
            or (stage["finished_at"] is not None and effect["committed_at"] > stage["finished_at"])
        ):
            problems.append("operation effect falls outside its declared stage")
    by_exposure = {exposure["exposure_id"]: exposure for exposure in exposures}
    for attempt in attempts.values():
        if (
            attempt["subject"]["kind"] == "exposure_reconciliation"
            and attempt["subject"]["exposure_id"] not in by_exposure
        ):
            problems.append("operation reconciliation names an absent exposure")
    receipts: dict[str, str] = {}
    for exposure in exposures:
        if not _references_belong(exposure, workspace):
            problems.append("operation exposure references belong to another workspace")
        stage, attempt = stages.get(exposure["stage_id"]), attempts.get(exposure["attempt_id"])
        if (
            stage is None
            or attempt is None
            or attempt["subject"] != {"kind": "stage", "stage_id": exposure["stage_id"]}
        ):
            problems.append("operation exposure stage or attempt joins differ")
        else:
            if exposure["state"] == "attempted" and attempt["finished_at"] is not None:
                problems.append(
                    "operation exposure retains active observation after its worker finished"
                )
            if exposure["state"] == "uncertain" and exposure["finished_at"] > (
                attempt["finished_at"] or attempt["deadline"]
            ):
                problems.append("operation exposure local observation outlives its worker")
            if (
                exposure["authority_audit_event_id"]
                != attempt["execution"]["authority_audit_event_id"]
            ):
                problems.append("operation exposure worker attribution differs")
            if not attempt["started_at"] <= exposure["admitted_at"] < attempt["deadline"] or (
                attempt["finished_at"] is not None
                and exposure["admitted_at"] > attempt["finished_at"]
            ):
                problems.append("operation exposure admission falls outside its attempt")
        if exposure["admitted_at"] < admitted_at or (
            exposure["finished_at"] is not None
            and exposure["finished_at"] < exposure["admitted_at"]
        ):
            problems.append("operation exposure observation interval differs")
        if (
            exposure["finished_at"] is not None
            and exposure["finished_at"] > execution["timing"]["reconciliation_deadline"]
        ):
            problems.append("operation exposure requires unavailable late confirmation evidence")
        cancellation = operation["control"]["cancellation"]
        if (
            cancellation["kind"] == "requested"
            and exposure["admitted_at"] > cancellation["requested_at"]
        ):
            problems.append("operation exposure dispatch follows cancellation")
        receipt = exposure["receipt"]
        if receipt["kind"] == "retained":
            if (
                receipt["receipt_id"] in receipts
                and receipts[receipt["receipt_id"]] != receipt["receipt_sha256"]
            ):
                problems.append("operation exposure receipt identity has different bytes")
            receipts[receipt["receipt_id"]] = receipt["receipt_sha256"]
        reconciliation = exposure["reconciliation"]
        matches = [
            attempt
            for attempt in attempts.values()
            if attempt["subject"]
            == {"kind": "exposure_reconciliation", "exposure_id": exposure["exposure_id"]}
        ]
        if len(matches) != reconciliation["attempts_started"]:
            problems.append("operation exposure reconciliation counter differs")
        producer = attempts.get(exposure["attempt_id"])
        if producer is not None and any(
            probe["ordinal"] <= producer["ordinal"] for probe in matches
        ):
            problems.append("operation reconciliation counter precedes its exposure producer")
        if (
            reconciliation["deadline"] is not None
            and reconciliation["deadline"] != execution["timing"]["reconciliation_deadline"]
        ):
            problems.append("operation exposure reconciliation fixed deadline differs")
        checked, next_at = reconciliation["last_checked_at"], reconciliation["next_attempt_at"]
        observation_end = exposure["finished_at"] or exposure["admitted_at"]
        if (
            checked is not None
            and not observation_end <= checked <= execution["timing"]["reconciliation_deadline"]
            or (
                next_at is not None
                and not max(checked or observation_end, observation_end)
                <= next_at
                < reconciliation["deadline"]
            )
        ):
            problems.append("operation exposure reconciliation observation interval differs")
        if any(
            attempt["started_at"] < observation_end
            or (
                attempt["finished_at"] is not None
                and (checked is None or attempt["finished_at"] > checked)
            )
            for attempt in matches
        ):
            problems.append("operation exposure reconciliation attempt interval differs")
        if reconciliation["state"] in ("resolved", "unresolved") and any(
            attempt["state"] == "running" for attempt in matches
        ):
            problems.append("operation resolved reconciliation retains a running probe")
        if operation["state"] == "succeeded" and (
            exposure["settlement"] != "settled"
            or exposure["state"] not in ("completed", "confirmed_not_performed")
            or exposure["finished_at"] > execution["timing"]["terminal_at"]
            or (checked is not None and checked > execution["timing"]["terminal_at"])
        ):
            problems.append("operation successful exposure is not settled before terminalization")
    return problems


def _retained_output_problems(
    operation: dict[str, Any],
    input_digest: str,
    profile_digest: str,
) -> list[str]:
    problems: list[str] = []
    execution = operation["execution"]
    stage = execution["stages"][0]
    outputs = execution["stage_results"]
    accepted = stage["accepted_result"]
    effects = [
        effect
        for effect in operation["effects"]["entries"]
        if effect["value"]["kind"] == "export_artifact_bound"
    ]
    if accepted is None:
        if effects:
            problems.append("operation artifact effect has no accepted output")
        return problems
    output = outputs[0]
    reference = {key: operation[key] for key in ("organization_id", "domain_id", "operation_id")}
    expected = {
        **reference,
        "stage_id": stage["stage_id"],
        "stage_result_id": output["stage_result_id"],
        "input_sha256": input_digest,
        "profile_sha256": profile_digest,
        "result_sha256": _digest(output),
        "kind": "export_artifact",
    }
    if accepted != expected:
        problems.append("operation accepted result reference differs")
    if (
        output["operation"] != reference
        or output["stage_id"] != stage["stage_id"]
        or output["input_sha256"] != input_digest
        or output["profile_sha256"] != profile_digest
        or not _scope(output["artifact"], reference)
    ):
        problems.append("operation complete stage-result joins differ")
    if len(effects) != 1 or any(
        effect["stage_id"] != stage["stage_id"] or effect["value"]["artifact"] != output["artifact"]
        for effect in effects
    ):
        problems.append("operation artifact effect differs from accepted output")
    stage_attempts = [
        attempt for attempt in execution["attempts"] if attempt["subject"]["kind"] == "stage"
    ]
    audit = stage_attempts[-1]["execution"]["authority_audit_event_id"] if stage_attempts else None
    if audit is None:
        problems.append("operation accepted output lacks worker attribution")
    for effect in effects:
        if effect["authority_audit_event_id"] != audit:
            problems.append("operation artifact effect attribution differs")
        if not stage["started_at"] <= effect["committed_at"] <= stage["finished_at"]:
            problems.append("operation artifact effect commit interval differs")
        cancellation = operation["control"]["cancellation"]
        if (
            cancellation["kind"] == "requested"
            and effect["committed_at"] > cancellation["requested_at"]
        ):
            problems.append("operation artifact bind follows cancellation")
    if operation["state"] == "succeeded" and (
        operation["result"]["command"] != operation["command"]
        or operation["result"]["artifact"] != output["artifact"]
    ):
        problems.append("operation successful result differs from retained output")
    if operation["state"] == "succeeded":
        problems.extend(
            _input_subject_problems(
                operation["command"],
                operation["input"],
                operation["context"],
                operation["result"]["subject"],
            )
        )
    return problems


def _output_problems(
    operation: dict[str, Any],
    core: dict[str, Any],
    source: dict[str, Any],
    input_digest: str,
    artifact: dict[str, Any] | None,
) -> list[str]:
    problems = _retained_output_problems(operation, input_digest, core["profile"]["content_sha256"])
    if operation["execution"]["stages"][0]["accepted_result"] is None:
        return problems
    output = operation["execution"]["stage_results"][0]
    reference = core["operation"]
    effects = [
        effect
        for effect in operation["effects"]["entries"]
        if effect["value"]["kind"] == "export_artifact_bound"
    ]
    if artifact is None:
        problems.append("operation complete retained artifact is unavailable")
    else:
        artifact_core = artifact["core"]
        if (
            artifact["reference"] != output["artifact"]
            or artifact["reference"]["artifact_record_sha256"] != _digest(artifact_core)
            or not _scope(artifact_core, reference)
            or artifact_core["subject"] != source
            or artifact_core["format"] != operation["input"]["format"]
            or artifact_core["export_id"] != output["artifact"]["export_id"]
            or not _scope(artifact["control"]["subject"], reference)
            or artifact["control"]["subject"]["export_id"] != artifact_core["export_id"]
            or artifact["control"]["last_event"]["organization_id"] != reference["organization_id"]
        ):
            problems.append("operation retained artifact differs from selected output")
        if any(effect["committed_at"] < artifact_core["sealed_at"] for effect in effects):
            problems.append("operation artifact bind precedes sealing")
    if operation["state"] == "succeeded" and operation["result"]["subject"] != source:
        problems.append("operation successful result differs from retained output")
    return problems


def operation_read_problems(
    selector: dict[str, Any],
    response: dict[str, Any],
    original_request: dict[str, Any],
    plan: dict[str, Any],
    profile: dict[str, Any],
    admission: dict[str, Any],
    artifact: dict[str, Any] | None,
) -> list[str]:
    """Compare an exact current read with the separately supplied original admission."""
    operation = response["result"]["operation"]
    problems = operation_problems(original_request, operation, plan, profile, admission, artifact)
    if selector["body"]["operation_id"] != operation["operation_id"] or not _scope(
        operation, selector["metadata"]["context"]
    ):
        problems.append("operation read selector differs")
    problems.extend(
        receipt_problems(
            original_request,
            response["result"]["idempotency_receipt"],
            profile,
            operation["execution"]["timing"]["terminal_at"],
            replayed=False,
        )
    )
    return problems


def plan_read_problems(
    selector: dict[str, Any], response: dict[str, Any], operation: dict[str, Any]
) -> list[str]:
    """Check the full exact retained plan, without authorizing its disclosure."""
    problems = _canonical_problems(selector, response, operation)
    if problems:
        return problems
    plan = response["result"]["plan"]
    core = plan["core"]
    expected = {
        "organization_id": core["operation"]["organization_id"],
        "domain_id": core["operation"]["domain_id"],
        "plan_id": core["plan_id"],
        "plan_sha256": plan["plan_sha256"],
    }
    if selector["body"]["plan_id"] != core["plan_id"] or not _scope(
        core["context"], selector["metadata"]["context"]
    ):
        problems.append("operation plan read selector differs")
    if operation["execution"]["plan"] != expected or plan["plan_sha256"] != _digest(core):
        problems.append("operation plan read identity or digest differs")
    if any(core["operation"][key] != operation[key] for key in core["operation"]):
        problems.append("operation plan read names another operation")
    if (
        core["command"] != operation["command"]
        or core["context"] != operation["context"]
        or core["input_sha256"] != _digest(operation["input"])
        or any(
            core["stages"][0][key] != operation["execution"]["stages"][0][key]
            for key in ("stage_id", "ordinal", "kind", "required")
        )
    ):
        problems.append("operation plan read immutable admission differs")
    if not _retained_cause_consistent(operation):
        problems.append("operation plan read retained causal attribution differs")
    declaration = core["stages"][0]
    if declaration["input_sha256"] != core["input_sha256"]:
        problems.append("operation plan read stage input differs from complete input")
    if declaration["profile"] != core["profile"]:
        problems.append("operation plan read stage profile differs from selected profile")
    if not _scope(core["profile"], core["context"]):
        problems.append("operation plan read selected profile belongs to another workspace")
    accepted = operation["execution"]["stages"][0]["accepted_result"]
    if accepted is not None and (
        accepted["input_sha256"] != core["input_sha256"]
        or accepted["profile_sha256"] != core["profile"]["content_sha256"]
        or accepted["kind"] != declaration["output_kind"]
        or any(
            output["input_sha256"] != core["input_sha256"]
            or output["profile_sha256"] != core["profile"]["content_sha256"]
            for output in operation["execution"]["stage_results"]
        )
    ):
        problems.append("operation plan read declaration differs from retained output")
    problems.extend(
        _execution_problems(
            operation,
            core["limits"],
            operation["causal_command"]["admitted_at"],
            operation["execution"]["stages"][0],
        )
    )
    problems.extend(
        _retained_output_problems(
            operation, core["input_sha256"], core["profile"]["content_sha256"]
        )
    )
    problems.extend(
        _ledger_problems(
            operation,
            operation["context"],
            operation["execution_attribution"],
            operation["causal_command"]["admitted_at"],
        )
    )
    if operation["state"] != "succeeded":
        problems.extend(
            _input_subject_problems(
                operation["command"], operation["input"], operation["context"], None
            )
        )
    return problems


def command_exchange_problems(
    request: dict[str, Any],
    exchange: dict[str, Any],
    plan: dict[str, Any],
    profile: dict[str, Any],
    admission: dict[str, Any],
    artifact: dict[str, Any] | None,
    *,
    replayed: bool,
) -> list[str]:
    """Compare initial admission versus authorized replay without inventing dispatch."""
    body = exchange["body"]
    if exchange["status"] == 202:
        operation, receipt = body["operation"], body["idempotency_receipt"]
    else:
        operation, receipt = body["result"]["operation"], body["result"]["idempotency_receipt"]
    problems = operation_problems(request, operation, plan, profile, admission, artifact)
    if exchange["status"] != (200 if replayed else 202):
        problems.append("operation command HTTP disposition differs")
    if exchange["status"] == 202 and exchange["headers"]["Location"] != (
        "/v1/operations/" + admission["operation"]["operation_id"]
    ):
        problems.append("operation admission Location differs")
    problems.extend(
        receipt_problems(
            request,
            receipt,
            profile,
            operation["execution"]["timing"]["terminal_at"],
            replayed=replayed,
        )
    )
    return problems


def cancellation_problems(
    request: dict[str, Any],
    response: dict[str, Any],
    prior: dict[str, Any],
    profile: dict[str, Any],
    decision: dict[str, Any] | None,
    *,
    replayed: bool,
) -> list[str]:
    """Check the own cancellation record/receipt and preserved target snapshot.

    Prior is the independently retained pre-decision operation, not reconstructed
    from a cancellation result. Current target authority and the canonical race
    fence remain producer obligations. Replay compares the original decision.
    """
    problems = _canonical_problems(request, response, prior, profile, decision)
    if problems:
        return problems
    if decision is None:
        return ["cancellation own qualified decision inputs are unavailable"]
    body, result = request["body"], response["result"]
    target, record = result["operation"], result["cancellation"]
    core = record["core"]
    reference = {key: prior[key] for key in ("organization_id", "domain_id", "operation_id")}
    if body["operation"] != reference or core["operation"] != reference:
        problems.append("cancellation names another target")
    if request["metadata"]["context"] != core["context"] or not _scope(reference, core["context"]):
        problems.append("cancellation context differs")
    expected_control = prior["control"]["control_revision"]
    if (
        body["expected_control_revision"] != expected_control
        or core["expected_control_revision"] != expected_control
    ):
        problems.append("cancellation prior control precondition differs")
    disposition = (
        "already_terminal"
        if prior["state"] in ("succeeded", "failed", "cancelled")
        else "already_requested"
        if prior["control"]["cancellation"]["kind"] == "requested"
        else "requested"
    )
    if core["disposition"] != disposition or core["reason"] != body["reason"]:
        problems.append("cancellation disposition or reason differs")
    if record["cancellation_sha256"] != _digest(core):
        problems.append("cancellation complete record digest differs")
    winning = prior["control"]["cancellation"]
    if winning["kind"] == "requested" and core["cancellation_id"] == winning["cancellation_id"]:
        problems.append("cancellation own record aliases the retained winning cancellation")
    if core["execution"]["command_id"] == prior["execution_attribution"]["command_id"]:
        problems.append("cancellation aliases the originating command")
    if any(
        core[key] != decision[key]
        for key in ("context", "execution", "recorded_at", "operation", "expected_control_revision")
    ):
        problems.append("cancellation differs from independently supplied own decision")
    prior_audits = {prior["execution_attribution"]["authority_audit_event_id"]}
    prior_audits.update(
        attempt["execution"]["authority_audit_event_id"]
        for attempt in prior["execution"]["attempts"]
    )
    if core["execution"]["authority_audit_event_id"] in prior_audits:
        problems.append("cancellation aliases prior command or worker audit")
    if _principal_kinds_conflict(
        [
            prior["execution_attribution"],
            prior["causal_command"]["execution"],
            *(attempt["execution"] for attempt in prior["execution"]["attempts"]),
            core["execution"],
        ]
    ):
        problems.append("cancellation principal identity has inconsistent kind")
    recorded = core["recorded_at"]
    if recorded < prior["execution"]["timing"]["admitted_at"]:
        problems.append("cancellation observation precedes admission")
    known = [prior["execution"]["timing"]["admitted_at"]]
    known.extend(prior["execution"]["timing"][key] for key in ("started_at", "terminal_at"))
    for entry in [*prior["execution"]["stages"], *prior["execution"]["attempts"]]:
        known.extend(entry[key] for key in ("started_at", "finished_at"))
    known.extend(effect["committed_at"] for effect in prior["effects"]["entries"])
    for exposure in prior["exposures"]["entries"]:
        known.extend(
            (
                exposure["admitted_at"],
                exposure["finished_at"],
                exposure["reconciliation"]["last_checked_at"],
            )
        )
    if prior["control"]["cancellation"]["kind"] == "requested":
        known.append(prior["control"]["cancellation"]["requested_at"])
    if recorded < max(value for value in known if value is not None):
        problems.append("cancellation observes future prior events")
    expected_target = {**prior}
    if disposition == "requested":
        expected_target["control"] = {
            **prior["control"],
            "control_revision": {
                "kind": "control",
                "value": str(int(expected_control["value"]) + 1),
            },
            "work_admission": "closed",
            "cancellation": {
                "kind": "requested",
                "cancellation_id": core["cancellation_id"],
                "reason": body["reason"],
                "requested_at": recorded,
            },
        }
        if target["state"] == "cancelled":
            # Immediate cancellation of a queued target shares the one increment.
            if (
                prior["state"] != "queued"
                or target["execution"]["timing"]["terminal_at"] != recorded
            ):
                problems.append("cancellation immediate terminal observation differs")
            expected_target["state"] = "cancelled"
            expected_target["execution"] = {
                **prior["execution"],
                "timing": {**prior["execution"]["timing"], "terminal_at": recorded},
                "stages": [
                    {
                        **stage,
                        "state": "cancelled",
                        "finished_at": recorded,
                        "diagnostic": "cancel_requested",
                    }
                    for stage in prior["execution"]["stages"]
                ],
            }
    if target != expected_target:
        problems.append("cancellation changed the preserved target snapshot")
    if (
        core["observed_state"] != target["state"]
        or core["observed_control_revision"] != target["control"]["control_revision"]
    ):
        problems.append("cancellation observed target joins differ")
    problems.extend(
        _execution_problems(
            target,
            profile["limits"],
            prior["causal_command"]["admitted_at"],
            target["execution"]["stages"][0],
        )
    )
    causal = prior["causal_command"]
    if not _scope(target["execution"]["plan"], causal["context"]):
        problems.append("cancellation retained plan belongs to another workspace")
    if not _retained_cause_consistent(prior):
        problems.append("cancellation retained causal attribution differs")
    if recorded < causal["admitted_at"]:
        problems.append("cancellation decision precedes retained causal admission")
    problems.extend(
        _ledger_problems(target, causal["context"], causal["execution"], causal["admitted_at"])
    )
    problems.extend(_retained_output_problems(target, _digest(prior["input"]), _digest(profile)))
    problems.extend(_profile_scope_problems(profile, causal["context"]))
    if target["state"] != "succeeded":
        problems.extend(
            _input_subject_problems(target["command"], target["input"], causal["context"], None)
        )
    problems.extend(
        receipt_problems(
            request, response["idempotency_receipt"], profile, recorded, replayed=replayed
        )
    )
    return problems
