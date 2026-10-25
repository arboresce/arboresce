use std::fmt;

use serde_json::Value;

use crate::{CanonicalJsonSelection, ContentRevision, Identifier, Sha256Digest};

use super::{
    InvalidProjectValue, MAX_PARTY_VALUE_BYTES, PartyAliasStatus, PartyKind, WorkspaceReference,
    parsed,
};

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct PartyAlias {
    value: String,
    status: PartyAliasStatus,
}

impl PartyAlias {
    pub fn value(&self) -> &str {
        &self.value
    }
    pub const fn status(&self) -> PartyAliasStatus {
        self.status
    }
}

impl fmt::Debug for PartyAlias {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("PartyAlias(..)")
    }
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct PartyDetail {
    kind: String,
    value: String,
    status: PartyAliasStatus,
}

impl PartyDetail {
    pub fn kind(&self) -> &str {
        &self.kind
    }
    pub fn value(&self) -> &str {
        &self.value
    }
    pub const fn status(&self) -> PartyAliasStatus {
        self.status
    }
}

impl fmt::Debug for PartyDetail {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("PartyDetail(..)")
    }
}

/// An owned content revision. Party kind/aliases convey no principal authority.
#[derive(Clone, Eq, Hash, PartialEq)]
pub struct PartyRecord {
    workspace: WorkspaceReference,
    party_id: Identifier,
    content_revision: ContentRevision,
    kind: PartyKind,
    aliases: Vec<PartyAlias>,
    details: Vec<PartyDetail>,
    canonical: CanonicalJsonSelection,
}

impl PartyRecord {
    /// Validate a complete closed parsed value before copying its bounded data.
    /// Raw transport decoding and canonical resource lookup are separate gates.
    pub fn from_value(value: &Value) -> Result<Self, InvalidProjectValue> {
        let object = parsed::object(
            value,
            &[
                "workspace",
                "party_id",
                "content_revision",
                "kind",
                "aliases",
                "details",
            ],
        )?;
        let workspace = parsed::workspace(&object["workspace"])?;
        let party_id = parsed::identifier(&object["party_id"])?;
        let content_revision = parsed::revision(&object["content_revision"])?;
        let kind = parsed::enumeration(&object["kind"])?;
        let mut aliases = Vec::new();
        for alias in parsed::array(&object["aliases"], 1, 32)? {
            let alias = parsed::object(alias, &["value", "status"])?;
            aliases.push(PartyAlias {
                value: parsed::text(&alias["value"], 256)?.to_owned(),
                status: parsed::enumeration(&alias["status"])?,
            });
        }
        let mut details = Vec::new();
        for detail in parsed::array(&object["details"], 0, 32)? {
            let detail = parsed::object(detail, &["kind", "value", "status"])?;
            details.push(PartyDetail {
                kind: parsed::slug(&detail["kind"])?.to_owned(),
                value: parsed::text(&detail["value"], 1024)?.to_owned(),
                status: parsed::enumeration(&detail["status"])?,
            });
        }
        let canonical = parsed::canonical(value, MAX_PARTY_VALUE_BYTES)?;
        Ok(Self {
            workspace,
            party_id,
            content_revision,
            kind,
            aliases,
            details,
            canonical,
        })
    }

    pub const fn workspace(&self) -> WorkspaceReference {
        self.workspace
    }
    pub const fn party_id(&self) -> Identifier {
        self.party_id
    }
    pub const fn content_revision(&self) -> ContentRevision {
        self.content_revision
    }
    pub const fn kind(&self) -> PartyKind {
        self.kind
    }
    pub fn aliases(&self) -> &[PartyAlias] {
        &self.aliases
    }
    pub fn details(&self) -> &[PartyDetail] {
        &self.details
    }
    pub fn as_bytes(&self) -> &[u8] {
        self.canonical.as_bytes()
    }
    pub fn byte_length(&self) -> usize {
        self.canonical.byte_length()
    }
    pub const fn sha256(&self) -> Sha256Digest {
        self.canonical.sha256()
    }
}

impl fmt::Debug for PartyRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("PartyRecord(..)")
    }
}
