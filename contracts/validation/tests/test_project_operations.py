"""Conformance against separately reviewed full focused operation literals."""

import copy

import pytest
from project_semantics_support import (
    byte_vector,
    case,
    counter_history_cancellation_negative,
    foreign_exposure,
    operation_arguments,
    operation_inputs,
    rehash_cancellation_negative,
    rehash_plan_input_negative,
    rehash_plan_negative,
    rehash_stage_result_negative,
    reversed_counter_history_negative,
    same_workspace_exposure_negative,
    validate_target,
)

from arboresce_contract_validation.project_exports import verify_artifact
from arboresce_contract_validation.project_operations import (
    cancellation_problems,
    command_exchange_problems,
    operation_problems,
    operation_read_problems,
    plan_read_problems,
    receipt_problems,
)

FORMATS = [
    "project-json-v1",
    "project-csv-v1",
    "project-financial-json-v1",
    "project-financial-csv-v1",
]
OTHER_ID = "ffffffff-ffff-4fff-8fff-ffffffffffff"
OTHER_HASH = "f" * 64


OPERATION_TARGETS = [
    "project_export_request",
    "project_export_operation_resource",
    "project_export_operation_plan_record",
    "project_export_operation_profile",
    "project_export_operation_admission_inputs",
    "project_export_artifact_resource",
]


@pytest.mark.parametrize("name", FORMATS)
def test_approved_complete_operation_snapshots(name):
    request, plan, profile, admission, artifact = operation_inputs(name)
    prefix = "async-" + name + "-"
    identities = [
        state + "-project_export_operation_resource"
        for state in (
            "queued",
            "running",
            "succeeded",
            "failed",
            "cancelled",
        )
    ] + [
        "failed-retained-output-resource",
        "cancelled-retained-output-resource",
        "retained-output-cancel-prior",
        "retained-output-cancel-observed",
    ]
    for identity in identities:
        value = case(prefix + identity)
        assert operation_problems(request, value, plan, profile, admission, artifact) == []
    assert verify_artifact(artifact, byte_vector(name)[1]) == byte_vector(name)[2]


@pytest.mark.parametrize("name", FORMATS)
def test_approved_reads_admission_and_replay(name):
    request, plan, profile, admission, artifact = operation_inputs(name)
    prefix = "async-" + name + "-"
    selector = case(prefix + "project_export_operation_get_request")
    validate_target(selector, "project_export_operation_get_request")
    for state in ("queued", "running", "succeeded", "failed", "cancelled"):
        response = case(prefix + state + "-project_export_operation_read_response")
        assert (
            operation_read_problems(selector, response, request, plan, profile, admission, artifact)
            == []
        )
        exchange = case(prefix + state + "-replay-project_export_operation_command_exchange")
        assert (
            command_exchange_problems(
                request, exchange, plan, profile, admission, artifact, replayed=True
            )
            == []
        )
    for state in ("queued", "running"):
        exchange = case(prefix + state + "-initial-project_export_operation_command_exchange")
        assert (
            command_exchange_problems(
                request, exchange, plan, profile, admission, artifact, replayed=False
            )
            == []
        )
    for state in ("failed", "cancelled"):
        response = case(prefix + state + "-retained-output-read")
        assert (
            operation_read_problems(selector, response, request, plan, profile, admission, artifact)
            == []
        )
        exchange = case(prefix + state + "-retained-output-replay")
        assert (
            command_exchange_problems(
                request, exchange, plan, profile, admission, artifact, replayed=True
            )
            == []
        )


@pytest.mark.parametrize("name", FORMATS)
def test_approved_exact_plan_reads(name):
    prefix = "async-" + name + "-"
    selector = case(prefix + "project_export_operation_plan_get_request")
    response = case(prefix + "project_export_operation_plan_get_response")
    operation = case(prefix + "succeeded-project_export_operation_resource")
    validate_target(selector, "project_export_operation_plan_get_request")
    validate_target(response, "project_export_operation_plan_get_response")
    assert plan_read_problems(selector, response, operation) == []


@pytest.mark.parametrize("name", FORMATS)
def test_approved_complete_cancellation_decisions_and_retained_effects(name):
    profile = operation_inputs(name)[2]
    prefix = "async-" + name + "-"
    prior = {
        "requested": case(prefix + "running-project_export_operation_resource"),
        "already_requested": case(prefix + "cancel-requested-response")["result"]["operation"],
        "already_terminal": case(prefix + "succeeded-project_export_operation_resource"),
    }
    for disposition, before in prior.items():
        request = case(prefix + "cancel-" + disposition + "-request")
        response = case(prefix + "cancel-" + disposition + "-response")
        replay = case(prefix + "cancel-" + disposition + "-replay")["body"]
        assert (
            cancellation_problems(
                request,
                response,
                before,
                profile,
                case(prefix + "cancel-" + disposition + "-decision-inputs"),
                replayed=False,
            )
            == []
        )
        assert (
            cancellation_problems(
                request,
                replay,
                before,
                profile,
                case(prefix + "cancel-" + disposition + "-decision-inputs"),
                replayed=True,
            )
            == []
        )
    before = case(prefix + "retained-output-cancel-prior")
    request = case(prefix + "retained-output-cancel-request")
    response = case(prefix + "retained-output-cancel-response")
    replay = case(prefix + "retained-output-cancel-replay")["body"]
    assert (
        cancellation_problems(
            request,
            response,
            before,
            profile,
            case(prefix + "retained-output-cancel-decision-inputs"),
            replayed=False,
        )
        == []
    )
    assert (
        cancellation_problems(
            request,
            replay,
            before,
            profile,
            case(prefix + "retained-output-cancel-decision-inputs"),
            replayed=True,
        )
        == []
    )


@pytest.mark.parametrize(
    ("index", "path", "replacement", "diagnostic"),
    [
        (1, ("operation_id",), OTHER_ID, "operation identity differs from admission"),
        (1, ("input", "project_id"), OTHER_ID, "operation command or complete input differs"),
        (1, ("context", "organization_id"), OTHER_ID, "operation context differs from admission"),
        (
            1,
            ("causal_command", "execution", "command_id"),
            OTHER_ID,
            "operation historical attribution differs",
        ),
        (
            1,
            ("execution", "plan", "plan_id"),
            OTHER_ID,
            "operation complete plan identity or digest differs",
        ),
        (2, ("plan_sha256",), OTHER_HASH, "operation complete plan identity or digest differs"),
        (
            2,
            ("core", "operation", "operation_id"),
            OTHER_ID,
            "operation plan admission joins differ",
        ),
        (
            2,
            ("core", "profile", "domain_id"),
            OTHER_ID,
            "operation selected profile or limits differ",
        ),
        (
            2,
            ("core", "limits", "max_canonical_effects"),
            1,
            "operation selected profile or limits differ",
        ),
        (3, ("limits", "operation_lifetime_ms"), 61000, "operation fixed deadlines differ"),
        (
            3,
            ("lifecycle_policy", "domain_id"),
            OTHER_ID,
            "operation selected delivery or policy scope differs",
        ),
        (
            3,
            ("delivery_profile", "organization_id"),
            OTHER_ID,
            "operation selected delivery or policy scope differs",
        ),
        (
            4,
            ("source", "project", "content_revision", "value"),
            "2",
            "operation source differs from selected project revision",
        ),
        (
            4,
            ("source", "observed_control_revision", "value"),
            "3",
            "operation source control differs",
        ),
        (4, ("execution", "command_id"), OTHER_ID, "operation historical attribution differs"),
        (2, ("core", "input_sha256"), OTHER_HASH, "operation complete domain-input digest differs"),
        (
            2,
            ("core", "stages", 0, "profile", "content_sha256"),
            OTHER_HASH,
            "operation stage declaration differs from selected profile",
        ),
        (
            1,
            ("execution", "stages", 0, "stage_id"),
            OTHER_ID,
            "operation stage differs from immutable plan",
        ),
        (
            1,
            ("execution", "timing", "work_deadline"),
            "2026-10-03T12:06:00.001Z",
            "operation fixed deadlines differ",
        ),
        (
            1,
            ("execution", "timing", "started_at"),
            "2026-10-03T12:04:59.999Z",
            "operation start falls outside work admission",
        ),
        (
            1,
            ("execution", "stages", 0, "deadline"),
            "2026-10-03T12:05:30.101Z",
            "operation stage fixed deadline differs",
        ),
        (
            1,
            ("execution", "stages", 0, "finished_at"),
            "2026-10-03T12:05:00.000Z",
            "operation stage finish interval differs",
        ),
        (
            1,
            ("execution", "attempts", 0, "deadline"),
            "2026-10-03T12:05:20.101Z",
            "operation attempt fixed deadline differs",
        ),
        (
            1,
            ("execution", "attempts", 0, "subject", "stage_id"),
            OTHER_ID,
            "operation attempt names another stage",
        ),
        (
            1,
            ("execution", "attempts", 0, "ordinal"),
            2,
            "operation attempt identities or ordinals differ",
        ),
        (
            1,
            ("execution", "stages", 0, "attempts_started"),
            2,
            "operation stage attempt ledger differs",
        ),
        (
            1,
            ("execution", "stages", 0, "last_attempt_id"),
            OTHER_ID,
            "operation stage attempt ledger differs",
        ),
        (
            1,
            ("execution", "attempts", 0, "started_at"),
            "2026-10-03T12:05:20.200Z",
            "operation attempt start interval differs",
        ),
        (
            1,
            ("execution", "attempts", 0, "finished_at"),
            "2026-10-03T12:05:20.200Z",
            "operation attempt finish interval differs",
        ),
        (
            1,
            ("execution", "stages", 0, "accepted_result", "result_sha256"),
            OTHER_HASH,
            "operation accepted result reference differs",
        ),
        (
            1,
            ("execution", "stage_results", 0, "operation", "domain_id"),
            OTHER_ID,
            "operation complete stage-result joins differ",
        ),
        (
            1,
            ("execution", "stage_results", 0, "artifact", "domain_id"),
            OTHER_ID,
            "operation complete stage-result joins differ",
        ),
        (
            1,
            ("effects", "entries", 0, "stage_id"),
            OTHER_ID,
            "operation artifact effect differs from accepted output",
        ),
        (
            1,
            ("effects", "entries", 0, "authority_audit_event_id"),
            OTHER_ID,
            "operation artifact effect attribution differs",
        ),
        (
            1,
            ("effects", "entries", 0, "committed_at"),
            "2026-10-03T12:05:01.100Z",
            "operation artifact effect commit interval differs",
        ),
        (
            5,
            ("control", "subject", "export_id"),
            OTHER_ID,
            "operation retained artifact differs from selected output",
        ),
        (
            5,
            ("control", "last_event", "organization_id"),
            OTHER_ID,
            "operation retained artifact differs from selected output",
        ),
        (
            5,
            ("core", "sealed_at"),
            "2026-10-03T12:05:01.100Z",
            "operation artifact bind precedes sealing",
        ),
        (
            1,
            ("result", "subject", "snapshot_id"),
            OTHER_ID,
            "operation successful result differs from retained output",
        ),
    ],
)
def test_shape_valid_cross_record_operation_substitutions(index, path, replacement, diagnostic):
    values = operation_arguments()
    parent = values[index]
    for key in path[:-1]:
        parent = parent[key]
    parent[path[-1]] = replacement
    validate_target(values[index], OPERATION_TARGETS[index])
    problems = operation_problems(*values)
    assert diagnostic in problems
    assert values[0]["metadata"]["idempotency"]["key"] not in "\n".join(problems)


def test_noncanonical_integer_representation_denies_before_hashing():
    values = operation_arguments()
    values[3]["limits"]["max_stage_count"] = 1.0
    validate_target(values[3], "project_export_operation_profile")
    assert operation_problems(*values) == ["operation input violates bounded canonical subset"]


def test_complete_artifact_unavailable_is_not_an_id_only_success():
    values = operation_arguments()
    values[5] = None
    assert "operation complete retained artifact is unavailable" in operation_problems(*values)


def test_another_approved_format_cannot_replace_exact_retained_artifact():
    values = operation_arguments()
    values[5] = operation_inputs("project-csv-v1")[4]
    assert "operation retained artifact differs from selected output" in operation_problems(*values)


@pytest.mark.parametrize(
    ("field", "value", "diagnostic"),
    [
        ("key_digest", OTHER_HASH, "receipt key digest differs"),
        ("request_fingerprint", OTHER_HASH, "receipt request fingerprint differs"),
        ("replayed", True, "receipt replay disposition differs"),
        ("expires_at", "2026-10-10T12:05:01.001Z", "receipt terminal-relative deadlines differ"),
        (
            "tombstone_expires_at",
            "2026-11-02T12:05:01.001Z",
            "receipt terminal-relative deadlines differ",
        ),
    ],
)
def test_receipt_original_identity_and_fixed_dates(field, value, diagnostic):
    request, _, profile, _, _ = operation_inputs("project-json-v1")
    response = case("async-project-json-v1-succeeded-project_export_operation_read_response")
    response["result"]["idempotency_receipt"][field] = value
    validate_target(response, "project_export_operation_read_response")
    assert diagnostic in receipt_problems(
        request,
        response["result"]["idempotency_receipt"],
        profile,
        "2026-10-03T12:05:01.000Z",
        replayed=False,
    )


def test_receipt_cannot_infer_terminalization_from_a_retained_receipt():
    request, _, profile, _, _ = operation_inputs("project-json-v1")
    receipt = case("async-project-json-v1-succeeded-project_export_operation_read_response")[
        "result"
    ]["idempotency_receipt"]
    assert "receipt terminal-relative deadlines differ" in receipt_problems(
        request, receipt, profile, None, replayed=False
    )


def test_receipt_retention_order_and_unrepresentable_clock_deny():
    request, _, profile, _, _ = operation_inputs("project-json-v1")
    receipt = case("async-project-json-v1-succeeded-project_export_operation_read_response")[
        "result"
    ]["idempotency_receipt"]
    profile["retention"]["replay_retention_ms"] = profile["retention"]["diagnostic_retention_ms"]
    validate_target(profile, "project_export_operation_profile")
    assert "receipt diagnostic retention is not later than replay" in receipt_problems(
        request, receipt, profile, "2026-10-03T12:05:01.000Z", replayed=False
    )
    assert "receipt deadline arithmetic is unrepresentable" in receipt_problems(
        request, receipt, profile, "9999-12-31T23:59:59.999Z", replayed=False
    )


def test_receipt_rejects_noncanonical_selected_profile():
    request, _, profile, _, _ = operation_inputs("project-json-v1")
    receipt = case("async-project-json-v1-succeeded-project_export_operation_read_response")[
        "result"
    ]["idempotency_receipt"]
    profile["limits"]["max_stage_count"] = 1.0
    assert receipt_problems(request, receipt, profile, None, replayed=False) == [
        "operation input violates bounded canonical subset"
    ]


@pytest.mark.parametrize("field", ["operation_id", "domain_id"])
def test_exact_read_selectors_cannot_switch_target_or_scope(field):
    request, plan, profile, admission, artifact = operation_inputs("project-json-v1")
    selector = case("async-project-json-v1-project_export_operation_get_request")
    response = case("async-project-json-v1-succeeded-project_export_operation_read_response")
    if field == "operation_id":
        selector["body"][field] = OTHER_ID
    else:
        selector["metadata"]["context"][field] = OTHER_ID
    validate_target(selector, "project_export_operation_get_request")
    assert "operation read selector differs" in operation_read_problems(
        selector, response, request, plan, profile, admission, artifact
    )


@pytest.mark.parametrize(
    ("path", "value", "diagnostic"),
    [
        (("core", "plan_id"), OTHER_ID, "operation plan read identity or digest differs"),
        (
            ("core", "operation", "operation_id"),
            OTHER_ID,
            "operation plan read names another operation",
        ),
        (("core", "input_sha256"), OTHER_HASH, "operation plan read immutable admission differs"),
    ],
)
def test_exact_plan_read_cannot_substitute_complete_plan(path, value, diagnostic):
    selector = case("async-project-json-v1-project_export_operation_plan_get_request")
    response = case("async-project-json-v1-project_export_operation_plan_get_response")
    operation = case("async-project-json-v1-succeeded-project_export_operation_resource")
    parent = response["result"]["plan"]
    for key in path[:-1]:
        parent = parent[key]
    parent[path[-1]] = value
    validate_target(response, "project_export_operation_plan_get_response")
    assert diagnostic in plan_read_problems(selector, response, operation)


def test_plan_read_rejects_noncanonical_values():
    selector = case("async-project-json-v1-project_export_operation_plan_get_request")
    response = case("async-project-json-v1-project_export_operation_plan_get_response")
    operation = case("async-project-json-v1-succeeded-project_export_operation_resource")
    response["result"]["plan"]["core"]["limits"]["max_stage_count"] = 1.0
    assert plan_read_problems(selector, response, operation) == [
        "operation input violates bounded canonical subset"
    ]


def test_initial_admission_requires_exact_location_and_http_disposition():
    request, plan, profile, admission, artifact = operation_inputs("project-json-v1")
    exchange = case(
        "async-project-json-v1-queued-initial-project_export_operation_command_exchange"
    )
    exchange["headers"]["Location"] = "/v1/operations/" + OTHER_ID
    validate_target(exchange, "project_export_operation_command_exchange")
    assert "operation admission Location differs" in command_exchange_problems(
        request, exchange, plan, profile, admission, artifact, replayed=False
    )
    assert "operation command HTTP disposition differs" in command_exchange_problems(
        request, exchange, plan, profile, admission, artifact, replayed=True
    )


@pytest.mark.parametrize(
    ("which", "path", "value", "diagnostic"),
    [
        (
            "request",
            ("body", "operation", "operation_id"),
            OTHER_ID,
            "cancellation names another target",
        ),
        ("request", ("metadata", "context", "domain_id"), OTHER_ID, "cancellation context differs"),
        (
            "request",
            ("body", "expected_control_revision", "value"),
            "1",
            "cancellation prior control precondition differs",
        ),
        (
            "request",
            ("body", "reason"),
            "Stop this exact export.",
            "cancellation disposition or reason differs",
        ),
        (
            "record",
            ("cancellation_sha256",),
            OTHER_HASH,
            "cancellation complete record digest differs",
        ),
        (
            "core",
            ("execution", "command_id"),
            "71000000-0000-4000-8000-000000000065",
            "cancellation aliases the originating command",
        ),
        (
            "core",
            ("observed_control_revision", "value"),
            "4",
            "cancellation observed target joins differ",
        ),
        (
            "target",
            ("input", "project_id"),
            OTHER_ID,
            "cancellation changed the preserved target snapshot",
        ),
    ],
)
def test_cancellation_cannot_alias_or_mutate_original_work(which, path, value, diagnostic):
    prefix = "async-project-json-v1-"
    request = case(prefix + "cancel-requested-request")
    response = case(prefix + "cancel-requested-response")
    prior = case(prefix + "running-project_export_operation_resource")
    profile = operation_inputs("project-json-v1")[2]
    values = {
        "request": request,
        "record": response["result"]["cancellation"],
        "core": response["result"]["cancellation"]["core"],
        "target": response["result"]["operation"],
    }
    parent = values[which]
    for key in path[:-1]:
        parent = parent[key]
    parent[path[-1]] = value
    validate_target(request, "project_export_operation_cancel_request")
    validate_target(response, "project_export_operation_cancel_response")
    assert diagnostic in cancellation_problems(
        request,
        response,
        prior,
        profile,
        case(prefix + "cancel-requested-decision-inputs"),
        replayed=False,
    )


def test_cancellation_cannot_hide_a_terminal_or_prior_requested_race():
    prefix = "async-project-json-v1-"
    request = case(prefix + "cancel-requested-request")
    response = case(prefix + "cancel-requested-response")
    profile = operation_inputs("project-json-v1")[2]
    for prior in [
        case(prefix + "succeeded-project_export_operation_resource"),
        case(prefix + "cancel-requested-response")["result"]["operation"],
    ]:
        assert "cancellation disposition or reason differs" in cancellation_problems(
            request,
            response,
            prior,
            profile,
            case(prefix + "cancel-requested-decision-inputs"),
            replayed=False,
        )


def test_cancellation_record_cannot_predate_original_admission():
    prefix = "async-project-json-v1-"
    request = case(prefix + "cancel-requested-request")
    response = case(prefix + "cancel-requested-response")
    prior = case(prefix + "running-project_export_operation_resource")
    prior["execution"]["timing"]["admitted_at"] = "2026-10-03T12:05:00.300Z"
    validate_target(prior, "project_export_operation_resource")
    assert "cancellation observation precedes admission" in cancellation_problems(
        request,
        response,
        prior,
        operation_inputs("project-json-v1")[2],
        case(prefix + "cancel-requested-decision-inputs"),
        replayed=False,
    )


def test_cancellation_rejects_noncanonical_retained_detail():
    prefix = "async-project-json-v1-"
    request = case(prefix + "cancel-requested-request")
    response = case(prefix + "cancel-requested-response")
    prior = case(prefix + "running-project_export_operation_resource")
    prior["execution"]["stages"][0]["attempts_started"] = 1.0
    assert cancellation_problems(
        request,
        response,
        prior,
        operation_inputs("project-json-v1")[2],
        case(prefix + "cancel-requested-decision-inputs"),
        replayed=False,
    ) == ["operation input violates bounded canonical subset"]


def test_success_requires_a_successful_retained_worker_attempt():
    values = operation_arguments()
    failed = case("async-project-json-v1-failed-project_export_operation_resource")["execution"][
        "attempts"
    ][0]
    values[1]["execution"]["attempts"] = [failed]
    validate_target(values[1], "project_export_operation_resource")
    assert "operation accepted stage has no successful final attempt" in operation_problems(*values)


def test_missing_retained_worker_is_not_reconstructed_from_output():
    values = operation_arguments()
    values[1]["execution"]["attempts"] = []
    validate_target(values[1], "project_export_operation_resource")
    problems = operation_problems(*values)
    assert "operation accepted stage has no successful final attempt" in problems
    assert "operation accepted output lacks worker attribution" in problems


def test_raw_key_is_not_retained_in_a_bounded_operation_extension():
    values = operation_arguments()
    values[1]["extension"] = {"private_value": [values[0]["metadata"]["idempotency"]["key"]]}
    validate_target(values[1], "project_export_operation_resource")
    assert "operation or plan retains the raw key" in operation_problems(*values)


def test_declared_budgets_and_clock_overflow_are_not_defaulted():
    values = operation_arguments()
    values[3]["limits"]["max_canonical_effects"] = 0
    validate_target(values[3], "project_export_operation_profile")
    assert "operation retained ledger exceeds selected budget" in operation_problems(*values)
    values[3]["limits"]["operation_lifetime_ms"] = 9007199254740991
    assert "operation deadline arithmetic is unrepresentable" in operation_problems(*values)


def test_cancellation_never_rebinds_an_artifact_after_request():
    values = operation_arguments()
    values[1] = case("async-project-json-v1-cancelled-retained-output-resource")
    values[1]["control"]["cancellation"]["requested_at"] = "2026-10-03T12:05:00.999Z"
    validate_target(values[1], "project_export_operation_resource")
    assert "operation artifact bind follows cancellation" in operation_problems(*values)


@pytest.mark.parametrize("name", FORMATS)
def test_approved_immediate_queued_cancellation(name):
    prefix = "async-" + name + "-"
    request = case(prefix + "queued-cancel-request")
    response = case(prefix + "queued-cancel-response")
    prior = case(prefix + "queued-project_export_operation_resource")
    decision = case(prefix + "queued-cancel-decision-inputs")
    profile = operation_inputs(name)[2]
    validate_target(decision, "project_export_cancellation_decision_inputs")
    assert cancellation_problems(request, response, prior, profile, decision, replayed=False) == []
    replay = case(prefix + "queued-cancel-replay")["body"]
    assert cancellation_problems(request, replay, prior, profile, decision, replayed=True) == []


@pytest.mark.parametrize("replayed", [False, True])
@pytest.mark.parametrize("clock", ["12:06:00.001", "12:07:00.000"])
def test_immediate_queued_cancellation_cannot_bypass_target_deadlines(replayed, clock):
    prefix = "async-project-json-v1-"
    request = case(prefix + "queued-cancel-request")
    response = case(prefix + "queued-cancel-response")
    prior = case(prefix + "queued-project_export_operation_resource")
    decision = case(prefix + "queued-cancel-decision-inputs")
    profile = operation_inputs("project-json-v1")[2]
    instant = "2026-10-03T" + clock + "Z"
    decision["recorded_at"] = instant
    response["result"]["cancellation"]["core"]["recorded_at"] = instant
    target = response["result"]["operation"]
    target["control"]["cancellation"]["requested_at"] = instant
    target["execution"]["timing"]["terminal_at"] = instant
    target["execution"]["stages"][0]["finished_at"] = instant
    response["idempotency_receipt"].update(
        replayed=replayed,
        expires_at="2026-10-10T" + clock + "Z",
        tombstone_expires_at="2026-11-02T" + clock + "Z",
    )
    rehash_cancellation_negative(response)
    for value, symbol in (
        (request, "project_export_operation_cancel_request"),
        (response, "project_export_operation_cancel_response"),
        (prior, "project_export_operation_resource"),
        (profile, "project_export_operation_profile"),
        (decision, "project_export_cancellation_decision_inputs"),
    ):
        validate_target(value, symbol)
    expected = [
        "operation stage finish interval differs",
        "operation terminalization requires unavailable late observation evidence",
        "operation business stage exceeds cancellation observation budget",
        "operation cancelled terminalization exceeds cancellation observation budget",
    ]
    assert (
        cancellation_problems(request, response, prior, profile, decision, replayed=replayed)
        == expected
    )
    values = operation_arguments()
    values[1] = target
    assert operation_problems(*values) == expected


@pytest.mark.parametrize("replayed", [False, True])
@pytest.mark.parametrize("clock", ["12:06:00.001", "12:07:00.000"])
def test_cancellation_cannot_renew_retained_causal_clock(replayed, clock):
    prefix = "async-project-json-v1-"
    request = case(prefix + "queued-cancel-request")
    response = case(prefix + "queued-cancel-response")
    prior = case(prefix + "queued-project_export_operation_resource")
    decision = case(prefix + "queued-cancel-decision-inputs")
    instant = "2026-10-03T" + clock + "Z"
    for operation in (prior, response["result"]["operation"]):
        operation["execution"]["timing"].update(
            admitted_at="2026-10-03T12:06:00.000Z",
            work_deadline="2026-10-03T12:07:00.000Z",
            reconciliation_deadline="2026-10-03T12:07:10.000Z",
        )
    target = response["result"]["operation"]
    target["control"]["cancellation"]["requested_at"] = instant
    target["execution"]["timing"]["terminal_at"] = instant
    target["execution"]["stages"][0]["finished_at"] = instant
    decision["recorded_at"] = instant
    response["result"]["cancellation"]["core"]["recorded_at"] = instant
    response["idempotency_receipt"].update(
        replayed=replayed,
        expires_at="2026-10-10T" + clock + "Z",
        tombstone_expires_at="2026-11-02T" + clock + "Z",
    )
    rehash_cancellation_negative(response)
    for value, symbol in (
        (request, "project_export_operation_cancel_request"),
        (response, "project_export_operation_cancel_response"),
        (prior, "project_export_operation_resource"),
        (decision, "project_export_cancellation_decision_inputs"),
    ):
        validate_target(value, symbol)
    expected = [
        "operation timing differs from admission",
        "operation fixed deadlines differ",
        "operation business stage exceeds cancellation observation budget",
        "operation cancelled terminalization exceeds cancellation observation budget",
    ]
    assert (
        cancellation_problems(
            request,
            response,
            prior,
            operation_inputs("project-json-v1")[2],
            decision,
            replayed=replayed,
        )
        == expected
    )
    values = operation_arguments()
    values[1] = target
    assert operation_problems(*values) == expected


@pytest.mark.parametrize("replayed", [False, True])
@pytest.mark.parametrize(
    "which", ["causal_future", "accepted_digest", "effect_stage", "audit_alias"]
)
def test_cancellation_validates_retained_output_and_ledger_joins(replayed, which):
    prefix = "async-project-json-v1-"
    request = case(prefix + "cancel-already_terminal-request")
    response = case(prefix + "cancel-already_terminal-response")
    prior = case(prefix + "succeeded-project_export_operation_resource")
    decision = case(prefix + "cancel-already_terminal-decision-inputs")
    if which == "causal_future":
        prior["causal_command"]["admitted_at"] = "2026-10-03T12:05:03.000Z"
        expected = [
            "operation timing differs from admission",
            "operation fixed deadlines differ",
            "cancellation decision precedes retained causal admission",
            "operation effect precedes admission",
        ]
    elif which == "accepted_digest":
        prior["execution"]["stages"][0]["accepted_result"]["result_sha256"] = OTHER_HASH
        expected = ["operation accepted result reference differs"]
    elif which == "effect_stage":
        prior["effects"]["entries"][0]["stage_id"] = OTHER_ID
        expected = [
            "operation effect names an absent stage",
            "operation artifact effect differs from accepted output",
        ]
    else:
        audit = prior["execution_attribution"]["authority_audit_event_id"]
        prior["execution"]["attempts"][0]["execution"]["authority_audit_event_id"] = audit
        prior["effects"]["entries"][0]["authority_audit_event_id"] = audit
        expected = ["operation worker audit identities alias prior execution"]
    response["result"]["operation"] = copy.deepcopy(prior)
    response["idempotency_receipt"]["replayed"] = replayed
    for value, symbol in (
        (request, "project_export_operation_cancel_request"),
        (response, "project_export_operation_cancel_response"),
        (prior, "project_export_operation_resource"),
        (decision, "project_export_cancellation_decision_inputs"),
    ):
        validate_target(value, symbol)
    assert (
        cancellation_problems(
            request,
            response,
            prior,
            operation_inputs("project-json-v1")[2],
            decision,
            replayed=replayed,
        )
        == expected
    )


@pytest.mark.parametrize(
    "which",
    [
        "attempt_before_stage",
        "attempt_after_stage",
        "late_success",
        "retry_with_running",
        "dispatch_after_cancel",
    ],
)
def test_review_execution_fences(which):
    values = operation_arguments()
    operation = values[1]
    stage = operation["execution"]["stages"][0]
    attempt = operation["execution"]["attempts"][0]
    if which == "attempt_before_stage":
        stage["started_at"] = "2026-10-03T12:05:00.200Z"
        stage["deadline"] = "2026-10-03T12:05:30.200Z"
        diagnostic = "operation attempt precedes its declared stage"
    elif which == "attempt_after_stage":
        attempt["finished_at"] = "2026-10-03T12:05:01.050Z"
        operation["execution"]["timing"]["terminal_at"] = "2026-10-03T12:05:02.000Z"
        diagnostic = "operation attempt outlives its declared stage"
    elif which == "late_success":
        operation["execution"]["timing"]["terminal_at"] = "2026-10-03T12:07:00.000Z"
        diagnostic = "operation successful terminalization exceeds fixed work deadline"
    elif which == "retry_with_running":
        operation = case("async-project-json-v1-running-project_export_operation_resource")
        values[1] = operation
        operation["execution"]["stages"][0].update(state="retry_wait", diagnostic="worker_lost")
        diagnostic = "operation retry wait lacks a finished transient failed attempt"
    else:
        operation = case("async-project-json-v1-cancelled-retained-output-resource")
        values[1] = operation
        operation["control"]["cancellation"]["requested_at"] = "2026-10-03T12:05:00.050Z"
        diagnostic = "operation attempt dispatch follows cancellation"
    validate_target(operation, "project_export_operation_resource")
    assert diagnostic in operation_problems(*values)


@pytest.mark.parametrize("which", ["resource", "read", "retry_wait"])
def test_terminal_failure_cannot_retain_business_work(which):
    values = operation_arguments()
    prefix = "async-project-json-v1-"
    operation = case(prefix + "failed-project_export_operation_resource")
    running = case(prefix + "running-project_export_operation_resource")
    operation["execution"]["stages"] = running["execution"]["stages"]
    operation["execution"]["attempts"] = running["execution"]["attempts"]
    if which == "retry_wait":
        stage = operation["execution"]["stages"][0]
        stage.update(state="retry_wait", diagnostic="dependency_unavailable")
        operation["execution"]["attempts"][0].update(
            state="failed",
            finished_at="2026-10-03T12:05:00.150Z",
            failure={"class": "transient", "code": "dependency_unavailable"},
        )
    values[1] = operation
    validate_target(operation, "project_export_operation_resource")
    if which == "read":
        response = case(prefix + "failed-project_export_operation_read_response")
        response["result"]["operation"] = operation
        validate_target(response, "project_export_operation_read_response")
        problems = operation_read_problems(
            case(prefix + "project_export_operation_get_request"),
            response,
            values[0],
            *values[2:],
        )
    else:
        problems = operation_problems(*values)
    assert "operation terminal resource retains unfinished business work" in problems


@pytest.mark.parametrize("name", FORMATS)
@pytest.mark.parametrize("clock", ["admission", "post-terminal"])
def test_selected_export_requires_stage_bound_canonical_effects(name, clock):
    value = case("async-" + name + "-unsupported-null-stage-effect-" + clock)
    with pytest.raises(ValueError, match="value violates owning contract"):
        validate_target(value, "project_export_operation_resource")


@pytest.mark.parametrize("which", ["stage", "attempt", "terminal", "all", "claimed_timeout"])
def test_finish_clocks_require_actual_late_observation_evidence(which):
    values = operation_arguments()
    operation = case("async-project-json-v1-failed-project_export_operation_resource")
    values[1] = operation
    stage, attempt = operation["execution"]["stages"][0], operation["execution"]["attempts"][0]
    timing = operation["execution"]["timing"]
    if which == "stage":
        stage["finished_at"] = "2026-10-03T12:05:31.000Z"
        timing["terminal_at"] = "2026-10-03T12:05:32.000Z"
        diagnostic = "operation stage finish interval differs"
    elif which == "attempt":
        attempt["finished_at"] = "2026-10-03T12:05:20.200Z"
        stage["finished_at"] = "2026-10-03T12:05:21.000Z"
        timing["terminal_at"] = "2026-10-03T12:05:22.000Z"
        diagnostic = "operation attempt finish interval differs"
    else:
        timing["terminal_at"] = "2026-10-03T12:07:00.000Z"
        diagnostic = "operation terminalization requires unavailable late observation evidence"
        if which != "terminal":
            stage["finished_at"] = attempt["finished_at"] = "2026-10-03T12:06:59.000Z"
        if which == "claimed_timeout":
            stage["diagnostic"] = "deadline_exceeded"
            attempt["failure"]["code"] = "deadline_exceeded"
    validate_target(operation, "project_export_operation_resource")
    problems = operation_problems(*values)
    assert diagnostic in problems
    if which in ("all", "claimed_timeout"):
        assert "operation stage finish interval differs" in problems
        assert "operation attempt finish interval differs" in problems


@pytest.mark.parametrize("surface", ["resource", "read"])
@pytest.mark.parametrize("which", ["late", "early"])
def test_winning_cancellation_bounds_business_completion(surface, which):
    values = operation_arguments()
    prefix = "async-project-json-v1-"
    operation = case(prefix + "cancelled-project_export_operation_resource")
    finish = "2026-10-03T12:05:05.000Z" if which == "late" else "2026-10-03T12:05:00.150Z"
    terminal = "2026-10-03T12:05:05.000Z" if which == "late" else "2026-10-03T12:05:01.000Z"
    operation["execution"]["stages"][0]["finished_at"] = finish
    operation["execution"]["attempts"][0]["finished_at"] = finish
    operation["execution"]["timing"]["terminal_at"] = terminal
    values[1] = operation
    validate_target(operation, "project_export_operation_resource")
    if surface == "read":
        response = case(prefix + "cancelled-project_export_operation_read_response")
        response["result"]["operation"] = operation
        receipt = response["result"]["idempotency_receipt"]
        receipt["expires_at"] = (
            "2026-10-10T12:05:05.000Z" if which == "late" else "2026-10-10T12:05:01.000Z"
        )
        receipt["tombstone_expires_at"] = (
            "2026-11-02T12:05:05.000Z" if which == "late" else "2026-11-02T12:05:01.000Z"
        )
        validate_target(response, "project_export_operation_read_response")
        problems = operation_read_problems(
            case(prefix + "project_export_operation_get_request"),
            response,
            values[0],
            *values[2:],
        )
    else:
        problems = operation_problems(*values)
    if which == "late":
        expected = [
            "operation business stage exceeds cancellation observation budget",
            "operation cancelled terminalization exceeds cancellation observation budget",
            "operation business attempt exceeds cancellation observation budget",
        ]
    else:
        expected = [
            "operation cancelled stage precedes winning cancellation",
            "operation cancelled attempt precedes winning cancellation",
        ]
    assert problems == expected


@pytest.mark.parametrize("surface", ["resource", "read"])
@pytest.mark.parametrize("state", ["failed", "skipped"])
def test_zero_attempt_business_finish_obeys_fixed_work_ceiling(surface, state):
    values = operation_arguments()
    prefix = "async-project-json-v1-"
    operation = case(prefix + "running-project_export_operation_resource")
    stage = case(prefix + "queued-project_export_operation_resource")["execution"]["stages"][0]
    stage.update(state=state, diagnostic="policy_changed", finished_at="2026-10-03T12:07:00.000Z")
    operation["execution"]["stages"] = [stage]
    operation["execution"]["attempts"] = []
    values[1] = operation
    validate_target(operation, "project_export_operation_resource")
    if surface == "read":
        response = case(prefix + "running-project_export_operation_read_response")
        response["result"]["operation"] = operation
        validate_target(response, "project_export_operation_read_response")
        problems = operation_read_problems(
            case(prefix + "project_export_operation_get_request"),
            response,
            values[0],
            *values[2:],
        )
    else:
        problems = operation_problems(*values)
    assert problems == ["operation stage finish interval differs"]


@pytest.mark.parametrize("surface", ["resource", "read"])
@pytest.mark.parametrize("which", ["stage", "attempt", "both"])
def test_business_cancellation_requires_its_retained_winning_request(surface, which):
    values = operation_arguments()
    prefix = "async-project-json-v1-"
    operation = case(prefix + "failed-project_export_operation_resource")
    if which != "attempt":
        operation["execution"]["stages"][0].update(state="cancelled", diagnostic="cancel_requested")
    if which != "stage":
        operation["execution"]["attempts"][0].update(state="cancelled", failure=None)
    values[1] = operation
    validate_target(operation, "project_export_operation_resource")
    if surface == "read":
        response = case(prefix + "failed-project_export_operation_read_response")
        response["result"]["operation"] = operation
        validate_target(response, "project_export_operation_read_response")
        problems = operation_read_problems(
            case(prefix + "project_export_operation_get_request"),
            response,
            values[0],
            *values[2:],
        )
    else:
        problems = operation_problems(*values)
    assert problems == ["operation cancelled business work lacks winning cancellation"]


@pytest.mark.parametrize("surface", ["resource", "read"])
def test_ended_worker_cannot_retain_active_exposure_observation(surface):
    values = operation_arguments()
    prefix = "async-project-json-v1-"
    operation = case(prefix + "failed-project_export_operation_resource")
    exposure = same_workspace_exposure_negative(operation, values[4])
    exposure.update(
        state="attempted", finished_at=None, receipt={"kind": "pending"}, settlement="reserved"
    )
    operation["exposures"]["entries"] = [exposure]
    values[1] = operation
    validate_target(operation, "project_export_operation_resource")
    if surface == "read":
        response = case(prefix + "failed-project_export_operation_read_response")
        response["result"]["operation"] = operation
        validate_target(response, "project_export_operation_read_response")
        problems = operation_read_problems(
            case(prefix + "project_export_operation_get_request"),
            response,
            values[0],
            *values[2:],
        )
    else:
        problems = operation_problems(*values)
    assert problems == ["operation exposure retains active observation after its worker finished"]


@pytest.mark.parametrize("surface", ["resource", "read"])
def test_counted_progress_requires_selected_immutable_unit_and_total(surface):
    values = operation_arguments()
    prefix = "async-project-json-v1-"
    operation = values[1]
    operation["execution"]["stages"][0]["progress"] = {
        "kind": "counted",
        "unit": "cases",
        "completed": 0,
        "total": 7,
    }
    validate_target(operation, "project_export_operation_resource")
    if surface == "read":
        response = case(prefix + "succeeded-project_export_operation_read_response")
        response["result"]["operation"] = operation
        validate_target(response, "project_export_operation_read_response")
        problems = operation_read_problems(
            case(prefix + "project_export_operation_get_request"),
            response,
            values[0],
            *values[2:],
        )
    else:
        problems = operation_problems(*values)
    assert problems == ["operation counted progress lacks independently selected unit and total"]


@pytest.mark.parametrize(
    "which", ["post_deadline", "post_success", "late_attempt", "running_probe", "late_confirmation"]
)
def test_same_workspace_reconciliation_has_no_implied_late_qualification(which):
    values = operation_arguments()
    if which != "post_success":
        values[1] = case("async-project-json-v1-failed-retained-output-resource")
    operation = values[1]
    exposure = same_workspace_exposure_negative(operation, values[4])
    operation["exposures"]["entries"] = [exposure]
    operation["control"]["reconciliation"] = "resolved"
    exposure["reconciliation"] = {
        "state": "resolved",
        "attempts_started": 0,
        "last_checked_at": "2026-10-03T12:05:00.700Z",
        "next_attempt_at": None,
        "deadline": "2026-10-03T12:06:10.000Z",
        "diagnostic": None,
    }
    if which == "post_deadline":
        exposure["reconciliation"]["last_checked_at"] = "2026-10-03T12:07:00.000Z"
        diagnostic = "operation exposure reconciliation observation interval differs"
    elif which == "post_success":
        exposure["reconciliation"]["last_checked_at"] = "2026-10-03T12:05:01.500Z"
        diagnostic = "operation successful exposure is not settled before terminalization"
    elif which == "late_confirmation":
        exposure["finished_at"] = "2026-10-03T12:07:00.000Z"
        diagnostic = "operation exposure requires unavailable late confirmation evidence"
    else:
        extra = copy.deepcopy(operation["execution"]["attempts"][0])
        extra.update(
            attempt_id=OTHER_ID,
            ordinal=2,
            subject={"kind": "exposure_reconciliation", "exposure_id": exposure["exposure_id"]},
        )
        extra["execution"]["authority_audit_event_id"] = OTHER_ID
        exposure["reconciliation"]["attempts_started"] = 1
        if which == "late_attempt":
            extra.update(
                state="failed",
                started_at="2026-10-03T12:06:09.000Z",
                deadline="2026-10-03T12:06:10.000Z",
                finished_at="2026-10-03T12:06:30.000Z",
                failure={"class": "transient", "code": "dependency_unavailable"},
            )
            exposure["reconciliation"]["last_checked_at"] = "2026-10-03T12:06:30.000Z"
            diagnostic = "operation attempt finish interval differs"
        else:
            extra.update(
                state="running",
                started_at="2026-10-03T12:05:00.600Z",
                deadline="2026-10-03T12:05:20.600Z",
                finished_at=None,
                failure=None,
            )
            diagnostic = "operation resolved reconciliation retains a running probe"
        operation["execution"]["attempts"].append(extra)
    validate_target(operation, "project_export_operation_resource")
    problems = operation_problems(*values)
    assert diagnostic in problems
    assert not any("another workspace" in problem for problem in problems)


@pytest.mark.parametrize(
    "label,time",
    [
        ("cancel-already_terminal", "00.500"),
        ("cancel-already_requested", "00.100"),
        ("retained-output-cancel", "00.200"),
    ],
)
def test_cancellation_cannot_observe_future_prior_events(label, time):
    prefix = "async-project-json-v1-"
    request = case(prefix + label + "-request")
    response = case(prefix + label + "-response")
    decision = case(prefix + label + "-decision-inputs")
    prior = case(
        prefix
        + (
            "succeeded-project_export_operation_resource"
            if label == "cancel-already_terminal"
            else "cancel-requested-response"
            if label == "cancel-already_requested"
            else "retained-output-cancel-prior"
        )
    )
    if label == "cancel-already_requested":
        prior = prior["result"]["operation"]
    instant = "2026-10-03T12:05:" + time + "Z"
    response["result"]["cancellation"]["core"]["recorded_at"] = instant
    decision["recorded_at"] = instant
    response["idempotency_receipt"]["expires_at"] = "2026-10-10T12:05:" + time + "Z"
    response["idempotency_receipt"]["tombstone_expires_at"] = "2026-11-02T12:05:" + time + "Z"
    rehash_cancellation_negative(response)
    validate_target(response, "project_export_operation_cancel_response")
    validate_target(decision, "project_export_cancellation_decision_inputs")
    assert "cancellation observes future prior events" in cancellation_problems(
        request, response, prior, operation_inputs("project-json-v1")[2], decision, replayed=False
    )


def test_cancellation_own_audit_cannot_alias_the_originating_audit():
    prefix = "async-project-json-v1-"
    request = case(prefix + "cancel-requested-request")
    response = case(prefix + "cancel-requested-response")
    decision = case(prefix + "cancel-requested-decision-inputs")
    prior = case(prefix + "running-project_export_operation_resource")
    response["result"]["cancellation"]["core"]["execution"]["authority_audit_event_id"] = prior[
        "execution_attribution"
    ]["authority_audit_event_id"]
    rehash_cancellation_negative(response)
    validate_target(response, "project_export_operation_cancel_response")
    problems = cancellation_problems(
        request, response, prior, operation_inputs("project-json-v1")[2], decision, replayed=False
    )
    assert "cancellation aliases prior command or worker audit" in problems
    assert "cancellation differs from independently supplied own decision" in problems


def test_cancellation_requires_independent_own_decision_evidence():
    prefix = "async-project-json-v1-"
    assert cancellation_problems(
        case(prefix + "cancel-requested-request"),
        case(prefix + "cancel-requested-response"),
        case(prefix + "running-project_export_operation_resource"),
        operation_inputs("project-json-v1")[2],
        None,
        replayed=False,
    ) == ["cancellation own qualified decision inputs are unavailable"]


@pytest.mark.parametrize(
    "which", ["duplicate_foreign", "absent_stage", "before_admission", "admission_audit"]
)
def test_complete_effect_ledger_rejects_inconsistent_nonartifact_effects(which):
    values = operation_arguments()
    original = values[1]["effects"]["entries"][0]
    event = dict(values[5]["control"]["last_event"])
    effect = {
        **original,
        "effect_id": OTHER_ID,
        "value": {"kind": "lifecycle_event_committed", "event": event},
    }
    if which == "duplicate_foreign":
        effect["effect_id"] = original["effect_id"]
        event["organization_id"] = OTHER_ID
        diagnostic = "operation effect identities are not distinct"
    elif which == "absent_stage":
        effect["stage_id"] = OTHER_ID
        diagnostic = "operation effect names an absent stage"
    elif which == "before_admission":
        effect["committed_at"] = "2026-10-03T12:04:59.000Z"
        diagnostic = "operation effect precedes admission"
    else:
        effect["authority_audit_event_id"] = OTHER_ID
        diagnostic = "operation effect lacks matching worker attribution or interval"
    values[1]["effects"]["entries"].append(effect)
    validate_target(values[1], "project_export_operation_resource")
    problems = operation_problems(*values)
    assert diagnostic in problems
    if which == "duplicate_foreign":
        assert "operation effect references belong to another workspace" in problems


def test_reconciliation_cannot_name_an_absent_exposure():
    values = operation_arguments()
    attempt = dict(values[1]["execution"]["attempts"][0])
    attempt.update(
        attempt_id=OTHER_ID,
        ordinal=2,
        subject={"kind": "exposure_reconciliation", "exposure_id": OTHER_ID},
        started_at="2026-10-03T12:05:00.200Z",
        deadline="2026-10-03T12:05:20.200Z",
        finished_at="2026-10-03T12:05:00.500Z",
    )
    attempt["execution"] = case("async-project-json-v1-running-project_export_operation_resource")[
        "execution"
    ]["attempts"][0]["execution"]
    attempt["execution"]["authority_audit_event_id"] = OTHER_ID
    values[1]["execution"]["attempts"].append(attempt)
    validate_target(values[1], "project_export_operation_resource")
    assert "operation reconciliation names an absent exposure" in operation_problems(*values)


@pytest.mark.parametrize(
    "which",
    [
        "absent_identity",
        "audit",
        "early_admission",
        "early_finish",
        "late_success",
        "duplicate",
        "counter",
        "deadline",
        "next",
        "early_checked",
        "pending_receipt",
        "recon_chronology",
        "dispatch_after_cancel",
    ],
)
def test_complete_foreign_exposure_ledger_is_never_accepted(which):
    values = operation_arguments()
    operation = values[1]
    worker = operation["execution"]["attempts"][0]
    exposure = foreign_exposure()
    exposure.update(
        stage_id=operation["execution"]["stages"][0]["stage_id"],
        attempt_id=worker["attempt_id"],
        authority_audit_event_id=worker["execution"]["authority_audit_event_id"],
        admitted_at="2026-10-03T12:05:00.200Z",
        finished_at="2026-10-03T12:05:00.500Z",
    )
    operation["exposures"]["entries"] = [exposure]
    diagnostic = "operation exposure references belong to another workspace"
    if which == "absent_identity":
        exposure.update(
            stage_id=OTHER_ID, attempt_id=OTHER_ID, admitted_at="2026-10-03T12:04:59.000Z"
        )
        diagnostic = "operation exposure stage or attempt joins differ"
    elif which == "audit":
        exposure["authority_audit_event_id"] = OTHER_ID
        diagnostic = "operation exposure worker attribution differs"
    elif which == "early_admission":
        exposure["admitted_at"] = "2026-10-03T12:05:00.050Z"
        diagnostic = "operation exposure admission falls outside its attempt"
    elif which == "early_finish":
        exposure["finished_at"] = "2026-10-03T12:05:00.100Z"
        diagnostic = "operation exposure observation interval differs"
    elif which == "late_success":
        exposure["finished_at"] = "2026-10-03T12:05:01.500Z"
        diagnostic = "operation successful exposure is not settled before terminalization"
    elif which == "duplicate":
        other = foreign_exposure()
        other.update(exposure)
        other["finished_at"] = "2026-10-03T12:05:00.600Z"
        other["receipt"] = {**other["receipt"], "receipt_sha256": OTHER_HASH}
        operation["exposures"]["entries"].append(other)
        diagnostic = "operation exposure identities are not distinct"
    elif which in ("counter", "deadline", "next", "pending_receipt", "recon_chronology"):
        # Preserve unknown liability in a failed retained-output operation.
        operation = case("async-project-json-v1-failed-retained-output-resource")
        values[1] = operation
        operation["exposures"]["entries"] = [exposure]
        exposure["settlement"] = "unsettled"
        exposure["reconciliation"] = {
            "state": "pending",
            "attempts_started": 0,
            "last_checked_at": None,
            "next_attempt_at": "2026-10-03T12:05:00.700Z",
            "deadline": "2026-10-03T12:06:10.000Z",
            "diagnostic": None,
        }
        operation["control"]["reconciliation"] = "pending"
        if which == "pending_receipt":
            exposure.update(state="uncertain", settlement="reserved", receipt={"kind": "pending"})
        elif which == "recon_chronology":
            exposure["reconciliation"].update(
                attempts_started=1, last_checked_at="2026-10-03T12:05:00.600Z"
            )
            extra = dict(worker)
            extra.update(
                attempt_id=OTHER_ID,
                ordinal=2,
                subject={"kind": "exposure_reconciliation", "exposure_id": exposure["exposure_id"]},
                finished_at="2026-10-03T12:05:00.650Z",
            )
            extra["execution"] = case(
                "async-project-json-v1-running-project_export_operation_resource"
            )["execution"]["attempts"][0]["execution"]
            extra["execution"]["authority_audit_event_id"] = OTHER_ID
            operation["execution"]["attempts"].append(extra)
            diagnostic = "operation exposure reconciliation attempt interval differs"
        elif which == "counter":
            exposure["reconciliation"].update(
                attempts_started=1, last_checked_at="2026-10-03T12:05:00.600Z"
            )
            diagnostic = "operation exposure reconciliation counter differs"
        elif which == "deadline":
            exposure["reconciliation"]["deadline"] = "2026-10-03T12:06:09.000Z"
            diagnostic = "operation exposure reconciliation fixed deadline differs"
        else:
            exposure["reconciliation"]["next_attempt_at"] = "2026-10-03T12:06:10.000Z"
            diagnostic = "operation exposure reconciliation observation interval differs"
    elif which == "dispatch_after_cancel":
        operation = case("async-project-json-v1-cancelled-retained-output-resource")
        values[1] = operation
        operation["exposures"]["entries"] = [exposure]
        exposure.update(
            admitted_at="2026-10-03T12:05:01.300Z", finished_at="2026-10-03T12:05:01.500Z"
        )
        diagnostic = "operation exposure dispatch follows cancellation"
    else:
        operation["control"]["reconciliation"] = "resolved"
        exposure["reconciliation"] = {
            "state": "resolved",
            "attempts_started": 0,
            "last_checked_at": "2026-10-03T12:05:00.100Z",
            "next_attempt_at": None,
            "deadline": "2026-10-03T12:06:10.000Z",
            "diagnostic": None,
        }
        diagnostic = "operation exposure reconciliation observation interval differs"
    validate_target(values[1], "project_export_operation_resource")
    problems = operation_problems(*values)
    assert "operation exposure references belong to another workspace" in problems
    assert diagnostic in problems


@pytest.mark.parametrize(
    "which",
    [
        "running_previous",
        "overlap",
        "audit_alias",
        "origin_alias",
        "kind_alias",
        "finished_running_stage",
    ],
)
def test_stage_attempt_intervals_and_audit_identity_are_not_fabricated(which):
    values = operation_arguments()
    operation = values[1]
    first = operation["execution"]["attempts"][0]
    if which == "origin_alias":
        first["execution"]["authority_audit_event_id"] = values[4]["execution"][
            "authority_audit_event_id"
        ]
        diagnostic = "operation worker audit identities alias prior execution"
    elif which == "kind_alias":
        first["execution"]["executor"]["principal_id"] = values[4]["execution"]["executor"][
            "principal_id"
        ]
        diagnostic = "operation principal identity has inconsistent kind"
    elif which == "finished_running_stage":
        operation = case("async-project-json-v1-running-project_export_operation_resource")
        values[1] = operation
        operation["execution"]["attempts"] = case(
            "async-project-json-v1-failed-project_export_operation_resource"
        )["execution"]["attempts"]
        diagnostic = "operation running stage lacks a running attempt"
    else:
        following = dict(first)
        following.update(
            attempt_id=OTHER_ID,
            ordinal=2,
            started_at="2026-10-03T12:05:01.000Z",
            deadline="2026-10-03T12:05:21.000Z",
        )
        following["execution"] = case(
            "async-project-json-v1-running-project_export_operation_resource"
        )["execution"]["attempts"][0]["execution"]
        following["execution"]["authority_audit_event_id"] = OTHER_ID
        operation["execution"]["attempts"].append(following)
        operation["execution"]["stages"][0].update(attempts_started=2, last_attempt_id=OTHER_ID)
        if which == "running_previous":
            first.update(state="running", finished_at=None)
            diagnostic = "operation stage attempt intervals overlap"
        elif which == "overlap":
            following.update(
                started_at="2026-10-03T12:05:00.200Z", deadline="2026-10-03T12:05:20.200Z"
            )
            diagnostic = "operation stage attempt intervals overlap"
        else:
            following["execution"]["authority_audit_event_id"] = first["execution"][
                "authority_audit_event_id"
            ]
            diagnostic = "operation worker audit identities alias prior execution"
    validate_target(values[1], "project_export_operation_resource")
    assert diagnostic in operation_problems(*values)


@pytest.mark.parametrize("surface", ["resource", "read"])
@pytest.mark.parametrize("which", ["local_finish", "probe", "checked", "next"])
def test_exposure_reconciliation_follows_ended_local_observation(surface, which):
    values = operation_arguments()
    operation = case("async-project-json-v1-failed-project_export_operation_resource")
    values[1] = operation
    exposure = same_workspace_exposure_negative(operation, values[4])
    operation["exposures"]["entries"] = [exposure]
    exposure["finished_at"] = "2026-10-03T12:05:00.800Z"
    if which == "local_finish":
        exposure.update(
            state="uncertain",
            finished_at="2026-10-03T12:05:01.500Z",
            receipt={"kind": "pending"},
            settlement="reserved",
        )
        exposure["reconciliation"] = {
            "state": "pending",
            "attempts_started": 0,
            "last_checked_at": None,
            "next_attempt_at": "2026-10-03T12:05:01.700Z",
            "deadline": "2026-10-03T12:06:10.000Z",
            "diagnostic": None,
        }
        operation["control"]["reconciliation"] = "pending"
        diagnostic = "operation exposure local observation outlives its worker"
    else:
        exposure["reconciliation"] = {
            "state": "resolved",
            "attempts_started": 0,
            "last_checked_at": "2026-10-03T12:05:00.900Z",
            "next_attempt_at": None,
            "deadline": "2026-10-03T12:06:10.000Z",
            "diagnostic": None,
        }
        operation["control"]["reconciliation"] = "resolved"
        diagnostic = "operation exposure reconciliation observation interval differs"
        if which == "checked":
            exposure["reconciliation"]["last_checked_at"] = "2026-10-03T12:05:00.400Z"
        elif which == "next":
            exposure["settlement"] = "unsettled"
            exposure["reconciliation"].update(
                state="pending",
                last_checked_at=None,
                next_attempt_at="2026-10-03T12:05:00.400Z",
            )
            operation["control"]["reconciliation"] = "pending"
        else:
            attempt = copy.deepcopy(operation["execution"]["attempts"][0])
            attempt.update(
                attempt_id=OTHER_ID,
                ordinal=2,
                subject={"kind": "exposure_reconciliation", "exposure_id": exposure["exposure_id"]},
                state="succeeded",
                failure=None,
                started_at="2026-10-03T12:05:00.300Z",
                deadline="2026-10-03T12:05:20.300Z",
                finished_at="2026-10-03T12:05:00.400Z",
            )
            attempt["execution"]["authority_audit_event_id"] = OTHER_ID
            operation["execution"]["attempts"].append(attempt)
            exposure["reconciliation"]["attempts_started"] = 1
            diagnostic = "operation exposure reconciliation attempt interval differs"
    validate_target(operation, "project_export_operation_resource")
    if surface == "read":
        response = case("async-project-json-v1-failed-project_export_operation_read_response")
        response["result"]["operation"] = operation
        validate_target(response, "project_export_operation_read_response")
        problems = operation_read_problems(
            case("async-project-json-v1-project_export_operation_get_request"),
            response,
            values[0],
            *values[2:],
        )
    else:
        problems = operation_problems(*values)
    assert problems == [diagnostic]


@pytest.mark.parametrize("surface", ["resource", "read"])
def test_cancelled_terminal_observation_uses_selected_cancellation_budget(surface):
    values = operation_arguments()
    operation = case("async-project-json-v1-cancelled-project_export_operation_resource")
    values[1] = operation
    operation["execution"]["timing"]["terminal_at"] = "2026-10-03T12:05:05.000Z"
    validate_target(operation, "project_export_operation_resource")
    if surface == "read":
        response = case("async-project-json-v1-cancelled-project_export_operation_read_response")
        response["result"]["operation"] = operation
        response["result"]["idempotency_receipt"].update(
            expires_at="2026-10-10T12:05:05.000Z",
            tombstone_expires_at="2026-11-02T12:05:05.000Z",
        )
        validate_target(response, "project_export_operation_read_response")
        problems = operation_read_problems(
            case("async-project-json-v1-project_export_operation_get_request"),
            response,
            values[0],
            *values[2:],
        )
    else:
        problems = operation_problems(*values)
    assert problems == [
        "operation cancelled terminalization exceeds cancellation observation budget"
    ]


@pytest.mark.parametrize("replayed", [False, True])
def test_observational_cancellation_preserves_winning_record_identity(replayed):
    prefix = "async-project-json-v1-"
    request = case(prefix + "cancel-already_requested-request")
    response = case(prefix + "cancel-already_requested-response")
    prior = case(prefix + "cancel-requested-response")["result"]["operation"]
    response["idempotency_receipt"]["replayed"] = replayed
    response["result"]["cancellation"]["core"]["cancellation_id"] = prior["control"][
        "cancellation"
    ]["cancellation_id"]
    rehash_cancellation_negative(response)
    validate_target(response, "project_export_operation_cancel_response")
    assert cancellation_problems(
        request,
        response,
        prior,
        operation_inputs("project-json-v1")[2],
        case(prefix + "cancel-already_requested-decision-inputs"),
        replayed=replayed,
    ) == ["cancellation own record aliases the retained winning cancellation"]


@pytest.mark.parametrize("replayed", [False, True])
@pytest.mark.parametrize("retained", ["command", "worker"])
def test_cancellation_executor_kind_agrees_with_retained_identity(replayed, retained):
    prefix = "async-project-json-v1-"
    request = case(prefix + "cancel-requested-request")
    response = case(prefix + "cancel-requested-response")
    prior = case(prefix + "running-project_export_operation_resource")
    decision = case(prefix + "cancel-requested-decision-inputs")
    execution = response["result"]["cancellation"]["core"]["execution"]
    known = (
        prior["execution_attribution"]
        if retained == "command"
        else prior["execution"]["attempts"][0]["execution"]
    )
    execution["executor"] = {"principal_id": known["executor"]["principal_id"], "kind": "agent"}
    execution["delegation_id"] = OTHER_ID
    decision["execution"] = copy.deepcopy(execution)
    response["idempotency_receipt"]["replayed"] = replayed
    rehash_cancellation_negative(response)
    validate_target(response, "project_export_operation_cancel_response")
    validate_target(decision, "project_export_cancellation_decision_inputs")
    assert cancellation_problems(
        request,
        response,
        prior,
        operation_inputs("project-json-v1")[2],
        decision,
        replayed=replayed,
    ) == ["cancellation principal identity has inconsistent kind"]


@pytest.mark.parametrize("replayed", [False, True])
@pytest.mark.parametrize(
    ("name", "path", "value", "diagnostic"),
    [
        (
            "project-json-v1",
            ("project", "project_id"),
            OTHER_ID,
            "operation source differs from selected project revision",
        ),
        (
            "project-json-v1",
            ("project", "content_revision", "value"),
            "2",
            "operation source differs from selected project revision",
        ),
        (
            "project-json-v1",
            ("observed_control_revision", "value"),
            "3",
            "operation source control differs",
        ),
        (
            "project-financial-json-v1",
            ("report", "content_sha256"),
            OTHER_HASH,
            "operation source differs from selected report",
        ),
        (
            "project-financial-json-v1",
            ("report", "project_id"),
            OTHER_ID,
            "operation source differs from selected report",
        ),
        (
            "project-financial-json-v1",
            ("observed_control_revision", "value"),
            "2",
            "operation source control differs",
        ),
    ],
)
def test_cancellation_retained_subject_matches_exact_input(replayed, name, path, value, diagnostic):
    prefix = "async-" + name + "-"
    request = case(prefix + "cancel-already_terminal-request")
    response = case(prefix + "cancel-already_terminal-response")
    prior = case(prefix + "succeeded-project_export_operation_resource")
    parent = prior["result"]["subject"]
    for key in path[:-1]:
        parent = parent[key]
    parent[path[-1]] = value
    response["result"]["operation"] = copy.deepcopy(prior)
    response["idempotency_receipt"]["replayed"] = replayed
    validate_target(prior, "project_export_operation_resource")
    validate_target(response, "project_export_operation_cancel_response")
    assert cancellation_problems(
        request,
        response,
        prior,
        operation_inputs(name)[2],
        case(prefix + "cancel-already_terminal-decision-inputs"),
        replayed=replayed,
    ) == [diagnostic]


@pytest.mark.parametrize("replayed", [False, True])
@pytest.mark.parametrize("field", ["organization_id", "domain_id"])
def test_cancellation_retained_financial_input_cannot_cross_workspace(replayed, field):
    prefix = "async-project-financial-json-v1-"
    request = case(prefix + "queued-cancel-request")
    response = case(prefix + "queued-cancel-response")
    prior = case(prefix + "queued-project_export_operation_resource")
    prior["input"]["report"][field] = OTHER_ID
    response["result"]["operation"]["input"] = copy.deepcopy(prior["input"])
    response["idempotency_receipt"]["replayed"] = replayed
    validate_target(prior, "project_export_operation_resource")
    validate_target(response, "project_export_operation_cancel_response")
    assert cancellation_problems(
        request,
        response,
        prior,
        operation_inputs("project-financial-json-v1")[2],
        case(prefix + "queued-cancel-decision-inputs"),
        replayed=replayed,
    ) == ["operation input report belongs to another workspace"]


@pytest.mark.parametrize("replayed", [False, True])
@pytest.mark.parametrize("member", ["delivery_profile", "lifecycle_policy"])
@pytest.mark.parametrize("field", ["organization_id", "domain_id"])
def test_cancellation_selected_profile_cannot_cross_workspace(replayed, member, field):
    prefix = "async-project-json-v1-"
    request = case(prefix + "queued-cancel-request")
    response = case(prefix + "queued-cancel-response")
    prior = case(prefix + "queued-project_export_operation_resource")
    profile = operation_inputs("project-json-v1")[2]
    profile[member][field] = OTHER_ID
    response["idempotency_receipt"]["replayed"] = replayed
    validate_target(profile, "project_export_operation_profile")
    validate_target(response, "project_export_operation_cancel_response")
    assert cancellation_problems(
        request,
        response,
        prior,
        profile,
        case(prefix + "queued-cancel-decision-inputs"),
        replayed=replayed,
    ) == ["operation selected delivery or policy scope differs"]


@pytest.mark.parametrize("which", ["input", "stage_profile", "selected_profile", "foreign_profile"])
def test_plan_read_checks_complete_stage_declarations(which):
    prefix = "async-project-json-v1-"
    selector = case(prefix + "project_export_operation_plan_get_request")
    response = case(prefix + "project_export_operation_plan_get_response")
    operation = case(prefix + "succeeded-project_export_operation_resource")
    core = response["result"]["plan"]["core"]
    declaration = core["stages"][0]
    if which == "input":
        declaration["input_sha256"] = OTHER_HASH
        diagnostic = "operation plan read stage input differs from complete input"
    elif which == "stage_profile":
        declaration["profile"]["content_sha256"] = OTHER_HASH
        diagnostic = "operation plan read stage profile differs from selected profile"
    elif which == "selected_profile":
        core["profile"]["content_sha256"] = OTHER_HASH
        declaration["profile"] = copy.deepcopy(core["profile"])
        diagnostic = "operation plan read declaration differs from retained output"
    else:
        core["profile"]["organization_id"] = OTHER_ID
        declaration["profile"] = copy.deepcopy(core["profile"])
        diagnostic = "operation plan read selected profile belongs to another workspace"
    rehash_plan_negative(response, operation)
    validate_target(response, "project_export_operation_plan_get_response")
    validate_target(operation, "project_export_operation_resource")
    expected = [diagnostic]
    if which == "selected_profile":
        expected += [
            "operation accepted result reference differs",
            "operation complete stage-result joins differ",
        ]
    assert plan_read_problems(selector, response, operation) == expected


@pytest.mark.parametrize("name", FORMATS)
@pytest.mark.parametrize("replayed", [False, True])
@pytest.mark.parametrize("field", ["organization_id", "domain_id"])
@pytest.mark.parametrize(
    ("label", "prior_label"),
    [
        ("cancel-requested", "running-project_export_operation_resource"),
        ("cancel-already_requested", "cancel-requested-response"),
        ("cancel-already_terminal", "succeeded-project_export_operation_resource"),
        ("retained-output-cancel", "retained-output-cancel-prior"),
        ("queued-cancel", "queued-project_export_operation_resource"),
    ],
)
def test_cancellation_retained_plan_cannot_cross_workspace(
    name, replayed, field, label, prior_label
):
    prefix = "async-" + name + "-"
    request = case(prefix + label + "-request")
    response = case(prefix + label + "-response")
    prior = case(prefix + prior_label)
    if prior_label == "cancel-requested-response":
        prior = prior["result"]["operation"]
    prior["execution"]["plan"][field] = OTHER_ID
    response["result"]["operation"]["execution"]["plan"][field] = OTHER_ID
    response["idempotency_receipt"]["replayed"] = replayed
    validate_target(prior, "project_export_operation_resource")
    validate_target(response, "project_export_operation_cancel_response")
    assert cancellation_problems(
        request,
        response,
        prior,
        operation_inputs(name)[2],
        case(prefix + label + "-decision-inputs"),
        replayed=replayed,
    ) == ["cancellation retained plan belongs to another workspace"]


@pytest.mark.parametrize("name", FORMATS)
@pytest.mark.parametrize(
    ("limit", "value", "diagnostic"),
    [
        ("operation_lifetime_ms", 61000, "operation fixed deadlines differ"),
        ("max_canonical_effects", 0, "operation retained ledger exceeds selected budget"),
    ],
)
def test_plan_read_limits_bind_retained_execution(name, limit, value, diagnostic):
    prefix = "async-" + name + "-"
    selector = case(prefix + "project_export_operation_plan_get_request")
    response = case(prefix + "project_export_operation_plan_get_response")
    operation = case(prefix + "succeeded-project_export_operation_resource")
    response["result"]["plan"]["core"]["limits"][limit] = value
    rehash_plan_negative(response, operation)
    validate_target(response, "project_export_operation_plan_get_response")
    validate_target(operation, "project_export_operation_resource")
    assert plan_read_problems(selector, response, operation) == [diagnostic]


@pytest.mark.parametrize("name", FORMATS)
@pytest.mark.parametrize("inline", [False, True])
def test_plan_read_reconciles_complete_retained_stage_result(name, inline):
    prefix = "async-" + name + "-"
    selector = case(prefix + "project_export_operation_plan_get_request")
    response = case(prefix + "project_export_operation_plan_get_response")
    operation = case(prefix + "succeeded-project_export_operation_resource")
    operation["execution"]["stages"][0]["accepted_result"]["stage_id"] = OTHER_ID
    expected = ["operation accepted result reference differs"]
    if inline:
        operation["execution"]["stage_results"][0]["stage_id"] = OTHER_ID
        rehash_stage_result_negative(operation)
        expected.append("operation complete stage-result joins differ")
    validate_target(operation, "project_export_operation_resource")
    validate_target(response, "project_export_operation_plan_get_response")
    assert plan_read_problems(selector, response, operation) == expected


@pytest.mark.parametrize("name", FORMATS)
@pytest.mark.parametrize("which", ["audit", "early_effect", "absent_attempt", "foreign_effect"])
def test_plan_read_checks_complete_ledger(name, which):
    prefix = "async-" + name + "-"
    selector = case(prefix + "project_export_operation_plan_get_request")
    response = case(prefix + "project_export_operation_plan_get_response")
    operation = case(prefix + "succeeded-project_export_operation_resource")
    worker = operation["execution"]["attempts"][0]
    effect = operation["effects"]["entries"][0]
    if which == "audit":
        worker["execution"]["authority_audit_event_id"] = operation["execution_attribution"][
            "authority_audit_event_id"
        ]
        effect["authority_audit_event_id"] = worker["execution"]["authority_audit_event_id"]
        diagnostic = "operation worker audit identities alias prior execution"
    elif which == "early_effect":
        worker.update(started_at="2026-10-03T12:05:00.200Z", deadline="2026-10-03T12:05:20.200Z")
        effect["committed_at"] = "2026-10-03T12:05:00.150Z"
        diagnostic = "operation effect lacks matching worker attribution or interval"
    elif which == "absent_attempt":
        exposure = same_workspace_exposure_negative(operation, operation_inputs(name)[3])
        exposure["attempt_id"] = OTHER_ID
        operation["exposures"]["entries"] = [exposure]
        diagnostic = "operation exposure stage or attempt joins differ"
    else:
        event = copy.deepcopy(operation_inputs(name)[4]["control"]["last_event"])
        event["organization_id"] = OTHER_ID
        operation["effects"]["entries"].append(
            {
                **effect,
                "effect_id": OTHER_ID,
                "value": {"kind": "lifecycle_event_committed", "event": event},
            }
        )
        diagnostic = "operation effect references belong to another workspace"
    validate_target(operation, "project_export_operation_resource")
    validate_target(response, "project_export_operation_plan_get_response")
    assert plan_read_problems(selector, response, operation) == [diagnostic]


@pytest.mark.parametrize("name", FORMATS)
@pytest.mark.parametrize("which", ["causal_context", "causal_audit", "origin_command"])
def test_plan_read_checks_intrinsic_retained_admission(name, which):
    prefix = "async-" + name + "-"
    selector = case(prefix + "project_export_operation_plan_get_request")
    response = case(prefix + "project_export_operation_plan_get_response")
    operation = case(prefix + "succeeded-project_export_operation_resource")
    if which == "causal_context":
        operation["causal_command"]["context"]["organization_id"] = OTHER_ID
    elif which == "causal_audit":
        operation["causal_command"]["execution"]["authority_audit_event_id"] = OTHER_ID
    else:
        operation["origin"]["command_id"] = OTHER_ID
    validate_target(operation, "project_export_operation_resource")
    validate_target(response, "project_export_operation_plan_get_response")
    assert plan_read_problems(selector, response, operation) == [
        "operation plan read retained causal attribution differs"
    ]


@pytest.mark.parametrize("name", FORMATS[2:])
@pytest.mark.parametrize("field", ["organization_id", "domain_id"])
@pytest.mark.parametrize("state", ["queued", "running", "failed", "cancelled"])
def test_plan_read_financial_input_scope_in_every_unfinished_or_unsuccessful_state(
    name, field, state
):
    prefix = "async-" + name + "-"
    selector = case(prefix + "project_export_operation_plan_get_request")
    response = case(prefix + "project_export_operation_plan_get_response")
    operation = case(prefix + state + "-project_export_operation_resource")
    operation["input"]["report"][field] = OTHER_ID
    rehash_plan_input_negative(response, operation)
    validate_target(operation, "project_export_operation_resource")
    validate_target(response, "project_export_operation_plan_get_response")
    assert plan_read_problems(selector, response, operation) == [
        "operation input report belongs to another workspace"
    ]


@pytest.mark.parametrize("name", FORMATS)
@pytest.mark.parametrize("state", ["failed", "cancelled"])
@pytest.mark.parametrize(
    "surface", ["resource", "read", "replay", "plan", "cancel", "cancel_replay"]
)
def test_global_attempt_counters_follow_exposure_causality(name, state, surface):
    operation = reversed_counter_history_negative(name, state, tied=False)
    validate_target(operation, "project_export_operation_resource")
    request, plan, profile, admission, artifact = operation_inputs(name)
    prefix = "async-" + name + "-"
    if surface == "resource":
        problems = operation_problems(request, operation, plan, profile, admission, artifact)
    elif surface == "read":
        response = case(prefix + state + "-project_export_operation_read_response")
        response["result"]["operation"] = operation
        validate_target(response, "project_export_operation_read_response")
        problems = operation_read_problems(
            case(prefix + "project_export_operation_get_request"),
            response,
            request,
            plan,
            profile,
            admission,
            artifact,
        )
    elif surface == "replay":
        exchange = case(prefix + state + "-replay-project_export_operation_command_exchange")
        exchange["body"]["result"]["operation"] = operation
        validate_target(exchange, "project_export_operation_command_exchange")
        problems = command_exchange_problems(
            request, exchange, plan, profile, admission, artifact, replayed=True
        )
    elif surface == "plan":
        response = case(prefix + "project_export_operation_plan_get_response")
        validate_target(response, "project_export_operation_plan_get_response")
        problems = plan_read_problems(
            case(prefix + "project_export_operation_plan_get_request"), response, operation
        )
    else:
        replayed = surface == "cancel_replay"
        cancel_request, response, decision = counter_history_cancellation_negative(
            name, operation, replayed=replayed
        )
        validate_target(cancel_request, "project_export_operation_cancel_request")
        validate_target(response, "project_export_operation_cancel_response")
        validate_target(decision, "project_export_cancellation_decision_inputs")
        problems = cancellation_problems(
            cancel_request, response, operation, profile, decision, replayed=replayed
        )
    assert problems == [
        "operation attempt counters contradict admission chronology",
        "operation reconciliation counter precedes its exposure producer",
    ]


@pytest.mark.parametrize("name", FORMATS)
@pytest.mark.parametrize("state", ["failed", "cancelled"])
def test_same_timestamp_counters_still_require_exposure_cause_first(name, state):
    operation = reversed_counter_history_negative(name, state, tied=True)
    validate_target(operation, "project_export_operation_resource")
    request, plan, profile, admission, artifact = operation_inputs(name)
    assert operation_problems(request, operation, plan, profile, admission, artifact) == [
        "operation reconciliation counter precedes its exposure producer"
    ]
