# Bounded listing and search selection

**Authority:** normative wire contract. **Status:** accepted contract; schema qualification and every runtime capability require their owning measured evidence. Documented behavior does not establish implemented or qualified capability support.

This document owns the finite listing, exact companion-read and search-selection wire profiles. [API](api.md#listing-and-filters) owns route registration and shared HTTP behavior; [common](common-contract.md) owns shared encodings/envelopes; [data model](data-model.md#evidence-and-review-identity) owns immutable identity and reviewed relationships; [security](security-and-privacy.md#canonical-admission-and-human-assurance) owns current disclosure/delegation; [storage and search](storage-and-search.md#bounded-search) owns canonical retrieval and persistence; [resource profiles](resource-profiles.md#initial-resource-envelope) owns numerical admission limits; [testing and coverage](testing-and-coverage.md) owns independent qualification. The corresponding owning amendments must agree before adoption.

There are eighteen operations: ten bodyless listing GETs, seven bodyless exact companion GETs, and one synchronous POST search. These operations do not mutate content, create durable search operations, expose general operation/audit/feedback histories, or turn selected content into an approved release. Exact context resolution and immutable financial report/export selection retain their existing owners and complete-or-error behavior.

Protocol ceilings selected here are page size 1–100 with default 20; raw GET target 8192 bytes and 32 query pairs; cursor token 4096 bytes and decoded payload 2048 bytes; a fixed nonrenewing 900-second cursor lifetime; a search query of at most 4096 Unicode scalars/16384 UTF-8 bytes; search request 65536 bytes; canonical selected output 262144 bytes; complete listing/search response 524288 bytes; complete content GET 1048576 bytes; exact supersession-record GET 65536 bytes. Common depth16 and additive32/collection bounds remain. These are protocol ceilings, not measured capacity or a selection of actual operational keys/models. Search retains k=1–20, two total retrieval rounds and 200 cumulative pre-deduplication candidates across every source, current authorization before external reranking, a ten-second API deadline and two-second ordinary database transaction budget.

All illustrative type compositions below are prose definitions, not schemas or fixture instances. Objects are closed unless expressly Additive32. Required fields, nullable positions and finite branches are explicit; imported types keep their exact owners. Typed GET request observations represent route/query decoding and never authorize a GET JSON body.

## Operation inventory

| Operation | Method and route | Named response |
| --- | --- | --- |
| capture.list | GET /v1/captures | listing_response_capture_heads |
| capture.revisions.list | GET /v1/captures/{capture_id}/revisions | listing_response_capture_revisions |
| candidate.list | GET /v1/candidates | listing_response_candidate_heads |
| candidate.revisions.list | GET /v1/candidates/{candidate_id}/revisions | listing_response_candidate_revisions |
| asset.list | GET /v1/assets | listing_response_asset_heads |
| asset.versions.list | GET /v1/assets/{asset_id}/versions | listing_response_asset_versions |
| expense.list | GET /v1/expenses | listing_response_expense_heads |
| expense.revisions.list | GET /v1/expenses/{expense_id}/revisions | listing_response_expense_revisions |
| channel.list | GET /v1/domains/{domain_id}/channels | listing_response_channel_heads |
| channel.generations.list | GET /v1/domains/{domain_id}/channels/{name}/generations | listing_response_channel_generations |
| candidate.read | GET /v1/candidates/{candidate_id} | candidate_current_get_response |
| candidate.revision.read | GET /v1/candidates/{candidate_id}/revisions/{version} | candidate_revision_get_response |
| observation.read | GET /v1/observations/{observation_id} | observation_get_response |
| asset.read | GET /v1/assets/{asset_id} | asset_current_get_response |
| asset.version.read | GET /v1/assets/{asset_id}/versions/{version_id} | asset_version_get_response |
| asset.supersession.read | GET /v1/assets/{asset_id}/supersessions/{supersession_id} | asset_supersession_get_response |
| channel.generation.read | GET /v1/domains/{domain_id}/channels/{name}/generations/{generation} | channel_generation_get_response |
| search.select.v1 | POST /v1/context/search | search_response |

Every GET is read-only and bodyless, with no command/idempotency receipt. Search alone takes the closed POST body. A declaration of these interfaces does not advertise a runnable server; actual engine and independent client qualifications precede support.

## L.2. Minimum useful list inventory

All ten operations are HTTP GET, read-only, bodyless, and return HTTP 200 common
success without a command/idempotency receipt. Every request supplies exactly
one `organization_id`, `domain_id` and `purpose`, using common context types.
All path scope must match query scope. The table adds only the declared query
keys from sections L.3–L.4 to the existing exact-GET selector rule. Nothing is a
generic type/ID endpoint.

| Family | Route | Why required | Fixed descending key |
| --- | --- | --- | --- |
| `capture_heads` | `/v1/captures` | Revisit recent domain captures and their actual submission/readiness; storage explicitly identifies this workload. | `(created_at, capture_id)` |
| `capture_revisions` | `/v1/captures/{capture_id}/revisions` | Inspect retained submitted/amended intent instead of substituting the current revision. | Exact numeric content `version.value` |
| `candidate_heads` | `/v1/candidates` | Discover pending/current review work and retain other visible outcomes. | `(created_at, candidate_id)` |
| `candidate_revisions` | `/v1/candidates/{candidate_id}/revisions` | Inspect the exact revisions and preserved verdicts required by review. | Exact numeric content `version.value` |
| `asset_heads` | `/v1/assets` | Discover accepted current asset versions for explicit build selection. | `(created_at, asset_id)` |
| `asset_versions` | `/v1/assets/{asset_id}/versions` | Inspect retained asset history and select exact historical versions under current eligibility. | Exact numeric `version_order.value`, defined below |
| `expense_heads` | `/v1/expenses` | Revisit proposed/confirmed/voided records and inspect period candidates without treating the queue as a report. | `(created_at, expense_id)` |
| `expense_revisions` | `/v1/expenses/{expense_id}/revisions` | Inspect preserved corrections, confirmations and void history. | Exact numeric content `version.value` |
| `channel_heads` | `/v1/domains/{domain_id}/channels` | Discover retained named channels and current generation/state. | ASCII `name` |
| `channel_generations` | `/v1/domains/{domain_id}/channels/{name}/generations` | Find the exact retained prior active generation and build required for rollback. | Exact numeric nonzero `generation.value` |

Current-head rows each have one immutable `created_at` belonging to the aggregate
discovery record. Set it once from the initial canonical creation transaction's
injected-clock observation, not on each revision or state transition. For
candidates/expenses this equals the initial revision's recorded time; for assets
it equals the first accepted mapping's recording time. Captures need this
explicit metadata outside their existing immutable capture content shape. The
creation metadata and unique aggregate ID make ordering stable even when the
current selected revision changes. The response labels this creation time
explicitly, rather than confusing it with the current revision's recorded time.

All integer keys are parsed/compared as checked exact integers in the common
range; neither lexicographic decimal-string order nor binary64 conversion is
permitted. Tuple tie-breaker IDs use canonical ASCII order. No timestamp or UUID
claims mutation or commit chronology. New current-head revisions do not move an
already traversed aggregate into the remaining key range.

### Asset ordering

The candidate-review contract gives accepted assets an opaque `version_id` and a candidate reference. Candidate revision counters belong to the candidate, and candidate supersession does not move an asset head. Neither that counter nor a version UUID can provide the required exact asset-history order.

Select `version_order: common.content_revision` on each immutable asset-version
discovery record: start at 1, allocate the next value for this asset under its
canonical aggregate lock/CAS, enforce unique `(organization_id, asset_id,
version_order)` and exact version-ID association, and reject overflow before
effects. Once assigned it never changes or gets reused. It is ordering metadata
outside the unchanged candidate-review contract `accepted_asset_reference` and content hashes. Asset
head changes remain explicit reviewed supersession; listing does not create them.
The asset domain model, canonical persistence constraints and actual acceptance/supersession writers must preserve this ordering. It is an owning data-model and producer obligation, not an incidental cursor detail.

### Exact companions, rather than additional broad enumeration

Complete this finite navigation inventory alongside the lists:

- GET `/v1/candidates/{candidate_id}` and
  `/v1/candidates/{candidate_id}/revisions/{version}` return complete current or
  exact candidate-review contract candidate resources, respectively; retain all content/evidence
  roles and distinguish per-revision state from aggregate control.
- GET `/v1/observations/{observation_id}` returns the complete observation contract observation
  after current source/dependency disclosure. Candidate evidence and accepted
  analysis outcomes already supply exact observation references; no global
  observation search/listing or writable observation route is added here.
- GET `/v1/assets/{asset_id}` and `/v1/assets/{asset_id}/versions/{version_id}`
  select the current or exact immutable asset. The resource sections below freeze their complete
  candidate-review contract content/review, bounded reviewed-supersession references and current
  head/exact-version lifecycle composition.
  An exact accepted reference alone is not the complete asset read resource.
- GET `/v1/assets/{asset_id}/supersessions/{supersession_id}` returns one complete
  retained reviewed head-transition record under current relationship disclosure.
  It contains exact old/new references, controls, approval and execution history;
  it does not inline either version's content or follow a relationship chain.
- GET `/v1/domains/{domain_id}/channels/{name}/generations/{generation}` returns
  the complete exact context contract `channel_event`. Unlike the compact list entry, this
  contains the retained reason and attribution. Path generation is the canonical
  nonzero decimal value; it cannot name current head implicitly.

These are **seven** new exact GET routes: two candidate, one observation, two asset
content reads, one asset-supersession record and one channel-generation.
Existing [capture](acquisition-contract.md#complete-commands-and-routes) and
[expense](financial-records.md#complete-commands-and-reads) exact reads,
[current-channel reads](context-contract.md#identity-and-observed-resources),
exact reports, builds and their operation/evaluation/approval/attestation references
retain their own routes and resources. They need no broad build/report/operation
history list to fulfill these journeys. No audit/hold/repair/feedback collection,
organization-wide evidence dump, report-item pagination or arbitrary graph
traversal is added by selection contract listing. Later actual product parity may add a reviewed
endpoint, not silently infer one from this profile.

## L.3. Wire request bounds and common normalization

Choose `listing_v1` with page size 1–100, default 20. `page_size` is a singular
canonical base-10 unsigned integer query value with no sign, exponent, leading
zero or whitespace. `cursor` is singular and optional only on the first page;
an empty cursor is invalid. Continuations resubmit the same full context and
filters. Resolved defaults, family, path selector, summary view, ordering and
page size must match the cursor. A cursor cannot replace omitted required
context or import prior filters implicitly. Changing page size restarts traversal.

Bound the raw origin-relative GET request target to 8192 bytes and query pairs
to 32 before allocation/decoding. Decode each key/value once as strict UTF-8;
reject malformed percent escapes, invalid Unicode, encoded NUL/control bytes,
empty components, empty values, and duplicate keys after decoding unless the
table explicitly marks that key repeatable. `+` is not a space alias: these
types have no plus-valued input, so reject it. Re-encoded percent sequences do
not acquire a second interpretation. Query keys are exact lower-case names
after decoding; no brackets, comma-separated set syntax or dotted expressions.
Canonical route IDs/names follow existing path rules rather than permissive
double-decoding or alternate path spellings.

Repeatable terms accept one exact primitive per occurrence. Reject duplicate
values before normalization, including percent aliases. Sort each accepted set
by ASCII to normalize OR order; distinct filter terms combine with AND. No
trim, Unicode normalization, case folding, empty-set or string `null` convention
exists. A missing term means that term is unrestricted or takes the explicit
default below; null and empty values are invalid GET input. Explicitly listing
every value in a finite enum normalizes to omission when omission means that
same complete enum. Unknown filters fail before any data query. There is no
SQL/Cypher/JSONPath/model/shell grammar, wildcard or operator-in-value escape.

`created_from` is an inclusive common UTC timestamp and `created_before` an
exclusive one. Both concern the immutable aggregate discovery time only.
Either can appear alone. If both appear, require strict lower < upper; equal or
reversed bounds are an invalid request. There is no implicit process timezone.

## L.4. Closed per-family filters

Context, `page_size` and `cursor` are transport selectors, not extra filters.
The following is the complete proposed listing filter vocabulary. History
families intentionally allow **no filters**; they enumerate visible retained
revisions/generations under the required exact parent path.

| Family | Permitted filters and exact default |
| --- | --- |
| `capture_heads` | Repeatable `submission_state=draft\|submitted` (default both); repeatable `readiness=incomplete\|ready\|failed` (default all); optional `created_from`, `created_before`. Readiness is the current canonical readiness observation, not the outcome of a processing operation. |
| `candidate_heads` | Repeatable `revision_state=pending\|accepted\|rejected\|contested` (default `pending`); repeatable `knowledge_kind` using all eight existing candidate-review contract values (default all); optional `created_from`, `created_before`; at most one review-due predicate below. A superseded revision cannot be a current head. |
| `asset_heads` | Repeatable `availability=active\|deprecated\|revoked\|purged` (default `active`); repeatable existing `knowledge_kind` (default all); singular `eligibility=eligible\|visible` (default `eligible`); optional `created_from`, `created_before`; at most one review-due predicate below. |
| `expense_heads` | Repeatable `revision_state=proposed\|confirmed\|voided` (default all); repeatable `currency=AUD\|CAD\|CHF\|EUR\|GBP\|USD\|JPY\|KWD` (default unrestricted, which includes unknown currency); optional `created_from`, `created_before`; the exact expense-date predicate below. Historical superseded revisions cannot be heads. |
| `channel_heads` | Repeatable `state=empty\|active\|suspended` (default all). No arbitrary prefix/name pattern or mutable generation-order sort. |
| Five revision/version/generation history families | No family-specific filter key. Current authorization remains mandatory; state/control changes do not erase exact history or turn it into an exhaustive snapshot. |

Eight candidate-review contract knowledge kinds are `reported_fact`, `thesis`, `heuristic`, `method`,
`exception`, `policy`, `evaluation`, `negative_lesson`. Requests selecting an
enum value never redefine the underlying classification or workflow state.

For asset heads, `eligible` requires current canonical permission to select/use
the exact version for the requested purpose, including applicable restrictions,
evidence and review policy. `visible` permits only independently disclosable
metadata and does not assert serving/build eligibility. Listing never repairs
availability or grants use. Do not return hidden causes, dependency identities
or denial counts. A syntactically valid combination such as
`availability=revoked&eligibility=eligible` can return an empty permitted set;
state/policy emptiness is not an error or an existence signal. No user-supplied
authorization flag changes that policy. Build/resolve reauthorize independently.

Optional knowledge review-due predicates are exactly one of:

- `review_due_unspecified=true`, selecting the explicit null value;
- `review_due_date_on_or_before=<calendar_date>`, selecting only date-kind
  values <= that date;
- `review_due_at_or_before=<timestamp>`, selecting only instant-kind values <=
  that exact instant.

They compare like types, never coerce a date into an instant and never interpret
`now`, `today` or a natural-language schedule. A mixed predicate is invalid.
There is no combined due-now filter requiring a hidden timezone or fresh
per-page clock cutoff. The caller can deliberately choose its literal date or
instant; actual policy scheduling remains the candidate owner.

Expense dates are already exact nullable `expense_date` values. Their listing
filter does not transform `source_observed_at` or require a second report date
policy. Accept either singular `undated=true|false`, or one/both of
`expense_date_from=<date>` inclusive and `expense_date_through=<date>` inclusive;
never combine `undated` with range fields. No date fields means all known and
unknown dates. `undated=true` means null only; false means known only. Ranges
exclude null. Normalize date ranges to the existing financial half-open end
union: the day after `through`, or `after_maximum_date` at 9999-12-31. Omitted
start/end select the corresponding domain minimum/terminal boundary. Equal
inclusive endpoints select one day; from > through rejects. Do not add one day
through a host-local timezone or manufacture year 10000. This browsing scope is
not financial-records contract's immutable report selection, inclusion/exclusion manifest or totals.

Currency omission includes null. Supplying all eight known currencies therefore
does **not** normalize to omission; it selects known currencies only. This is an
explicit exception to the general full-enum normalization rule, required to
preserve unknown financial values. Currency strings remain exact uppercase
existing values, without accepting future remote-registry entries implicitly.


## L.4a Exact normalized request/filter/key types

For every list family, its named list_request is the closed normalized observation
{context:common.command_context, path:<the exact closed selector below>,
filters:<its fully resolved closed filter object>, page_size:integer1..100,
cursor:cursor_input}. Null cursor represents an initial request with
the raw query key omitted. It is never a literal null-valued GET query parameter.
A required context value cannot be supplied implicitly by a cursor or profile.

Path selector objects are {} for captures/candidates/assets/expenses heads;
{capture_id}, {candidate_id}, {asset_id}, {expense_id} for their respective
revision/version histories; {domain_id} for channel heads; and {domain_id,name}
for channel generations. Name is context.channel_key's common.slug. Every path
domain must equal request context. Canonical resource paths are reconstructed
only from the matched family and canonical selectors in the route inventory.
No arbitrary path string or second decoded route is an authority input.

Exact normalized primitives:
created_interval = {from:timestamp|null, before:timestamp|null}.
Both null means no creation filter; otherwise keep literal endpoints, requiring
from < before when both are present.

review_due_filter =
  {kind:'any'} |
  {kind:'unspecified'} |
  {kind:'date_on_or_before', value:common.calendar_date} |
  {kind:'instant_on_or_before', value:common.timestamp}.

expense_date_filter =
  {kind:'any'} | {kind:'unknown_only'} | {kind:'known_only'} |
  {kind:'period', start_date:common.calendar_date,
   end_exclusive:financial.period_end}.
No date query means any; undated=true means unknown_only; false means known_only.
A provided date range uses explicit domain-minimum0001-01-01 when start omitted,
and after_maximum_date when through omitted. Do not collapse a date range into
any, because ranges exclude null. Existing financial date endpoint meaning is reused through the actual named
financial.period_end definition.

currency_filter = {kind:'any'} |
  {kind:'known_set', values:financial.currency[1..8]}.
Any includes unknown/null currency. Even the complete known_set is different.
Set values are unique and ASCII sorted.

Normalized family filter objects (all members required):
capture_heads_filters = {
  submission_states:('draft'|'submitted')[1..2],
  readiness_states:('incomplete'|'ready'|'failed')[1..3],
  created:created_interval
}.
candidate_heads_filters = {
  revision_states:('pending'|'accepted'|'rejected'|'contested')[1..4],
  knowledge_kinds:candidate_review.knowledge_kind[1..8],
  created:created_interval, review_due:review_due_filter
}.
asset_heads_filters = {
  availabilities:('active'|'deprecated'|'revoked'|'purged')[1..4],
  knowledge_kinds:candidate_review.knowledge_kind[1..8],
  eligibility:'eligible'|'visible',
  created:created_interval, review_due:review_due_filter
}.
expense_heads_filters = {
  revision_states:('proposed'|'confirmed'|'voided')[1..3],
  currencies:currency_filter, created:created_interval,
  expense_date:expense_date_filter
}.
channel_heads_filters = {states:('empty'|'active'|'suspended')[1..3]}.
Each history_filters = {}.

Defaults are exactly the preceding raw filter table, expanded into these values.
All selected sets are nonempty, unique and ASCII sorted. Explicit full enum input
and an omitted term normalize identically only when that omitted term means the
same full enum. The normalized representation still contains the full required
array; it does not randomly omit a property. Pending/active defaults and unknown
currency preserve their distinct meanings. Reversing an OR set is equivalent;
changing any effective member, literal temporal cutoff or field view is not.
No query-normalization step alters canonical candidate content or common
mutating-command fingerprint semantics.

Exact descending key/order types:
- capture_heads, candidate_heads, asset_heads, expense_heads:
  created_key={created_at:timestamp,aggregate_id:identifier};
  order='created_desc_id_desc_v1'. The ID is the route family's actual aggregate.
- capture_revisions, candidate_revisions, expense_revisions:
  revision_key={version:common.content_revision};
  order='content_revision_desc_v1'.
- asset_versions: asset_version_key={version_order:common.content_revision};
  order='asset_version_order_desc_v1'.
- channel_heads: channel_name_key={name:common.slug};
  order='channel_name_ascii_desc_v1'.
- channel_generations: channel_generation_key={generation:context.positive_generation};
  order='channel_generation_desc_v1'.

The cursor's family fixes both order constant and upper/after key shape.
No key type can substitute another merely because numeric values match.
Created tuples compare exact timestamps then ASCII UUIDs; revisions/generations
compare checked integers. These types are public documented token syntax for
verification, while clients still treat whole cursor tokens as opaque.

List result carries literal family plus entries of exactly that family's item
type and page with the corresponding order. Empty and final page representation
is entries[]/next_cursor:null as described below. A producer never invents an
empty key or a cursor with no actually returned after/upper key.

## L.5. Complete finite listing responses

The resource sections below freeze all ten closed item cores and exact additive response compositions.
A list is a discovery summary, not complete source content or an authority grant.
Every returned reference and metadata field requires current authorized disclosure.
No statement excerpt, hidden count or truncated qualification appears in a row.

`page` is closed `{profile:'listing_v1', view:'summary_v1', page_size,
semantics:'current_authorized_keyset', order:<family-specific constant>,
observed_at:<common timestamp>, next_cursor:<opaque string|null>}`. Do not return
total, estimated total, denied count, scanned count, exclusions, hidden IDs,
backend candidate count, raw policy details or a financial completeness flag.
The array's own visible length is sufficient; no second count is required.

Bound the complete emitted listing response to 524288 UTF-8 bytes, inside the
common 1 MiB/depth16/array256/additive32 envelope. One hundred maximum-size
entries account for 409600 bytes; the frozen known metadata/cursor must fit the
remaining capacity. Independently qualify escaped worst cases and extension
composition. Do not silently lower page size, truncate an entry or drop a
required field when encoding fails. Complete construction fails safely before
emission if required retained data or the full wire representation is unavailable.
Server wire-construction failure uses the existing unavailable/503 response,
distinct from invalid client query input. Known authorized missing required
retained details keep the existing missing-history/409 behavior.

## L.6. Watermark and advancement semantics

On the first page, in one bounded canonical currently authorized selection,
choose the greatest returned eligible key as the inclusive upper watermark W.
Return up to page_size rows in descending order under K <= W. A successful
lookahead for one additional currently authorized row determines whether a next
cursor exists. No hidden row or raw maximum over inaccessible records enters W,
the continuation key, public metadata or a cursor diagnostic.

For each continuation, reauthorize the request and all returned rows, enforce
K <= W AND K < A where A is the prior last actually returned key, and retain
the original W, issued_at and expiry. A is never a rejected/scanned-row key.
Deduplicate only impossible duplicate query joins before the LIMIT by fixing
the query; do not silently compensate for a broken uniqueness contract in the
client. Canonical selection and batched authorization must avoid N+1 queries.

An initially empty page is `entries:[]` with `next_cursor:null`. A final page is
nonempty or empty with null cursor. A continuation may become empty after
revocation/state changes; it returns null cursor only after a successful bounded
selection proves there are no further currently visible rows in that remaining
range. Database timeout, unqualified query plan or unavailable authorization is
an error, not an empty or apparently completed traversal. This does not require
unbounded SQL scanning: the actual selective/permission queries must qualify
inside the existing two-second transaction and ten-second request budgets.

The watermark is a key boundary, not a cross-request SQL snapshot or completeness
barrier. New rows above W are excluded. Rows newly committed or newly visible
below W may appear if still below A; rows becoming visible in an already passed
range are missed. Mutable eligibility/current heads may change or disappear.
Creation clock rollback can place a new aggregate below W and is disclosed by
this same rule; no commit-completeness claim follows from a recorded timestamp.
Permission/state changes may alter retry results for the same cursor. An expired
cursor requires a fresh first page and explicit new traversal, never an implicit
claim that earlier pages compose an immutable complete report.

## L.7. Authenticated opaque cursor profile

Choose a bounded stateless **HMAC-SHA-256** profile, `listing_hmac_sha256_v1`, using
a server-controlled dedicated 256-bit cursor key. This specifies an established
primitive and preimage, not a new primitive or an operational key. No existing
build-attestation signing key or client bearer token is reused. Actual selected
implementations, key source, authorized key provisioning and rotation support
must qualify before listing and exact-evidence implementation enables cursor issuance. This contract selects no operational cursor key.

Token syntax is ASCII `lc1.<kid>.<payload_b64url>.<tag_b64url>`. `kid` is a
1–32-character common-slug subset; it is a nonsecret configured selector, never
a path, URL, JWK or algorithm negotiation field. Base64url is the unpadded URL
alphabet with canonical zero pad bits; alternate spellings, padding and trailing
bytes fail. Tag is the complete 32-byte HMAC (43 base64url characters). Entire
token is at most 4096 bytes; decoded payload at most 2048 bytes. No compression,
encryption, nested token or remote key lookup occurs.

The payload is exact JCS of the following closed shape:

`{profile:'listing_hmac_sha256_v1', family, query_sha256, actor_sha256,
order, upper:<typed family key>, after:<same key type>,
issued_at:<common timestamp>, expires_at:<common timestamp>}`.

The order and key union are finite and tied to the family. Timestamps and exact
integer keys retain common encodings; no JSON binary64 integer carries a
revision. `after <= upper` is mandatory. Every key came from an already disclosed
returned row. Payloads never include credentials, email, source text, query
bodies, denied IDs/counts, backend scan boundaries or arbitrary authority claims.
Opaque means clients treat the complete token as uninterpreted, not that base64
provides confidentiality. Its nonsecret authenticated contents are deliberately
safe to decode. Do not log full request targets/cursors in unrestricted logs.

`query_sha256` is SHA-256 over:

`UTF8('arboresce.listing-query.v1\0') || JCS({family, resource, context,
filters:<fully normalized closed family object>, order,
view:'summary_v1', page_size:<resolved integer>})`.

`resource` is the matched canonical origin-relative path, with exact parent ID,
domain/name selectors where applicable; context is the exact common context.
The token itself is not included in its own fingerprint. A cursor for another
family, parent path, organization, domain, purpose, normalized filter, summary
view, order or page size is invalid. Explicit default and omitted default values
normalize identically; different non-default values do not.

listing_actor = {organization_id:common.identifier,
executor:context.principal_reference, delegation_id:common.identifier|null}.
actor_sha256 = SHA256(UTF8('arboresce.listing-actor.v1\0') || JCS(listing_actor)).
These values are server-derived under the immutable exact delegation-chain and
every-page current-authorization rules in R.8. A singular invented authorizer,
email, bearer bytes, cached role or client actor is not part of this identity.
Same-principal token refresh can preserve actor identity after fresh admission;
changing the selected exact delegation chain invalidates the cursor.

HMAC preimage is exactly:

`UTF8('arboresce.listing-cursor.v1\0') || ASCII('lc1') || 0x00 || ASCII(kid)
|| 0x00 || payload_bytes`.

Each named domain separator contains one NUL byte, not printable backslash-zero.
The two explicit 0x00 delimiters in the cursor preimage are additional zero bytes.
Version/kid are authenticated. Use a vetted constant-time full-tag comparison.
Check lexical/input bounds and select only a bounded local configured key
inventory before MAC work; verify MAC before interpreting authenticated payload
fields. Then require strict duplicate-free JSON, its exact closed schema and
canonical JCS byte equality. The profile does not allow an attacker-controlled
algorithm, truncated tag, fallback unsigned mode or a timing-revealing list of
trial keys.

Choose fixed **900-second TTL**, starting at the first successful page's server
observation. `expires_at = issued_at + 900 seconds` exactly; issuance must fail
before common timestamp overflow. Continuations retain these times and never
renew them. Enforce issued_at <= now < expires_at using an injected qualified
server clock; clock rollback/future issuance fails closed rather than extending
validity. Cursor key inventory contains at most four accepted selectors and one
issuer; retiring a key can invalidate remaining cursors. Normal planned rotation
retains a verifying key until its last possible 900-second token expires; emergency
retirement may require a fresh traversal. Key lifecycle is an explicit operator
profile/gate, separate from user authentication or build-signature trust.

A provided token with malformed components, unknown key/profile, bad tag,
expired/future time or any binding mismatch yields the same bounded
`invalid_command`/422 cursor rejection
without revealing which protected binding failed. Applicable authentication and
action failures retain 401/403; unknown/undisclosable parent uses 404; authorized
missing required history uses 409; unavailable required infrastructure/policy
uses 503. Internal logs may record a bounded reason code under controlled policy,
never the protected values or counts. Normal success with zero visible rows is
not an authentication bypass and cannot distinguish hidden versus absent rows.

## L.8. Required independent verification

Freeze literal normalized query objects/fingerprints and cursor payload/preimage
bytes from the adopted prose before schema implementation comparison. Include
all ten families and seven exact routes, every filter/default/OR alternative,
duplicate decoded keys/values, percent aliases/double decoding, null/empty values,
full-enum normalization with the currency-null exception, all-date maximum,
equal inclusive dates, reverse dates, and date/instant review-due discrimination.

Use deterministic chosen clocks and synthetic non-operational test keys for
actual primitive vectors when that qualification runs. Required outcomes include
payload/kid/version/tag substitutions; canonical base64url/JCS; unknown key;
TTL 899.999/900.000 seconds; future/rollback clock; unchanged-expiry continuation;
wrong actor/delegation/parent/context/filter/page size/order; 2048/2049 payload
and 4096/4097 token bytes; and old key retention/removal. These are orthogonal
parser bounds: do not label a padded/noncanonical payload as a valid positive
merely to reach a defensive cap. Independently derive the largest reachable
canonical payload from its closed field bounds. Synthetic vectors do
not create production key authority or qualify a server merely by schema shape.

Actual database/runtime tests must cover equal timestamps/different IDs, revisions
2/10 and values above binary64 precision, unique asset ordinal versus different
candidate lineages, same-time channel events ordered by generation, W/A boundary
exclusion, late commit, state changes/current-head replacement, new visibility,
revocation, empty continuation, retry drift, limited authorized lookahead,
inaccessible scan keys, bounded authorization batches and query failure. Assert
no total/denied count or protected cursor boundary leaks through any response or
error. Financial report compilation must independently preserve its fixed
selection/fences rather than consuming these pages.

Schema fixtures only qualify parsed values. The actual HMAC, normalization,
raw HTTP parsing, SQL keysets, authorization races, wire-byte/deadline limits,
key lifecycle and consumers remain mandatory executable evidence, with the
existing strict >90% applicable Rust/Python source/profile/file/package/repo
coverage and complete independently reviewed deterministic suites. Runtime qualification remains required before capability support.


## L.8a. Cursor-vector data contract

This is the value-free recording contract for
`contracts/validation/selection-v1.cursor-vectors.json`, validated separately by
`contracts/selection-v1.cursor-vectors.schema.json`. It fixes the shared data
representation before independent schema and literal authors begin. It is not
a cursor request or response. The [independent fixture rules](testing-and-coverage.md#independent-deterministic-fixtures)
apply; the existing [signing-vector contract](context-contract.md#signing-vector-data-contract)
remains unchanged and is not an interchangeable cursor format.

Every vector-recording object named or written inline in this section is closed.
Every listed vector metadata member is required; its only nullable position is
`cursor_vector.mutation`.
Imported query/actor values retain their own explicit nullable fields. There
are no extra metadata fields, extension bags, omitted defaults or implicit
component lookups. All byte strings are lower-case hexadecimal, exactly two
ASCII characters per byte, with no prefix, separators, whitespace or odd nibble.
`hex[0..N bytes]` includes the empty string and has at most twice N characters;
`hex[N bytes]` has exactly twice N characters. Recording malformed protocol
bytes as valid hex does not make those protocol bytes valid.

A vector ID is a 1–128-character ASCII name beginning with a lower-case letter,
followed only by lower-case letters, decimal digits, underscore or hyphen.
Names identify fixtures, never principals, keys or operations. The document
contains 1–4096 vectors with unique IDs in its independently frozen order. These
are finite fixture-artifact bounds, separate from common HTTP collection and
byte caps. The final literal freeze fixes the exact count, IDs and metadata;
no count or case outcome is selected here.

| Named component | Complete required members and types |
| --- | --- |
| `cursor_vectors_document` | `schema_version`: integer with constant 1; `protocol`: string with constant `listing_hmac_sha256_v1`; `classification`: string with constant `synthetic-non-operational`; `vectors`: array of `cursor_vector` with the bounds above. |
| `cursor_vector` | `id`: vector ID; `mutation`: null or the closed mutation descriptor below; `key`: `cursor_vector_key`; `query`: `cursor_vector_query`; `actor`: `cursor_vector_actor`; `payload_bytes`: `cursor_vector_payload_bytes`; `preimage_bytes`: `cursor_vector_preimage_bytes`; `tag_bytes`: `cursor_vector_tag_bytes`; `token`: `cursor_vector_token`; `verification`: the closed verification input below; `expectations`: `cursor_vector_expectations`. |
| `cursor_vector_key` | `classification`: string with constant `synthetic-non-operational`; `kid`: `cursor_key_id`; `key_hex`: hex[32 bytes]. Each key is explicitly synthetic fixture material, never an authorized issuer key. |
| `cursor_vector_query` | `target`: one of the ten exact `public.listing_query_fingerprint_input_<family>` names; `value`: the corresponding complete closed normalized query value from L.4a/L.7; `jcs_hex`: hex[1..8192 bytes]; `query_sha256`: `common.sha256`. The target suffix fixes the value's family, canonical resource-path grammar, filters and order. |
| `cursor_vector_actor` | `value`: complete `listing_actor` from R.8; `jcs_hex`: hex[1..2048 bytes]; `actor_sha256`: `common.sha256`. The value is the supplied synthetic current-verifier actor observation, not client authority. |
| `cursor_vector_payload_bytes` | `hex`: hex[0..2049 bytes], recording exact payload bytes, including malformed/oversized negative input. Successful protocol payloads still require L.7's exact closed family shape, canonical JCS and at most 2048 bytes. |
| `cursor_vector_preimage_bytes` | `hex`: hex[0..4096 bytes], recording the exact HMAC message input, including declared domain-separator/delimiter mutations. The canonical construction remains L.7's exact preimage. |
| `cursor_vector_tag_bytes` | `expected_hex`: hex[32 bytes], the independently frozen full comparison tag; `candidate_hex`: hex[0..33 bytes], the exact tag submitted to primitive verification, including empty/truncated/overlong controls. An admitted protocol tag remains exactly 32 bytes. |
| `cursor_vector_token` | `hex`: hex[0..4097 bytes], recording the exact token input bytes. Positive protocol tokens are ASCII; hex preserves invalid UTF-8, non-ASCII, missing components and one-byte-over-cap negative input without JSON string repair. The protocol cap remains 4096 bytes. |
| `cursor_vector_expectations` | `structural`: `cursor_vector_structural_expectation`; `canonical`: `cursor_vector_canonical_expectation`; `primitive`: `cursor_vector_primitive_expectation`; `semantic`: `cursor_vector_semantic_expectation`. |

The ten query targets are exactly the family bindings in L.2/L.4a: capture
heads/revisions, candidate heads/revisions, asset heads/versions, expense
heads/revisions and channel heads/generations. The `public.` prefix identifies
this library's existing named-target convention; it is not an arbitrary URI,
remote lookup or namespace fallback. Malformed query/actor object shapes belong
to their ordinary parsed-value cases. Cursor vectors keep these independently
well-shaped current query/actor observations so a changed effective binding
can be tested without also changing their structural interpretation. Their
recorded JCS bytes and hashes may still deliberately disagree in a declared
negative experiment; their exact agreement is a separate expectation.

The closed `verification` input has exactly these required fields:

- `now:common.timestamp`, the injected instant for this isolated observation;
- `clock_health`, one of `healthy`, `unhealthy`, `unknown`;
- `key_inventory`, one of `available`, `unavailable`;
- `accepted_keys`, an array of 0–4 `cursor_vector_key` objects with unique `kid`
  values; `unavailable` requires the empty array, while `available` may be empty
  to record retirement of all selectors;
- `authority`, one of `allowed`, `unauthenticated`, `forbidden`, `not_found`,
  `unavailable`, recording the synthetic current admission observation.

`key` is the exact key for the isolated primitive computation/comparison.
`verification.accepted_keys` is the complete synthetic configured verifier
inventory for that case; selecting only by token `kid` never falls back to
`key`. This distinction records wrong-key, unknown-selector and retirement
experiments without introducing a remote lookup. Every listed key is
non-operational. No issuer-key creation or actual authority/time service is
represented by these observations. Cross-instance/restart/rollback histories
still require their real runtime fault suites; a declared healthy clock is not
proof that a clock guard works.

`query` and `actor` describe the current verification binding. Recompute their
fingerprints using L.7's separate domain separators and compare them to the
authenticated payload only at the appropriate verifier stage. Payload and token
bytes are recorded separately so a noncanonical or undecodable token is not
silently repaired into a normalized token. The complete expected tag remains
available even when the submitted candidate tag is truncated. A consumer must
not use `expected_hex` as the token's submitted tag or replace declared bytes
with a computed value before comparison.

### One-component mutations and associations

A null `mutation` identifies an independently fixed base vector; it does not
itself declare acceptance. A nonnull mutation is closed and contains exactly
`base_id`, `component`, `path` and `derived_components`:

- `base_id` is the ID of an earlier vector whose mutation is null. It is never
  a filename, external reference or another mutation, so no chain or cycle exists.
- `component` is exactly one of `key`, `query`, `actor`, `payload_bytes`,
  `preimage_bytes`, `tag_bytes`, `token`, `verification`.
- `path` is an RFC 6901 JSON Pointer relative to that component, either empty
  for the complete component or beginning with `/`. It has at most 256 ASCII
  characters; `~` occurs only as `~0` or `~1`. Every token must resolve an existing
  member or canonical nonnegative array index. The author fixes exactly one
  changed value at this pointer, without an implicit second input mutation.
- `derived_components` is a unique ASCII-sorted array of 0–7 names from that
  same component vocabulary, excluding `component`. It lists only companions
  whose deterministic bytes must change to maintain the declared construction
  after that one input mutation. Each derivation follows the formulas in L.7
  and this section; no unrelated query, actor, clock or authority change is a
  derived update. An empty list retains every other input component exactly.

The comparison excludes the case ID, mutation descriptor and independently
chosen expectations. All other input fields outside the chosen pointer and
listed derivative components remain identical to the base. A derivative may
update only the fields implied by the changed input; it cannot rewrite the
other values in that component. Thus changing an authenticated payload field
may require recomputing its preimage/tag/token companions, while a tag-only
negative retains its complete original message/key. The author must name the
single cause and explicit derived companions before outputs are compared.
Comparison must preserve deliberately inconsistent bytes in negative vectors;
it must never repair a vector or infer its expected result from implementation.

The ordinary query/actor values, payload, preimage, tag and encoded token have
these exact associations when the corresponding expectation declares agreement:
query/actor JCS records encode their complete `value`; their hashes include the
proper one-NUL domain separators; the payload contains those query/actor hashes;
token payload bytes equal `payload_bytes.hex`; the cursor preimage binds the
fixed version, exact token selector and payload bytes under L.7; the token's
submitted tag bytes equal `tag_bytes.candidate_hex`. Query/resource/family/order
and upper/after correlations remain the exact L.4a/L.7 constraints. A declared
negative may violate its named association; the recording structure must not
exclude the experiment merely because the product contract rejects it.

### Four independent expectation vocabularies

The following are required closed expectation components. They define permitted
disposition words only, without assigning any word or Boolean to a case. Every
component is present even when an earlier layer cannot be evaluated. These are
independently authored oracles, not run receipts or a declaration that a gate
passed. The vector-structure schema freezes metadata and encodings; it neither
executes these layers nor derives one layer's answer from another's.

| Named expectation | Exact required fields and disposition vocabulary |
| --- | --- |
| `cursor_vector_structural_expectation` | `cursor_input`, `cursor_token`, `cursor_payload`: each one of `valid`, `invalid`. `cursor_input` checks strict UTF-8 plus the normalized visible-ASCII/nonempty/4096-byte shape against token bytes; `cursor_token` checks the strict complete token target; `cursor_payload` independently parses the recorded payload bytes as strict duplicate-free JSON and checks the family-bound payload target. An undecodable value is invalid at that boundary, not missing metadata. |
| `cursor_vector_canonical_expectation` | `query_jcs`, `query_fingerprint`, `actor_jcs`, `actor_fingerprint`, `payload_jcs`, `cursor_preimage`, `token_payload`, `token_tag`, `token_encoding`: each one of `matches`, `differs`, `not_evaluable`. Each field checks its namesake exact association above. `token_encoding` checks strict unpadded base64url with zero pad bits and canonical component spelling; malformed but inspectable encoding differs. |
| `cursor_vector_primitive_expectation` | `tag_computation`: one of `matches`, `differs`, comparing the full HMAC over recorded `preimage_bytes` under recorded `key` with `expected_hex`; `candidate_verification`: one of `accept`, `reject`, comparing that complete computed tag with `candidate_hex`. A candidate with other than 32 bytes is rejected without a truncated comparison or unsafe native call. |
| `cursor_vector_semantic_expectation` | `binding`: one of `matches`, `differs`, `not_evaluable`; `time`: one of `within_interval`, `outside_interval`, `unavailable`, `not_evaluable`; `authority`: one of `allowed`, `unauthenticated`, `forbidden`, `not_found`, `unavailable`; `disposition`: one of `usable`, `cursor_invalid`, `raw_request_invalid`, `unauthenticated`, `forbidden`, `not_found`, `unavailable`; `qualification`: string with constant `runtime_required`. |

Canonical `not_evaluable` is reserved for an association whose necessary decode
or parse cannot produce operands; a well-formed unequal byte sequence is
`differs`. Structural evaluation of recorded payload bytes is independent of
whether a runtime verifier would reach payload parsing. Primitive expectations
always concern the recorded message/key/tag bytes and never depend on token
parser acceptance. Missing/unsupported implementations, malformed tool output,
timeout or invocation failures remain infrastructure errors outside every
acceptance vocabulary; none may be recast as an expected rejection.

Semantic binding/time fields are `not_evaluable` when authenticated usable
payload fields are unavailable at their required stage. Unhealthy/unknown
clock confidence gives the time observation `unavailable`; a healthy instant
still obeys the exact fixed lifetime, representable arithmetic and zero-skew
interval. The final disposition uses L.9/L.10's existing raw-request, uniform
cursor, infrastructure and current-authentication/action distinctions. It is
an oracle for the actual verifier, not a successful runtime check. In
particular, `usable` describes cursor acceptance only, never permission to
skip page selection, current row authorization or the final page fence.

Ordinary parsed selection cases reuse the established public-family metadata
shape: a closed document with required `schema_version` (integer constant 1),
`validation_scope` (constant `parsed-json-values`) and `cases`; each closed case
has required `id` (nonempty string), `target` (finite target-name string), `valid`
(strict Boolean) and `value`. The `cases` array is nonempty, with its exact ordered
inventory fixed by the independent literal freeze. `value` may be any parsed
JSON value, including null or a malformed target shape; vector metadata closure
and null rules do not constrain this ordinary case input. Target names
are `public.<selection-library-target>`. The always-reject library root is
checked separately from named-definition cases. Exact case IDs, order, cardinality, metadata and
Boolean outcomes are fixed by the later independent literal freeze, then
preserved in the expectations schema. Only `value` is evaluated against that
target or its negation. The family entry refers exactly to
`validation/selection-v1.expectations.schema.json`. No schema, literal case,
expected outcome, key, canonical byte sequence, tag or token is supplied here.

## L.9 Cursor error and clock qualification details

The initial request omits cursor and is valid; continuation requires a nonempty
decoded cursor. The raw query decoder rejects invalid UTF-8/percent escapes,
control bytes, duplicate singular keys and empty query values as400 invalid_request.
A present cursor exceeding4096 bytes or having invalid lc1 components is422
invalid_command; it does not enter token decoding/MAC/JSON allocation first.
Unsupported/unknown supplied key selector, bad tag, noncanonical base64/JCS,
expired/future token and changed binding use that same safe422 disposition.
Do not conflate raw-query failure, malformed token, false MAC and infrastructure.

The named cursor_input definition accepts null for an omitted initial cursor
or a nonempty visible-ASCII string up to 4096 bytes. Normalized requests use
that same nullable definition. The separately named cursor_token
and cursor_payload definitions express strict token/parsed payload shape. A
structural fixture does not authenticate the token. Preserve malformed cursor
negative cases and exact future HTTP outcomes in their separate semantic inventory.
The fixed cursor_invalid_problem is common.problem specialized to
code invalid_command/status422/type urn:arboresce:problem:v1:invalid_command,
title 'Invalid listing cursor', detail 'The listing cursor cannot be used.',
reason 'cursor_invalid', plus the common required safe request association.

Missing required configured issuing/verification key infrastructure or unqualified
authoritative time yields503 unavailable. Unknown attacker-supplied kid is the
uniform422, not a remote key discovery request. First-page success that needs
a next cursor cannot emit unsigned/placeholder/no-key tokens or pretend the
page was final. An actually final/empty page needs no token but still satisfies
current-authority and time/deadline checks.

The operational clock profile must detect/regulate authoritative UTC regression
and loss of time confidence across all issuing/verifying instances and restarts.
Use zero positive skew allowance: issued_at <= now < expires_at. The guard is
a real qualification requirement, not implemented by comparing now only with
this token's issued_at: that check alone misses rollback within a token interval
or resurrection after an unobserved expiry. On unhealthy/unknown time, deny
issuance and acceptance; do not clamp an untrusted clock and claim qualification.
listing and exact-evidence implementation must select and test the actual shared authoritative-clock/health mapping
before activation, including restart, process divergence, forward jump, rollback,
expired-token resurrection and guard recovery. No host/client clock is authority.

Fixed TTL is exactly900000 milliseconds, checked for representable common UTC
arithmetic. No page renews it. Each newly selected operational key uses its own
selector; do not rebind a selector to different bytes while any issued token
could remain valid. At most four accepted selectors/one issuer are configured;
normal retention and emergency retirement follow the stated original TTL.
Key/time profile qualification and any needed operational key creation remain
their real recorded gates; this contract creates neither.

### L.10 Exact list/exact-read error mapping

Unsupported raw query/path/body shape, unknown filter, repeated/empty values,
invalid primitive encoding or reversed/equal creation interval uses400
invalid_request. A syntactically valid but domain-invalid date combination,
inadmissible known profile or authenticated cursor rejection uses422
invalid_command; resource_details_unavailable uses the fixed409 in R.7.
Request-target byte/pair limits are checked before allocation; target>8192 bytes
or query pairs>32 returns400 invalid_request with bounded safe detail.
Authentication401, disclosable forbidden403, foreign/unknown/undisclosable404,
shared admission429, required dependency/time/key/policy503 and unexpected500
retain their common meanings. No error names a hidden count, record, predicate,
scope, backend address or field value.

Every named future endpoint suite must distinguish malformed raw query, valid
parsed request, authentic cursor, current authority, successful bounded SQL and
complete encoded response. No schema Boolean is promoted into those results.

## L.11 Whole-page admission and authority races

Prepare the bounded page and complete required row metadata with one qualified
canonical selection/lookahead, then preflight the full wire response. Final
admission checks the actual scope/principal/delegation/policy and selected row
revision/control/visibility observations under the shared short canonical fence.
A changed required observation fails409 conflict safely; this profile does not
loop/requery until a stable page fills. The caller may deliberately retry its
same list cursor under the original fixed cursor expiry and fresh authority.

The chosen upper/after keys must still name the rows actually emitted. Do not
drop a newly forbidden row after signing its cursor, replace after with a hidden
scan key or claim an incomplete page is final. Current-state movement AFTER
admission remains the explicitly documented moving-set behavior; it does not
invalidate truthful past observations or grant future read authority.

For a first page requiring a cursor, issued_at is the injected-clock observation
used by its successful final admission, not a guessed future commit time.
Preflight variable-size row content outside database locks, then finalize only
bounded timestamp/cursor/framing fields and run the final fence inside the
qualified two-second transaction. No model/provider call or client stream occurs
under locks. A failed final fence emits no page/cursor. Subsequent pages retain
the original token expiry without renewal. All preparation plus admission and
delivery uses the original ten-second request deadline.


## R.1. Existing public definitions and ownership

The libraries below retain their original names, closure, semantics and ownership. The selection library is `contracts/public/v1/selection.schema.json`; its references resolve to these existing public siblings. Their complete source definitions, not inferred names, govern every import.

| Library | Reviewed existing definitions | Owning prose |
| --- | --- | --- |
| [common.schema.json](../../contracts/public/v1/common.schema.json) | identifier, timestamp, sha256, slug, command_context, content_revision, control_revision, success_response, response_object, result_object, problem, calendar_date, generation | [Common contract](common-contract.md) |
| [acquisition.schema.json](../../contracts/public/v1/acquisition.schema.json) | capture_revision, capture_resource | [Acquisition contract](acquisition-contract.md) |
| [observation.schema.json](../../contracts/public/v1/observation.schema.json) | capture_reference, observation_reference, observation | [Observation contract](observation-contract.md) |
| [candidate-review.schema.json](../../contracts/public/v1/candidate-review.schema.json) | candidate_reference, candidate_content, candidate_revision, candidate_resource, knowledge_kind, review_record, accepted_asset_reference | [Candidate review](candidate-review-contract.md) |
| [financial-records.schema.json](../../contracts/public/v1/financial-records.schema.json) | expense_reference, financial_revision, expense_snapshot, expense_resource_response, currency, period_end | [Financial records](financial-records.md) |
| [context.schema.json](../../contracts/public/v1/context.schema.json) | principal_reference, execution_attribution, policy_reference, channel_key, channel_core, channel_event, positive_generation, build_reference, observation_core, profile_reference | [Context contract](context-contract.md) |
| [lifecycle.schema.json](../../contracts/public/v1/lifecycle.schema.json) | lifecycle_subject, lifecycle_control, exposure_reference | [Lifecycle contract](lifecycle-contract.md) |

The table is the complete reviewed existing-definition inventory. Direct schema
references follow the concrete named selection types; a listed name may instead
be reached transitively or remain the owner of an unchanged capture/expense
companion read. Qualification records those actual relationships separately.
The inventory does not require an unused alias or reference, and it does not
change any existing definition or add a selection target.

`candidate_revision` and `review_record` are additive resources; their content and references are closed. The complete expense value is `expense_snapshot`; `expense_resource_response` is an HTTP response and cannot be nested as that value. `observation` is the complete observation union. A context `asset_snapshot` is a build-input projection and is not the complete accepted-asset GET resource. Selection does not add an internal-library import or change an existing imported definition.

## R.2. Notation and compatibility

Every field shown in a closed object below is required. Null is permitted only
where explicitly written. Every closed object uses `additionalProperties:false`.
Existing references retain their exact original closure, enums and ranges.
No old definition is edited or reinterpreted to make a new read work.

`Additive32(required fields)` means an outward resource/result composed with
the common response reader policy, at most 32 total properties and bounded
common extension values. Required known fields have their full declared types;
extensions cannot substitute arbitrary JSON for them. Additive does not mean
an extra `extensions` bag or a second canonical-content vocabulary.

The selection-local resource reservation set is exactly:
`state,status,code,problem,error,failure,failure_code,success,accepted,approved,
human_approved,authorized,confidence,ready,result,operation,operation_id,
actor,authority,reviewer,executor,metadata,body,command,idempotency_receipt,
receipt,signature,attestation,usage,usage_id,selection_id,record_sha256,
supersession,supersessions,introduced_by,superseded_by,supersession_id,
supersession_sha256,replacement,predecessor,successor,approver,
approval_event_id,approved_at`.

The selection-local result/envelope reservation set is that complete resource
set plus exactly `capture,candidate,asset,expense,observation,event,document,
core,reference,content,control,control_revision,availability,revision_state,
verdict,head,entries,page,family`. These are direct property-name reservations,
not recursive bans on imported content. A reserved property is forbidden for
every value, including null or a Boolean. Existing nested additive resources
retain their original owners' reservations.

### Direct helper acceptance and concrete specialization

The three selection-local helper names have fixed direct acceptance contracts.
They introduce no parameters inferred from a fixture, resource name or enclosing
object, and do not edit the existing common library:

| Helper target | Exact foundation and required fields | Exact direct reservations |
| --- | --- | --- |
| `selection_resource_object` | `common.response_object`; no required fields. It accepts an empty object and bounded additive members under that foundation. | Every name in the resource reservation set above is forbidden. No required-field exception applies. |
| `selection_result_object` | Both `common.result_object` and `common.response_object`; no required fields. The latter imposes the 32-property reader cap. | Every name in the result/envelope reservation set above is forbidden. No required-field exception applies. |
| `selection_success_response` | `common.success_response`; exactly the required known fields `state` with its succeeded constant, `request_id:common.identifier`, and `result:selection_result_object`. All three are nonnull. | Every name in the result/envelope reservation set except the required `state` and `result` is forbidden at the envelope. Its nested result keeps all direct `selection_result_object` reservations. |

The inherited reader bounds include at most 32 total properties, the common
1–128-character object-key rule, and common bounded extension values. These
helpers establish transport-container acceptance only: an empty direct helper
result is not a complete endpoint result, and a generic envelope does not
validate any endpoint's required resource. Direct helper cases cover empty or
required-envelope shapes, each reserved name, bounded harmless extensions,
wrong known-field types and the common boundary controls. There is no
`legitimate-required-field` exception case for a direct resource/result helper.

Concrete `Additive32(required fields)` resources use `common.response_object`
and their complete required typed field table. Their direct reservations are
the resource set minus exactly those required field names, together with any
additional direct exclusions expressly required for that concrete type. Concrete results use
both common foundations listed for the result helper and the result/envelope
set minus exactly their own required typed names and preserve their additional
route-specific exclusions. Thus a receipt wrapper's
required `record_sha256` and a supersession resource's required
`supersession_sha256` are typed by their concrete targets; they remain forbidden
in a direct `selection_resource_object`. The same rule permits only the selected
result's required resource siblings. A known required field never becomes an
untyped extension, and an optional extension cannot claim this exception.

`Read200(result)` uses `common.success_response`, with HTTP 200 and its required
`request_id`, `state=succeeded`, and complete required concrete result. Its
outer reservations are the result/envelope set minus `state` and `result`;
the selected result has its own concrete table as just defined. The three
helper definitions are independently addressable compatibility examples, not
mandatory intersections that would prohibit a concrete required field. Build
each concrete specialization from its stated common foundation and final field
table; a prohibition in one composed schema cannot be overridden by another.
`legitimate-required-field` branch coverage belongs to these concrete targets
and to the generic envelope's three fixed required fields, not to a guessed
consumer of a generic helper.

`idempotency_receipt` remains forbidden at both envelope and result (absence is
required, never a wire Boolean). No command, operation or approval alias can
enter through an extension. A failed complete read is the common problem
response, never a succeeded resource with missing required fields. Actual
serialized bytes retain the common 1 MiB, depth16 and aggregate collection
bounds; shape validity is not a claim that every maximal combination fits.

## R.3. All ten exact list item cores

Each item is `Additive32({core:<its closed family core>})`. No item is called a
complete candidate, capture, asset, expense or observation. Required reference
IDs themselves require current disclosure. The complete encoded item, including
any permitted additive fields, is at most 4096 UTF-8 bytes.

```text
capture_head_core = {
  reference:observation.capture_reference,
  control_revision:common.control_revision,
  submission_state:draft|submitted,
  readiness_state:incomplete|ready|failed,
  created_at:common.timestamp
}
capture_revision_core = { reference:observation.capture_reference }

candidate_head_core = {
  reference:candidate_review.candidate_reference,
  control_revision:common.control_revision,
  revision_state:pending|accepted|rejected|contested,
  knowledge_kind:candidate_review.knowledge_kind,
  recorded_at:common.timestamp,
  created_at:common.timestamp
}
candidate_revision_core = {
  reference:candidate_review.candidate_reference,
  control_revision:common.control_revision,
  revision_state:pending|accepted|rejected|contested|superseded,
  knowledge_kind:candidate_review.knowledge_kind,
  recorded_at:common.timestamp
}

asset_head_core = {
  reference:candidate_review.accepted_asset_reference,
  version_order:common.content_revision,
  asset_control_revision:common.control_revision,
  control_revision:common.control_revision,
  availability:active|deprecated|revoked|purged,
  supersession:asset_supersession_links,
  knowledge_kind:candidate_review.knowledge_kind,
  recorded_at:common.timestamp,
  created_at:common.timestamp
}
asset_version_core_summary = {
  reference:candidate_review.accepted_asset_reference,
  version_order:common.content_revision,
  asset_control_revision:common.control_revision,
  control_revision:common.control_revision,
  availability:active|deprecated|revoked|purged,
  supersession:asset_supersession_links,
  knowledge_kind:candidate_review.knowledge_kind,
  recorded_at:common.timestamp
}

expense_head_core = {
  reference:financial.expense_reference,
  control_revision:common.control_revision,
  revision_state:proposed|confirmed|voided,
  recorded_at:common.timestamp,
  created_at:common.timestamp
}
expense_revision_core = {
  reference:financial.expense_reference,
  control_revision:common.control_revision,
  revision_state:proposed|confirmed|superseded|voided,
  recorded_at:common.timestamp
}

channel_head_core = context.channel_core
channel_generation_core = {
  channel:context.channel_key,
  generation:context.positive_generation,
  event_id:common.identifier,
  action:create|promote|suspend|rollback,
  state:empty|active|suspended,
  target:context.build_reference|null,
  recorded_at:common.timestamp
}
```

The asset summary's unqualified `control_revision` is the exact selected
accepted-version lifecycle counter. `asset_control_revision` on both asset
summary families is the aggregate head/deprecation counter specified below;
it binds the currently observed immediate relationship references as well.
Candidate and expense summary control values are their aggregate review/content
counters, never their separate exact-revision lifecycle counters. Capture head
control is the acquisition aggregate counter. These types have the same wire
tag but different owners and cannot be substituted in a command.

Copy finite local state exclusions from their source semantics, without copying
unrelated complete-resource properties into the summaries:

- Submitted capture heads cannot have control1; noninitial capture content
  cannot have aggregate control1 because revision increments both counters.
- Candidate accepted/rejected/contested/superseded summary states cannot have
  control1; noninitial candidate content cannot have aggregate control1.
  A summary does not carry the retained review, so it cannot claim the stronger
  reviewed-superseded minimum from a missing field.
- Non-proposed expense states cannot have control1; noninitial financial content
  cannot have aggregate control1. The summary does not fabricate a confirmation
  merely to impose the stronger reviewed-history branch.
- Revoked assets cannot have exact lifecycle control1; purged assets need the
  lifecycle complete-purge reachable minimum (currently control>=5). A version
  can be directly deprecated with lifecycle control1: deprecation and lifecycle
  are distinct owners. Do not reject that reachable combination.
  An asset head with version_order>1 cannot have aggregate
  asset_control_revision1. Aggregate control1 also cannot report deprecated
  availability; it can report revoked/purged after independent lifecycle work.
  Both asset summaries with aggregate control1 require null introduced_by and
  superseded_by. A current-head summary always has null superseded_by; a
  historical summary may have either exact immediate relationship reference.
  A current head with version_order>1 requires nonnull introduced_by.
  Any nonnull relationship needs aggregate control>=2; version_order1 has
  introduced_by=null. General exact joins and ordering remain semantic checks.
- Channel create is generation1/empty/null target. Promote is generation>=2,
  active/non-null target; suspend is generation>=3, suspended/non-null target;
  rollback is generation>=4, active/non-null target. Only create can be empty.
  The full event owner supplies the missing predecessor/rollback/reason fields.

All exact cross-record equality and greater-than comparisons remain semantic
checks: summary IDs/content digest/version/state/kind/time must match the
canonical selected revision and current observed aggregate. No join to a
different current revision may supply a historical row's knowledge kind or
recorded time. A history row does not promise it is superseded; the current
revision is also a retained member of the history.

## R.4. All ten exact result/response compositions

Define one item and one result/response specialization per family. The closed
page requires the family-selected constant order; an arbitrary order enum at
the outer union is insufficient to reject a cross-family substitution.

```text
listing_page_<family> = {
  profile:'listing_v1', view:'summary_v1',
  page_size:integer 1..100,
  semantics:'current_authorized_keyset',
  order:<the exact constant below>,
  observed_at:common.timestamp,
  next_cursor:cursor_token|null
}
listing_result_<family> = Additive32({
  family:<the exact family constant>,
  entries:<the family item>[0..100],
  page:listing_page_<family>
})
listing_response_<family> = Read200(listing_result_<family>)
```

| Family | Route | Item core | Exact `order` value |
| --- | --- | --- | --- |
| capture_heads | GET /v1/captures | capture_head_core | created_desc_id_desc_v1 |
| capture_revisions | GET /v1/captures/{capture_id}/revisions | capture_revision_core | content_revision_desc_v1 |
| candidate_heads | GET /v1/candidates | candidate_head_core | created_desc_id_desc_v1 |
| candidate_revisions | GET /v1/candidates/{candidate_id}/revisions | candidate_revision_core | content_revision_desc_v1 |
| asset_heads | GET /v1/assets | asset_head_core | created_desc_id_desc_v1 |
| asset_versions | GET /v1/assets/{asset_id}/versions | asset_version_core_summary | asset_version_order_desc_v1 |
| expense_heads | GET /v1/expenses | expense_head_core | created_desc_id_desc_v1 |
| expense_revisions | GET /v1/expenses/{expense_id}/revisions | expense_revision_core | content_revision_desc_v1 |
| channel_heads | GET /v1/domains/{domain_id}/channels | context.channel_core | channel_name_ascii_desc_v1 |
| channel_generations | GET /v1/domains/{domain_id}/channels/{name}/generations | channel_generation_core | channel_generation_desc_v1 |

The general response union is exactly these ten specializations. Entries cannot
be a union accepting another family's item. Runtime checks require length <=
page_size, exact unique keys, strictly descending canonical order and cursor
after/upper correlation. The schema can reject exact duplicate item values;
same identity/different metadata still needs semantic rejection. If the selected
implementation uses `uniqueItems:true`, it must not claim that proves identity
uniqueness or fixes a duplicate-producing query.

The complete list response cap is 524288 UTF-8 bytes. No count, clipped prose,
partial result or complete-resource assertion is added. The listing sections above own exact
normalized filters, request parsing, actor/query/cursor
bindings, authorized keysets, current observations and continuation semantics.
The order constants above are the same exact literals selected in L.4a and
must agree in every request/response/cursor specialization.

## R.5. Complete accepted-asset resources

### Immutable accepted content and review

Use a local accepted-review specialization that consumes candidate_review.review_record,
requires its accepted branch, and closes its exact known accepted fields:

```text
accepted_review_core = {
  review_id:common.identifier,
  candidate:candidate_review.candidate_reference,
  control_revision:common.control_revision,   # existing decision minimum >=2
  verdict:'accepted',
  reason:string 1..8192,
  recorded_at:common.timestamp,
  audit_event_id:common.identifier,
  approval_event_id:common.identifier,
  asset:candidate_review.accepted_asset_reference
}
asset_version_core = {
  reference:candidate_review.accepted_asset_reference,
  content:candidate_review.candidate_content,
  review:accepted_review_core,
  version_order:common.content_revision,
  recorded_at:common.timestamp
}
asset_version_resource = Additive32({
  core:asset_version_core,
  asset_control_revision:common.control_revision,
  supersession:asset_supersession_links,
  control:<lifecycle.lifecycle_control specialized to its exact accepted-asset-version
           subject, restriction=unrestricted and bytes_state=available>,
  availability:active|deprecated
})
```

This is a local closed canonical projection of all required candidate-review contract accepted-review
facts, not a mutation of candidate_review.review_record or a consumer that silently drops a
required field. Transport-only review extensions are not canonical approval
facts. Candidate reference/content/review identities and all current trust
checks remain unchanged. There is no new asset-content digest or JCS profile.
`reference.candidate.content_sha256` retains the candidate-review contract canonical preimage of
the complete normalized content and original candidate context. The whole new
resource, discovery metadata, review, relationship observations, aggregate and
lifecycle controls and availability do not
enter that candidate content hash. Immutable `recorded_at` is the acceptance/
mapping transaction's injected-clock observation and equals review.recorded_at.

Required canonical joins are:

- core.reference.candidate equals core.review.candidate exactly, including the
  stored purpose; reference.review_id equals review.review_id; review.asset
  equals core.reference completely; organization and canonical accepted domain
  agree throughout. Review digest/control/approval/audit resolve to the exact
  original accepted decision, never a later review or a client assertion.
- The normalized complete content recomputes the exact reference content hash.
  Supporting/refuting/qualifying roles, scope, applicability, assumptions,
  qualifications, invalidation conditions and review_due are all present.
  The asset read does not inline an unbounded dependency graph. Existing exact
  observation reads provide complete direct observations when requested; the
  server still checks the full required source-disclosure closure before read.
- The review is retained accepted history even if the candidate revision has
  since become superseded. An asset read must not insert the current candidate
  resource's `revision_state` or relabel the retained accepted review.
- `control.subject` is the lifecycle.lifecycle_subject variant
  kind=accepted_asset_version with original organization,
  original accepted candidate domain, asset ID and exact version ID. It is
  never rebuilt using a different read/search purpose or consumer domain.
  Read context is its own current authorization input, not a replacement for
  the asset's stored candidate context or lifecycle subject.

### Current availability and two distinct controls

The exact version's lifecycle control is the sole material/restriction/hold/
purge counter. A separate aggregate head control tracks explicit asset head and
deprecation changes. This separation is necessary because lifecycle events
do not have a deprecate or asset-supersede action. Do not increment its counter
without a corresponding valid lifecycle event or mirror lifecycle changes
through a second availability counter.

`availability` is a current direct asset classification, with this precedence:

1. Exact control.bytes_state=purged => purged.
2. Otherwise exact control.restriction=revoked => revoked.
3. Otherwise a canonical explicit deprecation record => deprecated.
4. Otherwise => active.

This four-state mapping governs authorized retained discovery metadata. A fresh
successful complete asset_version_resource permits only active/deprecated and
requires own restriction=unrestricted and bytes_state=available. Its inherited
lifecycle control then also has purge_state=none. Holds remain possible but
supply no read authority. The schema enforces these local success restrictions.
The deprecation predicate is a canonical record relation and needs a semantic
check. Holds and a lifecycle restriction do not grant read or reuse permission.
Own bytes can be unavailable while the classification is active/deprecated;
missing content prevents a successful complete GET and denies use. An upstream
revocation can deny use while this subject's own classification remains active.
Do not reinterpret `availability` as all-source eligibility or insert an
`eligible:true` authority flag.

The mutable head observation is closed:

```text
asset_head = {
  organization_id:common.identifier,
  domain_id:common.identifier,
  asset_id:common.identifier,
  control_revision:common.control_revision,
  current:candidate_review.accepted_asset_reference,
  introduced_by:asset_supersession_reference|null,
  created_at:common.timestamp
}
```

Head organization/asset/domain equal its exact current reference and stored
candidate domain. All versions of one asset have that same canonical owner
scope. The current GET returns the head and exact selected version from the same
canonical observation; head.current equals asset.core.reference. A lifecycle
restriction, candidate revision or stale projection cannot silently move it.
The exact historical GET does not include another current head. Its bounded
relationship references identify records directly associated with the selected
version, not another version's content or the unrelated current-head reference.
Each such reference requires current disclosure as specified in R.5a; callers
need separate authority to inspect the record's counterpart and attribution.
head.introduced_by equals asset.supersession.introduced_by, and the current
asset's superseded_by is null. head.control_revision equals
asset.asset_control_revision. These are one fenced canonical observation.

The same finite head constraints apply across the current GET composition:
head.control_revision1 requires asset.core.version_order1, null
head.introduced_by, null asset.supersession.introduced_by and null
asset.supersession.superseded_by. It forbids deprecated availability. The
successful full-content resource requires unrestricted/available state.
Revoked/purged classification is available only through authorized retained
summary/lifecycle metadata, not a new ordinary complete-content response. Reference/counter/control
equality across response fields is a semantic invariant even when its finite
initial-value exclusions can also be expressed by local schema conditions.

Initial candidate-review contract acceptance of a newly issued asset creates head control1 and version
order1 atomically with the exact review/mapping. Under the unchanged candidate-review contract
acceptance command there is no caller-selected asset ID or target-head field;
The candidate acceptance writer cannot infer an undeclared existing-asset mutation from this read schema.
[Reviewed asset supersession](#r5a-exact-reviewed-asset-supersession-and-bounded-inspection) completely defines the later record and read meaning.
The reviewed supersession writer must qualify its complete new command/approval/expected-head transport and
atomic producer of those exact records before enabling the writer. It may not
reattach an already committed candidate-review contract mapping to a different asset/version or
convert candidate-head revision into asset-head movement. This contract exposes no writer
route and claims no runtime implementation; the read payload is already fixed,
not an unspecified record hidden behind a generic metadata field.

Asset head/deprecation mutations increment the aggregate head counter exactly
once per admitted transaction. Exact lifecycle events increment only the
selected version's lifecycle counter; both writer families share required
current authority/material fences. Each reviewed transition atomically binds
the same aggregate increment, old/new immediate relationship links and head.
Head/read observations do not increment either counter. Retain exact old
mappings, reviewed relationships and deprecation history; overflow or
a conflicting current control fails before any effect.

### Immutable ordering metadata

Each version has its own unique, positive common content-revision-encoded
`version_order`. Allocate per asset, starting1 and advancing without reuse or
wrap under the aggregate lock/uniqueness boundary. The exact mapping from
(organization_id,asset_id,version_order) to version_id is immutable. Candidate
revision counters and UUID ordering cannot stand in for it. If a future writer
creates a version without moving the head, allocation still has a deliberate
atomic aggregate mutation and cannot race another ordinal allocation.

The order is discovery/history metadata outside the unchanged accepted reference
and content hash. It does not imply approval, head status, use eligibility or
physical commit completeness. Head created_at is the first accepted mapping's
recorded time and never changes. The asset model, persistence constraints and acceptance/supersession writers must preserve these invariants.

## R.5a. Exact reviewed asset supersession and bounded inspection

### One explicit same-asset head transition

Choose the bounded MVP relationship already needed by the reviewed supersession writer: one exact current
version of an asset is replaced by one newly reviewed immutable version of that
same asset, in its unchanged organization/domain. This is an owning data-model
refinement, not a claim that the existing candidate-review contract accept command accepts a target
asset. Foreign asset/organization/domain links, self-links, merges, forks,
cross-asset replacement, arbitrary relationship kinds and asset-head rollback
are unsupported. They require a separately reviewed owner if ever proposed.

Every successful transition has exactly one immutable supersession record.
It records the old head, new head, their exact version ordinals, the admitted
aggregate control before and after, explicit deprecation of the old version,
the separate trusted human relationship approval and actual command execution.
The old and new candidate content, accepted reviews and mapping identities remain
their own immutable facts. Candidate superseded_by, greater version_order,
similar text, feedback, an approval of a build or an earlier acceptance review
cannot create this relationship or stand in for its authorization.

This first writer creates the replacement version and its unique accepted
mapping in the same transaction as the reviewed head transition. It never
reattaches a candidate revision already mapped by candidate-review contract to another asset/version.
The replacement must be an exact current pending candidate with no accepted
mapping, with actual trusted acceptance of that precise content and a distinct
approval of this precise asset replacement. A candidate accept already committed
to a new candidate-review contract asset cannot later be relabeled as this replacement. The existing
four candidate-review contract command shapes, review identity and fingerprint rules remain unchanged.
The reviewed supersession writer adds its complete explicit acceptance-plus-supersession command and trusted
approval bindings under those owners before its first real invocation; until
then only synthetic contract/read-model fixtures can contain such future records.

Resolve the actual current executor/delegation, requested supersession action,
owning organization/domain and purpose, expected old head and aggregate control,
the exact replacement candidate revision/digest/control, its current required
source-use closure, and both trusted approvals at the final mutation fence.
Historical acceptance or a retained relationship hash grants no current action.
An old revoked/deprecated version can be replaced only when current trusted
supersession policy explicitly admits that exact old head and retained metadata;
this does not require or authorize copying its unavailable content. The new
version independently passes all source/material/scope checks. Missing current
authority or required replacement evidence fails before any new mapping/effect.

### Complete immutable record, references and mutable observations

All cores, references and link objects below are CLOSED; only the expressly
marked resource is Additive32. There is no arbitrary relationship payload, free
form scope selector, embedded approval token, endpoint content or transitive
history array. References use the actual candidate-review, common and context definitions.

```text
asset_supersession_core = {
  kind:'arboresce.asset-supersession.v1',
  contract_version:'v1',
  organization_id:common.identifier,
  domain_id:common.identifier,
  asset_id:common.identifier,
  supersession_id:common.identifier,
  context:common.command_context,
  replaced:candidate_review.accepted_asset_reference,
  replacement:candidate_review.accepted_asset_reference,
  replaced_version_order:common.content_revision,
  replacement_version_order:common.content_revision,
  previous_asset_control_revision:common.control_revision,
  asset_control_revision:common.control_revision,
  previous_deprecation:not_deprecated|deprecated,
  resulting_deprecation:'deprecated',
  reason:string 1..8192,
  approval_event_id:common.identifier,
  approver:<context.principal_reference restricted to kind=human>,
  approved_at:common.timestamp,
  policy:context.policy_reference,
  attribution:context.execution_attribution,
  recorded_at:common.timestamp,
  audit_event_id:common.identifier,
  channel_effect:'none'
}
supersession_sha256 = SHA256(JCS(asset_supersession_core))
asset_supersession_reference = {
  organization_id:common.identifier,
  domain_id:common.identifier,
  asset_id:common.identifier,
  supersession_id:common.identifier,
  supersession_sha256:common.sha256
}
asset_supersession_resource = Additive32({
  core:asset_supersession_core,
  supersession_sha256:common.sha256
})
asset_supersession_links = {
  introduced_by:asset_supersession_reference|null,
  superseded_by:asset_supersession_reference|null
}
```

The reason is at most8192 Unicode scalars and32768 decoded UTF-8 bytes. Preserve
the exact approved reason and its order/Unicode; do not summarize or normalize
it. The JCS preimage contains every field shown, including both original
accepted references, original candidate contexts, transition purpose, reason,
approval, actual policy, attribution and injected times. It excludes the
supersession_sha256 field and all transport extensions. The constant kind
separates this preimage from candidate, build, selection and receipt identities.
A matching digest verifies retained bytes against an authenticated canonical
reference; it is not a signature, current approval grant or proof of delivery.

context.execution_attribution already contains executor, delegation_id,
authority_audit_event_id and command_id. Use that exact closed context contract type;
do not invent a gateway message or lifecycle operation_id for this atomic command.
The executor may be human, agent or service under actual current delegation;
approver is the human authority recorded by the trusted relationship approval.
approved_at is that event's actual trusted recorded approval time, not an agent
assertion. recorded_at is the injected observation used by the successful
mutation transaction, never a guessed physical commit timestamp.

Required authoritative joins and transition constraints are:

1. Record, reference, route, context, policy and both accepted mappings share
   organization/domain/asset ownership. Each nested candidate keeps its original
   intended context and purpose; read purpose never rewrites those fields.
   replaced and replacement exactly equal the canonical old/new accepted
   mappings, including candidate digest/revision and review_id. At writer
   admission recompute the replacement fingerprint and verify its complete
   acceptance approval; reject a pending/rejected/contested candidate presented
   as an already accepted asset. Later metadata reads verify the retained exact
   mapping/approval bindings; they do not reconstruct purged candidate content
   merely to read a retained relationship. Missing required retained binding
   proof still prevents a complete successful metadata read.
2. replaced was the exact head at previous_asset_control_revision. The new
   version ID and supersession ID are unique; replacement_version_order is the
   next never-used ordinal allocated under the asset lock, strictly greater
   than replaced_version_order. asset_control_revision is exactly previous+1
   without overflow; the recorded head after this transition equals replacement.
   Ordinals and control values are checked integers, not timestamp order.
3. The record creates the old version's sole outgoing superseded_by and the new
   version's sole incoming introduced_by, each with its exact record digest.
   Their immutable endpoint associations have unique canonical constraints.
   An existing outgoing edge, mismatching incoming edge, self-loop, cycle,
   reversal, or relationship rewrite fails before effects. Strict increasing
   same-asset ordinals prove acyclicity without an unbounded recursive graph walk.
   Head movement always follows this exact relationship, never an ordinal alone.
4. The actual trusted relationship approval binds action, organization/domain,
   transition purpose, exact old head/control, exact replacement candidate
   fingerprint and intended scope, replacement-acceptance approval, exact reason,
   permitted executor/delegation and policy. Its approval_event_id, human identity
   and approved_at resolve to that event; the candidate acceptance approval is a
   distinct trusted decision and cannot silently satisfy the relationship grant.
   A single UI review may collect both explicit decisions, but cannot infer one.
5. The audit and authority-audit/command associations resolve the actual
   admitted command/executor, approvals, canonical transaction and outbox cause.
   Retained policy identifies the actual immutable policy used then; current
   policy is checked afresh on every read/use and is not copied over this field.
   Approval and injected-clock ordering must be valid under the qualified time
   authority; impossible/unknown attribution is not filled with placeholders.
6. previous_deprecation matches the retained old-version deprecation relation
   observed at admission. The record explicitly establishes its resulting
   deprecation and moves the head in the same one aggregate increment. It does
   not increment either endpoint's lifecycle counter, revoke/purge/restore
   bytes, clear a hold or override an upstream source restriction. The new
   version initializes its own actually qualified lifecycle subject; it does
   not copy or reset the old version's lifecycle control or restriction.

Initial order1 acceptance has both links null and head.introduced_by=null.
After A1->A2, A1.superseded_by and A2.introduced_by name the same record;
A2.superseded_by remains null. After A2->A3, the A1->A2 record and A1 links
remain unchanged, A2 retains that incoming record and gains the exact A2->A3
outgoing record, and the current head references A3 and its incoming record.
There is at most one incoming and one outgoing reference per version, regardless
of history length. A future separately specified non-head version creator may
allocate another ordinal, but cannot fabricate a supersession edge or become a
head merely because that ordinal is newer; this contract activates no such writer.

Links are current canonically observed metadata outside asset_version_core.
asset_control_revision accompanies links even on an exact historical version.
It is the same aggregate counter, not a new per-link counter. A record's own
before/after controls remain the historical values at its transition; they do
not become the current aggregate value when later heads/deprecation change.
Null means canonical absence of that immediate relationship, never hidden,
unknown, purged or failed-to-fetch details. Exact read/list fences bind current
links and aggregate control together; concurrent transition either yields one
consistent observation or the existing safe conflict, not torn combinations.

### Disclosure, lifecycle and complete-or-error inspection

Relationship identity/absence, complete relationship details and endpoint
content have distinct current disclosure requirements. Historical candidate
acceptance and relation approval are retained facts, not read credentials.
An asset summary/full read requires current disclosure of its selected version
and its associated relationship identity/absence. Evaluate that metadata action
for the selected version before consulting/returning a nullable link; denied
metadata authority cannot be distinguished by whether a hidden link exists.
This action is checked independently of the old acceptance approval. It may
authorize an opaque exact record reference without authorizing that record's
reason, approver or counterpart accepted reference. Do not embed counterparts
in summaries/head/link fields, emit hidden relationship counts, or use null as
redaction. An ineligible list row is excluded through normal current-authorized
keyset selection; a complete exact read requiring undisclosable metadata fails
with the ordinary non-disclosing404 rather than exposing existence/state.

The exact record GET first requires current record-identity disclosure, then
authorization for the complete retained record: both exact endpoint reference
identities and their candidate-context metadata, reason, human/executor,
approval/policy/audit attribution and current read purpose. Authorization of the
old version alone cannot disclose a hidden replacement reference. Missing or
undisclosable record/counterpart uses the same safe404; do not reveal which
endpoint failed, a hidden record digest or an existence-specific reason. A
known disclosable forbidden action uses403 under the ordinary policy distinction.
Required unavailable policy/authorization service is503. After complete current
disclosure, known required retained-detail loss is409 with the fixed
resource_details_unavailable reason. Corrupt joins/hashes fail closed as their
actual invariant/infrastructure error, never successful partial metadata.

This GET contains exact references and review/approval history, not complete
old/new candidate content, lifecycle snapshots or a computed eligible flag.
It may expose authorized retained relationship metadata after endpoint content
revocation/purge only when every required field remains lawfully retained and
currently disclosable. It creates no retention exception. A retained hash or
reference alone cannot reconstruct missing reason/attribution/endpoint joins.
Absent bytes do not authorize body reconstruction; complete content still uses
its own exact asset GET and current source/material checks. A denied counterpart
content read does not by itself prove its retained reference is undisclosable;
the actual independent metadata policy decides, never a blanket substitution.

Neither relationship direction transfers approval, permissions, source use,
availability, validation or signatures from one version to the other. A
superseded version may remain eligible only under explicit current policy;
the replacement being active or approved never clears an old revocation. A new
version also cannot launder revoked required evidence by changing asset head.
Deprecation remains a canonical record relation with the existing classification
precedence: old own revoked/purged classification still takes precedence.

channel_effect='none' is a historical statement that this transition did not
mutate any channel, not a claim that all channels are unaffected or currently
usable. Existing builds/channels retain their exact old references. Asset inspection and reviewed supersession clients show that consequence and preserve exact old/new IDs. Current channel status and permitted generation history retain their separate complete reads.
No automatic channel promotion, build rewrite, silent reference following,
channel-impact count or unbounded transitive read is introduced.

### Capacity, persistence and earliest proof

The complete supersession GET response has a65536-byte UTF-8 cap, including its
common envelope, resource, full core/hash and any emitted additive fields; its
depth stays within16. It carries no endpoint content, so old/new content does
not multiply this cap. The record has no history-length-dependent array.
Its reference and two-link objects have fixed bounded compositions, and the
existing complete asset-summary item remains<=4096bytes/list<=524288bytes.
The full asset GET stays<=1MiB with these links, aggregate counter and head
introduced_by. The reviewed supersession writer preflights actual JCS/core and complete record response plus
every newly created or changed required asset-read composition before writes.
Earlier candidate/asset writers reserve this maximum fixed future relationship
overhead. Existing history uses that qualified reservation, not a per-command
walk or serialization of every earlier version. The affected old/new versions,
new record and head have a constant number of bounded joins; no unavailable
old content is reconstructed to inspect or record retained metadata. Include
worst-case escaped reason, principal/control/reference encodings and extensions
actually emitted; per-field maxima never establish combined byte admissibility.

The asset model and canonical store preserve the fixed records, digests, endpoint uniqueness and aggregate controls with bounded indexed joins. All three asset exact reads and both asset lists include complete relationship disclosure and errors. Before the production writer exists, read qualification may use independently frozen synthetic canonical persistence fixtures through a bounded test-only loader. That loader is measured maintained support, isolated from production admission and never evidence of a real approved supersession. The writer remains unavailable until actual command/approval/acceptance/deprecation/head/link/audit/outbox atomicity, races and readback qualify against the same immutable record contract. Asset inspection clients support listing/current/history/exact-record reads; the later reviewed-writer journey repeats readback against its real effects. Authorized customer exports preserve the exact relationships and endpoint references, with independent end-to-end attribution verification. Earlier synthetic read evidence does not claim the later writer ran.

### Independent parsed and semantic supersession oracles

For this selection contract freeze independent named targets and positive/negative parsed cases for
the complete core, reference, resource, links, every changed asset summary/head/
resource, and exact request/result/response. Include both previous-deprecation
variants, all actual executor kinds, nullable/exact delegation, human-only
approver, initial/null links and introduced/outgoing/both-link history, missing
fields, foreign-family objects, unknown kind/effect/field values, invalid IDs,
raw revision encodings, reason scalar limits, and reserved success/approval/
inline-content/current-head aliases. Enforce finite structural exclusions:
record resulting aggregate control>=2, replacement ordinal>=2, and the initial
head/link restrictions already stated. Common raw integer and UTF-8 rules still
apply; shape validity never proves equality, hash, increments or current policy.

Freeze semantic oracles independently from implementation for:

- Exact full JCS/core bytes and digest; every consequential reference/context/
  purpose/reason/approval/executor/control/time change alters the preimage.
  A well-shaped wrong digest, substituted review, contradictory endpoint or
  caller-claimed approver remains a semantic rejection, not an invented schema
  failure. Candidate/build/selection hashes do not validate this record.
- Initial A1, real future A1->A2 and A2->A3, historical immutable record controls
  versus current aggregate control, exact incoming/outgoing endpoint joins and
  no channel mutation. Candidate revision alone, standalone deprecation and
  version ordinal allocation alone cannot create a relationship. Reject
  old-head races, 50 concurrent contenders, duplicate/reused IDs or accepted
  mappings, wrong ordinal/control, counter overflow, branches, self/cross-asset/
  foreign links, cycles and attempts to move a head backwards. Retries preserve
  actual original command recognition rather than duplicate a transition.
- Historical candidate acceptance with no relationship approval; substituted,
  revoked or wrong-scope trusted approval; a permitted service executor with a
  real separate human approver; expired delegation; changed replacement source
  permissions/material and denied cross-domain ownership. Failed fences leave
  no partial acceptance, new version, deprecation, links, head, audit or outbox.
- Caller may read A1 content/metadata but not A2's identity or relationship
  details: A1's opaque authorized record link never contains A2, and the exact
  record GET fails safely without distinguishing absent/hidden counterparts.
  Denial of selected-version relationship-identity/absence authority is tested
  with both a nonexistent and an existing link before lookup. No null-redaction,
  hidden count, hash, error/log/metric difference becomes an existence oracle.
- Retained complete metadata after an endpoint's own revocation/purge versus
  missing retained reason/approval/reference; exact record GET never returns
  endpoint content or grants use. Current policy unknown503, authorized required
  retained-detail loss409, inaccessible404, and safe known forbidden403 remain
  distinct. Supersession/deprecation does not reset lifecycle control or a hold,
  does not restore revoked bytes, and cannot make replacement evidence eligible.
- Exact GET65,536/65,537 bytes; complete asset GET1,048,576/1,048,577 bytes;
  summary4,096/4,097 and list524,288/524,289 bytes; depth16/17; maximum escaped
  reason, nested exact references and revision widths; null-to-two-link growth
  at the reserved boundary. A legal reason exceeding combined capacity is
  rejected before writer effects, not truncated on later read.
- Actual CLI asset inspection JSON and human views preserve both endpoint
  references, historical versus current controls, exact reason/approval/attribution,
  unsigned hash and no-channel-effect meaning. The client fetches only an explicitly selected exact record,
  never automatically traverses/follows a chain or silently replaces a selected
  version. The reviewed supersession client and feedback/exit verification repeat
  the journey with the real later reviewed producer. Customer-export roundtrips
  preserve complete permitted record bytes and references.

This contract's parsed fixtures and fixed literal preimages do not execute JCS,
SQL, identity services, approvals or the supersession writer. The asset model,
canonical store and asset read service must qualify their actual paths with
deterministic independent fixtures and strict applicable source coverage. The
reviewed supersession writer, its client and feedback/exit verification retain
the real writer and full-process gates. No earlier synthetic loader is promoted
into those results.

## R.6. Seven exact companion routes and complete responses

All are bodyless read-only GETs with exactly the common required query selectors
organization_id, domain_id and purpose, no list filters/page/cursor/actor fields.
Canonical route IDs, domain/name, revision and generation follow owning encodings.
Each response is complete-or-error, HTTP 200 common success, with no command
receipt. The shared 1 MiB/depth limits apply; serializers must not truncate a
required field to fit. Their exact result compositions are:

| Route | Result and required interpretation |
| --- | --- |
| /v1/candidates/{candidate_id} | `{candidate:candidate_review.candidate_resource}`; exact current head chosen under current authority |
| /v1/candidates/{candidate_id}/revisions/{version} | `{candidate:candidate_review.candidate_resource}`; exact canonical positive content revision from path, complete retained content/review/successor and observed aggregate control |
| /v1/observations/{observation_id} | `{observation:observation.observation}`; complete existing three-kind union, full statement/source/grounding with current dependency disclosure |
| /v1/assets/{asset_id} | `{asset:asset_version_resource,head:asset_head}`; same canonical observation, exact head/current/reference/control/introduced_by joins |
| /v1/assets/{asset_id}/versions/{version_id} | `{asset:asset_version_resource}`; exact opaque version ID, aggregate control and immediate relationship references; head field reserved and absent |
| /v1/assets/{asset_id}/supersessions/{supersession_id} | `{supersession:asset_supersession_resource}`; complete retained immutable reviewed transition and exact digest, with current complete metadata disclosure; no inline endpoint content or current-head sibling; complete response cap65536bytes |
| /v1/domains/{domain_id}/channels/{name}/generations/{generation} | `{event:context.channel_event}`; exact positive generation of the exact retained key, including complete reason, previous/result state and target, rollback reference, executor/delegation/audit and recorded time |

Use distinct exact response definitions for the seven routes, even where their
nested result value is shared. The general exact-response union must not be used
to accept another route's result. Do not fabricate source history on the basis
of a reference alone. Complete candidate historical state may be superseded
while its retained review remains accepted/rejected/contested or null exactly
under candidate-review contract. Candidate control is aggregate observed control, not a counter
restarted for each revision; exact read grants no permission to mutate history.

The current candidate and asset routes are explicit moving-head selectors. The
versioned routes never replace a missing/forbidden historical version with the
current one. Candidate/asset reads preserve the stored intended-use purpose;
their requested read purpose can differ only under the actual owning allowed
purpose/action policy. Tenant and domain selectors must match the route/owning
scope; any separately permitted cross-domain search use does not rename that
origin or imply a broader listing collection scope.

An observation is organization-scoped. A domain context selects the actual
authorized read purpose/source policy; it does not add an invented immutable
observation.domain_id or prove organization-global permission. Its referenced
originals, captures, basis observations and transcript/audit metadata retain
their own current disclosure checks. Context channel events similarly require
authorization for each returned previous/current build and attribution field.

These companions do not change existing capture and expense exact GET shapes.
Their separate exact-revision lifecycle state remains available through the
already selected lifecycle resolve operation; no additive extension is used
to quietly alter an old required snapshot. No additional capture exact/current GET schema or route is introduced here;
those existing `{capture}` resource-versus-revision responses retain their lifecycle contract
owner and fixtures. Listing is not an incidental rewrite of their transport.


### Exact normalized request and named response targets

Each named exact-GET request is the CLOSED typed HTTP observation {context,
path:<the following closed selector>}; it is not a wire JSON request body.
- candidate_current_get_request: path={candidate_id:common.identifier}.
- candidate_revision_get_request: path={candidate_id,version:common.content_revision}.
- observation_get_request: path={observation_id:common.identifier}.
- asset_current_get_request: path={asset_id:common.identifier}.
- asset_version_get_request: path={asset_id,version_id:common.identifier}.
- asset_supersession_get_request: path={asset_id,supersession_id:common.identifier}.
- channel_generation_get_request: path={domain_id,name:common.slug,
  generation:context.positive_generation}.

For path revisions/generations, the raw segment is the canonical decimal value
only. The route parser constructs the declared tagged typed value after exact
raw admission; it does not accept a percent-encoded/tagged JSON path segment.
All unannotated IDs above are common.identifier. Query context is complete and
all route/domain/ownership correlations are enforced as stated above.

Define <operation>_result and <operation>_response for each of these seven request
names with _request removed, using exactly its route row's required result.
For example candidate_current_get_response is Read200 of the named result
{candidate:candidate_review.candidate_resource}; asset_version_get_response
requires only its exact asset and expressly forbids a head sibling.
asset_supersession_get_response requires only the exact supersession resource
and forbids asset/head/content or another supersession sibling; endpoint
references are legitimate closed core fields, not inline resource extensions.
No GET request, result or response acquires idempotency_receipt, actor/body
authority, listing cursor/filter fields or a lifecycle operation wrapper.

### Producer admission must preserve complete read capacity

The actual [complete exact/current GET response](#r6-seven-exact-companion-routes-and-complete-responses), including envelope, required
result/resource/core fields, retained review and current observations, is at
most 1048576 UTF-8 bytes. The candidate-review contract's preimage cap alone does not prove
the candidate resource or an asset resource containing its accepted review fits.
All declared field capacities remain upper bounds; their maximal Cartesian
product is not a claim of an admissible complete resource.

Before the first canonical content/review/mapping write, the producer must
preflight every mandatory complete current and exact read representation it
creates. Reserve the declared maximum known required head/control/history
overhead that can appear later without editing that immutable content, including
maximum revision encodings, nullable-to-reference successor transitions,
accepted-review fields/reason capacity where acceptance is a supported later
transition, the complete asset head wrapper, both maximum immediate supersession
references, the aggregate observation on exact versions and head.introduced_by.
The separate exact supersession record GET is preflighted at its65536-byte cap;
it is not nested inside full asset reads. This reservation does not
authorize clipping a later review reason, dropping qualifications or shrinking
the complete resource silently. Unknown transport extensions are outside
identity and must fit the remaining actual wire budget when emitted; the
reservation need not promise space for arbitrary future extensions.

Resource serializers and mutation admission use the same published complete
wire profiles with independently derived boundary oracles, not a guessed
constant based only on source-string character counts. Preserve escaped UTF-8,
full nested references and actual additive-field composition. A fresh command
whose complete proposed content exceeds its qualified combined resource budget
fails before effects under the common request/domain size classification;
no committed content becomes unreadable merely because a later required fixed
control grows. If already retained necessary details are missing or corrupt,
reads retain their actual missing-details/unavailable failure distinction.

Candidate, review and asset models, canonical persistence and actual acceptance/supersession producers must qualify this complete-read capacity. They do not widen the common cap or assert that
schema validation measured encoded bytes. Actual new source needs the required
strict coverage and unit/integration boundary cases before capability activation.

## R.7. Retained metadata, authorized missing details and errors

A list filter is evaluated from exact canonically retained data at its selected
revision, not text from a newer head, a stale search projection, or an inferred
null. In particular knowledge_kind and review_due belong to immutable content;
their values can be read from a separately retained canonical discovery record
only when its exact binding and retention/disclosure are actually authorized.
This design creates no exception to purge/privacy policy requiring those fields
to survive. An unavailable original does not automatically mean a candidate's
own complete content is missing; keep source permission, material loss and
retained local record availability distinct.

Purged/restricted candidates and assets can appear as discovery summaries only
when every required summary field and every relevant predicate is still exact,
retained and currently disclosable. A lifecycle tombstone/accepted reference
alone cannot supply the old knowledge kind, review date or complete content.
Do not coerce missing values to `null`, insert a placeholder kind, strip a
required field or substitute a current revision. Null is a recorded known value
only where its owner explicitly permits it.

First establish current disclosure safely. For an authorized potentially
matching retained record whose required predicate or summary details cannot be
established, the bounded canonical selection must fail as missing details;
it cannot silently exclude the unknown row and claim successful page exhaustion.
Rows authoritatively known not to match need not disclose or reconstruct their
irrelevant content. A policy/service uncertainty is different from confirmed
retention loss and produces unavailable/503, not a fabricated empty collection.

The exact missing-details specialization is:

```text
resource_details_unavailable_problem = common.problem specialized to
  type:'urn:arboresce:problem:v1:conflict',
  code:'conflict', status:409,
  title:'Resource details unavailable',
  detail:'Required retained resource details are unavailable.',
  reason:'resource_details_unavailable'
```

It retains the common required type/title/detail/status/code and safe request
association under the common failure envelope. `reason` is required and fixed
for this specialization. No affected ID, hidden count, previous/current head,
content fragment, foreign scope or internal location is added. This is not
`operation_details_unavailable`: that lifecycle contract reason belongs to retained operation/
receipt details. Adopt the new reason in the selection contract owner, API missing-details
matrix and relevant consumer oracles. Do not repurpose an existing common
machine code or change all conflict reasons globally.

Unknown or undisclosable parent/exact identity remains 404; invalid authentication
401; permitted same-scope forbidden action403; malformed selectors400; valid but
inadmissible query/cursor422; required unavailable authority/infrastructure503.
Known authorized required retained-detail loss409 comes only after current
disclosure. Unexpected corruption is preserved and fails closed under the
existing safe internal/unavailable classification, never relabeled as a
successful resource. No response leaks which hidden predicate/reference failed.

## R.8. Actor binding and current identity assertions

The actor binds the actual executor and exact nullable delegation-chain reference defined by the context contract. A request can be authorized through several grants/policies; no singular effective authorizing principal is invented for its cursor identity.

```text
listing_actor = {
  organization_id:common.identifier,
  executor:context.principal_reference,
  delegation_id:common.identifier|null
}
actor_sha256 = SHA256(
  UTF8('arboresce.listing-actor.v1\0') || JCS(listing_actor)
)
```

The separator contains one NUL. All values are server-derived from the actual
authenticated request, never query/body fields. Direct human, agent or service
execution has null delegation. A delegated request selects one exact admitted
delegation-chain identity, with the executor and every grant/issuer/scope link
validated under current policy. A chain ID must bind immutable chain identities
and interpretation; editing a chain's members/scope creates a new identity,
whereas revocation is checked canonically on every request. No choice of a
different chain is silently treated as the old cursor identity.

Principal identity resolves the stable existing issuer/subject or independently
issued service identity. Email, bearer token bytes, token refresh, audience text
supplied by the client, roles cached in a token and a claimed human authority
never become cursor inputs. Same-principal credential refresh can retain actor
identity, but the new credential must independently pass current scope, action,
organization/domain/purpose, membership, revocation and delegation admission.
Cursor validity is not credential validity or a permission snapshot.

This is a small owning clarification to security/identity and the listing
actor profile. It introduces no user identity API, new key, delegation issuer,
retention bypass or assumption that human authority is required for every read.
The actual identity owner must qualify its mapping and full-chain checks before
runtime listing enables the profile, using deterministic direct/service/agent,
chain-switch, revoke/regrant, token-refresh and current-policy-race cases.



## S.2. Exact request, route and capability

Register public capability `search.select.v1`, implemented only after both the
actual search and selection-receipt implementation gates pass. Method and canonical route:

`POST /v1/context/search`

Request is uncompressed UTF-8 `application/json`, with no query parameters,
fragment, alternate path spelling or method override. It is this CLOSED object:

```text
search_request = {
  contract_version: 'v1',
  context: common.command_context,
  query: string,
  k: integer 1..20,
  filters: search_filters,
  source_build: context.build_reference | null,
  retrieval_profile: context.profile_reference,
  fields: ['candidate'] | ['candidate', 'direct_observations'],
  limits: search_output_limits
}

search_filters = {
  knowledge_kinds: candidate_review.knowledge_kind[1..8],
  source_domains: common.identifier[1..16]
}

search_output_limits = {
  max_output_bytes: integer 1..262144,
  tokenizer: context.profile_reference,
  max_output_tokens: integer 1..1048576
}
```

All fields are required, including explicit `source_build:null`, a nonempty
filter object and the complete chosen fields array. No client actor, policy,
generation, budget, provider/model expression, external URL, authorization flag,
allow-revoked/deprecated switch, cursor, command metadata or idempotency field
exists. Client libraries may offer explicit defaults, but serialize the resulting
complete request; the wire has no ambient profile or implicit source-domain set.
There is no command name in this read request, and no mutating-command receipt.

Query is 1..4096 Unicode scalars and at most 16384 UTF-8 bytes after decoding.
Preserve exact text, case, whitespace and Unicode; do not trim, normalize or
interpret query text as a grammar, SQL, template, shell command or instruction.
Empty is invalid; whitespace-only text is not secretly rewritten into another
query. Common raw JSON validation still rejects invalid UTF-8, lone surrogates,
duplicate keys, trailing bytes and unsupported numeric tokens. Select a narrower
65536-byte request cap before parsing, retaining common depth 16 and other
container limits. Query token/model input limits are independently enforced by
the actual qualified profile before any gateway invocation.

Filter arrays are sets: reject repeated identities before normalization; sort
by ASCII. Distinct filters combine with AND and each nonempty set combines with
OR. Empty arrays, unknown members and nulls fail. All eight knowledge kinds
explicitly mean all currently supported knowledge kinds; no future kind becomes
selected automatically. Source domains belong to the request organization.
Request domain is the target use context, not an implicit source-domain override.
Every requested source domain must be currently authorized for this search action
and purpose, including cross-domain source use. A domain UUID grants nothing.
An undisclosable/foreign requested domain receives the ordinary safe 404;
an explicitly disclosable forbidden action receives 403. This prevents silently
changing the caller's requested domain set after hashing it.

This intentionally small search filter grammar does not copy discovery filters
for pending candidates, expenses, temporal review scheduling or availability.
Only accepted assets eligible for the requested use can contribute results.
For source_build=null, select current eligible asset heads in the permitted
source domains; superseded/non-head versions do not appear through accidental
stale projection entries. A future unrestricted historical-corpus search needs
a separately owned selector; no default follows from an asset-history listing.
The exact source-build selector below already has its own historical-version
meaning and does not inherit the null-source current-head predicate.

For a nonnull source build, use exactly that existing immutable build's asset
reference set as the search universe, intersected with the explicit typed filters.
The build's organization/domain must match request context. Each referenced
candidate retains its original intended context. Cross-domain inputs still need
current source-use policy, and source_domains restrict those candidate domains.
Require the complete source build to remain currently eligible at admission,
including every required source; a revoked required source fails the source-build
request rather than turning it into a permitted subset. The output is still a
search selection over that eligible build, never exact resolution or evidence
that the full-build signature approves different selected bytes.
As already required by context-contract.md's selection/normalization rules,
that set may contain multiple exact versions of one asset, including retained
non-head versions independently eligible under explicit current trusted policy.
Do not substitute a newer version or reject one solely because it is non-head.
Revocation, purge and missing required source authority still deny the build.

## S.3. Retry, deadline and read-admission semantics

Each accepted HTTP search dispatch owns one server-issued `selection_id`, one
fixed ten-second request deadline and one shared search-budget object. Allocate
the identity and controlled audit/reservation records before the first external
exposure. It is an internal read-run identity, not a lifecycle durable operation.
Use one monotonic absolute deadline locally, retain its injected UTC observation,
and give subcalls only the remaining allowance. Every SQL transaction stays
within the existing two-second budget; no transaction spans provider work or
slow delivery.

No automatic HTTP retry is permitted once any part of the request might have
reached the server. A pre-dispatch local failure with proof that zero request
bytes were sent can retry that unsent request. A connection loss, timeout, 5xx,
ambiguous write or lost response never automatically reissues it. The CLI reports
that the selection result is unavailable and that admitted external work may
have occurred. A deliberate new user/API invocation is new selection intent,
reauthorized and charged to shared admission; it must not be disguised as retry,
resume, backoff, page fill or continuation of the previous selection.

Inside a dispatched selection, all source retries, gateway retries, reconnects
and refills retain the original ID, deadline, exposure/cost reservations and
retrieval budget. The selected source profile in section S.6 permits no retry of
a dispatched retrieval branch. This conservative choice removes ambiguity over
candidate counts after a source timeout; future retry support must prove a
bounded profile using the same counters, not reset them. Gateway reconciliation
after deadline can settle existing reservations truthfully but cannot initiate
new business work or emit a belated successful response.

There is no search poll/cancel endpoint or result-lookup promise in this first
contract. Request cancellation prevents subsequent admission, cancels owned
local work, and retains actual/uncertain gateway exposure evidence. It cannot
recall already transmitted query/content. An immutable successful receipt is
returned inline only after complete output preflight and the final authority
fence. Ordinary transport failure can lose that receipt; it cannot justify a
success claim, fabricated receipt or hidden second retrieval. This contract
does not imply that absence of a success body proves absence of provider work.

## S.4. Existing identities and registered search profiles

Reuse exact context contract `profile_reference`, `principal_reference`, `policy_reference`
and `build_reference` definitions. Request profile and tokenizer references
resolve complete immutable registered bytes, exact content revisions and hashes;
no caller-provided name can select arbitrary code. Organization/domain must
match the consuming context, except for explicitly qualified source-domain
projection membership described below. Profiles must be known, supported,
rights-cleared, currently allowed and qualified for the actual environment.
Missing/unqualified references fail without model/projection work.

This contract owns the public retrieval/chunk/output meanings. Actual projection identity producers and vector/embedding adapters must qualify deterministic projection and gateway inputs. This contract imports no future internal message schema and makes no claim that a message producer or verifier is qualified.

One immutable registered retrieval profile binds, with no hidden defaults:

- Search contract version, exact chunk profile, feature extraction and complete
  content/provenance rules; every participating embedding/tokenizer/reranker
  implementation/artifact/configuration identity and compatible dimensions.
- One dense-vector source, or dense plus sparse-vector hybrid sources, in the
  fixed order `dense_vector`, then `sparse_vector`. Both address the same pinned
  logical projection generation and exact candidate/chunk identities. Sparse
  production has no support until its real producer/query profile is qualified;
  there is no implicit SQL full-text fallback or unqualified second backend.
- The fixed allocations, round and candidate caps in section S.6; immutable source
  query controls and filter index inventory; stable input order and integer-score
  interpretation; complete required query embedding/reranking behavior.
- Finite query and rerank input token/byte limits, output cardinality/byte limits,
  feature-specific timeout/retry/concurrency/scratch limits, complete admitted
  gateway exposure and shared cost/job reservations, and actual profile-region/
  policy admissibility. No field may be omitted as unlimited or inherited from
  a vendor default. All fit the ten-second outer deadline and common transport
  constraints; larger bulk input requires an actually qualified blob interface.

These are required profile fields, not invented model/tool selections or budgets.
The wire maxima do not promise every possible candidate set fits a selected
operational profile. A profile unable to preserve its required consequential
rerank input within its declared envelope rejects the selection before that
exposure; it does not silently remove qualifications or supporting/refuting roles.

The projection reference is closed:

```text
projection_reference = {
  organization_id, domain_id, projection_id: common.identifier,
  generation: common.generation with value >= '1',
  retrieval_profile: context.profile_reference
}
```

The generation integer uses exact checked decimal comparison, not string order.
It belongs to this logical projection, not a channel. `domain_id` is the target
search domain; the profile declares and validates its permitted source-domain
membership. A same-organization cross-domain asset is not automatically indexed
or eligible. One query pins one exact active projection reference; all branches
and rounds use it. If active generation/profile changes before the final serving
fence, return 409/conflict with safe retry guidance and no automatic restart.
There is no mix of generations, silent fallback to an old collection or token
whose numeric value is treated as global commit completeness.

The projection producer's generation membership inventory must preserve exact retained versions
needed by eligible source builds within the qualified projection/source-domain
scope, including independently eligible historical/non-head versions and all
their profile-defined chunks. Head movement alone must not remove those versions
from this inventory. The vector-source adapter's typed predicates and canonical search admission apply
current-head selection only when source_build is null; a nonnull source build
uses its exact version set and current eligibility instead. Required lifecycle
restrictions and physical purge retain their authority; this obligation grants
no extra retention, embedding exposure or historical-source permission.

Projection delivery and rebuilds must carry that same membership interpretation
into the pinned generation. A structurally current-head-only profile is not
qualified to serve this contract's exact source-build selector; it cannot report
silent empty/incomplete success to conceal its unsupported membership semantics.
Actual delayed delivery remains subject to the existing honest lagging/unknown
freshness and required-backend failure rules. Membership is not a promise that
approximate retrieval will return every eligible matching version: the same
two-round/200-candidate budget and exhaustive:false semantics still apply.

## S.5. Retrieval chunks and complete selected output

Select initial chunk algorithm `statement_utf8_4096_v1`. Its immutable registered
profile binds exact candidate.statement UTF-8, no text normalization, greedy
nonempty chunks at the largest complete Unicode-scalar boundary <=4096 bytes,
zero overlap, left-to-right order and full coverage of the statement. Nonfinal
chunks are at least 4093 bytes and therefore contain at least 1024 Unicode
scalars, because one scalar encodes to at most four bytes. The existing 16384
statement-scalar cap therefore permits at most 16 chunks, including multibyte
boundary cases. This bound uses both scalar and byte constraints, rather than
assuming a raw-byte division alone proves reachability. Empty statements/chunks
are invalid.

The chunk is a retrieval feature of one exact reviewed statement. It is not
the context contract's JSON transport chunk, an independent observation, an original
media locator, a complete approved build or a generated summary. Its text has
exact asset/candidate provenance; only independently existing observation contract grounding
may claim location in source media. One retrieval point cannot reassign an
assertion to document evidence or manufacture a quote locator.

```text
retrieval_chunk_core = {
  kind: 'arboresce.retrieval-chunk.v1', contract_version: 'v1',
  asset: candidate_review.accepted_asset_reference,
  chunk_profile: context.profile_reference,
  field: 'statement',
  start_byte: integer 0..65535,
  end_byte: integer 1..65536,
  text_sha256: common.sha256
}
retrieval_chunk_reference = {
  asset: candidate_review.accepted_asset_reference,
  chunk_profile: context.profile_reference,
  chunk_sha256: common.sha256
}
chunk_sha256 = SHA256(JCS(retrieval_chunk_core))
```

Offsets are half-open UTF-8 byte offsets within the exact candidate statement;
require start < end, end-start <=4096, scalar boundaries, correct complete
profile-defined segmentation and equality of text_sha256 with those actual
bytes. Chunk identity excludes generation and embedding provider: reindexing
unchanged content with the same chunk profile preserves chunks while projection
and embedding identities change. The reference's repeated asset/profile must
equal its core. Every accepted mapping and complete candidate fingerprint is
revalidated from canonical state; a syntactically valid hash is insufficient.

The selected output is a CLOSED canonical document, directly carried as a JSON
object, not doubly encoded text or context contract compiled_content:

```text
selection_document = {
  kind: 'arboresce.search-selection-document.v1', contract_version: 'v1',
  context: common.command_context,
  fields: ['candidate'] | ['candidate', 'direct_observations'],
  assets: selected_asset[0..20],
  matches: selected_match[0..20]
}

selected_asset = {
  reference: candidate_review.accepted_asset_reference,
  content: candidate_review.candidate_content,
  observations: context.observation_core[0..192]  // only with direct_observations
}

selected_match = {
  rank: integer 1..20,
  chunk: retrieval_chunk_core,
  chunk_sha256: common.sha256
}
```

Both selected_asset variants are closed. The `candidate` projection always
includes the entire immutable consequential candidate content: statement, scope,
applicability, all assumptions/qualifications, every supporting/refuting/qualifying
reference, invalidation conditions and explicit review_due. There is no arbitrary
JSONPath/field mask, text-only approved view, clipped statement or request to
drop qualifications. `direct_observations` additionally includes the COMPLETE
distinct direct observation union named by all three evidence-role sets, using
the context contract's closed projection of the complete observation union. Missing, contradictory or forbidden
required observations fails construction; never return an arbitrary partial union.

With only candidate, observation bodies are explicitly not included. Exact
evidence references and roles remain; the client may use the separately defined
exact observation reads after current authorization. This is a declared field
projection, not a declaration that omitted source bodies are irrelevant or that
the candidate contains independent proof. A consumer needing actual evidence
statements must request direct_observations or fetch/verify those complete cores
before that use. All underlying source restrictions are checked in either view.

Normalize evidence-role sets and observation unions exactly as candidate-review and context contracts; preserve
all meaningful prose/list/locator order and source attribution. Assets are unique
by (organization_id,asset_id,version_id), sorted by that ASCII tuple and exactly
the union referenced by matches. Conflicting values for the same identity fail.
Every match refers to one retained complete selected asset. Matches have
contiguous ranks 1..N and preserve section S.7 order. No additional unmatched
asset, orphan reference, duplicate exact chunk or stale digest is permitted.
An empty result has both arrays empty; it is valid even though a Context Build
cannot be empty. Do not validate it through the complete-build contract.

Canonical output bytes are JCS(selection_document), UTF-8 with no BOM/newline.
`output_sha256 = SHA256(output_bytes)`; this is not a build digest, candidate
digest or receipt digest. Profiles/query/selection ID/time are outside this
output preimage because they describe selection execution, while context and
explicit content field view affect selected bytes. The receipt binds all of them.

## S.6. Shared retrieval reservation and failure rules

Each selection owns an internal monotonic budget initialized to zero used rounds
and zero irrevocably charged retrieval slots. Hard ceilings are 2 rounds and
200 charged candidate slots BEFORE deduplication across every source. Allocate
before dispatch, using atomic counters shared by concurrent branches; neither
remaining counters nor reservations are separately instantiated per source.

Choose fixed maximum 100 candidate slots for each of at most two rounds. A
single-source profile reserves 100 dense slots per round; a hybrid profile
reserves 50 dense and 50 sparse slots per round. All planned branch reservations
and the round charge commit together before the first branch dispatch. Unused
or denied work does not mint a third round. These fixed allocations favor an
auditable bounded first profile; a later allocation profile needs independent
qualification and must preserve the global 2/200 ceilings.

For each source branch, reserve its declared limit before calling it. Once
dispatch could have happened, the entire reservation remains charged even if
the source returns fewer items, repeats items or times out; never recycle unknown
returned capacity. Actual returned occurrences, including duplicates across
sources/rounds, must not exceed the reserved amount. Enforce source envelope
byte/array bounds during intake; a lying/oversized/malformed source is a required
dependency error and no selection success. Do not merely discard item 51/101
and pretend the source complied. Validating a reservation count in a JSON fixture
does not prove the adapter enforced the actual remote response bound.

All branches in round 1 share that one round. Wait for its complete bounded
required outcomes; batch canonical metadata/current permission/dependency checks
over the returned occurrences, fetch canonical content in bounded batches, then
rerank only permitted content. SQL identity fetches may coalesce repeated keys
for I/O, but the original pre-dedup occurrence accounting is retained. No N+1
policy request, remote call containing rejected content or client-side ACL filter
can replace this step. Query embedding is separately gateway-authorized before
the first query exposure, even if the corpus later yields zero visible matches.

Round 2 is permitted only if the authorized deduplicated result set after round 1
is shorter than k and the original deadline/cost/admission remains valid. It is
the ONLY refill. Use the same pinned generation, original filters and immutable
source query profile. A source continuation marker, where that qualified source
supports one, is private bounded execution data within this second round; it is
not an authenticated public listing cursor or client-controlled offset.
Repeated candidates from a source lacking continuation consume slots normally.
Do not loop until the caller's k fills.

After round 2, rerank/merge the retained permitted occurrences using the same
profile, deduplicate and return the bounded prefix or an actual error. No retry
of a dispatched retrieval branch is allowed in this profile; source timeout or
transport ambiguity fails the search. Internal gateway retries, if qualified
for that feature, retain the original overall deadline and cost/exposure ledger;
they do not trigger a new retrieval round, extra query candidates or new selection.

Exhaustion is a safe Boolean observation: true when k remains unsatisfied and
the selected shared retrieval budget prevents further allowed retrieval. Do not
publish charged/returned/denied/filtered/duplicate/scanned counts, per-source
candidate counts, denied IDs or a protected reason explaining scarcity. Public
applied limits are the constant caps, not actual pre-authorization counters.
The visible array length and whether the public k was met are already disclosed.
Internal exact counters are mandatory controlled qualification/audit evidence.

## S.7. Ranking, ties, deduplication and output-capacity prefix

The registered rerank adapter returns one bounded exact unsigned integer score
0..1000000 for each identified permitted input occurrence, or a complete failure.
The profile fixes the deterministic conversion from its actual scorer to this
scale, including rounding and unsupported values. Scores are ranking metadata,
not probabilities, confidence in truth or calibrated quality. Reject NaN,
infinity, missing/extra/duplicate occurrence IDs and out-of-range/raw-number
aliases; never accept a model-selected identifier or ordering instruction as
canonical identity. No public score is needed in the first output.

Before each scorer invocation, order permitted input occurrences by ASCII
(organization_id,asset_id,version_id,chunk_sha256), then fixed source order,
round number and source position. Keep occurrence identity in the bounded
internal input/output contract. A reranker sees only content and provenance
authorized for that actual external exposure, under the declared profile.
Any claimed equivalence between different provider score formats is a qualified
profile decision, not an arbitrary float cast by the public API.

Dedup key is exact (organization_id,asset_id,version_id,chunk_sha256). The same
claimed key with inconsistent canonical bytes/profile is corruption and fails.
For repeated consistent keys, retain the maximum score; ties use the earliest
canonical occurrence order above. Then sort unique matches by descending integer
score and ASCII (organization_id,asset_id,version_id,chunk_sha256). Take at most
k. Raw backend order, floating point incidental ties, hash-map iteration order,
database arrival and task scheduling must not settle a tie. Stable ordering is
required for identical frozen source/scorer outputs. Approximate retrieval or a
different observed generation may legitimately produce a different selection;
the receipt records that actual result instead of promising identical relevance
or corpus completeness on a new invocation.

The provider/scorer consumes the qualified complete feature view; the public
output preserves every selected candidate's consequential content regardless of
the optional evidence-body view. Exact full evidence and source policy always
remain canonical admission dependencies. Source/reranker errors are not zero
scores, abstentions, fewer permitted candidates or successful budget exhaustion.

Build the complete selection_document for the ranked candidates before emission.
Its canonical byte cap is min(request.max_output_bytes,262144); tokenize the
EXACT full JCS document with request.limits.tokenizer and enforce the caller's
max_output_tokens plus the qualified tokenizer profile's own envelope. There is
no byte-to-token estimate, ambient tokenizer, model-context promise, silent model
fallback or omitted prompt framing. Prompt/system/other-message tokens remain
separate downstream admission, just as for exact build resolution.

Select the LARGEST ranked prefix that fits both caller caps, with complete assets
and the exact selected observation projection. Evaluate at most 21 prefixes,
including empty, under the same deadline and qualified tokenizer work budget;
do not assume token count is monotonic when content changes. Prefix search must
not reorder lower-ranked smaller candidates ahead of a larger higher-ranked one.
If at least one candidate exists and no nonempty prefix fits, return 422
invalid_command for the requested output envelope, with no successful empty
selection or precise protected size/count. If the valid empty document itself
cannot fit the limits, the same error applies. No retries or retrieval refill
occur to find smaller replacements merely because output caps were hit.

If a nonempty shorter prefix is chosen, `output_truncated:true` explicitly means
whole ranked matches were omitted by caller output caps. It never means any
returned candidate field, evidence role, observation union, string or chunk was
truncated. The selected immutable receipt binds the final actual prefix and view;
it does not claim omitted content shares its output identity or approval.

## S.8. Freshness, incompleteness and safe empty success

Selection is always a `bounded_selection`, never proof of exhaustive knowledge,
perfect relevance or a complete context build. Public completeness metadata:

```text
selection_completeness = {
  kind: 'bounded_selection',
  exhaustive: false,
  requested_k_met: boolean,
  retrieval_budget_exhausted: boolean,
  output_truncated: boolean
}

selection_freshness =
  {kind:'current_at_observation', observed_at: common.timestamp} |
  {kind:'lagging', observed_at: common.timestamp} |
  {kind:'unknown', observed_at: common.timestamp}
```

All objects/variants are closed. requested_k_met equals final visible match count
== k. output_truncated is true exactly when output caps remove at least one
otherwise selected match. retrieval_budget_exhausted is measured before output
prefix reduction: fitting fewer bytes does not rewrite retrieval history.
No exclusivity constraint falsely assumes only one reason can hold. An output
truncation can coexist with a short retrieved set and exhausted retrieval budget.

Freshness is a safe scoped observation made before final admission. Current
requires actual complete canonical-delivery/barrier evidence for the admitted
projection/source-domain scope at the named observation; a maximum event ID,
quiet queue, recent event timestamp, checksum of visible hits or zero observed
pending rows does not establish it. It remains an observation, not a promise
that no concurrent later write exists. Lagging requires an actual safe scoped
backlog/lag observation. Unknown is required when safe complete freshness cannot
be established. None exposes another principal's rows, exclusion counts,
commit identifiers, backend topology or age of protected individual work.
If even the coarse state is not permitted disclosure, use unknown.

Lagging/unknown may return 200 only when retrieval completed normally and all
returned bytes passed current canonical authority, identity and dependency
checks. Required backend, tokenizer, canonical database, policy or gateway
failure is 503/unavailable, not freshness=unknown, zero results or success with
an unavailable source. A profile's required hybrid branch is not silently
dropped. A generation change is the explicit conflict above.

A successful empty document means no permitted result was returned by this
bounded successful operation. It does not distinguish absent from inaccessible
matches and does not assert that a corpus/build contains nothing. When both
rounds completed but k remains short, exhaustion is reported truthfully. When a
qualified complete source ends early and no further branch/round can contribute,
the implementation may terminate before the second round and report exhaustion
false; it must possess complete source-end evidence within the selected profile,
not infer exhaustion from one short approximate response. Otherwise use the
remaining permitted refill. Freshness and exhaustive=false still apply.

## S.9. Immutable receipt, hashes and actual exposure truth

Use closed normalized request bytes, not ad hoc JSON or a hash of a display view:

```text
normalized_request = search_request with only filter set ASCII normalization
request_sha256 = SHA256(
  UTF8('arboresce.search-request.v1\0') ||
  JCS({method:'POST', resource:'/v1/context/search', body:normalized_request})
)

selection_reference = {
  organization_id, domain_id, selection_id, record_sha256
}

selection_receipt_core = {
  kind: 'arboresce.search-selection-receipt.v1', contract_version: 'v1',
  selection_id: common.identifier,
  context: common.command_context,
  request_sha256: common.sha256,
  source_build: context.build_reference | null,
  retrieval_profile: context.profile_reference,
  projection: projection_reference,
  fields: ['candidate'] | ['candidate', 'direct_observations'],
  selected: retrieval_chunk_reference[0..20],
  output: {
    sha256: common.sha256, byte_length: integer 1..262144,
    media_type:'application/json', encoding:'utf-8'
  },
  executor: context.principal_reference,
  delegation_id: common.identifier | null,
  policy: context.policy_reference,
  authority_audit_event_id: common.identifier,
  limits: {
    requested_k: integer 1..20, max_retrieval_rounds:2,
    max_pre_dedup_candidates:200,
    output: search_output_limits
  },
  actual_output_tokens: integer 1..1048576,
  freshness: selection_freshness,
  completeness: selection_completeness,
  gateway: search_gateway_observation,
  admitted_at: common.timestamp,
  transfer_expires_at: common.timestamp,
  delivery: 'admitted'
}

record_sha256 = SHA256(JCS(selection_receipt_core))
selection_receipt = {core:selection_receipt_core, record_sha256}
// The core is closed; this two-field transport wrapper has bounded additive readers.
```

Every displayed field is required. Receipt/core/reference identities agree with
actual organization/domain/selection ID; all profiles, selected references and
output fields agree with the normalized request and emitted document. `selected`
is the complete ordered match-reference list, not an unordered subset, and retains
every asset/candidate/review/chunk identity. The output hash is recomputed from
the exact canonical document, and its actual token count uses the declared
tokenizer. Receipt self-hash and response extensions never enter core identity.
No full-build attestation/signature, approval boolean, truth/confidence score,
downstream success or context contract usage_id exists in this receipt.

Actor is authenticated and server-derived, with the exact context contract principal kind
and canonical issuer/subject resolution. Delegation is the verified actual
chain's reference or null for direct execution. The actual authority audit event
binds current membership/delegation, source restrictions, declared purpose and
immutable policy content. Reusing these existing types does not allow a client
to submit authority claims or replace a missing selected identity/profile input.
The receipt records current admission once; subsequent revocation does not edit
its immutable past or turn it into permission for another exposure.

Freshness observed_at is an actual observation within this selection's request
lifetime and no later than admitted_at. Admission occurs before the fixed request
deadline; transfer_expires_at is strictly later than admitted_at. Actual token
and byte counts are positive and no larger than the caller and qualified-profile
limits. These arithmetic/equality constraints are semantic requirements, not
proofs obtainable from a parsed-value schema alone.

Use the lifecycle contract's existing `exposure_reference` for actual gateway record links;
do not fabricate an operation/stage merely to reuse the whole lifecycle contract exposure_entry.
Define the following public observation over the actual gateway ledger:

```text
search_gateway_observation = {
  kind:'complete_at_admission',
  query_embedding: search_feature_observation,
  reranking: search_feature_observation
}

search_feature_observation =
  {kind:'not_used'} |
  {kind:'local', profile:context.profile_reference} |
  {kind:'retained', profile:context.profile_reference,
   source_exposures:lifecycle.exposure_reference[0..8]} |
  {kind:'external', profile:context.profile_reference,
   exposures:search_exposure_observation[1..8]}

search_exposure_observation = {
  reference: lifecycle.exposure_reference,
  state:'attempted'|'completed'|'uncertain'|'confirmed_not_performed',
  settlement:'reserved'|'settled'|'unsettled',
  receipt: {kind:'pending'} |
           {kind:'retained',receipt_id:common.identifier,
            receipt_sha256:common.sha256}
}
```

The two feature positions and maximum eight invocations/references each give a
finite public observation; an actual qualified profile may be narrower. All
required feature work must yield complete validated output for selection success.
not_used is valid for reranking only when no permitted candidate was presented;
query_embedding is required by the selected dense profile and cannot be not_used.
local means the actual qualified implementation made no external call for that
feature in this selection. retained means it reused exact currently permitted
content/profile/scope-bound output and created no new call; its retained provenance
links are complete, with empty source_exposures only for originally local work.
Unknown retained provenance is a dependency error, not an empty list.

external carries every actual invocation admitted for that feature in this
selection, including failed/uncertain earlier attempts before an independently
permitted successful retry. Identities are unique and belong to current scope;
the gateway's controlled complete source/input/profile/policy/authority bytes
must bind these public links. A completed attempt does not imply settled billing;
uncertain does not imply performed or absent. Settlement/receipt consistency and
affirmative not-performed interpretation reuse the [lifecycle exposure-state
and settlement owner](lifecycle-contract.md#3-shared-operation-plan-clocks-and-evidence).
If a feature succeeded externally, at least one entry has completed output
accepted for that exact feature input/profile; extra failed/uncertain attempts
remain visible within authorized disclosure. Partial or invalid provider output
cannot be called completed for this purpose. Unknown outcome blocks unsafe
repeat admission until actual gateway reconciliation permits it.

This observation is complete for admitted feature invocations at the receipt's
immutable observation time, not a claim that every cost/outcome is settled or
that a downstream client received bytes. Later authentic settlement stays in
the gateway's retained records; it does not rewrite this receipt. Missing data,
over-eight invocations, unqualified public projection or unsafe disclosure fails
preflight, rather than truncating a ledger or inventing no exposure. Public
provider URLs, credentials, hidden candidate counts and content do not enter it.

The gateway must qualify the concrete producer/verifier and the selected read-run
attribution before search activation. The existing lifecycle exposure reference does not qualify the future read-run gateway producer/verifier. The actual consuming profile must establish those message and ledger bindings before its first search use. No new provider or signing key is selected.

The receipt is an authenticated-server assertion over TLS, with verifiable content
hashes after current authorization. It is UNSIGNED; there is no signature field,
offline authenticity promise or inheritance from a source-build attestation.
Obtaining a syntactically correct/hashed receipt from arbitrary storage does not
establish trusted origin. Creating independently verifiable selection attestations
or selecting new trust keys requires a later explicit owner contract and authority.

## S.10. Complete response, final fence and errors

HTTP 200 returns a common completed-success envelope with no idempotency receipt
and additive `result:{document:selection_document, receipt:selection_receipt}`.
Core/document objects remain closed. Only the declared outer result/receipt
transport containers use the existing at-most-32-member additive reader profile,
with reserved outcome/approval aliases excluded by their actual property tables.
Extension values cannot alter hashes, identity, qualification, limits or outcome.

Freeze a 524288-byte complete encoded response cap, within common 1 MiB. Output
canonical bytes have the separate 262144 cap. The document is emitted directly
under the result with a canonical compact UTF-8 serializer, so it is not escaped
again as a string; response framing plus the complete receipt and bounded producer
extensions must fit the remaining capacity. Materialize the complete response
and enforce actual byte/depth bounds before final admission. Do not assume the
per-field maxima prove a combined envelope fits. The consumer checks strict raw
input, byte/depth cap, full schema, joins, canonical output/receipt hashes and
byte/token evidence before presenting verified content or sending it downstream.

The output-prefix search in section S.7 considers output caps only. After choosing
that prefix, failure to fit the complete required response, retained exposure
observation or other fixed metadata is unavailable/503, not an excuse to remove
more selected content silently or manufacture a smaller receipt. Proposed known
metadata bounds are deliberately much smaller than the 262144-byte remaining
headroom, but the later design/fixtures must prove the reachable worst-case
composition, including escaping and nested observation depth.

Immediately before admission/exposure, recheck current executor/delegation,
organization/domain/purpose policy, every returned asset and its full required
source dependency closure, complete source-build eligibility when selected,
profile support, pinned active generation and availability. Required canonical
identity/authorization must remain valid under the same fenced transaction/audit
boundary as exposure admission. No stale cache/projection result can grant use.
If permission changes while preparing output, rebuild/revalidate locally under
the original remaining bounds or fail; no new retrieval budget or partial
stale-authority response follows. Use bounded batches, not one SQL call per match.

Commit immutable receipt/admission/audit linkage before emission. Set
transfer_expires_at to the minimum of admission+10 seconds, the original request
deadline and relevant current authorization expiry, with representable checked
UTC arithmetic and a positive remaining interval. Send only within that
already-bounded admission. Deadline/cancellation may leave incomplete transport;
the consumer rejects it and does not label delivery complete. `delivery:'admitted'`
never proves receipt, use or correct downstream reasoning.

Do not hash a guessed future database-commit timestamp into that receipt.
admitted_at is the injected clock observation used by the successful final
admission transaction, not a promise about its later physical commit time.
Prepare/tokenize the complete content and variable-size metadata before the
transaction. Within the bounded final transaction, choose that observation,
finalize the fixed-size time/ID/hash fields and complete encoded buffer, then
perform the current-authority/generation/deadline fence and atomically bind the
receipt/audit before commit. If final encoding or the fence fails, no receipt
is admitted or body emitted. Qualify this bounded serialization inside the
two-second transaction; no model work, tokenization, remote calls or stream
delivery occurs under its locks. Never emit a prepared-but-uncommitted receipt.

Use existing common problem types, not selection-specific success-shaped errors:

| Outcome | HTTP/common code |
| --- | --- |
| Malformed/duplicate JSON, unknown fields/filters, raw bounds other than request byte cap, invalid field array or unsupported shape | 400 invalid_request |
| Missing/invalid authentication | 401 unauthenticated, with required challenge |
| Disclosable forbidden action/purpose | 403 forbidden |
| Unknown/foreign/undisclosable source build, domain or referenced public resource | 404 not_found |
| Pinned active generation changes at final fence | 409 conflict |
| Request wire byte cap exceeded | 413 request_too_large |
| Unsupported content type/encoding | 415 unsupported_media_type |
| Well-shaped but unsupported/unqualified selected profile or valid output limits that cannot admit even the required first result/empty document | 422 invalid_command |
| Shared job/cost/request admission rejects before work | 429 rate_limited with bounded Retry-After |
| Required backend/policy/retained bytes/tokenizer/gateway unavailable, malformed source output, missed request deadline or impossible complete response | 503 unavailable |
| Unexpected internal invariant/implementation failure | 500 internal_error |

Profile existence and support diagnostics obey disclosure policy; inaccessible
profile identity is not revealed through 422. Return safe bounded errors without
query text, SQL, hidden IDs/counts, precise protected omitted sizes or provider
metadata. A 503/Retry-After is guidance for a later deliberately new invocation,
never authority for automatic retry of uncertain work. Existing source-build
unavailability/eligibility rules retain their documented non-disclosing behavior;
do not invent a successful empty build selection. No required backend failure,
timeout, failed final authorization or invalid identity qualifies as short success.

## S.11. Caches and immutable observation versus future feedback

Prefer uncached first implementation unless a measured repeated workload justifies
an explicit cache. Any later cache key binds organization, executor/delegation,
requested purpose, policy epoch/content, exact normalized request/filter/fields/
limits, exact source build when present, all participating profiles and active
projection generation. Query text alone, output digest alone or a shared vector
hash is insufficient. A cache hit never returns another principal's receipt or
reuses an old authority audit event as new admission; reauthorize all selected
dependencies and issue a fresh honest read receipt for the actual served bytes.
No denied count or cross-tenant hash-existence oracle is permitted.

This contract does not add search selection to the lifecycle contract's already closed feedback
union. Search `selection_id` is not context `usage_id`; a retrieval chunk is not
the exact build compiled identity. Existing asset/build feedback can retain its
actual owned asset/build attribution, but must not pretend it names this selection
receipt. If later requirements need direct search-receipt feedback, add a reviewed
finite selection contract selection_reference subject branch and actual feedback implementation and client consumers
before enabling it; never silently reinterpret the lifecycle contract's current identity.

Search audit/receipt/query and retained gateway records require explicit finite
storage quotas and approved retention/disclosure policy before runtime admission.
Immutability means retained observations are not rewritten; it does not grant
perpetual retention of query text, a seven-day search replay service or the lifecycle contract's
permanent mutating-command recognition semantics. Keep secrets outside those
records. Query/request hashes are protected observations, not anonymous data;
guessable queries must not become an unrestricted cross-user hash oracle.

## S.12. Independent fixtures, semantic oracles and runtime evidence

For this selection contract, independently freeze parsed literal values/outcomes and exact target/case
inventories from the final prose before comparing new implementation schemas.
Every closed branch, max/min and additive reader needs independent positive and
negative coverage. Do not create generation expectations by introspecting the
schema or merely mirror its constants in unmeasured helper code.

Minimum parsed targets include request/filter/fields/limits, source-build null
and exact variants, projection/profile references, chunk core/reference, both
selected-asset variants, empty/nonempty documents and ordered matches, all
freshness/completeness variants, every local/retained/external exposure observation,
receipt core/reference/response and common problem composition. Include k=0/1/20/21,
domain/filter empty/duplicate/unknown combinations, scalar versus UTF-8 byte
boundaries, start/end offsets, response aliases, every exposure state/settlement
branch and source/profile identity substitution. Structural schema success does
not compute hashes, check actual byte/token equality or authorize content.

Freeze independent semantic values/vectors for:

- Exact normalized request JCS and domain separator (one NUL, never printable
  backslash-zero); set permutation equality versus meaningful query/field/limit/
  source-build changes; no default profile or Unicode normalization.
- Complete candidate fingerprint and accepted mapping; chunk UTF-8 scalar
  boundaries, 4096-byte and 16-chunk edge cases, exact text/core/reference hashes,
  generation changes without chunk identity changes and wrong source-locator
  attribution. Context transport chunks must fail retrieval-chunk selection.
- Exact canonical output and receipt preimages, repeated asset elimination only
  in the document union, complete evidence-role/observation union, empty result,
  contiguous ranks and order, foreign/same-ID-different-byte contradictions.
- Stable integer-score ties and canonical occurrence order; source/round/task
  arrival permutations, duplicated cross-source chunks, a changed profile and
  no probability/approval interpretation. Independent actual ranking-quality
  holdouts remain a separate gate, not a deterministic synthetic claim.
- Two rounds with 100/100 or 50+50 per round; 199/200/201 actual occurrences;
  duplicates consume capacity; pre-dispatch shared reservations under races;
  short sources do not return charged capacity; no third round, dispatched
  branch retry, hidden cursor refill or client automatic replay after uncertainty.
- Canonical authorization before rerank, query-specific exposure authorization,
  source-build full eligibility, stale/foreign/superseded projection records,
  source revocation at every boundary, cross-domain policy and bounded SQL
  batches. Assert that denied text never reaches the actual controlled gateway.
- Exact-build historical membership scenario: freeze an eligible build containing
  v1 and v2 of one asset, with v2 the current head and explicit current trusted
  policy permitting retained non-head v1. Give each version one exact chunk and
  freeze source/scorer outputs returning both under an exact-build request with
  k=2 and sufficient output limits. Its permitted output retains both exact
  versions; null-source search excludes v1 under the current-head predicate.
  Head movement and a projection rebuild preserve the build's eligible version
  membership. Revoking v1 instead denies the entire source-build request before
  exposure, rather than selecting v2 alone. A current-head-only membership
  configuration fails qualification; actual delivery lag is truthfully recorded
  and never disguised as supported complete membership. The controlled source
  outcomes are a semantic oracle, not an approximate-recall or exhaustive claim.
- Full/short/empty permitted success versus source-end proof, budget exhaustion,
  lagging/unknown freshness and actual backend failure. No hidden count/ID leaks
  in output, cursor, problem, logs or metrics; max event IDs are not completeness.
- All 21 output-prefix checks, nonmonotonic tokenizer example, first oversized
  result rejection, whole-match truncation with all consequential fields retained,
  requested direct-observation union too large, exact 262144/262145 output and
  524288/524289 response bytes, depth 16/17, token cap equality/overflow and
  mismatched/unqualified tokenizer. Measure actual serializer, not a size estimate.
- Current principal/delegation/generation/profile changes during preflight;
  fixed outer deadline, no SQL lock across provider/streaming work, admission/
  audit/receipt atomicity, incomplete transport and truthful no-auto-retry client.
- Complete actual query/rerank exposure observations, local/retained provenance,
  uncertain attempt followed only by an actually permissible retry, unsettled
  usage, late reconciliation without new work and no fabricated no-exposure
  claim on failure. Validate exact producer/verifier bytes and full ledger joins.
- Signed full-build source versus unsigned distinct selection, tampered output/
  receipt, arbitrary receipt origin, expired source authority and lifecycle contract usage-ID
  substitution. A hash match cannot create a signature or current permission.

The contract checkpoint runs the existing locked contract checker and complete
family graph/oracle qualification, including reference consumption, invalid-value
versus infrastructure distinction, required negative probes and independent
ordinary consumers. It does not run a phantom search service or claim live keys,
tokenization, cursor MAC, JCS runtime, retrieval quality, latency or >90% source
coverage from declarative fixtures.

Later real RUST/PYTHON/SERVICE/E2E profiles must register deterministic independent
suites, complete collections and all exercised process/source identities. The
strict raw >90% file/package/repository/profile coverage gates apply to actual
maintained product and fixture support. Test actual Qdrant, canonical SQL,
gateway, encoder/tokenizer, concurrency/fault barriers and CLI process; required
native/capacity/real holdout evidence remains separately required. No mock, skipped
suite, stale coverage or nearby hardware result substitutes for those gates.
