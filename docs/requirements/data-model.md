# Canonical data model

Status: accepted normative requirements; the product domain and persistence
are not implemented. Exact field schemas and transition contracts follow the
contract steps in the [implementation plan](../execution/open-cli-mvp-v1-rcld.md).

This document owns domain identities, state separation, evidence locators and
canonical content/attestation identity. [Storage and search](storage-and-search.md)
owns relational enforcement. [Financial records](financial-records.md) owns
expense confirmation, money, reports and financial export.

## Identity and ownership

Use `organization_id` consistently as the tenant identity. A personal
deployment is an organization with one user. A client workspace is a named
connection/profile selecting an organization and domain, not another tenant
model. An opaque resource ID is never a bearer capability.

Use server-issued opaque UUIDs or another explicitly qualified
collision-resistant identity. UUID ordering or embedded time is not commit
order and is not proof that an event occurred. Every consequential reference
must preserve organization ownership; a resource cannot acquire another
organization's evidence merely by naming its ID or digest.

| Record | Identity and invariant |
| --- | --- |
| Organization | Security and policy boundary with explicit lifecycle state. |
| Principal | Canonical OIDC `(issuer, subject)` or independently issued service identity; mutable email is not identity authority. Human/service kind is separate from roles. |
| Membership and domain | Active same-organization ownership, explicit purposes and review policy; disabled membership overrides cached roles. |
| Capture revision | Organization/capture identity and content revision; ordered attachment roles and attributed user context. |
| Upload | Organization, principal, upload identity and generation, expected length/digest, expiry and finalization state. |
| Blob object | Organization-scoped object identity, exact SHA-256, byte length, media type and key reference; separate from artifact identity. |
| Artifact | Stable source identity with independently mutable control revision, current version and availability. |
| Artifact version | Immutable version identity and original-byte digest bound to a server-sealed object. |
| Representation | Exact derived bytes/digest linked to the original version and processor/profile; regeneration creates a new representation. |
| Observation | Exact evidence reference and source kind: document-derived observation, attributed user assertion or model inference. |
| Candidate revision | Complete statement, scope, assumptions, supporting/refuting/qualifying evidence and invalidation conditions; every edit changes revision. |
| Review | Attributable verdict bound to exact candidate revision and fingerprint; acceptance has one linked asset mapping. |
| Expense revision and report | Separate recordkeeping lineage with evidence, reviewed revisions and immutable report selections, governed by the financial owner. |
| Asset version | Immutable reviewed content, exact review link and applicability; mutable availability is separate. |
| Context Build | Organization-scoped SHA-256 content identity for exact selected asset versions and compiler output. |
| Evaluation, approval, signature | Separate append-only receipts bound to exact build, suite/profile, policy, actor or signing envelope. |
| Channel | Organization/domain/name with monotonically increasing generation and explicit state. |
| Usage receipt | Exact input/assembly digest, purpose, current policy context and any allowed selection limits. |
| Feedback and outcome | Exact subject version, author/source, observed event time and receipt time; assertions remain distinguishable from independently observed results. |
| Revocation, hold and purge obligation | Separate control/policy records, actor/scope and propagation receipts; preserve recovery-relevant restrictions. |

Keep knowledge and recordkeeping as separate branches. A receipt does not need
to become an intelligence asset to support a confirmed expense and report.
Likewise, an observation or model output is not a review or approval.

## Revisions, time and state

Content revision, control revision and channel generation are distinct integer
types. Compare and increment the type owned by the command, reject overflow,
and never order simultaneous updates by wall-clock timestamp. Content changes
create immutable revisions; availability or grants must not rewrite content
identity. Current privilege checks accompany the revision comparison.

The [common wire contract](common-contract.md) owns the exact tagged revision
encoding and bounded canonical decimal strings used to transport these integer
values without JSON numeric precision loss. Wire encoding does not merge their
independent domain types or replace the owning command's precondition.

Store server `recorded_at` in UTC using an injected clock. Preserve optional
source/user `observed_at` with attribution and precision. A date-only receipt
remains a date. An ambiguous local instant requires explicit timezone handling;
record an IANA timezone or declared reporting policy when dates become periods.
Do not infer expense time from upload time or use ID timestamps as evidence.
Where applicability changes over time, retain the period a statement applies
to independently from when the system learned or recorded it. A later correction
must preserve those two meanings rather than overwrite the earlier account.

Each aggregate has its own state machine:

| Aggregate | Required state distinctions |
| --- | --- |
| Upload | `created → uploading → uploaded → sealing → sealed`, with terminal `expired`, `aborted`, `rejected`. Network retries do not rewind state or replace an accepted sealed reference. |
| Capture submission | `draft → submitted`; cancellation requires an explicit operation. Submission fixes the expected attachment set; amendments create a capture revision. |
| Attachment readiness | `incomplete`, `ready`, `failed`, independently of submission or processing state. Required failure blocks eligibility; optional failure remains a visible warning. |
| Processing | `queued → running → succeeded`, `failed` or `cancelled`; retry/wait details and partial attachment results belong to stage records. |
| Candidate revision | `pending`, then an explicit `accepted`, `rejected`, `contested` or `superseded` transition. Revision creates new pending content; permitted reconsideration preserves prior verdicts. |
| Expense | Proposed or confirmed heads can be revised into new proposed content while retaining a superseded predecessor; confirmation is explicit and void is terminal under the financial owner. |
| Asset availability | `active`, `deprecated`, `revoked`, `purged`; content hashes remain stable across control changes. |
| Build availability | `available`, `unavailable`, `purged`; current release eligibility is computed separately from policy, source restrictions and valid receipts. |
| Channel | `empty`, `active`, `suspended`; active requires a build. An empty channel may omit it. |
| Operation | Explicit nonterminal progress and terminal `succeeded`, `failed`, `cancelled`; success cannot carry failure, and cancellation cannot erase committed effects. |

No attachment is silently dropped. A capture keeps the order, role,
required/optional classification and exact version of every expected part.
Foreign references fail. Processing readiness depends on this fixed revision,
not on a mutable list assembled by a worker.

The [acquisition contract](acquisition-contract.md) defines the wire identities,
immutable attachment intents and separately observed sealed bindings and
readiness. Its operation rules preserve these aggregate distinctions.

The [financial contract](financial-records.md#field-attribution-and-content-identity)
owns complete expense identity and effective field attribution, including
authenticated command assertions without a new observation route. Its exact
financial review records human reviewer and actual executor separately.
[Immutable reports](financial-records.md#immutable-report-selections) retain
selected references and observed controls; later corrections or availability
changes cannot rewrite their bytes. Report and export-payload digests have
separate preimages from financial content and transport/artifact digests.

## Evidence and review identity

The [candidate review contract](candidate-review-contract.md) owns the complete
candidate v1 content, fingerprint and review variants. Its parsed-value schema
does not replace canonical hashing, current authority or transactional checks.

Candidate fingerprints cover all consequential content, including scope,
assumptions, qualifications, all evidence roles and invalidation conditions.
A scope-only edit must change identity. Define deterministic ordering for
set-valued evidence while preserving semantically ordered content. There is
no accepted asset without its valid exact-review mapping.

Supporting evidence does not erase refuting or qualifying evidence. A model
must retain uncertainty, alternative explanations and absent grounding.
An assertion about purpose, attendance or outcome remains attributed to its
author; a photograph, transcript and inference are distinct sources.
Preserve the distinctions among reported facts, theses, heuristics, methods,
exceptions, policies, evaluations and negative lessons. Correlation or a
generated explanation is not proven causation. Retain attributable review dates
and explicit review-due or invalidation conditions where the content requires them.

[API review semantics](api.md#acquisition-and-review-operations) own the command
variants. Acceptance pins one exact revision/digest; replacement creates a new
pending revision and cannot auto-approve. Supersession requires an explicit
reviewed relationship, rejects cycles and foreign links, preserves old content,
and never silently promotes a channel.

Derived content retains the source restrictions required by
[security and privacy](security-and-privacy.md). Evidence dependency records
must support current audience/purpose intersection and reverse impact queries.
A highest classification label alone cannot represent independently restricted
audiences. Explicit reviewed declassification creates a distinct output with
retained provenance; mutable grants are not immutable content truth.

## Typed multimodal locators

The [observation contract](observation-contract.md) owns the concrete v1 locator
and source-kind shapes. Its wire schema does not replace validation against the
actual authorized representation, source content and transformation profile.

Locators bind to exact representation identity and digest, not just an artifact
head or user-visible filename. Validate their units and bounds against that
representation before accepting an observation.

- `utf8_range`: half-open byte interval with `0 <= start < end <= length`, both
  endpoints on valid UTF-8 boundaries. Newline normalization produces a new
  representation and therefore different offset identity.
- `json_pointer`: actual RFC 6901 parsing with valid `~` escapes and a selected
  value, including scalar values. The contract defines selected-value
  canonicalization; arbitrary JSONPath is not an equivalent locator.
- `page_region`: zero-based page index, exact render digest, orientation and
  coordinate-space version, plus a bounded rectangle in the declared units.
- `audio_span`: half-open millisecond interval within the exact decoded or
  normalized representation, with channel and transcript-segment identity
  where used. Preserve normalization/time mappings. Speaker labels do not
  establish a person's identity.

Unsupported or absent grounding is explicitly ungrounded and requires review;
never fabricate a locator or confidence value. Reject representation digest
substitution, reversed intervals, invalid code-point boundaries, nonexistent
JSON selections, out-of-page coordinates and out-of-duration spans. A locator
variant does not itself promise support for every media format.

## Canonical builds

A Context Build freezes a nonempty bounded selection of exact eligible asset
versions, their relevant content/qualifications, compiler version and compilation
configuration. The contract freezes the complete manifest, canonical preimage
and representation before implementation. Reject required inputs that are
unauthorized, unavailable or oversized; do not produce an implicitly partial
build. Cross-domain selection requires explicit policy.

Use vetted RFC 8785 JSON Canonicalization Scheme processing with a declared
bounded numeric and Unicode profile and cross-language vectors. Reject duplicate
keys, invalid Unicode, non-finite numbers and unsupported values before hashing.
Test UTF-16 property ordering explicitly. Canonically order
sets while preserving arrays or text whose order has meaning. Monetary decimal
strings retain the [financial type](financial-records.md#exact-monetary-values).

Hash the canonical immutable payload with SHA-256. Exclude its own hash,
execution timestamps, run/attempt IDs, evaluations, approvals and signatures
from that preimage. Exact input/profile retries reuse the same identity;
different execution timing cannot change it. Store sealed compiled bytes and
immutable input edges before binding the canonical result. A foreign content
hash cannot be used to probe another tenant's existence or access.

Evaluation receipts name the exact build, versioned suite/cases, evaluator,
dataset/profile, expected counts, actual results and validity. Unknown suites,
contradictory counts and failed required checks cannot confer release eligibility.
Evaluation must not rewrite the independent oracle to fit the output.
Deterministic correctness and live quality remain distinct evidence classes.

## Attestations and channel generations

The required signing envelope is domain-separated:

```text
"arboresce.context-attestation.v1\0" || canonical_bytes(
    organization_id, domain_id, build_digest, attestation_kind,
    subject_profile_digest, issuer_key_id, issued_at, expires_at
)
```

The machine-contract step must freeze field encoding and signing vectors for
this envelope. Use vetted Ed25519; key access belongs outside pure domain code.
Trust configuration must define issuer keys, rotation overlap, expiry and
revocation. Reject malformed signatures and unknown, expired or revoked keys,
wrong organization/domain/build/profile and unsupported envelopes.

Approval checks current actor, purpose, source eligibility and required
evaluation receipts. Human approval requires the authenticated mechanism owned
by [security and privacy](security-and-privacy.md). A signature shows only that
the configured issuer signed those bytes; it cannot prove truth, regulatory
correctness, current access or successful downstream use. Content attestations
do not sign native distributions or container images.

Channel creation, promotion, suspension and rollback compare the expected
generation and current eligibility in one transaction. Every mutation advances
the generation; deleting/recreating a name must not recycle its counter.
Rollback is a new authorized transition, not permission to serve revoked inputs.
Retain suspension and promotion history. Simultaneous same-generation mutations
have one winner.

## Feedback and lifecycle references

Feedback and outcome records use a constrained subject variant with exact
version and tenant-safe referential validation, not arbitrary URLs. Preserve
the asserted event time separately from receipt time. New feedback is evidence
for later review, not automatic approval, supersession or completed processing.

When version-linked evidence describes a decision, retain its alternatives,
rejected options, responsible actor and assumptions. When it describes an
experiment, retain the intervention, population, metrics and confounding
explanations as attributed evidence. Actions and later outcomes link to the
exact decision and asset versions used; missing information stays explicit.
These records preserve context for review and negative results without requiring
a standalone decision-management or experiment application.

Holds, logical revocation and physical purge are independent controls. A hold
can preserve bytes without restoring reads. Purged content is unavailable and
must have explicit markers in permitted history/exports; metadata cannot stand
in for missing bytes. [Security and privacy](security-and-privacy.md) owns
serving restrictions and export policy; [recovery](recovery.md) owns journal
completeness and restoration requirements.

## Required verification

Use independent domain oracles for state/variant contradictions, revision
overflow and races, tenant substitution, required/optional attachments, every
locator kind, scope-only fingerprint changes, canonical permutations,
cross-language bytes, malformed signatures, channel counter reuse and
held-but-revoked content. [Testing and coverage](testing-and-coverage.md) owns
fixture and source gates; [acceptance](acceptance.md) owns qualification counts.
