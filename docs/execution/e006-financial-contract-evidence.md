# Financial contract checkpoint evidence

Date: 2026-09-08. The starting source revision is
`147709770919e098ed50a60c2a0cf626e5dc63ae`, which records the completed
[candidate review prerequisite](e005-candidate-review-contract-evidence.md).
This checkpoint defines parsed financial contracts; it does not implement
expense processing, financial arithmetic, an HTTP service or export generation.

The [financial owner](../requirements/financial-records.md) defines monetary
meaning, attributable review, immutable reports and the two export profiles.
The [validator guide](../../contracts/validation/README.md) owns the exact
standalone commands and pinned isolated environment. The library and its
exact-family entry use the existing local reference graph without changing
earlier library definitions or fixture meanings.

## Actual verification

The guarded qualification completed from 06:42:35 through 06:54:59 UTC.
All 56 checks produced their expected result: 14 successful checks and 42
deliberate failures. No command timed out or produced an unexpected exit.

The prepared native macOS ARM64 contract environment retains CPython 3.14.7,
uv 0.12.10, check-jsonschema 0.38.0, jsonschema 4.26.0 and the existing 14
locked upstream packages. Commands use applicable external build-output
routing and the dedicated environment. Host defaults, project configuration
and lock are unchanged.

All eight documented preparation/check commands passed:

| Check | Actual result |
| --- | --- |
| `uv lock --project contracts/validation --check --offline` | Exit 0. |
| `uv sync --project contracts/validation --check --locked --offline` | Exit 0; the prepared environment remains synchronized. |
| Documented metaschema command naming all 15 schema paths | Exit 0. |
| Exact common entry and manifest | Exit 0; 248 expectations. |
| Exact acquisition entry and manifest | Exit 0; 217 expectations. |
| Exact observation entry and manifest | Exit 0; 168 expectations. |
| Exact candidate review entry and manifest | Exit 0; 237 expectations. |
| Exact financial records entry and manifest | Exit 0; 519 expectations. |

The new inventory has 171 accepted and 348 rejected values across 52 targets.
All five families together check 1,389 expected outcomes. Financial expectations
were chosen and frozen before their author inspected the new financial schema.
Their first actual comparison passed without changing any case identity,
target, value or expected outcome. The financial library has 57 definitions.

All five families also passed from a task-owned relocated input tree with
spaces and non-ASCII characters in its path. This checks local input resolution,
not a fresh Git consumer, network sandbox or runtime deployment.

Sixteen separate manifest/metadata/value-oracle mutations failed with validation
exit 1. Six infrastructure probes also failed with exit 1 and the expected
reference-resolution diagnostics: broken consumed positive and negative-`not`
references, then missing financial, common, observation and acquisition
libraries. Each missing library was exercised by a family/value that actually
consumes it. Infrastructure failure is not counted as a rejected domain value.

All 20 off-diagonal fixture-family swaps failed with validation exit 1. The
restored financial family passed again. The local graph has 1,878 references
across the 15 schemas, with no remote application reference or injected base
URI. Metaschema validation alone is not treated as reference-resolution proof.

All 22 original and relocated schema/manifest/project/lock input hashes and
six owning guidance hashes remained unchanged. The task-owned mirror was
retained after restoration. A final audit rehashed all 112 captured stdout/
stderr logs and confirmed every expected exit. No earlier fixture outcome was
changed to obtain these results.

| Input | SHA-256 |
| --- | --- |
| `contracts/financial-records-v1.fixtures.schema.json` | `4755bb73b2614c5c137950eba9501a4b24c267426a733e81399b2b3839df9e85` |
| `contracts/public/v1/financial-records.schema.json` | `41ba8ea6e0bbd868616b920898f4613fde2eed064e1388939c245788fa6d45b5` |
| `contracts/validation/financial-records-v1.cases.json` | `c76f33b4b440c039af26ff1b58eaa2cac735237b9810a2a85d352bc0f9ad11d9` |
| `contracts/validation/financial-records-v1.expectations.schema.json` | `13ae792b8bce1a09021c7d8bec9867adb217e9bfa0839772dbd64e862009f85a` |
| `docs/requirements/financial-records.md` | `176396caf9dae87b81b222d1b5654159dbff57433f70268577552c0a0805eef4` |
| `contracts/validation/README.md` | `e426960e24bb4147ffccf864d56d8355ef8b7ef1c423eb971ac3b4ff677f8f48` |

## Changes and independent review

The closed financial content preserves explicit unknowns, exact currency scales,
ordered components, original expense links and field attribution. Confirmation
requires the exact recorded human financial decision and separately attributable
executor. Complete commands keep content and control preconditions distinct;
only revision accepts replacement content. Historical confirmation and
supersession remain retained instead of rewriting old reports.

Report selection freezes current authorized exact references, exclusions,
per-currency totals and policy/cutoff metadata. The owning contract covers
concurrent inserts and timed authority changes through explicit selection
fences. Unknown lists do not fabricate component counts. Report availability
control is visible in create/read responses and stays outside immutable identity.

The export profiles preserve a complete financial payload and separately frozen
source-availability observation. JSON uses exact canonical values. CSV defines
24 fixed columns, typed text cells and a reversible complete tree representation.
Its actual byte/cell bounds and spreadsheet behavior remain runtime requirements.

Three independent design reviews passed after reconciling field names,
attribution units, allocation unknowns, source descriptors, the maximum-date
period boundary and the observable report-control precondition. Independent
schema, literal fixture and owning-document reviews also passed. Review checked
all 519 expectation positions, exact-reference and attribution consistency,
boundary identities, local references, disclosure and preserved source ownership.
Example digest consistency inspection is not qualified JCS implementation or
cross-language canonicalization evidence.

## Remaining qualification

The financial owner retains FIN-SEM-01–32 as required, pending semantic work.
Parsed fixtures check currency/amount grammar, null distinctions, static
reconciliation and lifecycle contradictions, complete commands/resources,
256/257 selection and exclusion boundaries, 33/34 direct-source boundaries
and complete attributed export payload shapes.

They do not prove arithmetic/count equations, raw token or Unicode admission,
canonical byte identities, current source/identity authority, human assurance,
event-consumption races, attribution/history joins, report linearization or
actual export bytes. The 16,384/16,385 global observation-array boundary,
encoded-byte/depth/node/cell limits, interruption and real spreadsheet
import/save/reopen remain explicit later qualifications. No repeated large
literal array is presented as proof of those stream limits.

The 250 monetary scenarios, actual receipt-to-export journey and all applicable
production gates remain required under [acceptance](../requirements/acceptance.md).
There is no first-party Rust or Python source change in this checkpoint and
no numerical source-coverage claim. The existing strict
[testing and coverage contract](../requirements/testing-and-coverage.md) applies
when the owning runtime and maintained test support are introduced.
