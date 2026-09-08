# Acquisition and sealed-byte v1 contract

**Authority:** normative acquisition wire shapes, state transitions and reference
validation. **Status:** specified contract; no upload service, storage adapter,
parser or processing qualification is established by schema validation.

The [acquisition schema](../../contracts/public/v1/acquisition.schema.json)
supplies named definitions and deliberately rejects every bare root instance.
It composes the [common contract](common-contract.md), preserving the common
identifier, typed revision, metadata, problem, idempotency and reader policies.
The [standalone validation lane](../../contracts/validation/README.md) consumes
independently authored parsed-value cases. Runtime obligations remain explicit
below until their owning implementation executes the required tests.

[API](api.md) owns shared admission and transport behavior;
[data model](data-model.md) owns canonical identity and state separation;
[storage and search](storage-and-search.md#sealed-objects-and-finalization) owns
fenced sealing and reference accounting. [Security and privacy](security-and-privacy.md)
owns current authority, acquisition confinement and controlled disclosure.
[Resource profiles](resource-profiles.md) own media, capture and adapter limits.
This document freezes their acquisition representation without introducing a
storage provider, account, deployment or command implementation.

## Definitions and ownership

| Definition family | Meaning |
| --- | --- |
| `byte_identity`, `declared_original` | Exact digest/length representation and the supported declared original format union. |
| `upload_generation`, `upload_resource`, `staging_grant` | Upload identity/epoch, lifecycle state and separately scoped staging authorization. |
| `original_reference`, `original_version`, `sealed_binding` | Immutable accepted original identity and one verified upload-to-original association. |
| `representation` | Independently identified derived bytes linked to an exact immutable original and processor profile. |
| `attachment_role`, `attachment_intent`, `attachment_intents` | Complete ordered source intent, independent required/optional classification and bounds. |
| `attachment_readiness`, `capture_readiness` | Current source availability, retaining every declared part and failure. |
| `capture_revision`, `capture_resource` | Immutable content revisions separated from mutable submission/control state. |
| The five `*_request` definitions | Complete closed request bodies and registered common metadata for the operations below. |
| Allocation, finalization and capture `*_response` definitions | Recorded command results or accepted operations, with resources nested under typed result fields. |
| `resource_response`, `required_part`, `required_failed_part`, `required_incomplete_part`, `capture_response` | Shared bounds and predicates, not extra API operations. |

Only server-issued opaque identities appear in accepted original references.
No public sealed binding contains backend bucket names, object keys, storage
credentials or a client-selected canonical destination. Original and derived
byte identity is independent of mutable grants, holds and availability.
The [lifecycle acquisition specialization](lifecycle-contract.md#5-acquisition-and-analysis)
owns required upload control, abort, explicit analysis, complete operation
results and exact current/revision reads. It composes these definitions without
changing their immutable leaves or enabling an unqualified download route.

## Complete commands and routes

Each request is exactly `{metadata, body}`. `metadata` conforms to the closed
common `command_metadata` and has the command name below. Each body is closed,
including all nested declared originals and attachment intents. The canonical
fingerprint covers the complete parsed `{metadata, body}` value, along with the
common method, route, version and purpose inputs. There is no body-field
exclusion for a repeated route ID or an expected digest.

All listed routes are required v1 contracts, not currently available endpoints.
Path identifiers use the common canonical UUID representation. A repeated body
ID must equal the path ID exactly; mismatch is `invalid_request`/400 before
admission. Resolve the path inside the metadata organization/domain and current
principal's permitted purpose. A foreign or inaccessible referenced identity
returns `not_found`/404 without revealing its actual organization or storage.

| Command and method/route | Exact body | Accepted result |
| --- | --- | --- |
| `upload.allocate` — `POST /v1/uploads` | `source: declared_original`. | HTTP 201, `upload_allocation_response`: a recorded created-upload snapshot, generation 1 and scoped staging grant. |
| `upload.finalize` — `POST /v1/uploads/{upload_id}/finalize` | `upload_id`, `expected_generation: upload_generation`, `expected_source: declared_original`. | HTTP 202 after durable admission of sealing; the lifecycle specialization owns complete operation/upload fields, nonterminal replay, direct successful 200, failed/cancelled replay and fresh-key existing-binding lookup. |
| `capture.create` — `POST /v1/captures` | `user_context` and the complete ordered `attachments` list. | HTTP 201, `capture_create_response`: draft, content revision 1, control revision 1. |
| `capture.revise` — `POST /v1/captures/{capture_id}/revisions` | `capture_id`, `expected_version: content_revision`, `expected_control_revision: control_revision`, replacement `user_context` and complete ordered `attachments`. | HTTP 200, `capture_revise_response`: a new draft content revision and advanced control revision. |
| `capture.submit` — `POST /v1/captures/{capture_id}/submit` | `capture_id`, `expected_version: content_revision`, `expected_control_revision: control_revision`. | HTTP 200, `capture_submit_response`: durable fixation of that revision's intent, with an advanced control revision. |

`user_context` is 0–65536 valid Unicode scalar values under the common JSON
profile. It remains an attributed user statement, not verified evidence or a
model instruction with authority. The server retains the authenticated
principal and recorded time in the immutable command/audit association for the
revision. A client cannot choose its attribution by supplying an actor field.
The identity contract owns any fuller public principal representation.

Every command checks the registered purpose and action, current membership,
domain scope, referenced uploads and evidence permissions. Shape-valid metadata
does not prove those facts. Check authority and declare the result/audit/outbox
atomically under the shared locking and idempotency policy. No byte stream,
storage network call or decoder runs while those database locks are held.
Allocation reserves applicable shared storage/transfer capacity before issuing
a grant. Finalization claims bounded work before returning 202; quota denial is
not accepted work. Private or real-data use still requires the security and
resource gates before the first adapter accepts it.

Each named command is checked against the domain's explicit allowed-purpose
and action registry. Unknown or unconfigured purposes deny admission; a valid
slug never implies an allow-all policy. The registry supplies the applicable
domain vocabulary without inventing a universal customer purpose list here.

The 202 response contains a public operation and `Location` identifying that
same operation, with applicable bounded `Retry-After`. Its additional `upload`
field is a sealing snapshot, not a canonical original. The
[lifecycle acquisition contract](lifecycle-contract.md#5-acquisition-and-analysis)
defines polling, abort/cancellation and current/exact-resource read routes.
Stopping client waiting or losing
the connection does not abort an upload or cancel accepted work.

## Declared media and byte identity

`declared_original` contains exactly `format`, `media_type`, `sha256` and
integer `byte_length`. The following canonical pairs are the v1 union; MIME
aliases, case changes, parameters and additional format variants are not
silently normalized into it.

| Format | Canonical media type | Inclusive encoded-byte range |
| --- | --- | --- |
| `plain_text` | `text/plain` | 0–10485760. |
| `markdown` | `text/markdown` | 0–10485760. |
| `json` | `application/json` | 1–10485760. |
| `jpeg` | `image/jpeg` | 1–26214400. |
| `png` | `image/png` | 1–26214400. |
| `wav` | `audio/wav` | 1–20971520. |
| `m4a` | `audio/mp4` | 1–20971520. |

The maximums retain the resource owner's 10/25/20 MiB envelopes. Allowing empty
plain text and Markdown, but requiring a nonempty encoding for other formats,
is an explicit v1 selection. An empty original has the SHA-256 of exactly zero
bytes; it is not an absent source. It cannot manufacture a grounded observation
or a successful processing result. Unknown or unsupported media is rejected
instead of accepting a filename extension as proof of support.

The declaration is immutable intent. Actual sniffing, decoding and qualified
format validation must confirm it before canonical binding. Text is valid
UTF-8; JSON rejects duplicate keys and invalid encoding. JPEG/PNG still require
the actual decoded 40-million-pixel ceiling and the qualified animation and
orientation policy. Audio still requires the actual five-minute duration,
qualified codec/channel limits and non-DRM M4A profile. A syntactically correct
MIME pair does not prove any of those properties.

`byte_identity` is a closed `{sha256, byte_length}` value with nonnegative integer
length up to 9007199254740991, the common safe JSON-integer ceiling. It supports
exact representation metadata; that numeric encoding ceiling is **not** a
permitted upload, derived-output, memory or storage size. Uploads use the
narrower union above. Every derived format must have explicit output, scratch,
decoded, CPU, memory and duration limits in its selected processing profile
before an adapter accepts it. Missing profile limits do not permit the numeric
maximum. Input-byte and decoded-byte budgets remain independent.

Preserve original bytes exactly, including line endings, Unicode spelling and
source metadata subject to the source's retention policy. Normalization changes
identity: an LF-normalized representation cannot replace a CRLF original, and
offsets into one cannot be used as offsets into the other.

## Staging grants and upload generations

Each allocation creates a new server-issued `upload_id`, public generation
`{"kind":"upload_generation","value":"1"}`, immutable declared source and
an explicit session `expires_at`. `upload_generation` uses positive canonical
decimal strings in the same 18-digit range as the common positive revision
helper, with its own tag. Content revision, control revision and channel
generation values are invalid substitutes.

V1 does not rotate an existing upload's public generation, renew a terminal
session, or restage different bytes behind a frozen intent. A retry within a
live session preserves its upload ID, generation and declared source. Expired,
aborted or rejected work requires a deliberately new allocation and, where
attached, a revised capture intent. A new identity does not overwrite an older
capture revision or its accepted original. Future generation-rotation behavior
requires its own reviewed operation and compatibility contract; accepting the
wire integer range does not implement that operation.

The public generation is distinct from an internal finalizer lease/fencing
token. Recovery of a lost finalizer can advance its internal fence while retaining
the exact public upload intent. An old lease holder cannot bind or acknowledge
results after takeover. A caller never supplies the internal fence or claims
that an upload is sealed by choosing a field.

`staging_grant` requires `method: "PUT"`, an HTTPS `url`, `required_headers` and
`expires_at`. Its wire limits are explicitly selected here: URL length 12–4096
characters; at most eight header entries; canonical lower-case header names of
1–64 ASCII letters/digits/hyphens with an initial letter; and values of 1–1024
characters without ASCII control characters or DEL. `content-type` is required
and must equal the declared media type. Header values, signed content length
where used and checksum encoding must match the exact selected storage profile
and source. A generic header map cannot authorize an unsupported adapter.
The selected adapter profile must define the permitted header bytes and exact
encoding, including checksum representation. Reject a value that cannot be
represented exactly before transfer; a schema-valid Unicode string does not
authorize lossy conversion or an arbitrary HTTP header encoding.

Clients must parse the URI, reject userinfo and fragments, and verify the
explicitly permitted staging origin and actual connection destination. Do not
accept a regex prefix as proof of a safe URL. Enforce the qualified address,
DNS, proxy and redirect policy; no redirect, arbitrary host override, ambient
credential forwarding or shell evaluation is authorized by this grant.
`authorization`, `proxy-authorization`, `cookie`, `host`, `connection` and
`transfer-encoding` are prohibited grant-supplied headers. Transport-managed
headers still obey the client/server transport contract. Only the profile's
reviewed grant headers may be sent, even if another header name is well formed.

The URL and permitted header values can be scoped bearer secrets. Return them
only to the currently authorized caller over protected transport. Do not put
them in logs, diagnostic bundles, reports, ordinary audit fields, copied shell
commands or fixture/evidence artifacts. Fixtures use reserved example domains
and synthetic values. Public sealed references never reuse the staging URL as
a source identity or expose a permanent download capability.

Before allocation is implemented, the selected upload profile must freeze a
bounded session lifetime, grant lifetime, transfer deadlines, finalization lease
and renewal budget, maximum attempts, total finalization deadline and orphan
safety delay. The grant cannot outlive its upload session. These durations have
not been assigned universal numbers here. Backend-specific support for the
selected lifetimes and transfer behavior needs actual qualification. A shorter
grant does not silently renew through idempotent allocation replay; an expired
grant that cannot complete the live session requires deliberate replacement
allocation/intent under this v1 interface.

## Upload state, finalization and replay

`upload_resource` contains upload ID, organization, generation, declared source,
state and the original session expiry. Its state distinctions are:

| State | Meaning and permitted canonical fields |
| --- | --- |
| `created` | Allocation exists. No accepted binding or failure problem. |
| `uploading` | Staging transfer is in progress. No accepted binding or failure problem. |
| `uploaded` | Staging transfer completion has been observed; bytes remain untrusted. No accepted binding or failure problem. |
| `sealing` | A durable bounded finalization operation owns or is recovering its fenced work. No accepted binding or failure problem. |
| `sealed` | Requires the one accepted `sealed_binding`; no failure problem. |
| `expired` | The unsealed session reached its deadline; no binding or problem field. A rejected finalization request returns its separate typed problem. |
| `aborted` | An explicit authorized cancellation/lifecycle action won; no binding or problem field. Client disconnect is insufficient. |
| `rejected` | A terminal validation/finalization failure requires its typed problem; no binding. |

The normal progression is created, uploading, uploaded, sealing, sealed. Transfer
progress comes from the authorized storage adapter's observations; delayed
progress delivery must be reconciled without treating client claims or object
metadata as proof of sealed content. Only an uploaded, unexpired session can
be newly admitted to sealing. A premature finalization request conflicts; it
does not silently declare an incomplete transfer canonical. The qualified
adapter must define how it observes transfer completion before it exposes this
state, including the direct staging-grant path.

Unsealed states can expire or be rejected under their qualified rules.
The lifecycle owner's upload.abort command handles created/uploading/uploaded;
sealing instead requires operation.cancel and its final binding fence. There is no backward state
transition that silently replaces intent. A retried or recovered finalizer
remains sealing until it binds, expires, is explicitly aborted or reaches its
terminal rejection policy. The resource's accepted original cannot disappear
because an old transfer notification arrives. Once sealed, the upload stays
sealed as historical acceptance; later availability/revocation is a separate
control and cannot rewrite its binding or relabel it as never accepted.

Finalization follows the storage owner's exact sequence:

1. Resolve the upload under current organization, principal, purpose and domain
   authority. Compare route/body identity, public generation and the complete
   expected source with the immutable allocation. Check state, current upload
   expiry and shared capacity. Claim the command and fenced finalization work
   in a short transaction, with audit/outbox, then end that transaction.
2. Read staging once as a bounded stream and write a fresh server-only sealed
   object for that attempt. Hash the exact bytes written, count them and enforce
   the encoded, actual decoded and quarantine requirements. A reused staging
   URL or simultaneous overwrite cannot alter bytes already written into that
   new server-owned object. No worker may overwrite a previously accepted sealed
   object; each failed attempt has a separately accountable orphan.
3. Compare actual bytes/length/SHA-256 with the declared source. ETag, filename,
   MIME assertion, earlier HEAD metadata and another object's digest are not
   shortcuts. A backend conditional-copy or versioning shortcut is permitted
   only after its exact guarantees are qualified against the same invariants.
4. In a second short transaction, recheck current authority, session expiry,
   public generation, state, lease ownership and fence. Bind exactly once and
   atomically record the result, audit and outbox. A deadline crossed during
   streaming, stale lease or revoked authority prevents binding, even if bytes
   were successfully copied. Retain only the controlled orphan/reconciliation
   responsibility from that failed attempt.
5. Reconcile unreferenced staging/sealed bytes after the selected safety delay,
   with dry-run reporting and authorized cleanup that rechecks tenant ownership,
   live finalizers, accepted references and holds. A scanner cannot promote
   plausible orphan bytes into canonical evidence.

Expiry uses the server's authoritative clock, not a client timestamp, and is
exclusive: a new finalization claim or final bind requires current time strictly
before `expires_at`. At the exact expiry instant it is too late.
An accepted bind that committed before expiry remains accepted afterward. The
retained expiry is historical session/grant information, not a deletion time
or a deadline for reading the immutable original under current authority.

Same-key replay reauthorizes and returns the recorded operation or result;
a different fingerprint conflicts. Concurrent finalizers have one accepted
binding and one recorded business outcome for that upload intent. An already
sealed original cannot be replaced by a different idempotency key. Identical
intent under another key may return the existing sealed result after current
authorization and claim recording; different expected bytes or generation
conflict. The command contract never promises a second original from that path.

Allocation and creation response definitions describe the original command
result snapshot. On replay they retain their original IDs, initial state,
initial revisions, grant and expiry rather than pretending to be a current
resource read. Replaying an allocation after its grant expired does not issue
a fresh grant, extend its lifetime, reserve another upload or make its historical
`created` state current again. The consumer must respect the recorded expiry
and use the later current-resource interface where needed. A fresh allocation
is deliberate new intent with a new key; it is not an automatic replay repair.

## Immutable originals and representations

`original_reference` is a closed value containing organization, artifact and
exact version IDs plus the complete verified `source`. `original_version`
wraps that reference with server `recorded_at`. Its record time does not imply
when the photographed, written or recorded event occurred. The original's
organization and source must match the accepted upload and its immutable intent.
An existing same-digest object is not a foreign-reference exception.

`sealed_binding` records the exact upload ID/generation and accepted original.
Persist a unique mapping for that identity. The attachment intent continues to
name the upload, generation, expected source and role; it is never rewritten
into a different reference when sealing completes. Readiness joins that fixed
intent to its separately accepted binding. Retain the binding as long as
canonical references and recovery obligations require it, even after staging
data and short-lived grants are cleaned up.

`representation` has its own representation ID, exact original reference,
independent byte identity, media type, processor slug, processing-profile digest
and server record time. Processor/profile identity must resolve to the actual
qualified transformation, dependencies and configuration. Unknown profiles
cannot confer validity. The media-type syntax allows a bounded canonical
lower-case type/subtype; the selected profile still declares which derived
formats it supports and their actual resource bounds.

Never resolve a representation through the artifact's mutable current head.
Every transformation links to the exact original version and source bytes.
Reprocessing produces a distinct representation when bytes or transformation
identity change; valid reusable derivatives require the exact original and
profile binding. Original identity, representation identity and current
availability remain separate. The grounded-observation contract owns exact
locator variants and interpretation against these representations.

## Capture revisions, controls and ordered attachments

`attachment_intent` is exactly upload ID, public generation, declared source,
role and boolean `required`. Array order is the canonical ordinal; there is no
redundant client `position` field that could disagree with it. Within a revision,
each upload ID appears at most once, including when another role, generation or
digest is supplied. Byte-identical but separately authorized source identities
are not automatically deduplicated or treated as fraudulent duplicates.

The role vocabulary is a deliberate initial v1 choice:

| Role | Meaning |
| --- | --- |
| `evidence` | Source material offered for grounded extraction or review. The label does not prove authenticity or correctness. |
| `user_note` | A supplied written or spoken note, preserved as attributed context. It does not authenticate the speaker or prove another person's agreement. |
| `supporting` | Additional source material associated with the capture, retaining its own grounding and restrictions. |

Role is independent of media and requiredness. A spoken note can be required;
an image can be optional. Unknown roles require an explicit version/profile
amendment rather than silently receiving a default interpretation. The role
does not elevate an embedded instruction into authority or turn a model
inference into a human assertion.

Drafts may contain zero through eight attachments, including an all-optional
list. A submitted revision must contain one through eight and at least one
required part. These minimums are selected here, not inferred from the existing
maximum-eight resource cap. An empty or all-optional draft remains editable but
cannot be submitted or qualify processing through vacuous readiness.

The sum of declared input bytes across the ordered list is at most 100 MiB,
enforced again against the actual immutable upload records after reference
authorization. Count every declared part before storage deduplication. Four
25 MiB images meet the limit; adding another one-byte valid part exceeds it.
An individually valid media envelope cannot bypass the cumulative cap.

`capture_revision` is the immutable capture/organization/domain identity,
content version, user context and complete ordered attachment intent.
`capture_resource` separately reports the current content revision, control
revision, submission state and readiness projection. Creating a capture starts
both revisions at one in draft state. Reading it or observing source readiness
does not edit its content or increment control merely because a projection
refreshed.
A submitted resource has control revision at least two because submission
advances the initial control revision; the schema rejects submitted/control-one.

Revise and submit compare both typed expectations in one current-authority
transaction. The body `expected_version` is a content revision;
`expected_control_revision` is a control revision. This additional explicit
precondition does not overload either type or introduce `If-Match`. Reject
stale expectations and either counter's overflow before effects.

- **Revise:** consume the complete replacement, advance content and control,
  and create a new immutable draft revision. This applies to editing a draft
  and deliberately amending a submitted revision under its observed control
  precondition. Do not patch the old revision, keep an omitted old attachment
  implicitly, or auto-submit the replacement. Both resulting revisions are at
  least two; the revise-response schema rejects either counter at one.
- **Submit:** require the current state to be draft and enforce the submitted
  attachment minimums. Preserve content identity and advance control once while
  recording that exact revision as submitted. A submitted-but-incomplete or
  submitted-with-required-failure snapshot is allowed; submission proves durable
  intent fixation, not processing eligibility or successful work.

Concurrent revise/submit requests with the same content/control expectations
have one winner. Idempotent replay of the winner returns its recorded result
after reauthorization without incrementing again. A different-key attempt
against stale control or an already submitted state conflicts. Changes to list
order, role, requiredness, source, generation or user context require a new
content revision even if bytes happen to match.

New draft creation cannot cancel, replace or retarget work already bound to an
earlier submitted revision. Preserve the old immutable content, submission
command/audit association, results and references under their qualified retention;
detailed command receipts have their separately bounded retention. Receipt expiry
or garbage collection cannot erase the attribution needed by retained submitted
work. Continuing work still requires current authorization;
stopping it requires the explicit operation/lifecycle cancellation rules.
The [lifecycle analysis contract](lifecycle-contract.md#5-acquisition-and-analysis)
owns separately admitted processing and its fixed input/deadlines. Submission
starts no processing operation on admission, replay or later readiness. Retain
its exact immutable command ID, submitted control and actual attribution
independently of detailed command receipts; receipt garbage collection cannot
erase the association required by later analysis.

### Required lifecycle acquisition producer

Every new upload consumer uses the lifecycle upload_resource specialization,
including allocation, GET, accepted/completed finalization and abort snapshots.
Its required control_revision starts at one and advances once per actual state
transition; polling, progress and duplicate delivery do not increment it.
The existing upload_generation remains a distinct type. The lifecycle owner
defines reachable minima, irreversible terminal states and all exact wrappers.
Final original binding also initializes material control and commits the exact
operation effect/success under the current-authority fence and qualified
independent continuity. A fresh-key already-sealed lookup creates no new
operation, original, material control or business effect and owns its own receipt.

capture.analyze requires the current exact submitted capture and expected control,
required readiness and a qualified profile; it freezes ordered input eligibility
and the retained submission association without advancing capture control.
There is at most one live analysis for that exact capture. Optional omitted
parts remain explicit and cannot be appended when readiness later changes.
Success binds the complete immutable analysis record and exact created outputs;
failed partial effects remain observable without a false success manifest.
GET capture revision never substitutes its current head. Complete reads and
current authority apply after command receipt expiry as well as before it.

E010 qualifies concrete stage/result bytes; E063 provides the earliest actual
shared operation, sealing, GET/cancel and upload-specialized consumer. E072–E074
qualify deterministic test Activities and orchestration histories. Actual
capture.analyze support requires E086's qualified real processing integration.

## Readiness and visible optional failure

Each `attachment_readiness` retains its exact `intent`:

- `incomplete` has neither accepted binding nor failure problem.
- `ready` requires its accepted binding and forbids a failure problem.
- `failed` requires a typed problem and may retain a previously accepted binding,
  for example where current source availability now prevents use. The failure
  cannot erase historical accepted bytes or make them accessible again.

The capture's readiness list has exactly the same members, order and intents
as its immutable revision. No optional item is removed, substituted or relabelled
required/optional when readiness changes. Runtime validation compares this
complete correspondence and every intent/binding organization, upload,
generation, format and byte identity; schemas alone cannot compare those
independent repeated values.

| Aggregate readiness | Exact rule |
| --- | --- |
| `failed` | At least one required part failed, even if another required part is incomplete. |
| `ready` | At least one required part exists and every required part is ready. |
| `incomplete` | No required part failed and either no required part exists or at least one required part is incomplete. |

Optional failure or incompleteness remains visible in the ordered part list
even when required readiness is satisfied. Processing may use only eligible
parts according to its explicit profile and must report optional omissions or
failures with permitted partial results. Required failure blocks dependent
processing; a draft's ready inputs still do not imply submission. Source
readiness is also distinct from a successful extraction, transcription,
candidate, review or expense confirmation.

Do not disclose formerly accessible or foreign resource details just to fill a
response. Current policy decides whether the caller may receive that capture
and its permitted failure information. Denying the whole read is preferable to
returning a silently shortened list or leaking an inaccessible source's identity,
bytes, count or owner. Exact export/lifecycle disclosure belongs to its owner.

## Typed failures and compatibility

Use the common registered problems and status/type/code agreement. Acquisition
does not invent a separate boolean success/error convention.

| Failure | Required problem |
| --- | --- |
| Malformed/unknown fields, wrong tags, route/body ID disagreement | `invalid_request`/400. |
| Missing/invalid authentication | `unauthenticated`/401 with its challenge. |
| Inaccessible upload, capture, binding or source identity | `not_found`/404; use `forbidden`/403 only where the shared disclosure policy permits it. |
| Stale generation/revision/control, expired/aborted state, premature finalization, competing transition or idempotency disagreement | `conflict`/409. |
| Encoded or cumulative admitted-byte cap exceeded | `request_too_large`/413. |
| Unsupported format, MIME or actual codec/encoding profile | `unsupported_media_type`/415. |
| Exact source/digest/length disagreement, invalid decoded content, duplicate upload identity in one capture, empty/all-optional submission | `invalid_command`/422, except stale/replaced intent conflicts as above. |
| Shared quota prevents acceptance | `rate_limited`/429 with bounded retry guidance. |
| Required service cannot perform the work | `unavailable`/503, with uncertain effects reconciled under the same command identity. |
| Unexpected server failure | Safe `internal_error`/500 without internal or submitted secrets. |

Same-format metadata equality cannot establish ownership, and error ordering
must not leak a foreign source's actual length, format or state. Authorize the
reference before exposing such comparisons. A stored failure problem in a
resource classifies that work; it does not turn a later successful resource
read into the same HTTP error. Terminal operation/replay behavior follows the
common distinction between admission, work outcome and resource-read transport.

Resources appear under `result.upload` or `result.capture`, preserving the
common ban on direct outcome aliases inside a successful `result`. Response-only
resources, bindings, representations, grants and readiness records tolerate
bounded nonessential extensions under the common reader policy. Fixed original
references, byte identities, upload intents, capture revisions and request
objects remain closed canonical values. Unknown extensions cannot supply an
accepted binding, failure outcome, sealed destination or new authority.

## Prospective semantic and service verification

The parsed-value fixture lane checks closed fields, lexical/media bounds,
reachable example shapes, state-dependent fields and readiness truth tables.
It does not run a server, inspect object bytes, query tenant ownership, compare
separately repeated reference values, add independent lengths or advance a clock.
Do not turn a fixture's asserted context into evidence that those checks ran.

The following stable scenario IDs are required runtime work. Their expected
outcomes must be independently reviewed when promoted into the domain/contract
runner and real service suites; this document does not report them as executed.

| Scenario ID | Required oracle and owning implementation |
| --- | --- |
| `ACQ-AUTH-01` | A same-digest foreign upload/original is denied with no ownership or byte disclosure; current domain/purpose and every consequential reference are checked. Domain/reference persistence and API admission suites. |
| `ACQ-ID-01` | Route/body IDs, immutable allocation/source, capture intent and sealed binding must agree exactly. Different role/expected source cannot disguise a repeated upload ID. Domain and relational suites. |
| `ACQ-TIME-01` | Finalizer claim and bind at expiry minus one tick, exact expiry and plus one tick; expired/aborted/rejected cannot restart, while an earlier sealed result remains accepted. Injected-clock domain and service suites. |
| `ACQ-FENCE-01` | Wrong public generation and stale/replaced internal lease reject binding; takeover preserves public intent and an old holder cannot overwrite the winner. Domain, persistence and finalizer suites. |
| `ACQ-READY-01` | Required failed/incomplete precedence, all-optional/empty drafts, submitted-but-incomplete state, optional failure/incompleteness retained, and exact list correspondence. Domain capture-readiness suite. |
| `ACQ-REV-01` | Role/order/membership/source/context changes create a new revision; concurrent revise/submit has one winner; submit changes only control; old submitted work remains bound to its original revision. Domain and relational suites. |
| `ACQ-SIZE-01` | Four individually valid 25 MiB images total exactly 100 MiB; a fifth one-byte part is rejected. Actual immutable lengths, not trusted client totals, establish the result. Domain and API admission suites. |
| `ACQ-BYTE-01` | Hash/length mismatch, missing/short/overlong objects, misleading ETag and streamed overrun never bind; exactly empty allowed text is distinguished from missing bytes. Actual storage/sealing suites. |
| `ACQ-MEDIA-01` | Every format's real valid/invalid decoder cases, MIME substitution, duplicate-key JSON, image bombs/animation, audio duration/channel/codec and DRM rejection run within qualified limits. Processing/sealing suites. |
| `ACQ-ORIGINAL-01` | Original CRLF/Unicode bytes remain exact; normalized bytes have distinct identity; representations reference the immutable original version, not a changed current head. Domain and processing suites. |
| `ACQ-OVERWRITE-01` | Staging overwrites before, during and after sealing cannot replace the accepted server-only bytes; alternate keys and duplicate finalizers yield one binding. Actual backend qualification. |
| `ACQ-CRASH-01` | Crash before copy, during copy, after sealed write, before bind and after committed bind preserves recovery truth; retry does not duplicate canonical effect/audit/outbox. Relational and service fault injection. |
| `ACQ-REVOKE-01` | Barrier-controlled revocation during streaming prevents a later bind; current authority gates replay and reference disclosure. Domain authorization and service race suites. |
| `ACQ-QUOTA-01` | Concurrent API instances cannot multiply transfer/storage/finalizer reservations; rejected admission creates no accepted work, and uncertain side effects retain controlled responsibility. Shared admission/service suites. |
| `ACQ-GRANT-01` | URI userinfo/fragment/origin/DNS/proxy/redirect and header injection/credential forwarding are denied; signed length/type and expiry agree; no secret appears in logs. Real client/API/storage transfer suites. |
| `ACQ-REPLAY-01` | Allocation/create replay returns recorded initial snapshots without renewing expired grants, changing current state or issuing identities; finalization replay returns its exact operation/result under current authority. API and CLI suites. |
| `ACQ-ORPHAN-01` | Dry-run and interrupted cleanup protect active leases, shared references, holds and another tenant's data; neither an orphan nor a hash match becomes canonical evidence. Orphan reconciliation and recovery suites. |
| `ACQ-CLI-01` | Actual capture/upload/finalization through the native CLI preserves exact ordered intent, digest/length, retry identity and visible optional failures across interruption/resume. Native CLI/API/storage integration. |

The [implementation plan](../execution/open-cli-mvp-v1-rcld.md) assigns capture
readiness and sealing domain transitions to E024/E025, canonical references to
E045, streaming/allocation/submission/finalization/reconciliation to E060–E064
and the real CLI path to E066. E011 promotes the independent scenarios into its
fixture inventory and the later contract runner executes them with the owning
implementation. Those public sequence identifiers record ownership, not
already passing implementation evidence.

[Testing and coverage](testing-and-coverage.md) owns deterministic independent
fixtures, fresh full collection and strict source coverage. [Acceptance](acceptance.md)
owns real-service and workload qualification. No new customer setting, account,
key or backend access is required to validate the declarative family; actual
adapter profiles and required real resources must be recorded and qualified
before the corresponding runtime gate can pass.
