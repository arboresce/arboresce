//! Four synthetic permission mutations share one private parent-first entry.

use arboresce_application::policy::authority::{
    CanonicalProjection, EvaluationRequest, RecordState,
};
use arboresce_domain::{Clock, ControlRevision, Identifier, UtcTimestamp};
use tokio::time::timeout_at;

use super::{AuthorityFailure as Failure, AuthoritySession, Guard, decision, read, rows};

pub struct PermissionSession {
    pub(crate) inner: AuthoritySession,
}

/// No principal/source creation, authority expansion, seal rewrite or raw SQL.
pub enum PermissionChange {
    RevokeMembership {
        principal: Identifier,
        expected: ControlRevision,
    },
    RevokeGrant {
        grant: Identifier,
        expected: ControlRevision,
    },
    RestrictPolicy {
        expected: ControlRevision,
        purposes: Vec<String>,
    },
    AdvanceResource {
        expected: ControlRevision,
    },
}

enum Resolved {
    Membership(Identifier, ControlRevision),
    Grant(Identifier, ControlRevision, UtcTimestamp),
    Policy(ControlRevision, Vec<String>),
    Resource(ControlRevision),
}

pub struct PermissionFence<'a> {
    guard: Guard<'a>,
    request: EvaluationRequest,
    change: Resolved,
}

impl PermissionChange {
    fn resolve(
        self,
        projection: &CanonicalProjection,
        request: &EvaluationRequest,
    ) -> Result<Resolved, Failure> {
        let workspace = request.selected_workspace.ok_or(Failure::InvalidRequest)?;
        match self {
            Self::RevokeMembership {
                principal,
                expected,
            } => {
                let row = projection
                    .memberships
                    .iter()
                    .find(|m| {
                        m.workspace == workspace
                            && m.principal.id == principal
                            && m.state == RecordState::Active
                            && m.revision == expected
                    })
                    .ok_or(Failure::Conflict)?;
                Ok(Resolved::Membership(
                    principal,
                    row.revision.checked_next().map_err(|_| Failure::Conflict)?,
                ))
            }
            Self::RevokeGrant { grant, expected } => {
                let row = projection
                    .grants
                    .iter()
                    .find(|g| {
                        g.id == grant
                            && g.state == RecordState::Active
                            && g.revoked_at.is_none()
                            && g.revision == expected
                    })
                    .ok_or(Failure::Conflict)?;
                Ok(Resolved::Grant(
                    grant,
                    row.revision.checked_next().map_err(|_| Failure::Conflict)?,
                    projection.now,
                ))
            }
            Self::RestrictPolicy { expected, purposes } => {
                let row = projection
                    .policies
                    .iter()
                    .find(|p| p.workspace == workspace && p.epoch == expected)
                    .ok_or(Failure::Conflict)?;
                if purposes.len() > 16
                    || purposes
                        .iter()
                        .enumerate()
                        .any(|(i, p)| !row.purposes.contains(p) || purposes[..i].contains(p))
                {
                    return Err(Failure::InvalidRequest);
                }
                Ok(Resolved::Policy(
                    row.epoch.checked_next().map_err(|_| Failure::Conflict)?,
                    purposes,
                ))
            }
            Self::AdvanceResource { expected } => {
                let row = projection
                    .resources
                    .iter()
                    .find(|r| r.reference == request.resource && r.revision == expected)
                    .ok_or(Failure::Conflict)?;
                Ok(Resolved::Resource(
                    row.revision.checked_next().map_err(|_| Failure::Conflict)?,
                ))
            }
        }
    }
}

impl PermissionSession {
    pub async fn backend_id(&mut self) -> Result<i32, Failure> {
        self.inner.backend_id().await
    }
    pub async fn close(self) -> Result<(), Failure> {
        self.inner.close().await
    }

    /// This isolated controller API does not assert product caller authority.
    pub async fn fence<'a>(
        &'a mut self,
        request: &EvaluationRequest,
        change: PermissionChange,
        clock: &mut impl Clock,
    ) -> Result<PermissionFence<'a>, Failure> {
        decision::shape(request)?;
        let mut guard = self.inner.begin(request, true).await?;
        let projection = timeout_at(
            guard.deadline,
            read::load(guard.connection(), request, true, true, clock),
        )
        .await
        .unwrap_or(Err(Failure::Unavailable));
        let resolved = projection.and_then(|p| change.resolve(&p, request));
        let change = match resolved {
            Ok(change) => change,
            Err(error) => {
                guard.finish(false).await?;
                return Err(error);
            }
        };
        let request = EvaluationRequest {
            executor: request.executor,
            selected_workspace: request.selected_workspace,
            resource: request.resource,
            action: request.action.clone(),
            purpose: request.purpose.clone(),
            selected_chain: request.selected_chain,
            expected_control: request.expected_control,
            claimed_authorizer: request.claimed_authorizer,
        };
        Ok(PermissionFence {
            guard,
            request,
            change,
        })
    }
}

impl PermissionFence<'_> {
    pub async fn rollback(self) -> Result<(), Failure> {
        self.guard.finish(false).await
    }

    pub async fn commit(mut self) -> Result<(), Failure> {
        let workspace = self
            .request
            .selected_workspace
            .ok_or(Failure::InvalidRequest)?;
        let org = workspace.organization().to_string();
        let domain = workspace.domain().to_string();
        let resource = rows::reference_id(self.request.resource)?.to_string();
        let kind = rows::resource_name(self.request.resource.kind());
        let result=timeout_at(self.guard.deadline,async {
            let connection=self.guard.connection();
            let changed=match self.change {
                Resolved::Membership(principal,revision)=>sqlx::query(
                    "UPDATE arboresce_authority.memberships SET state='revoked',control_revision=$4 \
                     WHERE organization_id=$1::uuid AND domain_id=$2::uuid AND principal_id=$3::uuid")
                    .bind(&org).bind(&domain).bind(principal.to_string()).bind(revision.value() as i64)
                    .execute(&mut *connection).await,
                Resolved::Grant(grant,revision,at)=>sqlx::query(
                    "UPDATE arboresce_authority.grants SET state='revoked',control_revision=$4,revoked_at=$5::timestamptz \
                     WHERE organization_id=$1::uuid AND domain_id=$2::uuid AND grant_id=$3::uuid")
                    .bind(&org).bind(&domain).bind(grant.to_string()).bind(revision.value() as i64).bind(at.to_string())
                    .execute(&mut *connection).await,
                Resolved::Policy(revision,purposes)=>sqlx::query(
                    "UPDATE arboresce_authority.workspace_policies SET epoch=$3,purposes=$4 \
                     WHERE organization_id=$1::uuid AND domain_id=$2::uuid")
                    .bind(&org).bind(&domain).bind(revision.value() as i64).bind(purposes)
                    .execute(&mut *connection).await,
                Resolved::Resource(revision)=>sqlx::query(
                    "UPDATE arboresce_authority.resources SET control_revision=$5 \
                     WHERE organization_id=$1::uuid AND domain_id=$2::uuid AND resource_kind=$3 AND resource_id=$4::uuid")
                    .bind(&org).bind(&domain).bind(kind).bind(&resource).bind(revision.value() as i64)
                    .execute(&mut *connection).await,
            }.map_err(decision::write_failure)?;
            if changed.rows_affected()!=1 {return Err(Failure::Conflict);}
            let epoch:i64=sqlx::query_scalar("SELECT epoch FROM arboresce_authority.namespace_guard WHERE singleton=1")
                .fetch_one(&mut *connection).await.map_err(|_|Failure::Unavailable)?;
            let epoch:ControlRevision=epoch.to_string().parse().map_err(|_|Failure::InvalidStoredData)?;
            let next=epoch.checked_next().map_err(|_|Failure::Conflict)?;
            let changed=sqlx::query("UPDATE arboresce_authority.namespace_guard SET epoch=$1 WHERE singleton=1 AND epoch=$2")
                .bind(next.value() as i64).bind(epoch.value() as i64).execute(connection).await
                .map_err(decision::write_failure)?;
            if changed.rows_affected()!=1 {return Err(Failure::Conflict);}
            Ok(())
        }).await.unwrap_or(Err(Failure::Unavailable));
        if let Err(error) = result {
            self.guard.finish(false).await?;
            return Err(error);
        }
        self.guard.finish(true).await
    }
}
