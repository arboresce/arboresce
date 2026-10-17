//! Decode only supplied runtime inputs and observe the actual pure scope API.
//! Expected values, case names and manifest metadata never enter this helper.

use arboresce_application::policy::bootstrap::{
    BootstrapGrant, BootstrapRequest, BootstrapScopeFailure, RegisteredBootstrapScope, check_scope,
};
use arboresce_domain::{Identifier, UtcTimestamp};
use serde_json::{Value, json};

fn id(v: &Value) -> Identifier {
    v.as_str().unwrap().parse().unwrap()
}

fn text(v: &Value) -> &str {
    v.as_str().unwrap()
}

fn time(v: &Value) -> UtcTimestamp {
    v.as_str().unwrap().parse().unwrap()
}

pub fn observe(input: &Value, configuration: &Value) -> Value {
    let g = &input["grant"];
    let grant = BootstrapGrant {
        id: id(&g["grant_id"]),
        operator: id(&g["operator_principal_id"]),
        namespace: text(&g["namespace"]),
        policy: text(&g["policy"]),
        organization: id(&g["organization_id"]),
        issued_at: time(&g["issued_at"]),
        expires_at: time(&g["expires_at"]),
        single_use: g["single_use"].as_bool().unwrap(),
        consumed: g["consumed"].as_bool().unwrap(),
        recognized_retired: g["recognized_retired"].as_bool().unwrap(),
    };
    let r = &input["request"];
    let request = BootstrapRequest {
        namespace: text(&r["namespace"]),
        policy: text(&r["policy"]),
        organization: id(&r["organization_id"]),
        new_owner: id(&r["new_owner_principal_id"]),
    };
    let registered = RegisteredBootstrapScope {
        namespace: text(&configuration["namespace"]),
        policy: text(&configuration["policy"]),
    };
    // The fixture's normalized operator is an injected authenticated context,
    // not a public request field that can establish live authentication.
    let result = check_scope(
        registered,
        &grant,
        &request,
        id(&r["operator_principal_id"]),
        time(&input["now"]),
    );
    let (failure, reason) = match result {
        Ok(()) => (None, "synthetic_scope_only"),
        Err(failure) => {
            let reason = match failure {
                BootstrapScopeFailure::InvalidConfiguration => "invalid_configuration",
                BootstrapScopeFailure::InvalidStoredData => "invalid_stored_data",
                BootstrapScopeFailure::InvalidInput => "invalid_input",
                BootstrapScopeFailure::OperatorMismatch => "operator_mismatch",
                BootstrapScopeFailure::NamespaceMismatch => "namespace_mismatch",
                BootstrapScopeFailure::PolicyMismatch => "policy_mismatch",
                BootstrapScopeFailure::TenantMismatch => "tenant_mismatch",
                BootstrapScopeFailure::GrantExpired => "grant_expired",
                BootstrapScopeFailure::GrantUsed => "grant_used",
                BootstrapScopeFailure::ScopeRetired => "scope_retired",
                BootstrapScopeFailure::GrantNotYetValid => "grant_not_yet_valid",
                BootstrapScopeFailure::UnsupportedBootstrapGrant => "unsupported_bootstrap_grant",
            };
            (Some(format!("{failure:?}")), reason)
        }
    };
    // Serialize the actual typed grant after the call, rather than copying an
    // expected object. The pure API has no writer, owner insertion or effect.
    json!({
        "scope": {
            "semantic_scope_decision": if result.is_ok() { "admit" } else { "deny" },
            "reason": reason,
            "writer_admitted": false,
            "grant_after": {
                "grant_id": grant.id.to_string(),
                "operator_principal_id": grant.operator.to_string(),
                "namespace": grant.namespace,
                "policy": grant.policy,
                "organization_id": grant.organization.to_string(),
                "issued_at": grant.issued_at.to_string(),
                "expires_at": grant.expires_at.to_string(),
                "single_use": grant.single_use,
                "consumed": grant.consumed,
                "recognized_retired": grant.recognized_retired,
            },
            "owner_rows_created": [],
            "effects": [],
        },
        "failure": failure,
    })
}
