//! Decode supplied inputs and observe actual calls, without expected answers.

use arboresce_application::lifecycle::recognition::{key_digest, recognition_reference};
use serde_json::{Value, json};

pub fn observe_key(input: &Value) -> Value {
    let before = input.clone();
    match key_digest(input["key"].as_str().unwrap()) {
        Ok(value) => json!({
            "accepted": true,
            "key_digest": value.to_string(),
            "error_display": null,
            "error_debug": null,
            "input_unchanged": input == &before,
        }),
        Err(error) => json!({
            "accepted": false,
            "key_digest": null,
            "error_display": error.to_string(),
            "error_debug": format!("{error:?}"),
            "input_unchanged": input == &before,
        }),
    }
}

pub fn observe_reference(input: &Value) -> Value {
    let before = input.clone();
    let result = recognition_reference(
        input["organization_id"].as_str().unwrap().parse().unwrap(),
        input["principal_id"].as_str().unwrap().parse().unwrap(),
        input["command"].as_str().unwrap(),
        input["key_digest"].as_str().unwrap().parse().unwrap(),
    );
    match result {
        Ok(reference) => json!({
            "accepted": true,
            "reference": {
                "scope": {
                    "organization_id": reference.organization_id().to_string(),
                    "principal_id": reference.principal_id().to_string(),
                },
                "recognition_sha256": reference.recognition_sha256().to_string(),
            },
            "reference_debug": format!("{reference:?}"),
            "error_display": null,
            "error_debug": null,
            "input_unchanged": input == &before,
        }),
        Err(error) => json!({
            "accepted": false,
            "reference": null,
            "reference_debug": null,
            "error_display": error.to_string(),
            "error_debug": format!("{error:?}"),
            "input_unchanged": input == &before,
        }),
    }
}
