# Common v1 wire contract

**Authority:** normative common wire definitions and admission profile.
**Status:** specified contract with declarative validation; the parser, HTTP
service, authorization and domain operations are not implemented by this work.

The common schema is owned at
[`contracts/public/v1/common.schema.json`](../../contracts/public/v1/common.schema.json).
It supplies named Draft 2020-12 `$defs` to the
[independent contract fixtures](../../contracts/validation/README.md) and
future operation contracts. Its root deliberately rejects every instance.
Consumers must select a named definition; validating against the bare library
cannot certify a request. The versioned path identifies the family. There is no
`$id`, nested identifier, remote reference or dynamic reference; all references
inside the library are internal fragments. A consumer uses a reviewed relative
reference to that file and the selected fragment. Record exact schema and
fixture digests with validation evidence.

[API requirements](api.md) own admission, HTTP semantics and idempotency policy.
[Data model](data-model.md) owns identity meaning and aggregate state machines.
[Resource profiles](resource-profiles.md) own the wider resource envelope.
This document freezes their common wire representation and the previously
unassigned control-JSON profile. It does not define a catch-all command, grant
authority to a syntactically valid request, or replace each operation's contract.

## Definition inventory and composition

| Definition | Purpose |
| --- | --- |
| `identifier`, `sha256` | Canonical opaque UUIDv4 and SHA-256 text. |
| `content_revision`, `control_revision`, `generation` | Noninterchangeable tagged integer values on the wire. |
| `command_context`, `idempotency`, `command_metadata` | Closed request metadata with no domain body or client authority claim. |
| `timestamp`, `idempotency_receipt` | Exact UTC receipt time representation and pending/terminal expiry distinction. |
| `problem` | Typed RFC 9457 failure with correlated type, code and status. |
| `result_object`, `success_response`, `operation`, `accepted_response` | Common outcome distinctions and reserved protocol fields. |
| `health_check`, `readiness`, `readiness_transport`, `liveness` | Role-dependent health values and normalized readiness status/body pairs. |
| `slug`, `command_name`, `positive_revision_value`, `generation_value`, `json_key`, `json_value`, `bounded_object`, `response_object`, `failed_required_check` | Reusable lexical, size and cross-field building blocks; these are not complete endpoint contracts. |

An operation owns a closed full request with its domain variants and the shared
metadata. For example, an operation may nest a reference to `command_metadata`
under its own `metadata` property and define its closed `body` beside it. The
operation contract must choose that concrete layout and required precondition;
the example does not establish a universal request envelope. Do not extend the
closed metadata object through `allOf` and then assume additional members are
accepted. Reference its component definitions when a different closed layout
is needed. Reject unknown operation names, bodies and variants against the
implemented capability inventory before admission. No arbitrary object body is
accepted merely because its metadata conforms.

Response definitions establish a common floor. A producing operation must also
define its exact result, progress and resource fields and enforce the associated
references and state transitions. Neither an empty conforming `result_object`
nor an unrecognized response extension proves domain completion.

## Identifiers and independent revisions

`identifier` is exactly 36 ASCII characters in canonical lower-case UUIDv4
form, including version `4` and variant `8`, `9`, `a` or `b`. The server issues
organization, domain, operation and request identifiers. Clients may refer to
existing identifiers but cannot choose ownership by supplying a UUID. Reject
uppercase, other UUID versions, braces, omitted hyphens, whitespace, appended
newlines and noncanonical alternatives. A UUID is not a capability, clock or
sequence number. Canonical principal identity remains the identity mechanism
in the [data model](data-model.md#identity-and-ownership); a client-supplied
principal UUID cannot replace authenticated identity.

Digests are exactly 64 lower-case hexadecimal characters. A digest validates
byte identity only after the server has checked the referenced resource and
organization. It is not an existence oracle or access grant.

| Domain integer | Wire object | Inclusive range |
| --- | --- | --- |
| Content revision | `{"kind":"content","value":"1"}` | 1 through 999999999999999999. |
| Control revision | `{"kind":"control","value":"1"}` | 1 through 999999999999999999. |
| Generation | `{"kind":"generation","value":"0"}` | 0 through 999999999999999999. |

These are integer domain types represented by tagged decimal strings, not
fractional JSON numbers. The tag prevents substitution between equal-looking
content, control and generation counters. Eighteen decimal digits fit within a
signed 64-bit PostgreSQL integer while avoiding binary64/JavaScript precision
loss. This is the selected v1 range, not a claim that all 64-bit values are
supported. Reject numeric JSON values, signs, decimal points, exponent notation,
leading zeroes, whitespace, wrong tags and extra fields. Zero is unavailable to
content/control revisions. Generation zero represents the initial position
before a channel's first mutation; every accepted creation or later mutation
advances it. An empty channel is not necessarily at generation zero.

Parse into the correct checked integer type. At the maximum value, a mutation
that would advance it fails without changing state; wrapping, resetting,
rounding and recycling a deleted name's counter are forbidden. Wall-clock time
does not resolve competing updates. Each operation chooses its exact typed
body `expected_version` or `expected_generation` and compares it atomically
with current authority and state. Schema validation cannot perform that check.

## Closed metadata and actor authority

`command_metadata` requires exactly `contract_version`, `command`, `context`
and `idempotency`. `contract_version` is the string `v1`. Command names contain
at least two dot-separated components; each starts with a lower-case ASCII
letter and continues with lower-case letters, digits or underscores. The whole
name is 3–64 characters. A well-formed name is not evidence that the operation
exists or is enabled.

`command_context` requires exactly `organization_id`, `domain_id` and `purpose`.
Purpose is a 1–64-character lower-case ASCII slug: an initial letter followed by
letters, digits or underscores. Each operation defines its permitted purpose
values and required resource scope. The purpose selects requested intent; it
does not assert an authorization result. The server checks active same-tenant
membership, domain ownership, current purpose policy and every referenced
resource using the authenticated principal. Actor identity, grants, roles,
approval authority, internal workflow IDs and a client assertion that policy
already passed are not metadata fields.

All three metadata objects are closed, including the nested idempotency object.
The server must reject a caller attempting to add `actor_id`, `roles`,
`authorized`, `approved`, a precomputed fingerprint or a second request body.
A human-approval flag cannot confer the scoped authority required by
[security and privacy](security-and-privacy.md). Earlier explicit delegation
remains usable according to that policy; this contract adds no per-command
human prompt.

## Idempotency identity and receipts

`idempotency` contains only `key`: 32–128 ASCII letters, digits, underscores or
hyphens, preserving case exactly. Clients generate keys with at least 128 bits
of cryptographic randomness, persist the key with its original intent before
submission, and reuse it after a lost response. The alphabet/length constraints
do not prove entropy. Keys contain no credentials or customer content and do
not appear in diagnostic URLs or unrestricted logs. The wire key is not a UUID
and must not be regenerated for an attempt, polling request or reconnect.

The key digest is SHA-256 of the key's exact UTF-8 bytes. Scope the durable claim
by organization, authenticated principal, registered command name and this
digest. Never trust client-supplied scope or digest to identify a claim.

The request fingerprint is SHA-256 over this exact preimage:

```text
UTF8("arboresce.command.v1\0") || JCS({
  "method": uppercase HTTP method,
  "resource": canonical origin-relative operation path,
  "contract_version": "v1",
  "purpose": context.purpose,
  "body": complete parsed request body
})
```

Here `\0` is one zero byte; it is not the two printable characters backslash
and zero. `JCS` means the vetted RFC 8785 canonicalization and numeric/Unicode
profile in the [data model](data-model.md#canonical-builds), narrowed by the
control-JSON profile below. `body` includes all submitted metadata, the exact
idempotency key, every domain field, explicit nulls and ordered arrays. There
is no exclusion for apparently cosmetic fields. Whitespace and object-member
order disappear through canonicalization; meaningful array order, strings and
values do not. Server-generated request IDs, execution time and retry attempt
are not added to this preimage.

The server derives the uppercase method and registered canonical path from the
matched operation, not an untrusted body assertion. Initial mutation routes
use ASCII path segments and canonical identifiers, with no query, fragment,
dot segment, duplicate separator or alternate percent-encoded spelling.
Each operation freezes its exact route and method. Reject alternate spellings
instead of allowing a proxy or client to choose a different fingerprint for
the same effect. Origin, host and authorization headers are not fingerprint
inputs; current authenticated principal and tenant remain part of claim scope.

`idempotency_receipt` requires `key_digest`, `request_fingerprint`, `expires_at`,
`tombstone_expires_at` and boolean `replayed`. Timestamp strings are exactly UTC
`YYYY-MM-DDTHH:MM:SS.mmmZ`, with a valid Gregorian date and seconds 00–59; offsets,
missing milliseconds, leap-second strings and year zero are unsupported.
Keep format assertion enabled when validating this schema. Millisecond receipt
precision is not an ordering or synchronization primitive.

Both expiry fields are null for pending durable work; neither key retention
nor a new effect starts merely because client waiting ended. Both become exact
timestamps when terminal policy applies. Mixed null/timestamp pairs are invalid.
`expires_at` precedes `tombstone_expires_at`; the server must enforce their
relationship to the original terminal time and the completed-key/tombstone
retention policy owned by [API](api.md#idempotent-command-admission). A receipt
does not shorten that policy. Replay does not restart retention. A null pair
does not authorize unlimited job execution: operation lifetime, reconciliation
and retry limits must be selected by the owning execution profile before it
accepts work.

An identical replay returns the recorded operation or result only after current
authorization. A different fingerprint conflicts. The server applies admission,
claim/effect/audit/outbox atomicity and revocation serialization from the API and
storage owners; no schema or fingerprint establishes those guarantees. Once a
claim reaches its explicit expiry, require deliberate new intent instead of
silently repeating an expired command.

## Problems and outcome variants

`problem` requires RFC 9457 `type`, `title`, `status` and `detail`, plus stable
`code` and opaque server `request_id`. Optional `instance` is a canonical
`urn:uuid:` reference to that request ID; equality with `request_id` is a
semantic check. The problem type is `urn:arboresce:problem:v1:` followed by the
code. These URNs identify the types documented here; they do not require a
network schema or a deployed documentation endpoint.

| Code | Status | Meaning |
| --- | --- | --- |
| `invalid_request` | 400 | Malformed or unsupported request shape. |
| `unauthenticated` | 401 | Missing or invalid authentication. |
| `forbidden` | 403 | Denied action where policy permits disclosure. |
| `not_found` | 404 | Unknown route or inaccessible resource identity. |
| `conflict` | 409 | Revision, generation, transition or idempotency conflict. |
| `request_too_large` | 413 | Enforced request size exceeded. |
| `unsupported_media_type` | 415 | Unsupported content or media type. |
| `invalid_command` | 422 | Valid request syntax with invalid domain meaning. |
| `rate_limited` | 429 | Shared admission prevents acceptance. |
| `internal_error` | 500 | Unexpected server failure, with a safe bounded diagnostic. |
| `unavailable` | 503 | A dependency required for the work is unavailable. |

`internal_error` is the common v1 fallback for an unexpected failure; it does
not expose internal exception classes or suggest retrying an uncertain effect
with a new key. The other status meanings are owned by the API contract. HTTP
error status, problem status, code and type must agree. Send top-level problems
as `application/problem+json`; retain the required authentication challenge for
401 and bounded retry guidance for 429. The operation contract specifies
applicable `Retry-After` behavior for 202 and 503.

`title` is a human-readable summary of 1–128 characters, not a machine selector.
`detail` is 1–2048 characters of safe explanation. Do not include foreign IDs,
resource-existence clues, SQL, addresses, credentials, submitted content or
stack traces. Clients select behavior from the registered code/type rather
than comparing prose. Body shape cannot prove safe disclosure.

| Definition/state | Required interpretation and shape |
| --- | --- |
| `success_response` / `succeeded` | Requires request ID and a domain `result_object`; no problem, failure code or operation-admission claim. A supplied idempotency receipt has terminal timestamps. Each mutating operation must require its receipt. |
| `operation` / `queued`, `running` | Requires operation, organization and domain IDs; no terminal result or problem. |
| `operation` / `succeeded` | Requires a domain result and forbids a problem. |
| `operation` / `failed` | Requires a typed problem and forbids a success result. |
| `operation` / `cancelled` | Has neither terminal success result nor failure problem. It does not imply that committed effects were undone. |
| `accepted_response` / `accepted` | Requires request ID, a queued/running operation and a pending receipt with null expiry pair. HTTP 202 reports durable admission, not success. |

The operation resource and durable admission are distinct facts. The initial
202 representation is a nonterminal admission snapshot and its `Location`
identifies the same public operation. It is not a promise that work remains
nonterminal when the client receives it. A later poll, fast-completion read or
reauthorized replay must return the actual recorded terminal operation/result;
it cannot force a finished operation back into an `accepted_response`.
Operation reads return HTTP 200 for a successfully retrieved operation resource,
including one whose state is `failed` or `cancelled`. That transport success is
not command success. A terminal failure's nested problem classifies the failed
work; its status need not equal the 200 status of a later resource read.
Each operation's endpoint contract must define this resource-read/replay variant
and terminal receipt, or its equivalent direct recorded result/problem variant,
before implementation. No replay repeats effects to obtain a new response shape.

The operation/lifecycle contract must define typed progress, stage/attempt
distinctions, committed-effect references, cancellation races and reconciliation.
Those fields are not invented as arbitrary extension payloads here. In
particular, a cancelled operation needs its independently retained committed
effect record; absence of a `result` does not mean absence of effects.
Cancellation requests and their revision preconditions belong to that later
contract, not to a generic common command.

## Bounded response compatibility

Common response readers tolerate unknown additive fields, with at most 32 total
properties and the bounded JSON values below. Producers use documented
lower-case slug names of 1–64 characters for new fields. Reader schemas accept
unknown names under the general 1–128-character object-key bound, including
uppercase and hyphenated RFC 9457 extension names; the producer convention must
not become a reader rejection of harmless future fields.
An extension cannot change the meaning or required interpretation of existing
fields. New server extensions need an owning specification; consumers ignore
unknown nonessential extensions without treating them as authorization or a
different outcome. Problem extensions, nested idempotency receipts and health
checks follow the same rule. Fixed tagged revision values and request objects
remain closed; their additional members are unsupported value/command variants,
not response extensions.

Known outcome names remain reserved even when their value is null or false.
The consuming schema prohibits inappropriate sibling fields such as `code`,
`problem`, `error`, `failure`, `failure_code`, `success`, `ready`, `accepted`,
`operation`, `operation_id`, `status`, `result` or `state`. A property's presence
cannot be used to smuggle a contradictory second result. Use the consuming
definition's exact property table; names meaningful to that variant are typed,
not prohibited. `result_object` itself also forbids direct `state`, `code`,
`problem`, `error`, `failure`, `failure_code`, `success`, `ready` and `accepted`.

An operation may expose an explicitly typed resource beneath a domain field
whose own state differs from the outer command outcome. For example, retrieving
a record successfully does not change that record's availability. Domain
contracts own those nested states and any optional-attachment warnings; they
cannot override the common envelope or disguise a failed required effect as
success. Arbitrary recursive field-name bans would incorrectly erase source
content and are not an authorization or semantic validator.

## Health and actual transport

`liveness` is a process-only response with `state: "live"` and `live: true`.
Its success does not prove dependency access, workload capacity or readiness.
A process that cannot respond does not need to manufacture a false healthy
body. The live endpoint's successful response uses HTTP 200.

`readiness` requires `state`, boolean `ready`, public role slug and a `checks`
object with 1–16 named public capabilities. Each check requires boolean
`required` and `available` and permits bounded nonessential reader extensions.
Names refer to declared public capabilities, not
database hostnames, provider accounts, internal addresses or private topology.
The role's required-check policy is configured and qualified before serving;
clients cannot mark a required dependency optional to force readiness.

`ready: true` requires `state: "ready"` and no unavailable required check.
`ready: false` requires `state: "unavailable"` and at least one unavailable
required check. An unavailable optional search capability can coexist with
ready exact-build serving. A failed required check cannot be hidden by adding
successful optional checks or reporting liveness instead.

`readiness_transport` is exactly `{status, body}` with status 200 for a ready
body and 503 for an unavailable body. Its positive and negative fixtures
validate this normalized observation. They do not start `/health/ready` or
prove its actual HTTP status. The future process test must inspect the real
status and JSON response under dependency loss/recovery, including the optional
search case. Health responses obey the same bounds and disclosure policy.

## Selected common control-JSON profile

The following are deliberate v1 control-plane choices. They complete the
previously unassigned nesting and collection limits in the
[resource envelope](resource-profiles.md#initial-resource-envelope); they are
not previously measured capacity results or universal artifact/parser limits.
Binary uploads, media, downloads and separately versioned exports retain their
own contracts. An operation may tighten these limits explicitly. A wider
control envelope requires a reviewed version/profile change and qualification.

Build/resolve consumers must preserve the existing 256 KiB compiled-text
envelope and the [API's complete exact-serving rule](api.md#exact-snapshot-resolution-and-search-selection).
The common 65536-scalar string bound cannot silently narrow that requirement.
The owning build/resolve contract must define a bounded complete-content
representation, including a separately qualified content stream or profile
where needed, before composing its schema with these definitions. Passing
common metadata or a smaller control envelope never permits truncation, an
implicitly partial build, or a claim that an incomplete response is exact.

| Property | Inclusive bound and enforcement |
| --- | --- |
| Command bytes | 1 MiB of UTF-8 JSON, enforced while reading and before full parsing; this retains the resource owner's existing cap. |
| Common control response bytes | 1 MiB of UTF-8 JSON; enforce before emission and while client reading. This is a newly selected common-response cap, not a download limit. |
| Nesting | At most 16 nested object/array containers; the root container has depth 1, scalar children add no container level. Enforce during parsing. |
| Object members | At most 128 per general object; smaller closed objects retain their exact fields. |
| Array elements | At most 256 per general array, before operation-specific narrower limits. |
| General string | At most 65536 Unicode scalar values; byte limit applies independently. |
| General object key | 1–128 Unicode scalar values; no U+0000–U+001F or U+007F. |
| Protocol response members | At most 32, with general object-key bounds for readers; producers name new fields with 1–64-character lower-case slugs. |
| Readiness checks | 1–16 public capability entries; each requires the two booleans and has the bounded response-member policy. |
| General JSON number | Finite IEEE 754 binary64, within -9007199254740991 through 9007199254740991. Exact integers, money and independent revisions use their declared types instead of approximate fractions. |

The byte cap bounds aggregate work across many small collections; depth and
per-container caps bound nesting and fan-out. The 18-digit revision string
range deliberately exceeds the safe integer range of JSON numbers. Never cast
it through binary64. Fractional general JSON numbers have binary64 semantics,
not arbitrary-precision decimal semantics. Reject overflow and a nonzero
numeric token that underflows to zero. Canonicalization of supported numbers
follows RFC 8785, including its treatment of negative zero. Monetary values
remain governed by [financial records](financial-records.md#exact-monetary-values).

Reject malformed UTF-8, unpaired surrogates, duplicate object keys at every
depth, non-finite tokens, trailing data and unsupported encodings. The initial
control request is uncompressed UTF-8 `application/json`; unsupported content
encodings are rejected rather than expanded without a declared bound. A UTF-8
byte-order mark is not part of a valid control document. No Unicode normalization
rewrites a submitted string before identity calculation. JSON escaped and
literal spellings that decode to the same valid string have the same canonical
meaning. Regex boundaries use a true end assertion and fixed lengths where
applicable; an appended line terminator is never accepted as a canonical ID,
revision, key or slug.

JSON Schema enforces the listed per-value, per-container and closed-object
bounds after parsing, plus typed variants and status relationships. It does
not enforce aggregate wire bytes, container depth, raw duplicate keys, valid
UTF-8, number-token underflow or the server's parsing/canonicalization policy.
Its string lengths agree with scalar-value counts only after invalid Unicode
has been rejected. Keep date-time format assertion and the documented regex
behavior enabled in the pinned validator profile. A schema check without those
semantics is not the supported contract check.

## Verification boundaries and pending runtime vectors

The [standalone fixture lane](../../contracts/validation/README.md) checks
independently authored parsed JSON values and a fixed expectation inventory.
Every expected rejection is a value assertion, not an inverted process exit:
broken schemas, absent references and validator errors stay failures. Fixed
manifest order is part of that declarative inventory; reordering records changes
the manifest. Future runtime suites may vary execution order using stable case
identities. A successful parsed-value lane proves only the constraints selected
by its named definitions.

The following runtime scenario IDs reserve required tests. They are not executed
by the parsed-value manifest and must not be reported as passing runtime evidence.

| Runtime scenario ID | Required independent oracle |
| --- | --- |
| `RAW-01` | Duplicate keys at root and nested objects fail before admission; include escaped/literal keys that decode identically. |
| `RAW-02` | Invalid UTF-8, lone surrogates, byte-order mark, non-finite/underflow tokens, incomplete JSON and trailing second documents fail. |
| `RAW-03` | Exactly 1 MiB versus one byte more; enforce the streamed cap without trusting Content-Length or buffering an unlimited body. |
| `RAW-04` | Depth 16 versus 17 with mixed objects/arrays; counts at and over 128 members and 256 elements; multi-byte string byte/scalar boundaries. |
| `RAW-05` | Valid binary64 and exact string-number distinctions produce independent RFC 8785 bytes/digests, including UTF-16 property order and Unicode escape equivalence. |
| `CMD-01` | Registered command/body variants accept only their complete closed shape; unknown names, nested intent fields, wrong revision types and overflow cannot mutate state. |
| `CMD-02` | Method/path/purpose/body changes alter the fingerprint; object-key order and whitespace alone do not; alternate routing spellings fail. |
| `CMD-03` | Same-key lost-response replay returns the recorded identity after current authorization; different fingerprint conflicts and expiry never silently repeats an effect. |
| `AUTH-01` | Tenant/domain substitution, fake actor/approval claims and inaccessible references fail without foreign-resource disclosure. |
| `AUTH-02` | Barrier-controlled replay/revocation and same-revision competing writers serialize against current canonical authority. |
| `HTTP-01` | Actual durable admission returns 202 with the matching operation `Location` and applicable retry guidance; failed admission returns no accepted operation. |
| `HTTP-02` | Actual HTTP error status, problem code/type/status, content type and required 401/429 headers agree; safe diagnostics disclose no prohibited values. |
| `HTTP-03` | Required dependency loss returns readiness 503, recovery returns 200, and optional-search loss can retain ready exact-build serving. |
| `HTTP-04` | An already terminal operation is returned faithfully on poll/replay without re-admission; transport 200 of a failed resource is not command success. |
| `STATE-01` | Completion/cancellation races preserve the actual terminal state and previously committed effects, with explicit reconciliation where needed. |
| `TIME-01` | Injected-clock expiry and tombstone arithmetic obey original terminal-time retention; pending/null and terminal timestamps cannot mix or reset on replay. |
| `LIMIT-01` | Response emission and client reads obey the new byte cap; unsupported content encodings fail and no truncated result is labelled complete. |

These tests join their owning parser, API, operation, authorization and storage
implementation slices. [Testing and coverage](testing-and-coverage.md) owns
fresh complete collection, deterministic independent fixtures and strict source
coverage; [acceptance](acceptance.md) owns real-service and workload qualification.
Passing schema fixtures cannot substitute for either gate.
