# Local architecture and verification commands

The first implementation delegates architecture, format, check, test and Clippy
to the current workspace. It is a repository tool, without product authority.
Its core source and process qualification must pass before dependent use. Whole
verification and automated coverage collection remain the next increment;
`verify` and `coverage` return usage failure today.
The separate report-count command below evaluates supplied reports; it does not
replace those unfinished collection commands.

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
Help goes to stdout; usage goes to stderr with exit 2. Missing commands and
owned output write/flush failures exit 1. Architecture consumes Cargo metadata
stdout, preserves Cargo diagnostics on stderr, checks the actual resolved graph
and every current production `src` file, reconciles actual Cargo target paths
with their owning sources, then emits its own success line. A
guard failure is a fixed stderr line with exit 1 and no success output.

The initial role registry recognizes only the current proof, domain and tool
packages at version 0.0.0. The domain remains standard-library-only. Pure
application and deterministic workflow profiles are tested explicitly, including
SDK 1.0.0 rather than a blanket SDK prohibition. New packages, pure dependencies
and supported SDK surfaces require their owning profile and first-consumer
qualification; unknown entries cannot become approved by omission. Controlled
dev dependencies grant no production import authority. Configured syntax/import
guards complement independent code review and actual compiler checks; they do
not certify arbitrary data flow or an actual SDK/runtime.

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
those facts nor completes the automated collector or whole verification.
