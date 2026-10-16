//! Typed tenant selection and relationship scope, independent of authentication.

use arboresce_domain::Identifier;

pub mod principal;

/// An explicitly selected canonical organization/domain pair.
///
/// Construction preserves checked identifiers. It establishes neither existence
/// nor membership; the canonical store must check both in the consuming transaction.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Workspace {
    organization: Identifier,
    domain: Identifier,
}

impl Workspace {
    pub const fn new(organization: Identifier, domain: Identifier) -> Self {
        Self {
            organization,
            domain,
        }
    }

    pub const fn organization(self) -> Identifier {
        self.organization
    }

    pub const fn domain(self) -> Identifier {
        self.domain
    }

    /// Check the complete workspace, independently of names or other memberships.
    ///
    /// Success is a scope check only. The owning reader must still resolve the
    /// exact typed record and enforce current action and disclosure authority.
    pub fn require_reference(
        self,
        reference: ScopedReference,
    ) -> Result<Identifier, ScopeMismatch> {
        if self != reference.workspace {
            return Err(ScopeMismatch);
        }
        Ok(reference.id)
    }
}

/// Server-internal context for one bounded database transaction.
///
/// The principal must originate at the separately qualified authentication
/// boundary. This value carries no caller-selected verification or authority flag.
/// Its constructor does not verify a key, signature, role, grant or human presence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TenantContext {
    workspace: Workspace,
    principal: Identifier,
}

impl TenantContext {
    pub const fn new(workspace: Workspace, principal: Identifier) -> Self {
        Self {
            workspace,
            principal,
        }
    }

    pub const fn workspace(self) -> Workspace {
        self.workspace
    }

    pub const fn principal(self) -> Identifier {
        self.principal
    }
}

/// The selected reference family is part of a relationship's identity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReferenceKind {
    Project,
    Party,
    Capture,
    Expense,
    Grant,
    Membership,
}

/// An exact typed reference, without any claim that the target exists.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScopedReference {
    workspace: Workspace,
    kind: ReferenceKind,
    id: Identifier,
}

impl ScopedReference {
    pub const fn new(workspace: Workspace, kind: ReferenceKind, id: Identifier) -> Self {
        Self {
            workspace,
            kind,
            id,
        }
    }

    pub const fn workspace(self) -> Workspace {
        self.workspace
    }

    pub const fn kind(self) -> ReferenceKind {
        self.kind
    }
}

/// An input-independent rejection that discloses no foreign resource identity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScopeMismatch;
