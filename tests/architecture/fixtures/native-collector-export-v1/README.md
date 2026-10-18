# Complete bounded LLVM coverage exports

These original public synthetic literals use MIT OR Apache-2.0. They define
complete export assembly before its consumer. They contain no customer records,
secrets, operational credentials or external provider use. Supplied values
qualify protocol boundaries; actual selected-tool and whole native qualification
remain separate.

The owning [testing contract](../../../../docs/requirements/testing-and-coverage.md)
still requires every raw region/function, exact source census and strict coverage
at every applicable scope. Its unchanged stream, format, phase, resource, receipt
and cleanup limits remain mandatory. This is a bounded implementation of that
contract, without a larger output class or a summary-only replacement.

For each existing export select exactly two ordered actual captures from the
SAME selected LLVM-cov executable, ordered complete object vector, indexed
profile and common option vector. Both use `export -format=text`, the same
`-instr-profile` and every ordered `-object`. The first appends
`-skip-functions`. The second appends `-ignore-filename-regex=.*`.
The tagged [LLVM23.1.1 exporter](https://raw.githubusercontent.com/llvm/llvm-project/llvmorg-23.1.1/llvm/tools/llvm-cov/CoverageExporterJson.cpp)
renders the complete file records and whole totals in the first capture, and
the global complete function vector in the second. Filename filtering does not
partition functions.

Both successful captures and every full stream, launch, actual PID/status,
context and completion remain recorded. Each stream is at most8,388,608 bytes.
Both invocations, input hashing, parsing, assembly and output writing share the
ORIGINAL single180-second export budget, also bounded by its existing enclosing
collection phase. Runtime, receipt and profile storage limits remain unchanged.
There are exactly two captures, at most16,777,216 combined stdout bytes, with
the original64MiB complete-format ceiling. If either required stream cannot fit,
or the shared budget expires, fail honestly. No fallback truncates data, drops
functions, switches to summary-only output or renews a phase budget.

Fingerprint the complete indexed profile and every selected object before the
first capture, between captures and after the second, comparing their complete
hash/size bindings with the same ordered selection. Each tool launch retains
the existing selected-executable hash check. Preserve all raw profiles, static
modules, complete FuncHash/counter vectors, selected source and final compiled
object comparisons. Changed inputs or tools, unknown selection or any failed,
late, missing, duplicate, extra or conflicting part fails.

Assembly strictly parses exactly two UTF-8 JSON documents without duplicate keys,
trailing data or nonfinite numbers. Root keys are exactly `data`, `type` and
`version`; type is `llvm.coverage.json.export`, version is exactly `3.1.0`,
and each data array contains exactly one object. The files data object has
exactly `files` and `totals`; the functions object has exactly `files`,
`functions` and `totals`. Both files vectors are arrays. Every first-part
file is an object with a distinct nonempty filename. The second files vector
is empty. The functions vector is an array, including legitimate repeated
function records; no name-based deduplication or sorting is permitted.

The functions part has exactly the complete six zero-file metric objects frozen
as `zero_file_totals`. Every numeric value is zero; integer or floating zero
is allowed. Unknown or missing metric keys fail. The first totals value must
be an object and is preserved ENTIRELY, never replaced by zero-file totals or
summed from partitions.

The result copies the complete files document and inserts the ENTIRE second
function vector into its single data object. Preserve every nested file,
segment, region, branch, expansion, MC-DC, summary, filename, function, count
and total value, including array order and duplicate function multiplicity.
Object key order is immaterial; numeric and string values must not change.
The original complete-format bound and downstream exact source/coverage/native
gates still apply. Assembly is not independent verification of supplied counts,
native execution or coverage.

`cases.json` contains 57 complete positive/negative assembly cases. Each
`case.input.parts` is an ordered byte-recipe vector. An element's `utf8`
string supplies its bytes; optional `pad_to_bytes` appends ASCII spaces to
that exact total length, and optional `suffix_bytes` appends the literal byte
array. These finite recipes exercise the actual unchanged8MiB boundary without
committing repeated padding. The helper receives only input; `expected` or
strict-parsed `expected_utf8` stays in thin assertions. The latter preserves
the exact unsigned64-bit count and floating percentage literals without a
fixture loader rounding them through another numeric representation.

Before consumers, independently compare ENTIRE assembled selected-LLVM output
with a genuine complete direct export on identical selected objects/profile:
all nested values, types, array order/multiplicity and whole totals must agree.
A retained rejected over-limit reference may establish that semantic comparison
only; it never establishes a successful collector, coverage or full run.
Later whole owning and public-only standalone collections must independently
qualify current consumer bytes, every existing unfiltered test, all strict
scopes, conservation, current model/probes, retention and owned cleanup.
