//! Bounded policy decisions over trusted canonical projections.
//!
//! The caller supplies server-derived identity, explicit scope and canonical
//! records. No request may supply this projection or trusted configuration.
//! A successful observation is not authentication or a database/effect fence.
//! Adapters must separately qualify bounded decoding, current reads and locks.

use std::{error::Error, fmt};

use arboresce_domain::{ControlRevision, Identifier, UtcTimestamp};

use super::{ReferenceKind, ScopedReference, Workspace, principal::IdentityKey};

const ROWS: usize = 32;
const SET: usize = 16;
const CHAIN: usize = 8;
const TOKEN: usize = 96;

/// Finite failures contain no supplied identity, resource or backend cause.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthorityFailure {
    InvalidInput,
    InvalidStoredData,
    UnsupportedPolicy,
    UnsupportedAction,
    Denied,
    Conflict,
}

impl fmt::Display for AuthorityFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl Error for AuthorityFailure {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PrincipalKind {
    Human,
    Agent,
    Service,
    Unsupported,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecordState {
    Active,
    Inactive,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PrincipalReference {
    pub id: Identifier,
    pub kind: PrincipalKind,
}

#[derive(Debug)]
pub struct OrganizationRecord {
    pub id: Identifier,
    pub state: RecordState,
}

#[derive(Debug)]
pub struct DomainRecord {
    pub workspace: Workspace,
    pub state: RecordState,
}

/// Stored representation only; an exact supported key is not key-control proof.
#[derive(Debug)]
pub struct PrincipalRecord {
    pub principal: PrincipalReference,
    pub identity_profile: String,
    pub identity_version: String,
    pub public_key: String,
    pub revision: ControlRevision,
    pub state: RecordState,
    pub created_at: UtcTimestamp,
    pub revoked_at: Option<UtcTimestamp>,
}

#[derive(Debug)]
pub struct MembershipRecord {
    pub id: Identifier,
    pub principal: PrincipalReference,
    pub workspace: Workspace,
    pub roles: Vec<String>,
    pub policy: String,
    pub revision: ControlRevision,
    pub state: RecordState,
}

#[derive(Debug)]
pub struct WorkspacePolicy {
    pub workspace: Workspace,
    pub policy: String,
    pub epoch: ControlRevision,
    pub state: RecordState,
    pub read_roles: Vec<String>,
    pub direct_review_roles: Vec<String>,
    pub purposes: Vec<String>,
    pub grant_root_actions: Vec<String>,
    pub source_policy_ids: Vec<String>,
}

#[derive(Debug)]
pub struct SourcePolicy {
    pub policy: String,
    pub state: RecordState,
    pub expires_at: UtcTimestamp,
    pub permitted_principals: Vec<Identifier>,
    pub permitted_purposes: Vec<String>,
}

#[derive(Debug)]
pub struct ResourceRecord {
    pub reference: ScopedReference,
    pub revision: ControlRevision,
    pub state: RecordState,
}

/// Broader scopes remain representable so exact-project admission can deny them.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GrantScope {
    Exact(ScopedReference),
    Workspace(Workspace),
}

#[derive(Debug, Eq, PartialEq)]
pub struct GrantRecord {
    pub id: Identifier,
    pub issuer: PrincipalReference,
    pub subject: PrincipalReference,
    pub parent: Option<Identifier>,
    pub scope: GrantScope,
    pub actions: Vec<String>,
    pub purposes: Vec<String>,
    pub delegable: bool,
    pub issued_at: UtcTimestamp,
    pub expires_at: UtcTimestamp,
    pub policy: String,
    pub revision: ControlRevision,
    pub state: RecordState,
    pub revoked_at: Option<UtcTimestamp>,
}

#[derive(Debug)]
pub struct ImmutableChain {
    pub id: Identifier,
    pub scope_interpretation: String,
    pub members: Vec<GrantRecord>,
}

/// A bounded projection, complete for every selected/participating required row.
///
/// This is not a global database export. Shape and uniqueness checks apply to
/// all included rows, but unrelated foreign references/policies are not resolved.
/// These owned decoded records preexist evaluation; adapters must bound their
/// reads and allocations. Evaluation borrows them and allocates no heap.
#[derive(Debug)]
pub struct CanonicalProjection {
    pub now: UtcTimestamp,
    pub organizations: Vec<OrganizationRecord>,
    pub domains: Vec<DomainRecord>,
    pub principals: Vec<PrincipalRecord>,
    pub memberships: Vec<MembershipRecord>,
    pub policies: Vec<WorkspacePolicy>,
    pub source_policies: Vec<SourcePolicy>,
    pub resources: Vec<ResourceRecord>,
    pub grants: Vec<GrantRecord>,
    pub immutable_chains: Vec<ImmutableChain>,
}

/// Trusted deployment selection of the profile's compiled policy meanings.
///
/// Fixture names are supplied by tests, never selected by production defaults.
/// This configuration must not come from an external request.
#[derive(Debug)]
pub struct PolicyConfiguration {
    pub supported_policies: Vec<String>,
    pub supported_source_policies: Vec<String>,
}

/// Normalized server-internal input, without a verifier or authority flag.
///
/// The executor originates at the separately qualified authentication boundary.
/// Cached authority, aliases and names are deliberately absent.
#[derive(Debug)]
pub struct EvaluationRequest {
    pub executor: Identifier,
    pub selected_workspace: Option<Workspace>,
    pub resource: ScopedReference,
    pub action: String,
    pub purpose: String,
    pub selected_chain: Option<Identifier>,
    pub expected_control: Option<ControlRevision>,
    pub claimed_authorizer: Option<Identifier>,
}

/// An immutable observation of this pure decision; never a reusable capability.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AuthorityObservation<'a> {
    executor: Identifier,
    workspace: Workspace,
    resource: ScopedReference,
    action: &'a str,
    purpose: &'a str,
    chain: Option<Identifier>,
    membership: Identifier,
    membership_revision: ControlRevision,
    policy_epoch: ControlRevision,
    resource_control: ControlRevision,
}

impl<'a> AuthorityObservation<'a> {
    pub const fn executor(self) -> Identifier {
        self.executor
    }
    pub const fn workspace(self) -> Workspace {
        self.workspace
    }
    pub const fn resource(self) -> ScopedReference {
        self.resource
    }
    pub const fn action(self) -> &'a str {
        self.action
    }
    pub const fn purpose(self) -> &'a str {
        self.purpose
    }
    pub const fn chain(self) -> Option<Identifier> {
        self.chain
    }
    pub const fn membership(self) -> Identifier {
        self.membership
    }
    pub const fn membership_revision(self) -> ControlRevision {
        self.membership_revision
    }
    pub const fn policy_epoch(self) -> ControlRevision {
        self.policy_epoch
    }
    pub const fn resource_control(self) -> ControlRevision {
        self.resource_control
    }
}

fn token(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= TOKEN
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'.' | b'-'))
}

fn unique<T>(rows: &[T], equal: impl Fn(&T, &T) -> bool) -> bool {
    rows.iter()
        .enumerate()
        .all(|(i, row)| !rows[..i].iter().any(|other| equal(row, other)))
}

fn tokens(values: &[String]) -> bool {
    values.len() <= SET && values.iter().all(|s| token(s)) && unique(values, |a, b| a == b)
}

fn grant_shape(g: &GrantRecord) -> bool {
    token(&g.policy)
        && tokens(&g.actions)
        && tokens(&g.purposes)
        && g.issuer.kind != PrincipalKind::Unsupported
        && g.subject.kind != PrincipalKind::Unsupported
}

impl CanonicalProjection {
    fn validate(&self, config: &PolicyConfiguration) -> Result<(), AuthorityFailure> {
        let counts = [
            self.organizations.len(),
            self.domains.len(),
            self.principals.len(),
            self.memberships.len(),
            self.policies.len(),
            self.source_policies.len(),
            self.resources.len(),
            self.grants.len(),
            self.immutable_chains.len(),
        ];
        if counts.iter().any(|n| *n > ROWS)
            || !tokens(&config.supported_policies)
            || !tokens(&config.supported_source_policies)
        {
            return Err(AuthorityFailure::InvalidStoredData);
        }
        // Validate bounds before pair comparisons or traversing nested records.
        if self.principals.iter().any(|p| {
            !token(&p.identity_profile)
                || !token(&p.identity_version)
                || p.public_key.len() != 64
                || !p.public_key.is_ascii()
                || p.principal.kind == PrincipalKind::Unsupported
        }) || self.memberships.iter().any(|m| {
            !token(&m.policy) || !tokens(&m.roles) || m.principal.kind == PrincipalKind::Unsupported
        }) || self.policies.iter().any(|p| {
            !token(&p.policy)
                || !tokens(&p.read_roles)
                || !tokens(&p.direct_review_roles)
                || !tokens(&p.purposes)
                || !tokens(&p.grant_root_actions)
                || !tokens(&p.source_policy_ids)
        }) || self.source_policies.iter().any(|p| {
            !token(&p.policy)
                || !tokens(&p.permitted_purposes)
                || p.permitted_principals.len() > ROWS
                || !unique(&p.permitted_principals, |a, b| a == b)
        }) || self.grants.iter().any(|g| !grant_shape(g))
            || self.immutable_chains.iter().any(|c| {
                !token(&c.scope_interpretation)
                    || c.members.is_empty()
                    || c.members.len() > CHAIN
                    || c.members.iter().any(|g| !grant_shape(g))
            })
        {
            return Err(AuthorityFailure::InvalidStoredData);
        }
        if !unique(&self.organizations, |a, b| a.id == b.id)
            || !unique(&self.domains, |a, b| a.workspace == b.workspace)
            || !unique(&self.principals, |a, b| {
                a.principal.id == b.principal.id
                    || (a.identity_profile == b.identity_profile
                        && a.identity_version == b.identity_version
                        && a.public_key == b.public_key)
            })
            || !unique(&self.memberships, |a, b| {
                a.workspace.organization() == b.workspace.organization()
                    && (a.principal.id == b.principal.id || a.id == b.id)
            })
            || !unique(&self.policies, |a, b| a.workspace == b.workspace)
            || !unique(&self.source_policies, |a, b| a.policy == b.policy)
            || !unique(&self.resources, |a, b| a.reference == b.reference)
            || !unique(&self.grants, |a, b| a.id == b.id)
            || !unique(&self.immutable_chains, |a, b| a.id == b.id)
            || self
                .immutable_chains
                .iter()
                .any(|c| !unique(&c.members, |a, b| a.id == b.id))
        {
            return Err(AuthorityFailure::InvalidStoredData);
        }
        Ok(())
    }

    fn principal(&self, id: Identifier) -> Result<&PrincipalRecord, AuthorityFailure> {
        let p = self
            .principals
            .iter()
            .find(|p| p.principal.id == id)
            .ok_or(AuthorityFailure::Denied)?;
        if p.state != RecordState::Active || p.revoked_at.is_some() || p.created_at > self.now {
            return Err(AuthorityFailure::Denied);
        }
        IdentityKey::parse(&p.identity_profile, &p.identity_version, &p.public_key)
            .map_err(|_| AuthorityFailure::InvalidStoredData)?;
        Ok(p)
    }

    fn membership(
        &self,
        p: PrincipalReference,
        policy: &WorkspacePolicy,
    ) -> Result<&MembershipRecord, AuthorityFailure> {
        let canonical = self.principal(p.id)?;
        if canonical.principal != p {
            return Err(AuthorityFailure::InvalidStoredData);
        }
        let m = self
            .memberships
            .iter()
            .find(|m| {
                m.workspace.organization() == policy.workspace.organization()
                    && m.principal.id == p.id
            })
            .ok_or(AuthorityFailure::Denied)?;
        if m.principal != canonical.principal {
            return Err(AuthorityFailure::InvalidStoredData);
        }
        if m.workspace != policy.workspace || m.state != RecordState::Active {
            return Err(AuthorityFailure::Denied);
        }
        if m.policy != policy.policy {
            return Err(AuthorityFailure::UnsupportedPolicy);
        }
        Ok(m)
    }

    fn sources(
        &self,
        config: &PolicyConfiguration,
        policy: &WorkspacePolicy,
        principal: Identifier,
        purpose: &str,
    ) -> Result<(), AuthorityFailure> {
        for id in &policy.source_policy_ids {
            if !config.supported_source_policies.contains(id) {
                return Err(AuthorityFailure::UnsupportedPolicy);
            }
            let p = self
                .source_policies
                .iter()
                .find(|p| p.policy == *id)
                .ok_or(AuthorityFailure::UnsupportedPolicy)?;
            if p.state != RecordState::Active {
                return Err(AuthorityFailure::UnsupportedPolicy);
            }
            if self.now >= p.expires_at
                || !p.permitted_principals.contains(&principal)
                || !p.permitted_purposes.iter().any(|p| p == purpose)
            {
                return Err(AuthorityFailure::Denied);
            }
        }
        Ok(())
    }

    fn chain(
        &self,
        id: Identifier,
        config: &PolicyConfiguration,
        policy: &WorkspacePolicy,
        request: &EvaluationRequest,
    ) -> Result<(), AuthorityFailure> {
        let chain = self
            .immutable_chains
            .iter()
            .find(|c| c.id == id)
            .ok_or(AuthorityFailure::Denied)?;
        if chain.scope_interpretation != "exact_project_v1" {
            return Err(AuthorityFailure::UnsupportedPolicy);
        }
        let mut previous: Option<&GrantRecord> = None;
        for g in &chain.members {
            let current = self
                .grants
                .iter()
                .find(|r| r.id == g.id)
                .ok_or(AuthorityFailure::Denied)?;
            if current != g
                || g.state != RecordState::Active
                || g.revoked_at.is_some()
                || g.issued_at > self.now
                || self.now >= g.expires_at
                || g.issued_at >= g.expires_at
                || g.scope != GrantScope::Exact(request.resource)
                || g.actions.is_empty()
                || g.purposes.is_empty()
                || !g.actions.contains(&request.action)
                || !g.purposes.contains(&request.purpose)
            {
                return Err(AuthorityFailure::Denied);
            }
            if g.policy != policy.policy {
                return Err(AuthorityFailure::UnsupportedPolicy);
            }
            let issuer = self.membership(g.issuer, policy)?;
            self.membership(g.subject, policy)?;
            for purpose in &g.purposes {
                if !policy.purposes.contains(purpose) {
                    return Err(AuthorityFailure::Denied);
                }
                self.sources(config, policy, g.issuer.id, purpose)?;
                self.sources(config, policy, g.subject.id, purpose)?;
            }
            if let Some(parent) = previous {
                if g.parent != Some(parent.id)
                    || g.issuer != parent.subject
                    || !parent.delegable
                    || g.scope != parent.scope
                    || g.issued_at < parent.issued_at
                    || g.expires_at > parent.expires_at
                    || !g.actions.iter().all(|a| parent.actions.contains(a))
                    || !g.purposes.iter().all(|p| parent.purposes.contains(p))
                {
                    return Err(AuthorityFailure::Denied);
                }
            } else if g.parent.is_some()
                || !g.actions.iter().all(|a| {
                    policy.grant_root_actions.contains(a)
                        && direct(g.issuer.kind, issuer, policy, a)
                })
            {
                return Err(AuthorityFailure::Denied);
            }
            previous = Some(g);
        }
        if previous.map(|g| g.subject.id) != Some(request.executor) {
            return Err(AuthorityFailure::Denied);
        }
        Ok(())
    }
}

fn direct(
    kind: PrincipalKind,
    membership: &MembershipRecord,
    policy: &WorkspacePolicy,
    action: &str,
) -> bool {
    match action {
        "project.read" => membership
            .roles
            .iter()
            .any(|r| policy.read_roles.contains(r)),
        "project.change.review" => {
            kind == PrincipalKind::Human
                && membership
                    .roles
                    .iter()
                    .any(|r| policy.direct_review_roles.contains(r))
        }
        _ => false,
    }
}

/// Evaluate current supplied records without performing or admitting an effect.
///
/// Call again with fresh canonical records in a separately qualified conflicting
/// database fence before an effect. A prior observation supplies no authority.
pub fn evaluate<'a>(
    projection: &CanonicalProjection,
    config: &PolicyConfiguration,
    request: &'a EvaluationRequest,
) -> Result<AuthorityObservation<'a>, AuthorityFailure> {
    if !token(&request.action) || !token(&request.purpose) {
        return Err(AuthorityFailure::InvalidInput);
    }
    if !matches!(
        request.action.as_str(),
        "project.read" | "project.change.review"
    ) {
        return Err(AuthorityFailure::UnsupportedAction);
    }
    if request.claimed_authorizer.is_some() {
        return Err(AuthorityFailure::Denied);
    }
    projection.validate(config)?;
    let workspace = request.selected_workspace.ok_or(AuthorityFailure::Denied)?;
    if request.resource.workspace() != workspace
        || request.resource.kind() != ReferenceKind::Project
    {
        return Err(AuthorityFailure::Denied);
    }
    if !projection
        .organizations
        .iter()
        .any(|o| o.id == workspace.organization() && o.state == RecordState::Active)
        || !projection
            .domains
            .iter()
            .any(|d| d.workspace == workspace && d.state == RecordState::Active)
    {
        return Err(AuthorityFailure::Denied);
    }
    let policy = projection
        .policies
        .iter()
        .find(|p| p.workspace == workspace)
        .ok_or(AuthorityFailure::UnsupportedPolicy)?;
    if policy.state != RecordState::Active || !config.supported_policies.contains(&policy.policy) {
        return Err(AuthorityFailure::UnsupportedPolicy);
    }
    let principal = projection.principal(request.executor)?;
    let membership = projection.membership(principal.principal, policy)?;
    if !policy.purposes.contains(&request.purpose) {
        return Err(AuthorityFailure::Denied);
    }
    let resource = projection
        .resources
        .iter()
        .find(|r| r.reference == request.resource)
        .ok_or(AuthorityFailure::Denied)?;
    if resource.state != RecordState::Active {
        return Err(AuthorityFailure::Denied);
    }
    if (request.action == "project.change.review" && request.expected_control.is_none())
        || request
            .expected_control
            .is_some_and(|r| r != resource.revision)
    {
        return Err(AuthorityFailure::Conflict);
    }
    projection.sources(config, policy, request.executor, &request.purpose)?;
    if let Some(id) = request.selected_chain {
        projection.chain(id, config, policy, request)?;
    } else if !direct(
        principal.principal.kind,
        membership,
        policy,
        &request.action,
    ) {
        return Err(AuthorityFailure::Denied);
    }
    Ok(AuthorityObservation {
        executor: request.executor,
        workspace,
        resource: request.resource,
        action: &request.action,
        purpose: &request.purpose,
        chain: request.selected_chain,
        membership: membership.id,
        membership_revision: membership.revision,
        policy_epoch: policy.epoch,
        resource_control: resource.revision,
    })
}
