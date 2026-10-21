//! Decode only supplied fixture inputs and observe actual typed projections.
//! Expected objects, case IDs and source metadata never reach this helper.

use arboresce_application::lifecycle::{
    CheckpointScope, EntryScope, Family, Namespace, Ordinal, Predecessor, match_scope,
};
use arboresce_domain::{Identifier, Sha256Digest};
use serde_json::{Value, json};

fn text(value: &Value) -> &str {
    value.as_str().unwrap()
}

fn integer(value: &Value) -> u64 {
    text(value).parse().unwrap()
}

fn ordinal(value: &Value) -> Ordinal {
    Ordinal::new(integer(value)).unwrap()
}

fn identifier(value: &Value) -> Identifier {
    text(value).parse().unwrap()
}

fn digest(value: &Value) -> Sha256Digest {
    text(value).parse().unwrap()
}

pub(super) fn namespace(value: &Value) -> Namespace {
    Namespace::new(
        identifier(&value["organization_id"]),
        identifier(&value["stream_id"]),
    )
}

fn namespace_view(value: Namespace) -> Value {
    json!({
        "organization_id": value.organization().to_string(),
        "stream_id": value.stream().to_string(),
    })
}

pub(super) fn predecessor(value: &Value) -> Predecessor {
    match text(&value["kind"]) {
        "beginning" => Predecessor::Beginning,
        "after" => Predecessor::After {
            namespace: namespace(&value["namespace"]),
            last_ordinal: ordinal(&value["last_ordinal"]),
            checkpoint_sha256: digest(&value["checkpoint_sha256"]),
        },
        _ => panic!("invalid fixture predecessor kind"),
    }
}

fn predecessor_view(value: Predecessor) -> Value {
    match value {
        Predecessor::Beginning => json!({"kind": "beginning"}),
        Predecessor::After {
            namespace,
            last_ordinal,
            checkpoint_sha256,
        } => json!({
            "kind": "after",
            "namespace": namespace_view(namespace),
            "last_ordinal": last_ordinal.value().to_string(),
            "checkpoint_sha256": checkpoint_sha256.to_string(),
        }),
    }
}

fn checkpoint(value: &Value) -> CheckpointScope {
    let previous = &value["previous_checkpoint_sha256"];
    CheckpointScope {
        namespace: namespace(&value["namespace"]),
        first: ordinal(&value["first_ordinal"]),
        last: ordinal(&value["last_ordinal"]),
        previous_checkpoint_sha256: if previous.is_null() {
            None
        } else {
            Some(digest(previous))
        },
    }
}

pub(super) fn checkpoint_view(value: &CheckpointScope) -> Value {
    json!({
        "namespace": namespace_view(value.namespace),
        "first_ordinal": value.first.value().to_string(),
        "last_ordinal": value.last.value().to_string(),
        "previous_checkpoint_sha256": value.previous_checkpoint_sha256.map(|digest| digest.to_string()),
    })
}

fn entry(value: &Value) -> EntryScope {
    EntryScope {
        organization: identifier(&value["organization_id"]),
        ordinal: ordinal(&value["ordinal"]),
        family: Family::from_name(text(&value["family"])).unwrap(),
    }
}

pub(super) fn entry_view(value: EntryScope) -> Value {
    json!({
        "organization_id": value.organization.to_string(),
        "ordinal": value.ordinal.value().to_string(),
        "family": value.family.as_str(),
    })
}

pub fn observe_family(input: &Value) -> Value {
    let family = Family::from_name(text(&input["value"]));
    json!({"kind": "family", "accepted": family.is_some(), "family": family.map(Family::as_str)})
}

pub fn observe_ordinal(input: &Value) -> Value {
    let ordinal = Ordinal::new(integer(&input["value"])).ok();
    json!({
        "kind": "ordinal",
        "accepted": ordinal.is_some(),
        "value": ordinal.map(|value| value.value().to_string()),
    })
}

pub fn observe_scope(input: &Value) -> Value {
    let expected_namespace = namespace(&input["expected_namespace"]);
    let predecessor = predecessor(&input["predecessor"]);
    let checkpoint = checkpoint(&input["checkpoint"]);
    let entries = input["entries"]
        .as_array()
        .unwrap()
        .iter()
        .map(entry)
        .collect::<Vec<_>>();
    let result = match_scope(expected_namespace, predecessor, &checkpoint, &entries);
    let (failure, range) = match result {
        Ok(range) => (
            None,
            Some(json!({
                "first": range.first.value().to_string(),
                "last": range.last.value().to_string(),
                "count": range.count,
            })),
        ),
        Err(failure) => (Some(format!("{failure:?}")), None),
    };
    json!({
        "kind": "scope",
        "matched": result.is_ok(),
        "failure": failure,
        "range": range,
        "expected_namespace_after": namespace_view(expected_namespace),
        "predecessor_after": predecessor_view(predecessor),
        "checkpoint_after": checkpoint_view(&checkpoint),
        "entries_after": entries.iter().copied().map(entry_view).collect::<Vec<_>>(),
    })
}
