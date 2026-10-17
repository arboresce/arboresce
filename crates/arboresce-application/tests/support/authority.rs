//! Input-only canonical projection decoder and actual pure API observations.
//! Expected objects stay in assertion bodies, outside these functions.

use std::error::Error;

use arboresce_application::policy::{
    ReferenceKind, ScopedReference, Workspace,
    authority::{
        AuthorityFailure, AuthorityObservation, CanonicalProjection, DomainRecord,
        EvaluationRequest, GrantRecord, GrantScope, ImmutableChain, MembershipRecord,
        OrganizationRecord, PolicyConfiguration, PrincipalKind, PrincipalRecord,
        PrincipalReference, RecordState, ResourceRecord, SourcePolicy, WorkspacePolicy, evaluate,
    },
};
use arboresce_domain::{ControlRevision, Identifier, UtcTimestamp};
use serde_json::{Value, json};

fn id(v: &Value) -> Identifier {
    v.as_str().unwrap().parse().unwrap()
}
fn text(v: &Value) -> String {
    v.as_str().unwrap().to_owned()
}
fn time(v: &Value) -> UtcTimestamp {
    v.as_str().unwrap().parse().unwrap()
}
fn optional_id(v: &Value) -> Option<Identifier> {
    (!v.is_null()).then(|| id(v))
}
fn optional_time(v: &Value) -> Option<UtcTimestamp> {
    (!v.is_null()).then(|| time(v))
}
fn revision(v: &Value) -> ControlRevision {
    assert_eq!(v["kind"], "control");
    v["value"].as_str().unwrap().parse().unwrap()
}
fn strings(v: &Value) -> Vec<String> {
    v.as_array().unwrap().iter().map(text).collect()
}
fn state(v: &Value) -> RecordState {
    if v == "active" {
        RecordState::Active
    } else {
        RecordState::Inactive
    }
}
fn kind(v: &Value) -> PrincipalKind {
    match v.as_str().unwrap() {
        "human" => PrincipalKind::Human,
        "agent" => PrincipalKind::Agent,
        "service" => PrincipalKind::Service,
        _ => PrincipalKind::Unsupported,
    }
}
fn workspace(v: &Value) -> Workspace {
    Workspace::new(id(&v["organization_id"]), id(&v["domain_id"]))
}
fn principal(v: &Value) -> PrincipalReference {
    PrincipalReference {
        id: id(&v["principal_id"]),
        kind: kind(&v["kind"]),
    }
}
fn reference(v: &Value) -> ScopedReference {
    let k = match v["kind"].as_str().unwrap() {
        "project" => ReferenceKind::Project,
        "expense" => ReferenceKind::Expense,
        _ => panic!("unadmitted resource fixture family"),
    };
    ScopedReference::new(workspace(&v["workspace"]), k, id(&v["id"]))
}
fn grant(v: &Value) -> GrantRecord {
    let s = &v["scope"];
    let scope = if s["resource_kind"] == "workspace" {
        assert!(s["resource_id"].is_null());
        GrantScope::Workspace(workspace(s))
    } else {
        let k = match s["resource_kind"].as_str().unwrap() {
            "project" => ReferenceKind::Project,
            "expense" => ReferenceKind::Expense,
            _ => panic!("unadmitted grant fixture family"),
        };
        GrantScope::Exact(ScopedReference::new(workspace(s), k, id(&s["resource_id"])))
    };
    GrantRecord {
        id: id(&v["grant_id"]),
        issuer: principal(&v["issuer"]),
        subject: principal(&v["subject"]),
        parent: optional_id(&v["parent_grant_id"]),
        scope,
        actions: strings(&v["actions"]),
        purposes: strings(&v["purposes"]),
        delegable: v["delegable"].as_bool().unwrap(),
        issued_at: time(&v["issued_at"]),
        expires_at: time(&v["expires_at"]),
        policy: text(&v["policy"]),
        revision: revision(&v["revision"]),
        state: state(&v["state"]),
        revoked_at: optional_time(&v["revoked_at"]),
    }
}

fn projection(v: &Value) -> CanonicalProjection {
    CanonicalProjection {
        now: time(&v["now"]),
        organizations: v["organizations"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| OrganizationRecord {
                id: id(&r["organization_id"]),
                state: state(&r["state"]),
            })
            .collect(),
        domains: v["domains"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| DomainRecord {
                workspace: workspace(r),
                state: state(&r["state"]),
            })
            .collect(),
        principals: v["principals"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| {
                let i = &r["identity_profile"];
                PrincipalRecord {
                    principal: principal(r),
                    identity_profile: text(&i["profile"]),
                    identity_version: text(&i["version"]),
                    public_key: text(&i["public_key"]),
                    revision: revision(&r["control_revision"]),
                    state: state(&r["state"]),
                    created_at: time(&r["created_at"]),
                    revoked_at: optional_time(&r["revoked_at"]),
                }
            })
            .collect(),
        memberships: v["memberships"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| MembershipRecord {
                id: id(&r["membership_id"]),
                principal: principal(&r["principal"]),
                workspace: workspace(&r["workspace"]),
                roles: strings(&r["roles"]),
                policy: text(&r["policy"]),
                revision: revision(&r["revision"]),
                state: state(&r["state"]),
            })
            .collect(),
        policies: v["policies"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| WorkspacePolicy {
                workspace: workspace(&r["workspace"]),
                policy: text(&r["policy"]),
                epoch: revision(&r["epoch"]),
                state: state(&r["state"]),
                read_roles: strings(&r["read_roles"]),
                direct_review_roles: strings(&r["direct_review_roles"]),
                purposes: strings(&r["purposes"]),
                grant_root_actions: strings(&r["grant_root_actions"]),
                source_policy_ids: strings(&r["source_policy_ids"]),
            })
            .collect(),
        source_policies: v["source_policies"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| SourcePolicy {
                policy: text(&r["policy_id"]),
                state: state(&r["state"]),
                expires_at: time(&r["expires_at"]),
                permitted_principals: r["permitted_principals"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(id)
                    .collect(),
                permitted_purposes: strings(&r["permitted_purposes"]),
            })
            .collect(),
        resources: v["resources"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| ResourceRecord {
                reference: reference(r),
                revision: revision(&r["control_revision"]),
                state: state(&r["state"]),
            })
            .collect(),
        grants: v["grants"].as_array().unwrap().iter().map(grant).collect(),
        immutable_chains: v["immutable_chains"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| ImmutableChain {
                id: id(&r["chain_id"]),
                scope_interpretation: text(&r["scope_interpretation"]),
                members: r["members"].as_array().unwrap().iter().map(grant).collect(),
            })
            .collect(),
    }
}

fn request(v: &Value) -> EvaluationRequest {
    EvaluationRequest {
        executor: id(&v["executor_principal_id"]),
        selected_workspace: (!v["selected_workspace"].is_null())
            .then(|| workspace(&v["selected_workspace"])),
        resource: reference(&v["resource"]),
        action: text(&v["action"]),
        purpose: text(&v["purpose"]),
        selected_chain: optional_id(&v["selected_chain_id"]),
        expected_control: (!v["expected_resource_control"].is_null())
            .then(|| revision(&v["expected_resource_control"])),
        claimed_authorizer: optional_id(&v["claimed_authorizer"]),
    }
}

fn workspace_output(v: Workspace) -> Value {
    json!({"organization_id":v.organization().to_string(), "domain_id":v.domain().to_string()})
}

fn control(v: ControlRevision) -> Value {
    json!({"kind":"control", "value":v.to_string()})
}

fn observation(v: AuthorityObservation<'_>) -> Value {
    let resource = v.resource();
    let resource_id = resource.workspace().require_reference(resource).unwrap();
    let resource_kind = match resource.kind() {
        ReferenceKind::Project => "project",
        _ => panic!("unsupported resource returned by actual evaluator"),
    };
    json!({
        "executor_principal_id":v.executor().to_string(),
        "workspace":workspace_output(v.workspace()),
        "action":v.action(), "purpose":v.purpose(),
        "resource":{"workspace":workspace_output(resource.workspace()), "kind":resource_kind, "id":resource_id.to_string()},
        "selected_chain_id":v.chain().map(|id| id.to_string()),
        "membership_id":v.membership().to_string(),
        "membership_revision":control(v.membership_revision()),
        "policy_epoch":control(v.policy_epoch()),
        "resource_control":control(v.resource_control()),
    })
}

/// Only the explicit runtime input projection enters this producer.
pub fn observe(input: &Value) -> Value {
    let projection = projection(&input["canonical_snapshot"]);
    let request = request(&input["request"]);
    let c = &input["configuration"];
    let config = PolicyConfiguration {
        supported_policies: strings(&c["supported_policies"]),
        supported_source_policies: strings(&c["supported_source_policies"]),
    };
    match evaluate(&projection, &config, &request) {
        Ok(value) => json!({"failure":null, "observation":observation(value)}),
        Err(failure) => json!({"failure":failure.to_string(), "observation":null}),
    }
}

struct FailingFormatter;

impl std::fmt::Write for FailingFormatter {
    fn write_str(&mut self, _value: &str) -> std::fmt::Result {
        Err(std::fmt::Error)
    }
}

pub fn error_observation(input: &Value) -> Value {
    let error = match input["variant"].as_str().unwrap() {
        "InvalidInput" => AuthorityFailure::InvalidInput,
        "InvalidStoredData" => AuthorityFailure::InvalidStoredData,
        "UnsupportedPolicy" => AuthorityFailure::UnsupportedPolicy,
        "UnsupportedAction" => AuthorityFailure::UnsupportedAction,
        "Denied" => AuthorityFailure::Denied,
        "Conflict" => AuthorityFailure::Conflict,
        _ => panic!("unadmitted error fixture"),
    };
    let boxed: Box<dyn Error> = Box::new(error);
    json!({
        "display":boxed.to_string(),
        "source_is_none":boxed.source().is_none(),
        "boxed_variant_preserved":boxed.downcast_ref::<AuthorityFailure>()==Some(&error),
        "formatter_failure_propagated":std::fmt::write(&mut FailingFormatter,format_args!("{error}")).is_err(),
    })
}
