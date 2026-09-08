# Candidate review contract checkpoint evidence

Date: 2026-09-08. The starting source revision
is `c0cd459a34b6effc922f8a9e1f8d003379f22e4f`, the independently reviewed
[observation checkpoint](e004-observation-contract-evidence.md). This record
separates parsed-value results from pending semantic and runtime requirements.

The [candidate review owner](../requirements/candidate-review-contract.md)
defines complete content and review variants. The
[validator guide](../../contracts/validation/README.md) owns the supported
standalone checks and the existing pinned isolated environment. The new
exact-family entry starts from the established local contract graph root.

## Actual verification

The prepared native macOS ARM64 environment retains CPython 3.14.7, uv 0.12.10,
check-jsonschema 0.38.0, jsonschema 4.26.0 and the existing 14 locked upstream
packages. Checks used applicable external build-output routing and the dedicated
contract environment. Host defaults, project configuration and lock are unchanged.

All seven documented preparation/check commands passed against the final schema
and fixture inputs:

| Check | Actual result |
| --- | --- |
| `uv lock --project contracts/validation --check --offline` | Exit 0; 15 records including the virtual project. |
| `uv sync --project contracts/validation --check --locked --offline` | Exit 0; 14 installed packages checked, no changes required. |
| Documented metaschema command with all 12 explicit schema paths | Exit 0. |
| Exact common entry and manifest | Exit 0; 248 expectations. |
| Exact acquisition entry and manifest | Exit 0; 217 expectations. |
| Exact observation entry and manifest | Exit 0; 168 expectations. |
| Exact candidate review entry and manifest | Exit 0; 237 expectations. |

The new independently authored inventory has 67 accepted and 170 rejected
values across 20 targets. All four families together check 870 expected outcomes.
The candidate library has 24 named definitions and 81 local references. Its
shared helpers do not substitute for complete command and response variants.

Sixteen metadata/oracle mutations each failed with validation exit 1. Five
separate infrastructure diagnostics failed with exit 1: broken positive and
negative-`not` definitions, missing common and observation libraries consumed
by candidate review, and a missing acquisition library consumed by observation.
The latter uses a family that actually loads the missing definition; a reference
to an observation ID alone does not execute every unrelated observation branch.
All 12 off-diagonal family swaps failed with validation exit 1. Infrastructure
failure is never counted as a rejected domain value.

All four families passed from a task-owned relocated contract-input mirror with
spaces and non-ASCII characters in its path. Restored candidate inputs passed
again. Eighteen schema/manifest/project/lock input hashes remained unchanged
from before the seven commands through the final diagnostics. The temporary
mirror was cleaned. This is input relocation evidence, not a fresh Git consumer,
canonicalization implementation or service qualification.

| Input | SHA-256 |
| --- | --- |
| `contracts/candidate-review-v1.fixtures.schema.json` | `ec0bbef885bd3c932948cff7fe9d053a4f0ee02916ae070cd98930bfe87f6ba6` |
| `contracts/public/v1/candidate-review.schema.json` | `07c23839716c5aa082d0ff5a74fb1608efffe48c7dd23ae5d26f8854a7253eed` |
| `contracts/validation/candidate-review-v1.cases.json` | `eceaf6345895b65061da7143b868e32a48f244536759e810435f7f957a8885fb` |
| `contracts/validation/candidate-review-v1.expectations.schema.json` | `e9f2eecf8582c503f9227318655c527139ebf40b1e991a88e168579bbbd1e7fb` |
| `contracts/public/v1/common.schema.json` | `26c99c9f2ae50a9276274914652217efd0577f67554b8011a8eec797834f3b58` |
| `docs/requirements/common-contract.md` | `6e95583e21220e87a36f90491d2008e6341265ccdcc86bffebd1fedc0ec0f4b7` |
| `docs/requirements/candidate-review-contract.md` | `d4a84a786078b88af6f24720392f7a93ce954c4727ec90253bc24d1b3de3f4bf` |
| `contracts/validation/README.md` | `5124746e17f0ae55a293684266589be85287d6c4e34920d2eeb050826324a513` |

## Changes and review boundaries

The candidate content shape includes knowledge kind, statement, explicit scope,
temporal applicability, assumptions, qualifications, all three evidence roles,
invalidation conditions and review-due declaration. Complete command variants
pin content revision, content digest and independent control revision. Only the
revision command accepts replacement content; verdict commands cannot quietly
edit what was reviewed.

The reusable calendar-date primitive now has one shared definition and owning
section in the common contract. The candidate family retains a named alias,
so its independently chosen date cases exercise the real common definition.
This additive change avoids a financial-domain dependency on candidate review
or a copied date grammar. Historical common evidence and existing fixture
inventories are preserved; the full current graph requires fresh checks.

Independent static review identified impossible initial counters in reviewed
and superseded resource shapes. The schema and isolated fixture expectations
now express the reachable fixed minima. General sibling revision comparisons,
successor identity and transaction ordering remain mandatory semantic checks.

The content identity covers the exact organization/domain/intended-use purpose
and normalized complete content. Each evidence role has an exact tuple ordering;
other arrays remain ordered content. Normalizing candidate evidence sets does
not change the common command fingerprint's sensitivity to submitted array order.
An unchanged normalized replacement is rejected without creating a revision.

Current authority, a trusted exact approval event and the executing principal
remain distinct. An executor's reason cannot be attributed to a human approver
without the corresponding trusted binding. Reauthorized replay returns the
original committed result before applying fresh-head preconditions; it neither
consumes the event twice nor grants another effect. Candidate-head supersession
preserves old reviews/mappings and does not mutate asset or channel controls.

Review clarified the prospective concurrency oracle: fifty otherwise-valid
accept contenders produce one accepted mapping; mixed verdict/revise contenders
produce one winning transition with its actual effects. A winning revision has
no verdict or asset mapping, and rejection/contest has no accepted mapping.
The transaction's required mapping invariant applies specifically to acceptance.

Independent schema, fixture and complete owning-document review passed after
those corrections. The complete candidate has 70 files, including 45 Markdown
files and no Rust/Python source. Local link/fragment inspection passed for 349
targets, with 12 distinct external URLs identified separately. All 1,146 local
references in the 12-schema graph resolve. The eight input hashes above match
their current bytes.

Public content, rights and prepared commit-message review passed. A bounded
configured-pattern scan found no file/name/message hits; that scan is separate
from the independent semantic disclosure review. All 14 staged files matched
their reviewed working bytes, and `git diff --cached --check` passed. The plan
records verified status; its following completion record supplies the source
commit hash without recursively embedding that hash here.

## Runtime boundaries

Parsed shapes cannot establish actual digest recomputation, canonical evidence
ordering, current authority, immutable history, stale preconditions, transaction
isolation, replay or accepted-mapping uniqueness. These requirements need fresh
evidence from their owning domain, persistence, service and CLI implementations.
There is no Rust/Python implementation, hidden validator helper, service or
product qualification in this declarative contract slice.
The owner records 17 required future semantic scenarios. Their independently
prepared examples cannot substitute for actual canonicalization, current-policy,
concurrency, issuer or CLI evidence. No source-coverage or product gate is
qualified by these parsed-value checks.
