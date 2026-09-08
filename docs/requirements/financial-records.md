# Financial records and reports

**Authority:** accepted normative requirements. Expense, report and export
runtime behavior is not implemented. This document owns financial meaning,
complete content identity, review, immutable selections and export encoding.
The [financial schema](../../contracts/public/v1/financial-records.schema.json)
owns structural fields and variants. Its
[standalone validation guide](../../contracts/validation/README.md) separates
parsed-value checks from the pending semantic qualifications below.

The required journey is receipt and note, extraction proposal, user correction,
confirmed record, period report and independently usable export. Knowledge
publication is not a prerequisite. [Product scope](product-and-scope.md) owns
the broader boundary; [data model](data-model.md) and
[common wire encoding](common-contract.md) own shared identities and encodings.
[Security and privacy](security-and-privacy.md) owns current authority and human
assurance. The [rolling plan](../execution/open-cli-mvp-v1-rcld.md) records
implementation and evidence separately.

## Evidence and record lineage

Receipts, photographs and notes are evidence. Extraction proposes financial
content; it cannot confirm a transaction. Preserve the actual distinction
between document-derived observations, user assertions and model inferences.
Purpose, participants, allocation and reported meeting outcomes are not facts
proved by a receipt. Their attribution and access rules remain independent.

An expense is an organization/domain-scoped aggregate with independent content
and control revisions. Each immutable revision retains its complete financial
content, exact field sources, date precision, reconciliation and attributable
review. Correcting a field creates new content. Changing currency cannot
reinterpret an older revision, its source amounts or an existing report.
Server-issued opaque IDs, a digest and a source label confer no authority.

The exact `expense_reference` contains `organization_id`, `domain_id`,
`expense_id`, `version` and `content_sha256`. Version uses the common tagged
content-revision type. References must resolve that exact historical content;
a current head, filename or similarly valued record is not a substitute.

For kind `expense`, `original` is null. Every admitted `refund` or `credit`,
including a proposed revision, requires an exact nonnull original reference.
The original must be a historically confirmed expense in the same
organization/domain, not a refund, credit or self-reference. Known interpreted
currencies must match; confirmation requires that check and current permission
to use the original's retained review/history. Later supersession or void of
the original does not erase its historical confirmation or change the link.
There is no implicit cumulative-refund ceiling. Unlinked extraction remains
unresolved source/review material; do not fabricate an original or relabel a
refund as an expense.

## Exact monetary values

The initial admitted currency policy is:

| Currency | Payable scale | Canonical zero for a component |
| --- | --- | --- |
| AUD, CAD, CHF, EUR, GBP, USD | 2 | `0.00` |
| JPY | 0 | `0` |
| KWD | 3 | `0.000` |

This selected set is project policy, not the complete ISO currency list.
[SIX's maintenance service](https://www.six-group.com/en/products-services/financial-information/market-reference-data/data-standards.html)
and its [published currency list](https://www.six-group.com/dam/download/financial-information/data-center/iso-currrency/lists/list-one.xml)
provide the currency/minor-unit reference. A later support-policy change needs
an explicit versioned contract decision; updating a remote list cannot silently
change stored interpretation.

Every wire amount is an ASCII decimal string. Its integer part is `0` or a
nonzero digit followed by at most 13 digits. Scale 0 has no decimal point;
scales 2 and 3 require exactly that many fractional digits. Reject signs,
leading zeros, exponents, whitespace, separators, Unicode digits, excess scale,
NaN, infinity, JSON numbers and booleans. Do not round or coerce. Canonical
patterns use a true end assertion, so an appended line terminator is invalid.

Gross is null or strictly positive. Components and allocation amounts are null
or nonnegative, including the one canonical zero at their currency's scale.
Zero gross and all negative stored magnitudes are invalid. One content-level
currency governs every interpreted amount, including allocation. Null currency
requires every interpreted monetary amount to be null. Unsupported interpreted
currencies fail admission; original unknown/unsupported lexemes can survive in
attribution with null interpretation. Never infer currency from a symbol,
locale, client timezone or upload environment.

Use checked exact minor-unit arithmetic, such as signed `i128` within the
declared bounds, or an equivalently exact bounded type. Do not use binary
floating point, locale-dependent PostgreSQL `money`, saturation or implicit
rounding. The candidate individual persistence type `numeric(20,6)` must be
qualified against this policy by the persistence owner. Report aggregates
require their own wider capacity and must not be squeezed into that candidate
individual-value column.

No quantity, percentage, exchange-rate or tax-rate arithmetic is defined by
this v1 family. Any later quantity/rate type needs separately declared precision;
a rate also needs source/date and versioned interpretation. No automatic FX
conversion or combined total across currencies is permitted.

### Complete content and components

`financial_content` has these required members, even when a value is null:
`kind`, `currency`, `gross`, `expense_date`, `source_observed_at`, `merchant`,
`components`, `original`, `business_purpose`, `business_allocation`,
`field_attribution` and `reconciliation`. The object is closed. Null preserves
an explicit unknown or absent optional value; it is not omission or numeric zero.

`components` requires `subtotal`, `tax_components`, `gratuity`,
`service_charges` and `discount`. The scalar amounts are nullable. Each list
is null or contains 0–16 closed `{label, amount}` items. Label is null or
1–128 Unicode scalars; amount is nullable and uses the enclosing currency.
Duplicate labels and identical-looking items retain their positions.
Do not deduplicate or reorder component arrays.

A null list has unknown cardinality and contributions. An empty list states
the known absence of that category, with exact zero contribution. A present
list containing a null amount has known cardinality and an unknown monetary
component. A null label does not make a known amount unknown or select a
tax classification.

Merchant and business purpose are null or 1–2,048 Unicode scalars. Purpose
must remain concise and permission-appropriate. Allocation is null or a closed
`{amount, reason}` object, where amount is nullable and reason has 1–2,048
scalars. A known allocation cannot exceed gross at confirmation. Allocation
is independent of tax treatment and reimbursement eligibility. Whole allocation
null and a present null allocation amount are both unknown monetary allocation;
only an explicit amount zero is known zero.

### Reconciliation and confirmation eligibility

The exact reconciliation equation is:

`subtotal + sum(tax_components) + gratuity + sum(service_charges) - discount = gross`.

All monetary components are known only when subtotal, gratuity and discount
are nonnull, both lists are nonnull and every listed amount is nonnull.
Empty lists contribute known zero. Labels, merchant and purpose do not enter
this predicate.

| Reconciliation state | Required meaning |
| --- | --- |
| `unresolved` | No accepted reconciliation decision; it can accompany any admitted content and cannot confirm. |
| `balanced` | Known currency and positive gross, all monetary components known, exact equality. |
| `accepted_discrepancy` | Known currency and positive gross, all monetary components known, exact inequality and an attributable nonempty reason. |
| `accepted_incomplete` | Known currency and positive gross, at least one unknown component or whole list, and an attributable nonempty reason; no equality or inequality claim. |

The three resolved predicates are disjoint. Missing currency or gross permits
only `unresolved`. Missing expense date can coexist with a resolved monetary
predicate but still prevents confirmation. Reconciliation reasons are null or
1–2,048 scalars; both accepted exceptions require the nonnull form. A machine
equation cannot manufacture a reviewed decision. Preserve source disagreement;
do not replace unknown tax with zero or edit a total to make equality hold.

Confirmation requires known reviewed gross, admitted currency, expense date,
resolved reconciliation, a valid reviewed original where required, valid
allocation and current financial authority. There is no incomplete-confirmed
class. Optional unknown components remain explicit when the accepted incomplete
policy permits confirmation.

## Field attribution and content identity

Each field binding has exactly `kind`, `target`, `basis` and `source_lexeme`.
The lexeme is required but nullable; a nonnull value has 0–1,024 Unicode scalars.
An actual empty source string is distinct from no lexeme. It remains evidence
text, not an alternate amount or source-kind override.

| Binding kind | Meaning |
| --- | --- |
| `source_derived` | Derivation from 1–16 exact immutable observation references. Preserve their actual source kinds, grounding and current use permissions; this does not claim a direct receipt quotation. |
| `command_assertion` | A value supplied by the actual authenticated command executor, with 0–16 optional observation references as context. It does not assert human authorship, receipt proof or financial approval. |

A human or agent can correct financial content directly through a revision
command without first creating an observation. Retained receipt/model references
are context when they do not state the new value. The trusted server separately
records the executor and exact correction; an agent assertion never becomes
human testimony because of a field label. Actual human confirmation remains
a separate exact-content event.

There are at most 96 bindings, 16 basis references per binding and 64 distinct
observations across the record. Reject duplicate targets and duplicate
references within a basis set. Resolve every observation's same-context and
current use authority; its organization-scoped identifier alone does not prove
domain membership. The [observation contract](observation-contract.md) owns
the immutable reference and actual document/assertion/inference meaning.

### Legal field targets

Targets are at most 64 ASCII characters and belong to this finite pointer
profile, rooted at financial content:

- Atomic units: `/kind`, `/currency`, `/gross`, `/expense_date`,
  `/source_observed_at`, `/merchant`, `/original`, `/business_purpose`,
  `/business_allocation` and `/reconciliation`.
- Scalar components: `/components/subtotal`, `/components/gratuity` and
  `/components/discount`.
- Whole-list units `/components/tax_components` and
  `/components/service_charges`, only for a null or empty list.
- Nonempty-list leaves `/components/tax_components/N/label`,
  `/components/tax_components/N/amount`,
  `/components/service_charges/N/label` and
  `/components/service_charges/N/amount`. `N` is the canonical decimal
  spelling 0–15 and must exist in that exact list.

Reject empty/root pointers, fragments, escaped or percent aliases, `-`,
leading-zero indices, unknown names, nonexistent elements, overlaps and pointers
into attribution. Original, allocation and reconciliation are atomic units;
there are no nested aliases for their fields. A whole nonempty list is not a
substitute for attribution of its items. The finite content shape has at most
77 simultaneously required units.

Each nonnull atomic/scalar unit, nonnull item label/amount and known empty list
requires exactly one binding. A null unit can retain a source lexeme or explicit
decision to unknown. Every deliberately changed unit, including one changed to
null, requires a binding. Target existence, coverage and attribution truthfulness
are semantic joins; lexical pointer validity alone does not establish them.

### Canonical financial identity

Normalize bindings by unique target in ASCII lexicographic order, including
lexical `10` before `2`. Normalize each observation basis set by canonical
organization ID and observation ID after rejecting duplicates. Preserve
component-array order and every other value. Do not trim, case-fold or normalize
Unicode. Equivalent binding/basis permutations preserve content identity;
component permutations remain consequential.

The exact preimage is the UTF-8 JCS encoding of this closed envelope:

```json
{
  "kind": "arboresce.financial-content.v1",
  "organization_id": "<exact organization ID>",
  "domain_id": "<exact domain ID>",
  "content": "<complete normalized financial_content object>"
}
```

The placeholders above describe members; `content` is an object on the wire.
SHA-256 hashes the entire canonical envelope, bounded to 131,072 bytes,
inclusive. The [shared JCS rules](data-model.md#canonical-builds) apply.
Record ID, version, digest, control, audit, reviewer, recording time and current
request purpose are outside this preimage. Exact references bind the omitted
record/revision identity separately. Server time does not replace attributed
source time.

Common idempotency fingerprints still cover the full submitted command body,
purpose and original array order. A reordered binding set can have unchanged
content identity and still conflict under the same idempotency key. A fresh
normalized identical replacement is 422, not another revision.

### Effective attribution and review snapshots

`financial_revision` contains its exact reference, complete content,
`recorded_at`, `audit_event_id` and an attribution array with one entry per
binding, sorted by target. Each attribution snapshot contains `captured_for`,
`target`, `origin` and nullable `prior_association_id`. The origin contains
`association_id`, `source_revision`, `target`, `value_sha256`, `executor`,
`authority_audit_event_id`, nullable `delegation_id`, `command_id` and
`recorded_at`. All identifiers are opaque references, not credentials.

`value_sha256` hashes JCS of the exact resolved atomic field value. A changed
unit obtains a new origin association bound to the new exact revision.
An unchanged binding/value at the same target can retain its original origin.
The server atomically binds that origin to the current `captured_for` revision
through a carry-forward association and prior ID. Preserve the complete
append-only predecessor history on the server. Export the bounded effective
association for each current binding, not recursive history.

Enforce target/value/origin/reference joins and preserve the actual original
executor. A later editor cannot replace earlier authorship merely by copying
a value. Changed/reordered units retain predecessor history and receive their
current attribution. These associations and their authority audit are outside
the financial content hash; each binds the exact resulting content, avoiding
a self-hash cycle.

A `principal_reference` has server-asserted `principal_id` and `kind`
(`human`, `agent` or `service`). It is output data, never a request authority
claim. The trusted identity owner must bind the principal correctly.
Confirmation retains the exact financial review event, expense reference,
confirmation control revision, human reviewer, actual executor, authority audit
event, nullable delegation ID, recording time and audit event. It records
who reviewed and who executed separately. Do not export raw credentials,
tokens, grants, policy bodies or unrelated identity metadata.

## Financial review and lifecycle

The public revision states are `proposed`, `confirmed`, `superseded` and
`voided`. The proposal owner initializes content 1/control 1. This contract adds no
manual-create endpoint. Each new financial content/review mutation targets the current head and compares
its expense ID, expected content version, content digest and aggregate control.
Typed counters increment without overflow or wrap; any overflow fails atomically.

| Current target | Fresh command | Result |
| --- | --- | --- |
| Proposed head | Revise | Old revision becomes superseded; new content is proposed; content and control each increment once. |
| Confirmed head | Revise | Same new proposed revision rule; old review and all old report references remain intact. |
| Proposed head | Confirm | Same immutable content becomes confirmed; control increments once and exact financial review is consumed. |
| Proposed head | Void | Same content becomes voided; control increments once; no confirmation is invented. |
| Confirmed head | Void | Same content becomes voided; control increments once and prior confirmation is retained. |
| Confirmed head | Confirm | Conflict for a fresh command. Original idempotent replay retains its prior success. |
| Voided head | Financial content/review mutation | Conflict for a fresh command; void is terminal. |
| Historical superseded revision | Financial content/review mutation | Conflict; no historical-head mutation or resurrection. |

The [lifecycle subject/control contract](lifecycle-contract.md#6-lifecycle-subjects-controls-and-commands)
permits independent restriction, hold and purge of these exact revisions without
rewriting financial content or review state. Exact expense lifecycle control is
distinct from aggregate financial control. Reports reuse the existing report
availability control row/counter, so export observes the same lifecycle counter.
All relevant writers still share the final current-authority/material fence.

A superseded revision names the exact immediate successor. Other states have
no successor. Proposed content has no confirmation; confirmed content requires
one. Superseded/voided content can retain a prior confirmation, which requires
confirmable historical content. An unconfirmed superseded or voided revision
can remain incomplete. The aggregate's observed control and a historical
confirmation's control are distinct observations.

A non-proposed state has control at least 2. Noninitial content cannot have
control 1; a successor/new revise version is at least 2. Superseded or voided
content retaining confirmation has control at least 3. Exact increments,
successor order, head equality and history joins remain semantic checks.
Never infer simultaneous order from timestamps.

Void changes current eligibility and preserves evidence/history. Correcting a
terminal void requires a separately created proposal through the later proposal
owner, not an undeclared unvoid operation. A linked refund/credit is a separate
record with kind and positive magnitude. It cannot delete or rewrite the purchase.

### Trusted financial review

`expense.confirm` requires `financial_review_event_id`, a server-issued
reference to a distinct authenticated financial decision. It is not a candidate
acceptance event, model label or boolean flag. No candidate or asset mapping
is created. The authoritative event must bind:

- Actual human reviewer and qualified identity/assurance profile.
- Exact organization/domain, financial purpose, expense ID, content version,
  content SHA-256 and decision `confirm` under its versioned review policy.
- Server recording time, an exclusive expiry, revocation state and the
  permitted executor/delegation scope.
- Durable association to the successful command and confirmation consuming it.

The configured identity policy supplies a concrete lifetime and qualified
assurance. An absent or unsupported policy cannot create usable authority.
The trusted review flow records this event before its ID is submitted. The
identity and financial-application owners must implement and qualify that flow;
these schemas introduce no public event-issuance endpoint or identity provider.

The human reviewer may execute confirmation. A separately authenticated agent
or service may execute that exact previously reviewed action only under its own
current authority and a valid current delegation chain for the action,
resource, purpose and lifetime. A general grant cannot manufacture the required
exact-content human decision. Do not lend the executor the reviewer's full
credential. Existing scoped review/delegation permits noninteractive execution
without a new prompt for each authorized attempt.

Within the short confirmation transaction, recheck reviewer/executor authority,
every delegation link, event assurance/scope/expiry/revocation, exact content and
financial predicates. Commit confirmation, one-time event consumption, audit,
result and outbox atomically. Competing valid confirmations have one committed
winner; invalid contenders have none. A fresh command cannot reuse consumed
authority. Revise and void use their own current financial actions and scoped
delegation and do not impersonate human review.

After current reauthorization, identical idempotent replay returns the original
committed result before fresh-head or unconsumed-event checks. Expiry prevents
new event consumption; it does not erase a previously recorded review.
Current permission loss still denies affected replay disclosure.
A stale precondition, invalid fresh transition or consumed/mismatched event
conflicts without a partial effect or disclosure of foreign resources.

## Dates, corrections and history

`expense_date` uses the common calendar date and preserves date-only precision.
`source_observed_at` is a separately attributed nullable source instant;
`recorded_at` is the server's UTC time. Neither upload time nor server time
fills an absent expense date. Choosing upload time as the date requires an
explicit attributable user decision. Ambiguous local instants require their
own explicit timezone interpretation; do not invent midnight for a date.

A report uses `date_basis: expense_date` and an exact `date_policy` containing
`policy_id`, tagged content `version` and `timezone` (1–128 scalars).
Resolve the actual organization policy and valid IANA zone; spelling alone
does not prove policy membership or current authority. A policy change cannot
reinterpret an immutable report.

`period` has `start_date` and `end_exclusive`. Start is a common calendar date.
The closed end union is either `{kind: "date", value: <calendar date>}` or
`{kind: "after_maximum_date"}`. The latter is the exclusive boundary immediately
after 9999-12-31 within the admitted date domain; it is not an instant or unknown
date. Require start strictly before end. Selection is half-open. A user-facing
inclusive last date advances one day; the maximum uses the terminal boundary.
Every admitted date can therefore occur in a report without an invalid year
10000 date or a manufactured timestamp.

## Immutable report selections

`expense_report.create` compiles a complete bounded manifest for one
organization/domain/purpose and date-policy/period scope. The request explicitly
contains `include_undated`; client default is true, not an omitted server guess.
The server issues a new report ID only on a successful compile; the durable
idempotency key identifies retry intent.

Consider exactly one current head per expense ID that the compiling principal
may currently know and whose relevant financial metadata they may disclose.
Its date must be in the period, or null with `include_undated: true`.
Historical superseded revisions and known dates outside the period are not
enumerated as exclusions. With false, undated heads are explicitly out of
scope, without an existence count. The manifest states these limits; it is
not a completeness claim about every record in the organization. Its audit
binds the actual compiling principal and authority to the exact manifest.

Select only confirmed, nonvoided, reconciled, currently authorized and eligible
heads. For each remaining considered head, record one exclusion with exact
reference, observed control, state and this deterministic primary-reason order:

1. `unassigned_date` for null date when included.
2. `voided`.
3. `unresolved` for a proposed head missing required financial inputs or
   having unresolved reconciliation.
4. `proposed` for another proposed head.
5. `ineligible` for an otherwise visible confirmed head that cannot be selected.

An inaccessible expense is omitted completely. A reason must not expose a
hidden source, identity, cause or count. Permission to disclose a retained
financial identity is distinct from permission to use all evidence for selection.

At most 256 selected references and 256 authorized exclusions are permitted.
An authorized 257th item in either set fails the whole compilation. Do not
truncate totals or traverse a moving paginated queue to claim a snapshot.
Sort both sets by canonical expense ID, then exact content version and digest;
sets are disjoint. An empty selection is valid, retaining its declared scope
and visible exclusions with empty totals, not invented zero currency entries.

`report_manifest` requires profile `financial-report-v1`, common command
context, date basis, exact date policy and period, `include_undated`,
`selection_cutoff`, calculation version `financial-calculation-v1`,
`selected`, `exclusions` and `totals`. A selected item contains the exact
expense reference, observed control revision and retained confirmation.
A report reference binds organization/domain, server-issued report ID and
content SHA-256. A report snapshot contains that reference and the immutable
manifest; there is no report content-revision counter.

The report preimage is JCS of the closed object
`{kind: "arboresce.financial-report.v1", manifest: <complete normalized manifest>}`.
SHA-256 covers those bytes. Report ID, its digest, current availability control
and transport receipts remain outside the preimage. Selection-time expense
controls and confirmations remain inside the manifest as recorded. New
selection/calculation requires a new report; later record correction, void or
current policy change cannot rewrite an earlier manifest.

### Exact report arithmetic and incomplete buckets

Each represented currency has one totals entry, sorted by currency code, with
its exact `record_count` (1–256) and buckets `gross`, `subtotal`, `tax`,
`gratuity`, `service_charges`, `discount` and `business_allocation`.
There are at most eight entries. Exclusions contribute nothing to accumulators.
Apply kind sign exactly once: +1 for expense, -1 for refund/credit.
Discount is retained as a separately signed component and is subtracted in
the reconciliation equation; do not apply a second refund sign.

Aggregate strings have at most 18 integer digits, exact currency scale,
an optional minus for a nonzero negative amount and one nonnegative zero.
Use checked signed intermediates for every operation, including formatting,
discount subtraction and comparison. Do not share an accumulator across
currencies or assume component sums are bounded by gross.

| KWD capacity | Maximum magnitude |
| --- | --- |
| One gross/component | `99999999999999.999` |
| 256 gross/scalar components | `25599999999999999.744` |
| 256 × 16 tax or charge components | `409599999999999995.904` |
| Positive equation intermediates, 256 × (1 + 16 + 1 + 16) | `870399999999999991.296` |

Count/scale predicates impose tighter semantic bounds than the general
aggregate-string grammar; boundary fixtures must preserve the exact capacities.

Each bucket carries `known_net`, `complete`, `unknown_record_count` (U),
`unknown_list_record_count` (L) and `known_null_component_count` (K).
For N selected records of that currency:

- Scalar bucket: L = 0 and U = K, with 0 ≤ U ≤ N ≤ 256.
- List bucket: 0 ≤ L ≤ U ≤ N. With P = U − L, require P ≤ K ≤ 16P ≤ 4096.
  Null whole lists contribute to L and U; they do not invent listed-item counts.
- `complete` is true exactly when U = L = K = 0.
- `known_net` sums the signed present amounts. If none are known, it is canonical
  zero with incomplete status, never a fabricated complete total.

Null labels do not affect monetary completeness. Empty lists are known zero.
Gross is a complete scalar bucket for every selected record. Whole allocation
null or its amount null contributes one unknown scalar; explicit zero is known.
Partial known amounts that cancel to zero remain incomplete when unknowns exist.

### Compilation and permission fences

A successful compile returns 200 only after selection, totals, audit, result and
outbox commit atomically. Use canonical database state, not projections or
cached roles. All relevant writers follow the common deterministic lock order.

The bounded implementation uses a monotonic organization authority epoch and
domain financial-scope epoch. Proposal insertion, correction, confirmation and
void lock/advance the domain fence. Membership, delegation, relevant
purpose/date policy and required source permission/availability changes
lock/advance the authority fence. No epoch can wrap, reset or be reused.
A cross-domain authority change cannot depend on updating every expense row.

For an existing command, first apply current authorization and common
fingerprint/replay rules. A prior result is not subjected to new snapshot
cardinality or arithmetic checks. Fresh compilation reads both epochs,
enumerates the bounded authorized head universe and reads epochs again.
Changing epochs makes the preparation stale. Resolve exact financial bytes,
source metadata and arithmetic outside database locks under current access.

The final short transaction locks the two fences and required current
authority/idempotency rows. If the identical command committed during
preparation, return its authorized recorded result before fresh-state checks.
For fresh work, reauthorize, compare exact epochs, policy and head/control
identities, then re-enumerate bounded canonical metadata at the final
authoritative instant. Require exact prepared/current universe equality.
This also handles time-based grant activation/expiry without a database write.

Assign the final cutoff, assemble the bounded final manifest and compute its
JCS/hash in this fenced transaction. No network, parsing or byte stream runs
under these locks. Referenced bytes/arithmetic were prepared beforehand.
Relevant writers cannot pass the fences until commit. Changed state is 409;
there is no hidden unbounded retry. Enforce report identity uniqueness without
overwriting another manifest. Metadata-only database work and final bounded
serialization need their own runtime capacity qualification.

Subsequent report reads and exports recheck current permissions. Ordinary
correction/void preserves historical report content; current revocation or
missing required content can deny new disclosure. Immutable identity cannot
retain a permission that has been revoked.

## Complete commands and reads

All five commands have closed `{metadata, body}` envelopes. Common metadata
supplies exact v1 command name, organization/domain/purpose and idempotency.
Each purpose must be enabled for its financial action by the configured
same-domain policy; unconfigured or unsupported purpose denies admission.
Body/route IDs must match and refer to the metadata scope. Clients cannot
supply a new resource ID, actor, approval flag, fingerprint or server timestamp.

| Command and HTTP route | Required closed body | Successful result |
| --- | --- | --- |
| `expense.revise` — POST `/v1/expenses/{id}/revisions` | `expense_id`, `expected_version`, `expected_content_sha256`, `expected_control_revision`, complete `replacement`, `reason` | 200; `{command, expense: <new proposed snapshot>, superseded: <old exact reference>}` |
| `expense.confirm` — POST `/v1/expenses/{id}/confirm` | `expense_id`, `expected_version`, `expected_content_sha256`, `expected_control_revision`, `financial_review_event_id` | 200; `{command, expense: <confirmed snapshot>}` |
| `expense.void` — POST `/v1/expenses/{id}/void` | `expense_id`, `expected_version`, `expected_content_sha256`, `expected_control_revision`, `reason` | 200; `{command, expense: <voided snapshot>}` |
| `expense_report.create` — POST `/v1/expense-reports` | `date_policy`, `period`, `include_undated` | 200; `{command, report: <report snapshot>, control_revision}` |
| `expense_report.export` — POST `/v1/expense-reports/{id}/exports` | `report` (exact report reference), `expected_control_revision`, `format` | 202; common accepted operation and pending receipt |

The 200 results use the common success envelope and terminal idempotency receipt.
Revise/void reasons are 1–2,048 scalars. Format is exactly `financial-json-v1`
or `financial-csv-v1`. Export 202 means durable admission, not completed output.
The [sealed financial export contract](lifecycle-contract.md#10-sealed-financial-export-and-bounded-delivery)
owns the exact financial_export_result, artifact core/reference/resource,
cancellation, fixed expiry, observed availability and complete bounded delivery.
These financial request, payload and format semantics remain fixed.

GET `/v1/expenses/{id}` returns a current expense snapshot. GET
`/v1/expenses/{id}/revisions/{version}` resolves exact historical content and
its observed lifecycle/control state. GET `/v1/expense-reports/{id}` returns
`{report: <immutable report snapshot>, control_revision}` in the common success
envelope without a command receipt. Report availability control starts at 1
and supplies the export precondition. A create replay retains its originally
observed control; current GET supplies current control. This control stays
outside report/export payload identity; its transitions belong to lifecycle.
Every current or exact historical financial GET returns the complete required
snapshot or an error under current disclosure. Retention loss never becomes a
null/truncated immutable snapshot or a substituted current revision. Known
authorized missing retained details conflict; required unavailable policy,
infrastructure or continuity fails with 503, and undisclosable resources remain
404. The lifecycle owner defines exact selectors and complete-or-error transport.

Canonical content, references, snapshots and export domain values are closed.
Outward resources and success results use the existing bounded additive
transport policy and reject contradictory outcome/actor/approval aliases.
Unknown response extensions cannot enter content hashes or exports. The
[API owner](api.md) and [common contract](common-contract.md) retain errors,
reauthorization, raw JSON and idempotency policy. A stale expected value or
invalid fresh transition is 409; normalized no-op replacement is 422.
Apply disclosure-safe errors to inaccessible references before revealing state.
Unavailable required authority infrastructure never authorizes an effect.

The common command/control profile remains 1 MiB UTF-8, at most 16 nested
containers, 128 members per general object and 256 elements per general array.
Financial content has its independent 128 KiB canonical-preimage cap.
The wider export limits below apply only to complete versioned export streams.
Every actual encoded-byte cap is enforced independently; shape validity does
not prove that a maximal combination fits.

## Export formats and privacy

A financial export must be independently interpretable without an engine
database, another checkout or proprietary importer. Both formats preserve one
complete frozen financial payload; JSON preserves canonical values and CSV
provides a reversible spreadsheet-oriented representation. They do not infer
tax treatment or introduce executable content.

The closed payload has exactly `profile`, `report`, `records` and
`source_availability`. Profile is `financial-export-payload-v1`. Report contains
its exact identity and full immutable manifest. Records contain every selected
exact immutable financial revision in selection order, each paired with its
selected reference/control/confirmation. Preserve complete content and bounded
effective attribution, including original executor, assertions, source lexemes,
original links and reconciliation. Never substitute a later head/review/control.

Accountant defaults include permitted financial fields and concise purpose.
They exclude raw receipts/audio, whole notes or observations, unrelated
participants, location metadata, credentials and raw grants/policies.
A linked source is not blanket authority to export it. Narrow financial
lexemes and references retain their own permission checks.

### Frozen source availability

`source_availability` has server UTC `observed_at` and `observations` sorted
by canonical observation organization/ID. Include each directly referenced
observation exactly once: at most 64 per record and 16,384 per export.
An entry retains the exact observation reference, trusted nullable
`observation_kind`, `observation_state`, `sources_state` and `sources`.

Kind is `document_observation`, `user_assertion` or `model_inference`, or null
only when that retained descriptor is unavailable. Available observations must
have a known kind. A client/provider label cannot upgrade an inference or
manufacture the absent descriptor. Observation state is `available`,
`unavailable` or `purged`. These describe availability, not authenticity or
permission.

Sources enumerate only direct originals from that observation's source and
grounding locators, including an audio locator's optional transcript original.
Use the existing acquisition original-reference type with each status.
Do not recursively expand inference bases. An entry has at most 33 originals:
one direct source plus at most two originals for each of 16 locators.
This is a conservative structural bound, not proof all combinations are valid.

Deduplicate originals within an observation by organization/artifact/version
identity and sort that tuple in ASCII order. Equal identities with contradictory
metadata fail. Inline assertions with no originals have an honest empty array.
`sources_state: complete` means the array is the full retained association;
`unavailable` requires an empty array and makes no claim that sources were absent.
A purged observation with a retained known-empty association may be complete/empty.
An available observation requires complete association. Never infer references
from another resource's current head.

Every included financial record, reference, descriptor and status must currently
be disclosable. Permission loss fails the whole export, not a missing-source
marker or silently deleted row. Authorized unavailable/purged source bytes
can be marked without fetching them under another identity. Permission to know
a retained reference differs from permission to retrieve its bytes.
Unavailable required historical financial content prevents lossless export;
preserve the report and fail that export under lifecycle availability.

### Snapshot identity and delivery

The formats encode the same frozen payload. Its identity is SHA-256 of exact
`financial-json-v1` JCS bytes and is outside those bytes. A new export can
observe a different availability/time snapshot without changing report identity.
The operation binds exact report, payload identity, format/profile and emitted
byte length/SHA-256. CSV has its own emitted-byte digest. The artifact digest
is outside its own bytes. None of these digests is an artifact signature.

After current reauthorization, same-key replay returns the original
operation/result and snapshot. It neither re-observes sources nor regenerates
changed bytes. Expired/deleted/unavailable artifacts have explicit lifecycle
outcomes; an old successful receipt cannot hide new generation. Current
authority fences every later read/download and new external exposure.

Prepare bytes outside database locks. Stage privately, enforce all limits,
validate the full encoding and recheck disclosure before committing a
downloadable artifact. Oversize, cancellation, interruption or authorization
loss cannot produce a falsely completed artifact or silently partial output.
An already admitted bounded delivery follows the security owner's grant rules.
Client installation/export rules own exact destination validation, atomic
protected writes and preservation of existing files/unsent edits.

The [delivery owner](lifecycle-contract.md#10-sealed-financial-export-and-bounded-delivery)
requires sealing the full artifact before binding, distinct payload/emitted/core
digests, fixed exclusive expiry and a finite selected generation/staging/transfer
profile with shared slots. Content uses a fresh current-authorized read-only POST
with expected artifact control, exact byte/header agreement and no Range/resume,
redirect, compression, presigned URL or multipart variant. Its retained delivery
receipt proves admission only. After response bytes begin, an error aborts the
transport; it never appends JSON or claims completion. E096 must qualify actual
full 64 MiB transfer, lost-response replay, expiry/revocation and safe client
length/digest verification. Financial delivery does not define complete customer
exit; that archive and restoration capability remains separately required.

### JSON profile

`financial-json-v1` serializes the complete payload with shared JCS Unicode
and number rules: UTF-8, no BOM, no trailing newline and no insignificant
whitespace. Object members use JCS ordering; arrays retain frozen domain order.
Money and revision values remain strings; protocol counters use canonical
unsigned integer tokens. Never pass amounts through binary float.

Limit the emitted stream to 67,108,864 bytes inclusive. Limit nested container
depth to 32 with root container depth 1 and total value/container nodes to
1,048,576, counting each once. Each object is a declared closed domain object;
arrays use their own bounds, including the 16,384 availability entries.
These limits do not import common 256-element control-array limits into export.
They also do not promise every combination of individually maximum values fits.
Reject overflow without omission, truncation or altered unknowns.

### CSV profile and cells

`financial-csv-v1` uses UTF-8 without BOM, comma separation, exactly one header
and CRLF record terminators including the final row. Quote every header and
data cell with ASCII double quotes; double embedded quotes. Emit no comment,
`sep=` directive, preamble, formula, macro or executable content.
[RFC 4180](https://www.rfc-editor.org/rfc/rfc4180) supplies Informational CSV
conventions; these exact choices define this project profile.

Every data cell begins with the literal five ASCII bytes `text:` and a typed
discriminator. This applies equally to financial numbers, IDs, dates, counts,
flags, formula-like text and empty values:

| Meaning | Exact cell after CSV quote decoding |
| --- | --- |
| String | `text:s:` followed by reversible escaped text. |
| Null | Exactly `text:n:`. |
| Boolean | Exactly `text:b:true` or `text:b:false`. |
| Unsigned integer | `text:i:` followed by canonical nonnegative decimal digits. |
| Inapplicable column | Exactly `text:x:`; this is not a JSON value. |

Empty string is `text:s:`, string `null` is `text:s:null`, unknown is
`text:n:`, zero money can be `text:s:0.00` and integer zero is `text:i:0`.
A literal string `text:n:` becomes `text:s:text:n:`. Amounts, IDs, revisions,
digests and dates always use strings; counts and ordinals use integers.

Before CSV quoting, encode strings once, scalar by scalar:

1. Backslash U+005C becomes two backslashes `\\`.
2. C0 U+0000–U+001F, DEL U+007F, C1 U+0080–U+009F and U+2028/U+2029 become
   a backslash, lowercase `u` and exactly four uppercase hexadecimal digits.
   LF is `\u000A`; there are no short `\n`, `\r` or `\t` escapes.
3. Other valid Unicode scalars remain unchanged. Do not normalize, trim or
   case-fold. Double quotes are handled by CSV quoting afterward.

Decode CSV quotes, validate the exact token, then decode the string escape
grammar once. Reject dangling/unknown escapes, lowercase hex, escaped ordinary
characters, raw forbidden controls, noncanonical integers, malformed UTF-8 and
unpaired surrogates. A literal backslash followed by `u000A` is encoded with
a doubled backslash and restores those six characters, not an LF.
Do not recursively decode or strip `text:` inside a spreadsheet cell.

Each decoded cell, including the token and escaped payload, is limited to
32,760 UTF-16 code units and 65,536 UTF-8 bytes. CSV framing/doubled quotes are
outside the cell bound but inside the same 64 MiB stream bound. An oversized
cell fails CSV explicitly; it is not split, shortened or silently converted.
A separately requested JSON export may still be valid.

This prefix is a deliberate exact-text encoding, not a universal spreadsheet
certification. [Microsoft's limits](https://support.microsoft.com/en-us/office/excel-specifications-and-limits-1672b34d-7043-467e-8e27-269d656771c3)
and [precision/import guidance](https://support.microsoft.com/en-us/office/keeping-leading-zeros-and-large-numbers-1bf7b935-36e1-4985-842f-5dfa51f85fe7)
explain why cell capacity and automatic number interpretation need explicit
qualification. [OWASP's CSV injection guidance](https://owasp.org/www-community/attacks/CSV_Injection)
describes the active-content hazard. Quoting alone neither neutralizes formulas
nor preserves long decimal/ID text under spreadsheet inference.

### CSV header and row families

The fixed 24-column header order is:

```text
row_kind,report_id,report_sha256,record_id,content_revision,content_sha256,observed_control_revision,record_kind,currency,gross,expense_date,merchant,business_purpose,reconciliation_state,exclusion_reason,bucket,known_net,complete,unknown_record_count,unknown_list_record_count,known_null_component_count,path,value_type,value
```

Physically quote each header name and terminate it with CRLF. Every row has
exactly 24 cells and repeats exact report ID/digest. Unassigned columns use
`text:x:`; an assigned unknown value uses `text:n:`. No cell contains an
opaque nested JSON object/array string.

| Row kind | Required projection |
| --- | --- |
| `report` | Exactly one first data row: report identity, `value_type` string `string` and `value` string `financial-csv-v1`. Other columns are inapplicable. Mandatory field rows carry complete report metadata. |
| `record` | One per selected record in selection order: exact record/content/control identity, kind, currency, gross, expense date, merchant, business purpose and reconciliation state. Nullable merchant/purpose use null tokens. |
| `summary` | One per represented currency and bucket, in currency-code order then `gross`, `subtotal`, `tax`, `gratuity`, `service_charges`, `discount`, `business_allocation` order. Include exact known net, completeness and three unknown counters. |
| `exclusion` | One per frozen authorized exclusion: only retained exact record/reference/control identities and reason. Never fetch excluded content to populate overview fields. |
| `field` | One typed node of the complete frozen payload, including policy/period/cutoff, selections, components, assertions, effective attribution, review, original links and availability. |

File order is report, records, summaries, exclusions, then all field rows.
Overview rows are deterministic projections verified against field rows, not
another data authority. Do not sum overviews and summaries as extra transactions.
An empty selection has no invented currency summary.

### Typed field rows and independent reconstruction

Emit one field row for every payload node, including the root, containers and
nulls. `path` is the canonical RFC 6901 pointer within this export tree:
empty at root, `~` escaped to `~0` and `/` to `~1`; array indices use
canonical nonnegative decimals. This is not an engine-resource selector.
`value_type` is exactly `object`, `array`, `string`, `integer`,
`boolean` or `null`.

Scalar `value` uses the corresponding typed token. A container uses an
unsigned integer value giving its exact immediate member/element count.
Zero-member object, empty array and null therefore remain distinct. Use
deterministic preorder: parent before children, JCS member order for objects
and increasing index for arrays. Other field-row columns, except row kind,
report identity, path, value type and value, are inapplicable.

The decoder reconstructs exactly one root, rejecting duplicate paths, missing
parents, wrong types, wrong counts, noncontiguous indices, repeated keys,
unknown row kinds/columns/paths and inconsistent overview values. It validates
the financial payload schema and all deterministic projection/order rules.
The same depth/node/cell/stream bounds apply independently. CSV can exceed
its byte cap before a JSON-valid payload is fully represented.

An independent consumer needs ordinary CSV and this typed-row convention,
not an engine database, archive, proprietary importer, JSON-in-cell parser or
spreadsheet formula. A spreadsheet user can filter record or summary rows.
The visible prefix preserves text rather than automatically enabling numeric
spreadsheet calculation. Removing it requires an explicit independent import
that preserves exact strings and does not activate untrusted content.

## Duplicate detection and professional limits

Checksums establish byte consistency, not document authenticity. Content
equality, repeated fields and suspicious similarity can suggest duplicates for
review; they cannot silently merge transactions or prove fraud. Idempotency
prevents repeating one command, not duplicate business transactions.

MVP is not an accounting ledger, payment processor or tax-filing system.
It does not determine deductibility, recoverable tax, tax rates, reimbursement
eligibility or filing obligations. Later regional treatment requires a reviewed,
effective-dated jurisdiction/entity-specific contract and implementation.
Do not infer jurisdiction from IP address, language, timezone or repository
location, or advise destruction of original evidence. Actual retention,
residency and hold obligations remain with selected [deployment](deployment.md),
[security](security-and-privacy.md) and [recovery](recovery.md) policies.

## Required verification

The finite [case manifest](../../contracts/validation/financial-records-v1.cases.json)
and [expectation schema](../../contracts/validation/financial-records-v1.expectations.schema.json)
use the [exact-family entry](../../contracts/financial-records-v1.fixtures.schema.json).
Parsed checks validate declared shapes and static contradictions. They do not
execute arithmetic, raw admission, canonicalization, source joins, identity,
transactional races, export encoding or spreadsheet behavior. Actual results
belong in checkpoint evidence; no case count or financial/runtime pass is
asserted by this document.

The following independent semantic scenarios are required and **not executed
qualification results**. Their later domain, persistence, application,
identity, contract and CLI suites must retain independent inputs and oracles.

| ID | Required scenario and outcome |
| --- | --- |
| FIN-SEM-01 | Every admitted currency and payable scale; gross/component/aggregate maximum and plus one; signs, zero/negative zero, leading zeros, exponents, Unicode digits, separators and line terminators. No coercion or rounding. |
| FIN-SEM-02 | Raw strings versus numbers/booleans, duplicate keys, malformed UTF-8/surrogates, raw unsigned-counter tokens, body/depth/collection limits and encoded-byte maxima/plus one. Parsed mathematical integers do not certify lexical admission. |
| FIN-SEM-03 | Null versus empty lists, null labels with known amounts, listed-null amounts, duplicate labels and position preservation. No invented cardinality or unknown-to-zero conversion. |
| FIN-SEM-04 | Disjoint balanced/discrepant/incomplete predicates; unresolved remains unresolved even when arithmetic balances; missing date/currency/gross and required original block confirmation independently. |
| FIN-SEM-05 | Exact `40.00 + 2.00 + 8.00 = 50.00`; separate unknown-tax case must not substitute zero. Preserve conflicting source totals and attributable exception reasons. |
| FIN-SEM-06 | Expense/refund/credit positive magnitudes and sign exactly once; historically confirmed original, wrong kind/tenant/currency, self/absent link and later original supersession/void. No implicit refund cap or purchase deletion. |
| FIN-SEM-07 | Exact individual/aggregate/intermediate maxima, checked overflow, separate CAD/USD totals, negative nets and known sums cancelling while incomplete. |
| FIN-SEM-08 | Scalar/list U/L/K equations, whole-null allocation versus null amount versus known zero, unknown lists versus known null components and no fabricated complete buckets. |
| FIN-SEM-09 | Every legal field path and bad alias/index/overlap, attribution coverage, changed-to-null, empty source lexeme versus no lexeme, 96/97 bindings, 16/17 per-basis and 64/65 distinct observations. |
| FIN-SEM-10 | Source-derived document/assertion/inference distinctions and direct human/agent command assertions without observation creation. False provider/source labels never upgrade evidence or review. |
| FIN-SEM-11 | Independently recompute complete content preimage/hash; each field, source lexeme and component-order change affects identity. Binding/basis set permutations preserve identity but same-key submitted-order changes conflict. |
| FIN-SEM-12 | Exact organization/domain isolation, cross-language JCS, key ordering, Unicode and 128 KiB preimage boundary. No-op normalized revision is 422 without counters/audit effects. |
| FIN-SEM-13 | Effective attribution carry-forward preserves original executor/source revision and exact current capture; changed value/target gets new association. Reject forged actor, wrong value digest and missing predecessor joins. No recursive export history. |
| FIN-SEM-14 | Every lifecycle transition, terminal void, historical-head denial, typed counter overflow and statically impossible initial counters. Retain old confirmations and report references. |
| FIN-SEM-15 | Financial review event binds exact content, human, assurance, purpose and scoped executor; forged/wrong/expired/revoked events or service impersonation fail. Prior valid scoped approval needs no invented repeated prompt. |
| FIN-SEM-16 | Concurrent valid confirmations yield one committed confirmation/event consumption; mixed correction/void/confirm contenders yield one permitted transition, possibly no confirmation. All unauthorized contenders yield none. |
| FIN-SEM-17 | Lost-response retry, same-key changed body, later head/event expiry and current permission loss. Authorized replay returns the original snapshot without consuming again; expiry cannot silently repeat a financial effect. |
| FIN-SEM-18 | Date-only/source/server time separation, leap/impossible/ambiguous dates, inclusive UI translation, half-open ordering and maximum-date terminal end. Never manufacture midnight/upload dates. |
| FIN-SEM-19 | Exact current-head report universe; known dates outside period, undated true/false, proposed/unresolved/voided/ineligible reason precedence and no historical superseded rows. No hidden existence/count disclosure. |
| FIN-SEM-20 | 256/257 selected and excluded boundaries, disjoint sorted identities, empty selection/totals, no truncation or moving-queue snapshot substitution. |
| FIN-SEM-21 | Concurrent insertion, correction/date change, confirmation, void, source/policy revocation and timed grant activation during prefetch. Epoch and final metadata fences reject stale universes without byte/network work under locks. |
| FIN-SEM-22 | Final cutoff/manifest/hash commit atomically; exact selection-time reviews/controls survive later changes. New selection/calculation yields a new report, never an in-place historical rewrite. |
| FIN-SEM-23 | Server-issued report identity, initial/current availability control, old create replay and stale export preconditions; report control never enters immutable report/payload identity. |
| FIN-SEM-24 | Authorized unavailable/purged sources, known-empty versus unavailable association, retained kind absence, exact direct-original deduplication and 33/34-source bounds. Never invent an original or recursively expand inference bases. |
| FIN-SEM-25 | Inaccessible reference/status or missing required historical financial content fails full export. Do not silently omit rows or mislabel permission denial as byte absence. |
| FIN-SEM-26 | Frozen payload identity versus separate JSON/CSV byte digests; new availability snapshot only for new intent, same-key replay never regenerates, and stale/deleted artifacts follow lifecycle outcomes. |
| FIN-SEM-27 | Independent JSON and CSV encoding/decoding: reconstruct every scalar/container/null, complete source attribution, policy, selections and original links. JCS of CSV reconstruction equals JSON bytes. |
| FIN-SEM-28 | Formula prefixes `=`, `+`, `-`, `@`; whitespace, tabs/CR/LF/NUL/C1, quotes/commas/backslashes, literal `\u000A`, literal `text:`, long digits/dates and astral Unicode. Exact one-pass escape and typed-token decoding. |
| FIN-SEM-29 | Missing/duplicate/reordered tree rows, wrong counts/parents/indices/types, malformed tokens/escapes and inconsistent overviews fail. Null, empty, zero and inapplicable remain distinct. |
| FIN-SEM-30 | Cell UTF-16/UTF-8 maxima and plus one; container/node and independent 64 MiB format caps; interruption/revocation/cancellation during generation/download. No completed partial artifact, hidden fallback or overwritten consumer file. |
| FIN-SEM-31 | Qualify named spreadsheet versions/import modes through actual import, save/reopen and independently recovered-value comparison. Encoded files must not execute formulas; no universal spreadsheet-safety claim from quoting or a schema pass. |
| FIN-SEM-32 | Actual receipt/note → proposal → correction → confirmation → report → independent export journey with current permissions, without knowledge approval or a paid entitlement. |

[Testing and coverage](testing-and-coverage.md) owns deterministic independent
fixtures, full suite/process inventory and strict applicable source coverage.
[Acceptance](acceptance.md) owns the 250 monetary scenarios and broader
experiments/gates. None is replaced by parsed shapes or a simulated service.
