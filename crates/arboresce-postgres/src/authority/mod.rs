//! Current authority in the explicitly prepared direct synthetic namespace.
//!
//! These sessions enable isolated markers and control changes only. They do not
//! authenticate a request, enable product writers, or qualify a production pool.

use std::{sync::Arc, time::Duration};

use sqlx::{Connection, PgConnection, Postgres, Transaction};
use tokio::{
    sync::{OwnedSemaphorePermit, Semaphore},
    time::{Instant, timeout, timeout_at},
};

use crate::Harness;
use arboresce_application::policy::{TenantContext, authority::EvaluationRequest};
use arboresce_domain::{ControlRevision, Identifier};

pub(crate) mod catalogue;
pub(crate) mod catalogue_policy;
pub(crate) mod decision;
pub(crate) mod permission;
pub(crate) mod read;
pub(crate) mod rows;

pub use decision::AdmissionFence;
pub use permission::{PermissionChange, PermissionFence, PermissionSession};

const BUDGET: Duration = Duration::from_millis(2000);
const CONNECT: Duration = Duration::from_millis(1000);

/// Finite diagnostics never contain identities, backend SQL or an inner cause.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthorityFailure {
    InvalidRequest,
    Unauthenticated,
    NotFound,
    Forbidden,
    Conflict,
    InvalidStoredData,
    Unavailable,
    UnsafeRole,
    RetainedContext,
    UnknownCompletion,
}

impl std::fmt::Display for AuthorityFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

impl std::error::Error for AuthorityFailure {}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum Role {
    Admission,
    Permission,
}

impl Role {
    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::Admission => "harness_authority_admission",
            Self::Permission => "harness_authority_permission",
        }
    }
}

/// One finite direct session set, shared by admission and permission writers.
///
/// The selected database also has CONNECTION LIMIT 2. Separate factories do not
/// establish a cross-process pool bound; actual backend census remains required.
pub struct AuthoritySessions {
    harness: Harness,
    slots: Arc<Semaphore>,
}

/// One owned physical admission connection; its raw connection is not exported.
pub struct AuthoritySession {
    pub(crate) connection: Option<PgConnection>,
    role: Role,
    slot: Slot,
    poisoned: bool,
}

struct Slot(Option<OwnedSemaphorePermit>);

impl Drop for Slot {
    fn drop(&mut self) {
        if let Some(slot) = self.0.take() {
            // Also covers cancellation during handshake, validation and close.
            slot.forget();
        }
    }
}

/// A finite failure at the owned transport completion boundary.
#[derive(Debug)]
pub(crate) enum TransportCloseFailure {
    Deadline,
    Task(tokio::task::JoinError),
}

impl std::fmt::Display for TransportCloseFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Deadline => "owned transport close deadline exceeded",
            Self::Task(_) => "owned transport close task failed",
        })
    }
}

impl std::error::Error for TransportCloseFailure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Deadline => None,
            Self::Task(error) => Some(error),
        }
    }
}

/// Await the actual owned destructor task and recheck its original deadline.
/// A ready JoinHandle alone cannot prove timely completion.
pub(crate) async fn close_completion(
    task: &mut tokio::task::JoinHandle<()>,
    deadline: Instant,
) -> Result<(), TransportCloseFailure> {
    match timeout_at(deadline, task).await {
        Ok(Ok(())) if Instant::now() <= deadline => Ok(()),
        Ok(Ok(())) | Err(_) => Err(TransportCloseFailure::Deadline),
        Ok(Err(error)) => Err(TransportCloseFailure::Task(error)),
    }
}

/// Destroy this physical transport without polling further driver I/O.
/// In particular, do not flush a rollback queued after an uncertain terminal.
pub(crate) async fn close_transport(
    connection: PgConnection,
    deadline: Instant,
) -> Result<(), TransportCloseFailure> {
    let mut task = tokio::task::spawn_blocking(move || drop(connection));
    close_completion(&mut task, deadline).await
}

impl Harness {
    pub fn authority_sessions(&self) -> AuthoritySessions {
        AuthoritySessions {
            harness: Harness {
                socket: self.socket.clone(),
            },
            slots: Arc::new(Semaphore::new(2)),
        }
    }
}

impl AuthoritySessions {
    /// Availability in this one finite session set, not database/global capacity
    /// or an authorization decision. Native qualification also censuses actual
    /// backends and observes the exact owned close independently.
    pub fn available_slots(&self) -> usize {
        self.slots.available_permits()
    }

    pub async fn admission(&self) -> Result<AuthoritySession, AuthorityFailure> {
        self.open(Role::Admission).await
    }

    pub async fn permission(&self) -> Result<PermissionSession, AuthorityFailure> {
        Ok(PermissionSession {
            inner: self.open(Role::Permission).await?,
        })
    }

    async fn open(&self, role: Role) -> Result<AuthoritySession, AuthorityFailure> {
        let options = self
            .harness
            .options(false)
            .map_err(|_| AuthorityFailure::Unavailable)?
            .username(role.name())
            .application_name(role.name());
        let slot = Slot(Some(
            self.slots
                .clone()
                .try_acquire_owned()
                .map_err(|_| AuthorityFailure::Unavailable)?,
        ));
        let connection = timeout(CONNECT, PgConnection::connect_with(&options)).await;
        let Ok(Ok(mut connection)) = connection else {
            // A cancelled handshake provides no positive owned-close witness.
            return Err(AuthorityFailure::Unavailable);
        };
        let checked = timeout(BUDGET, catalogue::validate(&mut connection, role))
            .await
            .unwrap_or(Err(AuthorityFailure::Unavailable));
        let session = AuthoritySession {
            connection: Some(connection),
            role,
            slot,
            poisoned: false,
        };
        if let Err(error) = checked {
            session.close().await?;
            return Err(error);
        }
        Ok(session)
    }
}

impl AuthoritySession {
    fn usable(&self, permission: bool) -> Result<(), AuthorityFailure> {
        if self.poisoned {
            return Err(AuthorityFailure::UnknownCompletion);
        }
        if (self.role == Role::Permission) != permission {
            return Err(AuthorityFailure::UnsafeRole);
        }
        Ok(())
    }

    async fn begin<'a>(
        &'a mut self,
        request: &EvaluationRequest,
        permission: bool,
    ) -> Result<Guard<'a>, AuthorityFailure> {
        self.usable(permission)?;
        let workspace = request
            .selected_workspace
            .ok_or(AuthorityFailure::InvalidRequest)?;
        self.begin_context(
            TenantContext::new(workspace, request.executor),
            request.selected_chain,
            permission,
        )
        .await
    }

    pub(crate) async fn begin_context<'a>(
        &'a mut self,
        context: TenantContext,
        chain: Option<Identifier>,
        permission: bool,
    ) -> Result<Guard<'a>, AuthorityFailure> {
        self.usable(permission)?;
        let workspace = context.workspace();
        self.poisoned = true;
        let deadline = Instant::now() + BUDGET;
        let role = self.role;
        let connection = self
            .connection
            .as_mut()
            .ok_or(AuthorityFailure::UnknownCompletion)?;
        let poisoned = &mut self.poisoned;
        timeout_at(deadline, async {
            catalogue::validate(connection, role).await?;
            let retained: bool = sqlx::query_scalar(
                "SELECT nullif(current_setting('arboresce.organization_id',true),'') IS NOT NULL \
                  OR nullif(current_setting('arboresce.domain_id',true),'') IS NOT NULL \
                  OR nullif(current_setting('arboresce.principal_id',true),'') IS NOT NULL \
                  OR nullif(current_setting('arboresce.chain_id',true),'') IS NOT NULL",
            )
            .fetch_one(&mut *connection)
            .await
            .map_err(|_| AuthorityFailure::Unavailable)?;
            if retained {
                return Err(AuthorityFailure::RetainedContext);
            }
            let mut transaction = connection
                .begin()
                .await
                .map_err(|_| AuthorityFailure::Unavailable)?;
            sqlx::query("SET TRANSACTION ISOLATION LEVEL READ COMMITTED, READ WRITE")
                .execute(&mut *transaction)
                .await
                .map_err(|_| AuthorityFailure::Unavailable)?;
            sqlx::query(
                "SELECT set_config('arboresce.organization_id',$1,true), \
                 set_config('arboresce.domain_id',$2,true), \
                 set_config('arboresce.principal_id',$3,true), \
                 set_config('arboresce.chain_id',$4,true)",
            )
            .bind(workspace.organization().to_string())
            .bind(workspace.domain().to_string())
            .bind(context.principal().to_string())
            .bind(chain.map(|id| id.to_string()).unwrap_or_default())
            .execute(&mut *transaction)
            .await
            .map_err(|_| AuthorityFailure::Unavailable)?;
            // Finish this statement before any authority/closure discovery. A
            // join here could retain a statement snapshot from before the wait.
            let parent = if permission {
                "SELECT epoch FROM arboresce_authority.namespace_guard WHERE singleton=1 FOR UPDATE"
            } else {
                "SELECT epoch FROM arboresce_authority.namespace_guard WHERE singleton=1 FOR SHARE"
            };
            let epoch: Option<i64> = sqlx::query_scalar(parent)
                .fetch_optional(&mut *transaction)
                .await
                .map_err(|_| AuthorityFailure::Unavailable)?;
            let _: ControlRevision = epoch
                .ok_or(AuthorityFailure::InvalidStoredData)?
                .to_string()
                .parse()
                .map_err(|_| AuthorityFailure::InvalidStoredData)?;
            Ok(Guard {
                transaction,
                poisoned,
                deadline,
            })
        })
        .await
        .map_err(|_| AuthorityFailure::Unavailable)?
    }

    /// Actual local backend identity for owned qualification barriers and census.
    pub async fn backend_id(&mut self) -> Result<i32, AuthorityFailure> {
        if self.poisoned {
            return Err(AuthorityFailure::UnknownCompletion);
        }
        self.poisoned = true;
        let result = timeout(
            BUDGET,
            sqlx::query_scalar("SELECT pg_backend_pid()").fetch_one(
                self.connection
                    .as_mut()
                    .ok_or(AuthorityFailure::UnknownCompletion)?,
            ),
        )
        .await
        .map_err(|_| AuthorityFailure::Unavailable)?
        .map_err(|_| AuthorityFailure::Unavailable)?;
        self.poisoned = false;
        Ok(result)
    }

    /// Release the finite slot only after positive close of this owned handle.
    pub async fn close(mut self) -> Result<(), AuthorityFailure> {
        let connection = self
            .connection
            .take()
            .ok_or(AuthorityFailure::UnknownCompletion)?;
        let deadline = Instant::now() + CONNECT;
        // Both ordinary and poisoned sessions share positive owned destruction.
        // Keep the permit here: cancellation, failure or late completion retains
        // charge, and no additional protocol write can change a known terminal.
        if close_transport(connection, deadline).await.is_ok() {
            self.slot.0.take();
            Ok(())
        } else {
            Err(AuthorityFailure::UnknownCompletion)
        }
    }
}

pub(crate) struct Guard<'a> {
    transaction: Transaction<'a, Postgres>,
    poisoned: &'a mut bool,
    pub(crate) deadline: Instant,
}

impl Guard<'_> {
    pub(crate) fn connection(&mut self) -> &mut PgConnection {
        &mut self.transaction
    }

    pub(crate) async fn finish(self, commit: bool) -> Result<(), AuthorityFailure> {
        let transaction = self.transaction;
        let result = timeout_at(self.deadline, async {
            if commit {
                transaction.commit().await
            } else {
                transaction.rollback().await
            }
        })
        .await;
        match result {
            Ok(Ok(())) => {
                *self.poisoned = false;
                Ok(())
            }
            _ => Err(AuthorityFailure::UnknownCompletion),
        }
    }
}
