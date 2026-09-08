# Financial records and reports

Status: accepted normative requirements; expense, report and export behavior
is not implemented. This document owns evidence-linked recordkeeping semantics,
exact money, confirmation, period selections and financial exports. Exact wire
types are frozen by the financial contract step in the
[implementation plan](../execution/open-cli-mvp-v1-rcld.md).

The required workflow is receipt and note, extraction proposal, user correction,
confirmed record, period report and independently usable export. It does not
require knowledge publication. [Product scope](product-and-scope.md) owns the
broader feature boundary; [data model](data-model.md) owns shared identities.

## Evidence and record lineage

Original receipts, photographs and notes are evidence. Extraction creates a
proposal, not a confirmed business transaction. Keep document-derived fields,
attributed user assertions and model inferences distinct. Business purpose,
participants, allocation and reported meeting outcomes are not facts proved by
a receipt. They are optional contextual fields with their own access rules.

An expense aggregate has organization-scoped identity and independent content
and control revisions. Each immutable expense revision retains exact evidence
versions, field sources, date source/precision, currency, components, reviewed
total, reconciliation decisions, any business allocation and attributable review.
Corrections compare expected revision and create new content; currency changes
cannot silently reinterpret an old revision.

States are proposed, confirmed, superseded and voided. Confirmation requires
current authority, the exact reviewed revision and the validation below.
Competing confirmations have one winner. A model or recorded provider cannot
confirm a record. Idempotent retries preserve the original command result under
the [API admission rules](api.md#idempotent-command-admission).

## Exact monetary values

Represent wire amounts as bounded decimal strings with explicit ISO currency.
Use exact arithmetic internally, with reviewed range and scale checks before
persistence. The candidate PostgreSQL representation is `numeric(20,6)`;
the persistence contract must validate that choice against the supported
currency/quantity policy before qualification. Exact scaled integers are also
valid domain representations within the declared range.

Do not use binary floating point, locale-dependent PostgreSQL `money`, or
implicit rounding for amounts or totals. Reject NaN, infinity, exponent forms,
ambiguous separators, excess scale and overflow at the command boundary.
The supported currency policy defines payable scale. Quantities and rates have
separately declared precision; any rate carries its source/date and versioned
type. Do not infer currency from a bare symbol, locale or client timezone.

Store subtotal, individual tax components, gratuity, service charges, discount
and gross total separately. Unknown is different from zero. Preserve missing
values and disagreement between source fields. Reconciliation classifies the
discrepancy and records the reviewed decision; it cannot fill an unknown
component with zero or alter a total merely to make an equation balance.

Missing reviewed total, currency or expense date keeps the record **proposed**.
Required reconciliation decisions must also be resolved before confirmation.
There is no alternate incomplete-confirmed class in MVP. Optional unknown
components can remain unknown when the reviewed reconciliation policy permits
confirmation; report them accurately instead of implying a complete breakdown.

Money uses a positive magnitude with explicit kind `expense`, `refund` or
`credit`. Refund/credit records require a reviewed link to the original record.
The calculation contract applies the kind's sign exactly once. Never mix this
with negative stored refund amounts or delete the purchase to represent its
refund. Zero and negative values require explicit rejection cases under this
positive-magnitude convention; component-specific zero values remain distinct
from missing components.

Business allocation is independent of tax treatment and reimbursement
eligibility. MVP performs no automatic currency conversion and emits no combined
total across currencies. Preserve the original amount/currency and calculate
separate totals per currency using the exact selected revisions.

## Dates, corrections and history

Preserve an expense date as a date when that is the source precision. Keep any
source-observed instant and server recording time separately. Upload time may
be chosen as the date only by an explicit attributed user decision; it is not
a default inference for a missing date.

Period selection names a date basis, organization timezone or explicit date
policy, and half-open endpoints. Document how those endpoints are displayed
to users, including an inclusive-looking date range. Test timezone and period
boundaries without inventing midnight timestamps for date-only records.

Void changes current eligibility and remains auditable. A linked refund/credit
is a separate revisioned record. Superseded corrections retain the old review,
evidence and report references. Neither a correction, refund nor void rewrites
the bytes of an already compiled report.

## Immutable report selections

Default selection includes confirmed, non-voided, currently authorized and
eligible expense revisions. Proposed, unresolved, revoked, superseded or voided
records must not silently enter default totals. The selection operation freezes
the exact record/revision set, date basis, period, timezone/date policy,
selection cutoff and calculation-version identifier.

Report per-currency totals, known components and the authorized exclusions or
unresolved records needed to explain the selection. Include visible exclusion
identities/reasons and counts without revealing any inaccessible record's
existence. Missing optional component totals must remain explicitly incomplete.
Do not claim that absent tax is zero tax or that an omitted record was absent
from the organization.

Selection order must not change set-based arithmetic. A changed record or
selection/calculation version requires a new report, not an in-place export
update. Current authorization still controls later report access and export;
immutable report identity cannot preserve permission that has been revoked.

## Export formats and privacy

Provide a lossless structured export for programmatic interpretation and a
separately specified spreadsheet-oriented CSV. Preserve exact amounts,
currencies, dates, revisions, field attribution, source references,
reconciliation and authorized exclusion details. The CSV policy must neutralize
formula-like text and document its transformation without changing canonical
stored values. Neither format may introduce macros or active content.

Accountant-facing defaults include permitted financial fields and concise
purpose. They do not include raw meeting audio, unrelated participant details,
location metadata or whole notes merely because those sources are linked.
Permission-check source references and use bounded streaming/output rules.
Source bytes that are unavailable or purged are marked as such; exports never
fabricate them. [Security and privacy](security-and-privacy.md) owns general
customer export and disclosure policy.

The export must be interpretable by an independent consumer without an engine
database or proprietary tooling. A safe delivery preserves existing user files
and reports actual transfer completion; client filesystem details belong to
the supported client's contract.

## Duplicate detection and professional limits

Content equality, suspicious similarity and repeated business fields can
propose possible duplicates for review. They cannot silently merge transactions
or prove fraud. A checksum establishes byte consistency, not document
authenticity. Idempotency prevents repeated execution of one command and does
not decide whether two business transactions are the same.

MVP is not an accounting ledger, payment processor or tax-filing system.
It does not determine deductibility, recoverable tax, tax rates, reimbursement
eligibility or filing obligations. Such behavior requires a separately reviewed,
effective-dated jurisdiction/entity-specific contract and implementation.
Do not infer jurisdiction from IP address, language, timezone or repository
location. Do not advise destruction of original evidence. Actual retention,
residency and hold obligations belong to the selected [deployment](deployment.md)
and [recovery](recovery.md) policies.

## Required verification

Independently reviewed synthetic oracles must include exact component arithmetic,
unknown tax, missing required dates/currencies/totals, disagreement, ambiguous
decimal input, currency scale, overflow, negative/zero magnitude rejection,
refund/credit sign application, original links, voids, concurrent corrections,
period boundaries, authorized exclusions and formula-injection text.

For example, known components `40.00 + 2.00 + 8.00 = 50.00` have an exact result;
that does not authorize substituting `0.00` for unknown tax in another fixture.
CAD and USD selections stay separate. Verify actual receipt-to-export behavior
without knowledge approval once the runtime exists. [Testing and coverage](testing-and-coverage.md)
owns fixture independence and numerical source gates, while
[acceptance](acceptance.md) owns the full monetary qualification corpus and
experiment requirements. No fixture collection or financial pass is claimed here.
