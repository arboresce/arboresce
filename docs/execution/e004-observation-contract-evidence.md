# Observation contract checkpoint evidence

Date: 2026-09-08. The starting source revision
is `7149b621713b351feb8996391cc95964a3324050`, the independently reviewed
[acquisition checkpoint](e003-acquisition-contract-evidence.md). This record
separates parsed-value results from pending semantic and runtime requirements.
The following plan checkpoint records the enclosing source commit's exact hash.

The [observation owner](../requirements/observation-contract.md) defines exact
locators, source kinds and grounding. The [validator guide](../../contracts/validation/README.md)
owns the supported standalone checks and existing pinned isolated environment.
The new exact-family entry starts from the established local contract graph root.

## Actual verification

The prepared native macOS ARM64 environment retains CPython 3.14.7,
uv 0.12.10, check-jsonschema 0.38.0, jsonschema 4.26.0 and the same 14 locked
upstream packages. Applicable external build-output routing and the dedicated
contract environment preserve host defaults. The project and lock are unchanged.

The initial observation entry check passed all 168 independently authored
expectations: 73 accepted and 95 rejected values across 17 targets. The library
has 18 named definitions and 46 local references; its shared response helper
does not substitute for complete observation variants.

All nine schemas passed the documented metaschema command. All three exact-family
entry commands passed: 248 common, 217 acquisition and 168 observation
expectations, or 633 total. The common/acquisition schemas, fixtures and graph
entries are unchanged. Both exact preparation checks also exited zero:
`uv lock --project contracts/validation --check --offline` resolved 15 records
including the virtual project; `uv sync --project contracts/validation --check
--locked --offline` checked 14 installed packages and required no changes.
All six documented preparation/check commands therefore passed against the
final contract inputs.

Sixteen metadata/oracle mutations each failed with validation exit 1. Four
separate infrastructure diagnostics failed with exit 1: broken positive and
negative-`not` references, missing common library and missing acquisition
library. All six off-diagonal fixture-family swaps failed with validation exit
1. None of those infrastructure failures counts as a rejected domain value.

All three complete families passed from a task-owned relocated contract-input
mirror with spaces and non-ASCII characters in its path. Restored observation
inputs passed again. Fourteen contract/environment input hashes were unchanged
before and after diagnostics, and the temporary mirror was cleaned. This
verifies input relocation, not a fresh Git consumer or a service deployment.

| Input | SHA-256 |
| --- | --- |
| `contracts/observation-v1.fixtures.schema.json` | `d5d0e5be7ebcf3bf781cf6209da64febdd0649e453e018a7ba7d0d67d88d14b0` |
| `contracts/public/v1/observation.schema.json` | `d9b92560eead95bf6eb38a531d7b64d09755afa9735bd0d8e89423a868536ca6` |
| `contracts/validation/observation-v1.cases.json` | `a1bce349e1f1f901fb575bbbe9fa3e467521af4a9bbc51739d88759207f3731b` |
| `contracts/validation/observation-v1.expectations.schema.json` | `75a01cb707a064120cc00a5a8e44738976e2800feb1674cdfd1e462236edd624` |
| `docs/requirements/observation-contract.md` | `bf29330ba7d44df512a8ce775e9e5f3886d7f23fdfa7ad5ca3bb8c6cdbe92eb6` |
| `docs/requirements/common-contract.md` | `b374ce8858678a8397315af58a1a9139c2a31b31c8d5b7174091764246da790d` |
| `contracts/validation/README.md` | `f9d82e8f37028df47cb73b9b9038732b11619140e0f3f35ab9aa6c0a9e8897ab` |

## Contract decisions and review

Locators preserve exact original, representation, byte and processing-profile
identity. UTF-8 and audio intervals use explicit half-open start/end bounds;
sibling ordering and real representation bounds remain mandatory semantic
checks. Parsed fixtures deliberately do not invent a rejection for an otherwise
well-shaped reversed interval.

JSON Pointer permits the root, empty keys and exact escaped/control keys.
Object keys `01` and `-` remain valid; array-index resolution has separate
semantic rules. Selected-value JCS identity is distinct from original-byte
identity. Explicit traversal/output budgets and decimal-value admission protect
that selection profile without narrowing the retention of valid original JSON
or applying control-object key restrictions to source data.

Page geometry uses upright pixel edges and all eight Exif transforms, including
mirroring. Audio retains the five-minute profile, exact channel/time provenance
and any transcript actually used. Capacity bounds do not qualify a renderer,
codec or larger media profile.

Inline capture context keeps its immutable revision and server-owned audit
attribution while remaining explicitly ungrounded and review-required. It has
no fabricated representation locator. Supplied representations preserve their
actual capture association. Document observations, user assertions and model
inferences stay distinct; a valid citation or extraction model does not create
approval or prove the underlying event.

A bounded common-contract amendment now specifies canonical raw unsigned
integer tokens for explicitly declared protocol integers. It preserves general
JSON and source-JCS number semantics, revision strings and all parsed-schema
outcomes. The new `RAW-06` requirement covers numeric coercion and canonical
emission before the first consuming decoder is implemented. No raw-decoder
result is claimed by this documentation amendment.

Independent schema, fixture, owning-document and public content/metadata review
passed. Review corrected the Exif citation to the published base standard; the
eight orientation transforms and independent rectangle oracle were unchanged.
Primary indexed standard text supported the citation; a full PDF download was
not claimed. The nine-schema graph has 827 resolvable local references. The
complete candidate has 64 files, including 43 Markdown files and no Rust/Python
source. Local link/fragment inspection passed for 325 targets; 12 distinct
external URLs were identified separately. Configured public-boundary pattern
checks found no candidate-file or prepared-message hits; independent semantic
review remains distinct from that bounded scan.

All 14 staged files match their reviewed working bytes, and
`git diff --cached --check` passed. The plan records this slice as verified;
its following completion record supplies the resulting source commit hash.

## Runtime boundaries

Parsed shape validation does not prove interval ordering, Unicode byte
boundaries, actual pointer selection or source identity, image transforms,
audio timing, grounding quality or current authority. These semantic scenarios
must be independently reviewed and executed by their owning implementations.
The observation owner records 18 prospective semantic scenarios with independent
UTF-8, pointer and orientation examples. Their later domain, processing, storage,
service and CLI suites must execute them with fresh evidence. There is no
Rust/Python product implementation, hidden validator helper, service or product
qualification in this declarative contract slice. Its parsed-value checks
provide no source-coverage result and do not waive any later coverage gate.
