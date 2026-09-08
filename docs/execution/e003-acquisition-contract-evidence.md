# Acquisition contract checkpoint evidence

Date: 2026-09-08. The starting source revision
is `1eea8a99dd36eb90ca7c4823ad34b19d2e4103f8`, the independently reviewed common
contract checkpoint. This record distinguishes parsed-value verification
from prospective runtime scenarios. The following plan record identifies the
enclosing source checkpoint's exact commit.

The [acquisition owner](../requirements/acquisition-contract.md) defines complete
upload and capture commands, sealed-byte bindings and attachment readiness.
The [validator guide](../../contracts/validation/README.md) owns the supported
standalone checks. The existing locked, isolated validator profile is reused;
there is no first-party executable helper or processing environment change.

## Baseline independence

An independent fresh origin consumer checked the exact E002 source revision
above. The single-branch `master` clone used neither local object sharing nor
tags. All 51 tracked blobs matched the source; history was complete, with no
alternates, shallow boundary or promisor objects. `git fsck --full --strict`
passed, the source remained clean and observed local/remote `master` matched.

The consumer prepared its own CPython 3.14.7 / uv 0.12.10 environment from the
committed lock. All 14 installed upstream packages matched, and all four E002
documented commands exited zero: 248 expectations, comprising 66 accepted and
182 rejected values. Its seven recorded input hashes matched E002 evidence.
Independent local documentation inspection checked 39 Markdown files and 276
local targets; eight external URLs were enumerated separately. No Rust/Python
source or other checkout was required by the public checks. These observations
verify the baseline contract consumer, not the new E003 candidate or a runtime.

## Actual verification

The prepared native macOS ARM64 profile retains CPython 3.14.7, uv 0.12.10,
check-jsonschema 0.38.0, jsonschema 4.26.0 and the exact 14 upstream packages
recorded by [E002](e002-common-contract-evidence.md). The project and lock are
unchanged. Builds and checks used the applicable external output routing and
the dedicated contract environment, preserving host defaults.

All five commands in the current validator guide passed against the final
candidate inputs:

| Check | Actual result |
| --- | --- |
| `uv lock --project contracts/validation --check --offline` | Exit 0; 15 lock records including the virtual project. |
| `uv sync --project contracts/validation --check --locked --offline` | Exit 0; 14 installed packages, no changes required. |
| `check-jsonschema --check-metaschema` with all six explicit schema paths and documented options | Exit 0. |
| Common entry and the explicit common manifest, with documented options | Exit 0; all 248 expectations passed. |
| Acquisition entry and the explicit acquisition manifest, with documented options | Exit 0; all 217 expectations passed. |

The new independent inventory contains 80 accepted and 137 rejected values
across 27 targets. Both families together check 465 expected outcomes. Negative
expectations use the actual target's logical negation and must produce a
successful expectation check. No infrastructure exception counts as a rejected
domain value.

| Input | SHA-256 |
| --- | --- |
| `contracts/common-v1.fixtures.schema.json` | `3d36bfa44e45b7f35d2be77e11966f3fffeefce994867a671ae4a6dc9fc4bc2d` |
| `contracts/acquisition-v1.fixtures.schema.json` | `d7d8f31e3a0c2964f27fb8fd9c882642a1bd2ed7b43af924cee5a33609b7ed1e` |
| `contracts/public/v1/acquisition.schema.json` | `e17c062fc08fa862fdad9f86445137b5514ed54b79a8a6578bbdbed967cae459` |
| `contracts/validation/acquisition-v1.cases.json` | `bb4b57c1a063a6172d7b74e294f50c3e6536a7b4c4add540265e2d4cc9de143e` |
| `contracts/validation/acquisition-v1.expectations.schema.json` | `a62e9a22dcf14963bffc830db7ee7188a95616d8192a15fcb7de4ba9d42d6dad` |
| `docs/requirements/acquisition-contract.md` | `83cf188342bc2aeed23cf7b7cb25d5a03d6082d271271113391377f770bf1aed` |
| `contracts/validation/README.md` | `2138e429d739f84cedf4014070973ad851518cdf428efa34f9ecfca168bfc2ac` |

The common schema, common fixture pair, prepared project and lock retain their
E002 digests. The new entries change the supported invocation without copying
or changing the owning contracts or their expected values.

## Resolution failure and review findings

The first acquisition instance check failed to resolve its nested reference to
the common library, despite both schemas passing metaschema validation.
Inspection of the installed upstream resolver confirmed that its original
retrieval-base fallback mishandled a leading parent traversal in this graph.
The guide links the matching upstream report. No oracle outcome was relabelled
to conceal this infrastructure failure.

Two tiny declarative entry schemas now start relative resolution at the local
contract graph's root. Each references its exact existing expectation family.
The entries introduce no copied definitions, injected identifiers, base URI
overrides, modified dependencies or first-party helper. A generic union entry
was not selected because a swapped family could satisfy the other branch.

Sixteen metadata/oracle mutations each rejected with validation exit 1. Three
broken-reference diagnostics separately failed with infrastructure exit 1:
positive-reference corruption, negative-`not` reference corruption and a missing
adjacent common schema. Both swapped-family pairings rejected with exit 1.
Restoring the original acquisition inputs passed again with exit 0.

Both complete manifests also passed from a task-owned relocated contract-input
mirror whose path contained spaces and non-ASCII characters. This was an input
relocation check, not another full Git consumer qualification. Ten validation
and environment input hashes were unchanged before and after these diagnostics;
the temporary mirror was cleaned afterward.

Independent review closed static response contradictions: submitted captures
cannot retain initial control revision one, and revise responses require both
counters to have advanced. Four isolated rejection cases cover those minima;
positive snapshots now show reachable counter values. Review also clarified
the authoritative server clock, configured purpose/action checks, exact header
byte qualification, immutable initial-response replay and the distinction
between public upload generation and internal finalizer fencing.

The six schemas contain 612 resolvable local references; the new acquisition
library has 32 definitions and 93 local references. The complete candidate has
58 files, including 41 Markdown files and no Rust/Python source. Local
link/fragment review passed for 301 targets, with nine distinct external URLs
identified separately. All 13 staged files match their reviewed working bytes,
and `git diff --cached --check` passed. Independent evidence and public metadata
review precede the green commit.

## Runtime boundaries

Parsed-value validation cannot establish actual object ownership, current
authority or clock time, streamed byte identity, cumulative byte arithmetic,
stored immutable history or finalization races. Those deterministic semantic
scenarios require their owning domain and service implementations. The contract
will record explicit expectations and later owners without substituting shape
validation for their required results.

The owner records 18 explicit prospective semantic scenarios and their later
domain, persistence, API, processing, CLI and service owners. These are not
executed runtime tests. No source-coverage, storage backend, codec, service,
platform or product gate is qualified by this documentation and schema work.
