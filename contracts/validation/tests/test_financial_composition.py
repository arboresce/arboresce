"""Independent literal conformance and consequential compound transition failures."""

import copy
import hashlib
import json
from pathlib import Path

import pytest
from project_semantics_support import (
    altered,
    case,
    composition_literal,
    proposal_problems,
    rehash_negative,
    validate_target,
    vector,
)

from arboresce_contract_validation.financial_composition import (
    confirmation_problems,
    content_problems,
    review_problems,
)

OTHER_ID = "ffffffff-ffff-4fff-8fff-ffffffffffff"
OTHER_HASH = "f" * 64
COUNTER = {"kind": "control", "value": "9"}
CONTENT = {"kind": "content", "value": "9"}


def test_approved_named_values_and_independent_mutable_fixture_images():
    literal = composition_literal()
    for target, value in literal["values"].items():
        validate_target(value, target)
    other = composition_literal()
    literal["proposal_vectors"][0]["expense"]["revision"]["content"]["gross"] = "0.00"
    assert other["proposal_vectors"][0]["expense"]["revision"]["content"]["gross"] == "50.00"
    assert composition_literal() == other


def test_project_fixture_encodings_preserve_reviewed_values_and_mutation_isolation():
    identity = "async-project-json-v1-command-request"
    manifest = json.loads(
        (Path(__file__).resolve().parents[1] / "project-workflow-v1.cases.json").read_text()
    )
    literal = next(row["value"] for row in manifest["cases"] if row["id"] == identity)
    first, second = case(identity), case(identity)
    assert first == second == literal
    first["metadata"]["context"]["purpose"] = "mutated_test_image"
    assert second == case(identity) == literal
    with pytest.raises(StopIteration):
        case("missing-independent-fixture-identity")


CASES = json.loads(
    (Path(__file__).resolve().parents[1] / "project-workflow-v1.cases.json").read_text()
)["cases"][1272:]


@pytest.mark.parametrize("row", CASES, ids=lambda row: row["id"])
def test_independently_declared_structural_expectations(row):
    if row["valid"]:
        validate_target(row["value"], row["target"])
    else:
        with pytest.raises(ValueError, match="value violates owning contract"):
            validate_target(row["value"], row["target"])


@pytest.mark.parametrize("index", range(4))
def test_approved_complete_proposal_transitions(index):
    row = vector("proposal", index)
    validate_target(row["request"], "financial_composition_propose_request_v1")
    validate_target(row["after_response"], "financial_composition_propose_response_v1")
    validate_target(row["project"], "financial_composition_project_observation_v1")
    if row["before_composition"] is not None:
        validate_target(row["before_composition"], "financial_composition_snapshot_v1")
    if row["before_association"] is not None:
        validate_target(row["before_association"], "financial_composition_association_snapshot_v1")
    assert proposal_problems(row) == []


@pytest.mark.parametrize("index", range(2))
def test_approved_complete_review_and_confirmation_observations(index):
    review = vector("review", index)
    for field, target in [
        ("challenge", "financial_composition_review_challenge_v1"),
        ("event", "financial_composition_review_event_v1"),
        ("proof", "financial_composition_review_proof_v1"),
        ("rendering", "financial_composition_review_rendering_v1"),
        ("project", "financial_composition_project_observation_v1"),
    ]:
        validate_target(review[field], target)
    assert review_problems(review) == []
    confirmation = vector("confirmation", index)
    validate_target(confirmation["request"], "financial_composition_confirm_request_v1")
    validate_target(confirmation["after"], "financial_composition_confirm_response_v1")
    assert (
        confirmation_problems(confirmation, composition_literal()["receipt_retention_profile"])
        == []
    )


@pytest.mark.parametrize(
    "index,path,replacement",
    [
        (0, "request/body/fences/expense_control_revision", COUNTER),
        (0, "request/body/fences/project_control_revision", COUNTER),
        (0, "request/body/fences/association_control_revision", COUNTER),
        (0, "request/body/fences/association/association_sha256", OTHER_HASH),
        (0, "request/body/composition/reference/content_sha256", OTHER_HASH),
        (0, "request/body/composition/reference/domain_id", OTHER_ID),
        (0, "request/body/composition/preimage/expense/content_sha256", OTHER_HASH),
        (0, "request/body/association/expected_control_revision", COUNTER),
        (0, "request/body/association/association_revision", CONTENT),
        (0, "request/body/association/association_id", OTHER_ID),
        (0, "after_composition/control_revision", COUNTER),
        (0, "after_association/control_revision", COUNTER),
        (0, "after_response/result/expense/control_revision", COUNTER),
        (0, "expense/control_revision", COUNTER),
        (0, "expense/revision_state", "voided"),
        (0, "expense/revision/reference/version", {"kind": "content", "value": "1"}),
        (0, "known_absence/composition", False),
        (0, "known_absence/target_relationship", False),
        (0, "before_association", None),
        (0, "financial_bindings/0/event/financial_review_event_id", OTHER_ID),
        (0, "financial_bindings/0/event/association/association_sha256", OTHER_HASH),
        (0, "financial_bindings/0/association/expense/content_sha256", OTHER_HASH),
        (0, "project/project/workspace/domain_id", OTHER_ID),
        (0, "financial_bindings", []),
        (0, "financial_bindings/0/expense_confirmation/reference/expense_id", OTHER_ID),
        (0, "financial_bindings/0/expense_confirmation/reference/version", CONTENT),
        (1, "known_absence/association", False),
        (1, "request/body/fences/association_control_revision", COUNTER),
        (1, "request/body/association/association_revision", CONTENT),
        (1, "request/body/association/expected_control_revision", COUNTER),
        (1, "after_association/control_revision", COUNTER),
        (2, "before_composition", None),
        (2, "request/body/fences/composition_control_revision", COUNTER),
        (2, "request/body/fences/composition/content_sha256", OTHER_HASH),
        (2, "before_composition/state", "voided"),
        (2, "request/body/composition/reference/composition_id", OTHER_ID),
        (2, "request/body/composition/reference/content_revision", CONTENT),
        (2, "after_composition/control_revision", COUNTER),
        (
            2,
            "before_composition/control_revision",
            {"kind": "control", "value": "999999999999999999"},
        ),
        (
            2,
            "before_association/control_revision",
            {"kind": "control", "value": "999999999999999999"},
        ),
    ],
)
def test_stale_or_partial_proposal_observations_are_rejected(index, path, replacement):
    assert proposal_problems(altered(vector("proposal", index), path, replacement))


@pytest.mark.parametrize(
    "path,replacement",
    [
        ("challenge/preimage/project/content_revision", CONTENT),
        ("event/preimage/composition/content_sha256", OTHER_HASH),
        ("challenge/review_sha256", OTHER_HASH),
        ("event/review_sha256", OTHER_HASH),
        ("proof/review_sha256", OTHER_HASH),
        ("proof/proof/digest", OTHER_HASH),
        ("rendering/review_sha256", OTHER_HASH),
        ("controls/composition", COUNTER),
        ("controls/association", COUNTER),
        ("controls/expense", COUNTER),
        ("controls/project", COUNTER),
        ("composition/state", "voided"),
        ("expense/revision_state", "voided"),
        ("association/association/expense/content_sha256", OTHER_HASH),
        ("event/base/challenge_id", OTHER_ID),
        ("event/base/expense_version", CONTENT),
        ("challenge/base/expense/expense_id", OTHER_ID),
        ("challenge/base/association/association_id", OTHER_ID),
        ("challenge/base/expense_version", CONTENT),
        ("challenge/base/content_sha256", OTHER_HASH),
        ("proof/challenge_id", OTHER_ID),
        ("proof/reviewer/principal_id", OTHER_ID),
        ("proof/assurance/assurance_profile", "other_profile"),
        ("challenge/base/assurance/state", "unqualified"),
        ("challenge/base/assurance/review_policy/version", "2"),
        ("event/base/state", "revoked"),
        ("event/base/revoked_at", "2026-10-05T12:00:30.000Z"),
        ("proof/asserted_at", "2026-10-05T11:59:59.000Z"),
        ("proof/asserted_at", "2026-10-05T12:01:01.000Z"),
        ("event/base/recorded_at", "2026-10-05T12:05:00.000Z"),
        ("composition/revision/provenance/recorded_at", "2026-10-05T12:00:01.000Z"),
        ("association/association/provenance/recorded_at", "2026-10-05T12:00:01.000Z"),
        ("expense/revision/recorded_at", "2026-10-05T12:01:01.000Z"),
        ("rendering/text", "Approve a different expense."),
        ("rendering/rendering_sha256", OTHER_HASH),
    ],
)
def test_compound_review_cannot_change_targets_proof_or_display(path, replacement):
    assert review_problems(altered(vector("review"), path, replacement))


@pytest.mark.parametrize(
    "index,path,replacement",
    [
        (0, "request/body/composition/content_sha256", OTHER_HASH),
        (0, "request/body/project/project_id", OTHER_ID),
        (0, "request/body/expense/expense_id", OTHER_ID),
        (0, "request/body/association/association_id", OTHER_ID),
        (0, "request/body/controls/project", COUNTER),
        (0, "request/body/controls/expense", COUNTER),
        (0, "request/body/controls/association", COUNTER),
        (0, "request/body/controls/composition", COUNTER),
        (0, "request/body/financial_review_event_id", OTHER_ID),
        (0, "before_composition/state", "voided"),
        (0, "before_expense/revision_state", "voided"),
        (0, "event/preimage/project/project_id", OTHER_ID),
        (0, "event/review_sha256", OTHER_HASH),
        (0, "event/base/expense/content_sha256", OTHER_HASH),
        (0, "event/base/association/association_id", OTHER_ID),
        (0, "event/base/expense_version", CONTENT),
        (0, "event/base/content_sha256", OTHER_HASH),
        (0, "event/base/domain_id", OTHER_ID),
        (0, "event/base/state", "revoked"),
        (0, "event/base/revoked_at", "2026-10-05T12:01:00.000Z"),
        (0, "event/base/assurance/state", "unqualified"),
        (0, "event/base/assurance/review_policy/version", "2"),
        (0, "after/result/composition/confirmation/recorded_at", "2026-10-05T12:05:00.000Z"),
        (0, "after/result/composition/confirmation/recorded_at", "2026-10-05T12:00:00.000Z"),
        (0, "after/result/composition/revision/provenance/audit_event_id", OTHER_ID),
        (0, "after/result/composition/state", "voided"),
        (0, "after/result/composition/superseded_by", {"organization_id": OTHER_ID}),
        (0, "after/result/association/association/provenance/audit_event_id", OTHER_ID),
        (0, "after/result/project/control_revision", COUNTER),
        (0, "after/result/composition/control_revision", COUNTER),
        (0, "after/result/association/control_revision", COUNTER),
        (0, "after/result/composition/confirmation/reference/content_sha256", OTHER_HASH),
        (0, "after/result/composition/confirmation/control_revision", COUNTER),
        (0, "after/result/composition/confirmation/association/association_sha256", OTHER_HASH),
        (0, "after/result/composition/confirmation/association_control_revision", COUNTER),
        (0, "after/result/composition/confirmation/financial_review_event_id", OTHER_ID),
        (0, "after/result/composition/confirmation/reviewer/principal_id", OTHER_ID),
        (0, "after/result/composition/confirmation/executor/principal_id", OTHER_ID),
        (0, "after/result/expense/revision/audit_event_id", OTHER_ID),
        (0, "after/result/expense/revision_state", "proposed"),
        (0, "after/result/expense/superseded_by", {"expense_id": OTHER_ID}),
        (0, "after/result/expense/control_revision", COUNTER),
        (0, "after/result/expense/confirmation/reference/content_sha256", OTHER_HASH),
        (0, "after/result/expense/confirmation/control_revision", COUNTER),
        (0, "after/result/expense/confirmation/financial_review_event_id", OTHER_ID),
        (0, "after/result/expense/confirmation/executor/principal_id", OTHER_ID),
        (0, "after/result/composition/confirmation/expense_disposition", "already_confirmed"),
        (1, "after/result/expense/confirmation/financial_review_event_id", OTHER_ID),
        (1, "after/result/expense/control_revision", COUNTER),
        (1, "after/result/composition/confirmation/expense_disposition", "confirmed_now"),
        (0, "after_consumptions", []),
        (0, "after_consumptions/0/financial_review_event_id", OTHER_ID),
        (0, "after_consumptions/0/composition/content_sha256", OTHER_HASH),
        (0, "after_consumptions/0/expense/content_sha256", OTHER_HASH),
        (0, "after_consumptions/0/association/association_sha256", OTHER_HASH),
        (0, "after_consumptions/0/executor_scope/executor/principal_id", OTHER_ID),
        (0, "after_consumptions/0/recorded_at", "2026-10-05T12:02:01.000Z"),
        (0, "after_consumptions/0/audit_event_id", OTHER_ID),
        (0, "after_consumptions/0/request_fingerprint", OTHER_HASH),
        (0, "after/idempotency_receipt/key_digest", OTHER_HASH),
        (0, "after/idempotency_receipt/request_fingerprint", OTHER_HASH),
        (0, "after/idempotency_receipt/replayed", True),
        (0, "after/idempotency_receipt/expires_at", "2026-10-13T12:02:00.000Z"),
        (0, "after/idempotency_receipt/tombstone_expires_at", "2026-11-05T12:02:00.000Z"),
    ],
)
def test_confirmation_rejects_stale_authority_partial_effects_or_changed_receipts(
    index, path, replacement
):
    row = altered(vector("confirmation", index), path, replacement)
    assert confirmation_problems(row, composition_literal()["receipt_retention_profile"])


def test_duplicate_consumption_and_changed_retained_history_are_rejected():
    row = vector("confirmation")
    row["prior_consumptions"] = copy.deepcopy(row["after_consumptions"])
    row["after_consumptions"].append(copy.deepcopy(row["after_consumptions"][0]))
    assert "composition selected global event consumption differs" in confirmation_problems(
        row, composition_literal()["receipt_retention_profile"]
    )
    row["after_consumptions"][0]["command_id"] = OTHER_ID
    assert "composition single event consumption append differs" in confirmation_problems(
        row, composition_literal()["receipt_retention_profile"]
    )
    proposal = vector("proposal")
    proposal["financial_bindings"].append(copy.deepcopy(proposal["financial_bindings"][0]))
    assert "composition financial binding history repeats an event" in proposal_problems(proposal)


@pytest.mark.parametrize(
    "profile",
    [
        {"replay_retention_ms": 86400000, "diagnostic_retention_ms": 2592000000},
        {"replay_retention_ms": 604800000, "diagnostic_retention_ms": 604800000},
        {"replay_retention_ms": 9007199254740990, "diagnostic_retention_ms": 9007199254740991},
    ],
)
def test_invalid_or_unrepresentable_original_retention_is_rejected(profile):
    assert confirmation_problems(vector("confirmation"), profile)


@pytest.mark.parametrize("kind", ["proposal", "review", "confirmation"])
def test_unencodable_comparison_inputs_fail_closed(kind):
    row = vector(kind)
    row["event" if kind == "review" else "request"]["noncanonical_extension"] = float("nan")
    if kind == "proposal":
        problems = proposal_problems(row)
    elif kind == "review":
        problems = review_problems(row)
    else:
        problems = confirmation_problems(row, composition_literal()["receipt_retention_profile"])
    assert problems == ["composition input violates bounded canonical subset"]


def test_unencodable_content_and_normalized_relationship_noop_are_rejected():
    row = vector("proposal")
    bad = copy.deepcopy(row["request"]["body"]["composition"])
    bad["noncanonical_extension"] = float("nan")
    assert content_problems(bad, row["request"]["body"]["association"]) == [
        "composition input violates bounded canonical subset"
    ]
    row["request"]["body"]["association"]["content_preimage"] = copy.deepcopy(
        row["before_association"]["association"]["content_preimage"]
    )
    assert "composition association is a normalized no-op" in proposal_problems(row)


@pytest.mark.parametrize("level", ["envelope", "result"])
def test_harmless_additive_response_fields_do_not_change_transition_checks(level):
    proposal = vector("proposal")
    response = proposal["after_response"]
    (response if level == "envelope" else response["result"])["future_hint"] = -1.25
    validate_target(response, "financial_composition_propose_response_v1")
    assert proposal_problems(proposal) == []
    confirmation = vector("confirmation")
    response = confirmation["after"]
    (response if level == "envelope" else response["result"])["future_hint"] = -1.25
    validate_target(response, "financial_composition_confirm_response_v1")
    assert (
        confirmation_problems(confirmation, composition_literal()["receipt_retention_profile"])
        == []
    )


def test_coherent_foreign_predecessor_cannot_authorize_same_workspace_successor():
    row = vector("proposal")
    prior = row["before_association"]["association"]
    prior["workspace"]["domain_id"] = OTHER_ID
    prior["expense"]["domain_id"] = OTHER_ID
    prior["content_preimage"]["domain_id"] = OTHER_ID
    prior["association_sha256"] = rehash_negative(prior["content_preimage"])
    row["request"]["body"]["fences"]["association"]["association_sha256"] = prior[
        "association_sha256"
    ]
    validate_target(row["before_association"], "financial_composition_association_snapshot_v1")
    assert "composition actual association predecessor ownership differs" in proposal_problems(row)


def test_distinct_events_cannot_confirm_one_immutable_revision_twice():
    row = vector("proposal")
    extra = copy.deepcopy(row["financial_bindings"][0])
    extra["event"]["financial_review_event_id"] = OTHER_ID
    extra["expense_confirmation"]["financial_review_event_id"] = OTHER_ID
    row["financial_bindings"].append(extra)
    validate_target(extra["event"], "financial_review_event")
    assert (
        "composition financial binding history repeats a confirmed revision"
        in proposal_problems(row)
    )


def test_fresh_review_cannot_target_a_superseded_expense_revision():
    row = vector("review")
    expense = row["expense"]
    expense["revision_state"] = "superseded"
    expense["superseded_by"] = {
        **expense["revision"]["reference"],
        "version": CONTENT,
        "content_sha256": OTHER_HASH,
    }
    assert "composition review current targets or controls differ" in review_problems(row)


@pytest.mark.parametrize(
    "path,replacement",
    [
        ("before_composition/revision/content/preimage/project/project_id", OTHER_ID),
        ("before_composition/revision/content/preimage/association/association_id", OTHER_ID),
        ("before_composition/confirmation/reference/content_sha256", OTHER_HASH),
        ("before_composition/confirmation/financial_review_event_id", OTHER_ID),
        ("before_composition/confirmation/reviewer/principal_id", OTHER_ID),
        ("before_composition/confirmation/executor/principal_id", OTHER_ID),
        ("before_composition/confirmation/delegation_id", OTHER_ID),
        ("before_composition/confirmation/authority_audit_event_id", OTHER_ID),
        ("before_composition/confirmation/association/association_sha256", OTHER_HASH),
        ("before_composition/confirmation/control_revision", COUNTER),
        ("before_composition/confirmation/association_control_revision", COUNTER),
        ("before_composition/confirmation/control_revision", {"kind": "control", "value": "1"}),
        (
            "before_composition/confirmation/association_control_revision",
            {"kind": "control", "value": "1"},
        ),
        ("before_composition/confirmation/recorded_at", "2026-10-05T11:59:59.000Z"),
        ("before_composition/confirmation/recorded_at", "2026-10-05T12:03:01.000Z"),
    ],
)
def test_retained_composition_content_and_confirmation_must_join(path, replacement):
    assert proposal_problems(altered(vector("proposal", 2), path, replacement))


def test_selected_confirmed_revision_requires_its_exact_retained_confirmation():
    row = vector("proposal", 2)
    row["financial_bindings"] = row["financial_bindings"][:1]
    assert "composition selected original confirmation is absent or contradictory" in (
        proposal_problems(row)
    )
    row = vector("proposal", 2)
    binding = row["financial_bindings"][1]
    reference = binding["expense_confirmation"]["reference"]
    reference["content_sha256"] = OTHER_HASH
    binding["event"]["expense"] = copy.deepcopy(reference)
    binding["event"]["content_sha256"] = OTHER_HASH
    binding["association"]["expense"] = copy.deepcopy(reference)
    binding["association"]["content_preimage"]["expense_content_sha256"] = OTHER_HASH
    binding["association"]["association_sha256"] = rehash_negative(
        binding["association"]["content_preimage"]
    )
    binding["event"]["association"]["association_sha256"] = binding["association"][
        "association_sha256"
    ]
    validate_target(binding["event"], "financial_review_event")
    assert "composition required financial binding history differs" in proposal_problems(row)


def test_historical_association_exists_before_its_original_review():
    row = vector("proposal")
    row["financial_bindings"][0]["association"]["provenance"]["recorded_at"] = (
        "2026-09-08T12:01:01.000Z"
    )
    assert "composition historical review precedes association creation" in proposal_problems(row)


def test_preserving_original_confirmation_does_not_admit_a_contradictory_reference():
    row = vector("confirmation", 1)
    row["before_expense"]["confirmation"]["reference"]["content_sha256"] = OTHER_HASH
    row["after"]["result"]["expense"] = copy.deepcopy(row["before_expense"])
    assert "composition selected expense original confirmation differs" in confirmation_problems(
        row, composition_literal()["receipt_retention_profile"]
    )


@pytest.mark.parametrize(
    "path,replacement",
    [
        ("expense/confirmation/recorded_at", "2026-10-05T12:01:01.000Z"),
        ("expense/confirmation/recorded_at", "2026-09-08T11:59:59.000Z"),
        ("expense/confirmation/control_revision", COUNTER),
        ("expense/confirmation/control_revision", {"kind": "control", "value": "1"}),
    ],
)
def test_review_checks_original_confirmed_snapshot_reference_control_and_time(path, replacement):
    row = altered(vector("review", 1), path, replacement)
    assert "composition selected expense original confirmation differs" in review_problems(row)


def test_proposal_cannot_precede_the_confirmation_it_claims_to_preserve():
    row = vector("proposal", 2)
    row["expense"]["confirmation"]["recorded_at"] = "2026-10-05T12:03:01.000Z"
    row["financial_bindings"][1]["expense_confirmation"] = copy.deepcopy(
        row["expense"]["confirmation"]
    )
    row["after_response"]["result"]["expense"] = copy.deepcopy(row["expense"])
    assert "composition retained financial history follows proposal" in proposal_problems(row)


@pytest.mark.parametrize(
    "path",
    [
        "expense/revision/recorded_at",
        "before_composition/revision/provenance/recorded_at",
        "before_association/association/provenance/recorded_at",
    ],
)
def test_actual_proposal_predecessors_exist_before_the_commit(path):
    assert proposal_problems(altered(vector("proposal", 2), path, "2026-10-05T12:03:01.000Z"))


@pytest.mark.parametrize(
    "created,problem",
    [
        ("2026-10-05T12:00:01.000Z", "composition new association follows proposal terminal"),
        ("2000-01-01T00:00:00.000Z", "composition new association precedes required inputs"),
    ],
)
def test_coherent_new_association_must_fit_proposal_creation_bounds(created, problem):
    row = vector("proposal")
    association = row["request"]["body"]["association"]
    association["provenance"]["recorded_at"] = created
    row["after_association"]["association"] = copy.deepcopy(association)
    row["after_response"]["result"]["association"] = copy.deepcopy(row["after_association"])
    preimage = {
        "method": "POST",
        "resource": "/v1/financial-compositions",
        "contract_version": "v1",
        "purpose": row["request"]["metadata"]["context"]["purpose"],
        "body": row["request"],
    }
    encoded = json.dumps(preimage, sort_keys=True, separators=(",", ":"))
    row["after_response"]["idempotency_receipt"]["request_fingerprint"] = hashlib.sha256(
        b"arboresce.command.v1\0" + encoded.encode()
    ).hexdigest()
    assert problem in proposal_problems(row)


@pytest.mark.parametrize(
    "path,replacement",
    [
        ("financial_bindings/0/event/assurance/state", "unqualified"),
        ("financial_bindings/0/event/assurance/review_policy/version", "2"),
        ("financial_bindings/0/event/expense_version", CONTENT),
        ("financial_bindings/0/event/content_sha256", OTHER_HASH),
        ("financial_bindings/0/event/domain_id", OTHER_ID),
        (
            "financial_bindings/0/expense_confirmation/control_revision",
            {"kind": "control", "value": "1"},
        ),
        ("after_response/idempotency_receipt/key_digest", OTHER_HASH),
        ("after_response/idempotency_receipt/request_fingerprint", OTHER_HASH),
        ("after_response/idempotency_receipt/replayed", True),
        ("after_response/idempotency_receipt/expires_at", "2026-10-13T12:00:00.000Z"),
    ],
)
def test_original_authority_and_proposal_receipt_remain_bound(path, replacement):
    assert proposal_problems(altered(vector("proposal"), path, replacement))


def test_revoked_original_authority_cannot_precede_its_successful_consumption():
    row = vector("proposal")
    binding = row["financial_bindings"][0]
    binding["event"]["state"] = "revoked"
    binding["event"]["revoked_at"] = binding["expense_confirmation"]["recorded_at"]
    validate_target(binding["event"], "financial_review_event")
    assert "composition original financial assurance declaration differs" in proposal_problems(row)


@pytest.mark.parametrize(
    "path",
    [
        "event/base/recorded_at",
        "before_composition/revision/provenance/recorded_at",
        "before_association/association/provenance/recorded_at",
        "before_expense/revision/recorded_at",
    ],
)
def test_event_cannot_authorize_targets_before_they_exist(path):
    instant = (
        "2026-10-05T11:59:59.000Z"
        if path == "event/base/recorded_at"
        else "2026-10-05T12:01:01.000Z"
    )
    row = altered(vector("confirmation"), path, instant)
    assert confirmation_problems(row, composition_literal()["receipt_retention_profile"])


def test_coherent_old_expense_cannot_replace_immutable_composition_selection():
    row = vector("confirmation")
    old = composition_literal()["original_expense"]
    reference = old["revision"]["reference"]
    row["before_expense"] = copy.deepcopy(old)
    row["request"]["body"]["expense"] = copy.deepcopy(reference)
    row["request"]["body"]["controls"]["expense"] = copy.deepcopy(old["control_revision"])
    event = row["event"]
    event["base"].update(
        expense=copy.deepcopy(reference),
        expense_version=copy.deepcopy(reference["version"]),
        content_sha256=reference["content_sha256"],
    )
    event["preimage"]["base"].update(
        expense_id=reference["expense_id"],
        expense_version=copy.deepcopy(reference["version"]),
        content_sha256=reference["content_sha256"],
    )
    event["preimage"]["controls"]["expense"] = copy.deepcopy(old["control_revision"])
    event["review_sha256"] = rehash_negative(event["preimage"])
    row["after"]["result"]["expense"] = copy.deepcopy(old)
    row["after"]["result"]["composition"]["confirmation"]["expense_disposition"] = (
        "already_confirmed"
    )
    row["after_consumptions"][0]["expense"] = copy.deepcopy(reference)
    fingerprint_preimage = {
        "method": "POST",
        "resource": "/v1/financial-compositions/"
        + row["request"]["body"]["composition"]["composition_id"]
        + "/confirm",
        "contract_version": "v1",
        "purpose": row["request"]["metadata"]["context"]["purpose"],
        "body": row["request"],
    }
    encoded = json.dumps(fingerprint_preimage, sort_keys=True, separators=(",", ":"))
    fingerprint = hashlib.sha256(b"arboresce.command.v1\0" + encoded.encode()).hexdigest()
    row["after_consumptions"][0]["request_fingerprint"] = fingerprint
    row["after"]["idempotency_receipt"]["request_fingerprint"] = fingerprint
    validate_target(row["request"], "financial_composition_confirm_request_v1")
    validate_target(row["after"], "financial_composition_confirm_response_v1")
    assert "composition fresh confirmation fences differ" in confirmation_problems(
        row, composition_literal()["receipt_retention_profile"]
    )


def test_coherent_compound_confirmation_cannot_reuse_original_consumed_event():
    row = vector("confirmation", 1)
    original = copy.deepcopy(row["before_expense"])
    event_id = original["confirmation"]["financial_review_event_id"]
    row["request"]["body"]["financial_review_event_id"] = event_id
    row["event"]["base"]["financial_review_event_id"] = event_id
    row["after"]["result"]["composition"]["confirmation"]["financial_review_event_id"] = event_id
    row["after_consumptions"][0]["financial_review_event_id"] = event_id
    preimage = {
        "method": "POST",
        "resource": "/v1/financial-compositions/"
        + row["request"]["body"]["composition"]["composition_id"]
        + "/confirm",
        "contract_version": "v1",
        "purpose": row["request"]["metadata"]["context"]["purpose"],
        "body": row["request"],
    }
    encoded = json.dumps(preimage, sort_keys=True, separators=(",", ":"))
    fingerprint = hashlib.sha256(b"arboresce.command.v1\0" + encoded.encode()).hexdigest()
    row["after_consumptions"][0]["request_fingerprint"] = fingerprint
    row["after"]["idempotency_receipt"]["request_fingerprint"] = fingerprint
    assert row["before_expense"] == row["after"]["result"]["expense"] == original
    validate_target(row["request"], "financial_composition_confirm_request_v1")
    validate_target(row["after"], "financial_composition_confirm_response_v1")
    assert "composition event was already consumed by the original expense confirmation" in (
        confirmation_problems(row, composition_literal()["receipt_retention_profile"])
    )


def test_fresh_compound_issuance_cannot_reuse_original_consumed_event():
    row = vector("review", 1)
    row["event"]["base"]["financial_review_event_id"] = row["expense"]["confirmation"][
        "financial_review_event_id"
    ]
    validate_target(row["event"], "financial_composition_review_event_v1")
    assert "composition event was already consumed by the original expense confirmation" in (
        review_problems(row)
    )
