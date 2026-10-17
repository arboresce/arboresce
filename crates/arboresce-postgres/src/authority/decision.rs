//! Internal disclosure and complete permission precede caller CAS disclosure.

use arboresce_application::policy::authority::{
    self, AuthorityObservation, CanonicalProjection, EvaluationRequest, GrantScope,
    PolicyConfiguration, PrincipalKind, RecordState,
};
use arboresce_application::policy::principal::IdentityKey;
use arboresce_domain::{Clock, Identifier};
use tokio::time::timeout_at;

use super::{AuthorityFailure as Failure, AuthoritySession, Guard, read, rows};

pub(crate) fn shape(request: &EvaluationRequest) -> Result<(), Failure> {
    let token = |s: &str| {
        !s.is_empty()
            && s.len() <= 96
            && s.bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'.' | b'-'))
    };
    if request.selected_workspace.is_none()
        || request.claimed_authorizer.is_some()
        || !token(&request.action)
        || !token(&request.purpose)
    {
        return Err(Failure::InvalidRequest);
    }
    Ok(())
}

fn current<'r>(
    projection: &CanonicalProjection,
    config: &PolicyConfiguration,
    request: &'r EvaluationRequest,
) -> Result<AuthorityObservation<'r>, Failure> {
    let principal = projection
        .principals
        .iter()
        .find(|p| p.principal.id == request.executor)
        .ok_or(Failure::Unauthenticated)?;
    if principal.state != RecordState::Active
        || principal.revoked_at.is_some()
        || principal.created_at > projection.now
        || principal.principal.kind == PrincipalKind::Unsupported
        || IdentityKey::parse(
            &principal.identity_profile,
            &principal.identity_version,
            &principal.public_key,
        )
        .is_err()
    {
        return Err(Failure::Unauthenticated);
    }
    let workspace = request.selected_workspace.ok_or(Failure::InvalidRequest)?;
    if request.resource.workspace() != workspace
        || !projection
            .organizations
            .iter()
            .any(|o| o.id == workspace.organization() && o.state == RecordState::Active)
        || !projection
            .domains
            .iter()
            .any(|d| d.workspace == workspace && d.state == RecordState::Active)
    {
        return Err(Failure::NotFound);
    }
    let policy = projection
        .policies
        .iter()
        .find(|p| {
            p.workspace == workspace
                && p.state == RecordState::Active
                && config.supported_policies.contains(&p.policy)
        })
        .ok_or(Failure::NotFound)?;
    let membership = projection
        .memberships
        .iter()
        .find(|m| {
            m.workspace == workspace
                && m.principal == principal.principal
                && m.state == RecordState::Active
                && m.policy == policy.policy
        })
        .ok_or(Failure::NotFound)?;
    if !membership
        .roles
        .iter()
        .any(|r| policy.read_roles.contains(r))
    {
        return Err(Failure::NotFound);
    }
    let resource = projection
        .resources
        .iter()
        .find(|r| r.reference == request.resource && r.state == RecordState::Active)
        .ok_or(Failure::NotFound)?;
    for id in &policy.source_policy_ids {
        if !config.supported_source_policies.contains(id) {
            return Err(Failure::NotFound);
        }
        if !projection.source_policies.iter().any(|s| {
            s.policy == *id
                && s.state == RecordState::Active
                && projection.now < s.expires_at
                && s.permitted_principals.contains(&request.executor)
        }) {
            return Err(Failure::NotFound);
        }
        // Purpose is action permission; only current audience is disclosure.
    }
    if let Some(chain) = request
        .selected_chain
        .and_then(|id| projection.immutable_chains.iter().find(|c| c.id == id))
        && chain.scope_interpretation == "exact_project_v1"
        && let Some(root) = chain.members.first().filter(|g| g.parent.is_none())
        && let GrantScope::Exact(reference) = root.scope
        && reference != request.resource
    {
        return Err(Failure::NotFound);
    }
    // The pure evaluator checks CAS before the chain. Neutralize only its
    // intermediate check with the actual locked control; discard this result.
    // Original request/control remains unchanged and is evaluated last again.
    let permission_request = EvaluationRequest {
        executor: request.executor,
        selected_workspace: request.selected_workspace,
        resource: request.resource,
        action: request.action.clone(),
        purpose: request.purpose.clone(),
        selected_chain: request.selected_chain,
        expected_control: Some(resource.revision),
        claimed_authorizer: request.claimed_authorizer,
    };
    authority::evaluate(projection, config, &permission_request).map_err(|_| Failure::Forbidden)?;
    if (request.action == "project.change.review" && request.expected_control.is_none())
        || request
            .expected_control
            .is_some_and(|r| r != resource.revision)
    {
        return Err(Failure::Conflict);
    }
    authority::evaluate(projection, config, request).map_err(|_| Failure::Forbidden)
}

/// A live fence in one owned transaction, never a reusable authorization token.
pub struct AdmissionFence<'s, 'r, 'c, C: Clock> {
    guard: Guard<'s>,
    observation: AuthorityObservation<'r>,
    request: &'r EvaluationRequest,
    configuration: &'r PolicyConfiguration,
    clock: &'c mut C,
}

impl AuthoritySession {
    pub async fn observe_current<'r>(
        &mut self,
        request: &'r EvaluationRequest,
        config: &PolicyConfiguration,
        clock: &mut impl Clock,
    ) -> Result<AuthorityObservation<'r>, Failure> {
        shape(request)?;
        let mut guard = self.begin(request, false).await?;
        let result = timeout_at(
            guard.deadline,
            read::load(guard.connection(), request, false, false, clock),
        )
        .await
        .unwrap_or(Err(Failure::Unavailable))
        .and_then(|projection| current(&projection, config, request));
        guard.finish(false).await?;
        result
    }

    pub async fn fence<'s, 'r, 'c, C: Clock>(
        &'s mut self,
        request: &'r EvaluationRequest,
        config: &'r PolicyConfiguration,
        clock: &'c mut C,
    ) -> Result<AdmissionFence<'s, 'r, 'c, C>, Failure> {
        shape(request)?;
        let mut guard = self.begin(request, false).await?;
        let result = timeout_at(
            guard.deadline,
            read::load(guard.connection(), request, false, true, clock),
        )
        .await
        .unwrap_or(Err(Failure::Unavailable));
        let projection = match result {
            Ok(p) => p,
            Err(error) => {
                guard.finish(false).await?;
                return Err(error);
            }
        };
        let observation = match current(&projection, config, request) {
            Ok(observation) => observation,
            Err(error) => {
                guard.finish(false).await?;
                return Err(error);
            }
        };
        Ok(AdmissionFence {
            guard,
            observation,
            request,
            configuration: config,
            clock,
        })
    }
}

impl<'r, C: Clock> AdmissionFence<'_, 'r, '_, C> {
    pub const fn observation(&self) -> AuthorityObservation<'r> {
        self.observation
    }

    pub async fn rollback(self) -> Result<(), Failure> {
        self.guard.finish(false).await
    }

    /// Commit one preallocated synthetic marker under this live current fence.
    pub async fn commit_marker(
        self,
        marker: Identifier,
        observation: Identifier,
    ) -> Result<(), Failure> {
        self.effect(marker, observation, false).await
    }

    /// Atomically advance the exact current control and insert its one marker.
    pub async fn compare_and_marker(
        self,
        marker: Identifier,
        observation: Identifier,
    ) -> Result<(), Failure> {
        self.effect(marker, observation, true).await
    }

    async fn effect(
        mut self,
        marker: Identifier,
        observation: Identifier,
        advance: bool,
    ) -> Result<(), Failure> {
        // Locks fence mutable authority, but cannot fence time. Retain the
        // originating trusted clock and perform the complete post-lock read
        // and current decision again immediately before every effect.
        let fresh = timeout_at(
            self.guard.deadline,
            read::load(
                self.guard.connection(),
                self.request,
                false,
                true,
                &mut *self.clock,
            ),
        )
        .await
        .unwrap_or(Err(Failure::Unavailable));
        let fresh = fresh.and_then(|projection| {
            let observation = current(&projection, self.configuration, self.request)?;
            let grant_revision = self
                .request
                .selected_chain
                .and_then(|id| projection.immutable_chains.iter().find(|c| c.id == id))
                .and_then(|c| c.members.last())
                .map(|g| g.revision);
            Ok((observation, grant_revision))
        });
        let (authority, grant_revision) = match fresh {
            Ok(values) => values,
            Err(error) => {
                self.guard.finish(false).await?;
                return Err(error);
            }
        };
        if advance && authority.action() != "project.change.review" {
            self.guard.finish(false).await?;
            return Err(Failure::Forbidden);
        }
        let next = if advance {
            if self.request.expected_control != Some(authority.resource_control()) {
                self.guard.finish(false).await?;
                return Err(Failure::Conflict);
            }
            match authority.resource_control().checked_next() {
                Ok(value) => Some(value),
                Err(_) => {
                    self.guard.finish(false).await?;
                    return Err(Failure::Conflict);
                }
            }
        } else {
            None
        };
        let grant_revision = grant_revision.map(|r| r.value() as i64);
        let result=timeout_at(self.guard.deadline,async {
            let connection=self.guard.connection();
            if let Some(next)=next {
                let changed=sqlx::query("UPDATE arboresce_authority.resources SET control_revision=$5 \
                    WHERE organization_id=$1::uuid AND domain_id=$2::uuid AND resource_kind=$3 \
                      AND resource_id=$4::uuid AND control_revision=$6")
                    .bind(authority.workspace().organization().to_string())
                    .bind(authority.workspace().domain().to_string())
                    .bind(rows::resource_name(authority.resource().kind()))
                    .bind(rows::reference_id(authority.resource())?.to_string())
                    .bind(next.value() as i64).bind(authority.resource_control().value() as i64)
                    .execute(&mut *connection).await.map_err(write_failure)?;
                if changed.rows_affected()!=1 {return Err(Failure::Conflict);}
            }
            sqlx::query("INSERT INTO arboresce_authority.qualification_markers \
                (organization_id,domain_id,marker_id,observation_id,executor_principal_id,selected_chain_id,\
                 action,purpose,resource_kind,resource_id,admitted_policy_epoch,admitted_membership_revision,\
                 admitted_grant_revision,admitted_resource_control) \
                VALUES ($1::uuid,$2::uuid,$3::uuid,$4::uuid,$5::uuid,$6::uuid,$7,$8,$9,$10::uuid,$11,$12,$13,$14)")
                .bind(authority.workspace().organization().to_string()).bind(authority.workspace().domain().to_string())
                .bind(marker.to_string()).bind(observation.to_string()).bind(authority.executor().to_string())
                .bind(authority.chain().map(|id|id.to_string())).bind(authority.action()).bind(authority.purpose())
                .bind(rows::resource_name(authority.resource().kind())).bind(rows::reference_id(authority.resource())?.to_string())
                .bind(authority.policy_epoch().value() as i64).bind(authority.membership_revision().value() as i64)
                .bind(grant_revision).bind(authority.resource_control().value() as i64)
                .execute(connection).await.map_err(write_failure)?;
            Ok(())
        }).await.unwrap_or(Err(Failure::Unavailable));
        if let Err(error) = result {
            self.guard.finish(false).await?;
            return Err(error);
        }
        self.guard.finish(true).await
    }
}

pub(crate) fn write_failure(error: sqlx::Error) -> Failure {
    if error.as_database_error().and_then(|e| e.code()).as_deref() == Some("23505") {
        Failure::Conflict
    } else {
        Failure::Unavailable
    }
}
