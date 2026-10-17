//! Bounded bootstrap scope observations over supplied trusted canonical records.
//! A successful check admits no writer and supplies no lifecycle proof.

use arboresce_domain::{Identifier, UtcTimestamp};

/// A trusted registered selection, supplied independently of the grant/request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RegisteredBootstrapScope<'a> {
    pub namespace: &'a str,
    pub policy: &'a str,
}

/// Current canonical grant data. Construction alone establishes no authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BootstrapGrant<'a> {
    pub id: Identifier,
    pub operator: Identifier,
    pub namespace: &'a str,
    pub policy: &'a str,
    pub organization: Identifier,
    pub issued_at: UtcTimestamp,
    pub expires_at: UtcTimestamp,
    pub single_use: bool,
    pub consumed: bool,
    pub recognized_retired: bool,
}

/// Normalized requested scope; no caller-selected executor or verifier flag.
/// Checked owner format establishes neither existence nor membership/key proof.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BootstrapRequest<'a> {
    pub namespace: &'a str,
    pub policy: &'a str,
    pub organization: Identifier,
    pub new_owner: Identifier,
}

/// Finite scope failures, without retaining rejected input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BootstrapScopeFailure {
    InvalidConfiguration,
    InvalidStoredData,
    InvalidInput,
    OperatorMismatch,
    NamespaceMismatch,
    PolicyMismatch,
    TenantMismatch,
    GrantExpired,
    GrantUsed,
    ScopeRetired,
    GrantNotYetValid,
    UnsupportedBootstrapGrant,
}

fn token(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 96
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'.' | b'-'))
}

/// Observe exact registered scope, operator, organization, lifetime and one-use
/// state. All names are bounded before comparisons; this borrows and allocates
/// no heap and performs no I/O or mutation.
///
/// The adapter supplies the authenticated operator, healthy trusted time and
/// current canonical records independently. Success is only a scope match. It
/// supplies no authentication, effect fence, grant consumption or capability.
/// Authentic independently retained lifecycle continuity and the separately
/// qualified command/writer remain prerequisites for every bootstrap effect.
pub fn check_scope(
    registered: RegisteredBootstrapScope<'_>,
    grant: &BootstrapGrant<'_>,
    request: &BootstrapRequest<'_>,
    authenticated_operator: Identifier,
    now: UtcTimestamp,
) -> Result<(), BootstrapScopeFailure> {
    if !token(registered.namespace) || !token(registered.policy) {
        return Err(BootstrapScopeFailure::InvalidConfiguration);
    }
    if !token(grant.namespace) || !token(grant.policy) || grant.issued_at >= grant.expires_at {
        return Err(BootstrapScopeFailure::InvalidStoredData);
    }
    if !token(request.namespace) || !token(request.policy) {
        return Err(BootstrapScopeFailure::InvalidInput);
    }
    if authenticated_operator != grant.operator {
        return Err(BootstrapScopeFailure::OperatorMismatch);
    }
    if grant.namespace != registered.namespace || request.namespace != registered.namespace {
        return Err(BootstrapScopeFailure::NamespaceMismatch);
    }
    if grant.policy != registered.policy || request.policy != registered.policy {
        return Err(BootstrapScopeFailure::PolicyMismatch);
    }
    if request.organization != grant.organization {
        return Err(BootstrapScopeFailure::TenantMismatch);
    }
    if grant.recognized_retired {
        return Err(BootstrapScopeFailure::ScopeRetired);
    }
    if !grant.single_use {
        return Err(BootstrapScopeFailure::UnsupportedBootstrapGrant);
    }
    if grant.consumed {
        return Err(BootstrapScopeFailure::GrantUsed);
    }
    if now < grant.issued_at {
        return Err(BootstrapScopeFailure::GrantNotYetValid);
    }
    if now >= grant.expires_at {
        return Err(BootstrapScopeFailure::GrantExpired);
    }
    Ok(())
}
