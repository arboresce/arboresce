# Testing and source coverage

**Authority:** normative requirements. **Status:** accepted; the bounded Rust
identifier suite and direct collection are implemented. Remaining collection
tools and product suites need their actual qualification before dependent work.

This document owns fixture governance, required test layers, complete collection
and source coverage. [Acceptance](acceptance.md) owns real experiment and release
gates; [resource profiles](resource-profiles.md) owns corpus/hardware definitions.
Documentation adoption is not a build, runtime, live-quality or release result.

## Every source checkpoint

[Project fixtures](project-fixtures.md) owns the focused deterministic JSON/photo/
short-audio/prior-context corpus and independent review trace. D0-project integration
uses actual CLI/API/DB/object-store/workflow/gRPC with recorded inference; live
OCR/ASR/synthesis and actual-user utility are separate qualification lanes. The
strict arithmetic, complete process collection, measured fixture support and
no-skipped-check obligations below apply unchanged to every introduced source.
An enabled-capability profile narrows what is built, not the coverage denominator
or required behavior of source that exists. Packaged security and actual focused
restoration precede personal evidence; later full experiments remain retained.

Each Rust or Python source checkpoint includes a useful complete change, its owning
contract update, independent expected-value review, applicable formatting/static/
type/build/tests, raw coverage and independent behavioral review. Repair a required
failure before starting dependent work. Stub success, unfinished advertised
commands, ignored failures and a test-only red checkpoint are not green slices.
Documentation-only work uses applicable content, link and contract checks without
inventing product commands or a source-coverage result.

Pin preparation inputs explicitly through [dependency qualification](dependency-qualification.md).
Verification must not install tools, fetch an unselected service version or fall
back to a provider. Default deterministic tests require no commercial credentials.
Use isolated task-owned disposable resources with bounded readiness polling,
explicit service identity and cleanup. Preserve host defaults, existing unrelated
data and repository-owned build-output routing where configured.

Required deterministic suites never contact production services, live inference
or production mail submission. Use isolated real services or declared recordings
according to the behavior being proved. Checks are read-only by default; resource
preparation, fixture mutation and cleanup are explicit bounded operations.

## Strict coverage arithmetic

For every applicable metric use raw integer counts:

```text
total > 0 and 10 * covered > 9 * total
```

Exactly 90% fails, regardless of display rounding. Apply each gate independently
for every supported native, feature and runtime profile before considering a
larger aggregate. A highly covered package or another platform cannot compensate
for an under-covered file or unexecuted behavior.

| Metric | Required independent scopes |
| --- | --- |
| Rust executable lines | Repository/language, each crate/package and each executable source file |
| Rust executable regions | Repository/language, each crate and each binary |
| Python executable lines | Repository/language, each package and each executable source file |
| Python branches | Repository/language, each package and each applicable executable module |

The aggregate source denominator and required test collection must be nonzero.
Declaration-only source requires source evidence and metric-specific N/A; a Python
module without instrumenter-detected branches is branch N/A. Neither is fictional
100%. An unexplained zero denominator, missing source, malformed or nonfinite
counts, overflow, duplicate measurement or inconsistent summary fails. There are
no source coverage exemptions or exclusion pragmas that alter the denominator.

Inventory runtime libraries, binaries/entrypoints, adapters, tools, build and
migration scripts, generators, maintained test support and generated executable
source. Include declared generated outputs even when Git discovery alone does
not find them. Reconcile unimported modules and files never executed by tests
against the actual report; they cannot disappear from measurement.

Assertion-only test bodies are inventoried as tests and cannot inflate production
or support coverage. Reusable builders, parsers, oracles, service lifecycle helpers
and harnesses are measured source regardless of their directory or filename.
Review role changes as coverage-policy changes. Do not hide support through a
broad test-directory omit, a generated label, an allowlist omission or a filename
that resembles an assertion suite. Behavioral security, money, sealing, state,
retry and recovery assertions remain mandatory even when all percentages pass.

## Complete and fresh collection

Bind each run to exact source, test, fixture, schema, configuration, dependency
lock, toolchain, instrumenter and feature/native-profile identities. Observe the
actual input inventory, including additions, removals and renamed fixtures.
Changing any bound input invalidates the prior result. Reports from incompatible
profiles or source versions must not be merged; publish each qualified profile's
raw counts and gate result independently.

Reconcile registered suites and collected node/case identities with executed
results. Required empty, deselected, duplicate or skipped cases fail. Unknown suite
selection, unavailable required services and ambient filters/plugins that silently
narrow execution fail. A missing dependency is a blocked lane, not a skipped pass.
Preserve actual exit codes, counts, failures and warnings; an intended command is
not execution evidence.

Explicitly collect every exercised native executable, CLI, worker and Python
service/subprocess. Record launched process identities, unique fresh raw profile
paths and current run/source context. Start instrumentation before relevant work,
perform bounded shutdown/flush, and prove all required process profiles exist and
are included before aggregation. A successful child exit does not establish its
coverage. Lost, empty, stale, mixed-context or incomplete service profiles fail.
Do not silently start collection via an ambient import hook or write reports into
the source checkout when tests are running without a measurement request.

Keep raw region/branch data, machine reports, human-readable diagnostics, selected
and executed test identities, process receipts and exact gate calculations. A
fresh output directory prevents accidental reuse. The gate and collection runner
measure themselves and must have meaningful adversarial tests for denominator,
inventory, profile, process-loss and freshness failures. Public evidence must not
expose host paths, credentials or source evidence content.

Use prepared stable `cargo-llvm-cov` for Rust lines/regions, nextest where suitable,
and separate Cargo/doctest coverage for tests not executed by that runner. Use
pytest and coverage for Python lines/branches, including async/service lifecycle.
A nightly toolchain is not required solely for these gates. Before repository
runners exist, direct prepared-tool collection must demonstrate the same complete
inventory and arithmetic. Implemented runners then replace repeated manual
orchestration without weakening it. Exact supported invocation belongs with the
tool once implemented. Current direct Rust collection is documented below;
this does not advertise future service or product commands.

### Current Rust source and direct collection

The current native profile is Rust 1.99.0, macOS ARM64 and the default workspace
feature set. Prepare `cargo-llvm-cov` 0.9.1 and that selected toolchain's LLVM
tools before measurement; checks never install tools. The two libraries have
no third-party dependencies. The first verification tool has separately pinned
public parser dependencies. The binding/build proof remains a placeholder and
does not qualify an API or FFI runtime.

| Rust path | Measurement role |
| --- | --- |
| `crates/arboresce-domain/src/lib.rs` | Checked identifier library; source lines/regions. |
| `crates/arboresce/src/lib.rs` | Existing binding/build proof; source lines/regions, no MVP runtime claim. |
| `crates/arboresce-domain/tests/support/mod.rs` | Maintained formatter-failure sink; source lines/regions. |
| `crates/arboresce-domain/tests/identifiers.rs` | Four assertion bodies; raw execution retained separately from source numerators. |
| `crates/arboresce/tests/smoke.rs` | Existing proof assertion body and observable name output; separately inventoried. |
| `crates/arboresce-domain/tests/fixtures/primitives-v1/expected.rs` | Literal declaration-only oracle table; proven metric-specific N/A. |
| `tools/xtask/src/lib.rs` | Explicit command/stream execution; measured source. |
| `tools/xtask/src/main.rs` | Actual verification-tool entrypoint; measured source and native binary. |
| `tools/xtask/src/architecture.rs` | Dependency/syntax/import guard; measured source. |
| `tools/xtask/src/workspace.rs` | Resolved-graph and source-directory guard; measured source. |
| `tools/xtask/tests/support/mod.rs` | Maintained syntax-input builder, process observer and failure sinks; measured support. |
| `tools/xtask/tests/support/peer.rs` | Actual controlled test-peer binary; measured support and native process. |
| `tools/xtask/tests/contract.rs` | Assertion bodies; raw execution separate from source metrics. |

The additional diagnostic text and JSON files are literal fixture data. No parser,
builder or oracle function is hidden there. New files require an owning role;
assertion bodies cannot absorb reusable support to alter source measurement.
The identifier suite executes every accepted 34 text and eight binary case IDs,
exact roundtrips, immutable equality/hash and bounded error behavior. Actual
formatter failures propagate. The existing proof's name-print function executes
and produces its original literal output. The tool retains all 96 independently
accepted policy/command/intake IDs, malformed/duplicate graph and source-path
negatives, actual native output/exit and controlled child boundaries. Peer/vector
success is not actual Cargo evidence: separately observe every real delegated
workspace command. Required suites and separate Cargo doctests have zero
failed/ignored/filtered cases. All introduced support and entrypoints remain in
the measured source/native scope; no source is exempted by its test location.

Choose a fresh external `ARB_COVERAGE_DIR`, distinct from a directory used for
an uninstrumented build, and collect directly before any custom runner exists:

```sh
env -u LLVM_PROFILE_FILE \
  CARGO_TARGET_DIR="$ARB_COVERAGE_DIR" \
  CARGO_LLVM_COV_TARGET_DIR="$ARB_COVERAGE_DIR" \
  CARGO_BUILD_BUILD_DIR="$ARB_COVERAGE_DIR" \
  cargo llvm-cov --locked --offline --workspace --all-targets --no-clean \
  --json --output-path "$ARB_COVERAGE_DIR/coverage.json" -- --nocapture
```

Apply the configured output router where present. Keep every fresh nonempty
native test-process profile and prove it enters aggregation. Reconcile actual
Cargo test executables, case/PID receipts, source/fixtures/locks, tools and profile
identities. Export the actual integration-test objects with the selected LLVM
tools and aggregate profile so maintained support cannot disappear from a
product-only default report. Gate every required source, file, library, package
and native profile strictly above 90% for both lines and regions. Raw assertion
execution remains retained without inflating source metrics. No exclusion applies.
Qualify the same exact source in a standalone public-only snapshot. Other native,
service, packaging, live-model and production profiles retain their later gates.

## Independent deterministic fixtures

Commit independently authored, versioned, minimal synthetic fixtures and scenario
inventories with rights, digests and exact expected outcomes. Do not depend on an
unpublished checkout, an archive or a developer's local files. Real customer data
must not be committed. A selected real evaluation corpus requires documented rights,
access and provenance and remains separate from ordinary unit fixtures.

Use controlled clocks, identities, randomness and policies where the contract
permits injection. Record stable seeds and a named bounded property-test profile
with reproducible failures. Fix locale, timezone, ordering and service identity
when relevant. Normalize only declared incidental output; never normalize money,
authority, subject identity or failures to make snapshots agree. Fixture mode
denies unexpected network. Recorded providers fail on a missing recording and
never fall through to a live call.

Expected values require independent semantic review. Recomputing expected output
with the production helper, or approving generated golden files merely because
tests now pass, is insufficient. Each scenario has a purpose, valid and invalid
assertions, explicit boundaries and traceability to its owning requirement.
Mutation checks must show that relevant invalid states or broken invariants are
detected. Property tests add broad invariant coverage to independently fixed
examples; avoid vacuous properties, excessive filtering and retry-until-green
behavior. Retain failing seeds and minimized fuzz cases as regressions.

Use barriers or observable protocol events to coordinate races. Arbitrary sleeps
do not prove ordering. Readiness polling has deadlines and identifies the service
actually reached. Parallel and reordered suites use isolated organizations,
namespaces, directories and process profiles; cleanup checks identify leaks and
must never touch another test's resources.

## Required verification layers

| Layer | Required behavior and negative cases |
| --- | --- |
| Pure domain and properties | Tenant/reference identity, separate revisions, constructors, state transitions, canonicalization, exact monetary arithmetic and idempotency. |
| Contracts and compatibility | Schema plus runtime cross-field validation, invalid variants, unknown references, generated-code drift and compatible unknown response fields. |
| PostgreSQL | Real migrations, direct and transaction-pooled connections, non-owner roles/RLS, same-tenant foreign keys, uniqueness and atomic command/outbox writes. Barrier-driven conflicting writers and reconnect/session isolation. |
| Durable work | Real Temporal histories and replay, worker/server restart, retry after ambiguous side effects, lease fencing, cancellation races and supported upgrade compatibility. |
| Object storage and acquisition | Real S3-compatible staging/sealing, interrupted/changed/overwritten input, checksum and length failures, unsafe URL/file authority, orphan reconciliation and scoped cleanup. |
| Processing | Python normalization, typed OCR/ASR outputs and locators, recorded-provider failure paths, parser bombs, resource limits and cross-language gRPC/service instrumentation. |
| CLI and consumers | Independently built client against actual API/services: accepted versus completed state, stdout/stderr, exit and interruption semantics, safe output and native packaging. |
| Retrieval | Restrictive ACLs, bounded shared budgets, canonical authorization before exposure, out-of-order/duplicate events, stale generations, revocation and maintenance rebuild. |
| Security | Credential/profile lifecycle, distinct human assurance, prompt injection, output encoding, source/destination separation, egress, quota races and content/artifact trust. |
| Recovery and exit | Real canonical database, sealed bytes, keys/trust and current independently retained lifecycle journal; closed serving on incomplete restore; usable export in a clean consumer. |
| Images and operating roles | Actual qualified native image builds, role-specific startup/readiness/shutdown, pinned input/SBOM identity, restricted permissions, secret/cache exclusion and declared resource envelopes. |
| Fuzzing | JSON, cursors, file metadata, locators, terminal output and parser boundaries, with bounded execution and reproduced regressions. |
| Live qualification | Untouched held-out model evaluation and independent human/agent utility; these are separate from deterministic tests and have [acceptance gates](acceptance.md). |

At minimum the fixed semantic oracle inventory includes the following assertions:

- Reviewed components 40.00 + 2.00 + 8.00 equal exactly 50.00; unknown tax remains
  unknown with unresolved reconciliation; CAD and USD totals stay separate.
- Missing reviewed total, currency or date keeps an expense proposed and outside
  default reports. Refund/credit kind, void, negative input, locale and reporting-date
  boundaries follow [financial records](financial-records.md).
- Accept rejects replacement content; revise requires complete replacement;
  stale control revisions conflict; success with a failure payload is rejected.
- Unavailable dependencies prevent readiness. An active channel requires a build.
  A required failed attachment prevents capture readiness.
- A revoked-and-held object denies read and retains held bytes. A revoked required
  snapshot source blocks complete resolution; it does not create a partial success.
  Evidence audiences combine by intersection.
- Interrupting observation does not cancel durable work or claim success.
  Reversed text ranges and invalid JSON-pointer escapes are rejected.
- Concurrent idempotent retries share accepted intent; stale delivery or reordered
  projection work cannot regain authority; shared search limits survive fan-out.
- Rust, Python and client canonical bytes agree on independently reviewed Unicode,
  numeric and key-order cases; malformed signatures and unknown trust fail safely.

The list is a minimum, not a coverage shortcut. New operations introduce their
own required positive, invalid, boundary, denied, unavailable, failed, expired,
cancelled, concurrent and cleanup cases before dependent implementation.

## Evidence and gate limits

Freeze the actual environment and interpreter base selected by every command,
including lock/synchronization checks, plus consumed wrappers, executables,
configuration, source and fixture identities. Equivalent isolated copies do not
establish the bytes of a shared environment actually selected during execution.
Before/after inventories must cover those consumed paths and resolution choices.

Negative qualification retains bounded decisive diagnostic witnesses sufficient
to reconstruct its classification independently, including exact missing paths
for missing-reference/library checks. A producer's exit and success flag or a tail
that omits the relevant error cannot substitute for that witness. Tool, profile,
timeout and unrelated resolution failures are not expected value rejections.
Capture evidence during a fresh run; preserve older packets and do not reconstruct
historical missing hashes or diagnostics. A changed primary validator profile
requires the complete documented protocol afresh. Any permitted unchanged-check
reuse binds its exact consumed inputs, effective profile and tools explicitly.

Record actual commands, source and locks, profile, test identities/counts, raw
coverage numerators/denominators, result, duration, reviewer and unresolved findings.
Store a checkpoint's own commit identity in a subsequent or external record rather
than attempting to embed its final hash into itself. Distinguish specified,
implemented, runtime-verified and user-accepted states.

Fakes can prove a unit contract but cannot satisfy a required service, replay,
CLI, restore, live-provider or independent-user experiment. If real credentials,
corpus rights, numeric scoring criteria, native resources or operational authority
are missing, name the exact blocked gate and missing input. Continue unrelated
ready work without substituting recordings, skipped tests or invented evidence.
Use the [acceptance experiment inventory](acceptance.md) for those obligations.

## Scoped fixture consumption and verification scheduling

A versioned independently reviewed package may qualify the complete inputs of a
bounded consumer before the whole canonical corpus is accepted. Its descriptor
identifies cases, rights, contracts, digests, oracle meanings, allowed consumers
and explicit exclusions. Missing groups outside that scope are not a whole-corpus
pass. The consumer must enforce that scope and qualify every input it actually
uses. Whole journey/corpus acceptance preserves every original required case.

Use narrow reproducers during red iterations, then complete fresh owning source,
helper/process collection, strict coverage and independent review before a green
source checkpoint. Changed consumed inputs invalidate affected evidence; unchanged
accepted tool/profile inputs may be reconciled as exact reuse, never called a new
run. Preserve full case inventory, zero required skips and per-native/profile gates.
After two equivalent full failures, diagnose a concrete cause before another full
run. Sharding requires reviewed equivalence, complete outcomes and compatible raw
profile conservation; no arbitrary test reduction or coverage exemption is allowed.
