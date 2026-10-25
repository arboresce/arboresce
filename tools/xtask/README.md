# Local architecture and verification commands

The first implementation delegates architecture, format, check, test and Clippy
to the current workspace. It is a repository tool, without product authority.
Its core source and process qualification must pass before dependent use. The
separate prepared native collector is described below. The original dispatcher
still returns usage failure for the `verify` and `coverage` aliases.
The separate report-count command below evaluates supplied reports; it does not
replace actual native collection.
The source-inventory command observes actual files and Cargo targets. Its own
source/process qualification precedes dependent use; the complete collector
applies its inventory and coverage gates to a fresh observed run.

Native launch failures retain a separate exclusive `launch-failure.json` and
trace record, bound to the exact saved command intent and original selected
measurement request. Records keep the actual stage, I/O kind and OS error,
known child identity and separate cleanup result. Existing receipts and partial
diagnostics remain evidence. The full diagnostic has an inclusive 64 KiB limit;
a bounded fallback retains finite causes and hashes of omitted paths and the
complete serialized record. A trace write or flush failure propagates failure.
The [literal diagnostic fixtures](../../tests/architecture/fixtures/native-launch-diagnostics-v1/README.md)
distinguish actual process observations from controlled writer and representation
units. This change requires fresh owning and standalone qualification; it does
not establish the cause of an earlier unrecorded failure or permit retry credit.
The fresh-artifact diagnostic input also governs an actual failing collector
invocation using real prepared tools and the existing finite command-error contract.

Prepare Rust 1.99.0 and the locked public dependencies, then run from the root:

```sh
cargo run --locked --offline -p arboresce-xtask -- architecture
cargo run --locked --offline -p arboresce-xtask -- fmt
cargo run --locked --offline -p arboresce-xtask -- check
cargo run --locked --offline -p arboresce-xtask -- test
cargo run --locked --offline -p arboresce-xtask -- clippy
```

Apply the configured build-output router and its diagnostic where present.
Formatting checks never rewrite sources. Build, test and lint use locked offline
workspace commands; Cargo streams and nonzero exits pass through unchanged.
Delegated format/check/test/Clippy have explicit 60/180/300/180-second operation
budgets. A timeout fails and performs bounded cleanup of the owned child.
These budgets are qualified against actual workspace work; native fixture and
metadata observation retain their separate 30-second budget.
Help goes to stdout; usage goes to stderr with exit 2. Missing commands and
owned output write/flush failures exit 1. Architecture consumes Cargo metadata
stdout, preserves Cargo diagnostics on stderr, checks the actual resolved graph
and every current production `src` file, reconciles actual Cargo target paths
with their owning sources, then emits its own success line. A
guard failure is a fixed stderr line with exit 1 and no success output.

The Initial profile retains the original proof, domain and tool package roles
at version 0.0.0. Its closed Initial profile retains the original
standard-library-only domain examples. Pure
application and deterministic workflow profiles are tested explicitly, including
SDK 1.0.0 rather than a blanket SDK prohibition. New packages, pure dependencies
and supported SDK surfaces require their owning profile and first-consumer
qualification; unknown entries cannot become approved by omission. Controlled
dev dependencies grant no production import authority. Configured syntax/import
guards complement independent code review and actual compiler checks; they do
not certify arbitrary data flow or an actual SDK/runtime.

The explicit package profile `jcs-number-v1` retains the first pure scalar-number
dependency graph. The actual workspace
checker dispatches that profile and binds exact normal roots, aliases, kinds,
conditions, complete transitive normal/build graph and effective features.
The [selected inputs](../../tests/architecture/fixtures/qualified-number-dependencies-v1/README.md)
retain separate resolvable policy contexts. Missing metadata uses the unchanged
Initial profile; unknown/wrong-type selections or selection by another owner fail.
The graph fingerprint is a fixed internal tool encoding, not domain JCS. Changes
require a new reviewed profile and fresh affected qualification. Controlled dev
edges grant no production authority; source checks remain required.

The separately retained `architecture-profile = "jcs-source-v1"` qualified the
bounded source parser and private JCS adapter before shared PostgreSQL features. It adds only the already locked exact
serde/serde_json direct roots and binds the separately frozen complete graph.
Its [source-profile fixtures](../../tests/architecture/fixtures/qualified-source-dependencies-v1/README.md)
retain all earlier graph/policy meanings and test the selected in-memory writer
type exception. Only Write/Result/ErrorKind and FileTooLarge are selected; Error
and its ambient last_os_error are closed. Macro imports are conservatively
rejected in this profile. Actual targets and recursive/unimported source receive
the same selected policy. Public check_source, Initial and number source guards
remain unchanged; this does not certify arbitrary data flow or macro expansion.

The PostgreSQL adapter workspace selects the separate exact
`jcs-source-postgres-v1` domain profile. Its
[literal graph and source policy](../../tests/architecture/fixtures/qualified-source-postgres-dependencies-v1/README.md)
bind the reviewed shared features and derive edge while preserving every older
profile fingerprint and source meaning. Current compiled/native/resource and
strict qualification remains separate from admitting those graph intentions.

The original application `continuity-wire-v1` profile covers its fixed journal decoder.
Its [literal dependency profile](../../tests/architecture/fixtures/qualified-continuity-wire-dependencies-v1/README.md)
admits exactly the owned domain, serde 1.0.229 and serde_json 1.0.151 normal roots
and their complete 24-row closure. All 23 previously qualified external rows,
features, archives and rights remain unchanged. The domain retains its existing
profile; Initial behavior and every older fingerprint are preserved. The
application retains its pure source policy and receives no domain-only writer
exception. Missing/unknown selection, wrong ownership and any graph drift fail.
Actual Cargo, full source/helper/process/native/strict and standalone evidence
remain required before accepting the consumer.

The selected application `continuity-recognition-v1` profile adds only the
already qualified sha2 0.10.9 direct root with defaults disabled and force-soft.
Its [separate closed graph](../../tests/architecture/fixtures/qualified-continuity-recognition-dependencies-v1/README.md)
retains all 24 transitive rows, 23 external archive/rights/feature facts and
the application's pure source policy. Its 46 graph and nine source-policy
contexts preserve the older three-root profile and every older fingerprint.
Actual fresh source/helper/process/native/strict and standalone qualification
remains separate from these dependency declarations.

For actual migration-service qualification, use the explicit service variant:

```sh
cargo run --locked --offline -p arboresce-xtask --bin arboresce-verify -- \
  postgres /source/root /source/root/tests/architecture/rust-source-roles-v1.json \
  /source/root/tests/architecture/rust-test-suites-v1.json /external/prepared.json \
  /external/postgres-input.json /external/fresh-output
```

Apply the output router. The
[service input and relay contract](../../tests/integration/postgres/fixtures/migration-service-v1/README.md)
fixes the separate native-file bounds and selected local namespace. Actual
readiness and distinct delivery admission precede use. The collector reopens and
hashes this input, accepted readiness and every selected native/configuration
file before and after execution and includes them in run identity. It clears
ambient service/PG selectors before injecting the explicit input. Required
service tests fail when input is absent. Prepared pg_ctl/postgres process
identities are separately recognized external tools, never Rust coverage objects;
their launch/exit receipts and indirect daemon observations remain required.
Each public-only standalone qualification receives a fresh isolated service.

The PostgreSQL input position also accepts the separately versioned
[two-cohort manifest](../../tests/architecture/fixtures/native-collector-cohorts-v1/README.md).
It binds two complete independently prepared service descriptors. The complete
`arboresce-postgres/test/authority_current` harness selects `current`; the legacy
race and migration harnesses select `legacy`. List and whole execution share
that selection. Both immutable vectors, physical ownership and exact allocation
enter the run identity and final unchanged-input check. Native tool bindings
match across cohorts; each service retains its original lease, one bounded
restart, separate owned cleanup and every existing per-service bound. Each full
owning or public-only standalone run needs its own fresh pair. Protocol fixture
tests alone establish no actual readiness, persistence or complete qualification.
The original collector preparation and help/usage meanings remain unchanged;
`arboresce-verify postgres --help` describes the new variant.

The [optional-service protocol fixtures](../../tests/architecture/fixtures/native-collector-v8/README.md)
extend the finite actual Cargo/LLVM miniature experiments. They retain the
original successful result while checking explicit selection and immutable
service inputs, and reject an actual bound-configuration change during a required
test. Separate literal resolver cases preserve ordered object, prepared-tool and
service-tool hash lookup followed by exact executable-byte matching. These
filesystem models never execute their synthetic tool files or qualify PostgreSQL;
the real-service, complete native/profile and strict coverage gates still apply.

The [registered descendant probe fixtures](../../tests/architecture/fixtures/native-collector-v9/README.md)
cover explicit same-harness helper execution. A required harness may register a
selected test as one measured child of its top-level run. Exact arguments,
successful completion, positive counters and the child's own fresh profile are
required. The PostgreSQL selector-denial probe uses this declaration without
adding a selected suite, command or test count. Complete owning and standalone
qualification must exercise the actual child and conserve its profile.

For selected domain frame inspection, retain compiler assembly with the exact
toolchain and profile. This is evidence for the source processing stack model,
alongside actual maximum native inputs; it does not replace strict collection:

```sh
cargo rustc --locked --offline -p arboresce-domain --lib -- --emit=asm
RUSTFLAGS='-C instrument-coverage --cfg=coverage' cargo rustc --locked --offline -p arboresce-domain --lib -- --emit=asm
```

Apply the output router as above. Compiler assembly, raw runs and measurements
belong outside the source tree. Inspect actual generated frames and called
dependencies before claiming a stack bound.

The independently approved [literals](../../tests/architecture/fixtures/guard-v1/README.md)
freeze policy, command vectors and streams. Native controlled Cargo peers prove
delegation/error boundaries; actual Cargo commands need separate observed runs.
The additional [regressions](../../tests/architecture/fixtures/guard-v2/README.md)
cover equivalent raw/aliased/UFCS spellings, platform I/O, declared source
inclusion/targets and late native observation. The initial profile rejects
module path attributes and source include! until their first consumer qualifies
the complete inventory; ordinary modules and text/byte resources remain allowed.
The [grouped self cases](../../tests/architecture/fixtures/guard-v6/README.md)
preserve namespace identity. The
[aliased inclusion cases](../../tests/architecture/fixtures/guard-v7/README.md)
fix the conservative terminal include import restriction for any root, including
effectful layers, while preserving text and byte resource aliases.
The test peer is maintained test support, never a product command or a substitute
for Cargo qualification. All probes, including broken output streams, use one
30-second process/collection budget and bounded task-owned file capture. Late
exit or collection fails. Timeout cleanup kills/reaps only the launched child
with a separate two-second cleanup budget; there are no pipe-reader joins. Its source, builders and observer are measured under
the [coverage contract](../../docs/requirements/testing-and-coverage.md).

## Supplied report counts

The report-count library and separate native command evaluate an explicit
supplied source-role inventory against a selected LLVM 3.1.0 export:

```sh
cargo run --locked --offline -p arboresce-xtask --bin arboresce-report-gate -- \
  /original/source/root /external/roles.json /external/llvm-export.json
```

The first argument is the report's original logical source prefix; evaluating
an archived report does not require that directory to exist. The inventory uses
version rust-report-roles-v1, a sources array with path/package/role entries,
and a targets array with kind/name/sources entries. Paths are relative to that
prefix. Roles are measured or assertion; target kinds are library or binary.
See the independently approved [literal input contract](../../tests/architecture/fixtures/report-gate-v1/README.md)
and its complete base example. Every selected file must occur exactly once.
Every package has measured source, and each target names measured source.
Directory names confer no exemption. Source, package, repository, library and
binary scopes use separate prefixes. All applicable source counts must exceed
90% independently using exact integer arithmetic. Raw totals and region
remainders must agree; duplicates, malformed counts and overflow fail.

Both UTF-8 JSON inputs are bounded to 67108864 bytes. Duplicate object keys and
trailing JSON fail. Native help uses stdout/0, usage errors stderr/2, and owned
input/gate failures fixed stderr/1. Success emits coverage report counts passed
with LF on stdout/0. Output write/flush failure exits 1. The command echoes no
input paths or contents. Actual native cases use the measured bounded observer;
input preparation is measured support, separate from assertion bodies.

This is a supplied-count result. The caller must separately establish actual
inventory, declaration-only N/A, exact source/tool/run identity, fresh complete
native process profiles and required test execution. Full raw region/function
data remains with the original report. The summary adapter neither attests
those facts nor establishes whole verification.

## Complete native coverage exports

The collector retains two complete complementary LLVM captures for each export.
Both use the same selected tool, complete ordered object set and indexed profile.
The first adds `-skip-functions`; the second adds `-ignore-filename-regex=.*`.
Complete files and whole totals come from the first capture. The entire global
function vector comes from the second, preserving every nested value, array
order and repeated function record. The
[independent export literals](../../tests/architecture/fixtures/native-collector-export-v1/README.md)
define exact assembly and rejection boundaries before this consumer.
The [lossless numeric literals](../../tests/architecture/fixtures/native-collector-export-literals-v1/README.md)
also compare complete raw UTF-8 renderings. Strict parsing validates the documents;
assembly copies the first document and the entire raw functions array without
reconstructing numeric tokens through binary64. Escaped keys, strings and nested
values are located with the existing JSON deserializer.
The [exact-zero literals](../../tests/architecture/fixtures/native-collector-export-zero-literals-v1/README.md)
require every second zero-file metric to have a genuinely zero decimal
coefficient. Numeric underflow to binary64 zero does not satisfy that rule.
Signed, fractional and exponent forms of zero remain valid. Tiny nonzero values
in first-part file data or second-part functions retain their original bytes.

Each stream retains its existing 8 MiB ceiling. Both actual invocations, input
hashing, parsing, assembly and writing share one original 180-second export
budget, also limited by the enclosing collection phase. All profile and object
hash/size bindings are retained before, between and after captures. Each launch
retains selected-tool verification. Either over-limit or late capture fails;
there is no omitted-function, summary-only or larger-stream fallback. The full
format ceiling, raw/static-module/counter/source/native/strict and final-object
checks remain unchanged. Assembly alone supplies no coverage qualification.

## Actual source inventory

The inventory library observes the filesystem and reconciles explicitly supplied
Cargo metadata. The separate native command obtains actual offline Cargo metadata
and applies the architecture guard before producing that inventory:

```sh
cargo run --locked --offline -p arboresce-xtask --bin arboresce-source-inventory -- \
  /source/root /source/root/tests/architecture/rust-source-roles-v1.json
```

Apply the configured output router. Success writes one JSON record and LF with
exit 0. Help uses stdout/0 and usage stderr/2; intake, discovery, metadata, guard
and output failures exit 1. Actual Cargo stderr is preserved. Metadata capture
reuses the measured bounded process observer: a fresh owned scratch directory,
one 30-second observation/collection budget, 8 MiB per captured stream and bounded
two-second owned-child cleanup. Its actual exit, PID and raw streams remain in
the record's `metadata_trace`. The command does not install tools or fetch crates.
The trace is local diagnostic evidence and can contain local Cargo paths.

The independently reviewed [inventory inputs](../../tests/architecture/fixtures/source-inventory-v1/README.md)
own path boundaries, exact file/count/depth/total limits, workspace-member
ownership, typed targets, generated outputs and declaration proof. Discovery
uses actual files rather than Git's index and retains exact SHA-256/size for every
included input. The versioned [roles](../../tests/architecture/rust-source-roles-v1.json)
register maintained support as measured source; assertion bodies remain separate.
Every discovered Rust file and actual Cargo target entrypoint has exactly one
role. Unknown or missing files/targets fail. Standard runtime paths cannot omit
maintained source trees, and explicit generated paths are reopened without
symlinked ancestors even inside a build boundary.

The selected `rust-source-roles-v2` input profile admits at most 8192 included
files. The original v1 profile retains its 4096-file ceiling. Both retain the
same per-file, complete-byte, depth, source-role, declaration and target bounds;
the [independent inventory literals](../../tests/architecture/fixtures/source-inventory-v1/README.md)
preserve the old answers and define the additional complete-input boundaries.
The inventory output schema remains `rust-source-inventory-v1` and binds the
exact selected profile hash. A larger profile omits no ordinary input and grants
no source or coverage exemption.

Declaration-only N/A needs both its independently reviewed exact digest and a
bounded value-only syntax proof, including nested type syntax. The record also
contains `coverage_roles`, the measured/assertion projection accepted by the
report-count gate. Actual Cargo library names, including underscores, identify
its library scopes. No zero metric is converted to fictional 100%.

This is a point-in-time inventory. The complete collector separately binds actual
toolchain/instrumenter/configuration/features, compares complete before/after
inputs, reconciles all test identities and fresh native profiles, exports raw
regions and invokes the count gate. The inventory command alone does not attest
run freshness, complete process collection or whole verification.


## Explicit native launch receipts

The measurement library records owned native launches and actual completion only
when the caller explicitly selects `ARBORESCE_NATIVE_CONTEXT`. Ordinary operation
without that request creates no measurement files. This component alone does
not attest a complete set of profiles.

The caller creates an external context directory with regular `context.json`,
`launches` and `exits` entries. The independently reviewed
[receipt inputs](../../tests/architecture/fixtures/native-measurement-v1/README.md)
own its bounded versioned schema, exact identities, preparation rules and literal
failure expectations. The initial profile is the current non-root Unix/macOS
qualification. The context and immutable receipt writers are caller-owned;
these records are local evidence, without an operating-system attestation claim.
They contain byte-preserving local paths and must not be published as-is.

Select a bare program to a concrete absolute path before constructing its active
spawn Command. Retain the selected symlink spelling for tools with basename
dispatch. The selector observes effective cwd and explicit command PATH, including
removal; there is no fallback PATH. Preparation rejects an unbound bare program.
It binds the canonical executable digest, actual Command program spelling,
positional argument bytes and actual cwd. Preparation and spawn must use the
same unchanged Command, without a custom argv[0] override. A selected inaccessible
image fails at spawn; exec search cannot silently run a different image.

Each successful launch exclusively writes its actual child/parent PID and
context/run/source identity. Completion derives exit and signal from that same
actual Child, binds the launch digest and exclusively writes a matching receipt.
An identical repeated completion preserves the existing bytes; a conflicting or
symlinked receipt fails. Failed launch recording performs bounded kill/reap of
only that owned child. Shared observation attempts completion recording after
stream, trace and timeout cleanup errors as well as successful collection.

Nested collector admission waits for the existing writer's final LF before
parsing its own launch receipt. It fixes the earlier of the original caller
deadline and one second at entry, retains the same regular descriptor through
the bounded read and identity checks, and rechecks the selected path and exact
context bytes before accepting. Missing or unfinished publication cannot renew
the deadline; malformed or conflicting complete frames refuse immediately.
LF does not attest validity, successful flush or durability. An explicit
descriptor-selection observer can acknowledge selection for controlled native
tests; its result supplies no admission authority and its work counts inside
the same endpoint. The independently admitted
[publication boundaries](../../tests/architecture/fixtures/native-launch-publication-v1/README.md)
cover all 37 outcomes, both deadline limits and selected-path substitutions.

Complete observation records stream borrowed byte slices through an 8 KiB
buffer in the existing sorted-field JSON representation. The writer explicitly
flushes inside the observation deadline and discards pending bytes on every
error before owned cleanup; buffer destruction cannot retry failed output.
Underlying I/O kinds, numeric byte arrays, full captured streams, the 8 MiB
per-stream limit and all existing deadlines remain unchanged. The
[native trace fixtures](../../tests/architecture/fixtures/native-trace-v1/README.md)
check whole trace bytes and hashes, both full stream boundaries, short writes,
write/flush/work-exhaustion errors and a genuinely late successful flush. They
retain real child streams and physical launch/completion receipts on every
outcome. Passing controlled peer cases alone cannot qualify actual Cargo or the
complete collection lane.

Shared test observation retains complete raw streams and the numeric-byte trace
in a task-owned directory and prints its location. This avoids replaying large
nested traces into the outer harness's bounded output stream. Actual child
output, observation limits, completion receipts and stream assertions are unchanged.

Preparation regressions additionally prove a real launch-directory write denial
before spawning an owned, still-running selected OS child, plus missing receipt
and occupied trace-directory errors after real test-peer completion. Assertions
receive the actual preparation error and retained diagnostic, then inspect real
status, streams and receipts. Each failure independently attempts bounded owned
termination and explicit, fallback and ambient completion; missing explicit
launch evidence cannot become a fabricated pair. Only owned permissions are
restored, and the owned missing directory is retained by rename. Unsupported
write-denial preparation fails. The existing cleanup budget includes launch
recording's own termination. Successful preparation still leaves the explicit
observer completion solely to the tested observer.

All existing owned spawn/status/output paths use this boundary. The controlled
peer exercises real operating-system executables and failure collection; it
cannot qualify actual Cargo. The complete collector selects prepared tools,
freezes complete before/after inputs, reconciles every selected/executed test and
native profile, retains complete raw LLVM data and invokes the count gate. These
gates cannot be inferred from successful receipt unit/native cases.

## Prepared native collection

The separate `arboresce-verify` executable implements the initial prepared
Rust 1.99.0, cargo-llvm-cov 0.9.1, macOS ARM64 default-feature profile. Prepare
the declared tools and resources beforehand, then supply their absolute paths:

```sh
cargo run --locked --offline -p arboresce-xtask --bin arboresce-verify -- \
  prepare /source/root /prepared/rust-sysroot /prepared/cargo-llvm-cov \
  /prepared/cargo-home /evidence/prepared.json
cargo run --locked --offline -p arboresce-xtask --bin arboresce-verify -- \
  /source/root /source/root/tests/architecture/rust-source-roles-v1.json \
  /source/root/tests/architecture/rust-test-suites-v1.json \
  /evidence/prepared.json /evidence/fresh-run
```

Apply the configured build-output router. Evidence paths must be outside the
source inventory, and the run directory must not exist. Preparation creates
the manifest and its retained `.evidence` directory; it installs or fetches
nothing. The collector verifies the selected compiler's actual sysroot, every
declared tool and complete runtime resource tree, ancestor Cargo configuration,
and the immutable source and test manifests before and after collection.

The [test registry](../../tests/architecture/rust-test-suites-v1.json) declares
all current required and incidental harnesses, exact test names and literal case
identities. Actual Cargo metadata and fresh no-run artifacts must match it.
The tool assertions are four ordinary Cargo integration-test targets:
`contract`, `collector_contract`, `native_contract` and `profile_contract`.
Each runs its complete registry without filtering under the same 300-second
suite limit. They share one canonical measured test-support tree; a physical
split preserves every test body, attribute, prefix and case ID exactly once.

Every harness is listed and run directly; incidental empty harnesses remain
evidence and never count as required suites. Optional ordinary-binary exercises
declare exact arguments, exit status and complete UTF-8 streams. Every selected
native PID needs its own matching launch, actual completion and fresh profile.

Complete raw profiles, commands, streams, receipts, inventory, LLVM reports and
scope counts remain in the evidence directory on success or failure. Each
ordinary binary's coverage uses its exact image and own profiles, with the
complete crate-local dependency closure from the matching hashed Cargo image's
Rust dependency file. Shared libraries retain their complete independent scopes.
No module may be omitted by naming only its entrypoint. Source, package,
repository, library and binary coverage must each exceed 90% where required.

Success emits exactly one JSON record and LF with exit 0. Help uses stdout/0,
usage errors stderr/2, and intake, execution, identity, coverage or output failure
stderr/1. Output failure invalidates even a previously written success frame.
The [coverage contract](../../docs/requirements/testing-and-coverage.md) owns
the exact limits and conservation requirements. These are tooling capabilities;
their whole owning and standalone acceptance, and product runtime qualification,
require their separately recorded complete evidence.
