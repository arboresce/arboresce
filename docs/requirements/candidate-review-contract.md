# Candidate content and review v1 contract

**Authority:** normative candidate identity, complete replacement, attributable
review and accepted-asset mapping contracts. **Status:** specified wire and
domain contract with parsed-value fixtures; no review service, identity issuer,
database transaction or production canonicalization is qualified by those checks.

The [candidate-review schema](../../contracts/public/v1/candidate-review.schema.json)
provides named definitions and rejects every instance at its library root.
The [standalone validation lane](../../contracts/validation/README.md) consumes
independently authored parsed cases through its exact-family entry. The
[common contract](common-contract.md) owns identifiers, tagged revisions,
calendar dates, command metadata, idempotency and response conventions. The
[observation contract](observation-contract.md) owns immutable evidence records
and their distinct sources and grounding.

[Data model](data-model.md) owns content/control separation and asset history;
[API](api.md) owns shared admission and non-disclosing errors;
[security and privacy](security-and-privacy.md) owns current policy and human
assurance; [processing](processing.md) owns untrusted candidate synthesis;
[resource profiles](resource-profiles.md) own runtime budgets. This document
freezes their candidate-review representation. It adds no candidate creation,
listing, asset supersession, revocation or channel mutation route. Those later
owners must consume the identities and invariants here without treating a
schema pass as authenticated approval.

## Complete content and ownership

`candidate_content` is a closed object. Every row below is required, including
explicit empty lists and a null review-due value when applicable. A replacement
must carry the whole object; omission is not permission to retain or discard an
old field implicitly.

| Field | Exact v1 representation and meaning |
| --- | --- |
| `knowledge_kind` | One of `reported_fact`, `thesis`, `heuristic`, `method`, `exception`, `policy`, `evaluation`, `negative_lesson`. This classification preserves evidential meaning, not a truth or approval claim. |
| `statement` | Nonempty text, at most 16384 Unicode characters. |
| `scope` | Closed `candidate_scope` containing nonempty `summary` of at most 8192 characters, `applies_to` with 1..32 nonempty entries, and `excludes` with 0..32 entries. Each entry is at most 2048 characters. |
| `applicability` | Explicit temporal applicability variant below, independent of the server's recorded time. |
| `assumptions` | Ordered list of 0..32 nonempty strings, each at most 2048 characters. |
| `qualifications` | The same ordered-string bounds; qualifications cannot be discarded when evidence is summarized. |
| `evidence` | Closed `evidence_roles` with required `supporting`, `refuting` and `qualifying` sets; each has 0..64 exact observation references. |
| `invalidation_conditions` | The same ordered-string bounds; preserve the conditions under which the claim stops applying or needs reconsideration. |
| `review_due` | Required null or a closed `{kind: date, value: calendar_date}` or `{kind: instant, value: timestamp}` object. Null means no date has been specified, not that review can never be required. |

These are selected v1 field capacities, not measured runtime support. The
common control-message byte, aggregate depth and nested collection limits still
apply even when each individual field is within its own maximum. The canonical
candidate preimage is also limited to 1048576 UTF-8 bytes. A processor must
respect its qualified output budget before proposing content; never truncate
the statement, scope, opposing evidence or later entries to make a proposal fit.

Scope and applicability describe the claim's intended use. They are not access
policies, grants or an instruction to execute an action. Empty evidence sets are
allowed for a pending proposal; neither an empty nor a nonempty set determines
acceptability by itself. Every referenced observation retains its document,
assertion or inference kind and located/ungrounded status. A reference to a
user assertion cannot become independent proof merely by appearing in the
supporting set. Current domain review policy determines whether the claim's
scope, uncertainty, grounding, review schedule and invalidation conditions are
sufficient for acceptance. Missing required assurance must block acceptance.

## Temporal applicability and review dates

`applicability` has exactly one of these closed shapes:

| Kind | Other required fields | Meaning |
| --- | --- | --- |
| `unspecified` | None. | The author has not specified a temporal applicability period. Do not infer perpetual applicability. |
| `unbounded` | None. | The author deliberately declares no temporal endpoint; non-temporal scope and current policy still constrain use. |
| `date_period` | `start_date`, `end_date`, each common calendar date or null. | A half-open interval of calendar dates; at least one endpoint must be present. |
| `instant_period` | `start_at`, `end_at`, each common UTC timestamp or null. | A half-open interval of exact instants; at least one endpoint must be present. |

An interval includes its start and excludes its end. Null opens that side of the
interval; two nulls are invalid because the `unbounded` variant is explicit.
When both endpoints exist, start must precede end. The schema checks types and
calendar validity; equality, ordering and policy relevance are mandatory domain
checks. Date and instant variants cannot mix endpoints. A date-only observation
must not acquire an invented time of day or timezone to fit an instant period.

`calendar_date` here is a named alias of the
[common calendar-date primitive](common-contract.md#calendar-dates). Calendar dates
compare as dates; they are not UTC instants. Any operation relating a date-only
period or review-due date to a clock must use an explicitly selected domain
calendar/timezone policy, including its IANA timezone and policy revision, and
retain that decision in its evidence or audit. Never use a process's local
timezone or silently reinterpret a past decision after a policy change. An
instant deadline uses the exact common UTC representation. A review-due date
becomes due when the selected local calendar date reaches it; an instant becomes
due when the authoritative clock reaches it. Those scheduling checks require
an injected, qualified clock and are not parsed-schema results.

Review-due and invalidation conditions are immutable content/policy inputs. They
do not automatically revoke an asset, edit its bytes, move a channel or grant
permission to use evidence. Later eligibility and lifecycle operations apply
current policy explicitly. `recorded_at` records when the server admitted a
candidate revision or verdict; it never substitutes for the period the claim
describes or the time an asserted event occurred.

## Canonical candidate identity

A closed `candidate_reference` contains `context`, `candidate_id`, `version`
and `content_sha256`. `version` is the common content revision, never a control
revision or channel generation. `context` uses the common organization/domain/
purpose shape and is the candidate's immutable intended-use context. Its purpose
is not an actor role or an authorization result. Every review command's metadata
context must equal this stored context exactly. An operation wanting a different
intended-use context needs an explicitly owned new-content workflow; it cannot
rewrite an existing candidate reference while reviewing it.

Each command name must be enabled for that exact purpose by the domain's
explicit allowed-purpose/action registry. Unknown, unconfigured or unauthorized
purpose denies admission. A later read may have its own permitted requested
purpose under its owner, but reading cannot alter the stored candidate context
or content fingerprint.
Any later asset-use request has its own current action, audience and purpose
admission. The candidate's intended-use purpose neither grants that use nor
silently becomes the asset consumer's authenticated request context.

To compute `content_sha256`, validate the complete closed content and context,
then normalize only the evidence sets as specified below. Construct one object
with exactly these four fields:

| Preimage field | Value |
| --- | --- |
| `contract_version` | The string `v1`. |
| `kind` | The string `candidate_content`. |
| `context` | The complete exact stored common context. |
| `content` | The complete normalized `candidate_content`. |

Serialize that object using the common vetted JCS numeric/Unicode profile, then
hash its exact canonical UTF-8 bytes with SHA-256. No byte-order mark, newline,
display formatting, signature or self-hash is added. This family presently has
no numeric content fields; it still requires the common Unicode, duplicate-key
and canonical object-order rules. The source-selection profile in the
observation contract is a separate identity and does not replace this preimage.

Candidate ID, content revision, aggregate control revision, server timestamps,
audit events, reviewer comments, verdicts and accepted asset mapping are outside
this content digest. The full candidate identity still binds its ID, revision
and exact context alongside the digest; identical content hashes do not merge
different candidates or authorize a reference from another organization.
Ownership/domain/purpose changes affect the preimage. Changing scope alone,
qualifications, an evidence role, invalidation conditions or review-due also
affects it. Mutable control or availability changes do not rewrite it.

Within each evidence role, sort references lexicographically by the exact
ASCII `organization_id`, then the exact ASCII `observation_id`. Both use the
common canonical UUID encoding. Reject repeated identities within one role
before normalization; never silently deduplicate an invalid request. The same
observation may occur in different roles, preserving its distinct relevance to
different parts or qualifications of the claim. Do not merge those roles or
erase refutation. All three role fields remain present when empty. Object-key
ordering is JCS ordering. Every other array is semantically ordered and retains
its complete supplied order; repeated prose entries are not automatically
deduplicated. Do not trim strings, fold case or normalize Unicode.

Evidence-set permutations therefore preserve normalized candidate identity.
They do **not** preserve the common command fingerprint: that fingerprint covers
the submitted complete parsed request, whose array order remains significant.
Reordering a replacement's evidence array while reusing an idempotency key is
a different request and conflicts, even if it would normalize to the same
candidate content. The candidate normalizer must not silently alter shared
idempotency semantics.

The server recomputes stored and supplied identities. A well-formed digest in a
parsed fixture does not establish its correspondence to any content. A hash
collision or inconsistent canonical record cannot be repaired by choosing one
claim; fail closed and preserve diagnostic evidence under the security policy.

## Commands and recorded responses

All four requests are closed `{metadata, body}` objects using common metadata.
Each body requires `candidate_id`, `expected_version`,
`expected_content_sha256`, `expected_control_revision` and nonempty `reason`
of at most 8192 characters. The candidate ID must equal the canonical route ID;
the expected revision tags must be the common content and control types.
Reason is an attributable explanation of the action, not replacement content.
For example, accepting "with a narrower exception" in a comment cannot silently
qualify the accepted statement: first revise the actual content, then review
that new revision.

| Command | Required method and route | Only additional body field | HTTP 200 recorded response |
| --- | --- | --- | --- |
| `candidate.revise` | `POST /v1/candidates/{candidate_id}/revisions` | `replacement`: the complete closed candidate content. | `candidate_revise_response`: new pending revision, without a verdict or asset mapping. |
| `candidate.accept` | `POST /v1/candidates/{candidate_id}/reviews/accept` | `approval_event_id`: an opaque reference to the trusted exact approval/authorization event described below. | `candidate_accept_response`: accepted revision and its exact review/asset mapping. |
| `candidate.reject` | `POST /v1/candidates/{candidate_id}/reviews/reject` | None. | `candidate_reject_response`: rejected revision and attributable rejection. |
| `candidate.contest` | `POST /v1/candidates/{candidate_id}/reviews/contest` | None. | `candidate_contest_response`: contested revision and attributable contest. |

These are required future routes, not currently implemented endpoints. Only
revise accepts replacement content; accept/reject/contest reject it and every
unsupported nested field. Only accept accepts the approval-event reference.
No command contains an actor assertion, human-approval boolean, replacement
digest supplied as authority, or an opaque generic body.

The shared `candidate_response` composes the common completed-success response
and requires a terminal idempotency receipt and `result.candidate` resource.
Each command-specific response fixes the resulting revision state. A completed
reject or contest command has outer `state=succeeded`; its verdict is not a
processing failure. There is no HTTP 202 accepted-operation variant for these
short canonical transactions. Success means the canonical transaction committed,
not that asynchronous projections have caught up or any channel was promoted.

Record the exact command result snapshot. A later replay returns that historical
result after reauthorization, even when a newer candidate revision exists or
the old asset's availability changed. It cannot relabel the snapshot as current
state, renew an approval, extend idempotency lifetime or repeat the effect.
The later read/list owner supplies current resources separately. Common receipt
expiry, tombstones and same-key payload rules remain unchanged.

## Revision state, control and retained history

The initial proposal, created by its later owning workflow, has content revision
1, aggregate control revision 1 and pending state. Creating that proposal does
not constitute a review. A `candidate_revision` response carries its exact
reference, normalized complete content, server `recorded_at` and `audit_event_id`.
The audit association retains the authenticated proposer or admitted processing
operation and actual source/profile provenance. Client-selected metadata cannot
replace that canonical attribution. The immutable revision never changes after
creation.

`candidate_resource` combines that immutable revision with the candidate
aggregate's observed `control_revision`, per-revision `revision_state`, a
nullable retained `review`, and nullable `superseded_by`. Control is the shared
candidate mutation counter, not a separate counter restarted for each revision.
Historical resources may report a later aggregate control value; a caller still
cannot mutate a historical revision because the current-head precondition is
independent. The review's own control value records its actual decision
transition and remains unchanged as the aggregate advances.

| Current head state and command | Atomic result |
| --- | --- |
| Pending + accept | Preserve content/version/digest; increment control once; record accepted review and exactly one asset-version mapping. |
| Pending + reject | Preserve content/version/digest; increment control once; record rejection, with no accepted mapping. |
| Pending + contest | Preserve content/version/digest; increment control once; record contest, with no accepted mapping. |
| Pending, accepted, rejected or contested + revise | Validate the whole replacement and current evidence use; increment content revision and aggregate control once; make a new pending head; mark the previous revision superseded with an exact immediate-successor reference. |
| Any other fresh transition, including mutating a superseded revision | Conflict without mutation. |

Every fresh command compares current head ID/version/content digest and current
aggregate control under the same admission boundary. A stale expectation is a
conflict even when the candidate text looks unchanged. Revision/control overflow
fails before writing; never wrap, reset, reuse or order contenders by timestamp.
After authorization and expected-precondition checks, a replacement whose
normalized content is identical is `invalid_command`/422. It creates no revision,
consumes no review approval and increments no counter. A pure evidence-set
permutation is unchanged; a meaningful ordered-text permutation can differ.

Revision preserves the old verdict, reason, approval/audit references and any
accepted mapping. The old resource's `revision_state` becomes `superseded`, but
its retained `review.verdict` still describes the decision actually made.
`superseded_by` names the immediate next revision of the same candidate/context,
not a moving current-head alias. Check exact successor identity, increasing
version, no cycles and same ownership. Superseding an unreviewed pending revision
retains `review=null`; superseding a reviewed revision retains its exact review.
No review record is fabricated merely to mark supersession.

Candidate-head supersession is not asset supersession. It cannot revoke or
deprecate an already accepted asset, edit its content, erase its review, replace
its current asset head or move a channel. The new pending candidate requires its
own exact review before it can create a new accepted mapping. Asset replacement,
revocation and release effects require their separately owned explicit operations.

The schema enforces finite impossible-initial-state exclusions: any verdict or
superseded resource needs control at least 2; a superseded resource retaining a
verdict needs at least 3; noninitial content cannot have control 1; an immediate
successor cannot have content revision 1; and a revise result needs both content
and control at least 2. General numeric ordering, exact increments and comparison
between the retained review's counter and the observed aggregate counter remain
mandatory runtime checks, not schema guarantees.

## Attributable approval and the transaction boundary

`review_record` contains the exact candidate reference, server-issued `review_id`,
decision `control_revision`, `verdict`, reason, `recorded_at` and server
`audit_event_id`. An accepted verdict also requires `approval_event_id` and a
closed `accepted_asset_reference`. Rejected and contested verdicts forbid both
acceptance-only fields. The asset reference contains organization, asset ID,
immutable asset version ID, exact candidate reference and review ID. These
identities must agree with the actual canonical review/mapping; they do not
declare current asset availability or permission to serve its bytes.
The submitted reason is attributable to the authenticated executing principal.
Do not attribute an agent's or delegated executor's wording to a human approver
unless the trusted approval event also binds that exact reason. The review's
audit association retains the executor separately from the granting/approving
principal and its approval event, including their distinct authenticated times.

The approval event is a reference to a server-owned trusted record, not a bearer
grant or a caller's proof. Before a new acceptance, resolve and validate the
event against the exact organization, domain, candidate ID, content revision,
content digest, intended-use purpose and accept decision. It must bind the
actual authenticated human authority, required assurance, permitted executing
principal or delegation, lifetime and applicable policy. The identity/security
implementation must establish the issuer and assurance mechanism before this
route can be implemented or qualified. This contract adds no issuer endpoint,
credential format or unsupported CLI approval command.

A valid recorded exact human approval or a valid scoped delegated grant may
authorize execution under the security contract. Any delegated path must retain
its human authorization and a trusted event binding this exact acceptance;
validate every link, scope, assurance, executor, expiry and revocation. Delegation
cannot manufacture human-review assurance its issuer lacks. Previously granted
authority permits noninteractive execution within its bounds; this specification
does not require another human prompt for every authorized command. Conversely,
a service credential, an arbitrary event UUID, an `approved` flag, a model's
reasoning or a copied human token cannot manufacture the required event.

For a fresh acceptance, consume the exact event for its one candidate-review
mapping in the same transaction as the verdict. A reusable broader delegation
does not permit reuse of that consumed exact decision event for another candidate,
revision, purpose or decision. Expired, revoked, foreign, mismatched or consumed
events cannot admit a new effect. Required assurance is obtained before taking
the short transaction lock; no human prompt or unbounded external call belongs
inside the transaction. Canonical policy and revocation state still need the
required authoritative check at the mutation boundary.

The transaction binds the candidate/head lock, current membership and purpose
policy, evidence permissions, expected revisions/digest, review, event
consumption where required, unique accepted mapping, audit/outbox append and
idempotency outcome. There is at most one accepted asset mapping for the exact
candidate revision. A failure at any required write cannot leave an accepted verdict
without its mapping, a mapping without its valid review, a consumed approval
without its committed outcome, or a falsely successful receipt. Preserve reverse
evidence dependencies so later restrictions can affect current eligibility.

Reauthorization precedes both new work and replay disclosure. Once an existing
matching idempotency claim is found, return its authorized recorded result before
applying fresh current-head/precondition checks. The same committed decision
does not require consuming the event again or recreating a new approval because
its original event is now consumed. Do not turn replay into a new authorization
grant. Current loss of action/resource/source permission still denies the
affected request or disclosure under the security policy. A later event expiry
does not rewrite history or authorize another use. A competing new request must
pass all current state and approval checks; its stale transition conflicts.

With fifty otherwise valid fresh accept contenders on the same pending revision,
exactly one accepted review/mapping transition wins. With mixed accept, reject, contest and
revise contenders, exactly one transition wins: a winning revise creates no
verdict or mapping, and a winning reject or contest creates no accepted asset
mapping. Exact same-key retries may join/return the same authorized recorded
result; distinct conflicting commands cannot create extra transitions or mappings.
Rejection and contest are also attributable current-policy decisions and retain
their own immutable audit events; they cannot hide the source or identity
restrictions by avoiding the acceptance-only event field.

## Validation and failure classification

Validate the complete request and current scope before revealing referenced
resources. Body/path identity mismatch or malformed/unknown fields is
`invalid_request`/400. Invalid or absent authentication is `unauthenticated`/401;
an authenticated same-scope denied action follows `forbidden`/403 where permitted
by the API disclosure policy. Foreign or inaccessible candidate, evidence or
approval references follow `not_found`/404 without disclosing real ownership.
Well-formed but inadmissible complete content, invalid applicability ordering or
unchanged replacement is `invalid_command`/422. Stale content/control/digest,
competing transition, reused conflicting idempotency payload or already consumed
exact approval for a new effect is `conflict`/409. Unsupported or unavailable
required policy/identity infrastructure cannot be treated as approval; return
the appropriate common failure without committing the command.

The schema's response resource policy allows bounded unknown extensions while
reserving direct outcome/approval aliases. Canonical content, references and
requests remain closed. The outer `state=succeeded` describes completed command
execution; candidate `revision_state`, review `verdict` and separately owned
asset availability describe different facts. A failed command uses the typed
common problem and cannot carry a successful candidate result.

## Parsed fixtures and pending semantic qualification

The finite parsed manifest validates complete content and commands, field/date
bounds, revision tags, response variants and static contradictions. Its fixed
declaration order is part of the reviewed fixture contract. Later runtime tests
may reorder execution by stable IDs. Independently prepared expected hashes do
not make a parsed-schema check a canonicalization implementation or prove
cross-language equality.

The following are required future semantic tests, **not executed results**.
Their owning domain, storage, application, identity and real CLI suites must
retain independent inputs/oracles and fresh evidence under the
[testing and coverage contract](testing-and-coverage.md). The later shared
runner promotes these scenarios into executable fixtures without substituting
schema checks for current policy, native runtime or actual transactional results.

| ID | Required independent scenario and result |
| --- | --- |
| REV-SEM-01 | Recompute exact normalized preimage bytes/digest from complete content. Independently change scope, qualification, evidence role, invalidation, review-due and context; each consequential change changes identity. Control, recorded time and review comment do not rewrite content identity. |
| REV-SEM-02 | Evidence-set permutations preserve content digest; duplicate same-role identity fails before normalization; cross-role occurrence is retained. Ordered prose permutations remain distinct. Same-key reordered submitted replacement conflicts under the unchanged common fingerprint. |
| REV-SEM-03 | Exercise key order, Unicode escaping and UTF-16 sorting across Rust/Python consumers. Invalid Unicode, duplicate keys, unsupported fields and preimage/aggregate limits fail without truncation or partial digest. |
| REV-SEM-04 | Compare missing versus null review-due; date-only versus instant; unspecified versus unbounded applicability; open endpoints and exact half-open boundaries. Equal/reversed periods fail semantically even when shapes pass. Calendar-policy/timezone changes cannot rewrite past attribution. |
| REV-SEM-05 | Every missing consequential replacement field fails. Scope-only valid revision changes digest/version, returns pending and preserves old content. Exact normalized replacement and set-only permutation return 422 without changing head or counters. |
| REV-SEM-06 | Independently stale content revision, digest, aggregate control, route ID and metadata context fail in their owning category. Check maximum counter overflow and atomic non-mutation; timestamp ordering cannot choose a winner. |
| REV-SEM-07 | Pending accept/reject/contest produce exactly their recorded verdict; replacement or approval flags on the wrong command fail. A reason that purports to narrow accepted content cannot silently edit its scope. |
| REV-SEM-08 | Revise each permitted current state. New head is pending; old revision is superseded with its exact immediate successor and retained verdict/mapping. Historical/superseded mutation, foreign successor and cycles fail. No asset control or channel changes occur implicitly. |
| REV-SEM-09 | A foreign same-digest observation/candidate, unavailable required evidence or changed current source restriction denies use without disclosing existence/counts. Assertions, inferences and ungrounded records cannot impersonate independently observed evidence. |
| REV-SEM-10 | Validate human approval and scoped delegation against exact actor/executor, organization/domain/purpose, candidate ID/revision/digest/decision, assurance and lifetime. Wrong, expired, revoked, forged or service-self-asserted proof cannot accept. Previously valid scoped authority requires no invented repeated prompt. |
| REV-SEM-11 | Atomically consume a valid exact approval event once. Same-key authorized replay returns its original review/mapping after later revisions without consuming again. New effect with consumed/mismatched event conflicts; current permission loss denies the affected replay disclosure. |
| REV-SEM-12 | Fifty otherwise valid accept contenders at explicit barriers produce one accepted review/mapping. Mixed otherwise valid accept/reject/contest/revise contenders produce one committed transition with exactly the winning variant's effects; revise creates no verdict/mapping, and reject/contest create no accepted mapping. Exact same-key retries return the recorded result; distinct conflicting commands fail. Repeat with current policy changes; if all lose authority before admission, no transition commits. |
| REV-SEM-13 | Inject failures before/after each review, mapping, event-consumption, audit/outbox and idempotency write. Required failure rolls back the whole transition; lost response replay returns one retained outcome. No phantom accepted asset exists. |
| REV-SEM-14 | Revoke membership, source permission or approval at the admission/commit boundary. The losing operation cannot persist, expose or enqueue an unauthorized effect; reverse dependencies retain exact source identities. |
| REV-SEM-15 | Compare review's retained decision control with later aggregate control and exact candidate/mapping IDs. Reject substituted asset/review/version references and impossible history, beyond the schema's finite initial-state exclusions. |
| REV-SEM-16 | Later asset revocation or candidate revision preserves historical review attribution and content hashes. Review-due crossing or new pending revision cannot silently supersede/revoke an asset or advance a channel. |
| REV-SEM-17 | Actual CLI human/service flows preserve complete replacement, exact expected versions, current authority, structured outcomes and lost-response retry identity. No local boolean or model-produced instruction supplies human assurance. |

No production digest implementation, current-policy check, issuer integration,
review transaction, concurrency race or real CLI behavior is claimed here.
Missing real identity or service inputs block their later qualification gates
while independently verifiable contract work can continue.
