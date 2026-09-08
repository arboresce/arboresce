# Security and privacy

**Authority:** normative requirements. **Status:** accepted; implementation and
qualification are tracked separately.

This document owns identity, current authorization, human assurance, acquisition
and exposure controls, and lifecycle protection. [API contracts](api.md) own wire
behavior; [storage and search](storage-and-search.md) own persistence and sealing;
[recovery](recovery.md) owns restoration. Numeric admission limits belong to
[resource profiles](resource-profiles.md). The requirements apply to self-operated
and remotely operated deployments alike.

## Provisioning and credentials

Default listeners bind loopback. Exposing a service remotely requires explicit
encrypted transport and identity configuration; local installation is not an
authentication exemption. First-owner provisioning requires local operator
possession and an expiring, single-use flow. Ship no reusable default password.
Provide a documented local/service-token route that requires no Arboresce account
or entitlement.

Qualify one concrete issuer contract for remote interactive login. The native
client uses the external browser, authorization code and PKCE, exact registered
redirect handling, and state/nonce where required by that contract. Validate issuer,
audience, signature, allowed algorithms, expiry and key rotation. A native client
cannot protect an embedded client secret. Device authorization is available only
when that issuer and the client implementation explicitly support it; it is not
an assumed fallback or a reason to implement a general identity provider.

Principal identity is the stable issuer/subject pair, not a mutable email address.
Resolve active organization membership and its revision canonically. Credentials
have bounded lifetime, revocation, and explicit action, organization, domain and
purpose scopes. A local CLI workspace name is not a tenant or an authorization
grant. Identifiers, URLs and possession of a content digest confer no authority.

Any cookie-based session needs explicit CSRF and SameSite controls. Token-based
APIs require an explicit CORS policy; neither browser policy nor a trusted-looking
origin replaces authentication and authorization.

Store credentials through qualified secret-storage facilities with restricted
file modes where files are necessary. Never place credentials in source, examples,
process arguments, terminal history, logs or exported diagnostics. Local logout
must invalidate the selected profile immediately even when the issuer is offline;
report cleanup failures, preserve other profiles and user data, and fence concurrent
refresh so it cannot restore the logged-out session. Issuer-side revocation is a
separate result. Supported uninstall removes managed secrets, caches and temporary
work while preserving exports and unsent edits. The CLI owns the installed-process
contract and must demonstrate those behaviors with the engine's credential rules.

## Canonical admission and human assurance

Every consequential operation combines current organization membership, resource
ownership, purpose, classification, lifecycle state and delegation. Policies from
all required evidence sources combine restrictively; a broader asset label cannot
expand their audience. Missing, expired or unsupported policy denies the affected
operation. Enforce same-tenant relationships before reading or mutating records.
Validate every link in a delegation chain; delegation cannot confer actions,
purposes, scope or assurance that its issuer lacks.
Use non-owner application database roles and transaction-local tenant context;
connection reuse must never carry authority into another request.

Authoritative checks precede mutations, delivery and external exposure. Caches and
projection payloads are hints, not substitutes for current canonical authorization.
Batch authorized retrieval before sending passages to a reranker. Denied object
identity, passages, counts or existence must not leak through errors, pagination,
search scores, metrics, suggestions or export manifests. [Storage and search](storage-and-search.md)
defines bounded candidate processing; [API](api.md) defines disclosure-safe errors.

An agent acts under its own scoped authority. It must not receive a person's full
credential and rely on a prompt to limit approvals. A command-line confirmation
flag or request field cannot assert that a human approved a result. Human-required
approval needs a distinct authenticated event that binds the actual principal,
assurance, exact content/revision, purpose and decision. The accepted identity
contract must define that mechanism before such approval can be implemented or
claimed. Model output, retrieved instructions and uploaded documents remain data.
Tool responses and embedded document instructions have the same untrusted status.
Consequential external actions in the first release require authenticated human
authority recorded in an explicit approval or valid delegated grant, scoped to
the action, resource, purpose and lifetime. Previously granted authority supports
noninteractive/API automation within those bounds; it does not require a new human
prompt for every authorized command or inference. A prompt or confirmation flag
alone is not a grant. Provider exposure still requires current gateway policy and
admission. Do not execute model-selected shell commands or arbitrary tools;
permitted tools and effects must be explicitly scoped.

Personal drafts remain separate from published organization context and are not
implicitly exposed to organization-wide search or administrators' ordinary views.
Do not introduce hidden employee productivity, loyalty or alignment scoring.

## Acquisition and execution boundaries

Source authority and destination authority are separate. Human-selected local
files require explicit intent and a rooted open that checks path traversal,
symlinks, special files and replacement between validation and use. A caller-
supplied root is not proof of confinement. Initial autonomous acquisition accepts
stdin or trusted descriptors granted outside the model's control. An agent running
as the same operating-system user is not a security boundary for arbitrary paths.
Do not recursively discover personal directories or follow agent-provided paths
without a separately qualified confinement mechanism.

Remote acquisition uses the bounded HTTPS policy in [resource profiles](resource-profiles.md).
Validate origin and destination separately, resolve and constrain address ranges,
and enforce the permitted destination on the connection actually made. Reject
loopback, private, link-local and metadata destinations, including equivalent
address spellings and DNS changes. Redirect, proxy and credential-forwarding
behavior must be explicit; ambient proxy settings cannot silently defeat the
policy. A redirected or changed object must not silently become the bytes of an
already accepted upload. Keep arbitrary remote acquisition out of privileged API
and model execution contexts.

Stream bytes under wire limits, then verify content length and digest, seal the
server-owned immutable object, and only then publish its canonical association.
Filename extensions and caller MIME claims do not establish a supported format.
Decode under the selected media profile, reject unsupported variants, and fail
closed on malformed, oversized or inconsistent output. Staging overwrite, retry
or interruption must not change accepted content. Originals and derivatives stay
separately identified. [Storage and search](storage-and-search.md) owns the exact
finalization and reference-accounting invariants.

Contain parsers with bounded memory, CPU, time, bytes and inodes, per-job scratch,
restricted filesystems and no shared writable customer directories. They receive
only the authorized inputs and scoped capabilities required for that stage; they
must not inherit approval, general database or provider credentials. Separate
parser and external network authority. Clean up controlled scratch after success,
failure, cancellation and crash. Typed validation, parameterized SQL and bounded
output apply even when a trusted transport delivered the bytes.

## Provider exposure

Users may explicitly choose third-party inference that requires credentials or
payment. Core security, review, recordkeeping, context and export must not require
a paid Arboresce entitlement. A credential-free local model is not a required MVP
profile. Provider availability does not establish permission to expose evidence.

All model, embedding and reranking calls pass the approved gateway policy for
provider, model/feature, purpose, organization scope, data class, region, retention
terms and policy expiry. Unknown or expired policy denies restricted material.
Reserve shared slots and cost before invocation. A worker or Python adapter cannot
bypass these checks by holding a broadly usable vendor key. Before each new
exposure, recheck current authority and the exact material being sent.

Record bounded exposure evidence: actor and policy decision, input/profile
identity, safe provider request identity, timing, known outcome and cost state.
Keep it distinct from operational telemetry and product audit. A timeout can leave
a request delivered and a charge uncertain; do not label it undelivered or refund
the reservation merely because a local call failed. Reconcile uncertain outcomes
through the [processing contract](processing.md). No cross-customer training is
enabled by default. Prompts, evidence, credentials and customer-specific datasets
remain subject to their owners' rights and policies.

## Revocation and lifecycle policy

Revocation is measured at canonical admission. A new operation admitted after the
revocation commits must fail for the revoked scope. An in-flight operation rechecks
before each new external exposure and stage transition. An already admitted bounded
transfer may finish within its documented grant; delivered bytes cannot be recalled.
The initial restricted-data profile requires proxy-authorized downloads. Any future
direct-download profile needs separate qualification under the grant ceiling in
[resource profiles](resource-profiles.md) and must disclose residual validity.

Availability, retention hold, purge and immutable history are different controls.
A hold retains bytes and grants no read access; a revoked-and-held object remains
unavailable. Revoke required snapshot evidence before new resolution and refuse
the incomplete snapshot instead of silently serving its remaining components.
Physical deletion must recheck current holds, policy and all authorized references;
a content hash alone is not a deletion scope. Retain minimal tombstones and
independently recoverable lifecycle evidence sufficient to prevent restored access.

The lifecycle policy is versioned and parameterized by data class, purpose,
jurisdiction, hold state, retention trigger, expiry and controlled-copy obligations.
It must cover originals, derivatives, projections, export files, caches, backups,
exposure records and retained audit metadata. Unknown operational retention or
residency policy blocks real-data admission and real purge. Independently authored
synthetic policy values may qualify generic lifecycle implementation; they must be
labelled as fixture policy and never become a customer retention promise.

Working data, retention-locked copies and exports have explicit different policies.
A lock that prevents deletion cannot be hidden behind an immediate-erasure claim.
Key destruction requires dependency and recovery analysis: a shared tenant key is
not selective deletion of one artifact. Encryption-at-rest and encrypted transport
do not prove crypto-erasure. Neither local deletion nor a provider deletion request
proves removal of previously exported or independently retained copies. Report
controlled purge completion, held items, partial failure and external deletion
request state accurately. [Recovery](recovery.md) owns journal completeness and
the rule that serving remains closed until later restrictions are reapplied.

Authenticate and encrypt cross-process transport. Use tenant-specific envelope
encryption where the selected classification/deployment policy requires it.
Signing keys and data-encryption keys have separate purposes and lifecycles;
record key identifiers without disclosing key material. Rotation, recovery and
destruction must document their consequences for historical data and trust.

## Personal information, exports and operations

The MVP supports evidence and reviewed recordkeeping, not tax filing, payments or
certification of deductibility. [Financial records](financial-records.md) owns
exact arithmetic, unresolved values and report eligibility. Any later regional
tax treatment needs separately maintained effective-dated rules and qualified
review; no model-generated legal conclusion or arithmetic is authoritative.

Upload is not blanket consent for processing or sharing. A personal voice note and
a meeting recording have different participants and authority. Preserve configured
purpose, access, sharing and retention limits for audio, participant identity,
business context and other personal information. Location metadata is optional
and must not be exported automatically. Do not solicit payment-card security codes,
banking credentials, health records or children's data for the MVP. Incidental
sensitive content needs classification, restrictions and review/redaction options;
the product must not claim compliance certifications it has not qualified.

Encode untrusted output for the actual consumer: terminals, browser views and
spreadsheets have different active-content hazards. Spreadsheet-safe CSV neutralizes
formula-like text; a separately specified structured JSON export preserves original
values. Do not ship unreviewed HTML, macros or executable archive contents through
an export. Public diagnostics and metric labels exclude content, signed URLs,
credentials and high-cardinality customer identifiers.

Before production, qualify operator MFA/access, credential and account recovery,
key rotation, tamper-evident audit, incident ownership, dependency patching, abuse
and cost controls, backup/deletion restoration, offboarding, and signed artifact
verification. Loss of a sole owner must have an explicit recovery policy that does
not rely on mutable email alone. Break-glass access is scoped, logged, time-limited
and reviewable; it cannot be a hidden content bypass. Contributors receive no
production secrets through their source access.

Append-only database records are not immutable against an administrator. Stronger
audit assurance requires signed checkpoints and independently governed retention,
with explicit verification and recovery of that evidence. Do not claim the audit
store provides those guarantees merely because ordinary application updates fail.

Maintain an escalation path for malicious uploads, credential theft, cross-tenant
exposure and repeated cost abuse. Preserve minimized evidence, suspend affected
admissions and document accountable recovery. G2 and the security experiments in
[acceptance](acceptance.md) are required before real-data and production claims;
passing synthetic tests alone is insufficient.
