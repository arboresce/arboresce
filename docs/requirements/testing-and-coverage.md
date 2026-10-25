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
| `crates/arboresce-application/src/policy/mod.rs` | Explicit workspace/principal context and typed same-workspace reference scope; measured source. |
| `crates/arboresce-application/src/policy/principal.rs` | Checked exact identity/principal directory port and finite mapping failures; measured source. |
| `crates/arboresce-application/tests/support/principal.rs` | Input-only directory producer, actual query/poll observations and native error API diagnostics; measured support. |
| `crates/arboresce-postgres/src/tenant.rs` | Bounded reused direct tenant session and canonical membership read; measured source. |
| `crates/arboresce-postgres/tests/support/tenant.rs` | Tenant fixture intake, canonical seed/catalogue and actual isolated-service fault/probe support; measured source. |
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
| `tools/xtask/tests/contract.rs`, `collector_contract.rs`, `native_contract.rs`, `profile_contract.rs` | Four complete Cargo integration-test targets containing assertion bodies; raw execution separate from source metrics. |
| `tools/xtask/src/coverage.rs` | Supplied LLVM count/role/scope gate; measured source. |
| `tools/xtask/src/strict_json.rs` | Bounded JSON visitor with duplicate rejection; measured parser support. |
| `tools/xtask/src/report_command.rs` | Supplied-report file/argument/output boundary; measured command support. |
| `tools/xtask/src/bin/arboresce-report-gate.rs` | Actual report-count entrypoint; measured source and native binary. |
| `tools/xtask/tests/support/report_inputs.rs` | Literal input recipe, patch and padding preparation; measured test support. |
| `tools/xtask/src/inventory.rs` | Actual file/source/target inventory, SHA-256 input identity and declaration proof; measured source. |
| `tools/xtask/src/inventory_command.rs` | Actual Cargo metadata and inventory command boundary; measured source. |
| `tools/xtask/src/process.rs` | Shared bounded native status/output observation, actual completion and owned-child cleanup; measured source. |
| `tools/xtask/src/measurement.rs` | Explicit native request, selected executable, immutable launch/completion identity; measured source. |
| `tools/xtask/tests/support/measurement_inputs.rs` | Maintained context, permission and executable-input preparation; measured support. |
| `tools/xtask/src/bin/arboresce-source-inventory.rs` | Actual inventory entrypoint; measured source and native binary. |
| `tools/xtask/tests/support/inventory_inputs.rs` | Maintained actual-filesystem model preparation; measured support. |

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

### Supplied report counts

The [report-count command](../../tools/xtask/README.md#supplied-report-counts)
and library evaluate a supplied selected role inventory and LLVM 3.1.0 integer
summary. The independently reviewed
[report literals](../../tests/architecture/fixtures/report-gate-v1/README.md)
own its exact input/output and negative cases. Source roles determine inclusion;
directory names do not. Raw sums include assertion rows for comparison with
the overall count summaries, while source gates exclude assertion numerators
and denominators. File, package, repository, library and binary scopes have
disjoint identities. Checked aggregation rejects overflow; wider integer
comparison applies the strict threshold without floating-point rounding.
Display percentages never override raw counts and consistent region remainders.
No unproved declaration role is admitted as an empty-source exception.

A supplied-report pass is a count-model result. It does not prove actual source
inventory, declaration N/A, source/tool/run identity, fresh complete native
profiles, test selection or whole verification. The actual collector must
establish each of those prerequisites and retain complete raw data before using
this result. Direct prepared-tool qualification still applies to all introduced
source, parser/input support and the actual native command. The complete
automated collector and whole verification remain unfinished.

### Actual file and Cargo target inventory

The [source-inventory command](../../tools/xtask/README.md#actual-source-inventory)
obtains actual offline Cargo metadata, applies the architecture guard and observes
actual included files. Its library explicitly accepts supplied metadata. The
reviewed [source profile](../../tests/architecture/rust-source-roles-v1.json)
registers every current Rust path and actual typed Cargo target. Maintained
builders and process support stay measured, while assertion-only bodies remain
separate. Actual Cargo library names identify scopes. Included source, fixtures,
schemas, manifests, locks and configuration receive exact SHA-256/size identities;
this is discovery rather than Git-only enumeration. Explicit generated outputs
remain included inside build boundaries and cannot traverse symlinked ancestors.

A declaration N/A needs the independently reviewed exact digest and a recursive
value-only syntax proof, including type syntax. Unregistered sources, nonmember
owners, unknown targets, unsupported declarations and malformed/duplicate inputs
fail. The [literal inventory contract](../../tests/architecture/fixtures/source-inventory-v1/README.md)
owns exact bounds and path precedence. A typed invalid-byte Path probe does not
qualify a filesystem that permits such filenames; the actual macOS preparation
failure and future native/filesystem qualification remain distinct.

The native command retains actual Cargo PID/exit/raw-stream observations through
the shared measured observer. Its own source, helper and actual native behavior
need complete direct prepared-tool qualification before dependent use. A snapshot
does not attest before/after freshness, exact external tool/configuration/feature
identity, executed test completeness or complete native profiles. The complete
fresh collector must establish those facts, preserve raw regions and reconcile
all profiles before consuming the snapshot and the supplied-report count gate.
Whole automated verification remains unfinished until that full gate is accepted.

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


### Explicit native receipt component

The receipt library and shared process paths implement an explicit measurement
request, separate from the complete fresh collector. Their owning literal
[receipt inputs](../../tests/architecture/fixtures/native-measurement-v1/README.md)
require independently reviewed request validation, concrete executable selection,
exact byte-preserving identity, actual Child completion, exclusive/idempotent
records and bounded owned cleanup. All reusable support remains measured.
This initial qualification profile uses the current non-root Unix/macOS user;
other platform and privilege profiles retain their own consuming gates.

No request means no measurement-file side effects. A caller-owned immutable
context and writer bind one run/source identity. A bare executable request is
resolved against the intended cwd and explicit PATH before constructing the
actual spawn Command. Its selected absolute program prevents kernel fallback to
a different image. Active preparation rejects any still-bare program; preparation
and spawn use the same unchanged Command, without custom argv[0]. Local paths
are byte arrays in local diagnostic evidence, not publication-safe evidence.

Completion comes from that same actual Child, including signal and cached status.
The deadline precedes status acceptance and is checked after receipt completion.
Native fixture and metadata observation retain a 30-second collection budget and
separate two-second owned cleanup. Delegated format/check/test/Clippy use reviewed
60/180/300/180-second budgets, allowing the observed complete workload. A timeout
fails. Larger future workspaces must qualify their actual commands and budgets.

### Retained native launch-failure diagnostics

The collector retains a bounded immutable diagnostic when native preparation,
spawn or launch-receipt recording fails. The independently reviewed
[diagnostic inputs](../../tests/architecture/fixtures/native-launch-diagnostics-v1/README.md)
own the initial non-root Unix/macOS fault profile. Existing launch and completion
formats, finite command stderr, deadlines and exclusive receipt semantics remain.

Retain the actual failing operation, original I/O error kind and raw OS error
when supplied, the actual child PID only after a successful spawn, the selected
receipt path when known, and the actual bounded terminate/reap result. Distinguish
context selection, preparation, OS spawn, context revalidation, receipt creation,
write and flush. Preserve a primary receipt error separately from any cleanup
error. Unknown PID, errno or status stays null in its required field; failure records cannot create
launch/completion pairs or attest successful execution. A receipt collision fails
without overwriting, adopting or deleting the existing record. Do not retry the
command automatically.

The diagnostic binds the exact retained command-intent bytes and selected
run/context/source identity when available. Paths remain byte arrays in local
evidence. Bound each serialized diagnostic to 64 KiB. Create its artifact
exclusively. If artifact creation, writing or flushing fails, preserve the primary
failure and actual secondary retention error in the already owned trace where
writable. Trace failures propagate; unwritable evidence never becomes success.
Do not invent missing evidence after a failure. An artifact or trace write result
does not replace an actual Child status.

The literal package freezes complete full and fallback envelopes, exact keysets,
types, nullability, stage vocabulary and external binding rules. Compare entire
objects after replacing only its declared dynamic tokens with independently
observed command bytes/digest, original context bytes/digest, canonical paths,
actual parent and Child identities, and fixed source/run identities. An omitted
field, extra field or invented dynamic value fails. Runtime scope is
`actual_runtime`. Controlled writer/real-cleanup and representation units have
separate explicit scopes and cannot qualify unobserved OS or Child behavior.

The 65,536-byte inclusive limit counts compact UTF-8 JSON, lexicographically
ordered object keys, and exactly one terminal LF. Accept the exact limit. Reject
one byte over before creating an artifact. On size or artifact-retention failure,
write the frozen compact fallback to the owned trace. It preserves every finite
primary and cleanup cause, actual identity or null, exact command-intent digest,
original serialized diagnostic byte count and SHA-256, and secondary retention
stage/kind/raw error. Replace long paths only with SHA-256 of their original raw
path bytes in this fallback; do not silently truncate or change the primary
cause. Existing or partially written artifacts remain evidence. Fallback is
subject to the same bound, and fallback trace failure is failure.

Controlled record-writer cases must use the production phase/error mapping and
actual owned-child cleanup, so record-write and record-flush failures remain
distinct. Separate error-composition units discriminate simultaneous primary and
cleanup errors and null status, while preserving the existing legacy error
return convention. Such units do not invent an unreapable or OS-denied child.

Qualification exercises genuine preparation, spawn, receipt collision and write
denial, actual owned cleanup, exclusive diagnostic creation and trace fallback.
Controlled writer faults qualify only the writer boundary, not a fabricated OS,
hardware or process failure. Reconcile every existing case and the complete fresh
owning and standalone source, coverage, process/profile and native gates before
accepting repaired source. A later green rerun alone does not resolve an unknown
earlier failure.

Receipt success does not attest native profile existence, complete execution,
actual Cargo or the whole verification tool. The collector must still bind exact
prepared tools/configuration/features, collect all test/process/profile identities,
compare complete inputs before/after, merge every fresh raw profile, retain full
raw regions and apply the accepted inventory/count gates. It must meaningfully
reject lost, stale, empty, mixed-context and incomplete profiles before acceptance.

### Maintained native collector contract

This section defines the next collector implementation; it is not an acceptance
claim. The initial profile remains Rust 1.99.0, stable cargo-llvm-cov 0.9.1,
macOS ARM64, the current non-root user and default workspace features. Actual
owning and public-only standalone qualification must precede whole verification
acceptance. Existing coverage scopes and denominators remain required.

The dev and test profiles optimize only the pinned external SHA-256 dependency
at level3 so complete toolchain hashing fits the input budgets. Maintained source
keeps its ordinary debug profile and every strict coverage gate. This profile
change requires fresh complete qualification; it does not reduce input discovery,
hashing, cases, counters or deadlines.

Preparation is explicit. A local prepared-tool manifest binds invoked program
spelling, canonical executable bytes, selected Rust sysroot runtime resources,
Cargo home, all discovered Cargo configurations and the effective tool/feature/
target policy. Its resource inventory covers sysroot bin/lib/libexec, including
codegen, standard libraries, LLVM and instrumentation support. Source-role, test
and prepared manifests are separately bound even when external to the workspace.
Compare complete source, resource, configuration and tool identities before and
after; additions, removals and renames invalidate the run. Never install tools or
fetch dependencies during preparation or verification. Require actual selected
Cargo/rustc 1.99.0, cargo-llvm-cov 0.9.1 and matching selected LLVM versions.

The output directory must be fresh, external to the canonical source workspace
and explicitly selected. Retain all failed output. Create an immutable context
whose run/source identity binds complete inputs and the prepared/native profile.
Capture actual locked/offline Cargo metadata and instrumented --no-run JSON
artifacts. Distinguish test harnesses from ordinary native binaries: only harnesses
receive libtest list/run arguments. Reconcile actual artifacts, packages, targets,
features and compiled source identity with a reviewed registry. Unknown selection
or changed ownership fails. Required suites are nonempty. Explicitly incidental
empty harnesses are listed and executed, retain valid profiles, and never count
as required-suite passes. Separately execute actual Cargo doctests; the initial
registered zero-example profile cannot admit newly discovered examples.

Use stable libtest --show-output --test-threads=1 and retain complete transcripts.
Freeze exact selected/executed names, complete success summaries and each test's
literal event IDs. Recognize events only as complete lines in that successful
test's captured stdout section, with its actual harness PID. Nested JSON/trace
text is not an event. Child diagnostics can interrupt a pending test's result
line; they cannot supply another selected name, success summary or case event.
Duplicate, unknown, missing, truncated, failed, ignored and filtered results fail.
The registry and transcript grammar need independent literals before consumption.

The tooling assertions occupy four ordinary, same-level Cargo integration-test
targets: `contract` for architecture and inventory, `collector_contract` for
collector intake and miniature workspaces, `native_contract` for native execution
and diagnostics, and `profile_contract` for profile and compilation conservation.
Each is a separately registered required harness and runs its complete unfiltered
test list with `--show-output --test-threads=1 --color=never`, under the unchanged
300-second required-suite budget. Moving a test preserves its body, outer
attributes, event prefix and ordered literal case IDs exactly once. A target
split requires independent admission of that disjoint allocation before its
consumer changes. It cannot admit filtered batches, retries or larger deadlines.
Each target exposes the same canonical test-only support module; all maintained
helpers remain measured, including every actual compiled module instantiation.
Physical source, Cargo artifact, object, process, profile and coverage inventories
must reconcile afresh. Owning and public-only standalone qualification remain
required; a previously failed aggregate run remains failed.

Every exercised instrumented object needs its actual launch, completion and one
fresh nonempty profile per PID in the initial profile. Every raw profile needs a
matching object and process under the same immutable context. Repeated observation
is one invocation. Retain valid listing and incidental-empty profiles; required
suites and ordinary native binaries also need positive counters. Prepared external
tools retain actual launch/status and executable identities without a requirement
to emit this project's Rust profiles. Inspect and conserve the exact raw set and
its unchanged bytes, merge every profile and export every owned object with full
raw regions/functions. Lost, duplicate, stale, empty, mixed or unknown sets fail.

A required harness may explicitly register at most 16 unique descendant probe
test names in `descendant_probes`. Names must select existing tests, contain no
NUL and occupy 1 through 4096 UTF-8 bytes. Each declaration requires exactly one
same-object direct child of that harness's selected top-level run, with exact
`--exact TEST --nocapture --test-threads=1` arguments, successful completion,
positive counters and its own fully conserved fresh profile. The original
context, executable, parent-chain and raw-set checks remain mandatory. Incidental
harnesses and ordinary binaries cannot declare probes. Unregistered, duplicate
or missing children fail. Probes add no selected command, suite, test or case-event
credit and cannot replace the complete unfiltered top-level suite run. The
[descendant probe literals](../../tests/architecture/fixtures/native-collector-v9/README.md)
define the independent boundary expectations; actual parent tests retain the
probe's behavior and output assertions.

Use a bounded merge fan-in of eight inputs per invocation, retaining every
intermediate indexed profile and the complete actual merge command graph. Repeat
grouping until the final invocation has at most eight inputs. Every selected raw
profile must reach the final aggregate exactly once. This preserves the 64 KiB
receipt limit and the shared merge-phase deadline as the raw set grows. Compare
the actual full export against a direct selected-tool merge at the eight/nine
boundary; supplied counters cannot qualify that comparison.

In addition to all existing aggregate/source/package/library/typed-target gates,
gate each actual binary crate's complete owned source/module regions independently
with only that binary's PID profiles and exact object. Establish the closure from
actual Cargo compilation dependency information and compare it with the reviewed
registry. Include transitive modules, generated executable source and code compiled
directly into multiple binaries in every applicable closure. Shared-library code
retains its complete independent file/crate/library denominator. Current thin
entrypoints do not create an entrypoint-only exemption for later modules. Unknown
or changed closure fails until reviewed. Earlier aggregate results do not establish
these additional per-binary measurements.

Nesting requires an explicit prepared-manifest declaration of outer context A.
Validate this collector's actual launch identity under A; its own final flushed
profile and exit remain A's parent's responsibility. Create distinct context B
and raw directory for the workspace being collected. Every B child uses the same
captured B Request for launch and completion, including errors and cleanup; its
per-Command environment selects B. Preserve ambient A in the collector process.
Never reload A to complete B children, mutate global environment, or combine or
remove the two raw sets. Without explicit nesting, inherited measurement and
instrumentation overrides fail. A running collector cannot attest its own final
exit/profile; actual outer owning/standalone qualification closes that evidence.

Nested launch-receipt publication uses the existing writer's single UTF-8 JSON
object followed by LF. LF marks a complete publication candidate, never validity,
successful flush, durability or operating-system attestation. An absent, empty
or unfinished own-PID receipt may be pending for at most 1000 milliseconds. The
effective endpoint is fixed once at entry to outer admission as the earlier of
that endpoint and the original passed caller deadline. Waiting, bounded reads,
identity hashes, context rechecks and any observation callback all count inside
that endpoint; no successful late frame, deadline renewal or retry into a
different completed frame is admitted.

Keep at most one selected regular receipt descriptor and at most 64 KiB of
complete receipt bytes. Before selection, absence alone may be pending. Reject
symlinks, special files, unreadable files and more than 64 KiB immediately.
After selection, retain the same descriptor and path identity while waiting,
reading and validating; unlink, replacement or inconsistent complete-file
observations fail. Use the final LF and strict single-value JSON from those same
bounded bytes. A complete malformed or identity-conflicting frame fails
immediately; later replacement bytes cannot repair it. Snapshot the exact context
bytes before waiting and require them unchanged afterward. Preserve every
existing actual PID, context/run/source, executable/hash, program/argv and cwd
check before any nested work.

An optional explicit descriptor-selection observer receives no descriptor,
parsed data, admission result or deadline. Invoke it at most once after actual
regular-file selection, with the endpoint already fixed. Its errors and elapsed
time fail admission; it cannot supply or authorize a receipt. The ordinary route
uses a no-op observer. Native controlled-input cases may use an acknowledgement
from this seam to mutate only an owned test receipt after actual selection.
Preserve original parent-created receipt bytes, actual child status, complete
raw evidence, unchanged ambient context and the existing full qualification
requirements. These controlled mutations do not attest a historical race or
replace the unfiltered owning/public qualification.

Use phase-specific budgets: metadata/version/instrumentation/list/profile inspection
30 seconds; format 60; check/Clippy 180; instrumented build/required suite 300;
incidental empty run 30; doctest/merge/export 180. Input/resource/finalization
work has a 180-second budget. Receipt, hash and output collection count inside
their phase. Retain the shared observer's 8 MiB per stream and separate two-second
owned-child cleanup. Runtime resources allow at most 16384 files, 1 GiB per file
and 4 GiB total; raw profiles allow 4096 files, 64 MiB each and 4 GiB total;
collector receipts allow 64 KiB each. Existing source and JSON bounds remain.
Over-limit, truncated or late evidence fails. Success follows all final identity,
selection, process/profile and arithmetic gates; final output failure is failure.

The actual native integration uses the same collector executable on a literal
standalone three-package workspace whose graph has no collector integration test.
It performs real Cargo/Rust/LLVM work and retains artifacts, receipts and profiles.
An intentional task-owned source-mutation variant must fail final input equality.
No flag skips outer required tests to prevent recursion. This proves the collector
protocol; actual complete owning/standalone self/source/native qualification and
independent whole-checkpoint acceptance remain separate mandatory gates.

The maintained collector invokes the selected cargo-llvm-cov program directly for
show-env. Its default profile stem is the actual workspace directory basename.
It verifies selected rustc --print sysroot and the prepared Rust/LLVM tool locations
against the resource tree actually inventoried. Raw profile content is matched to
the selected object's complete static instrumentation data, encoded names and
counter layout for the admitted ARM64 MachO/LLVM23 rawv11 profile; a PID or filename
alone does not establish object identity. The complete LLVM parser remains required.
All actual compiled object bytes are reopened and compared during finalization.
Dependency files use bounded regular-file intake. One enclosing deadline applies
to each declared180-second input, profile/binary and finalization phase, including
its nested hashing, parsing, receipts, commands and final result output.

The selected wrapper crate-name set includes normalized workspace member package
names, their _tests aliases and every actual member target name. Compare complete
sets; retained repeated aliases do not authorize unknown or missing names.

A registered ordinary native binary may declare independently reviewed exercises
with exact argument vectors, exit and complete bounded stdout/stderr expectations.
The collector runs those commands directly with B receipts, the30-second observer
and one enclosing180-second native-exercise phase. It adds no libtest flags.
An absent/empty exercise list preserves required-test-driven native collection.
The additional literal three-package mini has a transitive binary module and real
native exercise, plus output-mismatch and omitted-module failures. It preserves
all original tests/scopes and proves actual compiled closure rather than an
entrypoint-only native denominator. Task-owned conflicting plugin-discovery input
is never admitted as a selected tool or copied executable alias.
